"""X-6g : gabarits de l'intérieur du testbed (`games/testbed/assets/testbed/interieur.ldtk`), à partir de
`etage_sortie.ldtk` (définitions, champs de niveau `spawn` / `room_kind`) et de `porte_sortie.ldtk` (champs de porte
`lock_key` / `lock_consume`, X-6a1).

Toutes les salles font 12 × 10 cases (192 × 160 px) ; chaque ouverture fait 2 cases au milieu d'un côté (cases 5-6
au nord et au sud, 4-5 à l'ouest et à l'est) : tous les gabarits se raccordent, la grammaire choisit.

    gabarit           type (`room_kind`)  ouvertures   contenu
    Hall              depart              N E S W      deux PlayerSpawn (spawn: true)
    Combat_*          combat              couloirs, coudes, T, croix, vides
    Salon_*           salon               croix, couloirs, vides
    Atelier_*         atelier             croix, couloirs, vides
    Cellier_*         cellier             croix, couloirs, vides
    Cle_*             cle                 croix, couloirs   un `porteur_cle` (clé `cle_test`)
    Finale_N/E/S/W    finale              une seule      cul-de-sac ; sa porte est à serrure (`lock_key: cle_test`)

    python3 docs/taches/rapports/x-6g-interieur-graine.make_interieur.py games/testbed/assets/testbed
"""
import copy, json, sys, uuid

folder = sys.argv[1]
d = json.load(open(f'{folder}/etage_sortie.ldtk'))
door_src = json.load(open(f'{folder}/porte_sortie.ldtk'))
W, H = 12, 10
SIDES = {'N': [(5, 0), (6, 0)], 'S': [(5, H - 1), (6, H - 1)], 'W': [(0, 4), (0, 5)], 'E': [(W - 1, 4), (W - 1, 5)]}
DOOR = {'N': ('DoorHorizontal', (80, 0)), 'S': ('DoorHorizontal', (80, (H - 1) * 16)),
        'W': ('DoorVertical', (0, 64)), 'E': ('DoorVertical', ((W - 1) * 16, 64))}

# Les définitions de porte de `porte_sortie.ldtk` (avec `lock_key`, `lock_consume`) remplacent celles du gabarit.
src_defs = {e['identifier']: e for e in door_src['defs']['entities']}
for e in d['defs']['entities']:
    if e['identifier'] in ('DoorHorizontal', 'DoorVertical'):
        e['fieldDefs'] = copy.deepcopy(src_defs[e['identifier']]['fieldDefs'])
defs = {e['identifier']: e for e in d['defs']['entities']}
base_level = d['levels'][0]
player_base = next(e for li in base_level['layerInstances'] for e in li['entityInstances'] if e['__identifier'] == 'PlayerSpawn')
spawn_base = next(e for li in d['levels'][1]['layerInstances'] for e in li['entityInstances'])


def entity(base, ident, px, **fields):
    e = copy.deepcopy(base)
    definition = defs[ident]
    e.update(iid=str(uuid.uuid4()), defUid=definition['uid'], width=definition['width'], height=definition['height'],
             px=list(px))
    e['__identifier'] = ident
    e['__grid'] = [px[0] // 16, px[1] // 16]
    e['__worldX'], e['__worldY'] = px
    e['__tile'] = None
    values = {'price': 0, 'electrify': False, 'lock_consume': True, 'exit_next_level': False, 'lock_key': None}
    values.update(fields)
    out = []
    for f in definition['fieldDefs']:
        name, kind = f['identifier'], f['__type']
        if name in ('index', 'character', 'team') and name not in fields:
            continue
        value = values.get(name, fields.get(name))
        out.append({'__identifier': name, '__type': kind, '__value': value, '__tile': None, 'defUid': f['uid'],
                    'realEditorValues': [] if value is None else [{'id': 'V_' + kind, 'params': [value]}]})
    e['fieldInstances'] = out
    return e


def layer(level, name):
    return next(li for li in level['layerInstances'] if li['__identifier'] == name)


def make(ident, uid, kind, sides, spawn=False, players=0, carrier=False, locked_door=None):
    level = copy.deepcopy(base_level)
    level.update(identifier=ident, iid=str(uuid.uuid4()), uid=uid, pxWid=W * 16, pxHei=H * 16, worldX=uid * 256, worldY=0)
    for f in level['fieldInstances']:
        if f['__identifier'] == 'spawn':
            f['__value'] = spawn
            f['realEditorValues'] = [{'id': 'V_Bool', 'params': [spawn]}]
        if f['__identifier'] == 'room_kind':
            f['__value'] = kind
            f['realEditorValues'] = [{'id': 'V_String', 'params': [kind]}]
    for li in level['layerInstances']:
        li['iid'] = str(uuid.uuid4())
        li['levelId'] = uid
        li['__cWid'], li['__cHei'] = W, H
    walls, conn, ents = (layer(level, n) for n in ('Walls', 'LevelConnection', 'Entities'))
    walls['intGridCsv'] = [1 if x in (0, W - 1) or y in (0, H - 1) else 0 for y in range(H) for x in range(W)]
    conn['intGridCsv'] = [0] * (W * H)
    out = []
    for side in sides:
        for x, y in SIDES[side]:
            walls['intGridCsv'][y * W + x] = 0
            conn['intGridCsv'][y * W + x] = 1
        name, px = DOOR[side]
        fields = {'lock_key': 'cle_test'} if locked_door == side else {}
        out.append(entity(player_base, name, px, **fields))
    for i in range(players):
        out.append(entity(player_base, 'PlayerSpawn', (80 + 32 * i, 72), index=i))
    if carrier:
        out.append(entity(spawn_base, 'CharacterSpawn', (96, 96), character='porteur_cle'))
    ents['entityInstances'] = out
    return level


CROIX = 'NESW'
SHAPES = {'croix': CROIX, 'ew': 'EW', 'ns': 'NS'}
levels, uid = [make('Hall', 0, 'depart', CROIX, spawn=True, players=2)], 1
combat = {'croix': CROIX, 'ew': 'EW', 'ns': 'NS', 'ne': 'NE', 'es': 'ES', 'sw': 'SW', 'wn': 'WN',
          't_nes': 'NES', 't_esw': 'ESW', 't_swn': 'SWN', 't_wne': 'WNE'}
for shape, sides in combat.items():
    levels.append(make(f'Combat_{shape}', uid, 'combat', sides)); uid += 1
for kind in ('salon', 'atelier', 'cellier'):
    for shape, sides in SHAPES.items():
        levels.append(make(f'{kind.capitalize()}_{shape}', uid, kind, sides)); uid += 1
for shape, sides in SHAPES.items():
    levels.append(make(f'Cle_{shape}', uid, 'cle', sides, carrier=True)); uid += 1
for side in CROIX:
    levels.append(make(f'Finale_{side}', uid, 'finale', side, locked_door=side)); uid += 1
d['levels'] = levels
json.dump(d, open(f'{folder}/interieur.ldtk', 'w'), indent=1)
print('ok', len(levels), 'gabarits')
