#!/usr/bin/env bash
# D109 (mesure) : fréquence des démarrages en ligne dont la connexion WebRTC ne s'établit jamais. Lance
# STARTS fois deux clients dans une salle neuve (signaling local, réseau dégradé `tc netem` si NETEM) et note,
# pour chaque démarrage, le temps jusqu'à « All 2 players are connected » chez les deux (horloge murale,
# résolution 0,1 s). Un démarrage qui n'a pas connecté ses deux clients au bout de WAIT_S est **bloqué** ; un
# démarrage connecté après un `ggrs{… lobby_connect_timeout …}` est **récupéré** (binaire d'après D109).
#
# Avec PLAY_FRAMES=<n>, chaque démarrage connecté est ensuite **joué** jusqu'à la frame n (`ALACOD_EXIT_AT_FRAME`, traces
# d'état comparées) dans la limite de PLAY_WAIT_S : colonne `played` = `yes` (deux sorties 0, traces identiques) ou `no`.
#
# Usage : [NETEM="delay 100ms 10ms loss 3%"] scripts/p2p-connect-starts.sh [jeu]     (gungeon par défaut)
# Variables : STARTS (30), WAIT_S (75), PLAY_FRAMES (vide : pas de partie), PLAY_WAIT_S (240), OUT (/tmp/p2p-connect-starts), BIN_OVERRIDE (binaire à mesurer à la
#   place de celui du dépôt : l'état d'avant D109), ALACOD_CONNECT_TIMEOUT_SECS (transmise aux clients),
#   MATCHBOX_PORT, P2P_NO_BUILD (voir `p2p-lib.sh`). Écrit `$OUT/summary.tsv` (une ligne par démarrage) et garde
#   les journaux des démarrages bloqués ou récupérés. `NETEM` demande Linux (espace de noms réseau, `tc`) :
#   depuis un Mac, `scripts/p2p-linux-box.sh scripts/p2p-connect-starts.sh gungeon`. Bash ≥ 5.
set -uo pipefail

GAME=${1:-gungeon}
STARTS=${STARTS:-30}
WAIT_S=${WAIT_S:-75}
PLAY_FRAMES=${PLAY_FRAMES:-}
PLAY_WAIT_S=${PLAY_WAIT_S:-240}
OUT=${OUT:-/tmp/p2p-connect-starts}

# shellcheck source=scripts/p2p-lib.sh
. "$(dirname "${BASH_SOURCE[0]}")/p2p-lib.sh"
p2p_init "$GAME"
p2p_netns_reexec "$0" "$@"
BIN=${BIN_OVERRIDE:-$BIN}
mkdir -p "$OUT"
rm -f "$OUT"/*.log "$OUT"/summary.tsv

pids=()
cleanup() {
    [ "${#pids[@]}" -eq 0 ] || kill "${pids[@]}" 2>/dev/null || true
    p2p_signaling_down
}
trap cleanup EXIT
p2p_signaling_up
p2p_calm
echo "réseau : ${NETEM:-direct} ; binaire : $BIN ; $STARTS démarrages, délai $WAIT_S s"

PLAYERS=$(p2p_players 2)
printf 'start\tverdict\tconnect_s_0\tconnect_s_1\ttimeouts\tplayed\n' > "$OUT/summary.tsv"
for ((n = 1; n <= STARTS; n++)); do
    lobby="starts-$$-$n"
    t0=$EPOCHREALTIME
    pids=()
    for i in 0 1; do
        # shellcheck disable=SC2086
        env ALACOD_HEADLESS=1 ALACOD_INPUT=neutral APP_VERSION=x \
            ${PLAY_FRAMES:+ALACOD_STATE_TRACE="$OUT/start-$n-$i.trace" ALACOD_EXIT_AT_FRAME="$PLAY_FRAMES"} \
            "$BIN" --matchbox "$SIGNAL_URL" --lobby "$lobby" --number-player 2 \
            --players $PLAYERS --cid "client_$i" --name "client_$i" \
            > "$OUT/start-$n-$i.log" 2>&1 &
        pids+=("$!")
    done
    t_conn=("" "")
    for ((tick = 0; tick < WAIT_S * 10; tick++)); do
        for i in 0 1; do
            if [ -z "${t_conn[$i]}" ] && grep -q "transitioning to GameLoading" "$OUT/start-$n-$i.log" 2>/dev/null; then
                t_conn[$i]=$(awk -v a="$EPOCHREALTIME" -v b="$t0" 'BEGIN{printf "%.1f", a - b}')
            fi
        done
        [ -n "${t_conn[0]}" ] && [ -n "${t_conn[1]}" ] && break
        sleep 0.1
    done
    timeouts=$(cat "$OUT"/start-$n-*.log | grep -c "lobby_connect_timeout" || true)
    if [ -z "${t_conn[0]}" ] || [ -z "${t_conn[1]}" ]; then
        verdict=stuck
    elif [ "$timeouts" -gt 0 ]; then
        verdict=recovered
    else
        verdict=ok
    fi
    played=-
    if [ -n "$PLAY_FRAMES" ] && [ "$verdict" != stuck ]; then
        # Jouer jusqu'à la frame demandée : les deux clients sortent d'eux-mêmes (code 0, traces identiques).
        for ((tick = 0; tick < PLAY_WAIT_S * 10; tick++)); do
            kill -0 "${pids[0]}" 2>/dev/null || kill -0 "${pids[1]}" 2>/dev/null || break
            sleep 0.1
        done
        played=yes
        for i in 0 1; do
            if kill -0 "${pids[$i]}" 2>/dev/null; then kill "${pids[$i]}" 2>/dev/null; played=no; fi
            wait "${pids[$i]}" 2>/dev/null || played=no
        done
        [ "$played" = no ] || cmp -s "$OUT/start-$n-0.trace" "$OUT/start-$n-1.trace" || played=no
    fi
    kill "${pids[@]}" 2>/dev/null
    wait "${pids[@]}" 2>/dev/null
    pids=()
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$n" "$verdict" "${t_conn[0]:--}" "${t_conn[1]:--}" "$timeouts" "$played" | tee -a "$OUT/summary.tsv"
    if [ "$verdict" = ok ] && [ "$played" != no ]; then rm -f "$OUT/start-$n-0.log" "$OUT/start-$n-1.log"; fi
    rm -f "$OUT"/start-"$n"-*.trace
    sleep 1
done

awk -F'\t' 'NR > 1 { v[$2]++; total++; p[$6]++ } END { printf "démarrages %d : ok %d, récupérés (après nouvelle tentative) %d, bloqués %d", total, v["ok"], v["recovered"], v["stuck"]; if (p["yes"] + p["no"] > 0) printf " ; parties finies %d, perdues %d", p["yes"], p["no"]; printf "\n" }' "$OUT/summary.tsv"
awk -F'\t' 'NR > 1 && $2 != "stuck" { print ($3 > $4 ? $3 : $4) }' "$OUT/summary.tsv" | sort -n | awk '
    { t[NR] = $1 }
    END { if (NR > 0) printf "connexion (max des deux clients, démarrages connectés) : médiane %.1f s, p90 %.1f s, max %.1f s\n", t[int((NR + 1) / 2)], t[int(0.9 * NR + 0.5) < 1 ? 1 : int(0.9 * NR + 0.5)], t[NR] }'
