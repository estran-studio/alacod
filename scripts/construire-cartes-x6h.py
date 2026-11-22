#!/usr/bin/env python3
"""X-6h (docs/conventions.md §130) : cartes du testbed pour les portes de combat.

  - `salles_combat_cle.ldtk` (à partir de `portes_cle_salles.ldtk`) : départ (le `porteur_cle` y vit), salle de combat (deux `dummy`)
    et récompense. La porte départ ↔ combat porte `lock_key: "cle_test"` (consommée) : le joueur la rouvre avec la clé, entre,
    et la salle se verrouille **derrière lui** (la porte à serrure, déjà ouverte, se referme).
  - `salles_combat_sans_rappel.ldtk` (à partir de `salles.ldtk`) : la salle de combat est de type `combat_sans_rappel`
    (`rooms/combat_sans_rappel.ron` : `no_rally: true`, un coéquipier resté dehors n'est pas ramené à l'entrée).

Usage : python3 scripts/construire-cartes-x6h.py   (depuis la racine du dépôt)
"""
import copy
import json
import uuid

DIR = "games/testbed/assets/testbed/"


def entities(level):
    return next(l for l in level["layerInstances"] if l["__identifier"] == "Entities")["entityInstances"]


def spawn(template, character, grid):
    e = copy.deepcopy(template)
    e["iid"] = str(uuid.uuid4())
    e["__grid"] = list(grid)
    e["px"] = [grid[0] * 16, grid[1] * 16]
    for f in e["fieldInstances"]:
        if f["__identifier"] == "character":
            f["__value"] = character
            f["realEditorValues"] = [{"id": "V_String", "params": [character]}]
    return e


def save(project, name, tail):
    project["iid"] = project["iid"][:-4] + tail
    with open(DIR + name, "w") as out:
        json.dump(project, out, indent=2, ensure_ascii=False)
        out.write("\n")


def set_kind(level, kind):
    for f in level["fieldInstances"]:
        if f["__identifier"] == "room_kind":
            f["__value"] = kind
            for real in f.get("realEditorValues", []):
                real["params"] = [kind]


# 1. Porte à clé à l'entrée de la salle de combat.
cle = json.load(open(DIR + "portes_cle_salles.ldtk"))
depart, combat, finale = cle["levels"]
carrier = next(e for e in entities(combat) if e["__identifier"] == "CharacterSpawn")
entities(combat).remove(carrier)
lock_fields = None
for e in entities(combat):
    if e["__identifier"] == "DoorVertical" and e["px"] == [112, 16]:
        lock_fields = [f for f in e["fieldInstances"] if f["__identifier"].startswith("lock_")]
        e["fieldInstances"] = [f for f in e["fieldInstances"] if not f["__identifier"].startswith("lock_")]
for e in entities(combat):
    if e["__identifier"] == "DoorVertical" and e["px"] == [0, 48]:
        e["fieldInstances"] += lock_fields
entities(depart).append(spawn(carrier, "porteur_cle", (5, 4)))
for grid in ((6, 3), (6, 4)):
    entities(combat).append(spawn(carrier, "dummy", grid))
set_kind(finale, "recompense")
save(cle, "salles_combat_cle.ldtk", "6h01")

# 2. Salle de combat qui ne ramène pas l'équipe.
salles = json.load(open(DIR + "salles.ldtk"))
set_kind(salles["levels"][1], "combat_sans_rappel")
save(salles, "salles_combat_sans_rappel.ldtk", "6h02")
