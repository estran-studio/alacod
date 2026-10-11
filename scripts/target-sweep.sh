#!/usr/bin/env bash
# Purge du target Cargo (tâche outillage-build, décision du 2026-10-09 : disque partagé par
# plusieurs worktrees de tâche, 35 à 60 Go chacun).
#
#   scripts/target-sweep.sh [--dry-run] [target-dir]
#
# - supprime `incremental/` de tous les profils (inutile avec CARGO_INCREMENTAL=0, 1 à 3 Go sinon) ;
# - dans `<profil>/build/<crate>/` des **crates du workspace** (crates/*, games/*, alacod), ne garde
#   que la dernière génération de chaque artefact : un dossier `<hash>` est périmé si un autre
#   dossier du même crate produit les mêmes artefacts (mêmes noms une fois le suffixe `-<hash>`
#   retiré : `scenarios`, `libgame`, `alacod_sim`…) et a été écrit plus récemment. Les binaires de
#   test (1 Go chacun en statique, 0,3 Go en dynamique) sont ainsi dédoublonnés, une version
#   statique ET une version dynamique du même test ne coexistent pas.
# - ne touche jamais aux crates tierces (plusieurs versions légitimes d'un même crate) ni aux
#   scripts de build (`run/`).
# À lancer entre deux commandes cargo, jamais pendant : un build en cours perdrait ses fichiers
# (c'est exactement ce qui est arrivé en purgeant à la main pendant une suite de tests).
set -euo pipefail

dry=0
if [ "${1:-}" = "--dry-run" ]; then dry=1; shift; fi
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-${CARGO_TARGET_DIR:-$root/target}}"
[ -d "$target" ] || { echo "target-sweep : $target n'existe pas" >&2; exit 1; }

before=$(du -sk "$target" | cut -f1)
python3 - "$root" "$target" "$dry" <<'PY'
import os, re, shutil, sys, glob

root, target, dry = sys.argv[1], sys.argv[2], sys.argv[3] == "1"
members = {"alacod"}
for pattern in ("crates/*", "games/*"):
    for path in glob.glob(os.path.join(root, pattern)):
        if os.path.isfile(os.path.join(path, "Cargo.toml")):
            members.add(os.path.basename(path))

suffix = re.compile(r"-[0-9a-f]{16}(?=\.|$)")
freed = 0

def size(path):
    total = 0
    for base, _, files in os.walk(path):
        for name in files:
            try:
                total += os.lstat(os.path.join(base, name)).st_size
            except OSError:
                pass
    return total

def remove(path):
    global freed
    freed += size(path)
    print(("à supprimer : " if dry else "supprimé : ") + os.path.relpath(path, target))
    if not dry:
        shutil.rmtree(path, ignore_errors=True)

for profile in sorted(os.listdir(target)):
    pdir = os.path.join(target, profile)
    if not os.path.isdir(pdir):
        continue
    inc = os.path.join(pdir, "incremental")
    if os.path.isdir(inc):
        remove(inc)
    build = os.path.join(pdir, "build")
    if not os.path.isdir(build):
        continue
    for crate in sorted(members):
        cdir = os.path.join(build, crate.replace("-", "_"))
        if not os.path.isdir(cdir):
            cdir = os.path.join(build, crate)
        if not os.path.isdir(cdir):
            continue
        groups = {}
        for h in os.listdir(cdir):
            out = os.path.join(cdir, h, "out")
            if not os.path.isdir(out):
                continue
            names = sorted(suffix.sub("", n) for n in os.listdir(out) if suffix.search(n))
            if not names:
                continue
            mtime = max(os.lstat(os.path.join(out, n)).st_mtime for n in os.listdir(out))
            groups.setdefault(tuple(names), []).append((mtime, h))
        for names, gens in groups.items():
            gens.sort(reverse=True)
            for _, h in gens[1:]:
                remove(os.path.join(cdir, h))

print(f"{'libérerait' if dry else 'libéré'} : {freed / 1e9:.1f} Go")
PY
after=$(du -sk "$target" | cut -f1)
echo "target : $((before / 1048576)) Go -> $((after / 1048576)) Go ($target)"
