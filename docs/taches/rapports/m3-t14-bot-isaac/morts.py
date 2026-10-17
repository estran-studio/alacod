"""usage: morts.py <bots> <option alacod-sim…> -- <graine>… ; une ligne TSV par graine :
graine, issue, frame de la mort, étage, salles verrouillées à cet instant (type#n), tueur (nom de l'ennemi).
Rejoue la graine avec `--log` (même options que la campagne) et lit le journal : `entity killed by NetId(n)` d'un
joueur, `(nom-n)` des ennemis, `room_state`."""
import subprocess, sys, re, os
bots = sys.argv[1]
sep = sys.argv.index('--')
extra, seeds = sys.argv[2:sep], sys.argv[sep + 1:]
sim = os.environ.get('SIM', os.environ['CARGO_TARGET_DIR'] + '/headless/alacod-sim')
prof = ','.join(['explorateur'] * int(bots))
for seed in seeds:
    out = subprocess.run([sim, '--game', 'isaac', '--bots', bots, '--profiles', prof, '--floors', 'sous_sol',
                          '--seeds', f'{seed}..{seed}', '--until-floor', '5', '--max-frames', '20000', '--log', *extra],
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True).stdout
    rooms, names, floor, death, killer, res = {}, {}, 0, None, None, '?'
    for l in out.splitlines():
        f = re.search(r'ggrs\{f=(\d+)', l)
        m = re.search(r'room_state room=(\d+) kind=(\w+) state=(\w+)', l)
        if m and death is None:
            rooms[m[1]] = (m[2], m[3])
        for n in re.finditer(r'\((\w+)-(\d+)\)', l):
            names[n[2]] = n[1]
        if 'floor_transition' in l and death is None:
            floor += 1
        k = re.search(r'entity killed by NetId\((\d+)\)', l)
        if k and 'player-' in l and death is None:
            killer, death = k[1], int(f[1]) if f else -1
        s = re.search(r'^seed\s+(\d+).*fin (\S+)', l)
        if s:
            res = s[2]
    locked = ','.join(f'{k}#{i}' for i, (k, st) in sorted(rooms.items()) if st == 'Locked') or '-'
    print(seed, res, death, floor, locked, names.get(killer, '?'), sep='\t', flush=True)
