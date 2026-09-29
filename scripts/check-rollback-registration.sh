#!/usr/bin/env bash

# Vérifie qu'aucun appel direct à `rollback_component_with*` / `rollback_resource_with*`
# (bevy_ggrs::RollbackApp) ne traîne en dehors de l'extension `RollbackTraceApp`
# (plan §9.6, tâche K0) : ces appels enregistrent le rollback sans checksum GGRS, donc
# sans que le synctest ni la détection de desync p2p ne puissent voir de divergence.
#
# Usage : ./scripts/check-rollback-registration.sh
#
# Contrairement à check-forbidden.sh, ce script est toujours strict : trouver un appel
# direct est une régression (l'état correspondant redevient invisible au checksum), pas
# un avertissement.
#
# Exit codes:
#  - 0: aucun appel direct trouvé hors de l'extension
#  - 1: au moins un appel direct trouvé, ou erreur

set -euo pipefail

# Fichier où l'extension elle-même vit, seul endroit autorisé à appeler
# rollback_component_with*/rollback_resource_with* (voir crates/game/src/rollback.rs,
# qui n'est qu'un réexport et ne les appelle pas).
ALLOWED_FILE="crates/utils/src/rollback.rs"

PATTERN='rollback_component_with|rollback_resource_with'

matches="$(grep -rnE "$PATTERN" crates --include='*.rs' | grep -v -F "$ALLOWED_FILE:" || true)"

if [[ -n "$matches" ]]; then
    echo "ERROR: appel direct à rollback_component_with*/rollback_resource_with* hors de $ALLOWED_FILE :"
    echo ""
    echo "$matches" | sed 's/^/  /'
    echo ""
    echo "Utilise l'extension RollbackTraceApp (rollback_and_trace, rollback_and_trace_debug,"
    echo "rollback_and_trace_resource, rollback_and_trace_debug_resource,"
    echo "rollback_and_trace_copy_resource, rollback_and_trace_copy_resource_no_checksum)"
    echo "depuis $ALLOWED_FILE à la place, pour que l'état reste couvert par le checksum GGRS."
    exit 1
fi

echo "OK: aucun appel direct à rollback_component_with*/rollback_resource_with* hors de $ALLOWED_FILE"
exit 0
