import json,glob,sys,statistics as st,collections
p=sys.argv[1]; rep=sys.argv[2] if len(sys.argv)>2 else 'bot-explorateur-v2'
R=[]
for f in sorted(glob.glob(f'docs/taches/rapports/{rep}/{p}_*.json')): R+=json.load(open(f))
oc=collections.Counter(r['outcome'] for r in R)
print(p,len(R),dict(oc),'desync',sum(r['desync'] for r in R))
ok=[r['frames'] for r in R if r['outcome']=='objectif']
if ok: print(' frames objectif min/med/max',min(ok),int(st.median(ok)),max(ok))
m=[r for r in R if r['outcome']!='objectif']
print(' echecs',[(r['seed'],r['outcome'],r['frames'],r['floor']) for r in m])
print(' degats pris med',st.median(r['damage_taken'] for r in R))
print(' rooms etage0 visited==total',sum(1 for r in R if r['rooms'][0]['visited']==r['rooms'][0]['total']),'/',len(R))
