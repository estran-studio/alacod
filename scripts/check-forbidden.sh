#!/usr/bin/env bash
# Toujours bloquant ; --strict est conservé pour les anciens appels.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "$root/scripts/check-forbidden.py" "$@"
