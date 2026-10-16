#!/bin/bash
# usage: campagne.sh <prefixe> <bots> <sequence> [jobs]   (lots de 10 graines, un JSON par lot)
# Reprenable : saute les lots déjà présents. Depuis la racine du dépôt. `jobs` lots en parallèle (défaut 1).
# Variables : OUT (dossier des JSON, défaut ce dossier), LO/HI (bornes des lots, défaut 1 et 191).
# Dérivé de bot-explorateur-v2/campagne.sh (branche m2-t14d-gabarits) ; commit et push à la main.
P=$1; B=$2; SEQ=$3; J=${4:-1}
SIM=${SIM:-$CARGO_TARGET_DIR/headless/alacod-sim}
PROF=explorateur; [ $B = 2 ] && PROF=explorateur,explorateur
D=${OUT:-docs/taches/rapports/m2-t14d-gabarits}; mkdir -p $D
T=${TMPDIR:-/tmp}/t14d; mkdir -p $T
lot() {
  s=$1; e=$((s+9)); f=$(printf "%s%s_%03d-%03d.json" "$P" "$B" $s $e)
  [ -s $D/$f ] && return
  $SIM --game gungeon --bots $B --profiles $PROF --floors $SEQ --seeds $s..$e --until-floor 1 --max-frames 20000 --json $D/$f > $T/${f%.json}.log 2>&1
}
export -f lot; export P B SEQ SIM PROF D T
seq ${LO:-1} 10 ${HI:-191} | xargs -P $J -I{} bash -c 'lot {}'
echo FINI $P$B
