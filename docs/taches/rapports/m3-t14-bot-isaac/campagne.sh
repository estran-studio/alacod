#!/bin/bash
# usage: campagne.sh <prefixe> <bots> <de> <à> [options alacod-sim…]
# Joue les graines de..à par lots de 20 (un JSON par lot, dossier de ce script), séquence par défaut d'isaac
# (`sous_sol`, deux étages), jusqu'à la victoire ou la mort (`--until-floor 5` n'est jamais atteint).
# Reprenable : saute les lots déjà présents. Depuis la racine du dépôt (CARGO_TARGET_DIR exporté).
# Exemple : docs/taches/rapports/m3-t14-bot-isaac/campagne.sh i 1 1 200 --bombs 1
P=$1; B=$2; LO=$3; HI=$4; shift 4
SIM=${SIM:-$CARGO_TARGET_DIR/headless/alacod-sim}
export DYLD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath) LD_LIBRARY_PATH=$(./scripts/dev-features.sh libpath)
PROF=explorateur; [ "$B" -ge 2 ] && PROF=$(printf 'explorateur,%.0s' $(seq 1 $B) | sed 's/,$//')
D=docs/taches/rapports/m3-t14-bot-isaac
T=${TMPDIR:-/tmp}/m3-t14; mkdir -p $T
for s in $(seq $LO 20 $HI); do
  e=$((s+19)); [ $e -gt $HI ] && e=$HI
  f=$(printf "%s%s_%03d-%03d.json" "$P" "$B" $s $e)
  [ -s $D/$f ] && continue
  $SIM --game isaac --bots $B --profiles $PROF --floors sous_sol --seeds $s..$e --until-floor 5 --max-frames 20000 --pickup-progress "$@" --json $D/$f > $T/${f%.json}.log 2>&1
  echo "lot $f : $(grep -c '^seed' $T/${f%.json}.log) graines ; $(grep -E 'victoire|mort|soft-lock|plafond|desync' $T/${f%.json}.log | grep 'graine(s)' | tr '\n' ' ')"
done
echo FINI $P$B
