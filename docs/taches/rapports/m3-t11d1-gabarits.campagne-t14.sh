#!/bin/bash
# M3-T11d1, intégration de T14 (D106) : rejoue les campagnes de `m3-t14-bot-isaac` (mêmes options, `--pickup-progress`) sur le contenu
# livré (séquence par défaut `sous_sol`, bombe de départ réelle, boutique, rochers, piques, curses), SANS `--bombs`.
# usage : m3-t11d1-gabarits.campagne-t14.sh <prefixe> <bots> <de> <à> [options alacod-sim…]   (lots de 20, reprenable)
#   s 1 1 200 : s1 (1 bot)   i 2 1 200 : i2 (2 bots)   i 4 1 20 : i4 (4 bots)     sortie : $OUT (/tmp/t11d1/camp2)
P=$1; B=$2; LO=$3; HI=$4; shift 4
SIM=${SIM:-$CARGO_TARGET_DIR/headless/alacod-sim}
export DYLD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath) LD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)
PROF=explorateur; [ "$B" -ge 2 ] && PROF=$(printf 'explorateur,%.0s' $(seq 1 $B) | sed 's/,$//')
OUT=${OUT:-/tmp/t11d1/camp2}; mkdir -p $OUT
for s in $(seq $LO 20 $HI); do
  e=$((s+19)); [ $e -gt $HI ] && e=$HI
  f=$(printf "%s%s_%03d-%03d.json" "$P" "$B" $s $e)
  [ -s $OUT/$f ] && continue
  $SIM --game isaac --bots $B --profiles $PROF --floors sous_sol --seeds $s..$e --until-floor 5 --max-frames 20000 --pickup-progress "$@" --json $OUT/$f > $OUT/${f%.json}.log 2>&1
  echo "lot $f : $(grep -c '^seed' $OUT/${f%.json}.log) graines ; $(grep -E 'victoire|mort|soft-lock|plafond|desync' $OUT/${f%.json}.log | grep 'graine(s)' | tr '\n' ' ')" >> $OUT/progress.txt
done
echo "FINI $P$B $(date +%H:%M:%S)" >> $OUT/progress.txt
