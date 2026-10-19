"""usage : chutes.py <bots> <graine>…   (depuis la racine du dépôt, SIM = binaire d'alacod-sim)
M3-T18 : pour les graines perdues dont `flux.py` n'a pas trouvé de tueur (aucune ligne `entity killed by` pour le joueur : mort
par les piques, par sa propre bombe, ou à terre sans personne pour le relever), rejoue la graine avec `--log` (mêmes options que
`campagne.sh`) et écrit, par joueur, sa **dernière chute** (`receive … health is 0`) : graine, joueur, frame, source du coup
(`(nom-n)` de la ligne `resolve_damage` de la même frame ; `piques` si le coup porte le tag `spikes`, `soi` si la source est
le joueur lui-même sans ce tag) et les tags du coup."""
import os
import re
import subprocess
import sys

bots = sys.argv[1]
sim = os.environ.get('SIM', os.environ['CARGO_TARGET_DIR'] + '/headless/alacod-sim')
prof = ','.join(['explorateur'] * int(bots))
for seed in sys.argv[2:]:
    out = subprocess.run([sim, '--game', 'isaac', '--bots', bots, '--profiles', prof, '--floors', 'sous_sol', '--seeds',
                          f'{seed}..{seed}', '--until-floor', '5', '--max-frames', '20000', '--pickup-progress', '--log'],
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True).stdout
    source, fall = {}, {}
    for line in out.splitlines():
        f = re.search(r'ggrs\{f=(\d+)', line)
        d = re.search(r'\(player-(\d+)\) <- [\d.]+ dmg from \((\w+)-(\d+)\).*tags Tags\(\{(.*)\}\)', line)
        if d and f:
            who = 'piques' if 'spikes' in d[4] else ('soi' if d[2] == 'player' and d[3] == d[1] else d[2])
            source[(d[1], f[1])] = (who, d[4].replace('Tag(', '').replace(')', '').replace('"', ''))
        z = re.search(r'\(player-(\d+)\) receive [\d.]+ dmg health is 0\b', line)
        if z and f:
            fall[z[1]] = f[1]
    for player, frame in sorted(fall.items()):
        who, tags = source.get((player, frame), ('?', ''))
        print(seed, player, frame, who, tags, sep='\t', flush=True)
