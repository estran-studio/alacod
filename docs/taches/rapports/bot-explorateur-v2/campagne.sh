#!/bin/bash
# usage: campagne.sh <prefixe> <bots> <sequence> [binaire]   (lots de 10 graines, un JSON par lot, commité et poussé)
# Reprenable : saute les lots déjà présents. Depuis la racine du dépôt.
P=$1; B=$2; SEQ=$3; SIM=${4:-./target/headless/alacod-sim}
export DYLD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath) LD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)
PROF=explorateur; [ $B = 2 ] && PROF=explorateur,explorateur
D=docs/taches/rapports/bot-explorateur-v2
LOCK=${TMPDIR:-/tmp}/bot-explorateur-v2.git.lock
for s in $(seq 1 10 191); do
  e=$((s+9)); f=$(printf "%s%s_%03d-%03d.json" "$P" "$B" $s $e)
  [ -s $D/$f ] && continue
  $SIM --game gungeon --bots $B --profiles $PROF --floors $SEQ --seeds $s..$e --until-floor 1 --max-frames 20000 --json $D/$f > ${TMPDIR:-/tmp}/${f%.json}.log 2>&1
  until mkdir $LOCK 2>/dev/null; do sleep 1; done
  git add $D/$f
  git commit -q -m "bot-explorateur-v2 : campagne, lot $f" -- $D/$f
  for i in 1 2 3 4; do git push -q -u origin bot-explorateur-v2 && break || { git pull -q --rebase origin bot-explorateur-v2; sleep $((2**i)); }; done
  rmdir $LOCK
done
echo FINI $P$B
