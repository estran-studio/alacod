#!/usr/bin/env python3
"""Compare explicitement deux mesures headless prises sur la même machine, au calme.

Les FPS du runner incluent synctest et instrumentation : ce sont des mesures de débit du
harness, pas le temps d'une frame de production ni une preuve de fluidité du rendu.
"""
import argparse
import json
import math
from pathlib import Path
import sys


def positive(value):
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise argparse.ArgumentTypeError("valeur finie strictement positive attendue")
    return number


def load_metrics(path):
    metrics = json.loads(path.read_text())
    if not isinstance(metrics, dict) or not metrics:
        raise ValueError(f"{path} : mesures absentes ou invalides")
    for name, metric in metrics.items():
        if not isinstance(metric, dict):
            raise ValueError(f"{path} : métrique invalide pour {name}")
        for key in ("sim_fps", "frames", "players"):
            value = metric.get(key)
            if type(value) not in (int, float) or not math.isfinite(value) or value <= 0:
                raise ValueError(f"{path} : {name}.{key} doit être fini et positif")
        for key in ("frames", "players"):
            if not float(metric[key]).is_integer():
                raise ValueError(f"{path} : {name}.{key} doit être entier")
    return metrics


def compare(current, baseline, names, max_regression, min_fps):
    failures = []
    rows = []
    for name in names:
        if name not in current or name not in baseline:
            failures.append(f"{name} : mesure absente du courant ou de la référence")
            continue
        now, before = current[name], baseline[name]
        if any(now[key] != before[key] for key in ("frames", "players")):
            failures.append(f"{name} : charge incomparable (frames ou joueurs différents)")
            continue
        fps = now["sim_fps"]
        delta = (fps / before["sim_fps"] - 1) * 100
        rows.append(f"{name}: {fps:.1f} sim fps ; {1000 / fps:.2f} ms/frame du harness ; variation {delta:+.1f}%")
        if delta < -max_regression:
            failures.append(f"{name} : régression supérieure à {max_regression:g}%")
        if min_fps is not None and fps < min_fps:
            failures.append(f"{name} : débit inférieur à l'objectif explicite {min_fps:g} sim fps")
    return rows, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--current", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--scenario", action="append", help="répétable ; sinon union des scénarios des deux fichiers")
    parser.add_argument("--max-regression", type=positive, default=10.0, help="pourcentage, défaut 10")
    parser.add_argument("--min-sim-fps", type=positive, help="objectif de débit du harness ; indépendant des budgets historiques")
    args = parser.parse_args()
    if args.max_regression >= 100:
        parser.error("--max-regression doit être inférieur à 100")
    if args.current.resolve() == args.baseline.resolve():
        parser.error("la référence doit être un fichier distinct de la mesure courante")
    try:
        current, baseline = load_metrics(args.current), load_metrics(args.baseline)
        names = sorted(set(args.scenario) if args.scenario else current.keys() | baseline.keys())
        rows, failures = compare(current, baseline, names, args.max_regression, args.min_sim_fps)
    except (OSError, ValueError, TypeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    print(f"Référence : {args.baseline}\nMesure : {args.current}")
    print("Comparaison à utiliser sur une même machine, avec les mêmes options et la même instrumentation.")
    for row in rows:
        print(row)
    for failure in failures:
        print(f"ERROR: {failure}", file=sys.stderr)
    if not failures:
        print(f"OK: {len(rows)} scénario(s) dans les limites demandées")
    return int(bool(failures))


if __name__ == "__main__":
    sys.exit(main())
