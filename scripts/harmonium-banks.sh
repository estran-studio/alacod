#!/usr/bin/env bash
# Banques de la musique générative (M-7, docs/conventions.md § 121) : GeneralUser-GS (SF3, 7,6 Mo, licence dans
# `GeneralUser-GS-LICENSE.txt`) et harmonium_trad.sf2 (percussion, 0,15 Mo). Ce sont les mêmes fichiers que la vitrine
# web d'harmonium (release `studio-resources-2` de estran-studio/harmonium). Elles ne sont **jamais commitées** (gitignorées :
# `games/*/assets/harmonium/`) ; ce script les télécharge, une fois par poste, et les pose dans `games/<jeu>/assets/harmonium/`
# (chemins par défaut de `music.main_bank` / `music.percussion_bank` du manifeste). Un build web les copie avec les assets
# du jeu (`scripts/web-assets.mjs`). Idempotent : une deuxième exécution ne change rien.
#
# Usage :
#   scripts/harmonium-banks.sh [--source DOSSIER] [JEU ...]
#     (rien)           télécharge (gh) dans $ALACOD_HARMONIUM_BANKS_DIR (défaut ~/orca/workspaces/alacod/packs/harmonium-banks),
#                      puis copie dans games/testbed/assets/harmonium/ ;
#     --source DOSSIER copie depuis ce dossier tel quel (aucun accès réseau) ;
#     JEU ...          jeux visés (défaut : testbed).
# Variables : HARMONIUM_RELEASE (défaut studio-resources-2), HARMONIUM_REPO (défaut estran-studio/harmonium).
# Code de sortie : 0 fait, 1 erreur (fichier absent, gh manquant), 2 usage.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="${ALACOD_HARMONIUM_BANKS_DIR:-$HOME/orca/workspaces/alacod/packs/harmonium-banks}"
REPO="${HARMONIUM_REPO:-estran-studio/harmonium}"
RELEASE="${HARMONIUM_RELEASE:-studio-resources-2}"
FILES=(GeneralUser-GS.sf3 GeneralUser-GS-LICENSE.txt harmonium_trad.sf2)

source_dir=""
games=()
while [ $# -gt 0 ]; do
  case "$1" in
    --source) [ $# -ge 2 ] || { echo "harmonium-banks : --source demande un dossier" >&2; exit 2; }; source_dir="$2"; shift 2 ;;
    -h|--help) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    -*) echo "harmonium-banks : option inconnue « $1 »" >&2; exit 2 ;;
    *) games+=("$1"); shift ;;
  esac
done
[ ${#games[@]} -gt 0 ] || games=(testbed)

if [ -z "$source_dir" ]; then
  command -v gh >/dev/null || { echo "harmonium-banks : gh (GitHub CLI) est requis pour télécharger $REPO@$RELEASE" >&2; exit 1; }
  mkdir -p "$CACHE"
  for file in "${FILES[@]}"; do
    [ -s "$CACHE/$file" ] && continue
    gh release download "$RELEASE" --repo "$REPO" --pattern "$file" --dir "$CACHE"
  done
  source_dir="$CACHE"
fi

for file in "${FILES[@]}"; do
  [ -s "$source_dir/$file" ] || { echo "harmonium-banks : $source_dir/$file absent" >&2; exit 1; }
done
for game in "${games[@]}"; do
  dest="$ROOT/games/$game/assets/harmonium"
  [ -d "$ROOT/games/$game/assets" ] || { echo "harmonium-banks : games/$game/assets introuvable" >&2; exit 1; }
  mkdir -p "$dest"
  for file in "${FILES[@]}"; do
    cmp -s "$source_dir/$file" "$dest/$file" || cp "$source_dir/$file" "$dest/$file"
  done
  echo "harmonium-banks : games/$game/assets/harmonium/ ($(du -ch "$dest"/* | tail -1 | cut -f1))"
done
