#!/usr/bin/env bash
# M3-T11d1 : campagne de contrôle d'`isaac` (séquence par défaut), lots de 20 graines au plus.
#
#   docs/taches/rapports/m3-t11d1-gabarits.campagne.sh <bots> <de> <à> [--no-shopping]   un lot (JSON + journal dans $OUT)
#   docs/taches/rapports/m3-t11d1-gabarits.campagne.sh tout                                200 graines à 1 bot puis 200 à 2 bots
#   docs/taches/rapports/m3-t11d1-gabarits.campagne.sh resume                              tableau (issues, desync, soft-locks, victoires)
#
# `--floors sous_sol` est la séquence `entry.floors` du manifeste (deux étages, victoire à la fin) ; `--shopping` : les bots
# ouvrent les coffres et achètent (D62). Sortie : OUT (/tmp/t11d1/camp par défaut).
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT="${OUT:-/tmp/t11d1/camp}"
mkdir -p "$OUT"
cd "$ROOT"

lot() {
  local bots=$1 from=$2 to=$3
  local profiles shop="--shopping"
  profiles=$(python3 -c "print(','.join(['explorateur']*$bots))")
  [ "${4:-}" = "--no-shopping" ] && shop=""
  local out="$OUT/b${bots}_${from}_${to}"
  cargo run -q -p scenario --bin alacod-sim --profile headless -- --game isaac --bots "$bots" --profiles "$profiles" $shop \
    --floors sous_sol --until-floor 2 --seeds "${from}..${to}" --max-frames 20000 --json "$out.json" > "$out.log" 2>&1
  echo "exit=$?" >> "$out.log"
}

case "${1:-}" in
  tout)
    for bots in 1 2; do
      for from in $(seq 1 20 181); do
        to=$((from + 19))
        [ -f "$OUT/b${bots}_${from}_${to}.json" ] && continue
        lot "$bots" "$from" "$to"
        echo "lot b${bots} ${from}..${to} fini $(date +%H:%M:%S)" >> "$OUT/progress.txt"
      done
    done ;;
  resume)
    python3 - "$OUT" <<'PY'
import collections, glob, json, sys
out = sys.argv[1]
for bots in (1, 2):
    rows = []
    for f in sorted(glob.glob(f'{out}/b{bots}_*.json')):
        rows += json.load(open(f))
    if not rows:
        continue
    rows.sort(key=lambda r: r['seed'])
    issues = collections.Counter(r['outcome'] for r in rows)
    print(f'--- {bots} bot(s) : {len(rows)} graines ({rows[0]["seed"]} à {rows[-1]["seed"]})')
    print('issues', dict(issues), '| desync', [r['seed'] for r in rows if r['desync']])
    print('soft-locks', [r['seed'] for r in rows if r['outcome'] == 'soft-lock'])
    print('second étage atteint', sum(1 for r in rows if r['floor'] >= 1), '| victoires', [r['seed'] for r in rows if r['outcome'] == 'victoire'])
PY
    ;;
  *) lot "$@" ;;
esac
