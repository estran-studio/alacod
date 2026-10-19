import os,time,re
def snap():
    d={}
    for p in os.listdir('/proc'):
        if p.isdigit():
            try:
                s=open(f'/proc/{p}/stat').read()
                i=s.rindex(')'); f=s[i+2:].split()
                name=s[s.index('(')+1:i]
                cmd=open(f'/proc/{p}/cmdline').read().replace('\0',' ')[:60]
                d[p]=(int(f[11])+int(f[12]),name,cmd)
            except Exception: pass
    return d
a=snap(); time.sleep(1); b=snap()
ex=re.compile(r'scenarios-|cargo|^make|run\.sh|others\.py|python3 /tmp|rustc')
rows=[]
for p,(t,n,c) in b.items():
    if p in a and not ex.search(c) and not ex.search(n):
        rows.append(((t-a[p][0]),n,c[:30]))
rows.sort(reverse=True)
tot=sum(r[0] for r in rows)
print("autres=%d%% top:%s"%(tot," ".join("%d%%%s"%(r[0],r[1]) for r in rows[:3] if r[0])))
