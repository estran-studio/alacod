"""M2-T7 : deux gabarits d'arène (`room_kind` = `arene`) ajoutés aux neuf de la grammaire d'étage
de `gungeon` (`m2-t10-grammaire-etage.make_salles.py`) : `ArenaEW` (ouvertures O et E) et
`ArenaCroix` (quatre ouvertures), comme `CombatEW` et `CombatCroix` mais **sans ennemi posé** (les
vagues viennent de `rooms/arene.ron`).

    python3 docs/taches/rapports/m2-t7-boss-arene.make_arene.py \
        games/gungeon/assets/maps/salles.ldtk games/gungeon/assets/maps/salles.ldtk
"""
import copy, json, sys, uuid

src, dst = sys.argv[1], sys.argv[2]
d = json.load(open(src))
levels = [l for l in d['levels'] if not l['identifier'].startswith('Arena')]
by_name = {l['identifier']: l for l in levels}
uid = max(l['uid'] for l in levels) + 1
for ident, model in (('ArenaEW', 'CombatEW'), ('ArenaCroix', 'CombatCroix')):
    level = copy.deepcopy(by_name[model])
    level.update(identifier=ident, iid=str(uuid.uuid4()), uid=uid)
    for f in level['fieldInstances']:
        if f['__identifier'] == 'room_kind':
            f['__value'] = 'arene'
            f['realEditorValues'] = [{'id': 'V_String', 'params': ['arene']}]
    for li in level['layerInstances']:
        li['iid'] = str(uuid.uuid4())
        li['levelId'] = uid
        li['entityInstances'] = [e for e in li['entityInstances']
                                 if e['__identifier'] != 'CharacterSpawn']
    levels.append(level)
    uid += 1
d['levels'] = levels
json.dump(d, open(dst, 'w'), indent=1)
print('ok', [l['identifier'] for l in levels])
