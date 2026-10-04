#!/usr/bin/env python3
"""Affiche un tableau des métriques de performance des scénarios.

Lit target/metrics/latest.json et le fichier du commit précédent (s'il existe),
et affiche un tableau comparatif.
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
import os
# Le target de la tâche courante (worktree) ou ./target par défaut, comme cargo. Un chemin
# relatif (le Makefile exporte `CARGO_TARGET_DIR=./target`) se lit depuis la racine du dépôt,
# comme `metrics_dir` de `crates/scenario/tests/scenarios.rs` (D24), quel que soit le dossier
# d'où le script est lancé.
_TARGET = Path(os.environ.get("CARGO_TARGET_DIR") or ROOT / "target")
METRICS_DIR = (_TARGET if _TARGET.is_absolute() else ROOT / _TARGET) / "metrics"


def git(*args):
    try:
        return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout.strip()
    except subprocess.CalledProcessError:
        return ""


def load_metrics(path):
    """Charge les métriques depuis un fichier JSON."""
    try:
        with open(path) as f:
            return json.load(f)
    except (OSError, ValueError):
        return {}


def main():
    current_commit = git("rev-parse", "--short", "HEAD")

    if not METRICS_DIR.exists():
        print("Aucune métrique trouvée (target/metrics/latest.json n'existe pas)")
        print_sim_table(current_commit)
        return

    latest = load_metrics(METRICS_DIR / "latest.json")
    if not latest:
        print("Aucune métrique dans latest.json")
        print_sim_table(current_commit)
        return

    # Essayer de charger le commit précédent
    previous_metrics = {}
    if current_commit:
        parent_commit = git("rev-parse", "--short", f"{current_commit}^") if git("rev-parse", f"{current_commit}^", "--verify") else None
        if parent_commit:
            prev_path = METRICS_DIR / parent_commit / "metrics.json"
            if prev_path.exists():
                previous_metrics = load_metrics(prev_path)

    # Afficher le tableau
    print()
    print("╭─ Scénarios Performance ─────────────────────────────────────────────╮")
    print("│ Scénario              │ FPS   │  Δ FPS │ Entités │ Balles │ Ennemis │")
    print("├───────────────────────┼───────┼────────┼─────────┼────────┼─────────┤")

    for name in sorted(latest.keys()):
        metrics = latest[name]
        prev = previous_metrics.get(name)

        fps_str = f"{metrics['sim_fps']:.1f}"
        delta_str = ""
        if prev:
            delta = metrics['sim_fps'] - prev['sim_fps']
            pct = (delta / prev['sim_fps'] * 100) if prev['sim_fps'] > 0 else 0
            sign = "+" if delta > 0 else ""
            delta_str = f"{sign}{pct:.0f}%"

        print(f"│ {name:21s} │ {fps_str:>5s} │ {delta_str:>6s} │ {metrics['entities_max']:>7} │ {metrics['bullets_max']:>6} │ {metrics['enemies_max']:>7} │")

    print("╰───────────────────────┴───────┴────────┴─────────┴────────┴─────────╯")
    print()

    print_sim_table(current_commit)


def print_sim_table(commit):
    """Tableau des runs `alacod-sim` (T2.11), si `target/metrics/sim-<commit>.json` existe
    (écrit par `make sim`). Minimal : une ligne par graine."""
    if not commit:
        return
    path = METRICS_DIR / f"sim-{commit}.json"
    runs = load_metrics(path)
    if not runs:
        return

    # T1.14 : colonnes de niveaux quand une graine en a franchi, esquives quand non nulles
    with_floors = any(run.get("floor", 0) > 0 for run in runs)
    with_dodges = any(run.get("dodges", 0) > 0 for run in runs)
    headers = ["Graine", "Vague", "Morts", "Kills", "FPS"]
    if with_floors:
        headers += ["Niveau", "Frames/niveau"]
    if with_dodges:
        headers.append("Esquives")
    headers.append("Desync")
    rows = []
    for run in runs:
        row = [
            str(run["seed"]),
            str(run["wave"]),
            str(run["deaths"]),
            str(run["kills"]),
            f"{run['sim_fps']:.1f}",
        ]
        if with_floors:
            frames = run.get("floor_frames", [])
            per_floor = [b - a for a, b in zip([0] + frames, frames)]
            row += [str(run.get("floor", 0)), ",".join(map(str, per_floor)) or "-"]
        if with_dodges:
            row.append(str(run.get("dodges", 0)))
        row.append("OUI" if run.get("desync") else "")
        rows.append(row)
    widths = [max(len(h), *(len(r[i]) for r in rows)) for i, h in enumerate(headers)]
    line = lambda cells: "│ " + " │ ".join(c.rjust(w) for c, w in zip(cells, widths)) + " │"
    print("╭─ alacod-sim " + "─" * (sum(widths) + 3 * len(widths) - 14) + "╮")
    print(line(headers))
    print("├" + "┼".join("─" * (w + 2) for w in widths) + "┤")
    for row in rows:
        print(line(row))
    print("╰" + "┴".join("─" * (w + 2) for w in widths) + "╯")
    if with_floors:
        target = max(run.get("floor", 0) for run in runs)
        done = sum(1 for run in runs if run.get("floor", 0) >= target)
        print(f"Niveaux finis : {done}/{len(runs)} graines au niveau {target}")
    print()

if __name__ == "__main__":
    main()
