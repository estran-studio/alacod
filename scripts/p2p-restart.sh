#!/usr/bin/env bash
# Restart en ligne (D14, docs/conventions.md §33) : deux clients p2p jouent une partie, la
# relancent (`ALACOD_RESTART_AT_FRAME`, comme « Rejouer ») et jouent la seconde ; leurs traces
# doivent être identiques partie par partie. Puis un client local (synctest) fait de même :
# même graine au restart local, donc ses deux parties doivent être identiques entre elles
# (preuve que la sortie de partie remet tout à zéro).
#
# Usage : ./scripts/p2p-restart.sh   (signaling docker du README §4 démarré et arrêté ici)
# Bash, pas zsh (les `wait $pid` d'une variable non éclatée échouent en zsh).
set -euo pipefail

RESTART_AT=${RESTART_AT:-600}
EXIT_AT=${EXIT_AT:-600}
OUT=${OUT:-/tmp/p2p-restart}
LOBBY="restart-$$"
mkdir -p "$OUT"
rm -f "$OUT"/*.trace-g* "$OUT"/*.log

docker compose -f docker-compose.ci.yaml up -d signaling
trap 'docker compose -f docker-compose.ci.yaml down' EXIT
sleep 3
cargo build -q -p zombies --profile headless --no-default-features

run_client() {
    local i=$1
    ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" \
    ALACOD_RESTART_AT_FRAME="$RESTART_AT" APP_VERSION=x \
    cargo run -q -p zombies --profile headless --no-default-features -- \
        --matchbox ws://127.0.0.1:3536 --lobby "$LOBBY" --number-player 2 \
        --players localhost remote --cid "client_$i" --name "client_$i" \
        > "$OUT/p2p-$i.log" 2>&1
}

pids=()
for i in 0 1; do
    run_client "$i" &
    pids+=("$!")
done
status=0
for pid in "${pids[@]}"; do
    wait "$pid" || status=1
done
[ "$status" -eq 0 ] || { echo "p2p : un client a échoué (voir $OUT/p2p-*.log)"; exit 1; }

fail=0
for g in 1 2; do
    a="$OUT/p2p-0.trace-g$g"
    b="$OUT/p2p-1.trace-g$g"
    if cmp -s "$a" "$b"; then
        echo "p2p partie $g : traces identiques ($(wc -l < "$a") lignes)"
    else
        echo "p2p partie $g : traces DIFFÉRENTES ($a, $b)"
        fail=1
    fi
done
if cmp -s "$OUT/p2p-0.trace-g1" "$OUT/p2p-0.trace-g2"; then
    echo "p2p : partie 2 identique à la partie 1 (graine non dérivée ?)"
    fail=1
else
    echo "p2p : partie 2 différente de la partie 1 (graine dérivée du numéro de partie)"
fi
grep -h "restart en ligne\|salle" "$OUT"/p2p-*.log | sed 's/^/  /' | head -8 || true

# Local (synctest) : même graine au restart, donc parties 1 et 2 identiques.
ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/local.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" \
ALACOD_RESTART_AT_FRAME="$RESTART_AT" APP_VERSION=x \
cargo run -q -p zombies --profile headless --no-default-features -- \
    --number-player 1 --players localhost --cid local --name local \
    > "$OUT/local.log" 2>&1 || { echo "local : échec (voir $OUT/local.log)"; exit 1; }
head -n "$((EXIT_AT - 1))" "$OUT/local.trace-g1" > "$OUT/local.g1-head"
if cmp -s "$OUT/local.g1-head" "$OUT/local.trace-g2"; then
    echo "local : partie 2 identique à la partie 1 ($(wc -l < "$OUT/local.trace-g2") lignes)"
else
    echo "local : partie 2 DIFFÉRENTE de la partie 1"
    fail=1
fi

exit "$fail"
