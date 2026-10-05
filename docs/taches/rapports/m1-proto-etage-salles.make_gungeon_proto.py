"""Prototype m1-proto-etage-salles : trois gabarits de salles (départ, combat, récompense) à partir
de testbed/two_rooms_door.ldtk, avec un champ de niveau `room_kind`. Jetable."""
import copy, json, uuid, sys
src, dst = sys.argv[1], sys.argv[2]
CHAR = sys.argv[3] if len(sys.argv) > 3 else 'follower'
d = json.load(open(src))
A, B = d['levels']
uid = d['nextUid']
field = copy.deepcopy(d['defs']['levelFields'][0])
field.update(identifier='room_kind', __type='String', uid=uid, type='F_String', canBeNull=True)
d['defs']['levelFields'].append(field)
kind_uid = uid; uid += 1

def set_kind(level, kind):
    level['fieldInstances'] = [f for f in level['fieldInstances'] if f['__identifier'] != 'room_kind']
    level['fieldInstances'].append({'__identifier': 'room_kind', '__type': 'String', '__value': kind,
        '__tile': None, 'defUid': kind_uid, 'realEditorValues': [{'id': 'V_String', 'params': [kind]}]})

def layer(level, name):
    return next(li for li in level['layerInstances'] if li['__identifier'] == name)

def fresh(level, ident, n):
    level['identifier'] = ident
    level['iid'] = str(uuid.uuid4())
    level['uid'] = n
    for li in level['layerInstances']:
        li['levelId'] = n
        li['iid'] = str(uuid.uuid4())
        for e in li['entityInstances']:
            e['iid'] = str(uuid.uuid4())

depart = copy.deepcopy(A); fresh(depart, 'ProtoDepart', 0); set_kind(depart, 'depart')
recompense = copy.deepcopy(B); fresh(recompense, 'ProtoRecompense', 2); set_kind(recompense, 'recompense')
combat = copy.deepcopy(B); fresh(combat, 'ProtoCombat', 1); set_kind(combat, 'combat')
w = 8
walls, conn = layer(combat, 'Walls'), layer(combat, 'LevelConnection')
# Sortie est décalée (rangées 1-2) : seule la récompense (porte ouest en 1-2) s'y raccorde, et
# le départ (sortie est en 3-4) ne se raccorde qu'au combat (porte ouest en 3-4).
for r in (1, 2):
    walls['intGridCsv'][r * w + 7] = 0
    conn['intGridCsv'][r * w + 7] = 1
rw, rc = layer(recompense, 'Walls'), layer(recompense, 'LevelConnection')
for r in (3, 4):
    rw['intGridCsv'][r * w + 0] = 1
    rc['intGridCsv'][r * w + 0] = 0
for r in (1, 2):
    rw['intGridCsv'][r * w + 0] = 0
    rc['intGridCsv'][r * w + 0] = 1
for e in layer(recompense, 'Entities')['entityInstances']:
    if e['__identifier'] == 'DoorVertical':
        e['px'] = [e['px'][0], 16]; e['__grid'] = [e['__grid'][0], 1]
ents = layer(combat, 'Entities')['entityInstances']
east_door = copy.deepcopy(next(e for e in layer(A, 'Entities')['entityInstances'] if e['__identifier'] == 'DoorVertical'))
east_door['iid'] = str(uuid.uuid4())
east_door['px'] = [east_door['px'][0], 16]; east_door['__grid'] = [east_door['__grid'][0], 1]
ents.append(east_door)
spawn_template = json.load(open(src.replace('two_rooms_door', 'arena')))
cs = next(e for lv in spawn_template['levels'] for li in lv['layerInstances'] for e in li['entityInstances'] if e['__identifier'] == 'CharacterSpawn')
for (gx, gy) in ((6, 3), (6, 4)):
    e = copy.deepcopy(cs)
    e['iid'] = str(uuid.uuid4()); e['__grid'] = [gx, gy]; e['px'] = [gx * 16, gy * 16]
    for f in e['fieldInstances']:
        if f['__identifier'] == 'character':
            f['__value'] = CHAR; f['realEditorValues'] = [{'id': 'V_String', 'params': [CHAR]}]
        if f['__identifier'] == 'team':
            f['__value'] = 'enemies'; f['realEditorValues'] = [{'id': 'V_String', 'params': ['enemies']}]
    ents.append(e)
d['levels'] = [depart, combat, recompense]
d['nextUid'] = uid
json.dump(d, open(dst, 'w'), indent=1)
print('ok', [l['identifier'] for l in d['levels']])
