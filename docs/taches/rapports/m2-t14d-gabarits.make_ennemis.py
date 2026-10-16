"""M2-T14d (D64) : composition des ennemis des gabarits de combat et d'escouade de `maps/salles.ldtk`.

Chaque gabarit listé reçoit exactement ses `CharacterSpawn` (les autres entités ne bougent pas) ; les
positions sont en cases de 16 px, à deux cases au moins des ouvertures (milieu de chaque côté).
Les gabarits de départ, de boss, d'arène, de récompense, de boutique, de coffre et de secret ne bougent pas.

    python3 docs/taches/rapports/m2-t14d-gabarits.make_ennemis.py \\
        games/gungeon/assets/maps/salles.ldtk games/gungeon/assets/maps/salles.ldtk

Idempotent. Les identifiants (iid) dérivent du gabarit, du personnage et de la case.
"""
import copy, json, sys, uuid

PLACEMENT = {
    'CombatEW': [('bullet_kin', 2, 2), ('bullet_kin', 5, 2), ('fuyard_kin', 2, 5)],
    'CombatNS': [('bullet_kin', 2, 2), ('bullet_kin', 5, 2), ('veteran_kin', 4, 5)],
    'CombatT': [('bullet_kin', 2, 2), ('bullet_kin', 5, 2), ('bullet_kin', 2, 5)],
    'CombatCroix': [('bullet_kin', 2, 2), ('bullet_kin', 5, 2), ('shotgun_kin', 5, 5)],
    'EscouadeEW': [('bullet_kin_squad', 2, 2), ('bullet_kin_squad', 5, 2), ('bullet_kin_squad', 3, 6)],
}

src, dst = sys.argv[1], sys.argv[2]
d = json.load(open(src))
for level in d['levels']:
    if level['identifier'] not in PLACEMENT:
        continue
    layer = next(li for li in level['layerInstances'] if li['__identifier'] == 'Entities')
    ents = layer['entityInstances']
    model = next(e for e in ents if e['__identifier'] == 'CharacterSpawn')
    ents[:] = [e for e in ents if e['__identifier'] != 'CharacterSpawn']
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
