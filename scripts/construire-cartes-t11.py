#!/usr/bin/env python3
"""Cartes LDtk de M2-T11 (surfaces v2, docs/conventions.md §46), testbed :

  props_pits.ldtk     deux couloirs, une fosse de 2 cases dans le couloir A (pit_fall, pit_dash)
  props_room.ldtk     une salle avec une fosse centrale et un ennemi (pit_enemy_path)
  props_cover.ldtk    un couloir : joueur, table, cible (table_flip_cover)
  props_barrels.ldtk  deux barils voisins dans le couloir A, un joueur dans le couloir B (barrel_chain)

Part de `surfaces.ldtk` (mêmes définitions de calques et d'entités) : ajoute la surface `fosse`
(valeur IntGrid 4) et les entités `Table` et `Barrel`. Relance idempotente (iid dérivés des noms).
Usage : scripts/construire-cartes-t11.py
"""
import copy
import json
import pathlib
import uuid

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIR = ROOT / "games/testbed/assets/testbed"
BASE = json.loads((DIR / "surfaces.ldtk").read_text())
NS = uuid.UUID("5e1d0b6e-0000-4000-8000-00000000a11d")


def iid(*parts):
    return str(uuid.uuid5(NS, "/".join(map(str, parts))))


def defs_with_props():
    d = copy.deepcopy(BASE)
    ents = d["defs"]["entities"]
    spawn = next(e for e in ents if e["identifier"] == "PlayerSpawn")
    uid = d["nextUid"]
    for name, color, w, h in (("Table", "#B5651D", 32, 16), ("Barrel", "#C0392B", 16, 16)):
        e = copy.deepcopy(spawn)
        e.update(identifier=name, uid=uid, width=w, height=h, color=color, fieldDefs=[])
        e["tileRect"] = None
        e["renderMode"] = "Rectangle"
        e["tilesetId"] = None
        ents.append(e)
        uid += 1
    d["nextUid"] = uid
    surfaces = next(l for l in d["defs"]["layers"] if l["identifier"] == "Surfaces")
    surfaces["intGridValues"].append(
        {"value": 4, "identifier": "fosse", "color": "#202020", "tile": None, "groupUid": 0}
    )
    return d


def entity(d, level, name, col, row, w=16, h=16, fields=()):
    definition = next(e for e in d["defs"]["entities"] if e["identifier"] == name)
    px = [col * 16, row * 16]
    fi = []
    for ident, typ, value in fields:
        fi.append(
            {
                "__identifier": ident,
                "__type": typ,
                "__value": value,
                "__tile": None,
                "defUid": 0,
                "realEditorValues": [{"id": "V_" + typ, "params": [value]}],
            }
        )
    return {
        "__identifier": name,
        "__grid": [col, row],
        "__pivot": [0, 0],
        "__tags": [],
        "__tile": None,
        "__smartColor": definition["color"],
        "iid": iid(level, name, col, row),
        "width": w,
        "height": h,
        "defUid": definition["uid"],
        "px": px,
        "fieldInstances": fi,
        "__worldX": px[0],
        "__worldY": px[1],
    }


def build(name, level_id, cols, rows, walls, pits, entities):
    """walls/pits : ensembles de (col, row) ; entities : liste de (nom, col, row, w, h, champs)."""
    d = defs_with_props()
    lv = copy.deepcopy(d["levels"][0])
    lv.update(identifier=level_id, iid=iid(name, "level"), pxWid=cols * 16, pxHei=rows * 16)
    for li in lv["layerInstances"]:
        li["iid"] = iid(name, li["__identifier"])
        li["__cWid"], li["__cHei"] = cols, rows
        if li["__type"] == "IntGrid":
            grid = [0] * (cols * rows)
            if li["__identifier"] == "Walls":
                for c, r in walls:
                    grid[r * cols + c] = 1
            if li["__identifier"] == "Surfaces":
                for c, r in pits:
                    grid[r * cols + c] = 4
            li["intGridCsv"] = grid
        else:
            li["entityInstances"] = [
                entity(d, name, n, c, r, w, h, f) for (n, c, r, w, h, f) in entities
            ]
    d["levels"] = [lv]
    d["iid"] = iid(name, "project")
    (DIR / f"{name}.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print(name, len(d["levels"][0]["layerInstances"]))


def corridor_walls(cols, rows, bands):
    """Murs d'un niveau à couloirs : toutes les cases hors des rangées libres `bands`."""
    free = {r for (a, b) in bands for r in range(a, b + 1)}
    return {(c, r) for c in range(cols) for r in range(rows) if r not in free or c in (0, cols - 1)}


def spawn(index, col, row):
    return ("PlayerSpawn", col, row, 16, 16, [("index", "Int", index)])


# 1. Deux couloirs ; fosse de 2 cases (colonnes 30-31) dans le couloir A (rangées 1-3).
build(
    "props_pits",
    "PropsPits",
    60,
    9,
    corridor_walls(60, 9, [(1, 3), (5, 7)]),
    {(c, r) for c in (30, 31) for r in (1, 2, 3)},
    [spawn(0, 3, 2), spawn(1, 3, 6)],
)

# 2. Salle 30x11 : fosse centrale (colonnes 12-15, rangées 3-7), deux passages de 2 cases (rangées
# 1-2 et 8-9). Le joueur à droite, un `follower` à gauche.
room_walls = {(c, r) for c in range(30) for r in range(11) if c in (0, 29) or r in (0, 10)}
build(
    "props_room",
    "PropsRoom",
    30,
    11,
    room_walls,
    {(c, r) for c in range(12, 16) for r in range(3, 8)},
    [
        spawn(0, 26, 5),
        (
            "CharacterSpawn",
            3,
            5,
            16,
            16,
            [("character", "String", "follower"), ("team", "String", "enemies")],
        ),
    ],
)

# 3. Couloir de couverture : joueur col 3, table 32x32 col 12-13 (rangées 1-2), cible col 20.
build(
    "props_cover",
    "PropsCover",
    60,
    5,
    corridor_walls(60, 5, [(1, 3)]),
    set(),
    [
        spawn(0, 3, 2),
        ("Table", 12, 1, 32, 32, []),
        (
            "CharacterSpawn",
            20,
            2,
            16,
            16,
            [("character", "String", "target"), ("team", "String", "enemies")],
        ),
    ],
)

# 4. Deux barils voisins dans le couloir A (rangée 3), un joueur dans le couloir B à portée du
# seul second baril (colonne 16, rangée 5).
build(
    "props_barrels",
    "PropsBarrels",
    60,
    9,
    corridor_walls(60, 9, [(1, 3), (5, 7)]),
    set(),
    [
        spawn(0, 3, 3),
        spawn(1, 16, 5),
        ("Barrel", 12, 3, 16, 16, []),
        ("Barrel", 14, 3, 16, 16, []),
    ],
)
