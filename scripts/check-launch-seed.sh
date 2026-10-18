#!/usr/bin/env bash
# D105 : `--seed` a un consommateur dans chaque jeu (`map_ldtk::game::local::apply_launch_seed`, installé par
# `LdtkLocalGamePlugin`). Pour chaque jeu, trois parties locales headless (`--seed A`, `--seed B`, `--seed A`)
# écrivent leur trace d'état ; la frame 0 doit différer entre A et B et être identique entre les deux A.
#
# Usage : scripts/check-launch-seed.sh [jeu…]   (tous les jeux de `games/` par défaut)
# Variables : SEED_A (1), SEED_B (2), P2P_NO_BUILD=1 (binaires déjà construits), OUT (/tmp/launch-seed).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1
SEED_A=${SEED_A:-1}
SEED_B=${SEED_B:-2}
OUT=${OUT:-/tmp/launch-seed}
mkdir -p "$OUT"
GAMES=("$@")
[ "${#GAMES[@]}" -gt 0 ] || GAMES=(zombies gungeon isaac throne testbed)

features="$(./scripts/dev-features.sh features)"
if [ "${P2P_NO_BUILD:-0}" != 1 ]; then
    pkgs=()
    for g in "${GAMES[@]}"; do pkgs+=(-p "$g"); done
    # shellcheck disable=SC2086
    cargo build -q "${pkgs[@]}" --profile headless --no-default-features $features || exit 1
fi
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
libpath="$(./scripts/dev-features.sh libpath 2>/dev/null || true)"
export LD_LIBRARY_PATH="${libpath:+$libpath:}${LD_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="${libpath:+$libpath:}${DYLD_LIBRARY_PATH:-}"

# Frame 0 (« 0 <hash> <entités> ») d'une partie locale à la graine $2.
frame0() {
    local game=$1 seed=$2 trace="$OUT/$1-$2-$3.trace"
    ALACOD_HEADLESS=1 ALACOD_INPUT=neutral ALACOD_SKIP_HUB=1 ALACOD_STATE_TRACE="$trace" ALACOD_EXIT_AT_FRAME=2 \
        timeout 300 "$TARGET/headless/$game" --seed "$seed" > "$OUT/$game-$seed-$3.log" 2>&1 || return 1
    sed -n 1p "$trace"
}

fail=0
for game in "${GAMES[@]}"; do
    a1=$(frame0 "$game" "$SEED_A" 1) && b=$(frame0 "$game" "$SEED_B" 1) && a2=$(frame0 "$game" "$SEED_A" 2) || {
        echo "ÉCHEC $game : partie impossible (voir $OUT/$game-*.log)"; fail=1; continue
    }
    verdict=OK
    [ "$a1" = "$a2" ] || { verdict="ÉCHEC (même graine, frames 0 différentes)"; fail=1; }
    [ "$a1" != "$b" ] || { verdict="ÉCHEC (graines $SEED_A et $SEED_B : même frame 0)"; fail=1; }
    echo "$game : seed $SEED_A → $a1 ; seed $SEED_B → $b ; seed $SEED_A (bis) → $a2 : $verdict"
done
[ "$fail" = 0 ] && echo "check-launch-seed : OK" || echo "check-launch-seed : ÉCHEC"
exit "$fail"
