#!/usr/bin/env bash
# Restart en ligne (D14, docs/conventions.md §33) : N clients p2p jouent une partie, la relancent
# (`ALACOD_RESTART_AT_FRAME`, comme « Rejouer ») et jouent la seconde ; leurs traces doivent être
# identiques partie par partie. Puis un client local (synctest) fait de même : même graine au
# restart local, donc ses deux parties doivent être identiques entre elles (preuve que la sortie
# de partie remet tout à zéro).
#
# Usage : scripts/p2p-restart.sh [jeu]      (zombies par défaut ; signaling démarré et arrêté ici)
# Variables : RESTART_AT (600), EXIT_AT (600 : frame de fin de la partie 2, comptée depuis son
#   début), CLIENTS (2), GAME_ARGS (ex. `--seed 10`), INPUTS (scénario RON rejoué par chaque
#   client, `ALACOD_INPUT=script:` ; sans lui, les inputs de la machine), STRESS (0 : 1 suspend le
#   client 1 par à-coups comme `p2p-stress.sh`, RUN_MS/SUSPEND_MS), LOCAL (1 : 0 saute la partie
#   locale), OUT (/tmp/p2p-restart), MATCHBOX_PORT, MATCHBOX_URL, NETEM (voir `p2p-lib.sh`).
#   Tout autre `ALACOD_*` de l'environnement est transmis aux clients (`ALACOD_SKIP_HUB=1`).
# Bash, pas zsh (les `wait $pid` d'une variable non éclatée échouent en zsh).
set -euo pipefail

GAME=${1:-zombies}
RESTART_AT=${RESTART_AT:-600}
EXIT_AT=${EXIT_AT:-600}
CLIENTS=${CLIENTS:-2}
GAME_ARGS=${GAME_ARGS:-}
INPUTS=${INPUTS:-}
STRESS=${STRESS:-0}
SUSPEND_MS=${SUSPEND_MS:-12}
RUN_MS=${RUN_MS:-30}
LOCAL=${LOCAL:-1}
OUT=${OUT:-/tmp/p2p-restart}
LOBBY="restart-$$"
[ -z "$INPUTS" ] || INPUTS=$(cd "$(dirname "$INPUTS")" && pwd)/$(basename "$INPUTS")
SELF=$(cd "$(dirname "$0")" && pwd)/$(basename "$0")

# shellcheck source=scripts/p2p-lib.sh
. "$(dirname "${BASH_SOURCE[0]}")/p2p-lib.sh"
p2p_init "$GAME"
mkdir -p "$OUT"
rm -f "$OUT"/*.trace-g* "$OUT"/*.log
export NETEM OUT
p2p_netns_reexec "$SELF" "$@"

pids=()
suspender=""
cleanup() {
    [ -n "$suspender" ] && kill "$suspender" 2>/dev/null
    [ -n "${pids[1]:-}" ] && kill -CONT "${pids[1]}" 2>/dev/null
    p2p_signaling_down
}
trap cleanup EXIT
p2p_signaling_up

input_env=()
[ -z "$INPUTS" ] || input_env=("ALACOD_INPUT=script:$INPUTS" "ALACOD_INPUT_DELAY=0")
PLAYERS=$(p2p_players "$CLIENTS")
for ((i = 0; i < CLIENTS; i++)); do
    # shellcheck disable=SC2086
    env ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" \
        ALACOD_RESTART_AT_FRAME="$RESTART_AT" APP_VERSION=x ${input_env[@]+"${input_env[@]}"} \
        "$BIN" --matchbox "$SIGNAL_URL" --lobby "$LOBBY" --number-player "$CLIENTS" \
        --players $PLAYERS --cid "client_$i" --name "client_$i" $GAME_ARGS \
        > "$OUT/p2p-$i.log" 2>&1 &
    pids+=("$!")
done
if [ "$STRESS" = 1 ] && [ "$CLIENTS" -ge 2 ]; then
    (
        sleep 4
        while kill -0 "${pids[0]}" 2>/dev/null && kill -0 "${pids[1]}" 2>/dev/null; do
            kill -STOP "${pids[1]}" 2>/dev/null || break
            sleep "$(awk "BEGIN{print $SUSPEND_MS/1000}")"
            kill -CONT "${pids[1]}" 2>/dev/null || break
            sleep "$(awk "BEGIN{print $RUN_MS/1000}")"
        done
    ) >/dev/null 2>&1 &
    suspender=$!
fi
status=0
for pid in "${pids[@]}"; do
    wait "$pid" || status=1
done
[ -z "$suspender" ] || kill "$suspender" 2>/dev/null || true
[ "$status" -eq 0 ] || { echo "p2p : un client a échoué (voir $OUT/p2p-*.log)"; exit 1; }

fail=0
echo "p2p-restart : jeu=$GAME clients=$CLIENTS restart à la frame $RESTART_AT, fin de la partie 2 à $EXIT_AT, stress=$STRESS netem='${NETEM:-}' args='$GAME_ARGS'"
desync=$(grep -c "DesyncDetected" "$OUT"/p2p-*.log | awk -F: '{s+=$2} END{print s+0}')
echo "DesyncDetected : $desync"
[ "$desync" = 0 ] || { echo "ÉCHEC : desync signalé par GGRS"; fail=1; }
for ((i = 0; i < CLIENTS; i++)); do
    echo "client $i : frames resimulées (compteur du jeu, deux parties) : $(p2p_resimulated "$OUT/p2p-$i.log")"
done
for g in 1 2; do
    a="$OUT/p2p-0.trace-g$g"
    for ((i = 1; i < CLIENTS; i++)); do
        b="$OUT/p2p-$i.trace-g$g"
        if cmp -s "$a" "$b"; then
            echo "p2p partie $g : traces des clients 0 et $i identiques ($(wc -l < "$a") lignes)"
        else
            echo "p2p partie $g : traces des clients 0 et $i DIFFÉRENTES ($a, $b)"
            fail=1
        fi
    done
done
if cmp -s "$OUT/p2p-0.trace-g1" "$OUT/p2p-0.trace-g2"; then
    echo "p2p : partie 2 identique à la partie 1 (graine non dérivée ?)"
    fail=1
else
    echo "p2p : partie 2 différente de la partie 1 (graine dérivée du numéro de partie)"
fi
grep -h "restart en ligne\|salle" "$OUT"/p2p-*.log | sed 's/^/  /' | head -8 || true

# Local (synctest) : même graine au restart, donc parties 1 et 2 identiques.
if [ "$LOCAL" = 1 ]; then
    # shellcheck disable=SC2086
    env ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/local.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" \
        ALACOD_RESTART_AT_FRAME="$RESTART_AT" APP_VERSION=x ${input_env[@]+"${input_env[@]}"} \
        "$BIN" --number-player 1 --players localhost --cid local --name local $GAME_ARGS \
        > "$OUT/local.log" 2>&1 || { echo "local : échec (voir $OUT/local.log)"; exit 1; }
    head -n "$((EXIT_AT - 1))" "$OUT/local.trace-g1" > "$OUT/local.g1-head"
    if cmp -s "$OUT/local.g1-head" "$OUT/local.trace-g2"; then
        echo "local : partie 2 identique à la partie 1 ($(wc -l < "$OUT/local.trace-g2") lignes)"
    else
        echo "local : partie 2 DIFFÉRENTE de la partie 1"
        fail=1
    fi
fi

[ "$fail" = 0 ] && echo "p2p-restart : OK" || echo "p2p-restart : ÉCHEC"
exit "$fail"
