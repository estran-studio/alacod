#!/usr/bin/env python3
"""Sonde de la frontière simulation/présentation de `crates/game` (D69, étape 2).

    scripts/probe-game-sim.py [--sim-list scripts/d69-sim-files.txt] [--tests] [--json sortie.json] [--verbose]

Copie l'arbre de travail (fichiers suivis et non ignorés) dans un dossier temporaire, y réécrit
`crates/game/Cargo.toml` pour que `game` ne voie que bevy en ECS seul (`default-features = false`
+ `serialize`, `bevy_color`, `bevy_state`) et aucune des dépendances de présentation ou d'E/S
(`bevy_kira_audio`, `bevy_matchbox`, `leafwing-input-manager`, `bevy_common_assets`, `bevy_hui`,
`bevy_light_2d`, `bevy-inspector-egui`, `animation`, `telemetry`, `harmonium_bevy`, `tracing-web`),
puis lance `cargo check -p game --no-default-features` et range les erreurs de résolution par
fichier. Un fichier **sans erreur** compile en ECS seul : c'est un fichier de simulation ; un fichier
avec erreurs porte au moins une fuite (type de présentation, `Assets<…>`, crate d'E/S).

Le dépôt n'est jamais modifié. Le crate n'est pas encore découpé : tant que `game` est un seul
crate, cette sonde est la mesure de la dette (nombre de fichiers et d'erreurs) ; elle disparaît
quand `game_sim` porte seul la frontière (`scripts/check-sim-boundary.sh`).

Limite : rustc s'arrête après la résolution des noms tant qu'il y en a d'échouées. Les erreurs de
typage cachées derrière (méthode d'un type de présentation, trait absent) n'apparaissent qu'une
fois les noms résolus : relancer la sonde après chaque correction.

`--sim-list` : fichiers (chemins relatifs à `crates/game/src`, un par ligne, `#` commente) qui
doivent rester sans erreur **et** ne dépendre (`use crate::…`) que de fichiers de la liste ; sortie 1
sinon (cliquet : chaque étape y ajoute ses fichiers).
"""
import argparse
import collections
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GAME_SRC = "crates/game/src/"

DROP = (
    "bevy_kira_audio",
    "bevy_matchbox",
    "bevy_light_2d",
    "bevy_hui",
    "leafwing-input-manager",
    "bevy_common_assets",
    "animation",
    "telemetry",
    "harmonium_bevy",
    "bevy-inspector-egui",
    "tracing-web",
)
BEVY_ECS = 'bevy = { version = "0.19.1", default-features = false, features = ["serialize", "bevy_color", "bevy_state"] }'

CATEGORIES = [
    ("asset", r"Assets|Handle|AssetServer|AssetEvent|AssetPlugin|derive macro `Asset`|bevy_common_assets|`asset`|`image`"),
    ("entrée", r"leafwing|ActionState|InputMap|Actionlike|actionlike|KeyCode|Gamepad|ButtonInput|MouseButton"),
    ("sprite/caméra", r"Sprite|Visibility|TextureAtlas|Image|Camera|camera|Projection|Mesh|Light"),
    ("ui", r"bevy::ui|bevy::text|Node|Text|Val\b|BackgroundColor|Interaction|FontSize|Font\b|TextFont|TextColor|Button|PositionType|UiRect|BorderColor|bevy_hui|Display|Justify|FlexDirection|UiScale"),
    ("réseau", r"matchbox"),
    ("audio", r"kira|Audio"),
    ("fenêtre", r"Window|window|winit"),
    ("gizmos", r"Gizmos"),
    ("crate d'E/S", r"animation|telemetry|tracing_web"),
    ("rendu/remote", r"render|remote"),
]


def category(message: str) -> str:
    for name, pattern in CATEGORIES:
        if re.search(pattern, message):
            return name
    return "autre"


def copy_tree(dest: Path) -> None:
    listing = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=ROOT, check=True, capture_output=True,
    ).stdout.decode().split("\0")
    for name in filter(None, listing):
        source = ROOT / name
        if not source.is_file():
            continue
        target = dest / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)


def rewrite_cargo(manifest: Path) -> None:
    out = []
    section = ""
    for line in manifest.read_text().splitlines():
        header = re.match(r"^\[(.+)\]\s*$", line)
        if header:
            section = header.group(1)
            if section == "features":
                out += [line, "dev-dylib = []", "ui_hui = []", "lighting = []", "debug_ui = []", "harmonium = []"]
                continue
        if section == "features":
            continue
        key = re.match(r"^([A-Za-z0-9_-]+)\s*=", line)
        if key and key.group(1) in DROP:
            continue
        if key and key.group(1) == "bevy":
            line = BEVY_ECS
        out.append(line)
    manifest.write_text("\n".join(out) + "\n")


# --- fermeture des dépendances internes -------------------------------------------------------
# Un fichier sans fuite directe peut dépendre (`use crate::…`) d'un fichier qui fuit : il ne
# pourrait alors pas passer dans `game_sim` sans lui. `--sim-list` vérifie donc aussi que chaque
# fichier de la liste ne référence que des fichiers de la liste (résolution au fichier le plus long
# qui porte le chemin de module ; les `pub use` d'un mod.rs comptent sur ce mod.rs).

def module_files() -> dict:
    files = {}
    for path in (ROOT / GAME_SRC).rglob("*.rs"):
        rel = path.relative_to(ROOT / GAME_SRC)
        parts = list(rel.with_suffix("").parts)
        if parts[-1] == "mod":
            parts = parts[:-1]
        if parts == ["lib"]:
            parts = []
        files[tuple(parts)] = str(rel)
    return files


def expand_use(text: str) -> list:
    text = re.sub(r"\s+", "", text)

    def rec(prefix, chunk):
        out, parts, depth, cur = [], [], 0, ""
        for ch in chunk:
            depth += ch == "{"
            depth -= ch == "}"
            if ch == "," and depth == 0:
                parts.append(cur)
                cur = ""
            else:
                cur += ch
        if cur:
            parts.append(cur)
        for part in parts:
            if "{" in part:
                head = part[: part.index("{")].rstrip(":")
                inner = part[part.index("{") + 1 : part.rindex("}")]
                out += rec(prefix + [x for x in head.split("::") if x], inner)
            else:
                out.append(tuple(prefix + [x for x in part.split("::") if x]))
        return out

    return rec([], text)


def internal_deps(rel: str, modules: dict) -> set:
    here = tuple(Path(rel).with_suffix("").parts)
    if here and here[-1] == "mod":
        here = here[:-1]
    src = (ROOT / GAME_SRC / rel).read_text()
    cut = re.search(r"#\[cfg\(test\)\]\s*\n\s*mod\s+\w+", src)
    src = re.sub(r"//[^\n]*", "", src[: cut.start()] if cut else src)
    paths = []
    for m in re.finditer(r"\buse\s+((?:crate|super|self)::[^;]*);", src, re.S):
        paths += expand_use(m.group(1))
    paths += [tuple(m.group(1).split("::")) for m in re.finditer(r"(?<![\w:])((?:crate|super)(?:::\w+)+)", src)]
    deps = set()
    for path in paths:
        parts = list(path)
        if parts[0] == "crate":
            parts = parts[1:]
        elif parts[0] == "self":
            parts = list(here) + parts[1:]
        else:
            base = list(here)
            while parts and parts[0] == "super":
                base, parts = base[:-1], parts[1:]
            parts = base + parts
        for k in range(len(parts), -1, -1):
            if tuple(parts[:k]) in modules:
                target = modules[tuple(parts[:k])]
                if target != rel:
                    deps.add(target)
                break
    return deps


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--sim-list", help="fichiers qui doivent rester sans erreur")
    parser.add_argument("--json", help="écrit le détail (fichier -> erreurs) dans ce fichier")
    parser.add_argument("--verbose", action="store_true", help="une ligne par erreur")
    parser.add_argument("--tests", action="store_true",
                        help="compile aussi les tests du crate (`cargo check --tests`) : ceux d'un fichier de simulation ne doivent pas non plus tirer de présentation")
    args = parser.parse_args()

    with tempfile.TemporaryDirectory(prefix="alacod-probe-game-") as tmp:
        work = Path(tmp)
        copy_tree(work)
        rewrite_cargo(work / "crates/game/Cargo.toml")
        proc = subprocess.run(
            ["cargo", "check", "-p", "game", "--no-default-features", "--profile", "headless",
             "--message-format=json"] + (["--tests"] if args.tests else []),
            cwd=work, capture_output=True, text=True,
        )

    errors = collections.defaultdict(list)
    for line in proc.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get("reason") != "compiler-message" or message["message"]["level"] != "error":
            continue
        primary = [s for s in message["message"]["spans"] if s["is_primary"]]
        if not primary:
            continue
        span = primary[0]
        if not span["file_name"].startswith(GAME_SRC):
            continue
        errors[span["file_name"][len(GAME_SRC):]].append(
            (span["line_start"], category(message["message"]["message"]), message["message"]["message"])
        )
    if not errors and proc.returncode != 0:
        print(proc.stderr[-3000:], file=sys.stderr)
        print("sonde : cargo a échoué sans erreur attribuable à crates/game (voir ci-dessus)", file=sys.stderr)
        return 2

    all_files = sorted(
        str(p.relative_to(ROOT / GAME_SRC)) for p in (ROOT / GAME_SRC).rglob("*.rs")
    )
    leaking = sorted(errors, key=lambda f: (-len(errors[f]), f))
    print(f"sonde game en ECS seul : {len(leaking)} fichiers avec fuite sur {len(all_files)}, "
          f"{sum(len(v) for v in errors.values())} erreurs de résolution")
    totals = collections.Counter(c for v in errors.values() for _, c, _ in v)
    print("par catégorie : " + ", ".join(f"{k} {v}" for k, v in totals.most_common()))
    for name in leaking:
        cats = collections.Counter(c for _, c, _ in errors[name])
        print(f"  {name:48} {len(errors[name]):4}  " + ", ".join(f"{k} {v}" for k, v in cats.most_common(4)))
        if args.verbose:
            for lineno, cat, text in sorted(errors[name]):
                print(f"      {lineno:5} [{cat}] {text}")
    if args.json:
        Path(args.json).write_text(json.dumps(
            {f: [{"line": l, "category": c, "message": m} for l, c, m in errors[f]] for f in leaking},
            indent=1, ensure_ascii=False))

    status = 0
    if args.sim_list:
        wanted = [
            l.strip() for l in Path(args.sim_list).read_text().splitlines()
            if l.strip() and not l.lstrip().startswith("#")
        ]
        unknown = [f for f in wanted if f not in all_files]
        broken = [f for f in wanted if f in errors]
        modules = module_files()
        listed = set(wanted)
        outside = {}
        for f in wanted:
            if f in unknown:
                continue
            for dep in sorted(internal_deps(f, modules) - listed):
                outside.setdefault(f, []).append(dep)
        for f, deps in sorted(outside.items()):
            print(f"ERREUR : {f} dépend de fichiers hors liste : {', '.join(deps)}", file=sys.stderr)
        for f in unknown:
            print(f"ERREUR : {f} est dans {args.sim_list} mais n'existe plus dans crates/game/src", file=sys.stderr)
        for f in broken:
            print(f"ERREUR : {f} doit rester sans fuite ({len(errors[f])} erreurs, p. ex. "
                  f"ligne {sorted(errors[f])[0][0]})", file=sys.stderr)
        if unknown or broken or outside:
            status = 1
        else:
            print(f"OK : les {len(wanted)} fichiers de {args.sim_list} compilent en ECS seul et n'en appellent aucun autre")
    return status


if __name__ == "__main__":
    sys.exit(main())
