#!/usr/bin/env bash
# Déconnexion d'un pair en cours de partie (m2-p2p-run-complete, point 3 : « déconnexion d'un pair
# en cours d'étage »). N clients p2p headless rejouent un script d'inputs ; au bout de
# KILL_AFTER_S secondes, le client KILL_CLIENT est tué (SIGKILL : processus mort, pas de
# fermeture propre du socket). Le script décrit ce que font les survivants et ne juge que les
# invariants : pas de panique, pas de `DesyncDetected`, et traces des survivants identiques
# entre elles (le comportement de GGRS après le départ est constaté, pas imposé).
#
# Usage : scripts/p2p-disconnect.sh <jeu> <script-inputs.ron> [frames]
# Variables : CLIENTS (2), KILL_CLIENT (1), KILL_AFTER_S (40), GAME_ARGS, INPUT_DELAY (0),
#   OUT (/tmp/p2p-disconnect), TIMEOUT (300 s après la mort du pair : un survivant encore là est
#   tué et déclaré « bloqué »), MATCHBOX_PORT, MATCHBOX_URL, NETEM, KEEP_SIGNALING (voir
#   `scripts/p2p-lib.sh`). Les `ALACOD_*` de l'environnement sont transmis aux clients.
# Sortie 0 si les invariants tiennent (même quand les survivants ne terminent pas : c'est dit).
set -uo pipefail

[ $# -ge 2 ] || { sed -n '/^# Usage/,/^# Sortie/p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
GAME=$1
SCRIPT=$2
FRAMES=${3:-4500}
CLIENTS=${CLIENTS:-2}
KILL_CLIENT=${KILL_CLIENT:-1}
KILL_AFTER_S=${KILL_AFTER_S:-40}
GAME_ARGS=${GAME_ARGS:-}
INPUT_DELAY=${INPUT_DELAY:-0}
OUT=${OUT:-/tmp/p2p-disconnect}
TIMEOUT=${TIMEOUT:-300}
LOBBY="disc-$$"
[ -f "$SCRIPT" ] || { echo "p2p-disconnect : script introuvable : $SCRIPT"; exit 2; }
SCRIPT=$(cd "$(dirname "$SCRIPT")" && pwd)/$(basename "$SCRIPT")
SELF=$(cd "$(dirname "$0")" && pwd)/$(basename "$0")

# shellcheck source=scripts/p2p-lib.sh
. "$(dirname "${BASH_SOURCE[0]}")/p2p-lib.sh"
p2p_init "$GAME"
mkdir -p "$OUT"
rm -f "$OUT"/p2p-*.trace "$OUT"/p2p-*.log
export NETEM OUT
p2p_netns_reexec "$SELF" "$GAME" "$SCRIPT" ${3:+"$3"}

pids=()
trap 'p2p_signaling_down; kill "${pids[@]}" 2>/dev/null' EXIT
p2p_signaling_up

PLAYERS=$(p2p_players "$CLIENTS")
fail=0
for ((i = 0; i < CLIENTS; i++)); do
    # shellcheck disable=SC2086
    ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$FRAMES" \
    ALACOD_INPUT="script:$SCRIPT" ALACOD_INPUT_DELAY="$INPUT_DELAY" APP_VERSION=x \
    "$BIN" --matchbox "$SIGNAL_URL" --lobby "$LOBBY" --number-player "$CLIENTS" \
        --players $PLAYERS --cid "client_$i" --name "client_$i" $GAME_ARGS \
        > "$OUT/p2p-$i.log" 2>&1 &
    pids+=("$!")
done

sleep "$KILL_AFTER_S"
victim=${pids[$KILL_CLIENT]}
if kill -0 "$victim" 2>/dev/null; then
    kill -9 "$victim" 2>/dev/null
    echo "client $KILL_CLIENT tué (SIGKILL) après ${KILL_AFTER_S}s"
else
    echo "ÉCHEC : le client $KILL_CLIENT avait déjà quitté avant KILL_AFTER_S (partie trop courte ?) : aucune déconnexion n'a eu lieu"
    fail=1
fi
killed_at=$(date +%s)

( sleep "$TIMEOUT"; kill "${pids[@]}" 2>/dev/null ) >/dev/null 2>&1 &
watchdog=$!
codes=()
for ((i = 0; i < CLIENTS; i++)); do
    wait "${pids[$i]}"
    codes+=("$?")
done
kill "$watchdog" 2>/dev/null
wait "$watchdog" 2>/dev/null
waited=$(( $(date +%s) - killed_at ))

echo "p2p-disconnect : jeu=$GAME frames=$FRAMES clients=$CLIENTS victime=$KILL_CLIENT netem='${NETEM:-}' ; survivants terminés ${waited}s après la mort du pair"
echo "codes de sortie : ${codes[*]}   (victime : $KILL_CLIENT ; 137 = SIGKILL, 143 = tué par le garde-fou)"

survivors=()
for ((i = 0; i < CLIENTS; i++)); do
    [ "$i" = "$KILL_CLIENT" ] && continue
    survivors+=("$i")
    echo "--- client $i : code ${codes[$i]}"
    grep -h "disconnected\|Disconnected\|NetworkInterrupted\|GAME DISCONNECTED\|session perdue\|trace d.état" "$OUT/p2p-$i.log" | sed 's/^/    /' | sort | uniq -c | sort -rn | head -6
    grep -q "panicked" "$OUT/p2p-$i.log" && { echo "ÉCHEC : panique dans le journal du client $i"; grep -m3 -A2 "panicked" "$OUT/p2p-$i.log" | sed 's/^/    /'; fail=1; }
    if [ -f "$OUT/p2p-$i.trace" ]; then
        echo "    trace écrite : $(wc -l < "$OUT/p2p-$i.trace" | tr -d ' ') lignes (attendu $((FRAMES - 1)) s'il a joué jusqu'au bout)"
    else
        echo "    pas de trace : le client n'a pas atteint la frame $FRAMES (arrêt, blocage ou panique)"
    fi
done
grep -q "panicked" "$OUT/p2p-$KILL_CLIENT.log" && echo "note : le journal de la victime contient une panique (avant le SIGKILL ?)"

desync=$(grep -c "DesyncDetected" "$OUT"/p2p-*.log | awk -F: '{s+=$2} END{print s+0}')
echo "DesyncDetected : $desync"
[ "$desync" = 0 ] || { echo "ÉCHEC : desync signalé par GGRS"; fail=1; }

# Les survivants qui ont écrit leur trace doivent s'accorder entre eux.
ref=""
for i in "${survivors[@]}"; do
    [ -f "$OUT/p2p-$i.trace" ] || continue
    if [ -z "$ref" ]; then ref=$i; continue; fi
    if cmp -s "$OUT/p2p-$ref.trace" "$OUT/p2p-$i.trace"; then
        echo "cmp survivants $ref / $i : traces identiques"
    else
        echo "ÉCHEC : traces des survivants $ref et $i DIFFÉRENTES"
        fail=1
    fi
done
for c in "${codes[@]}"; do
    [ "$c" = 143 ] && { echo "constat : un survivant n'a pas fini avant le garde-fou de ${TIMEOUT}s (bloqué)"; break; }
done

[ "$fail" = 0 ] && echo "p2p-disconnect : invariants OK" || echo "p2p-disconnect : ÉCHEC (journaux : $OUT/p2p-*.log)"
exit "$fail"
