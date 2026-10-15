"""M2-T14c : pose les six nouveaux ennemis de `gungeon` dans les gabarits de combat existants de
`maps/salles.ldtk` (aucun gabarit ajouté : T14d). `CombatT` (étage `court`, scénarios scriptés) et
`EscouadeEW` ne bougent pas.

    python3 docs/taches/rapports/m2-t14c-ennemis.make_placement.py \
        games/gungeon/assets/maps/salles.ldtk games/gungeon/assets/maps/salles.ldtk

Idempotent : retire d'abord les ennemis déjà posés par ce script.
"""
import copy, json, sys, uuid

NEW = {'shotgun_kin', 'veteran_kin', 'bandana_kin', 'tourelle_kin', 'fuyard_kin', 'lanceur_kin'}
PLACEMENT = {
    'CombatEW': [('shotgun_kin', 5, 5), ('fuyard_kin', 2, 5)],
    'CombatNS': [('veteran_kin', 4, 5), ('tourelle_kin', 6, 6)],
    'CombatCroix': [('bandana_kin', 5, 5), ('lanceur_kin', 5, 6)],
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
