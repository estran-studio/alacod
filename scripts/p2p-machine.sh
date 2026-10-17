#!/usr/bin/env bash
# UN client p2p headless d'une partie à plusieurs MACHINES (m2-p2p-run-complete, point 2 : x86-64 et
# arm64, deux processus sur deux machines, vrai réseau). À lancer sur chaque machine, même commit,
# même binaire de jeu, mêmes inputs, même salle, même signaling joignable des deux :
#
#   # machine A (x86-64)                                  # machine B (arm64)
#   MATCHBOX_URL=wss://hôte LOBBY=run-1 \                 MATCHBOX_URL=wss://hôte LOBBY=run-1 \
#     ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" \             ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" \
#     scripts/p2p-machine.sh gungeon run.ron 4500 0         scripts/p2p-machine.sh gungeon run.ron 4500 1
#
# `run.ron` : `scripts/p2p-gungeon-inputs.sh 2 10 5000 run.ron` (sur UNE machine, le fichier est copié
# sur l'autre : les bots sont figés en inputs, ce sont ces inputs qui sont rejoués des deux côtés).
# Chaque client exige FRAMES-1 lignes contiguës (`check-p2p-traces.py --single`). Chaque machine écrit `$OUT/p2p-<index>.trace` et affiche : architecture, code de sortie,
# `DesyncDetected`, frames resimulées, transitions d'étage, nombre de lignes et empreinte de la trace.
# Comparaison : copier une trace sur l'autre machine puis
#   python3 scripts/check-p2p-traces.py --frames $((4500-1)) $OUT/p2p-0.trace $OUT/p2p-1.trace
# (ou `cmp`/`sha256sum` des deux fichiers).
#
# Usage : scripts/p2p-machine.sh <jeu> <script-inputs.ron> <frames> <index>
# Variables : MATCHBOX_URL (obligatoire), LOBBY (obligatoire, identique des deux côtés), CLIENTS (2 :
#   nombre total de pairs, un par machine), GAME_ARGS, INPUT_DELAY (0), OUT (/tmp/p2p-machine),
#   TIMEOUT (900 s), P2P_NO_BUILD=1. `ALACOD_*` transmis au client.
set -uo pipefail

[ $# -eq 4 ] || { sed -n '/^# Usage/,/^set -uo/p' "$0" | sed 's/^# \{0,1\}//' | head -6; exit 2; }
GAME=$1
SCRIPT=$2
FRAMES=$3
INDEX=$4
CLIENTS=${CLIENTS:-2}
GAME_ARGS=${GAME_ARGS:-}
INPUT_DELAY=${INPUT_DELAY:-0}
OUT=${OUT:-/tmp/p2p-machine}
TIMEOUT=${TIMEOUT:-900}
: "${MATCHBOX_URL:?MATCHBOX_URL obligatoire (signaling joignable des deux machines)}"
: "${LOBBY:?LOBBY obligatoire (la même salle des deux côtés)}"
[ -f "$SCRIPT" ] || { echo "p2p-machine : script introuvable : $SCRIPT"; exit 2; }
SCRIPT=$(cd "$(dirname "$SCRIPT")" && pwd)/$(basename "$SCRIPT")

# shellcheck source=scripts/p2p-lib.sh
. "$(dirname "${BASH_SOURCE[0]}")/p2p-lib.sh"
p2p_init "$GAME"
mkdir -p "$OUT"
rm -f "$OUT/p2p-$INDEX.trace" "$OUT/p2p-$INDEX.log"

sha() { if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi; }
p2p_calm
PLAYERS=$(p2p_players "$CLIENTS")
echo "p2p-machine : $(uname -sm), commit $(git rev-parse --short HEAD), client_$INDEX, salle $LOBBY, signaling $MATCHBOX_URL"
# shellcheck disable=SC2086
ALACOD_HEADLESS=1 ALACOD_STATE_TRACE="$OUT/p2p-$INDEX.trace" ALACOD_EXIT_AT_FRAME="$FRAMES" \
ALACOD_INPUT="script:$SCRIPT" ALACOD_INPUT_DELAY="$INPUT_DELAY" ALACOD_SNAPSHOT_PROFILE=1 APP_VERSION=x \
"$BIN" --matchbox "$MATCHBOX_URL" --lobby "$LOBBY" --number-player "$CLIENTS" \
    --players $PLAYERS --cid "client_$INDEX" --name "client_$INDEX" $GAME_ARGS \
    > "$OUT/p2p-$INDEX.log" 2>&1 &
pid=$!
# Garde-fou portable (pas de `timeout` sous macOS) : un pair qui n'arrive jamais bloque l'attente.
( sleep "$TIMEOUT"; kill "$pid" 2>/dev/null ) >/dev/null 2>&1 &
watchdog=$!
wait "$pid"
code=$?
kill "$watchdog" 2>/dev/null

fail=0
echo "code de sortie : $code"
[ "$code" = 0 ] || fail=1
desync=$(grep -c "DesyncDetected" "$OUT/p2p-$INDEX.log")
echo "DesyncDetected : $desync"
[ "$desync" = 0 ] || fail=1
echo "frames resimulées : $(p2p_resimulated "$OUT/p2p-$INDEX.log")"
echo "transitions d'étage distinctes : $(sed -n 's/.*floor_transition from=\([0-9]*\) to=\([0-9]*\).*/\1>\2/p' "$OUT/p2p-$INDEX.log" | sort -u | wc -l | tr -d ' ') (dernière : $(grep 'floor_transition' "$OUT/p2p-$INDEX.log" | tail -1 | sed -n 's/.*ggrs{\(f=[0-9]* floor_transition[^}]*\)}.*/\1/p'))"
if [ -f "$OUT/p2p-$INDEX.trace" ]; then
    echo "trace : $(wc -l < "$OUT/p2p-$INDEX.trace" | tr -d ' ') lignes (attendu $((FRAMES - 1))), sha256 $(sha "$OUT/p2p-$INDEX.trace")"
    echo "dernière ligne : $(tail -1 "$OUT/p2p-$INDEX.trace")"
    # Exactement FRAMES-1 lignes contiguës (de 0 à FRAMES-2) : `confirmed_end` tronque la trace si les
    # dernières frames n'ont jamais été confirmées (pair parti, 10 s d'attente), et un sha256 égal ne
    # prouve rien si les deux traces sont tronquées pareil.
    python3 "$(dirname "${BASH_SOURCE[0]}")/check-p2p-traces.py" --single --frames "$((FRAMES - 1))" "$OUT/p2p-$INDEX.trace" || fail=1
    grep -q "trace d'état tronquée" "$OUT/p2p-$INDEX.log" && { echo "ÉCHEC : le journal signale une trace tronquée (frames jamais confirmées)"; fail=1; }
else
    echo "ÉCHEC : trace absente (voir $OUT/p2p-$INDEX.log)"
    fail=1
fi
[ "$fail" = 0 ] && echo "p2p-machine : client_$INDEX OK (comparer les traces des deux machines)" || echo "p2p-machine : client_$INDEX ÉCHEC"
exit "$fail"
