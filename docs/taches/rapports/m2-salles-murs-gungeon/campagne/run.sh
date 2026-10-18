#!/bin/bash
# usage: run.sh bots seeds outprefix [extra]
cd "$(dirname "$0")/../../../../.."
B=$1; S=$2; O=$3; shift 3
if [ "$B" = 1 ]; then P=explorateur; else P=explorateur,explorateur; fi
${CARGO_TARGET_DIR:-target}/headless/alacod-sim --game gungeon --bots $B --profiles $P --floors etages_deux --until-floor 5 --seeds $S --max-frames 20000 --shopping --explored-progress --json $O.json "$@" > $O.log 2>&1
