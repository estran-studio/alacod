"""usage: morts.py <bots> <sequence> <seed>... ; une ligne TSV par graine : seed outcome frame salle_active(kind#id) tueur"""
import subprocess,sys,re,os,json
bots,seq=sys.argv[1],sys.argv[2]
prof='explorateur' if bots=='1' else 'explorateur,explorateur'
env=dict(os.environ)
for seed in sys.argv[3:]:
    out=subprocess.run([os.environ['CARGO_TARGET_DIR']+'/headless/alacod-sim','--game','gungeon','--bots',bots,'--profiles',prof,'--floors',seq,'--seeds',f'{seed}..{seed}','--until-floor','1','--max-frames','20000','--log'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=env,cwd='.').stdout
    rooms={};seen=set();death=None;killer=None;names={}
    for l in out.splitlines():
        m=re.search(r'room_state room=(\d+) kind=(\w+) state=(\w+) enemies=(\d+)',l)
        f=re.search(r'ggrs\{f=(\d+)',l)
        if m: rooms[m[1]]=(m[2],m[3],int(f[1]) if f else -1)
        for n in re.finditer(r'\((\w+)-(\d+)\)',l): names[n[2]]=n[1]
        k=re.search(r'entity killed by NetId\((\d+)\)',l)
        if k and 'player-' in l and death is None: killer=k[1]; death=int(f[1])
        s=re.search(r'^seed\s+(\d+).*frames\s+(\d+).*fin (\S+)',l)
        if s: res=(s[3],int(s[2]))
    act=[f'{k}#{i}' for i,(k,s,fr) in sorted(rooms.items()) if s=='Locked']
    print(seed,res[0] if 'res' in dir() else '?',death,','.join(act) or '-',names.get(killer,'?'),sep='\t',flush=True)
