#!/usr/bin/env python3
"""Author the LDtk room library. The Rust loader assembles these modules per run seed."""
import copy
import json
import re
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / 'games/zombies/assets'
p = json.loads((ASSETS / 'maps/avant_poste.ldtk').read_text())
base = copy.deepcopy(p['levels'][0])
samples = {}
for level in p['levels']:
    for layer in level['layerInstances']:
        for e in layer['entityInstances']:
            samples.setdefault(e['__identifier'], e)
role_def = copy.deepcopy(p['defs']['levelFields'][0])
role_def.update(identifier='building_role', uid=1024, __type='String', type='F_String',
                editorDisplayMode='NameAndValue', editorAlwaysShow=True,
                doc='Opt-in building module: Accueil, Radio, Atelier, Infirmerie, Reserve, Passage')
p['defs']['levelFields'].append(role_def)
for definition in p['defs']['entities']:
    if definition['identifier']=='ZombieSpawn':
        sample=next(f for e in p['defs']['entities'] for f in e['fieldDefs'] if f['__type']=='Int')
        for n,name in enumerate(['active_x','active_y','active_width','active_height']):
            field=copy.deepcopy(sample)
            field.update(identifier=name,uid=1025+n,defaultOverride={'id':'V_Int','params':[0]})
            definition['fieldDefs'].append(field)
modules = [
    ('Accueil_vestibule', 'Accueil', [(8,2,2,2)], ('pistol',500)),
    ('Accueil_repli', 'Accueil', [(8,6,2,1)], ('pistol',500)),
    ('Radio_pilier', 'Radio', [(5,4,2,2)], None),
    ('Radio_alcove', 'Radio', [(7,2,2,2)], None),
    ('Atelier_fusil', 'Atelier', [(5,4,2,1)], ('rifle',1200)),
    ('Atelier_pompe', 'Atelier', [(7,6,3,1)], ('shotgun',1000)),
    ('Atelier_mitraillette', 'Atelier', [(3,5,2,2)], ('machine_gun',1500)),
    ('Infirmerie_jug', 'Infirmerie', [(7,2,2,2)], ('juggernog',0)),
    ('Infirmerie_rapide', 'Infirmerie', [(7,6,2,1)], ('speed_cola',0)),
    ('Reserve_munitions', 'Reserve', [(5,6,2,1)], ('rifle',1500)),
    ('Reserve_survie', 'Reserve', [(7,3,2,2)], ('juggernog',0)),
    ('Passage_pilier', 'Passage', [(5,4,2,2)], None),
    ('Passage_decale', 'Passage', [(7,2,2,2)], None),
    ('Passage_ouvert', 'Passage', [], None),
]
levels = []
for i, (name, role, obstacles, purchase) in enumerate(modules):
    level = copy.deepcopy(base)
    level.update(identifier=name, iid=str(uuid.uuid5(uuid.NAMESPACE_URL, name)), uid=200+i,
                 pxWid=384, pxHei=320, worldX=(i%7)*400, worldY=(i//7)*336, __neighbours=[])
    level['fieldInstances'][0].update(__value=role=='Accueil', realEditorValues=[])
    level['fieldInstances'].append({'__identifier':'building_role','__type':'String',
        '__value':role,'__tile':None,'defUid':1024,'realEditorValues':[]})
    walls = [int(x in (0,23) or y in (0,19)) for y in range(20) for x in range(24)]
    for x,y,w,h in obstacles:
        for yy in range(y*2,y*2+h*2):
            for xx in range(x*2,x*2+w*2):walls[yy*24+xx]=1
    entities = []
    def entity(kind,x,y,fields):
        e = copy.deepcopy(samples[kind])
        e.update(iid=str(uuid.uuid5(uuid.NAMESPACE_URL,name+str(len(entities)))),px=[x*16,y*16],__grid=[x,y],
                 __worldX=level['worldX']+x*16,__worldY=level['worldY']+y*16)
        for f in e['fieldInstances']:
            f.update(__value=fields[f['__identifier']],realEditorValues=[])
        entities.append(e)
    if role=='Accueil':
        for index,(x,y) in enumerate([(6,6),(12,6),(6,12),(12,12)]):entity('PlayerSpawn',x,y,{'index':index})
    if purchase:
        name_id,price=purchase
        if price:entity('WeaponLocation',7,2,{'weapon':name_id,'price':price})
        else:entity('SodaLocation',7,2,{'perk':name_id})
    for layer in level['layerInstances']:
        layer.update(iid=str(uuid.uuid5(uuid.NAMESPACE_URL,name+layer['__identifier'])),levelId=level['uid'],
                     __cWid=24,__cHei=20,entityInstances=[],gridTiles=[],autoLayerTiles=[],intGridCsv=[])
        if layer['__identifier']=='Entities':layer['entityInstances']=entities
        elif layer['__identifier']=='LevelConnection':layer['intGridCsv']=[0]*480
        else:
            layer['intGridCsv']=walls
            for y in range(20):
                for x in range(24):
                    sx,sy=(256,96) if walls[y*24+x] else (320,272)
                    layer['autoLayerTiles'].append({'px':[x*16,y*16],'src':[sx,sy],'f':0,
                                                   't':sy//16*23+sx//16,'d':[5,0],'a':1})
    levels.append(level)
p.update(levels=levels,nextUid=1029)
s=json.dumps(p,ensure_ascii=False,indent='\t')+'\n'
s=re.sub(r'\[\s*-?\d+(?:\s*,\s*-?\d+)*\s*\]',lambda m:'['+', '.join(re.findall(r'-?\d+',m.group()))+']',s)
out=ASSETS/'maps/le_relais_modules.ldtk';out.write_text(s);print(out)
