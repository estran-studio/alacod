#!/bin/bash
# usage: run.sh N ; charge.txt : date load1 | autres = %CPU instantané (top) des processus hors bench (scenarios-*, cargo, make, run.sh, top, sampler)
cd /home/debian/orca/workers/b6/alacod
n=$1
L=/tmp/d84/charge-r$n.txt
: > $L
( while true; do
    l=$(cut -d' ' -f1 /proc/loadavg)
    o=$(python3 /tmp/d84/others.py)
    echo "$(date -u +%FT%TZ) $l $o" >> $L
    sleep 9
  done ) & S=$!
ALACOD_BENCH_STRICT=1 make bench > /tmp/d84/bench-r$n.log 2>&1
echo "exit=$?" >> /tmp/d84/bench-r$n.log
kill $S
cp $CARGO_TARGET_DIR/metrics/latest.json /tmp/d84/metrics-r$n.json 2>/dev/null
