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
# Le target de la tâche courante (worktree) ou ./target par défaut, comme cargo.
METRICS_DIR = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "metrics"


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
    if not METRICS_DIR.exists():
        print("Aucune métrique trouvée (target/metrics/latest.json n'existe pas)")
        return

    latest = load_metrics(METRICS_DIR / "latest.json")
    if not latest:
        print("Aucune métrique dans latest.json")
        return

    # Essayer de charger le commit précédent
    current_commit = git("rev-parse", "--short", "HEAD")
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


if __name__ == "__main__":
    main()
