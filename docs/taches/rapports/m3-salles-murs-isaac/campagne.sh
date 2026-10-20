#!/bin/bash
# m3-salles-murs-isaac : campagnes `alacod-sim --game isaac` de T18 (docs/taches/rapports/m3-t18-integration-sortie/campagne.sh), rejouées sur les salles ×2.
# Mêmes réglages que T18 (séquence par défaut `sous_sol` puis `sous_sol_profond`, politique de `bot.ron`, bombe de départ réelle, `--pickup-progress`,
# `--until-floor 5`, `--max-frames 20000`, lots de 20 graines), SANS `--log` (le tueur de chaque joueur n'est pas relevé ici), et AVEC `--explored-progress` :
# les étages font maintenant plus de 3 000 px, la marche d'une salle à la suivante dépasse la fenêtre du détecteur de soft-lock (600 frames sans mort, dégât ni
# ramassage) sans un seul coup (graine 2 à 1 bot : 2 000 px de marche vers le boss, détectée à tort) ; entrer dans une case de 32 px jamais visitée est une
# progression (même correctif que gungeon, `m2-salles-murs-gungeon` §3).
# usage : campagne.sh <préfixe> <bots> <de> <à> [workers]   (depuis la racine du dépôt ; lots de 20 graines en parallèle ; reprenable)
#   s 1 1 200 : s1     d 2 1 200 : d2     q 4 1 20 : q4        sortie : $OUT (défaut /tmp/b6-camp), binaire : $SIM
P=$1; B=$2; LO=$3; HI=$4; W=${5:-4}
SIM=${SIM:-$CARGO_TARGET_DIR/headless/alacod-sim}
OUT=${OUT:-/tmp/b6-camp}; mkdir -p $OUT/events
PROF=explorateur; [ "$B" -ge 2 ] && PROF=$(printf 'explorateur,%.0s' $(seq 1 $B) | sed 's/,$//')
export SIM OUT P B PROF HI
lot() {
  s=$1; e=$((s+19)); [ $e -gt $HI ] && e=$HI
  f=$(printf "%s%s_%03d-%03d" "$P" "$B" $s $e)
  [ -s $OUT/$f.json ] && return
  RUST_LOG=error $SIM --game isaac --bots $B --profiles $PROF --floors sous_sol --seeds $s..$e --until-floor 5 --max-frames 20000 \
    --pickup-progress --explored-progress --events $OUT/events/$P$B --json $OUT/$f.tmp.json > $OUT/$f.log 2>&1
  mv $OUT/$f.tmp.json $OUT/$f.json
  echo "lot $f : $(grep -c '^seed' $OUT/$f.log) graines ; $(grep 'graine(s)' $OUT/$f.log | grep -E 'victoire|mort|soft-lock|plafond|desync' | tr '\n' ' ')" >> $OUT/progress.txt
}
export -f lot
seq $LO 20 $HI | xargs -P $W -I{} bash -c 'lot {}'
echo "FINI $P$B $LO..$HI $(date +%H:%M:%S)" >> $OUT/progress.txt
