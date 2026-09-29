#!/usr/bin/env bash
# Hook PostToolUse (Edit|Write, voir .claude/settings.json) : quand le fichier édité est
# sous games/<jeu>/**, relance `alacod lint games/<jeu>` (T1.5, docs/taches.md). No-op pour
# tout le reste (silencieux, code 0) pour ne pas ralentir les autres éditions.
#
# Entrée : JSON du hook sur stdin (voir la doc des hooks Claude Code), champ
# `.tool_input.file_path` (chemin absolu édité par Edit ou Write).
set -euo pipefail

payload="$(cat)"
file_path="$(printf '%s' "$payload" | jq -r '.tool_input.file_path // empty' 2>/dev/null || true)"

if [[ -z "$file_path" ]]; then
    exit 0
fi

# Cherche .../games/<jeu>/... dans le chemin édité (chemin absolu attendu, mais on ne
# suppose rien d'autre sur le préfixe).
if [[ "$file_path" =~ /games/([^/]+)/ ]]; then
    game="${BASH_REMATCH[1]}"
else
    exit 0
fi

# Racine du dépôt = tout ce qui précède "/games/<jeu>/" dans le chemin édité (robuste au
# répertoire courant du hook, qui peut différer du dépôt selon comment il est lancé).
repo_root="${file_path%%/games/"$game"/*}"

if [[ ! -f "$repo_root/games/$game/assets/game.ron" ]]; then
    # Pas (ou plus) un dossier de jeu avec manifeste : rien à linter.
    exit 0
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-3}"

cd "$repo_root"
if ! output="$(cargo run -q -p content --bin alacod --profile headless -- lint "games/$game" 2>&1)"; then
    echo "alacod lint games/$game a échoué :" >&2
    echo "$output" >&2
    exit 1
fi
exit 0
