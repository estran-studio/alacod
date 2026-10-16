#!/usr/bin/env python3
"""Script d'inputs pour scripts/p2p-stress.sh : deux joueurs dont les entrées changent toutes
les 1 à 3 frames (graine fixe, donc le même fichier à chaque appel).

Chaque changement d'entrée d'un pair est une prédiction fausse pour l'autre (avec
`ALACOD_INPUT_DELAY=0`), donc un rollback.

    scripts/p2p-stress-inputs.py FRAMES [--seed N] [--pool Up,Down,...] [--hold Bouton:de:à ...]

--pool : boutons tirés au hasard (défaut : déplacement, tir, dash, recharge, sprint, mêlée).
--hold : bouton maintenu sur `[de, à[` pour les deux joueurs, ajouté à ce que le tirage donne
         (ex. `--hold Interaction:1:3000` pour ramasser des objets au sol à chaque passage).
Écrit un scénario RON minimal sur la sortie standard (champs par défaut pour le reste).
"""
import argparse
import random

DEFAULT_POOL = ["Up", "Down", "Left", "Right", "Fire", "Dash", "Reload", "Sprint", "Melee"]


def player(rng, frames, pool, holds):
    segments = []
    frame = 0
    while frame < frames:
        end = min(frames, frame + rng.randint(1, 3))
        buttons = [b for b in pool if rng.random() < 0.3]
        for name, start, stop in holds:
            if start <= frame < stop and name not in buttons:
                buttons.append(name)
        pan = (rng.randint(-100, 100), rng.randint(-100, 100))
        segments.append(
            f"            (from: {frame}, to: {end}, buttons: [{', '.join(buttons)}], pan: ({pan[0]}, {pan[1]})),"
        )
        frame = end
    return "        (inputs: [\n" + "\n".join(segments) + "\n        ]),"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("frames", type=int)
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--pool", default=",".join(DEFAULT_POOL))
    ap.add_argument("--hold", action="append", default=[])
    args = ap.parse_args()
    pool = [b for b in args.pool.split(",") if b]
    holds = []
    for spec in args.hold:
        name, start, stop = spec.split(":")
        holds.append((name, int(start), int(stop)))
    rng = random.Random(args.seed)
    print("// Généré par scripts/p2p-stress-inputs.py : entrées qui changent toutes les 1 à 3 frames.")
    print(f"Scenario(\n    frames: {args.frames},\n    players: [")
    for _ in range(2):
        print(player(rng, args.frames, pool, holds))
    print("    ],\n)")


if __name__ == "__main__":
    main()
