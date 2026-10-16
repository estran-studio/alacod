"""M3-T5 : gabarits de salle de `isaac` pour la disposition en grille (§70), à partir de l'étage de
M2-T0d (`games/gungeon/assets/maps/etage_1.ldtk`, dont on reprend les définitions, les champs de
niveau `spawn` / `room_kind` et les instances d'entités), sur le modèle de
`m4-t0f-squelette-isaac.make_salles.py` (gabarits 1x1 de M3-T0f, gardé comme trace).

Une case de salle fait 15 x 9 cases de 16 px (240 x 144 px, Q8). Un gabarit couvre `w x h` cases
de salle (1x1, 2x1, 1x2, 2x2) ; **chaque côté de chaque case de salle** a une ouverture de 3 cases
centrée sur son segment (colonnes 6-8 de la case au nord et au sud, rangées 3-5 à l'ouest et à
l'est) : toutes se raccordent entre elles, la grille choisit lesquelles sont des portes et mure
les autres à la génération (`GeneratedMap::close_unused_openings_exact`). Un cul-de-sac n'a donc
plus besoin d'un gabarit à une seule ouverture (les quatre `BossN/E/S/W` de M3-T0f disparaissent).

    gabarit        cases  type        contenu
    Depart         1x1    depart      deux PlayerSpawn
    Combat         1x1    combat      un suiveur et une mouche
    Boss           1x1    boss        gardien au centre
    BossGrand      2x2    boss        gardien au centre
    Tresor         1x1    tresor      (coffre posé par le type de salle)
    TresorLong     2x1    tresor
    TresorHaut     1x2    tresor
    Secrete        1x1    secrete     (coffre posé par le type de salle)
    Diable         1x1    diable      vide (le contenu est T8)
    Transit        1x1    transit     vide, sans ennemi : un étage « de passage » (`sous_sol_transit`)
                                      dont le portail s'ouvre aussitôt, pour atteindre le second étage
                                      d'une séquence sans jouer le premier (scénarios du trésor à clé)

    python3 docs/taches/rapports/m3-t5-grammaire-grille.make_salles.py \\
        games/gungeon/assets/maps/etage_1.ldtk games/isaac/assets/maps/salles.ldtk
"""
import copy, json, sys, uuid

src, dst = sys.argv[1], sys.argv[2]
NS = uuid.UUID('5e1d0b6e-0000-4000-8000-00000000f0f0')
d = json.load(open(src))
depart, combat = (next(l for l in d['levels'] if l['identifier'] == n) for n in ('Depart', 'Combat'))
defs = {e['identifier']: e for e in d['defs']['entities']}
CELL_COLS, CELL_ROWS = 15, 9
OPEN_COLS, OPEN_ROWS = (6, 7, 8), (3, 4, 5)


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


def make(ident, uid, kind, size=(1, 1), spawn=False, enemies=(), players=0):
    cw, ch = size
    cols, rows = CELL_COLS * cw, CELL_ROWS * ch
    level = copy.deepcopy(combat)
    level.update(identifier=ident, iid=iid(ident), uid=uid, pxWid=cols * 16, pxHei=rows * 16)
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
        li['__cWid'], li['__cHei'] = cols, rows
    walls['intGridCsv'] = [1 if x in (0, cols - 1) or y in (0, rows - 1) else 0
                           for y in range(rows) for x in range(cols)]
    conn['intGridCsv'] = [0] * (cols * rows)
    door_base, spawn_base = proto(combat, 'DoorVertical'), proto(combat, 'CharacterSpawn')
    player_base = proto(depart, 'PlayerSpawn')
    out = []

    def open_cell(x, y):
        walls['intGridCsv'][y * cols + x] = 0
        conn['intGridCsv'][y * cols + x] = 1

    for k in range(cw):  # nord et sud : une ouverture par case de salle
        for x in OPEN_COLS:
            open_cell(k * CELL_COLS + x, 0)
            open_cell(k * CELL_COLS + x, rows - 1)
        out.append(entity(door_base, ident, 'DoorHorizontal', (k * 240 + 96, 0), (48, 16)))
        out.append(entity(door_base, ident, 'DoorHorizontal', (k * 240 + 96, rows * 16 - 16), (48, 16)))
    for k in range(ch):  # ouest et est
        for y in OPEN_ROWS:
            open_cell(0, k * CELL_ROWS + y)
            open_cell(cols - 1, k * CELL_ROWS + y)
        out.append(entity(door_base, ident, 'DoorVertical', (0, k * 144 + 48), (16, 48)))
        out.append(entity(door_base, ident, 'DoorVertical', (cols * 16 - 16, k * 144 + 48), (16, 48)))
    centre = (cols * 8 - 8, rows * 8 - 8)
    for character, at in enemies:
        out.append(entity(spawn_base, ident, 'CharacterSpawn', at or centre, character=character))
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
    make('Combat', 1, 'combat', enemies=[('suiveur', (48, 32)), ('mouche', (176, 96))]),
    make('Boss', 2, 'boss', enemies=[('gardien', None)]),
    make('BossGrand', 3, 'boss', size=(2, 2), enemies=[('gardien', None)]),
    make('Tresor', 4, 'tresor'),
    make('TresorLong', 5, 'tresor', size=(2, 1)),
    make('TresorHaut', 6, 'tresor', size=(1, 2)),
    make('Secrete', 7, 'secrete'),
    make('Diable', 8, 'diable'),
    make('Transit', 9, 'transit'),
]
d['levels'] = levels
d['iid'] = iid('project')
json.dump(d, open(dst, 'w'), indent=1)
print('ok', [l['identifier'] for l in levels])
