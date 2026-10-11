#!/usr/bin/env bash
# Édition de liens dynamique de Bevy en développement (tâche outillage-build).
#
#   scripts/dev-features.sh features   → "--features game/dev-dylib" ou rien
#   scripts/dev-features.sh libpath    → chemin des bibliothèques partagées (LD_LIBRARY_PATH)
#
# Une seule source de vérité pour le Makefile et les scripts. Toutes les commandes cargo d'une
# même arborescence DOIVENT passer les mêmes features : sinon Bevy est compilé deux fois et le
# target double. La dynamique est coupée (pas de sortie) quand :
#   - ALACOD_DYLIB=0 (choix explicite) ;
#   - CI est défini (GitHub Actions et le runner nightly : builds de référence, statiques) ;
#   - ALACOD_PROFILE=prod ou ALACOD_TARGET=web (release, wasm : le Makefile les exporte).
# Dans tous ces cas, le build est identique à celui d'avant la tâche.
set -euo pipefail

enabled() {
    [ "${ALACOD_DYLIB:-1}" != "0" ] && [ -z "${CI:-}" ] \
        && [ "${ALACOD_PROFILE:-}" != "prod" ] && [ "${ALACOD_TARGET:-}" != "web" ]
}

case "${1:-}" in
features)
    enabled && echo "--features game/dev-dylib" || true
    ;;
libpath)
    # Le dossier des binaires (libbevy_dylib.so y est « uplifté » par cargo) et la libstd partagée
    # du toolchain. Cargo les ajoute tout seul pour `cargo run`/`cargo test` ; cette valeur sert
    # aux binaires lancés à la main (target/headless/alacod-sim, play_scenario des vidéos…).
    root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    target="${CARGO_TARGET_DIR:-$root/target}"
    sysroot="$(cd "$root" && rustc --print sysroot)"
    host="$(cd "$root" && rustc -vV | sed -n 's/^host: //p')"
    echo "$target/headless:$target/debug:$sysroot/lib/rustlib/$host/lib"
    ;;
*)
    echo "usage : $0 features|libpath" >&2
    exit 2
    ;;
esac
