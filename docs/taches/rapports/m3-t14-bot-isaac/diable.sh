#!/bin/bash
# usage: diable.sh <politique> <graine>…   (politique : never | above:4 | above:2 …)
# Rejoue les graines à salle du diable (1 bot, kit d'une bombe) avec `--devil <politique>` ; un JSON par appel
# (`diable_<politique>.json`, dossier de ce script) : comparaison de la règle du diable (Q14).
POL=$1; shift
SIM=${SIM:-$CARGO_TARGET_DIR/headless/alacod-sim}
export DYLD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)
D=docs/taches/rapports/m3-t14-bot-isaac
T=${TMPDIR:-/tmp}/m3-t14-diable; mkdir -p $T
OUT=$D/diable_${POL//:/_}.json
echo "[]" > $T/acc.json
for s in "$@"; do
  $SIM --game isaac --bots 1 --profiles explorateur --floors sous_sol --seeds $s..$s --until-floor 5 --max-frames 20000 --pickup-progress --bombs 1 --devil $POL --json $T/one.json > $T/one.log 2>&1
  python3 -c "
import json
a=json.load(open('$T/acc.json')); a+=json.load(open('$T/one.json')); json.dump(a,open('$T/acc.json','w'))"
done
cp $T/acc.json $OUT
echo FINI $OUT
