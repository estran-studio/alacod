#!/usr/bin/env bash
# Admission au lobby en ligne (D116, `jjrs::p2p::admit_players`) : TROIS clients dans une salle à DEUX places, lancés
# ensemble. Les deux premiers pairs par identifiant (même ensemble chez tous : même choix) sont admis et jouent une
# partie complète jusqu'à `EXIT_AT` ; le troisième est ignoré (`ggrs{f=0 lobby_admission … local=ignored}`), prévenu, ne
# bloque personne, puis revient au lobby au délai (`ggrs{f=0 lobby_room_full … action=lobby}`). Avant D116, les trois
# attendaient sans fin (« Too many peers »).
#
# Vert si : exactement un client ignoré et revenu au lobby ; les deux autres admis, partie démarrée, sortis en code 0,
# sans `DesyncDetected`, traces identiques. L'admission porte sur les pairs connectés au moment de la décision : un
# essai où un client voit un autre ensemble que les autres (course) n'a pas de valeur et se rejoue dans une salle
# neuve (`RACE_RETRIES` fois au plus ; le résumé donne le nombre d'essais perdus).
#
# Usage : [TRIALS=n] [GHOST=1] scripts/p2p-admission.sh [jeu]       (zombies par défaut ; signaling démarré et arrêté ici)
# Variables : TRIALS (1 : essais verts demandés), EXIT_AT (1500), ROOM_FULL_TIMEOUT_S (3 : délai de
#   l'ignoré, `ALACOD_CONNECT_TIMEOUT_SECS`), TIMEOUT (300 s), OUT (/tmp/p2p-admission), RACE_RETRIES (6),
#   STAGGER_MS (0 : écart de lancement entre clients), MATCHBOX_PORT, MATCHBOX_URL (voir `p2p-lib.sh`). Natif, sans `tc`.
#   Bash, pas zsh.
set -uo pipefail

GAME=${1:-zombies}
TRIALS=${TRIALS:-1}
EXIT_AT=${EXIT_AT:-1500}
ROOM_FULL_TIMEOUT_S=${ROOM_FULL_TIMEOUT_S:-3}
TIMEOUT=${TIMEOUT:-300}
OUT=${OUT:-/tmp/p2p-admission}
RACE_RETRIES=${RACE_RETRIES:-6}
STAGGER_MS=${STAGGER_MS:-0}

# shellcheck source=scripts/p2p-lib.sh
. "$(dirname "${BASH_SOURCE[0]}")/p2p-lib.sh"
p2p_init "$GAME"
mkdir -p "$OUT"

pids=()
cleanup() {
    [ "${#pids[@]}" -eq 0 ] || kill "${pids[@]}" 2>/dev/null || true
    p2p_signaling_down
}
trap cleanup EXIT
p2p_signaling_up
p2p_calm

# GHOST=1 : deux clients et un pair fantôme à moitié connecté (`p2p-ghost-peer.mjs` : annoncé par le signaling, il ne finit
# jamais sa négociation) dans la salle à deux places. Les deux clients ne doivent attendre que `ADMISSION_SETTLE_SECS` (3 s,
# le pair vu mais pas connecté retarde l'admission), puis jouer une partie complète aux traces identiques : avant ce
# délai borné, un fantôme à moitié connecté aurait bloqué la paire jusqu'au délai de D109.
if [ "${GHOST:-0}" = 1 ]; then
    command -v node >/dev/null || { echo "GHOST=1 : node requis (scripts/p2p-ghost-peer.mjs)"; exit 2; }
    dir="$OUT/fantome"
    mkdir -p "$dir"
    rm -f "$dir"/*.log "$dir"/*.trace
    LOBBY="admission-ghost-$$"
    node "$(dirname "${BASH_SOURCE[0]}")/p2p-ghost-peer.mjs" "$SIGNAL_URL/$LOBBY" > "$dir/ghost.log" 2>&1 &
    pids=("$!")
    for ((n = 0; n < 200; n++)); do grep -q "dans la salle" "$dir/ghost.log" 2>/dev/null && break; sleep 0.05; done
    grep -q "dans la salle" "$dir/ghost.log" || { echo "ÉCHEC : le pair fantôme n'a pas rejoint la salle (voir $dir/ghost.log)"; exit 1; }
    for i in 0 1; do
        # shellcheck disable=SC2086
        env ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$dir/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" APP_VERSION=x \
            "$BIN" --matchbox "$SIGNAL_URL" --lobby "$LOBBY" --number-player 2 \
            --players $(p2p_players 2) --cid "client_$i" --name "client_$i" > "$dir/p2p-$i.log" 2>&1 &
        pids+=("$!")
    done
    fail=0
    for i in 1 2; do
        ( sleep "$TIMEOUT"; kill "${pids[$i]}" 2>/dev/null ) >/dev/null 2>&1 &
        wd=$!
        wait "${pids[$i]}" || { echo "ÉCHEC : le client $((i - 1)) est sorti en erreur (voir $dir/p2p-$((i - 1)).log)"; fail=1; }
        kill "$wd" 2>/dev/null || true
    done
    for i in 0 1; do
        grep -q "vu(s) par le signaling" "$dir/p2p-$i.log" || echo "  (le client $i n'a pas vu le fantôme avant de jouer)"
        grep -q "All 2 players are connected and ready" "$dir/p2p-$i.log" || { echo "ÉCHEC : le client $i n'a pas démarré"; fail=1; }
        grep -q "lobby_connect_timeout" "$dir/p2p-$i.log" && { echo "ÉCHEC : le client $i a atteint le délai de D109 (le fantôme l'a bloqué)"; fail=1; }
    done
    if cmp -s "$dir/p2p-0.trace" "$dir/p2p-1.trace" && [ -s "$dir/p2p-0.trace" ]; then
        echo "traces des deux clients identiques ($(wc -l < "$dir/p2p-0.trace") lignes) avec un fantôme dans la salle"
    else
        echo "ÉCHEC : traces DIFFÉRENTES ou absentes"; fail=1
    fi
    [ "$fail" -eq 0 ] && echo "p2p-admission GHOST=1 : OK" || echo "p2p-admission GHOST=1 : ÉCHEC"
    exit "$fail"
fi

green=0
lost=0
attempt=0
while [ "$green" -lt "$TRIALS" ]; do
    attempt=$((attempt + 1))
    dir="$OUT/essai-$attempt"
    mkdir -p "$dir"
    rm -f "$dir"/*.log "$dir"/*.trace
    LOBBY="admission-$$-$attempt"
    PLAYERS=$(p2p_players 2)
    pids=()
    for i in 0 1 2; do
        # shellcheck disable=SC2086
        env ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$dir/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$EXIT_AT" \
            ALACOD_CONNECT_TIMEOUT_SECS="$ROOM_FULL_TIMEOUT_S" APP_VERSION=x \
            "$BIN" --matchbox "$SIGNAL_URL" --lobby "$LOBBY" --number-player 2 \
            --players $PLAYERS --cid "client_$i" --name "client_$i" \
            > "$dir/p2p-$i.log" 2>&1 &
        pids+=("$!")
        [ "$STAGGER_MS" -eq 0 ] || sleep "$(awk "BEGIN{print $STAGGER_MS/1000}")"
    done

    # Attend le client ignoré : « lobby_room_full … action=lobby » (ou la fin de l'essai au délai).
    ignored=-1
    deadline=$((SECONDS + TIMEOUT))
    while [ "$SECONDS" -lt "$deadline" ]; do
        for i in 0 1 2; do
            if grep -q "lobby_room_full .* action=lobby" "$dir/p2p-$i.log" 2>/dev/null; then ignored=$i; break; fi
        done
        [ "$ignored" -ge 0 ] && break
        # Une session perdue (synchronisation, pair parti) : course perdue.
        if grep -qs "session_sync_timeout\|session_peer_left" "$dir"/p2p-*.log; then break; fi
        sleep 0.2
    done
    # Les deux admis finissent seuls (EXIT_AT) ; le client ignoré, lui, boucle sur le lobby : on l'arrête (notre PID).
    status=()
    if [ "$ignored" -ge 0 ]; then
        for i in 0 1 2; do
            [ "$i" -ne "$ignored" ] || continue
            ( sleep "$TIMEOUT"; kill "${pids[$i]}" 2>/dev/null ) >/dev/null 2>&1 &
            wd=$!
            wait "${pids[$i]}"; status[$i]=$?
            kill "$wd" 2>/dev/null || true
        done
        kill "${pids[$ignored]}" 2>/dev/null || true
        wait "${pids[$ignored]}" 2>/dev/null || true
    else
        kill "${pids[@]}" 2>/dev/null || true
        wait "${pids[@]}" 2>/dev/null || true
    fi
    pids=()

    # Régression (pas une course) : le client ignoré, revenu au lobby, ne doit ni démarrer de partie ni construire de
    # session (il serait admis à des pairs qui jouent déjà : `session_sync_timeout` ou `session_peer_left` plus tard).
    if [ "$ignored" -ge 0 ] && sed -n '/lobby_room_full/,$p' "$dir/p2p-$ignored.log" | grep -q "All 2 players are connected\|session_sync_timeout\|session_peer_left"; then
        echo "ÉCHEC : le client ignoré $ignored a construit une session après son retour au lobby (voir $dir/p2p-$ignored.log)"
        exit 1
    fi
    # Course perdue : pas de client ignoré revenu au lobby, ou une session des admis perdue.
    nignored=$(grep -l "lobby_admission .* local=ignored" "$dir"/p2p-*.log 2>/dev/null | wc -l | tr -d ' ')
    nadmitted_with_excess=$(grep -l "lobby_admission .* local=admitted" "$dir"/p2p-*.log 2>/dev/null | wc -l | tr -d ' ')
    session_lost=0
    for i in 0 1 2; do
        [ "$i" -ne "$ignored" ] || continue
        grep -qs "session_sync_timeout\|session_peer_left" "$dir/p2p-$i.log" && session_lost=1
    done
    if [ "$ignored" -lt 0 ] || [ "$nignored" != 1 ] || [ "$session_lost" = 1 ]; then
        lost=$((lost + 1))
        echo "essai $attempt : course perdue (client ignoré revenu au lobby : $ignored ; clients ignorés : $nignored ; session perdue : $session_lost ; admis ayant vu le pair en trop : $nadmitted_with_excess), nouvel essai"
        [ "$lost" -le "$RACE_RETRIES" ] || { echo "ÉCHEC : trop d'essais perdus ($lost) ; journaux dans $OUT"; exit 1; }
        continue
    fi

    admitted=()
    for i in 0 1 2; do [ "$i" -eq "$ignored" ] || admitted+=("$i"); done
    a=${admitted[0]}
    b=${admitted[1]}
    fail=0
    echo "essai $attempt : client $ignored ignoré (retour au lobby), clients $a et $b admis"
    for i in "$a" "$b"; do
        [ "${status[$i]:-1}" -eq 0 ] || { echo "ÉCHEC : le client $i admis est sorti en code ${status[$i]:-?} (voir $dir/p2p-$i.log)"; fail=1; }
        grep -q "All 2 players are connected and ready" "$dir/p2p-$i.log" ||
            { echo "ÉCHEC : le client $i admis n'a pas démarré de partie"; fail=1; }
        grep -q "lobby_admission .* local=admitted" "$dir/p2p-$i.log" ||
            echo "  (le client $i n'a jamais vu le pair en trop : il a décidé avant son arrivée)"
    done
    grep -q "All 2 players are connected and ready" "$dir/p2p-$ignored.log" &&
        { echo "ÉCHEC : le client ignoré a démarré une partie"; fail=1; }
    echo "journal du client ignoré :"
    { grep -h "lobby_admission\|lobby_room_full" "$dir/p2p-$ignored.log" || true; } | sed 's/^/  /'
    echo "journal d'un admis :"
    { grep -h "lobby_admission" "$dir/p2p-$a.log" || true; } | sed 's/^/  /'
    desync=$( (grep -h -c "DesyncDetected" "$dir"/p2p-*.log || true) | awk '{s+=$1} END{print s+0}')
    echo "DesyncDetected : $desync"
    [ "$desync" -eq 0 ] || fail=1
    if cmp -s "$dir/p2p-$a.trace" "$dir/p2p-$b.trace" && [ -s "$dir/p2p-$a.trace" ]; then
        echo "traces des deux admis identiques ($(wc -l < "$dir/p2p-$a.trace") lignes)"
    else
        echo "ÉCHEC : traces des admis DIFFÉRENTES ou absentes ($dir/p2p-$a.trace, $dir/p2p-$b.trace)"
        fail=1
    fi
    [ "$fail" -eq 0 ] || { echo "p2p-admission : ÉCHEC"; exit 1; }
    green=$((green + 1))
done
echo "p2p-admission : OK ($green essai(s) vert(s), $lost essai(s) perdu(s) par course sur $attempt)"
