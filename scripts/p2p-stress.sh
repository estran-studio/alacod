#!/usr/bin/env bash
# p2p à N clients avec rollbacks RÉELS (outillage-p2p-stress ; recette de M3-T2 industrialisée,
# généralisée à N clients, à un réseau dégradé et à une run gungeon par m2-p2p-run-complete).
#
# Sans stress, deux clients sur une boucle locale ne rollbackent presque jamais (« frames
# resimulées après rollback : 0 à 3 ») : la comparaison des traces ne prouve alors rien. Ici :
#   - `ALACOD_INPUT_DELAY=0` : une entrée distante tardive est une prédiction fausse ;
#   - `ALACOD_INPUT=script:<script>` : des entrées qui changent toutes les 1 à 3 frames (ou les
#     inputs enregistrés d'une partie de bots, `scripts/p2p-gungeon-inputs.sh`) ;
#   - le client 1 est suspendu et repris en boucle (SIGSTOP/SIGCONT) : les autres le devancent,
#     prédisent faux, puis resimulent (`STRESS=0` : pas de suspension, le réseau dégradé suffit).
# Le compteur de frames resimulées est celui du journal du jeu (`StateTraceRecorder::resimulated`,
# une frame déjà enregistrée puis enregistrée de nouveau), jamais les lignes `ggrs{f=N}` en
# double (une ligne émise une fois par exemplaire n'est pas une resimulation).
#
# Usage : scripts/p2p-stress.sh <jeu> <script-inputs.ron> [frames]
#   <jeu>      : paquet cargo du jeu (zombies, testbed, isaac, gungeon, throne)
#   <script>   : scénario RON dont les `PlayerScript` des handles 0..N-1 sont rejoués
#   [frames]   : ALACOD_EXIT_AT_FRAME (3000 par défaut)
# Variables : CLIENTS (2 : nombre de clients sur la machine), GAME_ARGS (arguments en plus pour
#   chaque client, ex. `--seed 10`), EXPECT_FLOORS (0 : nombre minimal de `floor_transition` dans
#   le journal de chaque client, la preuve d'un franchissement d'étage), MIN_RESIMULATED (1000 :
#   échec si le client le plus resimulé en compte moins), STRESS (1), SUSPEND_MS (12 : durée de
#   chaque suspension), RUN_MS (30 : durée entre deux suspensions), INPUT_DELAY (0),
#   OUT (/tmp/p2p-stress), TIMEOUT (900 s), MATCHBOX_PORT (3536), MATCHBOX_URL, NETEM,
#   KEEP_SIGNALING=1 : voir `scripts/p2p-lib.sh`. Tout autre `ALACOD_*` de l'environnement est
#   transmis aux clients (ex. `ALACOD_SKIP_HUB=1` pour gungeon).
# Sortie 0 seulement si : tous les codes de sortie 0, aucun `DesyncDetected`, traces identiques et
# de même longueur (et contiguës : `check-p2p-traces.py`), assez de frames resimulées, assez
# d'étages franchis.
# Bash, pas zsh. Docker : `export PATH=$HOME/.orbstack/bin:$PATH` sous OrbStack.
set -uo pipefail

[ $# -ge 2 ] || { sed -n '/^# Usage/,/^# Sortie/p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
GAME=$1
SCRIPT=$2
FRAMES=${3:-3000}
CLIENTS=${CLIENTS:-2}
GAME_ARGS=${GAME_ARGS:-}
EXPECT_FLOORS=${EXPECT_FLOORS:-0}
MIN_RESIMULATED=${MIN_RESIMULATED:-1000}
STRESS=${STRESS:-1}
SUSPEND_MS=${SUSPEND_MS:-12}
RUN_MS=${RUN_MS:-30}
INPUT_DELAY=${INPUT_DELAY:-0}
OUT=${OUT:-/tmp/p2p-stress}
TIMEOUT=${TIMEOUT:-900}
LOBBY="stress-$$"
[ -f "$SCRIPT" ] || { echo "p2p-stress : script introuvable : $SCRIPT"; exit 2; }
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
suspender=""
cleanup() {
    [ -n "$suspender" ] && kill "$suspender" 2>/dev/null
    [ -n "${pids[1]:-}" ] && kill -CONT "${pids[1]}" 2>/dev/null
    p2p_signaling_down
}
trap cleanup EXIT
p2p_signaling_up
p2p_calm

PLAYERS=$(p2p_players "$CLIENTS")
START=$(date +%s)
for ((i = 0; i < CLIENTS; i++)); do
    # shellcheck disable=SC2086
    ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$FRAMES" \
    ALACOD_INPUT="script:$SCRIPT" ALACOD_INPUT_DELAY="$INPUT_DELAY" ALACOD_SNAPSHOT_PROFILE=1 \
    APP_VERSION=x \
    "$BIN" --matchbox "$SIGNAL_URL" --lobby "$LOBBY" --number-player "$CLIENTS" \
        --players $PLAYERS --cid "client_$i" --name "client_$i" $GAME_ARGS \
        > "$OUT/p2p-$i.log" 2>&1 &
    pids+=("$!")
done

# Suspend le client 1 par à-coups jusqu'à la fin de l'un des deux clients.
if [ "$STRESS" = 1 ] && [ "$CLIENTS" -ge 2 ]; then
    (
        # Laisse la connexion s'établir : suspendre dès le départ ferait expirer le handshake.
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

# Garde-fou : tue les clients au-delà de TIMEOUT.
( sleep "$TIMEOUT"; kill -CONT "${pids[1]:-0}" 2>/dev/null; kill "${pids[@]}" 2>/dev/null ) >/dev/null 2>&1 &
watchdog=$!

codes=()
for ((i = 0; i < CLIENTS; i++)); do
    wait "${pids[$i]}"
    codes+=("$?")
done
kill "$watchdog" ${suspender:+"$suspender"} 2>/dev/null
wait "$watchdog" ${suspender:+"$suspender"} 2>/dev/null
WALL=$(( $(date +%s) - START ))

fail=0
echo "p2p-stress : jeu=$GAME script=$SCRIPT frames=$FRAMES clients=$CLIENTS input_delay=$INPUT_DELAY stress=$STRESS netem='${NETEM:-}' args='$GAME_ARGS' durée=${WALL}s"
echo "codes de sortie : ${codes[*]}"
for c in "${codes[@]}"; do
    [ "$c" = 0 ] || { echo "ÉCHEC : code de sortie non nul"; fail=1; break; }
done

desync=$(grep -c "DesyncDetected" "$OUT"/p2p-*.log | awk -F: '{s+=$2} END{print s+0}')
echo "DesyncDetected : $desync"
[ "$desync" = 0 ] || { echo "ÉCHEC : desync signalé par GGRS"; fail=1; }

resim=()
max=-1
for ((i = 0; i < CLIENTS; i++)); do
    n=$(p2p_resimulated "$OUT/p2p-$i.log")
    resim+=("$n")
    [ "$n" -gt "$max" ] && max=$n
done
echo "frames resimulées (journal du jeu) : ${resim[*]}"
# Profondeur et fréquence des rollbacks (`StateTraceRecorder::rollback`, ligne « rollbacks : n=… » du journal).
for ((i = 0; i < CLIENTS; i++)); do
    echo "client $i : $(grep -h '^.* rollbacks : n=' "$OUT/p2p-$i.log" | tail -1 | sed 's/^.*INFO [^ ]* //' || true)"
done
if [ "$max" -lt "$MIN_RESIMULATED" ]; then
    echo "ÉCHEC : $max frames resimulées < $MIN_RESIMULATED (MIN_RESIMULATED) : run sans rollback réel, preuve sans valeur"
    fail=1
fi

# Étages franchis : `ggrs{f=N floor_transition from=0 to=1 …}`. Le journal le répète à chaque
# resimulation de la frame (rollback) : on compte les passages distincts (de → vers), pas les lignes.
floors=()
for ((i = 0; i < CLIENTS; i++)); do
    n=$(sed -n 's/.*floor_transition from=\([0-9]*\) to=\([0-9]*\).*/\1>\2/p' "$OUT/p2p-$i.log" | sort -u | wc -l | tr -d ' ')
    floors+=("$n")
    [ "$n" -ge "$EXPECT_FLOORS" ] || { echo "ÉCHEC : client $i : $n transition(s) d'étage < $EXPECT_FLOORS (EXPECT_FLOORS)"; fail=1; }
done
echo "transitions d'étage distinctes (journal) : ${floors[*]}"
for ((i = 0; i < CLIENTS; i++)); do
    grep -h 'floor_transition' "$OUT/p2p-$i.log" | tail -1 | sed -n 's/.*ggrs{f=\([0-9]*\) floor_transition from=\([0-9]*\) to=\([0-9]*\).*/  client '"$i"' : dernière résimulation f\1, étage \2 -> \3/p'
done

# Déconnexion GGRS (`GGRS Event: Disconnected`, « Player '…' disconnected ») AVANT l'écriture de
# la trace : un échec. Après, le pair qui a fini sa trace et quitté est normal. Une interruption
# réseau (`NetworkInterrupted`, silence de plus de 500 ms) suivie de sa reprise (`NetworkResumed`)
# est rapportée sans être un échec (réseau dégradé) ; une interruption jamais reprise en est un.
disc=0
interrupted=0
resumed=0
for ((i = 0; i < CLIENTS; i++)); do
    n=$(awk '/trace d.état écrite dans/ {exit} /GGRS Event: Disconnected/ {c++} END{print c+0}' "$OUT/p2p-$i.log")
    disc=$((disc + n))
    a=$(awk '/trace d.état écrite dans/ {exit} /GGRS Event: NetworkInterrupted/ {c++} END{print c+0}' "$OUT/p2p-$i.log")
    b=$(awk '/trace d.état écrite dans/ {exit} /GGRS Event: NetworkResumed/ {c++} END{print c+0}' "$OUT/p2p-$i.log")
    interrupted=$((interrupted + a))
    resumed=$((resumed + b))
    [ "$a" -le "$b" ] || { echo "ÉCHEC : client $i : interruption réseau jamais reprise ($a interruptions, $b reprises)"; fail=1; }
done
echo "déconnexions avant la fin de la trace : $disc ; interruptions réseau reprises : $resumed/$interrupted"
[ "$disc" = 0 ] || { echo "ÉCHEC : déconnexion GGRS en cours de partie"; fail=1; }

# Chaque trace fait exactement FRAMES-1 lignes (la frame FRAMES-1 n'est jamais écrite).
want=$((FRAMES - 1))
missing=0
for ((i = 0; i < CLIENTS; i++)); do [ -f "$OUT/p2p-$i.trace" ] || missing=1; done
if [ "$missing" = 0 ]; then
    for ((i = 0; i < CLIENTS; i++)); do
        l=$(wc -l < "$OUT/p2p-$i.trace" | tr -d ' ')
        [ "$l" = "$want" ] || { echo "ÉCHEC : longueur de la trace du client $i : $l lignes, attendu $want (FRAMES-1)"; fail=1; }
    done
    for ((i = 1; i < CLIENTS; i++)); do
        if cmp -s "$OUT/p2p-0.trace" "$OUT/p2p-$i.trace"; then
            echo "cmp client 0 / client $i : identiques ($want lignes attendues)"
        else
            first=$(cmp "$OUT/p2p-0.trace" "$OUT/p2p-$i.trace" 2>&1 | sed -n 's/.*line \([0-9]*\).*/\1/p')
            echo "ÉCHEC : traces 0 et $i DIFFÉRENTES (première ligne différente : ${first:-?} ; $OUT/p2p-{0,$i}.trace)"
            fail=1
        fi
    done
    traces=()
    for ((i = 0; i < CLIENTS; i++)); do traces+=("$OUT/p2p-$i.trace"); done
    python3 "$(dirname "${BASH_SOURCE[0]}")/check-p2p-traces.py" --frames "$want" "${traces[@]}" || fail=1
else
    echo "ÉCHEC : trace absente (voir $OUT/p2p-*.log)"
    fail=1
fi

[ "$fail" = 0 ] && echo "p2p-stress : OK" || echo "p2p-stress : ÉCHEC (journaux : $OUT/p2p-*.log)"
exit "$fail"
