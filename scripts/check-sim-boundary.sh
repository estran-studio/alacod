#!/usr/bin/env bash
# Frontière simulation / présentation, prouvée par la compilation (CLAUDE.md « Simulation et
# présentation », plan §7 : une cible console remplace ce qui est derrière `PresentationPlugin`,
# jamais la simulation ; les crates de simulation ne dépendent de bevy qu'en ECS seul).
#
#   scripts/check-sim-boundary.sh [--no-check]
#
# Pour chaque crate de simulation (SIM) :
#   1. son graphe de dépendances **seul** (`cargo tree -p <crate> -e normal
#      --no-default-features`, sans unification des features du reste du workspace) ne contient
#      aucun crate de présentation ou d'E/S (FORBIDDEN) ;
#   2. les crates SIM compilent ensemble en ECS seul (`cargo check --no-default-features`,
#      sauté avec `--no-check`) : le code ne s'appuie sur aucune feature arrivée par un voisin.
# Les crates PENDING portent une fuite connue (dette D69, `docs/taches/dettes.md`) : leur graphe
# est affiché, pas bloquant ; retirer un crate de PENDING vers SIM quand sa fuite est fermée.
# Tout crate de `crates/` absent des trois listes fait échouer : un nouveau crate se classe.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# ECS seul, bloquant.
SIM=(sim_core stats effects behaviors world run items meta content bevy_fixed utils)
# Simulation avec une fuite de présentation connue (D69) : combat (animation, loaders RON,
# bevy_matchbox), animation (sprites), map/map_ldtk (bevy_ecs_ldtk, rendu), game (sim + UI,
# caméra, audio, réseau).
PENDING=(combat animation map map_ldtk game)
# Présentation, outillage, tests : hors frontière.
APP=(bots scenario telemetry)

FORBIDDEN='bevy_render|bevy_winit|bevy_asset|bevy_audio|bevy_ui|bevy_sprite|bevy_text|bevy_pbr|bevy_window|bevy_camera|bevy_gizmos|bevy_picking|bevy_ecs_ldtk|bevy_matchbox|bevy_kira_audio|winit|wgpu'

status=0
for dir in crates/*/; do
    crate="$(basename "$dir")"
    case " ${SIM[*]} ${PENDING[*]} ${APP[*]} " in
    *" $crate "*) ;;
    *)
        echo "ERREUR : crates/$crate n'est classé ni SIM, ni PENDING, ni APP (scripts/check-sim-boundary.sh)" >&2
        status=1
        ;;
    esac
done

leaks() {
    cargo tree -p "$1" -e normal --no-default-features --prefix none 2>/dev/null \
        | awk '{print $1}' | sort -u | grep -Ex "$FORBIDDEN" | tr '\n' ' ' || true
}

for crate in "${SIM[@]}"; do
    found="$(leaks "$crate")"
    if [ -n "$found" ]; then
        echo "ERREUR : $crate (simulation) tire des crates de présentation : $found" >&2
        status=1
    else
        echo "ok : $crate (ECS seul)"
    fi
done

for crate in "${PENDING[@]}"; do
    found="$(leaks "$crate")"
    echo "dette D69 : $crate tire ${found:-(rien)}"
done

if [ "$status" -ne 0 ]; then
    exit "$status"
fi

if [ "${1:-}" != "--no-check" ]; then
    echo "cargo check ECS seul : ${SIM[*]}"
    # shellcheck disable=SC2046
    cargo check -q --profile headless --no-default-features $(printf -- '-p %s ' "${SIM[@]}")
fi
echo "OK : frontière simulation/présentation respectée par ${#SIM[@]} crates ; ${#PENDING[@]} en dette D69"
