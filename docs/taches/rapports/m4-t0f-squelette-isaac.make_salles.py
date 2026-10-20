"""M3-T0f : les trois gabarits de salle de la grammaire d'étage de `isaac`, à partir de l'étage de
M2-T0d (`games/gungeon/assets/maps/etage_1.ldtk`, dont on reprend les définitions, les champs de
niveau `spawn` / `room_kind` et les instances d'entités), sur le modèle de
`m2-t10-grammaire-etage.make_salles.py`.

Toutes les salles font 15 x 9 cases (240 x 144 px, Q8 : une seule taille, préparation de la grille de
M3-T5) ; chaque côté a une ouverture de 3 cases **centrée** (colonnes 6-8 au nord et au sud, rangées
3-5 à l'ouest et à l'est) : toutes se raccordent entre elles, la grammaire choisit. Les ouvertures
libres sont murées à la génération (`GeneratedMap::close_unused_openings`).

    gabarit   ouvertures   contenu
    Depart    N E S W      deux PlayerSpawn
    Combat    N E S W      un suiveur et une mouche
    BossN/E/S/W  une seule   gardien au centre (cul-de-sac : le planificateur en arbre exige un
                              gabarit de boss à une connexion ; la grille de M3-T5 lèvera cette
                              contrainte avec ses propres gabarits)

    python3 docs/taches/rapports/m4-t0f-squelette-isaac.make_salles.py \\
        games/gungeon/assets/maps/etage_1.ldtk games/isaac/assets/maps/salles.ldtk
"""
import copy, json, sys, uuid

src, dst = sys.argv[1], sys.argv[2]
NS = uuid.UUID('5e1d0b6e-0000-4000-8000-00000000f0f0')
d = json.load(open(src))
depart, combat, boss = (next(l for l in d['levels'] if l['identifier'] == n)
                        for n in ('Depart', 'Combat', 'Boss'))
defs = {e['identifier']: e for e in d['defs']['entities']}
COLS, ROWS = 15, 9
OPEN_COLS, OPEN_ROWS = (6, 7, 8), (3, 4, 5)
SIDES = {'N': [(x, 0) for x in OPEN_COLS], 'S': [(x, ROWS - 1) for x in OPEN_COLS],
         'W': [(0, y) for y in OPEN_ROWS], 'E': [(COLS - 1, y) for y in OPEN_ROWS]}
# (entité, px, largeur, hauteur) : 3 cases d'ouverture = 48 px
DOOR = {'N': ('DoorHorizontal', (96, 0), 48, 16), 'S': ('DoorHorizontal', (96, 128), 48, 16),
        'W': ('DoorVertical', (0, 48), 16, 48), 'E': ('DoorVertical', (224, 48), 16, 48)}


def iid(*parts):
    return str(uuid.uuid5(NS, '/'.join(map(str, parts))))


def layer(level, name):
    return next(li for li in level['layerInstances'] if li['__identifier'] == name)


def proto(level, ident):
    return next(e for li in level['layerInstances'] for e in li['entityInstances']
                if e['__identifier'] == ident)


def entity(base, level_name, ident, px, size=None, **fields):
    e = copy.deepcopy(base)
    definition = defs[ident]
    w, h = size or (definition['width'], definition['height'])
    e.update(iid=iid(level_name, ident, *px), defUid=definition['uid'], width=w, height=h,
             px=list(px))
    e['__identifier'] = ident
    e['__grid'] = [px[0] // 16, px[1] // 16]
    e['__worldX'], e['__worldY'] = px
    e['__tile'] = None
    by_name = {f['identifier']: f['uid'] for f in definition['fieldDefs']}
    for f in e['fieldInstances']:
        f['defUid'] = by_name.get(f['__identifier'], f['defUid'])
        if f['__identifier'] in fields:
            f['__value'] = fields[f['__identifier']]
            f['realEditorValues'] = [{'id': 'V_String', 'params': [fields[f['__identifier']]]}]
    return e


def make(ident, uid, kind, spawn=False, enemies=(), players=0, sides='NESW'):
    level = copy.deepcopy(combat)
    level.update(identifier=ident, iid=iid(ident), uid=uid, pxWid=COLS * 16, pxHei=ROWS * 16)
    for f in level['fieldInstances']:
        if f['__identifier'] == 'spawn':
            f['__value'] = spawn
        if f['__identifier'] == 'room_kind':
            f['__value'] = kind
            f['realEditorValues'] = [{'id': 'V_String', 'params': [kind]}]
    walls, conn, ents = (layer(level, n) for n in ('Walls', 'LevelConnection', 'Entities'))
    for li in level['layerInstances']:
        li['iid'] = iid(ident, li['__identifier'])
        li['levelId'] = uid
        li['__cWid'], li['__cHei'] = COLS, ROWS
    walls['intGridCsv'] = [1 if x in (0, COLS - 1) or y in (0, ROWS - 1) else 0
                           for y in range(ROWS) for x in range(COLS)]
    conn['intGridCsv'] = [0] * (COLS * ROWS)
    door_base, spawn_base = proto(combat, 'DoorVertical'), proto(combat, 'CharacterSpawn')
    player_base = proto(depart, 'PlayerSpawn')
    out = []
    for side in sides:
        for x, y in SIDES[side]:
            walls['intGridCsv'][y * COLS + x] = 0
            conn['intGridCsv'][y * COLS + x] = 1
        name, px, w, h = DOOR[side]
        out.append(entity(door_base, ident, name, px, (w, h)))
    for character, px in enemies:
        out.append(entity(spawn_base, ident, 'CharacterSpawn', px, character=character))
    for i in range(players):
        e = entity(player_base, ident, 'PlayerSpawn', (96 + 32 * i, 64))
        for f in e['fieldInstances']:
            if f['__identifier'] == 'index':
                f['__value'] = i
        out.append(e)
    ents['entityInstances'] = out
    return level


levels = [
    make('Depart', 0, 'depart', spawn=True, players=2),
    make('Combat', 1, 'combat',
         enemies=[('suiveur', (48, 32)), ('mouche', (176, 96))]),
    make('BossN', 2, 'boss', enemies=[('gardien', (112, 64))], sides='N'),
    make('BossE', 3, 'boss', enemies=[('gardien', (112, 64))], sides='E'),
    make('BossS', 4, 'boss', enemies=[('gardien', (112, 64))], sides='S'),
    make('BossW', 5, 'boss', enemies=[('gardien', (112, 64))], sides='W'),
]
d['levels'] = levels
d['iid'] = iid('project')
json.dump(d, open(dst, 'w'), indent=1)
print('ok', [l['identifier'] for l in levels])
