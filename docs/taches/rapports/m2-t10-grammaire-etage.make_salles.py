"""M2-T10 : les neuf gabarits de salle de la grammaire d'étage de `gungeon`, à partir de l'étage
de M2-T0d (`games/gungeon/assets/maps/etage_1.ldtk`, dont on reprend les définitions, les champs
de niveau `spawn` / `room_kind` et les instances d'entités).

Toutes les salles font 8 × 8 tuiles (128 px) ; chaque ouverture fait 2 cases au milieu d'un côté
(cases 3-4) : toutes se raccordent entre elles, la grammaire choisit. Les ouvertures libres sont
murées à la génération (`GeneratedMap::close_unused_openings`).

    gabarit        ouvertures   contenu
    depart         N E S W      deux PlayerSpawn
    combat_ew      W E          2 bullet_kin
    combat_ns      N S          2 bullet_kin
    combat_t       W E S        3 bullet_kin
    combat_cross   N E S W      3 bullet_kin
    recompense     N E S W      vide (récompense : T3/T5)
    boutique       N E S W      vide (boutique : T6)
    boss_w         W            gatling au centre (cul-de-sac)
    boss_n         N            gatling au centre (cul-de-sac)

    python3 docs/taches/rapports/m2-t10-grammaire-etage.make_salles.py \\
        games/gungeon/assets/maps/etage_1.ldtk games/gungeon/assets/maps/salles.ldtk
"""
import copy, json, sys, uuid

src, dst = sys.argv[1], sys.argv[2]
d = json.load(open(src))
depart, combat, boss = (next(l for l in d['levels'] if l['identifier'] == n)
                        for n in ('Depart', 'Combat', 'Boss'))
defs = {e['identifier']: e for e in d['defs']['entities']}
W = 8
SIDES = {'N': [(3, 0), (4, 0)], 'S': [(3, 7), (4, 7)], 'W': [(0, 3), (0, 4)], 'E': [(7, 3), (7, 4)]}
DOOR = {'N': ('DoorHorizontal', (48, 0)), 'S': ('DoorHorizontal', (48, 112)),
        'W': ('DoorVertical', (0, 48)), 'E': ('DoorVertical', (112, 48))}


def layer(level, name):
    return next(li for li in level['layerInstances'] if li['__identifier'] == name)


def proto(level, ident):
    return next(e for li in level['layerInstances'] for e in li['entityInstances']
                if e['__identifier'] == ident)


def entity(base, ident, px, **fields):
    e = copy.deepcopy(base)
    definition = defs[ident]
    e.update(iid=str(uuid.uuid4()), defUid=definition['uid'], width=definition['width'],
             height=definition['height'], px=list(px))
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


def make(ident, uid, kind, sides, spawn=False, enemies=0, character='bullet_kin', players=0):
    level = copy.deepcopy(combat)
    level.update(identifier=ident, iid=str(uuid.uuid4()), uid=uid)
    for f in level['fieldInstances']:
        if f['__identifier'] == 'spawn':
            f['__value'] = spawn
        if f['__identifier'] == 'room_kind':
            f['__value'] = kind
            f['realEditorValues'] = [{'id': 'V_String', 'params': [kind]}]
    walls, conn, ents = (layer(level, n) for n in ('Walls', 'LevelConnection', 'Entities'))
    for li in level['layerInstances']:
        li['iid'] = str(uuid.uuid4())
        li['levelId'] = uid
    walls['intGridCsv'] = [1 if x in (0, W - 1) or y in (0, W - 1) else 0
                           for y in range(W) for x in range(W)]
    conn['intGridCsv'] = [0] * (W * W)
    door_base, spawn_base = proto(combat, 'DoorVertical'), proto(combat, 'CharacterSpawn')
    player_base = proto(depart, 'PlayerSpawn')
    out = []
    for side in sides:
        for x, y in SIDES[side]:
            walls['intGridCsv'][y * W + x] = 0
            conn['intGridCsv'][y * W + x] = 1
        name, px = DOOR[side]
        out.append(entity(door_base, name, px))
    spots = [(32, 32), (80, 32), (32, 80), (80, 80)]
    if character == 'gatling':
        out.append(entity(spawn_base, 'CharacterSpawn', (64, 48), character='gatling'))
    else:
        for px in spots[:enemies]:
            out.append(entity(spawn_base, 'CharacterSpawn', px, character=character))
    for i in range(players):
        e = entity(player_base, 'PlayerSpawn', (48 + 32 * i, 64))
        for f in e['fieldInstances']:
            if f['__identifier'] == 'index':
                f['__value'] = i
        out.append(e)
    ents['entityInstances'] = out
    return level


levels = [
    make('Depart', 0, 'depart', 'NESW', spawn=True, players=2),
    make('CombatEW', 1, 'combat', 'WE', enemies=2),
    make('CombatNS', 2, 'combat', 'NS', enemies=2),
    make('CombatT', 3, 'combat', 'WES', enemies=3),
    make('CombatCroix', 4, 'combat', 'NESW', enemies=3),
    make('Recompense', 5, 'recompense', 'NESW'),
    make('Boutique', 6, 'boutique', 'NESW'),
    make('BossW', 7, 'boss', 'W', character='gatling'),
    make('BossN', 8, 'boss', 'N', character='gatling'),
]
d['levels'] = levels
json.dump(d, open(dst, 'w'), indent=1)
print('ok', [l['identifier'] for l in levels])
