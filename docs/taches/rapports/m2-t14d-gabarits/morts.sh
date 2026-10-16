#!/bin/bash
# usage: morts.sh <bots> <sequence> <debut> <fin> [jobs] : une ligne TSV par graine (morts.py), triée par graine
B=$1; SEQ=$2; A=$3; Z=$4; J=${5:-7}
seq $A $Z | xargs -P $J -I{} python3 docs/taches/rapports/m2-t14d-gabarits/morts.py $B $SEQ {} | sort -n
