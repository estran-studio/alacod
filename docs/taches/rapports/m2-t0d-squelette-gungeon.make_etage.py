"""M2-T0d : l'étage de `gungeon` (trois salles typées : depart, combat, boss) à partir du script de
M2-E1 (`m2-e1-salles.make_salles.py`, gabarit `testbed/two_rooms_door.ldtk`), puis :
- niveaux renommés `Depart`, `Combat`, `Boss`, la troisième salle devient `room_kind: boss` ;
- salle de combat : deux `bullet_kin` ; salle du boss : un `gatling` au centre (copie d'un
  `CharacterSpawn`) ;
- chemins de tilesets relatifs au dossier `maps/` (`../atlas/…`).

    python3 docs/taches/rapports/m2-t0d-squelette-gungeon.make_etage.py \\
        games/testbed/assets/testbed/two_rooms_door.ldtk games/gungeon/assets/maps/etage_1.ldtk
"""
import copy, json, os, subprocess, sys, uuid

src, dst = sys.argv[1], sys.argv[2]
here = os.path.dirname(os.path.abspath(__file__))
subprocess.run([sys.executable, os.path.join(here, 'm2-e1-salles.make_salles.py'), src, dst, 'bullet_kin'],
               check=True, stdout=subprocess.DEVNULL)
d = json.load(open(dst))
names = {'SalleDepart': 'Depart', 'SalleCombat': 'Combat', 'SalleRecompense': 'Boss'}
for level in d['levels']:
    level['identifier'] = names[level['identifier']]
    for f in level['fieldInstances']:
        if f['__identifier'] == 'room_kind' and level['identifier'] == 'Boss':
            f['__value'] = 'boss'
            f['realEditorValues'] = [{'id': 'V_String', 'params': ['boss']}]
combat = next(l for l in d['levels'] if l['identifier'] == 'Combat')
boss = next(l for l in d['levels'] if l['identifier'] == 'Boss')
spawn = next(e for li in combat['layerInstances'] for e in li['entityInstances']
             if e['__identifier'] == 'CharacterSpawn')
boss_entities = next(li for li in boss['layerInstances'] if li['__identifier'] == 'Entities')['entityInstances']
e = copy.deepcopy(spawn)
e['iid'] = str(uuid.uuid4())
e['px'] = [64, 48]
e['__grid'] = [4, 3]
for f in e['fieldInstances']:
    if f['__identifier'] == 'character':
        f['__value'] = 'gatling'
        f['realEditorValues'] = [{'id': 'V_String', 'params': ['gatling']}]
boss_entities.append(e)
for t in d['defs']['tilesets']:
    if t.get('relPath') and not t['relPath'].startswith('../'):
        t['relPath'] = '../' + t['relPath']
json.dump(d, open(dst, 'w'), indent=1)
print('ok', [l['identifier'] for l in d['levels']])
