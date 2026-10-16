"""usage: rencontre.py <bots> <sequence> <graine> [--shopping] : ce qu'une graine de `alacod-sim` fait rencontrer.

Joue la graine avec `--log` et rend, par mécanique de la fiche M2-T14d, la première frame où le journal la
montre (ou `-`) : escouade, arène (vagues), coffres (`--shopping` : les bots les ouvrent), boutique (achat),
baril qui explose, table renversée, chute dans une fosse, récompense de salle. Les types de salle viennent
du dump `rooms` du JSON (`--json`) et des lignes `room_state kind=…`.
"""
import json, os, re, subprocess, sys, tempfile

bots, seq, seed = sys.argv[1], sys.argv[2], sys.argv[3]
shopping = '--shopping' in sys.argv
prof = 'explorateur' if bots == '1' else 'explorateur,explorateur'
sim = os.environ['CARGO_TARGET_DIR'] + '/headless/alacod-sim'
out = tempfile.NamedTemporaryFile(suffix='.json', delete=False).name
cmd = [sim, '--game', 'gungeon', '--bots', bots, '--profiles', prof, '--floors', seq, '--seeds', f'{seed}..{seed}',
       '--until-floor', '1', '--max-frames', '20000', '--log', '--json', out] + (['--shopping'] if shopping else [])
log = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True).stdout
first = {}


def note(key, frame):
    if key not in first or frame < first[key]:
        first[key] = frame


for line in log.splitlines():
    m = re.search(r'ggrs\{f=(\d+) (\w+)(.*)\}', line)
    if not m:
        continue
    f, name, rest = int(m[1]), m[2], m[3]
    if name == 'room_state':
        k = re.search(r'kind=(\w+) state=(\w+)', rest)
        if k:
            note(f'{k[1]} {k[2]}', f)
    elif name in ('arena_wave', 'chest_open', 'shop_buy', 'barrel_explosion', 'table_flip', 'pit_fall', 'room_reward'):
        note(name, f)
res = json.load(open(out))[0]
print(f"graine {seed} ({bots} bot(s), {seq}{' --shopping' if shopping else ''}) : {res['outcome']} f{res['frames']}, "
      f"salles étage 0 : {res['rooms'][0]['visited']}/{res['rooms'][0]['total']} visitées, {res['rooms'][0]['cleared']} nettoyées")
for key in ('escouade Locked', 'escouade Cleared', 'arene Locked', 'arena_wave', 'arene Cleared', 'combat Cleared',
            'chest_open', 'shop_buy', 'barrel_explosion', 'table_flip', 'pit_fall', 'room_reward', 'boss Locked',
            'boss Cleared'):
    print(f'  {key:18s} {first.get(key, "-")}')
os.unlink(out)
