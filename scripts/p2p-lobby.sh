#!/usr/bin/env bash
# Retour au lobby en ligne (D86, docs/conventions.md §57) : TRIALS essais CONSÉCUTIFS de `p2p-restart.sh`
# avec `RESTART_KIND=lobby` (bouton « Lobby » de l'écran de fin), les deux clients posant leur requête à
# un écart de 100 à 400 ms (`GAPS_MS`, `RESTART_DELAYS_MS` de `p2p-restart.sh`) ; le client retardé
# alterne d'un essai à l'autre. Un essai est vert si `p2p-restart.sh` l'est : les deux clients
# reviennent au lobby, se retrouvent (ni l'un ni l'autre ne se connecte à l'ancien socket du
# pair), jouent la partie 2 (graine d'origine : identique à la partie 1) et leurs traces sont
# identiques pour la partie 1 et pour la partie 2, sans `DesyncDetected`. Le script s'arrête au premier
# essai rouge (« consécutifs ») et sort en code 1 ; le résumé donne, par essai, l'écart RÉEL entre les deux
# requêtes (horodatages du journal), les pairs ignorés (journal « d'une session précédente ignoré ») et
# les abandons de session non synchronisée.
#
# Usage : scripts/p2p-lobby.sh [jeu]          (zombies par défaut)
# Variables : TRIALS (5), GAPS_MS (« 100 175 250 325 400 », un écart par essai, repris en boucle),
#   OUT (/tmp/p2p-lobby : un dossier par essai), CLIENTS (2 : un seul client est retardé par essai,
#   à tour de rôle), RESTART_AT (300), EXIT_AT (300),
#   le reste (INPUTS, EXPECT_VICTORY, GAME_ARGS, ALACOD_SKIP_HUB, NETEM, MATCHBOX_PORT…) est transmis à
#   `p2p-restart.sh` (voir son en-tête). Fin de run réelle : `RESTART_AT=7200 EXIT_AT=1500 EXPECT_VICTORY=1
#   INPUTS=<enregistrement de bots> ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" scripts/p2p-lobby.sh gungeon`.
# Bash, pas zsh.
set -uo pipefail

GAME=${1:-zombies}
TRIALS=${TRIALS:-5}
GAPS_MS=${GAPS_MS:-"100 175 250 325 400"}
OUT=${OUT:-/tmp/p2p-lobby}
CLIENTS=${CLIENTS:-2}
export RESTART_AT=${RESTART_AT:-300}
export EXIT_AT=${EXIT_AT:-300}
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
read -r -a gaps <<< "$GAPS_MS"
mkdir -p "$OUT"

# Secondes depuis minuit d'un horodatage de journal tracing (« 2026-10-17T01:19:54.123456Z »).
log_seconds() {
    sed -n 's/.*T\([0-9][0-9]\):\([0-9][0-9]\):\([0-9][0-9]\)\.\([0-9]*\)Z.*/\1 \2 \3 \4/p' | head -1 |
        awk '{ printf "%.3f", $1 * 3600 + $2 * 60 + $3 + ("0." $4) }'
}

green=0
summary=()
for ((t = 1; t <= TRIALS; t++)); do
    gap=${gaps[(t - 1) % ${#gaps[@]}]}
    late=$(((t - 1) % CLIENTS))
    delays=()
    for ((i = 0; i < CLIENTS; i++)); do
        if [ "$i" = "$late" ]; then delays+=("$gap"); else delays+=(0); fi
    done
    dir="$OUT/essai-$t"
    echo "=== essai $t/$TRIALS : écart demandé ${gap} ms, client $late retardé, $(date +%T) ==="
    RESTART_KIND=lobby RESTART_DELAYS_MS="${delays[*]}" OUT="$dir" CLIENTS="$CLIENTS" \
        "$HERE/p2p-restart.sh" "$GAME" 2>&1 | tee "$OUT/essai-$t.txt"
    status=${PIPESTATUS[0]}
    first="" last=""
    for ((i = 0; i < CLIENTS; i++)); do
        s=$(grep -m1 'RunRequest::ToLobby' "$dir/p2p-$i.log" 2>/dev/null | log_seconds)
        [ -n "$s" ] || continue
        if [ -z "$first" ] || awk "BEGIN{exit !($s < $first)}"; then first=$s; fi
        if [ -z "$last" ] || awk "BEGIN{exit !($s > $last)}"; then last=$s; fi
    done
    real="?"
    [ -z "$first" ] || real=$(awk "BEGIN{printf \"%.0f\", ($last - $first) * 1000}")
    ignored=$( (grep -h -c "d'une session précédente ignoré" "$dir"/p2p-*.log 2>/dev/null || true) | awk '{s+=$1} END{print s+0}')
    timeouts=$( (grep -h -c "session_sync_timeout" "$dir"/p2p-*.log 2>/dev/null || true) | awk '{s+=$1} END{print s+0}')
    if [ "$status" = 0 ]; then verdict=VERT; green=$((green + 1)); else verdict=ROUGE; fi
    summary+=("essai $t : $verdict, écart réel entre les requêtes ${real} ms (demandé ${gap}), pairs obsolètes ignorés : $ignored, abandons de session : $timeouts")
    [ "$status" = 0 ] || break
done
echo "=== p2p-lobby : $GAME, $green/$TRIALS essais verts consécutifs ==="
printf '%s\n' "${summary[@]}"
[ "$green" = "$TRIALS" ]
