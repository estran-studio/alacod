#!/bin/bash
# p2p gungeon à deux clients (README §4), 600 frames, script d'inputs à deux joueurs. Depuis la racine.
set -uo pipefail
export MATCHBOX_PORT=3566 COMPOSE_PROJECT_NAME=b2
OUT=${OUT:-/tmp/p2p-gungeon}; mkdir -p $OUT; rm -f $OUT/*
SCRIPT=$PWD/docs/taches/rapports/m2-t14d-gabarits/p2p_gungeon.ron
BIN=$CARGO_TARGET_DIR/headless/gungeon
docker compose -f docker-compose.ci.yaml up -d signaling
trap 'docker compose -f docker-compose.ci.yaml down' EXIT
sleep 3
run() { i=$1
  ALACOD_HEADLESS=1 ALACOD_INPUT_DELAY=0 ALACOD_INPUT=script:$SCRIPT ALACOD_STATE_TRACE=$OUT/p2p-$i.trace ALACOD_EXIT_AT_FRAME=600 APP_VERSION=x \
  $BIN --matchbox ws://127.0.0.1:$MATCHBOX_PORT --lobby gungeon-$$ --number-player 2 --players localhost remote --cid client_$i --name client_$i > $OUT/p2p-$i.log 2>&1; }
run 0 & p0=$!; sleep 8; run 1 & p1=$!
wait $p0; e0=$?; wait $p1; e1=$?
echo "exit $e0 $e1"
wc -l $OUT/p2p-0.trace $OUT/p2p-1.trace
cmp $OUT/p2p-0.trace $OUT/p2p-1.trace && echo "cmp: identiques"
for i in 0 1; do
  echo "client $i : frames resimulées (ggrs{f=N} en double) :" $(grep -o 'ggrs{f=[0-9]*' $OUT/p2p-$i.log | sort | uniq -c | awk '$1>1' | wc -l) ; grep -iE "resim|DesyncDetected|DESYNC" $OUT/p2p-$i.log | tail -2
done
