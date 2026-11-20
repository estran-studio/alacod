#!/usr/bin/env python3
"""Trous de la musique dans un enregistrement (M-7b, docs/conventions.md § 124).

Lit le WAV d'un flux de musique (sortie de tools/alsa-pacer : flottant 32 bits, ou PCM 16 bits) et, en option, celui des
effets sonores du même processus, puis :
  - énergie par fenêtre de 10 ms (dBFS) ;
  - début de la musique (première fenêtre au-dessus du seuil) ;
  - trous : au moins --min-gap-ms de fenêtres sous --silence-db après le début (la fin de fichier exclue : --tail-ms) ;
  - début de chaque effet (fenêtre au-dessus du seuil après au moins 100 ms de silence) et trous à moins d'une seconde
    d'un effet.
Chaque WAV du pacer commence à l'ouverture de **son** flux (kira et harmonium n'ouvrent pas au même instant) : les débuts
d'effets sont ramenés dans le repère du WAV musical par les origines `monotonic=` des journaux voisins (`<flux>.log`),
ou par --music-origin / --effects-origin (s, même horloge) quand un journal n'en a pas. Tous les instants affichés sont
ceux du WAV musical.
Code de sortie : 0 sans trou, 1 avec au moins un trou, 2 fichier illisible, musique absente ou origine inconnue.
Sans dépendance (pas de numpy) : bibliothèque standard seule.
"""
import argparse
import array
import math
import re
import struct
import sys
from pathlib import Path

WINDOW_MS = 10


def stream_origin(wav_path):
    """Instant d'ouverture du flux (`monotonic=` de la première ligne du journal du pacer), `None` sans journal ou sans champ."""
    log = Path(wav_path).with_suffix(".log")
    try:
        first = log.read_text(encoding="utf-8", errors="replace").splitlines()[0]
    except (OSError, IndexError):
        return None
    match = re.search(r"\bmonotonic=([0-9.]+)", first)
    return float(match.group(1)) if match else None


def read_wav(path):
    with open(path, "rb") as f:
        data = f.read()
    if data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise ValueError(f"{path} : pas un WAV")
    pos = 12
    fmt = None
    samples = None
    while pos + 8 <= len(data):
        chunk, size = data[pos : pos + 4], struct.unpack("<I", data[pos + 4 : pos + 8])[0]
        body = data[pos + 8 : pos + 8 + size] if chunk != b"data" else data[pos + 8 :]
        if chunk == b"fmt ":
            fmt = struct.unpack("<HHIIHH", body[:16])
        elif chunk == b"data":
            # Taille réelle d'après le fichier : l'en-tête d'un processus arrêté net peut être en retard d'une seconde.
            samples = body
            break
        pos += 8 + size + (size & 1)
    if fmt is None or samples is None:
        raise ValueError(f"{path} : en-tête incomplet")
    audio_format, channels, rate, _, _, bits = fmt
    if audio_format == 3 and bits == 32:
        values = array.array("f")
        values.frombytes(samples[: len(samples) // 4 * 4])
    elif audio_format == 1 and bits == 16:
        raw = array.array("h")
        raw.frombytes(samples[: len(samples) // 2 * 2])
        values = array.array("f", (v / 32768.0 for v in raw))
    else:
        raise ValueError(f"{path} : format {audio_format}/{bits} bits non lu")
    return rate, channels, values


def window_db(rate, channels, values):
    per = rate * WINDOW_MS // 1000 * channels
    out = []
    for start in range(0, len(values) - per + 1, per):
        chunk = values[start : start + per]
        energy = sum(v * v for v in chunk) / per
        out.append(10 * math.log10(energy) if energy > 1e-20 else -200.0)
    return out


def runs_below(levels, threshold, first, last):
    """Suites de fenêtres sous le seuil entre first (inclus) et last (exclu) : [(début, fin)] en index de fenêtre."""
    runs, start = [], None
    for i in range(first, last):
        if levels[i] < threshold:
            if start is None:
                start = i
        elif start is not None:
            runs.append((start, i))
            start = None
    if start is not None:
        runs.append((start, last))
    return runs


def onsets(levels, threshold, quiet_windows):
    out, quiet = [], quiet_windows
    for i, level in enumerate(levels):
        if level >= threshold:
            if quiet >= quiet_windows:
                out.append(i)
            quiet = 0
        else:
            quiet += 1
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("music")
    parser.add_argument("--effects", help="WAV des effets sonores du même processus")
    parser.add_argument("--music-origin", type=float, help="ouverture du flux de musique (s, horloge monotone) ; défaut : son journal")
    parser.add_argument("--effects-origin", type=float, help="ouverture du flux d'effets (s, même horloge) ; défaut : son journal")
    parser.add_argument("--silence-db", type=float, default=-60.0)
    parser.add_argument("--min-gap-ms", type=int, default=150)
    parser.add_argument("--tail-ms", type=int, default=500)
    parser.add_argument("--from-s", type=float, default=0.0, help="ignorer ce qui précède (s)")
    parser.add_argument("--to-s", type=float, default=None, help="ignorer ce qui suit (s)")
    args = parser.parse_args()

    try:
        rate, channels, values = read_wav(args.music)
    except (OSError, ValueError) as error:
        print(f"audio-gaps : {error}", file=sys.stderr)
        return 2
    levels = window_db(rate, channels, values)
    to_index = len(levels) - args.tail_ms // WINDOW_MS
    if args.to_s is not None:
        to_index = min(to_index, int(args.to_s * 1000 / WINDOW_MS))
    from_index = int(args.from_s * 1000 / WINDOW_MS)
    start = next((i for i in range(from_index, to_index) if levels[i] >= args.silence_db), None)
    duration = len(levels) * WINDOW_MS / 1000
    print(f"musique : {args.music} ({duration:.2f} s, {rate} Hz, {channels} canaux)")
    if start is None:
        print("musique : aucune fenêtre au-dessus du seuil : pas de musique")
        return 2
    print(f"musique : début à {start * WINDOW_MS / 1000:.2f} s")
    gaps = [
        (a, b)
        for a, b in runs_below(levels, args.silence_db, start, to_index)
        if (b - a) * WINDOW_MS >= args.min_gap_ms
    ]
    effect_onsets = []
    if args.effects:
        try:
            e_rate, e_channels, e_values = read_wav(args.effects)
        except (OSError, ValueError) as error:
            print(f"audio-gaps : {error}", file=sys.stderr)
            return 2
        e_levels = window_db(e_rate, e_channels, e_values)
        music_origin = args.music_origin if args.music_origin is not None else stream_origin(args.music)
        effects_origin = args.effects_origin if args.effects_origin is not None else stream_origin(args.effects)
        if music_origin is None or effects_origin is None:
            print(
                "audio-gaps : origine d'un flux inconnue (pas de `monotonic=` dans son journal) : --music-origin / --effects-origin",
                file=sys.stderr,
            )
            return 2
        # Fenêtre i du WAV des effets = fenêtre i + shift du WAV musical.
        shift = round((effects_origin - music_origin) * 1000 / WINDOW_MS)
        effect_onsets = [i + shift for i in onsets(e_levels, args.silence_db, 100 // WINDOW_MS)]
        print(
            f"effets : origine décalée de {shift * WINDOW_MS / 1000:+.2f} s ; {len(effect_onsets)} débuts (repère de la musique) : "
            + ", ".join(f"{i * WINDOW_MS / 1000:.2f}" for i in effect_onsets)
        )
    for a, b in gaps:
        near = [i for i in effect_onsets if a - 100 <= i <= b]
        tag = f" ; effet à {', '.join(f'{i * WINDOW_MS / 1000:.2f}' for i in near)} s" if near else ""
        print(f"trou : {a * WINDOW_MS / 1000:.2f} s → {b * WINDOW_MS / 1000:.2f} s ({(b - a) * WINDOW_MS} ms){tag}")
    total = sum(b - a for a, b in gaps) * WINDOW_MS
    print(f"bilan : {len(gaps)} trou(s) de plus de {args.min_gap_ms} ms sous {args.silence_db:.0f} dBFS, {total} ms au total")
    return 1 if gaps else 0


if __name__ == "__main__":
    sys.exit(main())
