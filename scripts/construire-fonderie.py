#!/usr/bin/env python3
"""Outil auteur de La Fonderie : bibliothèque statique LDtk, jamais appelé en jeu.

Relancer depuis n'importe quel dossier réécrit uniquement la_fonderie.ldtk.
Les rectangles sont exprimés en cases, borne droite/basse exclue. Les cours sont
des extérieurs ouverts sur le bord de la carte (le vide, comme CoD) : le zombie
apparaît dehors, derrière le mur du bâtiment, et n'entre qu'en cassant une fenêtre.
Le joueur, lui, ne peut jamais franchir une fenêtre : il sécurise l'intérieur.
"""

from collections import deque
from copy import deepcopy
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "games/zombies/assets/maps/avant_poste.ldtk"
CHARACTERS = ROOT / "games/zombies/assets/exemples/zombie_scaled.ldtk"
OUTPUT = ROOT / "games/zombies/assets/maps/la_fonderie.ldtk"
CELL = 16
ATLAS = 38


def stable(key):
    return str(uuid5(NAMESPACE_URL, "alacod/la_fonderie/" + key))


def rectangle(x0, y0, x1, y1):
    return {(x, y) for y in range(y0, y1) for x in range(x0, x1)}


def flood(start, allowed):
    reached = {start}
    pending = deque([start])
    while pending:
        x, y = pending.popleft()
        for point in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if point in allowed and point not in reached:
                reached.add(point)
                pending.append(point)
    return reached


class Section:
    def __init__(self, name, kind, size, ports, design):
        self.name, self.kind = name, kind
        self.width, self.height = size
        self.ports, self.design = ports, design
        self.free, self.masses, self.exterior = set(), set(), set()
        self.entities, self.courts, self.windows = [], [], set()
        self.connections = set()

    def carve(self, *rects):
        for rect in rects:
            self.free.update(rectangle(*rect))
        return self

    def block(self, *rects):
        for rect in rects:
            cells = rectangle(*rect)
            self.free.difference_update(cells)
            self.masses.update(cells)
        return self

    def entity(self, kind, x, y, width=1, height=1, **fields):
        self.entities.append((kind, x, y, width, height, fields))
        return self

    def weapon(self, x, y, name, price):
        return self.entity("WeaponLocation", x, y, weapon=name, price=price)

    def perk(self, x, y, name):
        return self.entity("SodaLocation", x, y, perk=name)

    def players(self, points):
        for index, (x, y) in enumerate(points):
            self.entity("PlayerSpawn", x, y, index=index)
        return self

    def court(self, x, y, facing):
        # Cour extérieure ouverte sur le bord de la carte : le zombie apparaît
        # DEHORS, derrière le mur du bâtiment, et n'entre qu'en cassant la fenêtre
        # (modèle CoD). `x, y` : coin intérieur de la cour (le spawner est en
        # (x+3, y+3)) ; `facing` : côté du bâtiment qui porte la fenêtre. Les bords
        # opposés au bâtiment restent ouverts sur le vide de la carte.
        open_west = x < self.width // 2
        open_north = y < self.height // 2
        x0 = 0 if open_west else x
        y0 = 0 if open_north else y
        x1 = self.width if not open_west else x + 7
        y1 = self.height if not open_north else y + 7
        yard = rectangle(x0, y0, x1, y1)
        self.free.update(yard)

        if facing == "E":
            window = rectangle(x + 6, y + 3, x + 7, y + 5)
            approach = (x + 7, y + 3)
        elif facing == "W":
            window = rectangle(x, y + 3, x + 1, y + 5)
            approach = (x - 1, y + 3)
        elif facing == "S":
            window = rectangle(x + 3, y + 6, x + 5, y + 7)
            approach = (x + 3, y + 7)
        elif facing == "N":
            window = rectangle(x + 3, y, x + 5, y + 1)
            approach = (x + 3, y - 1)
        else:
            raise ValueError(facing)

        # Mur du bâtiment : les deux côtés intérieurs (pas ceux ouverts sur le
        # bord), fenêtre exceptée. Le joueur est ainsi confiné à l'intérieur.
        inner_x = "E" if open_west else "W"
        inner_y = "S" if open_north else "N"
        walls = set()
        for side in (inner_x, inner_y):
            if side == "E":
                walls |= rectangle(x + 6, y0, x + 7, y1)
            elif side == "W":
                walls |= rectangle(x, y0, x + 1, y1)
            elif side == "S":
                walls |= rectangle(x0, y + 6, x1, y + 7)
            elif side == "N":
                walls |= rectangle(x0, y, x1, y + 1)
        walls -= window
        self.free.difference_update(walls)
        self.exterior.update(yard - walls)
        self.windows.update(window)

        # Noms historiques : Horizontal est vertical géométriquement.
        if facing in ("E", "W"):
            wx = x + 6 if facing == "E" else x
            self.entity("WindowHorizontal", wx, y + 3, width=1, height=2)
        else:
            wy = y + 6 if facing == "S" else y
            self.entity("WindowVertical", x + 3, wy, width=2, height=1)
        self.entity("ZombieSpawn", x + 3, y + 3, active_x=0, active_y=0,
                    active_width=self.width * CELL, active_height=self.height * CELL)
        self.courts.append((yard - walls - window, window, approach, (x + 3, y + 3)))
        return self

    def finish_ports(self):
        # Le bord reste ouvert sur les cours extérieures (les zombies viennent du
        # dehors) ; partout ailleurs il est muré, prises exceptées.
        self.free = {(x, y) for x, y in self.free
                     if (0 < x < self.width - 1 and 0 < y < self.height - 1)
                     or (x, y) in self.exterior}
        for side, start in self.ports:
            if side == "N":
                corridor = (start, 0, start + 4, 7)
                opening = (start, 0, 4, 1)
            elif side == "S":
                corridor = (start, self.height - 7, start + 4, self.height)
                opening = (start, self.height - 1, 4, 1)
            elif side == "W":
                corridor = (0, start, 7, start + 4)
                opening = (0, start, 1, 4)
            else:
                corridor = (self.width - 7, start, self.width, start + 4)
                opening = (self.width - 1, start, 1, 4)
            self.carve(corridor)
            x, y, width, height = opening
            self.connections.update(rectangle(x, y, x + width, y + height))
            self.entity("DoorHorizontal" if width == 4 else "DoorVertical",
                        *opening, price=0, electrify=False)

    def validate(self):
        def require(condition, detail):
            if not condition:
                raise ValueError(f"{self.name}: {detail}")

        bounds = rectangle(0, 0, self.width, self.height)
        require(self.free <= bounds, "géométrie hors limites")
        border = {p for p in bounds if p[0] in (0, self.width - 1)
                  or p[1] in (0, self.height - 1)}
        expected_border = {p for p in self.free if p in border}
        require(expected_border == self.connections | (self.exterior & border),
                "bord ouvert hors prise ou cour extérieure")
        require(2 <= len(self.courts) <= 4, "deux à quatre cours requises")
        courtyard_cells = set().union(*(court[0] for court in self.courts))
        player_space = self.free - self.windows - courtyard_cells
        start = min(player_space)
        reached = flood(start, player_space)
        require(reached == player_space, "plateau joueur déconnecté sans fenêtres")
        # Un centre avec voisinage 3×3 libre prouve des liaisons de trois
        # cases, et pas seulement une connexion topologique d'une case.
        wide = {p for p in player_space if all(
            (p[0] + dx, p[1] + dy) not in bounds
            or (p[0] + dx, p[1] + dy) in player_space
            for dy in (-1, 0, 1) for dx in (-1, 0, 1))}
        require(bool(wide), "aucun passage large")
        wide_reached = flood(min(wide), wide)
        players = []
        for kind, x, y, width, height, fields in self.entities:
            cells = rectangle(x, y, x + width, y + height)
            require(cells <= self.free, f"{kind} dans un mur à {x},{y}")
            if kind in ("PlayerSpawn", "WeaponLocation", "SodaLocation", "CharacterSpawn"):
                require((x, y) in wide_reached, f"{kind} sans accès ≥3 cases à {x},{y}")
            if kind.startswith("Door"):
                require(bool(cells & wide_reached), f"prise {x},{y} sans accès large")
            if kind == "PlayerSpawn":
                players.append((x, y))
        require(len(players) == (4 if self.kind == "depart" else 0), "départs coop incorrects")
        for i, point in enumerate(players):
            require(all(abs(point[0] - other[0]) + abs(point[1] - other[1]) >= 3
                        for other in players[i + 1:]), "départs trop proches")
        for inside, window, approach, spawn in self.courts:
            require(approach in reached, "fenêtre sans contact côté joueur")
            require(rectangle(spawn[0] - 2, spawn[1] - 2,
                              spawn[0] + 3, spawn[1] + 3) <= inside,
                    "dégagement du spawner inférieur à deux cases")
            require(flood(spawn, inside | window | reached) >= reached,
                    "zombie dépendant d'une porte payante")
            for x, y in inside:
                for adjacent in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    require(adjacent not in reached, "courette contournable sans fenêtre")
        bosses = [(x, y) for kind, x, y, *_ in self.entities if kind == "CharacterSpawn"]
        require(len(bosses) == (1 if self.kind == "boss" else 0), "boss/gardes inattendus")
        # Chaque centre de case de la prise est masqué, pas seulement son axe.
        for boss_x, boss_y in bosses:
            for entry_x, entry_y in self.connections:
                distance = max(abs(boss_x - entry_x), abs(boss_y - entry_y)) * 8
                ray = {(int(entry_x + .5 + (boss_x - entry_x) * step / distance),
                        int(entry_y + .5 + (boss_y - entry_y) * step / distance))
                       for step in range(distance + 1)}
                require(bool(ray - self.free), "boss visible depuis la prise")


def layouts():
    cardinal = [("N", 14), ("S", 14), ("W", 14), ("E", 14)]
    sections = []

    s = Section("Accueil_Comptoir", "depart", (32, 32), [("W", 14), ("E", 14)],
                "Hall est-ouest ; comptoir décentré, repli sud et pression opposée. Pistolet près de la sortie est.")
    s.carve((2, 10, 30, 23), (9, 4, 23, 29)).block((12, 11, 17, 17))
    s.court(2, 2, "E").court(23, 23, "W")
    s.players([(11, 20), (14, 20), (17, 20), (20, 20)]).weapon(26, 19, "pistol", 300)
    sections.append(s)

    s = Section("Accueil_Consigne", "depart", (32, 32), [("N", 14), ("S", 14)],
                "Hall nord-sud décalé ; consigne et casiers latéraux, traverse centrale libre pour quatre joueurs.")
    s.carve((10, 2, 23, 30), (5, 9, 27, 23)).block((5, 12, 9, 18), (15, 10, 19, 16))
    s.court(3, 3, "E").court(22, 22, "W")
    s.players([(11, 19), (14, 19), (17, 19), (20, 19)]).weapon(18, 26, "pistol", 300)
    sections.append(s)

    s = Section("Atelier_Etablis", "atelier", (32, 32), cardinal,
                "Long établi nord-sud ; voies ouest/est de largeurs différentes, achat exposé au bout sud-est.")
    s.carve((2, 2, 30, 30)).block((13, 10, 18, 23))
    s.court(2, 2, "E").court(23, 23, "W").weapon(20, 22, "machine_gun", 750)
    sections.append(s)

    s = Section("Stockage_Serpentin", "atelier", (32, 32), cardinal,
                "Deux rangées décalées dessinent un S ; cartouches dans l'angle est, fenêtres décalées nord-ouest/sud-est.")
    s.carve((2, 2, 30, 30)).block((10, 7, 21, 11), (12, 20, 24, 24))
    s.court(2, 2, "S").court(23, 23, "N").weapon(26, 19, "shotgun", 600)
    sections.append(s)

    s = Section("Galerie_Transversale", "atelier", (32, 32), [("W", 14), ("E", 14)],
                "Galerie étroite est-ouest, deux dents de couverture ; courettes perpendiculaires au nord et au sud.")
    s.carve((0, 11, 32, 22), (8, 7, 13, 25), (20, 7, 25, 25),
            (5, 9, 12, 14), (21, 19, 28, 23)).block((13, 12, 15, 15), (19, 18, 21, 20))
    s.court(3, 2, "S").court(22, 23, "N")
    sections.append(s)

    s = Section("Vestiaires_Coude", "atelier", (32, 32), [("N", 14), ("S", 14), ("E", 14)],
                "Coude autour des casiers ; traverse secondaire ouest, poche sud-est menacée par sa propre fenêtre.")
    s.carve((10, 2, 23, 29), (10, 16, 30, 29), (3, 9, 18, 22)).block((14, 12, 20, 20))
    s.court(2, 2, "S").court(23, 23, "W")
    sections.append(s)

    s = Section("Poste_Aiguillage", "atelier", (32, 32), cardinal,
                "Carrefour asymétrique sans achat ; deux masses courtes décalées coupent les diagonales NW/SE.")
    s.carve((2, 7, 30, 26), (8, 2, 25, 30)).block((12, 8, 17, 13), (18, 19, 23, 24))
    s.court(2, 2, "E").court(23, 23, "W")
    sections.append(s)

    s = Section("Reserve_Traversante", "atelier", (32, 32), cardinal,
                "Petites caisses et large voie de fuite centrale ; trois arrivées distinctes, ravitaillement à l'est.")
    s.carve((2, 2, 30, 30)).block((12, 9, 14, 11), (18, 12, 20, 14),
                                       (10, 19, 12, 21), (22, 20, 24, 22))
    s.court(2, 2, "E").court(23, 2, "S").court(2, 23, "E").weapon(25, 15, "machine_gun", 750)
    sections.append(s)

    s = Section("Cour_Chargement", "arene", (64, 32),
                [("W", 14), ("E", 14), ("N", 14), ("S", 14)],
                "Grande boucle libre autour du quai allongé ; quatre courettes, perk et munitions sur des bords opposés. Aucun verrou de combat.")
    s.carve((2, 2, 62, 30)).block((23, 10, 42, 22))
    s.court(2, 2, "E").court(55, 2, "W").court(2, 23, "E").court(55, 23, "W")
    s.perk(46, 25, "stamin_up").weapon(17, 6, "machine_gun", 1000)
    sections.append(s)

    s = Section("Halle_Convoyeur", "arene", (32, 64),
                [("N", 14), ("S", 14), ("W", 14), ("E", 14)],
                "Deux convoyeurs disjoints, boucle longue et coupe médiane ; speed cola à l'est, shotgun à l'ouest, quatre fenêtres. Aucun verrou de combat.")
    s.carve((2, 2, 30, 62)).block((11, 13, 20, 26), (12, 39, 21, 52))
    s.court(2, 2, "E").court(23, 2, "W").court(2, 55, "E").court(23, 55, "W")
    s.perk(25, 34, "speed_cola").weapon(6, 32, "shotgun", 800)
    sections.append(s)

    s = Section("Armurerie_Guichet", "armurerie", (32, 32), [("W", 14), ("E", 14)],
                "Comptoir en U ouvert au nord : rifle au guichet intérieur, deux contournements et deux fenêtres couvrant l'achat.")
    s.carve((2, 8, 30, 26), (9, 3, 23, 29)).block((12, 11, 14, 21), (14, 19, 21, 21), (19, 11, 21, 19))
    s.court(2, 2, "S").court(23, 23, "N")
    s.weapon(16, 14, "rifle", 1000).weapon(25, 16, "machine_gun", 750)
    sections.append(s)

    s = Section("Armurerie_Magasin", "armurerie", (32, 32), [("N", 14), ("S", 14)],
                "Dépôt nord-sud aux rayonnages courts alternés ; rifle au fond est, shotgun dans la poche sud, fenêtres opposées.")
    s.carve((4, 2, 28, 30), (2, 10, 30, 23)).block((11, 9, 15, 12), (19, 13, 23, 16), (10, 20, 14, 23))
    s.court(2, 2, "E").court(23, 23, "W")
    s.weapon(25, 20, "rifle", 1000).weapon(18, 26, "shotgun", 600)
    sections.append(s)

    s = Section("Infirmerie_Annexe", "infirmerie", (32, 32), [("W", 14)],
                "Impasse à retour large ; caisse contournable, juggernog en alcôve sud et fenêtre derrière l'acheteur.")
    s.carve((0, 11, 28, 24), (10, 5, 28, 28), (5, 9, 15, 14))
    s.block((13, 14, 17, 18), (10, 24, 16, 28), (24, 24, 28, 28))
    s.court(3, 2, "S").court(23, 23, "N").perk(20, 24, "juggernog")
    sections.append(s)

    s = Section("Infirmerie_Dispensaire", "infirmerie", (32, 32), [("E", 14)],
                "Dispensaire en baïonnette, mobilier en deux blocs ; juggernog au sud-ouest, pression latérale asymétrique.")
    s.carve((4, 8, 32, 22), (4, 16, 24, 29), (10, 3, 24, 16), (9, 4, 14, 10))
    s.block((10, 11, 16, 15), (15, 20, 19, 24))
    s.court(2, 2, "E").court(23, 23, "W").perk(7, 25, "juggernog")
    sections.append(s)

    s = Section("Four_Finale", "boss", (64, 32), [("W", 14)],
                "Sas ouest en chicane derrière cloison opaque ; grand four central contournable, boss au fond et seulement deux courettes. Munitions à l'intérieur du sas.")
    s.carve((0, 11, 14, 23), (7, 4, 60, 28)).block((10, 9, 14, 23), (30, 10, 40, 21))
    s.court(17, 2, "S").court(54, 23, "N").weapon(9, 25, "machine_gun", 1000)
    s.entity("CharacterSpawn", 54, 16, character="boss_zombie", team="enemy")
    sections.append(s)

    s = Section("Presse_Finale", "boss", (32, 64), [("N", 14)],
                "Sas nord masqué par cloison transversale ; deux presses indépendantes, boss au sud, deux fenêtres et ravitaillement à l'est.")
    s.carve((8, 0, 24, 15), (3, 8, 29, 60)).block((9, 12, 23, 16), (7, 23, 14, 33), (18, 38, 25, 47))
    s.court(2, 15, "E").court(23, 52, "W").weapon(25, 17, "shotgun", 800)
    s.entity("CharacterSpawn", 16, 56, character="boss_zombie", team="enemy")
    sections.append(s)

    for section in sections:
        section.finish_ports()
    return sections


def field_instance(definition, value):
    return {"__identifier": definition["identifier"], "__type": definition["__type"],
            "__value": value, "__tile": None, "defUid": definition["uid"],
            "realEditorValues": []}


def tile(x, y, tile_id, rule=None, cell_id=None):
    result = {"px": [x * CELL, y * CELL],
              "src": [(tile_id % 10) * CELL, (tile_id // 10) * CELL],
              "f": 0, "t": tile_id, "d": [cell_id] if rule is None else [rule, cell_id], "a": 1}
    return result


def make_project():
    project = json.loads(SOURCE.read_text(encoding="utf-8"))
    character_project = json.loads(CHARACTERS.read_text(encoding="utf-8"))
    project = deepcopy(project)
    definitions = project["defs"]
    uid = 200

    def allocate():
        nonlocal uid
        uid += 1
        return uid

    string_template = next(d for d in definitions["entities"] if d["identifier"] == "WeaponLocation")["fieldDefs"][0]
    int_template = next(d for d in definitions["entities"] if d["identifier"] == "WeaponLocation")["fieldDefs"][1]

    def field(name, kind, nullable=False, doc=None):
        result = deepcopy(string_template if kind == "String" else int_template)
        result.update(identifier=name, uid=allocate(), __type=kind, type="F_" + kind,
                      canBeNull=nullable, doc=doc, defaultOverride=None)
        return result

    definitions["levelFields"].extend([
        field("room_kind", "String", True, "Type de salle de la grammaire ; seul boss verrouille le combat."),
        field("design", "String", False, "Intention de géométrie, de pression et de récompense."),
    ])
    character = deepcopy(next(d for d in character_project["defs"]["entities"] if d["identifier"] == "CharacterSpawn"))
    character["uid"] = allocate()
    for definition in character["fieldDefs"]:
        definition["uid"] = allocate()
    definitions["entities"].append(character)
    entities = {d["identifier"]: d for d in definitions["entities"]}
    for name, width, height in (("DoorHorizontal", 64, 16), ("DoorVertical", 16, 64)):
        entities[name].update(width=width, height=height, resizableX=False, resizableY=False)
    for name in ("active_x", "active_y", "active_width", "active_height"):
        entities["ZombieSpawn"]["fieldDefs"].append(field(name, "Int", doc="Pixels LOCAUX au niveau ; convertis en monde par l'assembleur."))

    # Le sol est une couche Tiles dédiée : ses variantes sont une formule
    # stable de coordonnées. Les vides restent le fond sombre, pas de briques.
    wall = next(d for d in definitions["layers"] if d["identifier"] == "Walls")
    rule_template = deepcopy(wall["autoRuleGroups"][0]["rules"][0])
    floor = deepcopy(wall)
    floor.update(identifier="Sol", uid=allocate(), __type="Tiles", type="Tiles", intGridValues=[],
                 autoRuleGroups=[], tilesetDefUid=ATLAS, autoSourceLayerDefUid=None)
    masses = deepcopy(floor)
    masses.update(identifier="Mobilier", uid=allocate(), doc="Dessus des établis, rayonnages, presses et quais ; silhouettes dessinées dans Walls.")
    wall.update(tilesetDefUid=ATLAS, autoSourceLayerDefUid=None,
                doc="1 = toute région inaccessible. Auto-tuiles uniquement au contact du sol ; vide profond sans brique.")
    # Patterns LDtk : 1 exige Walls, -1 exclut Walls, 0 est indifférent.
    # Le même ordre de première correspondance est utilisé à l'export.
    rules = []
    for neighbor, tile_id in ((7, 1), (1, 41), (5, 10), (3, 15)):
        pattern = [0] * 9
        pattern[4], pattern[neighbor] = 1, -1
        rule = deepcopy(rule_template)
        rule.update(uid=allocate(), pattern=pattern, tileRectsIds=[[tile_id]],
                    flipX=False, flipY=False, outOfBoundsValue=1, perlinSeed=0)
        rules.append(rule)
    wall["autoRuleGroups"] = [{"uid": allocate(), "name": "Contours Fonderie", "color": "#7E657B",
                                "icon": None, "active": True, "isOptional": False, "rules": rules,
                                "biomeRequirementMode": 0, "requiredBiomeValues": [],
                                "usesWizard": False}]
    # Ordre LDtk du dessus vers le dessous.
    definitions["layers"].extend([masses, floor])
    definitions["layers"] = [next(d for d in definitions["layers"] if d["identifier"] == name)
                             for name in ("Entities", "LevelConnection", "Walls", "Mobilier", "Sol")]
    project.update(iid=stable("project"), nextUid=1000, worldLayout="Free",
                   worldGridWidth=512, worldGridHeight=512, defaultLevelWidth=512,
                   defaultLevelHeight=512, bgColor="#15131C", defaultLevelBgColor="#15131C",
                   minifyJson=False, toc=[], worlds=[], levels=[])
    template_level = json.loads(SOURCE.read_text(encoding="utf-8"))["levels"][0]
    template_layer = template_level["layerInstances"][0]
    fields = {d["identifier"]: d for d in definitions["levelFields"]}
    for number, section in enumerate(layouts()):
        section.validate()
        # Espacement du catalogue : aucune proximité visuelle n'est une prise.
        world_x, world_y = (number % 4) * 1280, (number // 4) * 1280
        level = deepcopy(template_level)
        level.update(identifier=section.name, iid=stable(section.name), uid=500 + number,
                     worldX=world_x, worldY=world_y, pxWid=section.width * CELL,
                     pxHei=section.height * CELL, useAutoIdentifier=False,
                     __bgColor="#15131C", __smartColor="#9B7653", bgColor="#15131C",
                     __neighbours=[], layerInstances=[], fieldInstances=[
                         field_instance(fields["spawn"], section.kind == "depart"),
                         field_instance(fields["room_kind"], section.kind),
                         field_instance(fields["design"], section.design)])
        for definition in definitions["layers"]:
            name = definition["identifier"]
            instance = deepcopy(template_layer)
            instance.update(__identifier=name, __type=definition["type"],
                            __cWid=section.width, __cHei=section.height,
                            iid=stable(section.name + "/" + name), levelId=level["uid"],
                            layerDefUid=definition["uid"], seed=0, intGridCsv=[],
                            entityInstances=[], autoLayerTiles=[], gridTiles=[],
                            __tilesetDefUid=definition["tilesetDefUid"],
                            __tilesetRelPath="../exemples/atlas/Dungeon_Tileset.png" if definition["tilesetDefUid"] == ATLAS else None)
            if name == "Entities":
                for index, (kind, x, y, width, height, values) in enumerate(section.entities):
                    entity_def = entities[kind]
                    instance["entityInstances"].append({
                        "__identifier": kind, "__grid": [x, y], "__pivot": [0, 0],
                        "__tags": entity_def["tags"], "__tile": entity_def["tileRect"],
                        "__smartColor": entity_def["color"],
                        "iid": stable(f"{section.name}/entity/{index}/{kind}"),
                        "width": width * CELL, "height": height * CELL, "defUid": entity_def["uid"],
                        "px": [x * CELL, y * CELL], "__worldX": world_x + x * CELL,
                        "__worldY": world_y + y * CELL,
                        "fieldInstances": [field_instance(f, values.get(f["identifier"]))
                                           for f in entity_def["fieldDefs"]],
                    })
            elif name == "LevelConnection":
                instance["intGridCsv"] = [int((x, y) in section.connections)
                                          for y in range(section.height) for x in range(section.width)]
            elif name == "Walls":
                instance["intGridCsv"] = [int((x, y) not in section.free)
                                          for y in range(section.height) for x in range(section.width)]
                for y in range(section.height):
                    for x in range(section.width):
                        if (x, y) in section.free:
                            continue
                        for rule, (dx, dy) in zip(rules, ((0, 1), (0, -1), (1, 0), (-1, 0))):
                            if (x + dx, y + dy) in section.free:
                                instance["autoLayerTiles"].append(tile(x, y, rule["tileRectsIds"][0][0], rule["uid"], x + y * section.width))
                                break
            elif name == "Sol":
                # Le sol ne couvre que l'intérieur : les cours extérieures restent
                # le vide de la carte (le « dehors » où les zombies se déplacent).
                palette = (6, 7, 8, 16, 17, 18, 26, 27, 28)
                for x, y in sorted(section.free - section.exterior, key=lambda p: (p[1], p[0])):
                    variant = palette[((x // 3) * 7 + (y // 2) * 11 + x * y) % len(palette)]
                    instance["gridTiles"].append(tile(x, y, variant, cell_id=x + y * section.width))
            elif name == "Mobilier":
                for x, y in sorted(section.masses - section.free, key=lambda p: (p[1], p[0])):
                    instance["gridTiles"].append(tile(x, y, 2 + ((x + y) % 3), cell_id=x + y * section.width))
            level["layerInstances"].append(instance)
        project["levels"].append(level)
    return project


def main():
    project = make_project()
    OUTPUT.write_text(json.dumps(project, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Écrit {OUTPUT.relative_to(ROOT)} : {len(project['levels'])} sections éditables.")


if __name__ == "__main__":
    main()
