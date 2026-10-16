"""M2-T14d : trois gabarits de plus (quinze au total) dans `maps/salles.ldtk` de `gungeon`, et la couche
IntGrid `Surfaces` (§26) dans le projet.

    gabarit      type (`room_kind`)  ouvertures  contenu
    Coffre       coffre              N E S W     3 tables (props, §46, jamais devant une ouverture), sable autour du coffre
    Secret       secret              W           3 tables, eau et fosse aux angles (cul-de-sac)
    ArenaNS      arene               N S         2 barils et 1 table (les vagues viennent de rooms/arene.ron)

Le coffre de `coffre` et de `secret` est posé par le type de salle (`rooms/<id>.ron`, §48), pas par une
entité LDtk. Les ouvertures libres sont murées à la génération. Les gabarits existants ne bougent pas
(leur hash non plus) : seule la couche `Surfaces` est ajoutée à la liste des définitions.

    python3 docs/taches/rapports/m2-t14d-gabarits.make_gabarits.py \\
        games/gungeon/assets/maps/salles.ldtk games/gungeon/assets/maps/salles.ldtk

Idempotent : retire d'abord les gabarits qu'il ajoute, et la couche `Surfaces` si elle existe.
Les identifiants (iid) sont dérivés des noms (uuid5), le fichier ne change pas d'une exécution à l'autre.
"""
import copy, json, sys, uuid

SIDES = {'N': [(3, 0), (4, 0)], 'S': [(3, 7), (4, 7)], 'W': [(0, 3), (0, 4)], 'E': [(7, 3), (7, 4)]}
W = 8
NEW = ('Coffre', 'Secret', 'ArenaNS')
SURFACES = {'eau': 1, 'sable': 2, 'fosse': 4}  # pas de glace : aucun gabarit n'en pose


def iid(*parts):
    return str(uuid.uuid5(uuid.NAMESPACE_URL, 'alacod/gungeon/salles/' + '/'.join(map(str, parts))))


def layer(level, name):
    return next(li for li in level['layerInstances'] if li['__identifier'] == name)


def set_kind(level, kind):
    for f in level['fieldInstances']:
        if f['__identifier'] == 'room_kind':
            f['__value'] = kind
            f['realEditorValues'] = [{'id': 'V_String', 'params': [kind]}]


def prop(model, name, col, row, level_name):
    e = copy.deepcopy(model)
    e['iid'] = iid(level_name, name, col, row)
    e['__grid'] = [col, row]
    e['px'] = [col * 16, row * 16]
    e['__worldX'], e['__worldY'] = e['px']
    return e


def open_sides(level, sides, doors):
    """Ouvertures de `sides` (2 cases au milieu du côté) ; portes copiées de `doors` (nom -> modèle)."""
    walls, conn = layer(level, 'Walls'), layer(level, 'LevelConnection')
    walls['intGridCsv'] = [1 if x in (0, W - 1) or y in (0, W - 1) else 0 for y in range(W) for x in range(W)]
    conn['intGridCsv'] = [0] * (W * W)
    out = []
    for side in sides:
        for x, y in SIDES[side]:
            walls['intGridCsv'][y * W + x] = 0
            conn['intGridCsv'][y * W + x] = 1
        name, px = {'N': ('DoorHorizontal', (48, 0)), 'S': ('DoorHorizontal', (48, 112)),
                    'W': ('DoorVertical', (0, 48)), 'E': ('DoorVertical', (112, 48))}[side]
        e = copy.deepcopy(doors[name])
        e['iid'] = iid(level['identifier'], name, side)
        e['px'] = list(px)
        e['__grid'] = [px[0] // 16, px[1] // 16]
        e['__worldX'], e['__worldY'] = px
        out.append(e)
    return out


def surfaces_layer(level, model, cells):
    """Couche `Surfaces` du niveau : `cells` = {(col, row): valeur}."""
    inst = copy.deepcopy(model)
    inst['iid'] = iid(level['identifier'], 'Surfaces')
    inst['levelId'] = level['uid']
    inst['intGridCsv'] = [cells.get((x, y), 0) for y in range(W) for x in range(W)]
    # même ordre que le projet de test du moteur : Entities, Surfaces, LevelConnection, Walls
    level['layerInstances'].insert(1, inst)


src, dst = sys.argv[1], sys.argv[2]
d = json.load(open(src))
# identifiants à réutiliser quand le script a déjà tourné (idempotence : `nextUid` ne dérive pas)
old = [l['uid'] for l in d['levels'] if l['identifier'] in NEW] + \
      [l['uid'] for l in d['defs']['layers'] if l['identifier'] == 'Surfaces']
if old:
    d['nextUid'] = min(old)
d['levels'] = [l for l in d['levels'] if l['identifier'] not in NEW]
for l in d['levels']:
    l['layerInstances'] = [li for li in l['layerInstances'] if li['__identifier'] != 'Surfaces']
d['defs']['layers'] = [l for l in d['defs']['layers'] if l['identifier'] != 'Surfaces']

# Définition de la couche `Surfaces` : copie de celle du projet de test du moteur (même uid de valeurs).
surface_def = copy.deepcopy(next(l for l in d['defs']['layers'] if l['identifier'] == 'LevelConnection'))
surface_uid = d['nextUid']
surface_def.update(identifier='Surfaces', uid=surface_uid)
surface_def['intGridValues'] = [
    {'value': v, 'identifier': name, 'color': color, 'tile': None, 'groupUid': 0}
    for (name, v), color in zip(SURFACES.items(), ('#3B7DD8', '#D8C27A', '#202020'))
]
d['defs']['layers'].append(surface_def)
d['nextUid'] = surface_uid + 1

by_name = {l['identifier']: l for l in d['levels']}
doors = {e['__identifier']: e for l in d['levels'] for e in layer(l, 'Entities')['entityInstances']
         if e['__identifier'] in ('DoorHorizontal', 'DoorVertical')}
tables = next(e for e in layer(by_name['CombatEW'], 'Entities')['entityInstances'] if e['__identifier'] == 'Table')
barrel = next(e for e in layer(by_name['CombatEW'], 'Entities')['entityInstances'] if e['__identifier'] == 'Barrel')
# modèle : une couche IntGrid du projet (mêmes clés que LDtk écrit), reprise du niveau Depart
surface_model = copy.deepcopy(layer(by_name['Depart'], 'LevelConnection'))
surface_model.update({'__identifier': 'Surfaces', 'layerDefUid': surface_uid, 'iid': '', 'levelId': 0})

uid = d['nextUid']  # uid global à tout le projet (définitions comprises) : jamais max(niveaux) + 1
for ident, base, kind, sides, decor, cells in (
    # sable autour du coffre central (4 x 4 cases au milieu : ralentit, ne bloque pas)
    ('Coffre', 'Recompense', 'coffre', 'NESW',
     [(tables, 'Table', 1, 1), (tables, 'Table', 5, 1), (tables, 'Table', 1, 6)],
     {(x, y): SURFACES['sable'] for x in range(2, 6) for y in range(2, 6)}),
    # cul-de-sac à une ouverture : eau en bas à gauche, fosse dans l'angle haut droit, hors du chemin
    ('Secret', 'BossW', 'secret', 'W',
     [(tables, 'Table', 3, 1), (tables, 'Table', 3, 6), (tables, 'Table', 5, 6)],
     {(1, 6): SURFACES['eau'], (2, 6): SURFACES['eau'], (6, 1): SURFACES['fosse'], (6, 2): SURFACES['fosse'],
      (5, 1): SURFACES['fosse']}),
    # comme CombatNS sans ennemi posé
    ('ArenaNS', 'CombatNS', 'arene', 'NS',
     [(tables, 'Table', 1, 3), (barrel, 'Barrel', 6, 3), (barrel, 'Barrel', 6, 4)], {}),
):
    level = copy.deepcopy(by_name[base])
    level.update(identifier=ident, iid=iid(ident), uid=uid)
    set_kind(level, kind)
    for li in level['layerInstances']:
        li['iid'] = iid(ident, li['__identifier'])
        li['levelId'] = uid
    ents = layer(level, 'Entities')
    ents['entityInstances'] = open_sides(level, sides, doors) + [prop(m, n, c, r, ident) for m, n, c, r in decor]
    if cells:
        model = copy.deepcopy(surface_model)
        surfaces_layer(level, model, cells)
    d['levels'].append(level)
    uid += 1
d['nextUid'] = uid

json.dump(d, open(dst, 'w'), indent=1)
print('ok', [(l['identifier'], layer(l, 'Walls')['intGridCsv'].count(0)) for l in d['levels']])
