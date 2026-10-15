"""usage: arene.py <bots> <sequence> <seed>... ; une ligne TSV par graine : seed issue frame_verrou_arene frame_nettoyage_arene salles_non_verrouillantes_visitees"""
import subprocess,sys,re,os
bots,seq=sys.argv[1],sys.argv[2]
prof='explorateur' if bots=='1' else 'explorateur,explorateur'
sim=os.environ.get('SIM','./target/headless/alacod-sim')
# macOS : SIP retire DYLD_* de l'environnement de python3 ; la valeur arrive sous un autre nom.
env=dict(os.environ)
if 'SIM_LIBPATH' in env: env['DYLD_LIBRARY_PATH']=env['SIM_LIBPATH']
for seed in sys.argv[3:]:
    out=subprocess.run([sim,'--game','gungeon','--bots',bots,'--profiles',prof,'--floors',seq,'--seeds',f'{seed}..{seed}','--until-floor','1','--max-frames','20000','--log'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,env=env).stdout
    out=re.sub(r'\x1b\[[0-9;]*m','',out)
    lock=clear=None
    for l in out.splitlines():
        m=re.search(r'ggrs\{f=(\d+) room_state room=\d+ kind=arene state=(\w+)',l)
        if m:
            if m[2]=='Locked' and lock is None: lock=int(m[1])
            if m[2]=='Cleared' and clear is None: clear=int(m[1])
    s=re.search(r'^seed\s+(\d+).*fin (.+)$',out,re.M)
    print(seed,s[2].strip() if s else '?',lock,clear,sep='\t',flush=True)
