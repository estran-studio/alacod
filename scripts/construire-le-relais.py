#!/usr/bin/env python3
"""Build the LDtk blockout: five interior rooms, a continuous exterior, four mirrored variants."""
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
for path in ['maps/avant_poste.ldtk', 'exemples/test_map.ldtk']:
    for level in json.loads((ASSETS / path).read_text())['levels']:
        for layer in level['layerInstances']:
            for entity in layer['entityInstances']:
                samples.setdefault(entity['__identifier'], entity)
W, H = 34, 30

def identity(name):
    return str(uuid.uuid5(uuid.NAMESPACE_URL, 'alacod/le-relais-defense/' + name))

levels = []
for variant, (flip_x, flip_y) in enumerate([(False, False), (True, False), (False, True), (True, True)]):
    name = f'Relais_{variant + 1}'
    level = copy.deepcopy(base)
    level.update(identifier=name, iid=identity(name), uid=200 + variant,
                 pxWid=W * 16, pxHei=H * 16, worldX=variant * 576, worldY=0,
                 __neighbours=[])
    level['fieldInstances'][0].update(__value=True, realEditorValues=[{'id': 'V_Bool', 'params': [True]}])
    walls = [int(x in (0, W - 1) or y in (0, H - 1)) for y in range(H) for x in range(W)]
    entities = []
    def wall(x, y):
        walls[y * W + x] = 1
    for x in range(6, 28):
        wall(x, 6)
        wall(x, 23)
    for y in range(6, 24):
        wall(6, y)
        wall(27, y)
    for y in range(7, 23):
        wall(17, y)
    for x in range(7, 27):
        wall(x, 15)
    for y in range(16, 23):
        wall(23, y)
    # Solid workbenches/cover; corridors stay wide enough for 20 px bodies.
    for x, y, width, height in [(12, 10, 2, 2), (11, 19, 3, 1), (21, 10, 2, 1)]:
        for yy in range(y, y + height):
            for xx in range(x, x + width):
                wall(xx, yy)
    def entity(kind, x, y, fields=None):
        if kind not in samples:
            d = next(d for d in p['defs']['entities'] if d['identifier'] == kind)
            sample = copy.deepcopy(samples['WindowVertical'])
            sample.update(__identifier=kind, defUid=d['uid'], width=d['width'], height=d['height'])
            samples[kind] = sample
        e = copy.deepcopy(samples[kind])
        e.update(iid=identity(name + kind + str(len(entities))), px=[x * 16, y * 16], __grid=[x, y])
        for f in e['fieldInstances']:
            if fields and f['__identifier'] in fields:
                f.update(__value=fields[f['__identifier']], realEditorValues=[])
        entities.append(e)
    def opening(kind, x, y):
        d = next(d for d in p['defs']['entities'] if d['identifier'] == kind)
        for yy in range(y, y + d['height'] // 16):
            for xx in range(x, x + d['width'] // 16):
                walls[yy * W + xx] = 0
        entity(kind, x, y)
    # Every zombie entry is in the building's outer wall, never inside a room.
    for x in [10, 21]:
        opening('WindowVertical', x, 6)
    for x in [10, 24]:
        opening('WindowVertical', x, 23)
    for y in [10, 18]:
        opening('WindowHorizontal', 6, y)
        opening('WindowHorizontal', 27, y)
    # Internal doors form a loop when purchased; one exterior exit to the yard.
    for x, y in [(17, 10), (17, 19), (23, 19)]:
        opening('DoorVertical', x, y)
    for x, y in [(10, 15), (20, 15), (14, 6)]:
        opening('DoorHorizontal', x, y)
    for index, (x, y) in enumerate([(9, 9), (9, 12), (15, 9), (15, 12)]):
        entity('PlayerSpawn', x, y, {'index': index})
    for x, y in [(3, 3), (17, 3), (30, 3), (30, 15), (30, 26), (17, 26), (3, 26), (3, 15)]:
        entity('ZombieSpawn', x, y)
    for weapon, x, y, price in [('pistol', 8, 7, 500), ('shotgun', 8, 17, 1000), ('machine_gun', 24, 17, 1500)]:
        entity('WeaponLocation', x, y, {'weapon': weapon, 'price': price})
    entity('SodaLocation', 19, 7, {'perk': 'juggernog'})
    if flip_x:
        walls = [walls[y * W + (W - 1 - x)] for y in range(H) for x in range(W)]
    if flip_y:
        walls = [walls[(H - 1 - y) * W + x] for y in range(H) for x in range(W)]
    for e in entities:
        x, y = e['__grid']
        if flip_x:
            x = W - x - e['width'] // 16
        if flip_y:
            y = H - y - e['height'] // 16
        e.update(__grid=[x, y], px=[x * 16, y * 16],
                 __worldX=level['worldX'] + x * 16, __worldY=y * 16)
    for layer in level['layerInstances']:
        layer.update(iid=identity(name + layer['__identifier']), levelId=level['uid'],
                     __cWid=W, __cHei=H, autoLayerTiles=[], gridTiles=[], entityInstances=[], intGridCsv=[])
        if layer['__identifier'] == 'Entities':
            layer['entityInstances'] = entities
        elif layer['__identifier'] == 'LevelConnection':
            layer['intGridCsv'] = [0] * (W * H)
        else:
            layer['intGridCsv'] = walls
            for y in range(H):
                for x in range(W):
                    sx, sy = (256, 96) if walls[y * W + x] else (320, 272)
                    layer['autoLayerTiles'].append({'px': [x * 16, y * 16], 'src': [sx, sy],
                                                   'f': 0, 't': sy // 16 * 23 + sx // 16, 'd': [5, 0], 'a': 1})
    levels.append(level)
p.update(levels=levels, nextUid=204, iid=identity('project'))
text = json.dumps(p, ensure_ascii=False, indent='\t') + '\n'
text = re.sub(r'\[\s*-?\d+(?:\s*,\s*-?\d+)*\s*\]',
              lambda m: '[' + ', '.join(re.findall(r'-?\d+', m.group())) + ']', text)
out = ASSETS / 'maps/le_relais_prototype.ldtk'
out.write_text(text)
print(out)
