#!/usr/bin/env bash
# Preuve « l'ordre de la simulation n'a pas bougé » (D69 étape 2, `docs/taches/dette-d69-etape2.md`).
#
#   scripts/schedule-order-diff.sh [REF_DE_BASE]      (défaut : origin/main)
#
# Pour la référence de base (un worktree git jetable) puis pour l'arbre de travail, construit l'app
# headless d'un scénario par jeu sans jouer de frame (`crates/scenario/tests/schedule_order.rs`) et
# écrit l'ordre exact des systèmes de `ReadInputs`, `GgrsSchedule`, `AdvanceWorld`, `SaveWorld`,
# `LoadWorld` et l'ordre d'enregistrement des types rollback. Noms normalisés (sans chemin de
# module) : déplacer un module d'un crate à l'autre ne change rien ; un système déplacé dans
# l'ordre, ajouté, retiré, ou un type rollback réordonné, oui. Sortie 0 = fichiers identiques.
#
# Si la base n'a pas encore l'outil, le fichier du test est copié depuis l'arbre de travail.
# `ALACOD_SCHEDULE_OUT=<dossier>` garde les deux sorties (`base/`, `head/`).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
base_ref="${1:-origin/main}"
out="${ALACOD_SCHEDULE_OUT:-$(mktemp -d)}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"  # absolu : chaque checkout écrit depuis son propre répertoire
tests_file="crates/scenario/tests/schedule_order.rs"

dump() { # <dossier du dépôt> <dossier de sortie>
    # features du checkout lui-même (E9 transfère `dev-dylib` : la base garde les siennes)
    (cd "$1" && APP_VERSION=v0.0.0 ALACOD_SCHEDULE_DUMP="$2" \
        cargo test -q -p scenario --profile headless $(./scripts/dev-features.sh features) \
        --test schedule_order -- --include-ignored >/dev/null)
}

worktree="$(mktemp -d)/base"
git worktree add --detach --quiet "$worktree" "$base_ref"
trap 'git worktree remove --force "$worktree" >/dev/null 2>&1 || true' EXIT
[ -f "$worktree/$tests_file" ] || cp "$tests_file" "$worktree/$tests_file"

echo "ordre des systèmes : base $base_ref"
dump "$worktree" "$out/base"
echo "ordre des systèmes : arbre de travail"
dump "$root" "$out/head"

if diff -r "$out/base" "$out/head"; then
    echo "OK : ordre des systèmes et des types rollback identiques ($(cat "$out"/head/*.schedules.txt | wc -l) lignes de schedules, $(cat "$out"/head/*.rollback.txt | wc -l) types), sorties dans $out"
else
    echo "ERREUR : l'ordre des systèmes ou des types rollback a changé (voir le diff ci-dessus ; $out)" >&2
    exit 1
fi
