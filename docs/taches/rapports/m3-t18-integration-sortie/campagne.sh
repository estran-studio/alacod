#!/bin/bash
# M3-T18 (critère 4 du §8.1) : campagnes `alacod-sim --game isaac` sur la séquence par défaut (`sous_sol` puis `sous_sol_profond`, sans
# `FloorsOverride`), politique de `games/isaac/assets/bot.ron`, sans `--bombs` (bombe de départ réelle), `--pickup-progress` (comme T14,
# T11d1, T11d2). Ajouts : `--events` (moments clés par graine, pour la couverture) et `--log` filtré au vol par `flux.py` (tueur de chaque
# joueur mort, salles verrouillées, étages à salle du diable) : aucun rejeu.
# usage : campagne.sh <préfixe> <bots> <de> <à> [options alacod-sim…]   (lots de 20, reprenable ; depuis la racine du dépôt)
#   s 1 1 200 : s1     d 2 1 200 : d2     q 4 1 20 : q4        sortie : $OUT (défaut /tmp/m3-t18/camp)
P=$1; B=$2; LO=$3; HI=$4; shift 4
HERE=$(cd "$(dirname "$0")" && pwd)
SIM=${SIM:-$CARGO_TARGET_DIR/headless/alacod-sim}
export DYLD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath) LD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)
PROF=explorateur; [ "$B" -ge 2 ] && PROF=$(printf 'explorateur,%.0s' $(seq 1 $B) | sed 's/,$//')
OUT=${OUT:-/tmp/m3-t18/camp}; mkdir -p $OUT/events
for s in $(seq $LO 20 $HI); do
  e=$((s+19)); [ $e -gt $HI ] && e=$HI
  f=$(printf "%s%s_%03d-%03d" "$P" "$B" $s $e)
  [ -s $OUT/$f.json ] && continue
  RUST_LOG=${RUST_LOG:-info} $SIM --game isaac --bots $B --profiles $PROF --floors sous_sol --seeds $s..$e --until-floor 5 --max-frames 20000 \
    --pickup-progress --log --events $OUT/events/$P$B "$@" --json $OUT/$f.tmp.json 2>&1 | python3 $HERE/flux.py $OUT/$f.morts.tsv > $OUT/$f.log
  mv $OUT/$f.tmp.json $OUT/$f.json
  echo "lot $f : $(grep -c '^seed' $OUT/$f.log) graines ; $(grep 'graine(s)' $OUT/$f.log | grep -E 'victoire|mort|soft-lock|plafond|desync' | tr '\n' ' ')" >> $OUT/progress.txt
done
echo "FINI $P$B $LO..$HI $(date +%H:%M:%S)" >> $OUT/progress.txt
