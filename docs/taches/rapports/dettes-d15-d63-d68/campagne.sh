#!/usr/bin/env bash
# Depuis la racine : CARGO_TARGET_DIR=<target> bash docs/taches/rapports/dettes-d15-d63-d68/campagne.sh
# Le binaire doit être compilé avant la campagne. Chaque lot est joué au premier plan, commité et poussé.
set -euo pipefail
branch=dettes-d15-d63-d68
[[ $(git branch --show-current) == "$branch" ]]
report=docs/taches/rapports/dettes-d15-d63-d68
binary=${CARGO_TARGET_DIR:-./target}/headless/alacod-sim
export DYLD_LIBRARY_PATH="$(./scripts/dev-features.sh libpath)"
export LD_LIBRARY_PATH="$DYLD_LIBRARY_PATH"
git push origin "$branch"
for ((start=1; start<=191; start+=10)); do
    end=$((start+9))
    file=$(printf '%s/e2_%03d-%03d.json' "$report" "$start" "$end")
    if [[ -s "$file" ]] && git ls-files --error-unmatch "$file" >/dev/null 2>&1; then continue; fi
    "$binary" --game gungeon --bots 2 --profiles explorateur,explorateur --floors etages_deux \
        --seeds "$start..$end" --until-floor 1 --max-frames 20000 --json "$file" \
        > "${TMPDIR:-/tmp}/dettes-e2-$start-$end.log" 2>&1
    python3 - "$file" "$start" <<'PY'
import json, sys, collections
rows = json.load(open(sys.argv[1]))
start = int(sys.argv[2])
assert [r['seed'] for r in rows] == list(range(start, start + 10))
assert all(not r['desync'] and not r.get('failures') and r['outcome'] in ['mort', 'objectif', 'victoire'] for r in rows), [(r['seed'], r['outcome']) for r in rows]
print(sys.argv[1], dict(collections.Counter(r['outcome'] for r in rows)), flush=True)
PY
    git check-ignore -q "$file" && exit 1
    git add "$file"
    git commit -m "D68 : campagne e2, graines $start à $end" \
        -m 'Co-Authored-By: Codex <noreply@openai.com>' -- "$file"
    git push origin "$branch"
done
