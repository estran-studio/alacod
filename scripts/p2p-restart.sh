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
#   locale ; forcé à 0 pour lobby et hub), RESTART_KIND (restart : `ALACOD_RESTART_KIND`, la requête
#   posée à RESTART_AT : `restart` = « Rejouer », `lobby` = bouton « Lobby » (les pairs reviennent
#   dans la salle d'origine, graine d'origine : la partie 2 doit être IDENTIQUE à la 1), `hub` =
#   « Retour au hub » des jeux à hub : hub → run → hub → run, graine dérivée à chaque passage),
#   EXPECT_VICTORY=1 (échec si un client n'a pas journalisé `run summary … outcome=Victory` : la requête
#   est posée APRÈS la fin de run), TIMEOUT (900 s : au-delà, les clients sont tués et le script échoue),
#   RESTART_DELAYS_MS (vide : aucun délai ; « 0 250 » : `ALACOD_RESTART_DELAY_MS` du client i, la requête arrive plus tard dans sa partie ;
#   `p2p-lobby.sh` s'en sert pour étaler les retours au lobby, D86), OUT (/tmp/p2p-restart), MATCHBOX_PORT,
#   MATCHBOX_URL, NETEM (voir `p2p-lib.sh`).
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
RESTART_DELAYS_MS=${RESTART_DELAYS_MS:-}
read -r -a delays <<< "$RESTART_DELAYS_MS"
TIMEOUT=${TIMEOUT:-900}
RESTART_KIND=${RESTART_KIND:-restart}
case "$RESTART_KIND" in
    restart) ;;
    lobby | hub) LOCAL=0; export ALACOD_RESTART_KIND=$RESTART_KIND ;;
    *) echo "p2p-restart : RESTART_KIND inconnu : $RESTART_KIND (restart, lobby, hub)"; exit 2 ;;
esac
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
p2p_netns_reexec "$SELF" ${1:+"$1"}

pids=()
suspender=""
cleanup() {
    # `set -e` : un `kill` sur un processus déjà parti ne doit pas changer le code de sortie du script.
    [ -z "$suspender" ] || kill "$suspender" 2>/dev/null || true
    [ -z "${pids[1]:-}" ] || kill -CONT "${pids[1]}" 2>/dev/null || true
    p2p_signaling_down
}
trap cleanup EXIT
p2p_signaling_up
p2p_calm

input_env=()
[ -z "$INPUTS" ] || input_env=("ALACOD_INPUT=script:$INPUTS" "ALACOD_INPUT_DELAY=0")
PLAYERS=$(p2p_players "$CLIENTS")
for ((i = 0; i < CLIENTS; i++)); do
    # shellcheck disable=SC2086
    env ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" \
        ALACOD_RESTART_AT_FRAME="$RESTART_AT" ALACOD_RESTART_DELAY_MS="${delays[i]:-0}" \
        APP_VERSION=x ${input_env[@]+"${input_env[@]}"} \
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
# Garde-fou : tue les clients au-delà de TIMEOUT (une partie 2 qui ne démarre jamais, D86).
( sleep "$TIMEOUT"; kill -CONT "${pids[1]:-0}" 2>/dev/null; kill "${pids[@]}" 2>/dev/null ) >/dev/null 2>&1 &
watchdog=$!
status=0
for pid in "${pids[@]}"; do
    wait "$pid" || status=1
done
kill "$watchdog" 2>/dev/null || true
[ -z "$suspender" ] || kill "$suspender" 2>/dev/null || true
[ "$status" -eq 0 ] || { echo "p2p : un client a échoué (voir $OUT/p2p-*.log)"; exit 1; }

fail=0
echo "p2p-restart : jeu=$GAME clients=$CLIENTS requête=$RESTART_KIND à la frame $RESTART_AT, fin de la partie 2 à $EXIT_AT, stress=$STRESS netem='${NETEM:-}' args='$GAME_ARGS'"
desync=$( (grep -c "DesyncDetected" "$OUT"/p2p-*.log || true) | awk -F: '{s+=$2} END{print s+0}')
echo "DesyncDetected : $desync"
[ "$desync" = 0 ] || { echo "ÉCHEC : desync signalé par GGRS"; fail=1; }
for ((i = 0; i < CLIENTS; i++)); do
    echo "client $i : frames resimulées (compteur du jeu, deux parties) : $(p2p_resimulated "$OUT/p2p-$i.log")"
done
if [ -n "${EXPECT_VICTORY:-}" ]; then
    for ((i = 0; i < CLIENTS; i++)); do
        v=$(grep -o 'f[0-9]* run summary:.*outcome=[A-Za-z]*' "$OUT/p2p-$i.log" | head -1 || true)
        echo "client $i : ${v:-aucun résumé de run}"
        case "$v" in *outcome=Victory) ;; *) echo "ÉCHEC : client $i : pas de victoire avant la relance"; fail=1 ;; esac
    done
fi
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
if [ "$RESTART_KIND" = lobby ]; then
    # Retour au lobby d'origine : graine d'origine, la partie 2 rejoue la 1 (comme le restart local).
    head -n "$((EXIT_AT - 1))" "$OUT/p2p-0.trace-g1" > "$OUT/p2p-0.g1-head"
    if cmp -s "$OUT/p2p-0.g1-head" "$OUT/p2p-0.trace-g2"; then
        echo "p2p : partie 2 identique à la partie 1 ($(wc -l < "$OUT/p2p-0.trace-g2") lignes : lobby d'origine, graine d'origine)"
    else
        echo "p2p : partie 2 DIFFÉRENTE de la partie 1 après retour au lobby (graine d'origine attendue)"
        fail=1
    fi
elif cmp -s "$OUT/p2p-0.trace-g1" "$OUT/p2p-0.trace-g2"; then
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
