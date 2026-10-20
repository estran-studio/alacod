"""usage : alacod-sim … --log 2>&1 | flux.py <morts.tsv>
Filtre au vol le journal d'`alacod-sim --log` (M3-T18) : recopie sur stdout les lignes d'`alacod-sim` (résumés, `seed …`,
erreurs) et écrit dans <morts.tsv> une ligne par graine et par joueur mort : graine, joueur (net id), frame, étage, salles
verrouillées à cet instant (type#salle), tueur (nom de l'ennemi, `?` s'il n'est nommé par aucune ligne du journal), puis
une ligne `diable` par graine : étages où une salle du diable a reçu ses présentoirs (`slot_assign pool=diable`).
Logique de `m3-t14-bot-isaac/morts.py` (journal `entity killed by NetId(n)`, `(nom-n)`, `room_state`, `floor_transition`),
lue sans rejouer la graine. Le synctest resimule (une ligne peut revenir à la même frame) : seule la première mort d'un
joueur compte, une transition d'étage est comptée par frame distincte."""
import re
import sys

out = open(sys.argv[1], 'w')


def reset():
    return {'rooms': {}, 'names': {}, 'floors': set(), 'deaths': {}, 'devil': set()}


st = reset()
for line in sys.stdin:
    if line.startswith('seed ') or line.startswith('alacod-sim') or 'panicked' in line:
        sys.stdout.write(line)
        sys.stdout.flush()
        m = re.match(r'seed\s+(-?\d+)', line)
        if m:
            seed = m[1]
            for victim, (frame, floor, locked, killer) in sorted(st['deaths'].items()):
                print(seed, victim, frame, floor, locked, st['names'].get(killer, '?'), sep='\t', file=out)
            print(seed, 'diable', ','.join(map(str, sorted(st['devil']))) or '-', sep='\t', file=out)
            out.flush()
            st = reset()
        continue
    f = re.search(r'ggrs\{f=(\d+)', line)
    m = re.search(r'room_state room=(\d+) kind=(\w+) state=(\w+)', line)
    if m:
        st['rooms'][m[1]] = (m[2], m[3])
    for n in re.finditer(r'\((\w+)-(\d+)\)', line):
        st['names'][n[2]] = n[1]
    if 'floor_transition' in line and f:
        st['floors'].add(int(f[1]))
    if 'slot_assign pool=diable' in line:
        d = re.search(r'floor=(\d+)', line)
        if d:
            st['devil'].add(int(d[1]))
    k = re.search(r'entity killed by NetId\((\d+)\)', line)
    v = re.search(r'\(player-(\d+)\)', line)
    if k and v and v[1] not in st['deaths']:
        locked = ','.join(f'{kind}#{i}' for i, (kind, s) in sorted(st['rooms'].items()) if s == 'Locked') or '-'
        st['deaths'][v[1]] = (int(f[1]) if f else -1, len(st['floors']), locked, k[1])
