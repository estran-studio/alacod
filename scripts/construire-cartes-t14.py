#!/usr/bin/env python3
"""Carte LDtk de M3-T14 (bot `explorateur` et rochers, docs/conventions.md §79), testbed :

  bot_rocks.ldtk   deux salles de 40 x 11 cases séparées par un mur (colonne 20) dont le seul passage, deux cases
                   (rangées 4 et 5), est bouché par deux rochers : aucun détour. Deux `PlayerSpawn` (le joueur 0 à
                   la colonne 12, l'index 1, libre, à la colonne 35) : le portail du mode `Floors` est au
                   barycentre des spawns, donc dans la salle de droite, derrière les rochers. Aucun ennemi : le
                   portail est ouvert dès le départ et le seul moyen de l'atteindre est la bombe. Séquence
                   `bot_rocks` (la même carte deux fois), scénario `bot_isaac_rock`.

Réutilise le constructeur de `construire-cartes-t4.py` (relance idempotente, iid dérivés des noms).
Usage : scripts/construire-cartes-t14.py
"""
import importlib.util
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("t4", ROOT / "scripts/construire-cartes-t4.py")
t4 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(t4)

COLS, ROWS = 40, 11
walls = {(c, r) for c in range(COLS) for r in range(ROWS) if c in (0, COLS - 1) or r in (0, ROWS - 1)}
walls |= {(20, r) for r in range(1, ROWS - 1) if r not in (4, 5)}
t4.build(
    "bot_rocks",
    "BotRocks",
    COLS,
    ROWS,
    walls,
    {(20, 4): 1, (20, 5): 1},
    {},
    [(12, 4, 0), (35, 4, 1)],
)
