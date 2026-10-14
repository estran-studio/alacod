"""M2-T7 : cartes testbed des scénarios de boss et d'arène, dérivées de `salles.ldtk`
(départ + combat + récompense, voir m2-e1-salles.make_salles.py).

    python3 docs/taches/rapports/m2-t7-boss-arene.make_cartes.py games/testbed/assets/testbed

Produit dans ce dossier : `salles_boss.ldtk` (salle de combat devenue `boss`, un `gardien`),
`salles_boss_tir.ldtk` (`gardien_tir`), `salles_boss_renforts.ldtk` (`gardien_renforts`),
`salles_arene.ldtk` (salle `arene`, sans ennemi posé) et `salles_arene_mortelle.ldtk`.
"""
import copy, json, sys, os
d = sys.argv[1]
src = json.load(open(os.path.join(d, 'salles.ldtk')))

def build(name, kind, character):
    j = copy.deepcopy(src)
    combat = next(l for l in j['levels'] if l['identifier'] == 'SalleCombat')
    for f in combat['fieldInstances']:
        if f['__identifier'] == 'room_kind':
            f['__value'] = kind
            f['realEditorValues'] = [{'id': 'V_String', 'params': [kind]}]
    for li in combat['layerInstances']:
        spawns = [e for e in li['entityInstances'] if e['__identifier'] == 'CharacterSpawn']
        for e in spawns:
            li['entityInstances'].remove(e)
        if character and spawns:
            e = spawns[0]
            e['px'] = [96, 56]; e['__grid'] = [6, 3]
            for f in e['fieldInstances']:
                if f['__identifier'] == 'character':
                    f['__value'] = character
                    f['realEditorValues'] = [{'id': 'V_String', 'params': [character]}]
            li['entityInstances'].append(e)
    json.dump(j, open(os.path.join(d, name), 'w'), indent=2)

build('salles_boss.ldtk', 'boss', 'gardien')
build('salles_boss_tir.ldtk', 'boss', 'gardien_tir')
build('salles_boss_renforts.ldtk', 'boss', 'gardien_renforts')
build('salles_arene.ldtk', 'arene', None)
build('salles_arene_mortelle.ldtk', 'arene_mortelle', None)
