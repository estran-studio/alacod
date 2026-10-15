#!/usr/bin/env python3
"""Refuse les faux succès P2P : traces absentes, vides, tronquées ou divergentes."""
import argparse
from pathlib import Path
import re
import sys

LINE = re.compile(r"([0-9]+) ([0-9a-f]{32}) ([0-9]+)")


def read_trace(path, expected_frames):
    lines = path.read_text().splitlines()
    if len(lines) != expected_frames:
        raise ValueError(f"{path} : {len(lines)} lignes, {expected_frames} attendues")
    for frame, line in enumerate(lines):
        match = LINE.fullmatch(line)
        if match is None or int(match[1]) != frame:
            raise ValueError(f"{path} : ligne {frame + 1} invalide ou frame manquante")
    return lines


def check(paths, expected_frames):
    if expected_frames < 1 or len(paths) < 2:
        raise ValueError("au moins deux traces et une frame sont requises")
    if len({path.resolve() for path in paths}) != len(paths):
        raise ValueError("chaque pair doit avoir son propre fichier de trace")
    reference = read_trace(paths[0], expected_frames)
    for path in paths[1:]:
        lines = read_trace(path, expected_frames)
        for frame, (before, after) in enumerate(zip(reference, lines)):
            if before != after:
                raise ValueError(f"{paths[0]} / {path} : divergence frame {frame}\n{before}\n{after}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frames", type=int, required=True, help="nombre exact de lignes ; EXIT_AT_FRAME - 1 sur un boot frais")
    parser.add_argument("traces", type=Path, nargs="+")
    args = parser.parse_args()
    try:
        check(args.traces, args.frames)
    except (OSError, ValueError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print(f"OK: {len(args.traces)} pairs, {args.frames} frames contiguës et identiques")
    return 0


if __name__ == "__main__":
    sys.exit(main())
