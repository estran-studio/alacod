#!/usr/bin/env python3
"""Carte LDtk de M4-T0c (contrats du monde, docs/conventions.md §62), testbed :

  rocks_static.ldtk   un couloir fermé par trois rochers (colonne 12) et un rocher à butin
                      (colonne 14) : scénario rocks_static

Part de `surfaces.ldtk` (mêmes définitions de calques et d'entités) et lui ajoute la couche IntGrid
`Rocks` (1 = rocher, 2 = rocher à butin). Relance idempotente (iid dérivés des noms).
Usage : scripts/construire-cartes-t0c.py
"""
import copy
import json
import pathlib
import uuid

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIR = ROOT / "games/testbed/assets/testbed"
BASE = json.loads((DIR / "surfaces.ldtk").read_text())
NS = uuid.UUID("5e1d0b6e-0000-4000-8000-00000000a0c0")


def iid(*parts):
    return str(uuid.uuid5(NS, "/".join(map(str, parts))))


def defs_with_rocks():
    d = copy.deepcopy(BASE)
    surfaces = next(l for l in d["defs"]["layers"] if l["identifier"] == "Surfaces")
    rocks = copy.deepcopy(surfaces)
    rocks.update(identifier="Rocks", uid=d["nextUid"])
    rocks["intGridValues"] = [
        {"value": 1, "identifier": "rocher", "color": "#8A6A4A", "tile": None, "groupUid": 0},
        {"value": 2, "identifier": "rocher_butin", "color": "#C9A227", "tile": None, "groupUid": 0},
    ]
    d["nextUid"] += 1
    d["defs"]["layers"].append(rocks)
    return d, rocks["uid"]


def spawn_entity(d, level, col, row, index):
    definition = next(e for e in d["defs"]["entities"] if e["identifier"] == "PlayerSpawn")
    px = [col * 16, row * 16]
    return {
        "__identifier": "PlayerSpawn",
        "__grid": [col, row],
        "__pivot": [0, 0],
        "__tags": [],
        "__tile": copy.deepcopy(definition["tileRect"]),
        "__smartColor": definition["color"],
        "iid": iid(level, "PlayerSpawn", col, row),
        "width": 16,
        "height": 16,
        "defUid": definition["uid"],
        "px": px,
        "fieldInstances": [
            {
                "__identifier": "index",
                "__type": "Int",
                "__value": index,
                "__tile": None,
                "defUid": 0,
                "realEditorValues": [{"id": "V_Int", "params": [index]}],
            }
        ],
        "__worldX": px[0],
        "__worldY": px[1],
    }


def build(name, level_id, cols, rows, walls, rocks, spawns):
    """walls : ensemble de (col, row) ; rocks : dict (col, row) -> valeur (1 ou 2)."""
    d, rocks_uid = defs_with_rocks()
    lv = copy.deepcopy(d["levels"][0])
    lv.update(identifier=level_id, iid=iid(name, "level"), pxWid=cols * 16, pxHei=rows * 16)
    surfaces = next(li for li in lv["layerInstances"] if li["__identifier"] == "Surfaces")
    rocks_layer = copy.deepcopy(surfaces)
    rocks_layer.update(
        __identifier="Rocks", layerDefUid=rocks_uid, iid=iid(name, "Rocks"), intGridCsv=[]
    )
    lv["layerInstances"].append(rocks_layer)
    for li in lv["layerInstances"]:
        li["iid"] = iid(name, li["__identifier"])
        li["__cWid"], li["__cHei"] = cols, rows
        if li["__type"] == "IntGrid":
            grid = [0] * (cols * rows)
            if li["__identifier"] == "Walls":
                for c, r in walls:
                    grid[r * cols + c] = 1
            if li["__identifier"] == "Rocks":
                for (c, r), value in rocks.items():
                    grid[r * cols + c] = value
            li["intGridCsv"] = grid
        else:
            li["entityInstances"] = [spawn_entity(d, name, c, r, i) for (c, r, i) in spawns]
    d["levels"] = [lv]
    d["iid"] = iid(name, "project")
    (DIR / f"{name}.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print(name, len(lv["layerInstances"]))


def corridor_walls(cols, rows, bands):
    free = {r for (a, b) in bands for r in range(a, b + 1)}
    return {(c, r) for c in range(cols) for r in range(rows) if r not in free or c in (0, cols - 1)}


# Couloir 40x5 (rangées 1-3 libres) : le joueur à la colonne 3 ; trois rochers ferment le couloir
# à la colonne 12 ; un rocher à butin derrière, à la colonne 14.
build(
    "rocks_static",
    "RocksStatic",
    40,
    5,
    corridor_walls(40, 5, [(1, 3)]),
    {(12, 1): 1, (12, 2): 1, (12, 3): 1, (14, 2): 2},
    [(3, 2, 0)],
)
