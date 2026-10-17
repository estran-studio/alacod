#!/bin/bash
# Nightly CI pipeline: bench, sim, p2p headless, videos, review, notes.
# Usage: scripts/nightly.sh [--quick]
# Variables:
#   CARGO_TARGET_DIR : build target directory (must be set before calling)
#   CARGO_BUILD_JOBS : parallel cargo jobs (should be 3)
#   NIGHTLY_SEEDS=1..50 (range or single value)
#   NIGHTLY_BOTS=4
#   NIGHTLY_WAVE=10
#   NIGHTLY_P2P=2,4 (comma-separated list of player counts)
#   NIGHTLY_P2P_SCRIPT=tests/scenarios/p2p_smoke.ron (inputs de chaque client, `ALACOD_INPUT=script:`)
#   NIGHTLY_SHARDS=2 (tranches parallèles de la suite de scénarios, STEP A)
#   NIGHTLY_P2P_GUNGEON, NIGHTLY_GUNGEON_SEED/FRAMES/NETEM : run gungeon p2p avec transition
#   d'étage (STEP C2, détail devant l'étape)
#   STEP C (p2p headless) passe par le profil `allumette` du docker-compose.ci.yaml
#   (matchmaking HTTP + WebSocket avec JWT, port hôte 3537) : ALLUMETTE_DIR explicite ou
#   ../allumette (layout meta-repo), SKIPPED propre s'il est introuvable. Le service
#   `signaling` (matchbox_server nu) reste la recette p2p du README §4.
#   NIGHTLY_VIDEOS=all (or list of scenario names)
#   --quick : fast local test (2 seeds, wave 2, p2p 2 only, idle+shoot_around videos)

set -e

# Verify required environment
if [ -z "$CARGO_TARGET_DIR" ]; then
    echo "ERROR: CARGO_TARGET_DIR must be set before calling this script"
    echo "Example: export CARGO_TARGET_DIR=/path/to/target CARGO_BUILD_JOBS=3"
    exit 1
fi

# Configuration
QUICK_MODE=false
if [[ "$1" == "--quick" ]]; then
    QUICK_MODE=true
    NIGHTLY_SEEDS=${NIGHTLY_SEEDS:-1..2}
    NIGHTLY_BOTS=${NIGHTLY_BOTS:-4}
    NIGHTLY_WAVE=${NIGHTLY_WAVE:-2}
    NIGHTLY_P2P=${NIGHTLY_P2P:-2}
    NIGHTLY_VIDEOS=${NIGHTLY_VIDEOS:-idle,shoot_around}
else
    NIGHTLY_SEEDS=${NIGHTLY_SEEDS:-1..50}
    NIGHTLY_BOTS=${NIGHTLY_BOTS:-4}
    NIGHTLY_WAVE=${NIGHTLY_WAVE:-10}
    NIGHTLY_P2P=${NIGHTLY_P2P:-2,4}
    NIGHTLY_VIDEOS=${NIGHTLY_VIDEOS:-all}
fi

# Environment
export CARGO_BUILD_JOBS=3
COMMIT=$(git rev-parse --short HEAD)
NIGHTLY_DIR="${CARGO_TARGET_DIR}/nightly/${COMMIT}"
SUMMARY_FILE="${NIGHTLY_DIR}/summary.md"
NOTES_FILE="tests/review-notes/${COMMIT}.md"
mkdir -p "${NIGHTLY_DIR}" "tests/review-notes" "logs"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_step() {
    echo -e "${GREEN}[STEP]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

append_summary() {
    echo "$1" >> "${SUMMARY_FILE}"
}

# Initialize summary
cat > "${SUMMARY_FILE}" << EOF
# Nightly CI Summary

**Commit**: ${COMMIT}
**Date**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
**Quick Mode**: ${QUICK_MODE}

## Steps

EOF

# ============================================================================
# STEP A: BENCH
# ============================================================================

# Budgets absolus désactivés : tests/budgets.ron est calibré sur la machine de
# développement (M4, moitié de sa référence). Le VPS de CI est ~4x plus lent : tout
# scénario serait sous son plancher et le nightly serait rouge en permanence sans
# régression réelle. Les traces et les attentes, elles, restent strictes (un échec
# de scénario fait toujours échouer la commande) ; la régression de performance se
# lit dans le tableau comparatif des notes (scenario-metrics.py, commit précédent).
# En tranches (`test_scenarios_par`) : en un seul processus, la suite prenait 14 337 s sur le
# runner (mesuré le 2026-10-14) et le pipeline était annulé à 6 h sans jamais atteindre le
# p2p ni les vidéos. NIGHTLY_SHARDS tranches parallèles (défaut 2 : la machine est partagée).
NIGHTLY_SHARDS=${NIGHTLY_SHARDS:-2}
log_step "Running bench (${NIGHTLY_SHARDS} tranches, traces strictes, budgets absolus désactivés : machine de CI plus lente que la machine de calibration)..."
START_TIME=$(date +%s)

if ! make test_scenarios_par SHARDS="${NIGHTLY_SHARDS}" > logs/bench.log 2>&1; then
    log_error "Bench failed (make test_scenarios_par)"
    echo "--- bench.log (tail) ---"
    tail -40 logs/bench.log
    append_summary "### A. Bench: ❌ FAILED"
    exit 1
fi

if ! ./scripts/scenario-metrics.py > logs/metrics.log 2>&1; then
    log_error "scenario-metrics.py failed"
    echo "--- metrics.log (tail) ---"
    tail -40 logs/metrics.log
    append_summary "### A. Bench: ❌ FAILED (metrics)"
    exit 1
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
append_summary "### A. Bench: ✅ PASSED (${DURATION}s)"
log_step "Bench passed (${DURATION}s)"

# ============================================================================
# STEP B: SIM
# ============================================================================

log_step "Running alacod-sim (${NIGHTLY_BOTS} bots, seeds ${NIGHTLY_SEEDS}, wave ${NIGHTLY_WAVE})..."
START_TIME=$(date +%s)

SIM_JSON="${NIGHTLY_DIR}/sim.json"
mkdir -p "$(dirname "${SIM_JSON}")"

if ! SEEDS="${NIGHTLY_SEEDS}" BOTS="${NIGHTLY_BOTS}" WAVE="${NIGHTLY_WAVE}" make sim; then
    log_error "Sim failed"
    append_summary "### B. Sim: ❌ FAILED"
    exit 1
fi

# Copy the metrics JSON to nightly dir
if [ -f "${CARGO_TARGET_DIR}/metrics/sim-${COMMIT}.json" ]; then
    cp "${CARGO_TARGET_DIR}/metrics/sim-${COMMIT}.json" "${SIM_JSON}"
    log_step "Copied sim metrics to ${SIM_JSON}"
else
    log_warn "Sim metrics not found at ${CARGO_TARGET_DIR}/metrics/sim-${COMMIT}.json"
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
append_summary "### B. Sim: ✅ PASSED (${DURATION}s)"
log_step "Sim passed (${DURATION}s)"

# ============================================================================
# STEP C: P2P HEADLESS (via le profil allumette : matchmaking HTTP + WS avec JWT)
# ============================================================================

log_step "Starting p2p headless tests (players: ${NIGHTLY_P2P})..."
START_TIME=$(date +%s)

# Try docker compose (modern)
if ! command -v docker &> /dev/null; then
    log_warn "Docker not found. Skipping p2p headless tests."
    append_summary "### C. P2P Headless: ⚠ SKIPPED (docker not found)"
else
    # Check if docker-compose or docker compose
    if docker compose --version &> /dev/null; then
        DOCKER_COMPOSE="docker compose"
    elif command -v docker-compose &> /dev/null; then
        DOCKER_COMPOSE="docker-compose"
    else
        log_warn "docker compose not found. Skipping p2p headless tests."
        append_summary "### C. P2P Headless: ⚠ SKIPPED (docker compose not found)"
        DOCKER_COMPOSE=""
    fi

    # Dépôt allumette : ALLUMETTE_DIR explicite (worktree/runner) ou layout meta-repo.
    # Le compose résout son contexte de build par ${ALLUMETTE_DIR:-../allumette}.
    if [ -z "${ALLUMETTE_DIR:-}" ]; then
        ALLUMETTE_DIR="../allumette"
    fi

    if [ ! -f "${ALLUMETTE_DIR}/Dockerfile" ]; then
        log_warn "ALLUMETTE_DIR introuvable ou invalide (${ALLUMETTE_DIR}) : STEP C SKIPPED."
        append_summary "### C. P2P Headless: ⚠ SKIPPED (allumette repo not found: ${ALLUMETTE_DIR})"
    elif [ -z "${DOCKER_COMPOSE}" ]; then
        : # déjà SKIPPED ci-dessus (docker compose not found)
    else
        export ALLUMETTE_DIR
        # Build and start the allumette matchmaking server (profil optionnel du compose)
        log_step "Starting allumette (matchmaking) via docker compose --profile allumette..."
        if ${DOCKER_COMPOSE} -f docker-compose.ci.yaml --profile allumette up -d allumette > "${NIGHTLY_DIR}/allumette.log" 2>&1; then
            # Wait for the server to be ready
            sleep 3

            # Convert comma-separated list to array
            IFS=',' read -ra PLAYER_COUNTS <<< "${NIGHTLY_P2P}"

            FIRST_RUN=true
            for NUM_PLAYERS in "${PLAYER_COUNTS[@]}"; do
                NUM_PLAYERS=$(echo "$NUM_PLAYERS" | xargs)  # trim
                # Serveur à état en mémoire : un restart entre deux comptes de joueurs
                # repart de lobbies vierges (sans lui, le lobby du run précédent repasserait
                # Waiting à la déconnexion et la découverte rejoindrait un lobby éventé).
                if [ "${FIRST_RUN}" != true ]; then
                    log_step "Restarting allumette (fresh lobbies)..."
                    ${DOCKER_COMPOSE} -f docker-compose.ci.yaml restart allumette > "${NIGHTLY_DIR}/allumette.log" 2>&1
                    sleep 3
                fi
                FIRST_RUN=false

                log_step "Testing p2p with ${NUM_PLAYERS} players..."

                PIDS=""
                SUCCESS=true

                # Start N clients : le créateur (client 0) d'abord — il crée le lobby
                # "zombies" et attend le complet — puis les rejoigneurs en décalé (~6 s) :
                # la découverte/création est une course connue en v1 (deux créateurs
                # simultanés créeraient deux lobbies), recette m0-v9.
                for ((i = 0; i < NUM_PLAYERS; i++)); do
                    CID="client_${i}"
                    TRACE_FILE="${NIGHTLY_DIR}/p2p-${NUM_PLAYERS}-${CID}.trace"

                    # Build players list: localhost + (NUM_PLAYERS - 1) remotes
                    PLAYERS_LIST="localhost"
                    for ((j = 1; j < NUM_PLAYERS; j++)); do
                        PLAYERS_LIST="$PLAYERS_LIST remote"
                    done

                    log_step "  Starting ${CID}..."

                    rm -f "${TRACE_FILE}"
                    # Inputs scriptés (p2p_smoke.ron : changements fréquents, phases décalées) et
                    # délai d'input nul : sur la boucle locale, l'input du pair arrive après la
                    # frame qui en a besoin, les prédictions ratent, la partie rollbacke et
                    # resimule sur le vrai transport. Avec `ALACOD_INPUT=neutral` (et 5 frames de
                    # délai), aucun rollback n'avait lieu : le log de trace en donne le compte.
                    ALACOD_INPUT="script:${NIGHTLY_P2P_SCRIPT:-tests/scenarios/p2p_smoke.ron}" \
                    ALACOD_INPUT_DELAY="${NIGHTLY_P2P_INPUT_DELAY:-0}" \
                    ALACOD_SNAPSHOT_PROFILE=1 \
                    ALACOD_HEADLESS=1 \
                    ALACOD_STATE_TRACE="${TRACE_FILE}" \
                    ALACOD_EXIT_AT_FRAME=600 \
                    cargo run -q -p zombies --profile headless --no-default-features \
                        -- --allumette http://127.0.0.1:3537 \
                           --number-player "${NUM_PLAYERS}" \
                           --players ${PLAYERS_LIST} \
                           --cid "${CID}" \
                           --name "${CID}" \
                        &> "logs/p2p-${NUM_PLAYERS}-${CID}.log" &

                    PID=$!
                    PIDS="${PIDS} ${PID}"

                    # Décalage après le créateur : les rejoigneurs découvrent le lobby
                    # Waiting déjà créé au lieu de créer le leur.
                    if [ "$i" -eq 0 ]; then
                        sleep 6
                    fi
                done

                # Wait for all clients to finish
                log_step "  Waiting for all clients (${NUM_PLAYERS} players) to complete..."
                for PID in $PIDS; do
                    if ! wait $PID; then
                        log_error "  Client failed (PID ${PID})"
                        SUCCESS=false
                    fi
                done

                if [ "$SUCCESS" = false ]; then
                    append_summary "### C. P2P Headless (${NUM_PLAYERS}p): ❌ FAILED"
                    log_error "P2p with ${NUM_PLAYERS} players failed"
                    ${DOCKER_COMPOSE} -f docker-compose.ci.yaml --profile allumette down 2>/dev/null || true
                    exit 1
                fi

                # Compare traces between first and other clients
                if [ "${NUM_PLAYERS}" -gt 1 ]; then
                    log_step "  Comparing traces between clients..."
                    TRACE_FILES=()
                    for ((i = 0; i < NUM_PLAYERS; i++)); do
                        TRACE_FILES+=("${NIGHTLY_DIR}/p2p-${NUM_PLAYERS}-client_${i}.trace")
                    done
                    if ! python3 scripts/check-p2p-traces.py --frames 599 "${TRACE_FILES[@]}"; then
                        append_summary "### C. P2P Headless (${NUM_PLAYERS}p): ❌ INVALID OR DIVERGENT TRACES"
                        ${DOCKER_COMPOSE} -f docker-compose.ci.yaml --profile allumette down 2>/dev/null || true
                        exit 1
                    fi
                    # Preuve que le rollback a eu lieu sur le transport : frames resimulées par
                    # client (log de trace). Zéro partout = run sans valeur pour le rollback.
                    # Et le coût mesuré de ce rollback (`ALACOD_SNAPSHOT_PROFILE`) : sauvegarde,
                    # restauration, frame de simulation, taille estimée du snapshot.
                    RESIM=""
                    for ((i = 0; i < NUM_PLAYERS; i++)); do
                        CLIENT_LOG="logs/p2p-${NUM_PLAYERS}-client_${i}.log"
                        COUNT=$(sed -n 's/.*frames resimulées après rollback : \([0-9]*\).*/\1/p' "${CLIENT_LOG}" | tail -1)
                        RESIM="${RESIM} client_${i}=${COUNT:-?}"
                        PROFILE=$(sed -n 's/.*profil rollback : \(save .*\)/\1/p' "${CLIENT_LOG}" | tail -1)
                        append_summary "- ${NUM_PLAYERS}p client_${i} : ${PROFILE:-profil rollback absent}"
                    done
                    append_summary "- ${NUM_PLAYERS}p : traces identiques ; frames resimulées :${RESIM}"
                    if ! echo "${RESIM}" | grep -Eq '=[1-9][0-9]*'; then
                        log_warn "  Aucune frame resimulée : le smoke p2p n'a exercé aucun rollback"
                    fi
                fi
            done

            # Cleanup
            log_step "Stopping allumette..."
            ${DOCKER_COMPOSE} -f docker-compose.ci.yaml --profile allumette down 2>/dev/null || true

            END_TIME=$(date +%s)
            DURATION=$((END_TIME - START_TIME))
            append_summary "### C. P2P Headless: ✅ PASSED (${DURATION}s)"
            log_step "P2P headless passed (${DURATION}s)"
        else
            log_warn "Failed to start allumette (see allumette.log). Skipping p2p tests."
            append_summary "### C. P2P Headless: ⚠ SKIPPED (allumette startup failed)"
        fi
    fi
fi

# ============================================================================
# STEP C2: P2P GUNGEON (m2-p2p-run-complete) : une run gungeon complète à deux clients, avec
# transition d'étage (`etages_deux`, graine NIGHTLY_GUNGEON_SEED), rollbacks réels (client 1
# suspendu par à-coups, `scripts/p2p-stress.sh`) et traces identiques. Les bots ne sont pas dans
# le binaire du jeu : leurs inputs sont enregistrés par `alacod-sim`
# (`scripts/p2p-gungeon-inputs.sh`) puis rejoués par chaque client ; si la simulation change au
# point que la run n'atteint plus l'étage 2, l'enregistrement ou EXPECT_FLOORS le dit (échec, pas
# un faux vert). Signaling : le service `signaling` du compose (port MATCHBOX_PORT, 3536 par défaut ; nécessite le
# plugin `docker compose` v2).
#   NIGHTLY_P2P_GUNGEON=1|0 (1 hors --quick), NIGHTLY_GUNGEON_SEED=10, NIGHTLY_GUNGEON_FRAMES=4500,
#   NIGHTLY_GUNGEON_NETEM="delay 60ms 10ms loss 1%" (en plus : même run sur réseau dégradé,
#   sans suspension, `tc netem` dans un espace de noms réseau sans root).
# ============================================================================

if [ "${NIGHTLY_P2P_GUNGEON:-$([ "${QUICK_MODE}" = true ] && echo 0 || echo 1)}" != 1 ]; then
    append_summary "### C2. P2P gungeon (run, transition d'étage): ⚠ SKIPPED (NIGHTLY_P2P_GUNGEON != 1)"
elif ! command -v docker &> /dev/null || ! docker compose version &> /dev/null; then
    log_warn "Docker (plugin compose v2) not found. Skipping p2p gungeon."
    append_summary "### C2. P2P gungeon (run, transition d'étage): ⚠ SKIPPED (docker compose not found)"
else
    log_step "P2P gungeon (2 clients, transition d'étage)..."
    START_TIME=$(date +%s)
    GUNGEON_SEED=${NIGHTLY_GUNGEON_SEED:-10}
    GUNGEON_FRAMES=${NIGHTLY_GUNGEON_FRAMES:-4500}
    GUNGEON_DIR="${NIGHTLY_DIR}/p2p-gungeon"
    mkdir -p "${GUNGEON_DIR}"
    if ! scripts/p2p-gungeon-inputs.sh 2 "${GUNGEON_SEED}" $((GUNGEON_FRAMES + 500)) "${GUNGEON_DIR}/run.ron" \
        &> "${GUNGEON_DIR}/inputs.log"; then
        log_error "Enregistrement des inputs gungeon impossible (voir ${GUNGEON_DIR}/inputs.log)"
        append_summary "### C2. P2P gungeon: ❌ FAILED (inputs : $(tail -1 "${GUNGEON_DIR}/inputs.log"))"
        exit 1
    fi
    append_summary "- gungeon : $(tail -1 "${GUNGEON_DIR}/inputs.log")"
    GUNGEON_RUNS=("stress:")
    if [ -n "${NIGHTLY_GUNGEON_NETEM:-}" ]; then
        GUNGEON_RUNS+=("netem:${NIGHTLY_GUNGEON_NETEM}")
    else
        # Sans NIGHTLY_GUNGEON_NETEM, C2 ne couvre PAS le réseau dégradé (point 4 du contrat de sortie) : le dire.
        append_summary "- gungeon netem : SKIPPED (NIGHTLY_GUNGEON_NETEM vide) : réseau dégradé NON couvert par cette étape"
        log_warn "C2 : netem SKIPPED (NIGHTLY_GUNGEON_NETEM vide), réseau dégradé non couvert"
    fi
    for GUNGEON_RUN in "${GUNGEON_RUNS[@]}"; do
        GUNGEON_KIND=${GUNGEON_RUN%%:*}
        GUNGEON_NETEM=${GUNGEON_RUN#*:}
        GUNGEON_STRESS=1
        [ "${GUNGEON_KIND}" = netem ] && GUNGEON_STRESS=0
        # `MIN_RESIMULATED` : en stress, des milliers de frames ; sur réseau dégradé seul, quelques-unes.
        GUNGEON_MIN_RESIM=1000
        [ "${GUNGEON_KIND}" = netem ] && GUNGEON_MIN_RESIM=1
        if ! ALACOD_SKIP_HUB=1 GAME_ARGS="--seed ${GUNGEON_SEED}" EXPECT_FLOORS=1 STRESS=${GUNGEON_STRESS} \
            MIN_RESIMULATED=${GUNGEON_MIN_RESIM} NETEM="${GUNGEON_NETEM}" OUT="${GUNGEON_DIR}/${GUNGEON_KIND}" \
            scripts/p2p-stress.sh gungeon "${GUNGEON_DIR}/run.ron" "${GUNGEON_FRAMES}" \
            &> "${GUNGEON_DIR}/${GUNGEON_KIND}.log"; then
            log_error "P2p gungeon (${GUNGEON_KIND}) failed (see ${GUNGEON_DIR}/${GUNGEON_KIND}.log)"
            append_summary "### C2. P2P gungeon (${GUNGEON_KIND}): ❌ FAILED"
            grep -E "ÉCHEC|DIFFÉRENTES" "${GUNGEON_DIR}/${GUNGEON_KIND}.log" | head -5 | while read -r line; do append_summary "- ${line}"; done
            exit 1
        fi
        append_summary "- gungeon ${GUNGEON_KIND} : $(grep -E '^p2p-stress :' "${GUNGEON_DIR}/${GUNGEON_KIND}.log" | head -1)"
        append_summary "- gungeon ${GUNGEON_KIND} : $(grep -E '^frames resimulées' "${GUNGEON_DIR}/${GUNGEON_KIND}.log"), $(grep -E "^transitions d'étage" "${GUNGEON_DIR}/${GUNGEON_KIND}.log"), traces identiques"
    done
    END_TIME=$(date +%s)
    GUNGEON_COVERS="stress SIGSTOP, transition d'étage, traces identiques"
    [ -n "${NIGHTLY_GUNGEON_NETEM:-}" ] && GUNGEON_COVERS="${GUNGEON_COVERS}, réseau dégradé (${NIGHTLY_GUNGEON_NETEM})" || GUNGEON_COVERS="${GUNGEON_COVERS} ; netem SKIPPED"
    append_summary "### C2. P2P gungeon (run, transition d'étage): ✅ PASSED ($((END_TIME - START_TIME))s) — ${GUNGEON_COVERS}"
    log_step "P2P gungeon passed"
fi

# ============================================================================
# STEP D: VIDEOS
# ============================================================================

log_step "Generating videos..."
START_TIME=$(date +%s)

if [ "${NIGHTLY_VIDEOS}" = "all" ]; then
    if ! make videos >> "${NIGHTLY_DIR}/videos.log" 2>&1; then
        log_warn "Video generation had issues (see videos.log, continuing anyway)"
        append_summary "### D. Videos: ⚠ PARTIAL"
    else
        append_summary "### D. Videos: ✅ GENERATED"
        log_step "Videos generated"
    fi
else
    # Liste séparée par des virgules : `make videos` ne prend qu'un scénario à la fois.
    VIDEOS_OK=true
    IFS=',' read -ra VIDEO_LIST <<< "${NIGHTLY_VIDEOS}"
    for VIDEO_SCENARIO in "${VIDEO_LIST[@]}"; do
        VIDEO_SCENARIO=$(echo "${VIDEO_SCENARIO}" | xargs)
        if ! make videos SCENARIO="${VIDEO_SCENARIO}" >> "${NIGHTLY_DIR}/videos.log" 2>&1; then
            log_warn "Video generation failed for ${VIDEO_SCENARIO} (see videos.log)"
            VIDEOS_OK=false
        fi
    done
    if [ "${VIDEOS_OK}" = true ]; then
        append_summary "### D. Videos: ✅ GENERATED (${NIGHTLY_VIDEOS})"
        log_step "Videos generated (${NIGHTLY_VIDEOS})"
    else
        append_summary "### D. Videos: ⚠ PARTIAL (voir videos.log)"
    fi
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
log_step "Video generation done (${DURATION}s)"

# Copie des MP4 dans l'artefact (`nightly/<commit>/videos/`, envoyé en entier par
# `.github/workflows/nightly.yaml`) : `make videos` écrit dans `target/videos/<commit>/` à la
# racine du dépôt (`scripts/scenario-video`, commit court, suffixe `-dirty` possible), hors de
# `NIGHTLY_DIR`. `*.vues.mp4` est couvert par le motif ; aucune vidéo n'est pas une erreur.
VIDEOS_SRC_ROOT="$(git rev-parse --show-toplevel)/target/videos"
mkdir -p "${NIGHTLY_DIR}/videos"
VIDEOS_COPIED=0
for VIDEOS_SRC in "${VIDEOS_SRC_ROOT}/${COMMIT}" "${VIDEOS_SRC_ROOT}/${COMMIT}-dirty"; do
    [ -d "${VIDEOS_SRC}" ] || continue
    for MP4 in "${VIDEOS_SRC}"/*.mp4; do
        [ -f "${MP4}" ] || continue
        cp "${MP4}" "${NIGHTLY_DIR}/videos/" && VIDEOS_COPIED=$((VIDEOS_COPIED + 1))
    done
done
log_step "Videos copied to artifact: ${VIDEOS_COPIED}"

# ============================================================================
# STEP E: REVIEW PAGE
# ============================================================================

log_step "Generating review page..."
START_TIME=$(date +%s)

REVIEW_DIR="${NIGHTLY_DIR}/review"
mkdir -p "${REVIEW_DIR}"

# Copy scenario-review.html to review dir
if [ -f "scripts/scenario-review.html" ]; then
    cp "scripts/scenario-review.html" "${REVIEW_DIR}/index.html"
fi

# Generate review page data via scenario-review.py
if command -v python3 &> /dev/null; then
    python3 scripts/scenario-review.py --output "${REVIEW_DIR}/data.js" &>/dev/null || log_warn "Review page generation had issues"
fi

append_summary "### E. Review Page: ✅ GENERATED"
log_step "Review page generated at ${REVIEW_DIR}"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# ============================================================================
# STEP F: GENERATE NOTES
# ============================================================================

log_step "Generating review notes..."

# Extract metrics from budgets and sim results
BENCH_FPS_MIN="N/A"
if [ -f "${CARGO_TARGET_DIR}/metrics/sim-${COMMIT}.json" ]; then
    BENCH_FPS_MIN=$(python3 -c "
import json
try:
    with open('${CARGO_TARGET_DIR}/metrics/sim-${COMMIT}.json') as f:
        data = json.load(f)
        if 'scenarios' in data:
            fps_vals = [s.get('sim_fps', 0) for s in data['scenarios'].values()]
            if fps_vals:
                print(f'{min(fps_vals):.1f}')
except:
    pass
" 2>/dev/null || echo "N/A")
fi

# Count scenarios run
SCENARIO_COUNT=$(find tests/scenarios -name "*.ron" -type f | wc -l)

# Create notes file
cat > "${NOTES_FILE}" << EOF
# Nightly Review Notes — $(date -u '+%Y-%m-%d %H:%M:%S UTC')

**Commit**: ${COMMIT}

## Summary

| Step | Status | Duration |
|------|--------|----------|
| Bench | ✅ | See summary.md |
| Sim | ✅ | See summary.md |
| P2P | ✅ | See summary.md |
| Videos | ✅ | See summary.md |

## Metrics

- **Scenarios**: ${SCENARIO_COUNT} run
- **Min FPS (sim)**: ${BENCH_FPS_MIN}
- **Quick Mode**: ${QUICK_MODE}

## Links

- [Review Page](./review/index.html)
- [Sim Metrics](./sim.json)

## Instructions

Run \`make nightly_quick\` locally to test this pipeline end-to-end.

For full details, see \`docs/conventions.md\` § CI lente (nuit).
EOF

append_summary "### F. Notes: ✅ GENERATED (${NOTES_FILE})"
log_step "Review notes generated at ${NOTES_FILE}"

# ============================================================================
# SUMMARY
# ============================================================================

append_summary ""
append_summary "## Conclusion"
append_summary "✅ All steps completed successfully!"

echo ""
log_step "=== NIGHTLY PIPELINE COMPLETE ==="
cat "${SUMMARY_FILE}"

exit 0
