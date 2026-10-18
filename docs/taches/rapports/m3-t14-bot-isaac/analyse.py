"""usage: analyse.py <préfixe>  (ex. i1) : tableau d'une campagne (JSON du dossier de ce script)."""
import json, glob, sys, os, statistics as st, collections
here = os.path.dirname(os.path.abspath(__file__))
p = sys.argv[1]
R = []
for f in sorted(glob.glob(f'{here}/{p}_*.json')):
    R += json.load(open(f))
n = len(R)
oc = collections.Counter(r['outcome'] for r in R)
print(f'{p} : {n} graines', dict(oc), 'desync', sum(r['desync'] for r in R))
vic = [r['frames'] for r in R if r['outcome'] == 'victoire']
if vic:
    print(' frames de victoire min/méd/max', min(vic), int(st.median(vic)), max(vic))
floors = collections.Counter(r['floor'] for r in R)
print(' étage atteint (0 = premier)', dict(sorted(floors.items())))
print(' échecs', [(r['seed'], r['outcome'], r['frames'], r['floor']) for r in R if r['outcome'] != 'victoire'])
mech = collections.Counter()
seeds = collections.defaultdict(list)
for r in R:
    for k, v in r.get('mechanics', {}).items():
        mech[k] += v
        seeds[k].append(r['seed'])
print(' mécaniques (total ; graines qui la rencontrent)')
for k in sorted(mech):
    print(f'   {k:12} {mech[k]:4}  graines {len(seeds[k]):3}  ex. {seeds[k][:6]}')
print(' dégâts pris médiane', st.median(r['damage_taken'] for r in R) if R else 0)
