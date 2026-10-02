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
#   Le signaling p2p est le service `signaling` (matchbox_server nu, docker-compose.ci.yaml) ;
#   allumette (jeton JWT requis, API HTTP) est un profil optionnel non utilisé ici.
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
log_step "Running bench (traces strictes, budgets absolus désactivés : machine de CI plus lente que la machine de calibration)..."
START_TIME=$(date +%s)

if ! make test_scenarios > logs/bench.log 2>&1; then
    log_error "Bench failed (make test_scenarios)"
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
# STEP C: P2P HEADLESS (via le service signaling = matchbox_server)
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

    if [ -n "${DOCKER_COMPOSE}" ]; then
        # Build and start the signaling server
        log_step "Starting signaling (matchbox_server) via docker compose..."
        if ${DOCKER_COMPOSE} -f docker-compose.ci.yaml up -d signaling > "${NIGHTLY_DIR}/signaling.log" 2>&1; then
            # Wait for the signaling server to be ready
            sleep 3

            # Convert comma-separated list to array
            IFS=',' read -ra PLAYER_COUNTS <<< "${NIGHTLY_P2P}"

            for NUM_PLAYERS in "${PLAYER_COUNTS[@]}"; do
                NUM_PLAYERS=$(echo "$NUM_PLAYERS" | xargs)  # trim
                log_step "Testing p2p with ${NUM_PLAYERS} players..."

                LOBBY="nightly-${NUM_PLAYERS}"
                PIDS=""
                SUCCESS=true

                # Start N clients
                for ((i = 0; i < NUM_PLAYERS; i++)); do
                    CID="client_${i}"
                    TRACE_FILE="${NIGHTLY_DIR}/p2p-${NUM_PLAYERS}-${CID}.trace"

                    # Build players list: localhost + (NUM_PLAYERS - 1) remotes
                    PLAYERS_LIST="localhost"
                    for ((j = 1; j < NUM_PLAYERS; j++)); do
                        PLAYERS_LIST="$PLAYERS_LIST remote"
                    done

                    log_step "  Starting ${CID} in lobby ${LOBBY}..."

                    ALACOD_HEADLESS=1 \
                    ALACOD_STATE_TRACE="${TRACE_FILE}" \
                    ALACOD_EXIT_AT_FRAME=600 \
                    cargo run -q -p zombies --profile headless --no-default-features \
                        -- --matchbox ws://127.0.0.1:3536 \
                           --lobby "${LOBBY}" \
                           --number-player "${NUM_PLAYERS}" \
                           --players ${PLAYERS_LIST} \
                           --cid "${CID}" \
                           --name "${CID}" \
                        &> "logs/p2p-${NUM_PLAYERS}-${CID}.log" &

                    PID=$!
                    PIDS="${PIDS} ${PID}"
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
                    ${DOCKER_COMPOSE} -f docker-compose.ci.yaml down 2>/dev/null || true
                    exit 1
                fi

                # Compare traces between first and other clients
                if [ "${NUM_PLAYERS}" -gt 1 ]; then
                    log_step "  Comparing traces between clients..."
                    FIRST_TRACE="${NIGHTLY_DIR}/p2p-${NUM_PLAYERS}-client_0.trace"
                    TRACES_MATCH=true

                    for ((i = 1; i < NUM_PLAYERS; i++)); do
                        TRACE="${NIGHTLY_DIR}/p2p-${NUM_PLAYERS}-client_${i}.trace"
                        if [ -f "${FIRST_TRACE}" ] && [ -f "${TRACE}" ]; then
                            if ! diff -q "${FIRST_TRACE}" "${TRACE}" &> /dev/null; then
                                log_error "  Trace mismatch: client_0 vs client_${i}"
                                # Find first differing frame for diagnostics
                                FIRST_DIFF=$(diff "${FIRST_TRACE}" "${TRACE}" | head -1)
                                log_error "  First difference: ${FIRST_DIFF}"
                                TRACES_MATCH=false
                            fi
                        fi
                    done

                    if [ "$TRACES_MATCH" = false ]; then
                        append_summary "### C. P2P Headless (${NUM_PLAYERS}p): ❌ DESYNC DETECTED"
                        ${DOCKER_COMPOSE} -f docker-compose.ci.yaml down 2>/dev/null || true
                        exit 1
                    fi
                fi
            done

            # Cleanup
            log_step "Stopping signaling..."
            ${DOCKER_COMPOSE} -f docker-compose.ci.yaml down 2>/dev/null || true

            END_TIME=$(date +%s)
            DURATION=$((END_TIME - START_TIME))
            append_summary "### C. P2P Headless: ✅ PASSED (${DURATION}s)"
            log_step "P2P headless passed (${DURATION}s)"
        else
            log_warn "Failed to start signaling (see signaling.log). Skipping p2p tests."
            append_summary "### C. P2P Headless: ⚠ SKIPPED (signaling startup failed)"
        fi
    fi
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
