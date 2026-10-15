#!/bin/bash
# usage: campagne.sh <prefixe> <bots> <sequence>
cd /home/user/alacod
export LD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)
P=$1; B=$2; SEQ=$3; PROF=explorateur; [ $B = 2 ] && PROF=explorateur,explorateur
D=docs/taches/rapports/m2-t16-campagne-boss
for s in $(seq 1 10 191); do
  e=$((s+9)); f=$(printf "%s%s_%03d-%03d.json" "$P" "$B" $s $e)
  [ -s $D/$f ] && continue
  ./target/headless/alacod-sim --game gungeon --bots $B --profiles $PROF --floors $SEQ --seeds $s..$e --until-floor 1 --max-frames 20000 --json $D/$f > $D/${f%.json}.log 2>&1
  tail -12 $D/${f%.json}.log | grep -E "^(seed|alacod-sim|  )" > /dev/null
  rm -f $D/${f%.json}.log.tmp
  (
    flock 9
    git add $D/$f
    git commit -q -m "m2-t16-campagne-boss : lot $f"
    for i in 1 2 3 4; do git push -q -u origin m2-t16-campagne-boss && break || sleep $((2**i)); done
  ) 9>/tmp/claude-0/git.lock
done
echo FINI $P$B >> /tmp/claude-0/campagnes.status
