#!/usr/bin/env bash
# Suite de scénarios en N tranches parallèles (même résultat que `make test_scenarios`, ~N fois
# plus vite sur une machine libre).
#
#   scripts/test-scenarios-sharded.sh [N]        (défaut : SHARDS, sinon 4)
#
# Compile une fois le binaire de test `scenarios` (mêmes options que `make test_scenarios`), puis
# le lance N fois avec `ALACOD_SCENARIO_SHARD=i/N` : chaque processus joue un scénario sur N.
# `cargo test` garde le verrou du target pendant toute l'exécution des tests : N `cargo test`
# en parallèle s'attendraient, d'où le lancement direct du binaire. Journaux par tranche dans
# `<target>/scenario-shards/shard-<i>.log`, métriques fusionnées en `metrics.json`/`latest.json`.
# Variables respectées : ALACOD_BLESS, ALACOD_SCENARIO, ALACOD_BENCH_STRICT, ALACOD_EVENTS…
set -euo pipefail

shards="${1:-${SHARDS:-4}}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target}"
features="$(./scripts/dev-features.sh features)"

# shellcheck disable=SC2086
binary="$(cargo test -p scenario --profile headless $features --test scenarios --no-run \
    --message-format=json 2>/dev/null \
    | python3 -c '
import json, sys
for line in sys.stdin:
    try:
        msg = json.loads(line)
    except ValueError:
        continue
    if msg.get("reason") == "compiler-artifact" and msg["target"]["name"] == "scenarios" \
            and msg.get("executable"):
        print(msg["executable"])
')"
if [ -z "$binary" ]; then
    echo "test-scenarios-sharded : binaire de test introuvable (relancer cargo test --no-run pour voir l'erreur)" >&2
    exit 2
fi

# Ce que `cargo test` fournit au binaire : bibliothèques partagées (Bevy en dylib, libstd) et
# dossier du paquet comme répertoire courant.
libpath="$(./scripts/dev-features.sh libpath 2>/dev/null || true)"
export LD_LIBRARY_PATH="${libpath:+$libpath:}${LD_LIBRARY_PATH:-}"
export APP_VERSION="${APP_VERSION:-${TEST_VERSION:-v0.0.0}}"

logs="$CARGO_TARGET_DIR/scenario-shards"
mkdir -p "$logs"
started=$(date +%s)
pids=()
for ((i = 0; i < shards; i++)); do
    (cd crates/scenario && ALACOD_SCENARIO_SHARD="$i/$shards" \
        "$binary" --exact scenarios --nocapture >"$logs/shard-$i.log" 2>&1) &
    pids+=("$!")
done

status=0
for ((i = 0; i < shards; i++)); do
    if wait "${pids[$i]}"; then
        echo "tranche $i/$shards : ok"
    else
        echo "tranche $i/$shards : ÉCHEC (journal : $logs/shard-$i.log)"
        status=1
    fi
done

# Échecs de toutes les tranches, comme les afficherait la suite en un seul processus.
# (le test liste ses échecs dans son message de panique, jusqu'à la note de backtrace).
if [ "$status" -ne 0 ]; then
    for log in "$logs"/shard-*.log; do
        sed -n '/panicked at/,/^note:/p' "$log"
    done
fi

python3 - "$CARGO_TARGET_DIR/metrics" "$shards" <<'PY'
import glob, json, os, sys
metrics_dir, shards = sys.argv[1], sys.argv[2]
parts = sorted(glob.glob(os.path.join(metrics_dir, "*", f"metrics.shard-*-of-{shards}.json")),
               key=os.path.getmtime)
if not parts:
    sys.exit(0)
commit_dir = os.path.dirname(parts[-1])
merged = {}
for part in glob.glob(os.path.join(commit_dir, f"metrics.shard-*-of-{shards}.json")):
    merged.update(json.load(open(part)))
    os.remove(part)
text = json.dumps(dict(sorted(merged.items())), indent=2)
for path in (os.path.join(commit_dir, "metrics.json"), os.path.join(metrics_dir, "latest.json")):
    with open(path, "w") as f:
        f.write(text)
print(f"métriques : {len(merged)} scénarios fusionnés dans {os.path.relpath(commit_dir)}")
PY
echo "suite en $shards tranches : $(($(date +%s) - started)) s"
exit "$status"
