#!/usr/bin/env python3
"""Script d'inputs pour scripts/p2p-stress.sh : deux joueurs dont les entrées changent toutes
les 1 à 3 frames (graine fixe, donc le même fichier à chaque appel).

Chaque changement d'entrée d'un pair est une prédiction fausse pour l'autre (avec
`ALACOD_INPUT_DELAY=0`), donc un rollback.

    scripts/p2p-stress-inputs.py FRAMES [--seed N] [--pool Up,Down,...] [--hold Bouton:de:à ...] [--hub-door]

--pool : boutons tirés au hasard (défaut : déplacement, tir, dash, recharge, sprint, mêlée).
--hold : bouton maintenu sur `[de, à[` pour les deux joueurs, ajouté à ce que le tirage donne
         (ex. `--hold Interaction:1:3000` pour ramasser des objets au sol à chaque passage).
--hub-door : ajoute les pistes du HUB de gungeon (`hub: Some((stay: false, inputs: …))`, rejouées par
         `ALACOD_INPUT=script:` tant que la session est au hub) : le joueur 0 va au marchand puis à la
         porte (Interaction f170), qui lance la run en ligne. Les `players` jouent la run.
Écrit un scénario RON minimal sur la sortie standard (champs par défaut pour le reste).
"""
import argparse
import random
import sys

DEFAULT_POOL = ["Up", "Down", "Left", "Right", "Fire", "Dash", "Reload", "Sprint", "Melee"]


# Route du hub de gungeon (même parcours que `tests/scenarios/gungeon_full_run.ron`) : le marchand
# (f80), puis la porte (f170). Le joueur 1 ne fait rien.
HUB_DOOR = """    hub: Some((stay: false, inputs: [
        [
            (from: 10, to: 42, buttons: [Right]),
            (from: 42, to: 74, buttons: [Up]),
            (from: 80, to: 82, buttons: [Interaction]),
            (from: 100, to: 138, buttons: [Right]),
            (from: 138, to: 157, buttons: [Down]),
            (from: 170, to: 172, buttons: [Interaction]),
        ],
        [],
    ])),
"""


def player(rng, frames, pool, holds):
    """Segments d'un joueur. Les segments sont coupés aux bornes des `holds` : un maintien
    `[de, à[` s'applique donc à la frame près.
    Testé par `_test_bounds` (`python3 scripts/p2p-stress-inputs.py --selftest`, `make test_tools`).
    """
    cuts = sorted({b for _, start, stop in holds for b in (start, stop)})
    segments = []
    frame = 0
    while frame < frames:
        end = min(frames, frame + rng.randint(1, 3))
        end = min([end] + [b for b in cuts if frame < b < end])
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
    ap.add_argument("--hub-door", action="store_true")
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
    print("    ],")
    if args.hub_door:
        print(HUB_DOOR, end="")
    print(")")


def _test_bounds():
    """Chaque maintien couvre exactement `[de, à[` (`python3 scripts/p2p-stress-inputs.py --selftest`)."""
    import re

    for seed in range(50):
        out = player(random.Random(seed), 100, ["Up"], [("Interaction", 13, 14), ("Dash", 40, 77)])
        held = {"Interaction": set(), "Dash": set()}
        for m in re.finditer(r"from: (\d+), to: (\d+), buttons: \[([^\]]*)\]", out):
            for name in held:
                if name in m.group(3):
                    held[name].update(range(int(m.group(1)), int(m.group(2))))
        assert held["Interaction"] == {13}, (seed, held["Interaction"])
        assert held["Dash"] == set(range(40, 77)), seed


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        _test_bounds()
        print("selftest OK")
    else:
        main()
