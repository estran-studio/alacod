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
if ! [[ "$shards" =~ ^[1-9][0-9]*$ ]]; then
    echo "test-scenarios-sharded : nombre de tranches entier strictement positif requis" >&2
    exit 2
fi
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
export DYLD_LIBRARY_PATH="${libpath:+$libpath:}${DYLD_LIBRARY_PATH:-}"
export APP_VERSION="${APP_VERSION:-${TEST_VERSION:-v0.0.0}}"

logs="$CARGO_TARGET_DIR/scenario-shards"
mkdir -p "$logs"
started=$(date +%s)
commit="$(git rev-parse --short HEAD)"
if ! git diff --quiet; then commit="$commit-dirty"; fi
metrics_commit="$CARGO_TARGET_DIR/metrics/$commit"
mkdir -p "$metrics_commit"
# Une ancienne part ne doit jamais remplacer une tranche qui n'a rien écrit dans ce run.
rm -f "$metrics_commit"/metrics.shard-*-of-"$shards".json
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
    exit "$status"
fi

python3 - "$CARGO_TARGET_DIR/metrics" "$metrics_commit" "$shards" <<'PY'
import json, sys
from pathlib import Path
metrics_dir, commit_dir = map(Path, sys.argv[1:3])
shards = int(sys.argv[3])
parts = [commit_dir / f"metrics.shard-{i}-of-{shards}.json" for i in range(shards)]
merged = {}
for part in parts:
    # Absence, JSON invalide ou collision : échec, sans écraser latest.json.
    data = json.loads(part.read_text())
    if not isinstance(data, dict):
        raise ValueError(f"métriques invalides : {part}")
    duplicates = merged.keys() & data.keys()
    if duplicates:
        raise ValueError(f"scénarios dupliqués entre tranches : {sorted(duplicates)}")
    merged.update(data)
if not merged:
    raise ValueError("aucun scénario exécuté")
text = json.dumps(dict(sorted(merged.items())), indent=2)
for path in (commit_dir / "metrics.json", metrics_dir / "latest.json"):
    temporary = path.with_suffix(".json.tmp")
    temporary.write_text(text)
    temporary.replace(path)
for part in parts:
    part.unlink()
print(f"métriques : {len(merged)} scénarios fusionnés dans {commit_dir}")
PY
echo "suite en $shards tranches : $(($(date +%s) - started)) s"
exit "$status"
