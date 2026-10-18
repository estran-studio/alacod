#!/usr/bin/env bash
# Attente bornée d'un pair vu par le signaling mais jamais connecté (D109, `jjrs::p2p::wait_for_players`) : deux clients
# se trouvent au lobby ; le client 1 est SUSPENDU (SIGSTOP) dès qu'il a vu son pair, donc avant de finir la négociation
# WebRTC (il ne répond plus aux échanges ICE/DTLS, son socket reste ouvert côté signaling : c'est la situation
# de D109, un pair annoncé qui ne se connecte jamais). Au bout de `ALACOD_CONNECT_TIMEOUT_SECS`, le client 0 journalise
# `ggrs{f=0 lobby_connect_timeout attempt=1/2 action=retry …}` et rouvre un socket (ATTEMPTS=2, le défaut du jeu) ou, avec
# ATTEMPTS=1 (`ALACOD_CONNECT_ATTEMPTS=1`), `… attempt=1/1 action=lobby …` et revient au lobby (`LobbyOnline` ré-entré).
# Le client 1 est alors repris : les deux se retrouvent, jouent une partie complète jusqu'à `EXIT_AT` et leurs traces
# doivent être identiques. (Après une nouvelle tentative, le client 0 ne revoit pas un pair suspendu : le signaling
# n'annonce un pair déjà là qu'à travers l'offre que celui-ci envoie ; le second délai n'est donc exercé que par ATTEMPTS=1.)
#
# Usage : [ATTEMPTS=1] scripts/p2p-connect-timeout.sh [jeu]       (zombies par défaut ; signaling démarré et arrêté ici)
# Variables : ATTEMPTS (2), EXIT_AT (300), CONNECT_TIMEOUT_S (4), TIMEOUT (300 s), OUT (/tmp/p2p-connect-timeout),
#   MATCHBOX_PORT, MATCHBOX_URL (voir `p2p-lib.sh`). Natif, sans `tc` : tourne sur Mac comme sur Linux. Bash, pas zsh.
set -euo pipefail

GAME=${1:-zombies}
ATTEMPTS=${ATTEMPTS:-2}
EXIT_AT=${EXIT_AT:-300}
CONNECT_TIMEOUT_S=${CONNECT_TIMEOUT_S:-4}
TIMEOUT=${TIMEOUT:-300}
OUT=${OUT:-/tmp/p2p-connect-timeout}
LOBBY="connecttimeout-$$"

# shellcheck source=scripts/p2p-lib.sh
. "$(dirname "${BASH_SOURCE[0]}")/p2p-lib.sh"
p2p_init "$GAME"
mkdir -p "$OUT"
rm -f "$OUT"/*.log "$OUT"/*.trace

pids=()
cleanup() {
    [ -z "${pids[1]:-}" ] || kill -CONT "${pids[1]}" 2>/dev/null || true
    [ "${#pids[@]}" -eq 0 ] || kill "${pids[@]}" 2>/dev/null || true
    p2p_signaling_down
}
trap cleanup EXIT
p2p_signaling_up
p2p_calm

PLAYERS=$(p2p_players 2)
for i in 0 1; do
    # shellcheck disable=SC2086
    env ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" \
        ALACOD_CONNECT_TIMEOUT_SECS="$CONNECT_TIMEOUT_S" ALACOD_CONNECT_ATTEMPTS="$ATTEMPTS" APP_VERSION=x \
        "$BIN" --matchbox "$SIGNAL_URL" --lobby "$LOBBY" --number-player 2 \
        --players $PLAYERS --cid "client_$i" --name "client_$i" \
        > "$OUT/p2p-$i.log" 2>&1 &
    pids+=("$!")
done

# Suspend le client 1 dès qu'il a vu son pair par le signaling (avant « peer … connected »).
stopped=0
for ((n = 0; n < 12000; n++)); do
    if grep -q "vu(s) par le signaling" "$OUT/p2p-1.log" 2>/dev/null; then
        kill -STOP "${pids[1]}"
        stopped=1
        break
    fi
    if grep -q "peer .* connected" "$OUT/p2p-1.log" 2>/dev/null; then
        break
    fi
    sleep 0.005
done
[ "$stopped" = 1 ] || { echo "ÉCHEC : le client 1 a connecté son pair avant d'être suspendu (voir $OUT/p2p-1.log)"; exit 1; }
echo "client 1 suspendu ($(date +%T)) : annoncé par le signaling, négociation inachevée"

# Le client 0 doit abandonner (nouvelle tentative, ou retour au lobby avec ATTEMPTS=1) ; le client 1 reste suspendu jusque-là.
if [ "$ATTEMPTS" -ge 2 ]; then expect="attempt=1/$ATTEMPTS action=retry"; else expect="attempt=1/1 action=lobby"; fi
seen_timeout=0
for ((n = 0; n < 6000; n++)); do
    if grep -q "lobby_connect_timeout $expect" "$OUT/p2p-0.log" 2>/dev/null; then
        seen_timeout=1
        break
    fi
    sleep 0.05
done
[ "$seen_timeout" = 1 ] || { echo "ÉCHEC : le client 0 n'a jamais journalisé « $expect » (voir $OUT/p2p-0.log)"; exit 1; }
kill -CONT "${pids[1]}"
echo "client 1 repris ($(date +%T))"

( sleep "$TIMEOUT"; kill -CONT "${pids[1]}" 2>/dev/null; kill "${pids[@]}" 2>/dev/null ) >/dev/null 2>&1 &
watchdog=$!
status=0
for pid in "${pids[@]}"; do
    wait "$pid" || status=1
done
kill "$watchdog" 2>/dev/null || true
pids=()

fail=0
[ "$status" -eq 0 ] || { echo "ÉCHEC : un client est sorti en erreur ou a été tué au délai (voir $OUT/p2p-*.log)"; fail=1; }
n=$(grep -c "lobby_connect_timeout $expect" "$OUT/p2p-0.log" || true)
echo "client 0 : « $expect » : $n"
{ grep -h "lobby_connect_timeout" "$OUT"/p2p-*.log || true; } | sed 's/^/  /'
[ "$n" -ge 1 ] || { echo "ÉCHEC : le client 0 n'a pas fait « $expect »"; fail=1; }
connected=$(grep -c "All 2 players are connected and ready" "$OUT/p2p-0.log" || true)
echo "client 0 : parties démarrées après l'abandon : $connected"
[ "$connected" -ge 1 ] || { echo "ÉCHEC : le client 0 n'a jamais démarré de partie"; fail=1; }
echo "DesyncDetected : $( (grep -h -c "DesyncDetected" "$OUT"/p2p-*.log || true) | awk '{s+=$1} END{print s+0}')"
if cmp -s "$OUT/p2p-0.trace" "$OUT/p2p-1.trace"; then
    echo "traces des deux clients identiques ($(wc -l < "$OUT/p2p-0.trace") lignes, partie jouée après l'abandon)"
else
    echo "ÉCHEC : traces DIFFÉRENTES ou absentes ($OUT/p2p-0.trace, $OUT/p2p-1.trace)"
    fail=1
fi
[ "$fail" = 0 ] && echo "p2p-connect-timeout : OK" || echo "p2p-connect-timeout : ÉCHEC"
exit "$fail"
