#!/usr/bin/env python3
"""Cartes LDtk de M3-T5 (portes à clé, docs/conventions.md §70), testbed :

  salles_cle.ldtk         deux salles assemblées (SalleDepart, SalleTresor de type `tresor`, copie de la salle
                          de combat de `salles.ldtk` sans ennemi ni porte à l'est : un cul-de-sac) : la porte
                          de l'est de la salle de départ et sa jumelle (bord ouest du trésor) sont la porte à
                          clé (`door_lock: Key(1)`) : scénarios key_door_open et key_door_no_key
  salles_cle_combat.ldtk  `salles.ldtk` (départ, combat aux deux `dummy`, récompense) dont la récompense devient
                          le trésor : la porte de l'est de la salle de combat et sa jumelle (bord ouest du
                          trésor) sont la porte à clé ; nettoyer la salle de combat voisine ne doit pas
                          l'ouvrir : scénario key_door_after_clear
  salles_cle_floors.ldtk  `salles_cle.ldtk` dont le trésor est de type `tresor_ferme` (`lock_from_floor: 1`),
                          plus un `dummy` ennemi dans la salle de départ (le portail du mode `Floors` reste
                          fermé tant qu'il vit) : séquence `deux_cles`, scénario key_door_floors

Part de `salles.ldtk` (salles de 8 x 8 cases). Relance idempotente (iid dérivés des noms).
Usage : scripts/construire-cartes-t5.py
"""
import copy
import json
import pathlib
import uuid

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIR = ROOT / "games/testbed/assets/testbed"
SALLES = json.loads((DIR / "salles.ldtk").read_text())
NS = uuid.UUID("5e1d0b6e-0000-4000-8000-00000000a0c5")


def iid(*parts):
    return str(uuid.uuid5(NS, "/".join(map(str, parts))))


def set_room_kind(level, kind):
    for field in level["fieldInstances"]:
        if field["__identifier"] == "room_kind":
            field["__value"] = kind


# 1. Deux salles assemblées : SalleDepart (spawn, porte à l'est) et SalleTresor (copie de la salle de combat
# de `salles.ldtk`, de type `tresor`, sans ennemi ni porte à l'est : un cul-de-sac).
def cle_project():
    d = copy.deepcopy(SALLES)
    depart, combat = d["levels"][0], copy.deepcopy(d["levels"][1])
    combat.update(identifier="SalleTresor", iid=iid("salles_cle", "SalleTresor"))
    set_room_kind(combat, "tresor")
    for layer in combat["layerInstances"]:
        layer["iid"] = iid("salles_cle", "SalleTresor", layer["__identifier"])
        if layer["__type"] == "Entities":
            layer["entityInstances"] = [
                e for e in layer["entityInstances"] if e["__identifier"] == "DoorVertical" and e["px"] == [0, 48]
            ]
            for e in layer["entityInstances"]:
                e["iid"] = iid("salles_cle", "door", e["px"][0], e["px"][1])
        else:
            c = layer["__cWid"]
            for r in (1, 2):  # ferme le passage de l'est
                layer["intGridCsv"][r * c + 7] = 1 if layer["__identifier"] == "Walls" else 0
    d["levels"] = [depart, combat]
    d["iid"] = iid("salles_cle", "project")
    (DIR / "salles_cle.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print("salles_cle", len(d["levels"]))


cle_project()


# 2. `salles.ldtk` dont la récompense devient le trésor, collé à l'est de la salle de combat.
def cle_after_combat():
    d = copy.deepcopy(SALLES)
    for level in d["levels"]:
        if level["identifier"] == "SalleRecompense":
            level["identifier"] = "SalleTresorVoisine"
            level["iid"] = iid("salles_cle_combat", "SalleTresorVoisine")
            set_room_kind(level, "tresor")
            for layer in level["layerInstances"]:
                layer["iid"] = iid("salles_cle_combat", "tresor", layer["__identifier"])
                for e in layer.get("entityInstances", []):
                    e["iid"] = iid("salles_cle_combat", "door", e["px"][0], e["px"][1])
    d["iid"] = iid("salles_cle_combat", "project")
    (DIR / "salles_cle_combat.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print("salles_cle_combat", len(d["levels"]))


cle_after_combat()


# 3. `salles_cle.ldtk` : trésor de type `tresor_ferme` + un `dummy` ennemi en haut de la salle de départ
# (hors du couloir vers la porte) : deux étages identiques en `Floors` (séquence `deux_cles`).
def cle_floors():
    d = json.loads((DIR / "salles_cle.ldtk").read_text())
    template = next(
        e
        for level in SALLES["levels"]
        if level["identifier"] == "SalleCombat"
        for layer in level["layerInstances"]
        for e in layer.get("entityInstances", [])
        if e["__identifier"] == "CharacterSpawn"
    )
    dummy = copy.deepcopy(template)
    dummy["iid"] = iid("salles_cle_floors", "dummy")
    dummy["px"] = [32, 16]
    dummy["__grid"] = [2, 1]
    depart, tresor = d["levels"]
    set_room_kind(tresor, "tresor_ferme")
    entities = next(l for l in depart["layerInstances"] if l["__type"] == "Entities")
    entities["entityInstances"].append(dummy)
    d["iid"] = iid("salles_cle_floors", "project")
    (DIR / "salles_cle_floors.ldtk").write_text(json.dumps(d, indent=2) + "\n")
    print("salles_cle_floors", len(d["levels"]))


cle_floors()
