"""usage: resume.py <dossier> <prefixe> : issues, desyncs, dégâts et salles d'une campagne (JSON de alacod-sim)."""
import json, glob, sys, statistics as st, collections
d, p = sys.argv[1], sys.argv[2]
R = []
for f in sorted(glob.glob(f'{d}/{p}_*.json')):
    R += json.load(open(f))
oc = collections.Counter(r['outcome'] for r in R)
print(p, len(R), dict(oc), 'desync', sum(r['desync'] for r in R))
if not R:
    sys.exit()
ok = [r['frames'] for r in R if r['outcome'] == 'objectif']
if ok:
    print(' frames objectif min/med/max', min(ok), int(st.median(ok)), max(ok))
print(' degats pris moy/med', int(st.mean(r['damage_taken'] for r in R)), st.median(r['damage_taken'] for r in R))
print(' mort a l etage 0 / 1', sum(1 for r in R if r['outcome'] == 'mort' and r['floor'] == 0), sum(1 for r in R if r['outcome'] == 'mort' and r['floor'] == 1))
print(' non objectif hors mort', [(r['seed'], r['outcome'], r['frames']) for r in R if r['outcome'] not in ('objectif', 'mort')])
print(' salles etage0 visitees/total (moy)', round(st.mean(r['rooms'][0]['visited'] for r in R), 1), round(st.mean(r['rooms'][0]['total'] for r in R), 1))
