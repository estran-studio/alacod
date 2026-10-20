"""m3-salles-murs-isaac : agrandit les gabarits de `games/isaac/assets/maps/salles.ldtk` (unité 15 x 9 -> 30 x 18 cases).

Les gabarits d'origine (T5, T11d1, T11d2) font 15 x 9 cases de 16 px par unité de salle (intérieur 13 x 7) : un héros fait 32 px, soit une
salle de 6,5 x 3,5 héros (revue de William, 2026-10-19 : « salles minuscules où tout le monde se chevauche »). Ce script réécrit chaque gabarit à
**l'échelle ×2 uniforme** : unité 30 x 18 cases (480 x 288 px, intérieur 28 x 16 cases = 14 x 8 héros), toutes les tailles restent des multiples
de l'unité (1x1, 2x1, 1x2, 2x2) : la grille de M3-T5 reste cohérente (`cell: (30, 18)` dans les grammaires).

Ce qui change : la taille, le mur (anneau d'une case), les ouvertures (**4 cases** au milieu de chaque côté d'unité, 64 px : deux héros de front ; 30 et 18
sont pairs, une ouverture de 3 cases ne se centre plus, 6 cases faisait un trou de 96 px dans un mur de 480), les portes (même ouverture : 4 x 1 ou
1 x 4 cases), les entités posées (apparitions d'ennemis : position **proportionnelle** dans l'intérieur ; points de départ : même centre, écarts ×2),
les rochers (couche `Rocks` : **même nombre**, un rocher reste une case de 16 px, chaque amas garde sa forme et son écart au mur le plus proche), les
piques (couche `Surfaces` : une case devient un bloc 2 x 2, comme gungeon : c'est un terrain, sa part de la salle reste la même) et la couleur du sol.
Ce qui ne change pas : types de salle, champs, `iid`, ennemis, nombre de rochers, de piques-cases d'origine, de portes.

    git show 46a4e8f3:games/isaac/assets/maps/salles.ldtk > /tmp/salles_15x9.ldtk
    python3 docs/taches/rapports/m3-salles-murs-isaac.make_gabarits_grands.py /tmp/salles_15x9.ldtk games/isaac/assets/maps/salles.ldtk

Idempotent : la source est toujours la version 15 x 9 (le script refuse un fichier déjà agrandi). Aucun aléa : le fichier produit est le même à chaque
exécution. Contrôles (échec explicite) : anneau de murs de la source = bord moins les ouvertures ; aucune entité ne recouvre un rocher, une pique ou une
autre entité ; rien (entité, rocher, pique) dans le **couloir de passage** devant une ouverture (largeur de l'ouverture, 3 cases de profondeur) ; rien sur
la ligne droite qu'un bot suit entre deux ouvertures, d'une ouverture à un ennemi ou au centre (rochers à moins de 16 px, piques à moins de 12 px : la règle
de T11d1, `docs/conventions.md` §76.4) ; ouvertures reliées côté LevelConnection et côté portes.
"""
import json
import math
import sys

OLD_W, OLD_H = 15, 9
NEW_W, NEW_H = 30, 18
OPEN_OLD = 3          # cases d'une ouverture d'origine (centrée dans l'unité)
OPEN_NEW = 4          # cases d'une ouverture agrandie (centrée : 13..16 sur 30, 7..10 sur 18)
FLOOR = '#7B6A5F'     # sol « sous-sol » (terre brune claire), plus clair que les murs (palette de `scripts/generer-sprites-isaac.py`)
DOORS = ('DoorHorizontal', 'DoorVertical')
SIDES = 'NSWE'


def layer(level, name):
    return next((li for li in level['layerInstances'] if li['__identifier'] == name), None)


def old_openings(level):
    """{(côté, k)} des ouvertures d'origine : k = indice de l'unité le long du côté."""
    w, h = level['pxWid'] // 16, level['pxHei'] // 16
    nx, ny = w // OLD_W, h // OLD_H
    conn = layer(level, 'LevelConnection')['intGridCsv']
    cell = lambda x, y: conn[y * w + x]
    out = set()
    o0 = (OLD_W - OPEN_OLD) // 2
    p0 = (OLD_H - OPEN_OLD) // 2
    for k in range(nx):
        xs = [k * OLD_W + o0 + i for i in range(OPEN_OLD)]
        if all(cell(x, 0) for x in xs):
            out.add(('N', k))
        if all(cell(x, h - 1) for x in xs):
            out.add(('S', k))
    for k in range(ny):
        ys = [k * OLD_H + p0 + i for i in range(OPEN_OLD)]
        if all(cell(0, y) for y in ys):
            out.add(('W', k))
        if all(cell(w - 1, y) for y in ys):
            out.add(('E', k))
    assert sum(1 for v in conn if v) == OPEN_OLD * len(out), (level['identifier'], 'LevelConnection : cases hors ouverture')
    return out


def new_cells(side, k, w, h):
    """Cases (x, y) de l'ouverture agrandie `(côté, k)`."""
    o = (NEW_W - OPEN_NEW) // 2
    p = (NEW_H - OPEN_NEW) // 2
    if side in 'NS':
        y = 0 if side == 'N' else h - 1
        return [(k * NEW_W + o + i, y) for i in range(OPEN_NEW)]
    x = 0 if side == 'W' else w - 1
    return [(x, k * NEW_H + p + i) for i in range(OPEN_NEW)]


def centre(cells):
    return (sum(x for x, _ in cells) / len(cells) + 0.5, sum(y for _, y in cells) / len(cells) + 0.5)


def place(entity, col, row, cw=None, ch=None):
    if cw is not None:
        entity['width'], entity['height'] = cw * 16, ch * 16
    entity['__grid'] = [col, row]
    entity['px'] = [col * 16, row * 16]
    entity['__worldX'], entity['__worldY'] = entity['px']


def interior_map(old, old_size, new_size, size):
    """Position d'une boîte de `size` cases dont le bord bas est en `old` (cases d'intérieur 1..old_size-2) : proportionnelle."""
    inner_old, inner_new = old_size - 2, new_size - 2
    f = (old + size / 2 - 1) / inner_old
    return round(1 + f * inner_new - size / 2)


def scaled_cell(old, old_size, new_size):
    """Intervalle [début, fin[ de la nouvelle grille couvert par la case `old` de l'ancien intérieur (comme gungeon)."""
    io, inn = old_size - 2, new_size - 2
    return 1 + (old - 1) * inn // io, 1 + old * inn // io


def components(cells):
    """Composantes 4-connexes d'un ensemble de cases {(x, y): valeur}, déterministes (ordre de lecture)."""
    seen, out = set(), []
    for start in sorted(cells, key=lambda c: (c[1], c[0])):
        if start in seen:
            continue
        stack, comp = [start], []
        seen.add(start)
        while stack:
            x, y = stack.pop()
            comp.append((x, y))
            for n in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if n in cells and n not in seen:
                    seen.add(n)
                    stack.append(n)
        out.append(sorted(comp, key=lambda c: (c[1], c[0])))
    return out


def anchor_axis(lo, size_box, old_size, new_size):
    """Position (bord bas) d'un amas : collé au même mur avec le même écart s'il en est proche, sinon proportionnel."""
    gap_lo = lo - 1
    gap_hi = (old_size - 1) - (lo + size_box)
    if gap_lo <= 2 and gap_lo <= gap_hi:
        return 1 + gap_lo
    if gap_hi <= 2:
        return (new_size - 1) - gap_hi - size_box
    return interior_map(lo, old_size, new_size, size_box)


def seg_dist(p, a, b):
    ax, ay, bx, by = a[0], a[1], b[0], b[1]
    dx, dy = bx - ax, by - ay
    n = dx * dx + dy * dy
    t = 0.0 if n == 0 else max(0.0, min(1.0, ((p[0] - ax) * dx + (p[1] - ay) * dy) / n))
    return math.hypot(p[0] - (ax + t * dx), p[1] - (ay + t * dy))


def box_seg_dist(cell, a, b):
    """Distance (px) entre la case `cell` (carrée de 16 px) et le segment [a, b] (px) : échantillonnage fin, déterministe."""
    x0, y0 = cell[0] * 16, cell[1] * 16
    best = 1e9
    for i in range(33):
        for j in range(33):
            best = min(best, seg_dist((x0 + i / 2, y0 + j / 2), a, b))
    return best


def rescale(level):
    name = level['identifier']
    w0, h0 = level['pxWid'] // 16, level['pxHei'] // 16
    assert (w0 % OLD_W, h0 % OLD_H) == (0, 0), f'{name} : taille {w0} x {h0} hors unité'
    nx, ny = w0 // OLD_W, h0 // OLD_H
    w, h = nx * NEW_W, ny * NEW_H
    opens = old_openings(level)

    # --- anciennes données lues avant d'écraser ---
    old_walls = layer(level, 'Walls')['intGridCsv']
    o0, p0 = (OLD_W - OPEN_OLD) // 2, (OLD_H - OPEN_OLD) // 2
    open_old_cells = set()
    for side, k in opens:
        if side in 'NS':
            open_old_cells |= {(k * OLD_W + o0 + i, 0 if side == 'N' else h0 - 1) for i in range(OPEN_OLD)}
        else:
            open_old_cells |= {(0 if side == 'W' else w0 - 1, k * OLD_H + p0 + i) for i in range(OPEN_OLD)}
    for y in range(h0):
        for x in range(w0):
            border = x in (0, w0 - 1) or y in (0, h0 - 1)
            assert old_walls[y * w0 + x] == (1 if border and (x, y) not in open_old_cells else 0), (name, x, y)
    rocks_old = layer(level, 'Rocks')
    surf_old = layer(level, 'Surfaces')
    rock_cells = {(i % w0, i // w0): v for i, v in enumerate(rocks_old['intGridCsv']) if v} if rocks_old else {}
    surf_cells = {(i % w0, i // w0): v for i, v in enumerate(surf_old['intGridCsv']) if v} if surf_old else {}

    # --- couches ---
    open_new = {c for side, k in opens for c in new_cells(side, k, w, h)}
    ring = [1 if (x in (0, w - 1) or y in (0, h - 1)) and (x, y) not in open_new else 0 for y in range(h) for x in range(w)]
    conn = [1 if (x, y) in open_new else 0 for y in range(h) for x in range(w)]
    level['pxWid'], level['pxHei'] = w * 16, h * 16
    level['__bgColor'], level['bgColor'] = FLOOR, FLOOR
    level['__smartColor'] = FLOOR
    for li in level['layerInstances']:
        li['__cWid'], li['__cHei'] = w, h
        li['gridTiles'], li['autoLayerTiles'] = [], []
    layer(level, 'Walls')['intGridCsv'] = ring
    layer(level, 'LevelConnection')['intGridCsv'] = conn

    new_rocks = {}
    for comp in components(rock_cells):  # un amas = une composante ; sa forme et son écart au mur sont conservés
        xs, ys = [c[0] for c in comp], [c[1] for c in comp]
        bw, bh = max(xs) - min(xs) + 1, max(ys) - min(ys) + 1
        ox, oy = anchor_axis(min(xs), bw, w0, w), anchor_axis(min(ys), bh, h0, h)
        for (x, y) in comp:
            new_rocks[(ox + x - min(xs), oy + y - min(ys))] = rock_cells[(x, y)]
    assert len(new_rocks) == len(rock_cells), (name, 'rochers fusionnés')
    if rocks_old:
        csv = [0] * (w * h)
        for (x, y), v in new_rocks.items():
            csv[y * w + x] = v
        rocks_old['intGridCsv'] = csv

    new_spikes = {}
    for (x, y), v in surf_cells.items():
        (x0, x1), (y0, y1) = scaled_cell(x, w0, w), scaled_cell(y, h0, h)
        for yy in range(y0, y1):
            for xx in range(x0, x1):
                new_spikes[(xx, yy)] = v
    if surf_old:
        csv = [0] * (w * h)
        for (x, y), v in new_spikes.items():
            csv[y * w + x] = v
        surf_old['intGridCsv'] = csv

    # --- entités ---
    ents = layer(level, 'Entities')['entityInstances']
    boxes = []   # (nom, col, ligne, largeur, hauteur) des entités non porte
    door_sides = set()
    for e in ents:
        n = e['__identifier']
        cw, ch = e['width'] // 16, e['height'] // 16
        col, row = e['px'][0] // 16, e['px'][1] // 16
        if n == 'DoorHorizontal':
            side, k = ('N' if row == 0 else 'S'), col // OLD_W
            assert row in (0, h0 - 1) and col == k * OLD_W + o0 and cw == OPEN_OLD, (name, n, col, row)
            place(e, k * NEW_W + (NEW_W - OPEN_NEW) // 2, 0 if side == 'N' else h - 1, OPEN_NEW, 1)
            door_sides.add((side, k))
        elif n == 'DoorVertical':
            side, k = ('W' if col == 0 else 'E'), row // OLD_H
            assert col in (0, w0 - 1) and row == k * OLD_H + p0 and ch == OPEN_OLD, (name, n, col, row)
            place(e, 0 if side == 'W' else w - 1, k * NEW_H + (NEW_H - OPEN_NEW) // 2, 1, OPEN_NEW)
            door_sides.add((side, k))
        elif n == 'PlayerSpawn':
            # les points de départ restent groupés autour du centre de la salle, écarts ×2
            place(e, w // 2 + 2 * (col - w0 // 2), h // 2 + 2 * (row - h0 // 2))
            boxes.append((n, e['px'][0] // 16, e['px'][1] // 16, cw, ch))
        else:
            place(e, max(1, min(w - 1 - cw, interior_map(col, w0, w, cw))), max(1, min(h - 1 - ch, interior_map(row, h0, h, ch))))
            boxes.append((n, e['px'][0] // 16, e['px'][1] // 16, cw, ch))
    assert door_sides == opens, (name, 'portes et ouvertures ne correspondent pas', door_sides, opens)

    # --- contrôles ---
    keep_clear = []   # couloir de passage : largeur de l'ouverture, 3 cases de profondeur
    for side, k in sorted(opens):
        cells = new_cells(side, k, w, h)
        dx, dy = {'N': (0, 1), 'S': (0, -1), 'W': (1, 0), 'E': (-1, 0)}[side]
        xs = [c[0] for c in cells] + [cells[0][0] + dx * 3, cells[-1][0] + dx * 3]
        ys = [c[1] for c in cells] + [cells[0][1] + dy * 3, cells[-1][1] + dy * 3]
        keep_clear.append((min(xs), min(ys), max(xs), max(ys)))
    inside = lambda c, r: r[0] <= c[0] <= r[2] and r[1] <= c[1] <= r[3]
    for i, (n, x, y, cw, ch) in enumerate(boxes):
        for r in keep_clear:
            assert not (x <= r[2] and x + cw - 1 >= r[0] and y <= r[3] and y + ch - 1 >= r[1]), f'{name} : {n} en ({x},{y}) gêne une ouverture'
        for (x2, y2) in list(new_rocks) + list(new_spikes):
            assert not (x <= x2 < x + cw and y <= y2 < y + ch), f'{name} : {n} en ({x},{y}) sur un rocher ou des piques'
        for (n2, x2, y2, cw2, ch2) in boxes[i + 1:]:
            assert not (x < x2 + cw2 and x2 < x + cw and y < y2 + ch2 and y2 < y + ch), f'{name} : {n} et {n2} se recouvrent'
    for c in list(new_rocks) + list(new_spikes):
        assert not any(inside(c, r) for r in keep_clear), f'{name} : rocher ou pique {c} dans un couloir de passage'
        assert ring[c[1] * w + c[0]] == 0, (name, c, 'sur un mur')
    # lignes de marche du bot : ouverture <-> ouverture, ouverture -> ennemi, ouverture -> centre
    mids = [tuple(v * 16 for v in centre(new_cells(s, k, w, h))) for s, k in sorted(opens)]
    mid = (w * 8, h * 8)
    enemies = [((x + cw / 2) * 16, (y + ch / 2) * 16) for (n, x, y, cw, ch) in boxes if n == 'CharacterSpawn']
    segments = [(a, b) for i, a in enumerate(mids) for b in mids[i + 1:]] + [(a, e) for a in mids for e in enemies] + [(a, mid) for a in mids]
    for cells, limit, what in ((new_rocks, 16, 'rocher'), (new_spikes, 12, 'pique')):
        for c in cells:
            for a, b in segments:
                assert box_seg_dist(c, a, b) >= limit, f'{name} : {what} {c} à moins de {limit} px d’une ligne de marche'
    return nx, ny, len(opens), len(rock_cells), len(surf_cells), len(new_spikes)


def main(src, dst):
    d = json.load(open(src))
    rows = []
    for level in d['levels']:
        assert level['__bgColor'] != FLOOR, f"{level['identifier']} est déjà agrandi : partir de la version 15 x 9"
        before = (level['pxWid'] // 16, level['pxHei'] // 16)
        info = rescale(level)
        rows.append((level['identifier'], before, (level['pxWid'] // 16, level['pxHei'] // 16)) + info)
    d['defaultLevelBgColor'] = FLOOR
    with open(dst, 'w') as f:
        json.dump(d, f, indent=1, ensure_ascii=False)  # même mise en forme que LDtk : relecture sans bruit
    for r in rows:
        print('%-18s %2dx%-2d -> %2dx%-2d  unités %dx%d  ouvertures %d  rochers %d  piques %d -> %d cases' % (r[0], *r[1], *r[2], r[3], r[4], r[5], r[6], r[7], r[8]))


if __name__ == '__main__':
    main(*sys.argv[1:3])
