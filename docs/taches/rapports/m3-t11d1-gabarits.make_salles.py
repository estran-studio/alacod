"""M3-T11d1 : les vingt gabarits de salle de `isaac` (`maps/salles.ldtk`, disposition en grille §70), à partir de
l'étage de M2-T0d (`games/gungeon/assets/maps/etage_1.ldtk`, dont on reprend les définitions, les champs de niveau
`spawn` / `room_kind` et les instances d'entités). Reprend `m3-t5-grammaire-grille.make_salles.py` (dix gabarits,
conservés tels quels, T11c y avait posé `tourelle_croix` dans `Combat`) et ajoute dix gabarits, la couche IntGrid
`Rocks` (1 = rocher, 2 = rocher à butin, §62) et la couche `Surfaces` (valeur 5 = piques, `surfaces/piques.ron`).

Une case de salle fait 15 x 9 cases de 16 px (240 x 144 px, Q8). Un gabarit couvre `w x h` cases de salle ;
**chaque côté de chaque case de salle** a une ouverture de 3 cases centrée sur son segment (colonnes 6-8 de la
case au nord et au sud, rangées 3-5 à l'ouest et à l'est) : la grille choisit lesquelles sont des portes.

    gabarit         cases  type     contenu
    Depart          1x1    depart   deux PlayerSpawn
    Combat          1x1    combat   suiveur, mouche, tourelle_croix (T0f, T11c : inchangé)
    CombatRochers   1x1    combat   un amas de rochers dans chaque angle, suiveur, mouche, tourelle_croix au centre
    CombatButin     1x1    combat   deux angles de rochers dont un rocher à butin, suiveur, mouche, cracheur
    CombatPiques    1x1    combat   deux angles de piques, suiveur, suiveur, bourdon
    CombatMixte     1x1    combat   rochers à butin, rochers et piques dans les quatre angles, suiveur, rampant, mouche
    CombatCroix     1x1    combat   deux angles de rochers, tourelle_croix, cracheur, mouche
    CombatLong      2x1    combat   deux angles de rochers à butin, deux de piques, quatre ennemis
    CombatHaut      1x2    combat   trois angles de rochers (deux à butin), un angle de piques, quatre ennemis
    CombatGrand     2x2    combat   quatre angles de rochers (deux à butin), quatre ennemis
    Boss            1x1    boss     gardien au centre (inchangé)
    BossGrand       2x2    boss     gardien au centre (inchangé)
    BossAbris       1x1    boss     gardien au centre, un amas de rochers dans chaque angle
    Tresor          1x1    tresor   (coffre posé par le type de salle) (inchangé)
    TresorLong      2x1    tresor   (inchangé)
    TresorHaut      1x2    tresor   (inchangé)
    Secrete         1x1    secrete  (coffre posé par le type de salle) (inchangé)
    Diable          1x1    diable   (présentoirs posés par le type de salle) (inchangé)
    Boutique        1x1    boutique (présentoirs posés par le type de salle)
    Transit         1x1    transit  vide, sans ennemi : l'étage de passage `sous_sol_transit` (inchangé)

**Rochers et piques dans les angles.** Le bot `explorateur` ne voit pas les rochers (ils n'ont pas le marqueur `Wall` que
`bots::input` lit pour son maillage) : il marche en ligne droite et reste collé à un rocher posé sur son chemin (premier
essai, rochers au centre : un étage sans progrès après 700 frames). Les gabarits ne posent donc rochers et piques que
**dans les angles**, hors de toute ligne droite entre deux ouvertures, entre une ouverture et un ennemi ou le centre ;
casser un rocher n'est jamais nécessaire pour traverser une salle (la bombe ouvre le butin, pas le passage). Apprendre
les rochers au bot est M3-T14.

Le script **vérifie** chaque gabarit : aucun rocher ni pique sur un mur, une ouverture, une case devant une ouverture
(deux cases de profondeur) ni sous un ennemi ; les ouvertures et les ennemis sont tous reliés par des cases libres
**larges** (un bloc libre de 2 x 2 : le collider de 20 px d'un joueur ou d'un zombie ne passe pas dans un couloir
d'une case) ; aucun rocher à moins d'une case (piques : trois quarts de case) d'un segment qui joint deux ouvertures, une
ouverture et un ennemi, ou une ouverture et le centre.

    python3 docs/taches/rapports/m3-t11d1-gabarits.make_salles.py \\
        games/gungeon/assets/maps/etage_1.ldtk games/isaac/assets/maps/salles.ldtk [--ascii]
"""
import copy, json, sys, uuid

src, dst = sys.argv[1], sys.argv[2]
NS = uuid.UUID('5e1d0b6e-0000-4000-8000-00000000f0f0')
d = json.load(open(src))
depart, combat = (next(l for l in d['levels'] if l['identifier'] == n) for n in ('Depart', 'Combat'))
defs = {e['identifier']: e for e in d['defs']['entities']}
CELL_COLS, CELL_ROWS = 15, 9
OPEN_COLS, OPEN_ROWS = (6, 7, 8), (3, 4, 5)
ROCK, ROCK_LOOT, PIKE = 1, 2, 5


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


# Couches de définition : copie de `LevelConnection` (IntGrid sans tuile), comme M2-T14d pour `Surfaces`.
LAYER_MODEL = next(l for l in d['defs']['layers'] if l['identifier'] == 'LevelConnection')
rocks_uid, surfaces_uid = d['nextUid'], d['nextUid'] + 1
d['nextUid'] += 2
for ident, uid, values in (
    ('Rocks', rocks_uid, [(ROCK, 'rocher', '#8A6A4A'), (ROCK_LOOT, 'rocher_butin', '#C9A227')]),
    ('Surfaces', surfaces_uid, [(PIKE, 'piques', '#9A9A9A')]),
):
    definition = copy.deepcopy(LAYER_MODEL)
    definition.update(identifier=ident, uid=uid)
    definition['intGridValues'] = [{'value': v, 'identifier': n, 'color': c, 'tile': None, 'groupUid': 0}
                                   for v, n, c in values]
    d['defs']['layers'].append(definition)
LAYER_INSTANCE_MODEL = layer(combat, 'LevelConnection')


def extra_layer(level, ident, def_uid, cells, cols, rows):
    """Couche IntGrid `ident` du niveau : `cells` = {(col, row): valeur}, insérée avant `LevelConnection`."""
    inst = copy.deepcopy(LAYER_INSTANCE_MODEL)
    inst.update({'__identifier': ident, 'layerDefUid': def_uid, 'iid': iid(level['identifier'], ident),
                 'levelId': level['uid'], '__cWid': cols, '__cHei': rows})
    inst['intGridCsv'] = [cells.get((x, y), 0) for y in range(rows) for x in range(cols)]
    index = next(i for i, li in enumerate(level['layerInstances']) if li['__identifier'] == 'LevelConnection')
    level['layerInstances'].insert(index, inst)


def check(ident, cols, rows, cw, ch, enemies, rocks, pikes):
    """Contrôle d'un gabarit (voir l'en-tête) ; lève AssertionError avec le gabarit et la case fautive."""
    wall = {(x, y) for x in range(cols) for y in range(rows) if x in (0, cols - 1) or y in (0, rows - 1)}
    openings, front = set(), set()
    for k in range(cw):
        for x in OPEN_COLS:
            for y, dy in ((0, 1), (rows - 1, -1)):
                openings.add((k * CELL_COLS + x, y))
                front |= {(k * CELL_COLS + x, y + dy), (k * CELL_COLS + x, y + 2 * dy)}
    for k in range(ch):
        for y in OPEN_ROWS:
            for x, dx in ((0, 1), (cols - 1, -1)):
                openings.add((x, k * CELL_ROWS + y))
                front |= {(x + dx, k * CELL_ROWS + y), (x + 2 * dx, k * CELL_ROWS + y)}
    wall -= openings
    blocked = set(rocks) | wall
    # intérieur seulement : jamais sur un mur, une ouverture ou devant une ouverture
    for cell in list(rocks) + list(pikes):
        assert 1 <= cell[0] < cols - 1 and 1 <= cell[1] < rows - 1, f'{ident} : {cell} sur le bord'
        assert cell not in front, f'{ident} : {cell} devant une ouverture'
    for cell in pikes:
        assert cell not in rocks, f'{ident} : piques {cell} sous un rocher'
    for name, (x, y) in enemies:
        assert (x, y) not in blocked and (x, y) not in pikes, f'{ident} : {name} {(x, y)} sur un obstacle'
        assert 1 <= x < cols - 1 and 1 <= y < rows - 1, f'{ident} : {name} {(x, y)} sur le bord'

    def free(x, y):
        return 0 <= x < cols and 0 <= y < rows and (x, y) not in blocked

    wide = {(x, y) for x in range(cols) for y in range(rows)
            if any(all(free(x + dx + i, y + dy + j) for i in (0, 1) for j in (0, 1))
                   for dx in (0, -1) for dy in (0, -1)) and free(x, y)}
    start = next(iter(sorted(openings)))
    seen, todo = {start}, [start]
    while todo:
        x, y = todo.pop()
        for n in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if n in wide and n not in seen:
                seen.add(n)
                todo.append(n)
    for cell in sorted(openings):
        assert cell in seen, f'{ident} : ouverture {cell} non reliée'
    for name, cell in enemies:
        assert cell in seen, f'{ident} : {name} {cell} non relié'
    return blocked, openings


def _segment_distance(p, q, c):
    """Distance du point c au segment pq (cases, flottants)."""
    (px, py), (qx, qy), (cx, cy) = p, q, c
    dx, dy = qx - px, qy - py
    t = 0 if dx == dy == 0 else max(0, min(1, ((cx - px) * dx + (cy - py) * dy) / (dx * dx + dy * dy)))
    return ((px + t * dx - cx) ** 2 + (py + t * dy - cy) ** 2) ** 0.5


def check_lines(ident, cols, rows, cw, ch, enemies, rocks, pikes):
    """Les rochers et les piques restent hors des lignes droites qu'un bot suit (voir l'en-tête)."""
    points = []
    for k in range(cw):
        points += [(k * CELL_COLS + 7.5, 0.5), (k * CELL_COLS + 7.5, rows - 0.5)]
    for k in range(ch):
        points += [(0.5, k * CELL_ROWS + 4.5), (cols - 0.5, k * CELL_ROWS + 4.5)]
    ends = list(points) + [(x + 0.5, y + 0.5) for _, (x, y) in enemies] + [(cols / 2, rows / 2)]
    for a in points:
        for b in ends:
            if a == b:
                continue
            for cell in rocks:
                gap = _segment_distance(a, b, (cell[0] + 0.5, cell[1] + 0.5))
                assert gap >= 1.0, f'{ident} : rocher {cell} à {gap:.2f} case(s) de la ligne {a} -> {b}'
            for cell in pikes:
                gap = _segment_distance(a, b, (cell[0] + 0.5, cell[1] + 0.5))
                assert gap >= 0.75, f'{ident} : piques {cell} à {gap:.2f} case(s) de la ligne {a} -> {b}'


def ascii_room(ident, cols, rows, blocked, openings, enemies, rocks, pikes):
    names = {cell: n[0].upper() for n, cell in ((n, c) for n, c in enemies)}
    print(ident)
    for y in range(rows):
        row = ''
        for x in range(cols):
            c = (x, y)
            row += (names.get(c) or ('#' if c in blocked and c not in rocks else
                                     'L' if rocks.get(c) == ROCK_LOOT else 'o' if c in rocks else
                                     '^' if c in pikes else '+' if c in openings else '.'))
        print('  ' + row)


ALL = []


def make(ident, uid, kind, size=(1, 1), spawn=False, enemies=(), players=0, rocks=None, pikes=()):
    cw, ch = size
    cols, rows = CELL_COLS * cw, CELL_ROWS * ch
    rocks, pikes = dict(rocks or {}), set(pikes)
    grid_enemies = [(name, at) for name, at in enemies if at is not None]
    blocked, openings = check(ident, cols, rows, cw, ch, grid_enemies, rocks, pikes)
    check_lines(ident, cols, rows, cw, ch, grid_enemies, rocks, pikes)
    ALL.append((ident, cols, rows, blocked, openings, grid_enemies, rocks, pikes))
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
        e = entity(spawn_base, ident, 'CharacterSpawn', centre if at is None else (at[0] * 16, at[1] * 16),
                   character=character)
        if (ident, character) == ('Combat', 'tourelle_croix'):  # iid posé par M3-T11c (son script de placement)
            e['iid'] = str(uuid.uuid5(uuid.NAMESPACE_URL, 'alacod/Combat/tourelle_croix/13/7'))
        out.append(e)
    for i in range(players):
        e = entity(player_base, ident, 'PlayerSpawn', (96 + 32 * i, 64))
        for f in e['fieldInstances']:
            if f['__identifier'] == 'index':
                f['__value'] = i
        out.append(e)
    ents['entityInstances'] = out
    if rocks:
        extra_layer(level, 'Rocks', rocks_uid, rocks, cols, rows)
    if pikes:
        extra_layer(level, 'Surfaces', surfaces_uid, {c: PIKE for c in pikes}, cols, rows)
    return level


def square(x, y, w=2, h=2):
    return {(x + i, y + j) for i in range(w) for j in range(h)}


def corner(cols, rows, name, loot=False, kind=ROCK):
    """Amas en L dans un angle (trois cases : le coin et ses deux voisines le long des murs), `loot` : le coin est un
    rocher à butin. `name` : NO, NE, SO, SE."""
    x = 1 if name[1] == 'O' else cols - 2
    y = 1 if name[0] == 'N' else rows - 2
    dx = 1 if name[1] == 'O' else -1
    dy = 1 if name[0] == 'N' else -1
    cells = {(x, y): ROCK_LOOT if loot else kind, (x + dx, y): kind, (x, y + dy): kind}
    return cells


def corners(cols, rows, plain=(), loot=(), pikes=()):
    """Rochers (`plain`), rochers à butin (`loot`) et piques (`pikes`) : les angles nommés, un amas en L chacun."""
    rocks = {}
    for name in plain:
        rocks.update(corner(cols, rows, name))
    for name in loot:
        rocks.update(corner(cols, rows, name, loot=True))
    spikes = set()
    for name in pikes:
        spikes |= set(corner(cols, rows, name))
    return rocks, spikes


C, R = CELL_COLS, CELL_ROWS
L = ROCK_LOOT
rocks_a, _ = corners(C, R, plain=('NO', 'NE', 'SO', 'SE'))
rocks_b, _ = corners(C, R, loot=('NO', 'SE'))
_, pikes_c = corners(C, R, pikes=('NO', 'SE'))
rocks_d, pikes_d = corners(C, R, plain=('SE',), loot=('NO',), pikes=('NE', 'SO'))
rocks_e, _ = corners(C, R, plain=('NE', 'SO'))
rocks_long, pikes_long = corners(2 * C, R, loot=('NO', 'SE'), pikes=('NE', 'SO'))
rocks_haut, pikes_haut = corners(C, 2 * R, plain=('NE',), loot=('NO', 'SO'), pikes=('SE',))
rocks_grand, pikes_grand = corners(2 * C, 2 * R, plain=('NE', 'SO'), loot=('NO', 'SE'))
rocks_abris, _ = corners(C, R, plain=('NO', 'NE', 'SO', 'SE'))
levels = [
    make('Depart', 0, 'depart', spawn=True, players=2),
    make('Combat', 1, 'combat', enemies=[('suiveur', (3, 2)), ('mouche', (11, 6)), ('tourelle_croix', (13, 7))]),
    make('Boss', 2, 'boss', enemies=[('gardien', None)]),
    make('BossGrand', 3, 'boss', size=(2, 2), enemies=[('gardien', None)]),
    make('Tresor', 4, 'tresor'),
    make('TresorLong', 5, 'tresor', size=(2, 1)),
    make('TresorHaut', 6, 'tresor', size=(1, 2)),
    make('Secrete', 7, 'secrete'),
    make('Diable', 8, 'diable'),
    make('Transit', 9, 'transit'),
    make('CombatRochers', 10, 'combat',
         enemies=[('suiveur', (11, 2)), ('mouche', (3, 6)), ('tourelle_croix', (7, 4))], rocks=rocks_a),
    make('CombatButin', 11, 'combat',
         enemies=[('suiveur', (11, 2)), ('mouche', (3, 6)), ('cracheur', (7, 4))], rocks=rocks_b),
    make('CombatPiques', 12, 'combat',
         enemies=[('suiveur', (11, 2)), ('suiveur', (3, 6)), ('bourdon', (7, 4))], pikes=pikes_c),
    make('CombatMixte', 13, 'combat',
         enemies=[('suiveur', (11, 3)), ('rampant', (4, 6)), ('mouche', (7, 4))], rocks=rocks_d, pikes=pikes_d),
    make('CombatCroix', 14, 'combat',
         enemies=[('tourelle_croix', (3, 2)), ('cracheur', (11, 6)), ('mouche', (11, 2))], rocks=rocks_e),
    make('CombatLong', 15, 'combat', size=(2, 1),
         enemies=[('suiveur', (4, 2)), ('suiveur', (25, 6)), ('mouche', (14, 3)), ('cracheur', (15, 5))],
         rocks=rocks_long, pikes=pikes_long),
    make('CombatHaut', 16, 'combat', size=(1, 2),
         enemies=[('suiveur', (11, 3)), ('mouche', (3, 14)), ('rampant', (3, 3)), ('tourelle_croix', (11, 14))],
         rocks=rocks_haut, pikes=pikes_haut),
    make('CombatGrand', 17, 'combat', size=(2, 2),
         enemies=[('suiveur', (4, 3)), ('mouche', (25, 14)), ('cracheur', (14, 9)), ('bourdon', (26, 3))],
         rocks=rocks_grand, pikes=pikes_grand),
    make('BossAbris', 18, 'boss', enemies=[('gardien', None)], rocks=rocks_abris),
    make('Boutique', 19, 'boutique'),
]
if '--ascii' in sys.argv:
    for item in ALL:
        ascii_room(*item[:1], *item[1:3], item[3], item[4], item[5], item[6], item[7])
d['levels'] = levels
d['iid'] = iid('project')
json.dump(d, open(dst, 'w'), indent=1)
print('ok', len(levels), [l['identifier'] for l in levels])
