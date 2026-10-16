#!/usr/bin/env bash
# p2p à deux clients avec rollbacks RÉELS (outillage-p2p-stress ; recette de M3-T2 industrialisée).
#
# Sans stress, deux clients sur une boucle locale ne rollbackent presque jamais (« frames
# resimulées après rollback : 0 à 3 ») : la comparaison des traces ne prouve alors rien. Ici :
#   - `ALACOD_INPUT_DELAY=0` : une entrée distante tardive est une prédiction fausse ;
#   - `ALACOD_INPUT=script:<script>` : des entrées qui changent toutes les 1 à 3 frames ;
#   - le client 1 est suspendu et repris en boucle (SIGSTOP/SIGCONT) : le client 0 le devance,
#     prédit faux, puis resimule.
# Le compteur de frames resimulées est celui du journal du jeu (`StateTraceRecorder::resimulated`,
# une frame déjà enregistrée puis enregistrée de nouveau), jamais les lignes `ggrs{f=N}` en
# double (une ligne émise une fois par exemplaire n'est pas une resimulation).
#
# Usage : scripts/p2p-stress.sh <jeu> <script-inputs.ron> [frames]
#   <jeu>      : paquet cargo du jeu (zombies, testbed, isaac, gungeon, throne)
#   <script>   : scénario RON dont les `PlayerScript` des handles 0 et 1 sont rejoués
#   [frames]   : ALACOD_EXIT_AT_FRAME (3000 par défaut)
# Variables : MIN_RESIMULATED (1000 : échec si le client le plus resimulé en compte moins),
#   SUSPEND_MS (12 : durée de chaque suspension), RUN_MS (30 : durée entre deux suspensions),
#   INPUT_DELAY (0), OUT (/tmp/p2p-stress), TIMEOUT (900 s), MATCHBOX_PORT (3536),
#   KEEP_SIGNALING=1 (laisse le signaling démarré).
# Sortie 0 seulement si : deux codes de sortie 0, aucun `DesyncDetected`, traces identiques et
# de même longueur, assez de frames resimulées.
# Bash, pas zsh. Docker : `export PATH=$HOME/.orbstack/bin:$PATH` sous OrbStack.
set -uo pipefail

[ $# -ge 2 ] || { sed -n '/^# Usage/,/^# Sortie/p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
GAME=$1
SCRIPT=$2
FRAMES=${3:-3000}
MIN_RESIMULATED=${MIN_RESIMULATED:-1000}
SUSPEND_MS=${SUSPEND_MS:-12}
RUN_MS=${RUN_MS:-30}
INPUT_DELAY=${INPUT_DELAY:-0}
OUT=${OUT:-/tmp/p2p-stress}
TIMEOUT=${TIMEOUT:-900}
export MATCHBOX_PORT=${MATCHBOX_PORT:-3536}
LOBBY="stress-$$"
[ -f "$SCRIPT" ] || { echo "p2p-stress : script introuvable : $SCRIPT"; exit 2; }

cd "$(dirname "${BASH_SOURCE[0]}")/.."
ROOT=$PWD
mkdir -p "$OUT"
rm -f "$OUT"/p2p-*.trace "$OUT"/p2p-*.log

DEV_FEATURES="$(./scripts/dev-features.sh features)"
cargo build -q -p "$GAME" --profile headless --no-default-features $DEV_FEATURES || exit 1
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
BIN="$TARGET/headless/$GAME"
[ -x "$BIN" ] || { echo "p2p-stress : binaire introuvable : $BIN"; exit 2; }
libpath="$(./scripts/dev-features.sh libpath 2>/dev/null || true)"
export LD_LIBRARY_PATH="${libpath:+$libpath:}${LD_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="${libpath:+$libpath:}${DYLD_LIBRARY_PATH:-}"

docker compose -f docker-compose.ci.yaml up -d signaling >/dev/null || exit 1
cleanup() {
    [ -n "${suspender:-}" ] && kill "$suspender" 2>/dev/null
    [ -n "${pids[1]:-}" ] && kill -CONT "${pids[1]}" 2>/dev/null
    [ "${KEEP_SIGNALING:-0}" = "1" ] || docker compose -f docker-compose.ci.yaml down >/dev/null 2>&1
}
trap cleanup EXIT
sleep 3

pids=()
for i in 0 1; do
    ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$i.trace" ALACOD_EXIT_AT_FRAME="$FRAMES" \
    ALACOD_INPUT="script:$SCRIPT" ALACOD_INPUT_DELAY="$INPUT_DELAY" APP_VERSION=x \
    "$BIN" --matchbox "ws://127.0.0.1:$MATCHBOX_PORT" --lobby "$LOBBY" --number-player 2 \
        --players localhost remote --cid "client_$i" --name "client_$i" \
        > "$OUT/p2p-$i.log" 2>&1 &
    pids+=("$!")
done

# Suspend le client 1 par à-coups jusqu'à la fin de l'un des deux clients.
(
    # Laisse la connexion s'établir : suspendre dès le départ ferait expirer le handshake.
    sleep 4
    while kill -0 "${pids[0]}" 2>/dev/null && kill -0 "${pids[1]}" 2>/dev/null; do
        kill -STOP "${pids[1]}" 2>/dev/null || break
        sleep "$(awk "BEGIN{print $SUSPEND_MS/1000}")"
        kill -CONT "${pids[1]}" 2>/dev/null || break
        sleep "$(awk "BEGIN{print $RUN_MS/1000}")"
    done
) &
suspender=$!

# Garde-fou : tue les clients au-delà de TIMEOUT.
( sleep "$TIMEOUT"; kill -CONT "${pids[1]}" 2>/dev/null; kill "${pids[@]}" 2>/dev/null ) &
watchdog=$!

codes=()
for i in 0 1; do
    wait "${pids[$i]}"
    codes+=("$?")
done
kill "$watchdog" "$suspender" 2>/dev/null
wait "$watchdog" "$suspender" 2>/dev/null

fail=0
echo "p2p-stress : jeu=$GAME script=$SCRIPT frames=$FRAMES input_delay=$INPUT_DELAY"
echo "codes de sortie : client 0 = ${codes[0]}, client 1 = ${codes[1]}"
{ [ "${codes[0]}" = 0 ] && [ "${codes[1]}" = 0 ]; } || { echo "ÉCHEC : code de sortie non nul"; fail=1; }

desync=$(grep -c "DesyncDetected" "$OUT/p2p-0.log" "$OUT/p2p-1.log" | awk -F: '{s+=$2} END{print s+0}')
echo "DesyncDetected : $desync"
[ "$desync" = 0 ] || { echo "ÉCHEC : desync signalé par GGRS"; fail=1; }

resim=()
for i in 0 1; do
    n=$(sed -n 's/.*frames resimulées après rollback : \([0-9][0-9]*\).*/\1/p' "$OUT/p2p-$i.log" | tail -1)
    resim+=("${n:--1}")
done
echo "frames resimulées (journal du jeu) : client 0 = ${resim[0]}, client 1 = ${resim[1]}"
max=${resim[0]}
[ "${resim[1]}" -gt "$max" ] && max=${resim[1]}
if [ "$max" -lt "$MIN_RESIMULATED" ]; then
    echo "ÉCHEC : $max frames resimulées < $MIN_RESIMULATED (MIN_RESIMULATED) : run sans rollback réel, preuve sans valeur"
    fail=1
fi

# Déconnexion GGRS (`Player '…' disconnected`, `NetworkInterrupted`) AVANT l'écriture de la
# trace : un échec. Après, le pair qui a fini sa trace et quitté est normal.
disc=0
for i in 0 1; do
    n=$(awk '/trace d.état écrite dans/ {exit} /disconnected|Disconnected|NetworkInterrupted/ {c++} END{print c+0}' "$OUT/p2p-$i.log")
    disc=$((disc + n))
done
echo "déconnexions avant la fin de la trace : $disc"
[ "$disc" = 0 ] || { echo "ÉCHEC : déconnexion GGRS en cours de partie"; fail=1; }

# Chaque trace fait exactement FRAMES-1 lignes (la frame FRAMES-1 n'est jamais écrite).
want=$((FRAMES - 1))
if [ -f "$OUT/p2p-0.trace" ] && [ -f "$OUT/p2p-1.trace" ]; then
    l0=$(wc -l < "$OUT/p2p-0.trace" | tr -d ' ')
    l1=$(wc -l < "$OUT/p2p-1.trace" | tr -d ' ')
    if [ "$l0" != "$want" ] || [ "$l1" != "$want" ]; then
        echo "ÉCHEC : longueur des traces : client 0 = $l0, client 1 = $l1 lignes, attendu $want (FRAMES-1)"
        fail=1
    fi
    if cmp -s "$OUT/p2p-0.trace" "$OUT/p2p-1.trace"; then
        echo "cmp des traces : identiques ($l0 lignes comparées, attendu $want)"
    else
        first=$(cmp "$OUT/p2p-0.trace" "$OUT/p2p-1.trace" 2>&1 | sed -n 's/.*line \([0-9]*\).*/\1/p')
        echo "ÉCHEC : traces DIFFÉRENTES (première ligne différente : ${first:-?} ; $l0 et $l1 lignes ; $OUT/p2p-{0,1}.trace)"
        fail=1
    fi
else
    echo "ÉCHEC : trace absente (voir $OUT/p2p-*.log)"
    fail=1
fi

[ "$fail" = 0 ] && echo "p2p-stress : OK" || echo "p2p-stress : ÉCHEC (journaux : $OUT/p2p-*.log)"
exit "$fail"
