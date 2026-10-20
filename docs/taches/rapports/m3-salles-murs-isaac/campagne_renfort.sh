#!/bin/bash
# Renfort de `campagne.sh` : mêmes lots, parcourus **à l'envers** (le lot de graines le plus haut d'abord) par des workers supplémentaires ; un lot déjà commencé
# (`.log` présent) ou fini (`.json`) n'est pas repris. Les workers de `campagne.sh` (ordre croissant) et ceux-ci se rejoignent au milieu.
# usage : campagne_renfort.sh <préfixe> <bots> <de> <à> [workers]
P=$1; B=$2; LO=$3; HI=$4; W=${5:-2}
SIM=${SIM:-$CARGO_TARGET_DIR/headless/alacod-sim}
OUT=${OUT:-/tmp/b6-camp}; mkdir -p $OUT/events
PROF=explorateur; [ "$B" -ge 2 ] && PROF=$(printf 'explorateur,%.0s' $(seq 1 $B) | sed 's/,$//')
export SIM OUT P B PROF HI
lot() {
  s=$1; e=$((s+19)); [ $e -gt $HI ] && e=$HI
  f=$(printf "%s%s_%03d-%03d" "$P" "$B" $s $e)
  [ -e $OUT/$f.log ] && return
  : > $OUT/$f.log
  RUST_LOG=error $SIM --game isaac --bots $B --profiles $PROF --floors sous_sol --seeds $s..$e --until-floor 5 --max-frames 20000 \
    --pickup-progress --explored-progress --events $OUT/events/$P$B --json $OUT/$f.tmp.json > $OUT/$f.log 2>&1
  mv $OUT/$f.tmp.json $OUT/$f.json
  echo "lot $f : $(grep -c '^seed' $OUT/$f.log) graines ; $(grep 'graine(s)' $OUT/$f.log | grep -E 'victoire|mort|soft-lock|plafond|desync' | tr '\n' ' ')" >> $OUT/progress.txt
}
export -f lot
seq $LO 20 $HI | sort -rn | xargs -P $W -I{} bash -c 'lot {}'
echo "FINI renfort $P$B $LO..$HI $(date +%H:%M:%S)" >> $OUT/progress.txt
