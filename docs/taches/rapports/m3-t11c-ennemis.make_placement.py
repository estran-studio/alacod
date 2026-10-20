"""M3-T11c : pose les ennemis d'`isaac` dans le gabarit de combat existant de `maps/salles.ldtk` (aucun gabarit
ajouté : T11d). `suiveur` (3,2) et `mouche` (11,6), posés par M3-T0f, ne bougent pas.

    python3 docs/taches/rapports/m3-t11c-ennemis.make_placement.py \
        games/isaac/assets/maps/salles.ldtk games/isaac/assets/maps/salles.ldtk

Idempotent : retire d'abord les ennemis déjà posés par ce script.
"""
import copy, json, sys, uuid

NEW = {'tourelle_croix'}
PLACEMENT = {
    'Combat': [('tourelle_croix', 13, 7)],
}

src, dst = sys.argv[1], sys.argv[2]
d = json.load(open(src))
for level in d['levels']:
    if level['identifier'] not in PLACEMENT:
        continue
    layer = next(li for li in level['layerInstances'] if li['__identifier'] == 'Entities')
    ents = layer['entityInstances']
    ents[:] = [e for e in ents if not (e['__identifier'] == 'CharacterSpawn' and
               next(f for f in e['fieldInstances'] if f['__identifier'] == 'character')['__value'] in NEW)]
    model = next(e for e in ents if e['__identifier'] == 'CharacterSpawn')
    for name, gx, gy in PLACEMENT[level['identifier']]:
        e = copy.deepcopy(model)
        e['iid'] = str(uuid.uuid5(uuid.NAMESPACE_URL, f"alacod/{level['identifier']}/{name}/{gx}/{gy}"))
        e['__grid'] = [gx, gy]
        e['px'] = [gx * 16, gy * 16]
        e['__worldX'] = e['px'][0] + level['worldX']
        e['__worldY'] = e['px'][1] + level['worldY']
        for f in e['fieldInstances']:
            if f['__identifier'] == 'character':
                f['__value'] = name
                f['realEditorValues'] = [{'id': 'V_String', 'params': [name]}]
        ents.append(e)
json.dump(d, open(dst, 'w'), indent=1)
print('ok')
