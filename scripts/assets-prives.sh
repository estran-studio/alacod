#!/usr/bin/env bash
# Surcharge privée des clones (S-0b, docs/conventions.md § 99) : images payantes (TheLazyStone, Aymen B, Ittai Manero) et
# extraits Sonniss, qui n'entrent jamais dans le dépôt public. Ils vivent dans le dépôt privé
# https://github.com/estran-studio/alacod-assets, arborescence `<jeu>/…` = chemins de `games/<jeu>/assets/…`, et ce script
# les copie dans les chemins gitignorés du poste (ou du checkout de la CI). Idempotent : une deuxième exécution ne change rien.
#
# Usage :
#   scripts/assets-prives.sh [--source DOSSIER] [--ci] [JEU ...]
#     (rien)           clone, ou met à jour (pull --ff-only), alacod-assets dans $ALACOD_ASSETS_DIR
#                      (défaut ~/orca/workspaces/alacod/packs/alacod-assets), puis copie ;
#     --source DOSSIER copie depuis ce dossier tel quel (aucun accès réseau, aucun git) ;
#     --ci             mode de l'étape du site : clone alacod-assets en SSH avec la clé ALACOD_ASSETS_DEPLOY_KEY ; sans la clé
#                      (fork, pull request) : message, code 0, rien n'est écrit (repli public) ; avec la clé et un clone qui
#                      échoue : code non nul. Pose ALACOD_WEB_PRIVATE_ASSETS=1 dans $GITHUB_ENV quand la copie a réussi ;
#     JEU ...          restreint à ces jeux (zombies, throne) ; défaut : tous ceux que le dépôt privé contient.
# Variables : ALACOD_ASSETS_URL (défaut https://github.com/estran-studio/alacod-assets.git ; SSH en --ci).
# Après la copie, un build web n'embarque la surcharge que sous ALACOD_WEB_PRIVATE_ASSETS=1 (D161) :
#   ALACOD_WEB_PRIVATE_ASSETS=1 make build_wasm_apps TARGET=web
# Code de sortie : 0 copie faite (ou --ci sans clé), 1 erreur (accès, chemin non couvert, chemin non gitignoré), 2 usage.
set -euo pipefail
set +x # la clé de déploiement ne passe jamais dans un journal

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PATTERNS="$ROOT/scripts/assets-prives.patterns"
DEFAULT_DIR="${ALACOD_ASSETS_DIR:-$HOME/orca/workspaces/alacod/packs/alacod-assets}"
HTTPS_URL="${ALACOD_ASSETS_URL:-https://github.com/estran-studio/alacod-assets.git}"
SSH_URL="${ALACOD_ASSETS_URL:-git@github.com:estran-studio/alacod-assets.git}"

source_dir=""
ci=0
games=()
while [ $# -gt 0 ]; do
  case "$1" in
    --source) [ $# -ge 2 ] || { echo "assets-prives : --source demande un dossier" >&2; exit 2; }; source_dir="$2"; shift 2 ;;
    --ci) ci=1; shift ;;
    -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    -*) echo "assets-prives : option inconnue « $1 »" >&2; exit 2 ;;
    *) games+=("$1"); shift ;;
  esac
done

die() { echo "assets-prives : $*" >&2; exit 1; }

tmp=""
cleanup() { [ -n "$tmp" ] && rm -rf "$tmp"; return 0; }
trap cleanup EXIT

# --- obtenir le dépôt privé -------------------------------------------------------------------------------------------------
if [ "$ci" = 1 ]; then
  if [ -z "${ALACOD_ASSETS_DEPLOY_KEY:-}" ]; then
    echo "::notice title=assets privés::ALACOD_ASSETS_DEPLOY_KEY absent (fork ou pull request) : surcharge privée sautée, le repli public est construit."
    echo "assets-prives : pas de clé de déploiement, étape sautée (repli public)."
    exit 0
  fi
  tmp="$(mktemp -d)"
  key="$tmp/deploy_key"
  ( umask 077; printf '%s\n' "$ALACOD_ASSETS_DEPLOY_KEY" > "$key" )
  chmod 600 "$key"
  source_dir="$tmp/alacod-assets"
  export GIT_SSH_COMMAND="ssh -i $key -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$tmp/known_hosts"
  git clone --quiet --depth 1 "$SSH_URL" "$source_dir" \
    || die "clone de alacod-assets impossible avec ALACOD_ASSETS_DEPLOY_KEY (clé de déploiement absente du dépôt, révoquée, ou réseau) ; ni la surcharge ni le repli public ne sont masqués : l'étape échoue."
elif [ -z "$source_dir" ]; then
  source_dir="$DEFAULT_DIR"
  if [ -d "$source_dir/.git" ]; then
    git -C "$source_dir" pull --quiet --ff-only \
      || die "mise à jour de $source_dir impossible (git pull --ff-only a échoué : accès au dépôt privé estran-studio/alacod-assets, ou historique divergent)."
  else
    [ -e "$source_dir" ] && die "$source_dir existe mais n'est pas un clone git."
    mkdir -p "$(dirname "$source_dir")"
    git clone --quiet "$HTTPS_URL" "$source_dir" \
      || die "clone de $HTTPS_URL impossible : le dépôt est privé, il faut un accès (gh auth login, ou ALACOD_ASSETS_URL en SSH). Rien n'a été copié."
  fi
fi
[ -d "$source_dir" ] || die "dossier source « $source_dir » introuvable."

# --- règles de chemin ---------------------------------------------------------------------------------------------------------
dir_rules=(); suffix_rules=()
while IFS= read -r line || [ -n "$line" ]; do
  line="${line#"${line%%[![:space:]]*}"}"; line="${line%"${line##*[![:space:]]}"}"
  case "$line" in ''|'#'*) continue ;; '*'*) suffix_rules+=("${line#\*}") ;; */) dir_rules+=("$line") ;; esac
done < "$PATTERNS"
[ $(( ${#dir_rules[@]} + ${#suffix_rules[@]} )) -gt 0 ] || die "$PATTERNS ne contient aucune règle."

is_private() { # $1 : chemin relatif à games/<jeu>/assets/
  local rule
  for rule in ${dir_rules[@]+"${dir_rules[@]}"}; do case "$1" in "$rule"*) return 0 ;; esac; done
  for rule in ${suffix_rules[@]+"${suffix_rules[@]}"}; do case "$1" in *"$rule") return 0 ;; esac; done
  return 1
}

# --- copie ---------------------------------------------------------------------------------------------------------------------
if [ ${#games[@]} -eq 0 ]; then
  for d in "$source_dir"/*/; do
    name="$(basename "$d")"
    [ "$name" = licences ] && continue
    games+=("$name")
  done
fi
[ ${#games[@]} -gt 0 ] || die "$source_dir ne contient aucun dossier de jeu (attendu : zombies/, throne/…)."

total_copied=0; total_same=0
for game in "${games[@]}"; do
  assets="$ROOT/games/$game/assets"
  [ -d "$assets" ] || die "jeu « $game » : $assets n'existe pas dans ce checkout."
  [ -d "$source_dir/$game" ] || { echo "assets-prives : $game : rien dans alacod-assets (dossier absent), ignoré."; continue; }
  copied=0; same=0
  while IFS= read -r -d '' file; do
    rel="${file#"$source_dir/$game/"}"
    [ "$(basename "$rel")" = .DS_Store ] && continue
    is_private "$rel" || die "$game/$rel n'est couvert par aucune règle de scripts/assets-prives.patterns : refus (il serait embarqué dans un build web public)."
    dest="$assets/$rel"
    if git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
      git -C "$ROOT" check-ignore -q -- "games/$game/assets/$rel" \
        || die "games/$game/assets/$rel n'est pas gitignoré (ou déjà suivi par git) : refus, il pourrait être commité dans le dépôt public. Ajouter la règle dans .gitignore."
    fi
    if [ -f "$dest" ] && cmp -s "$file" "$dest"; then same=$((same + 1)); continue; fi
    mkdir -p "$(dirname "$dest")"
    cp "$file" "$dest"
    copied=$((copied + 1))
  done < <(find "$source_dir/$game" -type f -print0 | sort -z)
  echo "assets-prives : $game : $copied copiés, $same déjà à jour."
  total_copied=$((total_copied + copied)); total_same=$((total_same + same))
done

commit="$(git -C "$source_dir" rev-parse --short HEAD 2>/dev/null || echo "sans git")"
echo "assets-prives : alacod-assets @ $commit : $total_copied fichiers copiés, $total_same déjà à jour."
if [ "$ci" = 1 ] && [ -n "${GITHUB_ENV:-}" ]; then
  echo "ALACOD_WEB_PRIVATE_ASSETS=1" >> "$GITHUB_ENV"
  echo "assets-prives : ALACOD_WEB_PRIVATE_ASSETS=1 posé pour les étapes suivantes."
else
  echo "assets-prives : pour un build web avec la surcharge : ALACOD_WEB_PRIVATE_ASSETS=1 make build_wasm_apps TARGET=web (sans la variable, rien de la surcharge n'est embarqué)."
fi
