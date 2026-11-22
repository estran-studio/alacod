#!/usr/bin/env python3
"""X-6f (docs/conventions.md §128) : cartes du testbed à porte de sortie vers le niveau suivant.

À partir de `testbed/portes_cle.ldtk` (salle de départ 12×8 à deux `PlayerSpawn`, porte à l'est à
`lock_key: "cle_test"`, seconde salle) :
  - `porte_sortie.ldtk`       : la porte garde sa clé et gagne `exit_next_level: true` ;
  - `porte_sortie_libre.ldtk` : `exit_next_level: true`, sans serrure (s'ouvre par Interaction).
Les définitions `DoorHorizontal`/`DoorVertical` gagnent le champ booléen `exit_next_level`.

Usage : python3 scripts/construire-cartes-x6f.py   (depuis la racine du dépôt)
"""
import copy
import json

DIR = "games/testbed/assets/testbed/"
src = json.load(open(DIR + "portes_cle.ldtk"))


def add_exit_field(project):
    """Ajoute le champ `exit_next_level` aux deux définitions de porte ; renvoie l'uid par définition."""
    uid = project["nextUid"]
    uids = {}
    for definition in project["defs"]["entities"]:
        if not definition["identifier"].startswith("Door"):
            continue
        field = copy.deepcopy(
            next(f for f in definition["fieldDefs"] if f["identifier"] == "lock_consume")
        )
        field["identifier"] = "exit_next_level"
        field["uid"] = uid
        field["defaultOverride"] = None
        uids[definition["uid"]] = uid
        definition["fieldDefs"].append(field)
        uid += 1
    project["nextUid"] = uid
    return uids


def build(name, locked):
    project = copy.deepcopy(src)
    uids = add_exit_field(project)
    first = True
    for level in project["levels"]:
        for layer in level["layerInstances"]:
            for entity in layer["entityInstances"]:
                if not entity["__identifier"].startswith("Door"):
                    continue
                exit_here = level["identifier"] == "PortesCleA"
                entity["fieldInstances"].append(
                    {
                        "__identifier": "exit_next_level",
                        "__type": "Bool",
                        "__value": exit_here,
                        "__tile": None,
                        "defUid": uids[entity["defUid"]],
                        "realEditorValues": [{"id": "V_Bool", "params": [exit_here]}]
                        if exit_here
                        else [],
                    }
                )
                if not locked:
                    for f in entity["fieldInstances"]:
                        if f["__identifier"] == "lock_key":
                            f["__value"] = None
                            f["realEditorValues"] = []
    # iid stables, distinctes de la carte d'origine.
    project["iid"] = project["iid"][:-4] + ("a1b2" if locked else "c3d4")
    with open(DIR + name, "w") as out:
        json.dump(project, out, indent=2, ensure_ascii=False)
        out.write("\n")


build("porte_sortie.ldtk", True)
build("porte_sortie_libre.ldtk", False)


# Gabarits de l'étage assemblé (`floor_grammars/etage_sortie.ron`, §40) : départ (un `dummy` y dort) et boss, à partir de
# `salles_cle_floors.ldtk` (deux salles reliées, une seule connexion chacune : le boss est un cul-de-sac). Le second
# niveau devient une salle de type `boss`.
gabarits = json.load(open(DIR + "salles_cle_floors.ldtk"))
for level in gabarits["levels"]:
    for field in level["fieldInstances"]:
        if field["__identifier"] == "room_kind" and field["__value"] == "tresor_ferme":
            field["__value"] = "boss"
            for real in field.get("realEditorValues", []):
                real["params"] = ["boss"]
gabarits["iid"] = gabarits["iid"][:-4] + "e5f6"
with open(DIR + "etage_sortie.ldtk", "w") as out:
    json.dump(gabarits, out, indent=2, ensure_ascii=False)
    out.write("\n")
