#!/usr/bin/env bash
# Abandon d'une session p2p qui ne se synchronise pas (D86 (b), `jjrs::p2p::abandon_unsynchronized_session`) :
# deux clients se trouvent au lobby ; le client 1 est SUSPENDU (SIGSTOP) dès qu'il a vu son pair, donc avant de
# construire sa session. Le client 0 passe en partie et reste en `Synchronizing` : au bout de
# `ALACOD_SYNC_TIMEOUT_SECS` il doit abandonner (journal `ggrs{f=… session_sync_timeout …}`) et revenir au lobby.
# Le client 1, relancé après STALL_S secondes, construit sa session sur l'identifiant que le client 0 vient de
# fermer, n'est jamais synchronisé, abandonne à son tour ; les deux se retrouvent (chacun ignore l'ancien
# identifiant de l'autre, (a)) et jouent une partie complète jusqu'à `EXIT_AT` : leurs traces doivent être identiques.
#
# Usage : scripts/p2p-sync-timeout.sh [jeu]       (zombies par défaut ; signaling démarré et arrêté ici)
# Variables : EXIT_AT (300), SYNC_TIMEOUT_S (4), STALL_S (8, > SYNC_TIMEOUT_S), TIMEOUT (300 s), OUT (/tmp/p2p-sync-timeout),
#   MATCHBOX_PORT, MATCHBOX_URL (voir `p2p-lib.sh`). Bash, pas zsh.
set -euo pipefail

GAME=${1:-zombies}
EXIT_AT=${EXIT_AT:-300}
SYNC_TIMEOUT_S=${SYNC_TIMEOUT_S:-4}
STALL_S=${STALL_S:-8}
TIMEOUT=${TIMEOUT:-300}
OUT=${OUT:-/tmp/p2p-sync-timeout}
LOBBY="synctimeout-$$"

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
        ALACOD_SYNC_TIMEOUT_SECS="$SYNC_TIMEOUT_S" APP_VERSION=x \
        "$BIN" --matchbox "$SIGNAL_URL" --lobby "$LOBBY" --number-player 2 \
        --players $PLAYERS --cid "client_$i" --name "client_$i" \
        > "$OUT/p2p-$i.log" 2>&1 &
    pids+=("$!")
done

# Suspend le client 1 dès qu'il a vu son pair (avant `GameStarting`, qui construit la session).
stopped=0
for ((n = 0; n < 6000; n++)); do
    if grep -q "transitioning to GameLoading" "$OUT/p2p-1.log" 2>/dev/null; then
        kill -STOP "${pids[1]}"
        stopped=1
        break
    fi
    sleep 0.005
done
[ "$stopped" = 1 ] || { echo "ÉCHEC : le client 1 n'a jamais vu son pair"; exit 1; }
echo "client 1 suspendu ($(date +%T.%3N)) pour ${STALL_S} s"
sleep "$STALL_S"
kill -CONT "${pids[1]}"
echo "client 1 repris ($(date +%T.%3N))"

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
# Le client 0 attend un pair suspendu : il abandonne au délai (`session_sync_timeout`). Le client 1, repris après, trouve
# le client 0 déjà parti de la salle : depuis D109 il abandonne aussitôt (`session_peer_left`), sans attendre son propre délai ;
# l'un ou l'autre journal est un abandon de session non synchronisée.
for i in 0 1; do
    n=$(grep -c "session_sync_timeout\|session_peer_left" "$OUT/p2p-$i.log" || true)
    echo "client $i : abandons de session non synchronisée : $n"
    { grep -h "session_sync_timeout\|session_peer_left" "$OUT/p2p-$i.log" || true; } | sed 's/^/  /'
    [ "$n" -ge 1 ] || { echo "ÉCHEC : le client $i n'a jamais abandonné de session"; fail=1; }
done
grep -q "session_sync_timeout" "$OUT/p2p-0.log" || { echo "ÉCHEC : le client 0 n'a pas abandonné au délai de synchronisation"; fail=1; }
echo "pairs obsolètes ignorés : $( (grep -h -c "d'une session précédente ignoré" "$OUT"/p2p-*.log || true) | awk '{s+=$1} END{print s+0}')"
echo "DesyncDetected : $( (grep -h -c "DesyncDetected" "$OUT"/p2p-*.log || true) | awk '{s+=$1} END{print s+0}')"
if cmp -s "$OUT/p2p-0.trace" "$OUT/p2p-1.trace"; then
    echo "traces des deux clients identiques ($(wc -l < "$OUT/p2p-0.trace") lignes, partie jouée après l'abandon)"
else
    echo "ÉCHEC : traces DIFFÉRENTES ou absentes ($OUT/p2p-0.trace, $OUT/p2p-1.trace)"
    fail=1
fi
[ "$fail" = 0 ] && echo "p2p-sync-timeout : OK" || echo "p2p-sync-timeout : ÉCHEC"
exit "$fail"
