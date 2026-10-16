#!/usr/bin/env python3
"""Cartes LDtk de M3-T4 (bombes, rochers, piques, mur secret, docs/conventions.md §69), testbed :

  spikes.ldtk         un couloir dont quatre colonnes (12 à 15) sont des piques (Surfaces, valeur 5) :
                      scénario spikes_damage
  rock_flow.ldtk      une salle coupée par un mur à la colonne 15 : un passage de deux cases au milieu
                      bouché par deux rochers, un passage ouvert en haut (long détour) : scénarios
                      rock_flow_field et rock_flow_field_blocked
  salles_secret_combat.ldtk  `salles.ldtk` dont la salle de récompense (collée à l'est de la salle de combat) est
                      de type `secrete` : scénario secret_wall_after_clear (nettoyer la salle de combat voisine
                      n'ouvre pas le mur secret)
  salles_secret_floors.ldtk  `salles_secret.ldtk` plus un `dummy` ennemi dans la salle de départ (le portail du mode
                      `Floors` reste fermé tant qu'il vit) : séquence `deux_secrets`, scénario secret_wall_floors
  salles_secret.ldtk  deux salles assemblées (SalleDepart, SalleSecrete de type `secrete`, copie de
                      `salles.ldtk` sans ennemi) : scénarios secret_wall_bomb et secret_wall_closed

Part de `surfaces.ldtk` (calques et entités) et de `salles.ldtk` (salles de 8 x 8 cases). Relance
idempotente (iid dérivés des noms).
Usage : scripts/construire-cartes-t4.py
"""
import copy
import json
import pathlib
import uuid

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIR = ROOT / "games/testbed/assets/testbed"
BASE = json.loads((DIR / "surfaces.ldtk").read_text())
SALLES = json.loads((DIR / "salles.ldtk").read_text())
NS = uuid.UUID("5e1d0b6e-0000-4000-8000-00000000a0c4")


def iid(*parts):
    return str(uuid.uuid5(NS, "/".join(map(str, parts))))


def defs_with_rocks_and_spikes():
    d = copy.deepcopy(BASE)
    surfaces = next(l for l in d["defs"]["layers"] if l["identifier"] == "Surfaces")
    surfaces["intGridValues"].append(
        {"value": 5, "identifier": "piques", "color": "#9A9A9A", "tile": None, "groupUid": 0}
    )
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


def build(name, level_id, cols, rows, walls, rocks, surfaces, spawns):
    """walls : ensemble de (col, row) ; rocks : dict (col, row) -> 1 ou 2 ; surfaces : dict -> valeur."""
    d, rocks_uid = defs_with_rocks_and_spikes()
    lv = copy.deepcopy(d["levels"][0])
    lv.update(identifier=level_id, iid=iid(name, "level"), pxWid=cols * 16, pxHei=rows * 16)
    surfaces_layer = next(li for li in lv["layerInstances"] if li["__identifier"] == "Surfaces")
    rocks_layer = copy.deepcopy(surfaces_layer)
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
            if li["__identifier"] == "Surfaces":
                for (c, r), value in surfaces.items():
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


# 1. Couloir de piques : rangées 1 à 3 libres, piques aux colonnes 12 à 15, joueur colonne 3.
build(
    "spikes",
    "Spikes",
    40,
    5,
    corridor_walls(40, 5, [(1, 3)]),
    {},
    {(c, r): 5 for c in range(12, 16) for r in range(1, 4)},
    [(3, 2, 0)],
)

# 2. Salle coupée en deux par un mur (colonne 15) : passage de deux cases en bas (rangées 8 et 9) bouché
# par deux rochers, passage ouvert en haut (rangées 1 et 2) : un long détour. Joueur colonne 12, rangée 8.
COLS, ROWS = 32, 11
flow_walls = {(c, r) for c in range(COLS) for r in range(ROWS) if c in (0, COLS - 1) or r in (0, ROWS - 1)}
flow_walls |= {(15, r) for r in range(1, ROWS - 1) if r not in (1, 2, 8, 9)}
build(
    "rock_flow",
    "RockFlow",
    COLS,
    ROWS,
    flow_walls,
    {(15, 8): 1, (15, 9): 1},
    {},
    [(12, 8, 0)],
)


# 3. Deux salles assemblées : SalleDepart (spawn, porte à l'est) et SalleSecrete (copie de la salle
# de combat de `salles.ldtk`, de type `secrete`, sans ennemi ni porte à l'est : un cul-de-sac).
def secret_project():
    d = copy.deepcopy(SALLES)
    depart, combat = d["levels"][0], copy.deepcopy(d["levels"][1])
    combat.update(identifier="SalleSecrete", iid=iid("salles_secret", "SalleSecrete"))
    for field in combat["fieldInstances"]:
        if field["__identifier"] == "room_kind":
            field["__value"] = "secrete"
    for layer in combat["layerInstances"]:
        layer["iid"] = iid("salles_secret", "SalleSecrete", layer["__identifier"])
        if layer["__type"] == "Entities":
            layer["entityInstances"] = [
                e for e in layer["entityInstances"] if e["__identifier"] == "DoorVertical" and e["px"] == [0, 48]
            ]
            for e in layer["entityInstances"]:
                e["iid"] = iid("salles_secret", "door", e["px"][0], e["px"][1])
        else:
            c = layer["__cWid"]
            for r in (1, 2):  # ferme le passage de l'est
                layer["intGridCsv"][r * c + 7] = 1 if layer["__identifier"] == "Walls" else 0
    d["levels"] = [depart, combat]
    d["iid"] = iid("salles_secret", "project")
    (DIR / "salles_secret.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print("salles_secret", len(d["levels"]))


secret_project()


# 4. `salles.ldtk` (départ, combat aux deux `dummy`, récompense) dont la récompense devient la salle
# secrète : la porte de l'est de la salle de combat et sa jumelle (bord ouest de la salle secrète) forment
# le mur secret ; nettoyer la salle de combat ne doit pas l'ouvrir.
def secret_after_combat():
    d = copy.deepcopy(SALLES)
    for level in d["levels"]:
        if level["identifier"] == "SalleRecompense":
            level["identifier"] = "SalleSecreteVoisine"
            level["iid"] = iid("salles_secret_combat", "SalleSecreteVoisine")
            for field in level["fieldInstances"]:
                if field["__identifier"] == "room_kind":
                    field["__value"] = "secrete"
            for layer in level["layerInstances"]:
                layer["iid"] = iid("salles_secret_combat", "secrete", layer["__identifier"])
                for e in layer.get("entityInstances", []):
                    e["iid"] = iid("salles_secret_combat", "door", e["px"][0], e["px"][1])
    d["iid"] = iid("salles_secret_combat", "project")
    (DIR / "salles_secret_combat.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print("salles_secret_combat", len(d["levels"]))


secret_after_combat()


# 5. `salles_secret.ldtk` + un `dummy` ennemi dans la salle de départ, en haut de la salle (hors du
# couloir vers la porte secrète) : deux étages identiques en `Floors` (séquence `deux_secrets`).
def secret_floors():
    d = json.loads((DIR / "salles_secret.ldtk").read_text())
    template = next(
        e
        for level in SALLES["levels"]
        if level["identifier"] == "SalleCombat"
        for layer in level["layerInstances"]
        for e in layer.get("entityInstances", [])
        if e["__identifier"] == "CharacterSpawn"
    )
    dummy = copy.deepcopy(template)
    dummy["iid"] = iid("salles_secret_floors", "dummy")
    dummy["px"] = [32, 16]
    dummy["__grid"] = [2, 1]
    depart = d["levels"][0]
    entities = next(l for l in depart["layerInstances"] if l["__type"] == "Entities")
    entities["entityInstances"].append(dummy)
    d["iid"] = iid("salles_secret_floors", "project")
    (DIR / "salles_secret_floors.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print("salles_secret_floors", len(d["levels"]))


secret_floors()
