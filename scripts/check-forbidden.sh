#!/usr/bin/env bash

# Script to check for forbidden patterns in specific crates.
# Usage: ./scripts/check-forbidden.sh [--strict]
#
# Forbidden patterns:
#  - std::collections::HashMap (use deterministic alternatives)
#  - std::collections::HashSet (use deterministic alternatives)
#  - rand:: (RNG forbidden in simulation, use deterministic seeding)
#  - Instant::now (system clock forbidden, use simulation frame time)
#
# Exit codes:
#  - 0: Forbidden patterns found but warnings only (no --strict)
#  - 1: Forbidden patterns found with --strict flag, or parsing error

set -euo pipefail

STRICT=false
if [[ "${1:-}" == "--strict" ]]; then
    STRICT=true
fi

# Crates to check
CRATES=(
    "crates/sim_core/src"
    "crates/combat/src"
    "crates/game/src"
    "crates/map/src"
    "crates/map_ldtk/src"
    "crates/bevy_fixed/src"
    "crates/bots/src"
)

# Forbidden patterns
PATTERNS=(
    "std::collections::HashMap"
    "std::collections::HashSet"
    "rand::"
    "Instant::now"
)

# Compteurs par motif, même index que PATTERNS (D33 : pas de `declare -A`, inexistant sous
# bash 3.2, le bash de macOS).
pattern_counts=()
for i in "${!PATTERNS[@]}"; do
    pattern_counts[i]=0
done

# Search for forbidden patterns
for crate in "${CRATES[@]}"; do
    if [[ ! -d "$crate" ]]; then
        echo "Warning: Crate directory not found: $crate"
        continue
    fi

    for i in "${!PATTERNS[@]}"; do
        pattern="${PATTERNS[i]}"
        # Grep for the pattern, showing file and line number
        while IFS= read -r line; do
            echo "  $line"
            pattern_counts[i]=$((pattern_counts[i] + 1))
        done < <(grep -rn "$pattern" "$crate" --include="*.rs" 2>/dev/null || true)
    done
done

# Display summary
echo ""
echo "=== Forbidden Pattern Check Summary ==="
total_found=0
for i in "${!PATTERNS[@]}"; do
    pattern="${PATTERNS[i]}"
    count=${pattern_counts[i]:-0}
    total_found=$((total_found + count))
    printf "  %s: %d occurrences\n" "$pattern" "$count"
done

echo "  Total: $total_found occurrences"
echo ""

# Determine exit code
if [[ $total_found -gt 0 ]]; then
    if [[ "$STRICT" == true ]]; then
        echo "ERROR: Forbidden patterns found (--strict mode)"
        exit 1
    else
        echo "WARNING: Forbidden patterns found (use --strict to fail)"
        exit 0
    fi
else
    echo "OK: No forbidden patterns found"
    exit 0
fi
