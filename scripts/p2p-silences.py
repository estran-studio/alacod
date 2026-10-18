#!/usr/bin/env python3
"""Silences réseau d'une session p2p (D87) : lit les journaux horodatés des clients
(`$OUT/p2p-<i>.log` de `scripts/p2p-stress.sh` ou de `scripts/p2p-machine.sh`) et mesure, par client, la
durée de chaque silence du pair.

GGRS notifie `NetworkInterrupted` après `disconnect_notify_start` (500 ms) sans paquet reçu, puis
`NetworkResumed` au paquet suivant ou `Disconnected` après `disconnect_timeout` (5 s par défaut du jeu depuis D87, 2 s pour GGRS). Le silence
total d'une interruption reprise vaut donc 500 ms + (instant de `NetworkResumed` - instant de
`NetworkInterrupted`) ; pour une déconnexion, c'est le délai de déconnexion (500 ms + écart entre les
deux événements, à quelques millisecondes près). Les horodatages sont ceux du journal du client
(`tracing`, microsecondes) : la précision est celle de la boucle de jeu (une frame, 16,7 ms).

    scripts/p2p-silences.py [--notify-ms 500] [--load load.log] journal.log [journal.log ...]

`--load` : fichier `epoch valeur` (un échantillon par ligne : le taux d'occupation du CPU en %, `scripts/p2p-d87.sh`) ;
le min/moyenne/max pendant la session du client (de sa première à sa dernière ligne horodatée) est affiché.
Classement d'une déconnexion : « FIN DE PARTIE » si un **autre** journal passé en argument s'est terminé avant (le pair avait fini et
quitté : le survivant valide alors les dernières frames avec les inputs par défaut d'un pair déconnecté, et les dernières lignes de sa
trace diffèrent), sinon « EN COURS DE PARTIE » (avec un seul journal, toujours ce dernier ; deux machines : horloges à synchroniser).
Sortie : une ligne par événement, puis un résumé par client sur une ligne `silences client=… interruptions=…
reprises=… deconnexions=… en_cours=… max_ms=… moyenne_reprises_ms=…` (pour le rapport).
"""
import argparse
import datetime
import re
import sys

TS = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+)Z")
EVENT = re.compile(r"GGRS Event: (NetworkInterrupted|NetworkResumed|Disconnected)\b(.*)")
PEER = re.compile(r"addr: (PeerId\([0-9a-f-]+\))")
TIMEOUT = re.compile(r"disconnect_timeout: (\d+)")


def parse_ts(text):
    return datetime.datetime.fromisoformat(text + "+00:00").timestamp()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--notify-ms", type=float, default=500.0)
    ap.add_argument("--load")
    ap.add_argument("logs", nargs="+")
    args = ap.parse_args()
    loads = []
    if args.load:
        for line in open(args.load):
            parts = line.split()
            if len(parts) == 2:
                loads.append((float(parts[0]), float(parts[1])))
    # Dernière ligne horodatée de chaque journal : un `Disconnected` postérieur à la sortie d'un autre client est
    # la fin de partie (le pair a fini sa trace et quitté), pas une perte de la session en cours.
    ends = {}
    for path in args.logs:
        stamps = [parse_ts(m.group(1)) for m in (TS.match(line) for line in open(path, errors="replace")) if m]
        ends[path] = stamps[-1] if stamps else 0.0
    for path in args.logs:
        first = last = None
        pending = {}  # pair -> instant du dernier `NetworkInterrupted` (une session à N pairs en a N-1)
        resumed = []
        disconnects = []
        midrun = 0
        interrupted = 0
        for line in open(path, errors="replace"):
            m = TS.match(line)
            if not m:
                continue
            t = parse_ts(m.group(1))
            first = first if first is not None else t
            last = t
            e = EVENT.search(line)
            if not e:
                continue
            kind = e.group(1)
            who = PEER.search(e.group(2))
            who = who.group(1) if who else "?"
            if kind == "NetworkInterrupted":
                interrupted += 1
                pending[who] = t
                budget = TIMEOUT.search(e.group(2))
                print(f"{path} t={t - first:8.3f}s NetworkInterrupted (reste {budget.group(1) if budget else '?'} ms)")
            elif kind == "NetworkResumed" and who in pending:
                silence = args.notify_ms + (t - pending.pop(who)) * 1000
                resumed.append(silence)
                print(f"{path} t={t - first:8.3f}s NetworkResumed silence={silence:.0f} ms")
            elif kind == "Disconnected":
                silence = args.notify_ms + (t - pending.pop(who)) * 1000 if who in pending else float("nan")
                disconnects.append(silence)
                after_exit = any(end < t for other, end in ends.items() if other != path)
                kind = "FIN DE PARTIE (le pair avait quitté)" if after_exit else "EN COURS DE PARTIE"
                midrun += 0 if after_exit else 1
                print(f"{path} t={t - first:8.3f}s Disconnected silence={silence:.0f} ms {kind}")
        allsil = resumed + [s for s in disconnects if s == s]
        mx = max(allsil) if allsil else 0
        mean = sum(resumed) / len(resumed) if resumed else 0
        load = ""
        if loads and first is not None:
            window = [v for (ts, v) in loads if first <= ts <= last]
            if window:
                load = f" cpu_min={min(window):.1f} cpu_moy={sum(window) / len(window):.1f} cpu_max={max(window):.1f}"
        print(
            f"silences client={path} interruptions={interrupted} reprises={len(resumed)} "
            f"deconnexions={len(disconnects)} en_cours={midrun} max_ms={mx:.0f} moyenne_reprises_ms={mean:.0f}{load}"
        )


if __name__ == "__main__":
    sys.exit(main())
