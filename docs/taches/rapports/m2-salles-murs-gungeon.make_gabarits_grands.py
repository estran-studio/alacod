"""m2-salles-murs-gungeon : agrandit les quinze gabarits de `games/gungeon/assets/maps/salles.ldtk`.

Les gabarits d'origine font 8 x 8 cases (128 px, six cases de sol de large) ; William les trouve « minuscules »
(revue M2, 2026-10-18). Ce script les réécrit à l'échelle de `SIZES` : même disposition, même type
(`room_kind`), mêmes ouvertures (côtés), portes **au milieu de chaque côté** (deux cases), entités placées
**proportionnellement** dans la salle agrandie (le contenu ne change pas : mêmes ennemis, mêmes props, mêmes
nombres ; seul l'espace entre eux grandit), surfaces (`Surfaces`) étirées de la même façon.

    git show f1d5a093:games/gungeon/assets/maps/salles.ldtk > /tmp/salles_8x8.ldtk
    python3 docs/taches/rapports/m2-salles-murs-gungeon.make_gabarits_grands.py \\
        /tmp/salles_8x8.ldtk games/gungeon/assets/maps/salles.ldtk

Idempotent : la source est toujours la version 8 x 8 (le script refuse un fichier déjà agrandi). Aucun
aléa : le fichier produit est le même à chaque exécution. Vérifie que les entités d'une salle ne se
recouvrent pas et qu'aucun prop (table, baril) ni apparition d'ennemi ne gêne le passage devant une ouverture.
"""
import copy
import json
import sys

OLD = 8

# gabarit -> (largeur, hauteur) en cases de 16 px, murs compris. Voir `docs/conventions.md` §40.
SIZES = {
    'Depart': (14, 12),
    'CombatEW': (16, 12), 'CombatNS': (16, 12), 'CombatT': (16, 12), 'CombatCroix': (16, 12),
    'EscouadeEW': (16, 12),
    'Recompense': (12, 10),
    'Boutique': (14, 10),
    'Coffre': (14, 12),
    'Secret': (12, 10),
    'ArenaEW': (20, 14), 'ArenaCroix': (20, 14), 'ArenaNS': (20, 14),
    'BossW': (22, 16), 'BossN': (22, 16),
}

# côtés de l'ancienne ouverture (2 cases au milieu)
OLD_OPENINGS = {'N': [(3, 0), (4, 0)], 'S': [(3, 7), (4, 7)], 'W': [(0, 3), (0, 4)], 'E': [(7, 3), (7, 4)]}
DOORS = ('DoorHorizontal', 'DoorVertical')


def layer(level, name):
    return next((li for li in level['layerInstances'] if li['__identifier'] == name), None)


def sides_of(level):
    conn = layer(level, 'LevelConnection')['intGridCsv']
    return [s for s, cells in OLD_OPENINGS.items() if all(conn[y * OLD + x] == 1 for x, y in cells)]


def openings(sides, w, h):
    """Cases d'ouverture du côté, par côté : deux cases au milieu."""
    mid_x, mid_y = w // 2 - 1, h // 2 - 1
    return {
        'N': [(mid_x, 0), (mid_x + 1, 0)], 'S': [(mid_x, h - 1), (mid_x + 1, h - 1)],
        'W': [(0, mid_y), (0, mid_y + 1)], 'E': [(w - 1, mid_y), (w - 1, mid_y + 1)],
    }, [s for s in 'NSWE' if s in sides]


def place(entity, col, row):
    entity['__grid'] = [col, row]
    entity['px'] = [col * 16, row * 16]
    entity['__worldX'], entity['__worldY'] = entity['px']


def scaled_cell(old, w):
    """Intervalle [début, fin[ de cases de la nouvelle grille couvert par la case `old` (1..6) de l'ancien sol."""
    inner = w - 2
    return 1 + (old - 1) * inner // 6, 1 + old * inner // 6


def rescale(level, w, h):
    sides = sides_of(level)
    spots, sides = openings(sides, w, h)
    cells_open = {c for s in sides for c in spots[s]}
    ring = [1 if (x in (0, w - 1) or y in (0, h - 1)) and (x, y) not in cells_open else 0
            for y in range(h) for x in range(w)]
    conn = [1 if (x, y) in cells_open else 0 for y in range(h) for x in range(w)]

    # --- anciennes données lues avant d'écraser ---
    old_walls = layer(level, 'Walls')['intGridCsv']
    for y in range(OLD):
        for x in range(OLD):
            border = x in (0, OLD - 1) or y in (0, OLD - 1)
            opening = (x, y) in {c for s in sides for c in OLD_OPENINGS[s]}
            assert old_walls[y * OLD + x] == (1 if border and not opening else 0), (level['identifier'], x, y)
    old_surfaces = layer(level, 'Surfaces')['intGridCsv'] if layer(level, 'Surfaces') else None

    level['pxWid'], level['pxHei'] = w * 16, h * 16
    for li in level['layerInstances']:
        li['__cWid'], li['__cHei'] = w, h
        li['gridTiles'], li['autoLayerTiles'] = [], []
    layer(level, 'Walls')['intGridCsv'] = ring
    layer(level, 'LevelConnection')['intGridCsv'] = conn
    if old_surfaces is not None:
        new = [0] * (w * h)
        for r in range(1, OLD - 1):
            for c in range(1, OLD - 1):
                v = old_surfaces[r * OLD + c]
                if v:
                    (x0, x1), (y0, y1) = scaled_cell(c, w), scaled_cell(r, h)
                    for y in range(y0, y1):
                        for x in range(x0, x1):
                            new[y * w + x] = v
        layer(level, 'Surfaces')['intGridCsv'] = new

    # --- entités ---
    ents = layer(level, 'Entities')['entityInstances']
    rects = []
    for e in ents:
        name = e['__identifier']
        cw, ch = e['width'] // 16, e['height'] // 16
        col, row = e['px'][0] // 16, e['px'][1] // 16
        if name in DOORS:
            # porte : un côté (d'après sa position d'origine), centrée sur l'ouverture
            if name == 'DoorHorizontal':
                new_col = w // 2 - 1
                new_row = 0 if row == 0 else h - 1
                place(e, new_col, new_row)
            else:
                new_row = h // 2 - 1
                new_col = 0 if col == 0 else w - 1
                place(e, new_col, new_row)
            continue
        if name == 'PlayerSpawn':
            # les points de départ restent groupés autour du centre de la salle (mêmes écarts)
            place(e, w // 2 + (col - OLD // 2), h // 2 + (row - OLD // 2))
        else:
            fx, fy = (col + cw / 2 - 1) / 6, (row + ch / 2 - 1) / 6
            new_col = round(1 + fx * (w - 2) - cw / 2)
            new_row = round(1 + fy * (h - 2) - ch / 2)
            place(e, max(1, min(w - 1 - cw, new_col)), max(1, min(h - 1 - ch, new_row)))
        rects.append((name, e['px'][0] // 16, e['px'][1] // 16, cw, ch))

    # --- contrôles ---
    keep_clear = []  # couloir de deux cases devant chaque ouverture, sur deux de profondeur
    for s in sides:
        (x0, y0), (x1, y1) = spots[s]
        dx, dy = {'N': (0, 1), 'S': (0, -1), 'W': (1, 0), 'E': (-1, 0)}[s]
        keep_clear.append((min(x0, x1, x0 + dx * 2, x1 + dx * 2), min(y0, y1, y0 + dy * 2, y1 + dy * 2),
                           max(x0, x1, x0 + dx * 2, x1 + dx * 2), max(y0, y1, y0 + dy * 2, y1 + dy * 2)))
    for i, (n, x, y, cw, ch) in enumerate(rects):
        for (a, b, c, d) in keep_clear:
            if x <= c and x + cw - 1 >= a and y <= d and y + ch - 1 >= b:
                raise SystemExit(f"{level['identifier']} : {n} en ({x},{y}) gêne une ouverture")
        for (n2, x2, y2, cw2, ch2) in rects[i + 1:]:
            if x < x2 + cw2 and x2 < x + cw and y < y2 + ch2 and y2 < y + ch:
                raise SystemExit(f"{level['identifier']} : {n} et {n2} se recouvrent")
    return sides


def main(src, dst):
    d = json.load(open(src))
    for level in d['levels']:
        assert level['pxWid'] == OLD * 16, f"{level['identifier']} est déjà agrandi : partir de la version 8 x 8"
        w, h = SIZES[level['identifier']]
        rescale(level, w, h)
    assert {l['identifier'] for l in d['levels']} == set(SIZES)
    with open(dst, 'w') as f:
        json.dump(d, f, indent=1, ensure_ascii=False)  # même mise en forme que LDtk : relecture sans bruit


if __name__ == '__main__':
    main(*sys.argv[1:3])
