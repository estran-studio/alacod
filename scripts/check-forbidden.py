#!/usr/bin/env python3
"""Contrôle lexical de déterminisme ; les exceptions portent sur une ligne et un compte exacts.

Ce contrôle ne prouve pas le déterminisme : ordre des queries, usages des collections et état
hors rollback restent couverts par revue, synctest, fuzz et tests P2P.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
CRATES = (
    "sim_core", "combat", "game", "map", "map_ldtk", "bevy_fixed", "bots",
    "stats", "effects", "behaviors", "world", "run", "items", "meta",
)
EXCLUDED_CRATES = {"animation", "content", "scenario", "telemetry", "utils"}
PATTERN = re.compile(r"\b(?:HashMap|HashSet)\b|\brand\s*::|\bInstant\s*::\s*now\s*\(")


def findings(root):
    found = Counter()
    locations = {}
    for crate in CRATES:
        if not (root / "crates" / crate / "src").is_dir():
            raise ValueError(f"dossier de simulation absent : {crate}")
    sources = sorted(path / "src" for path in (root / "crates").iterdir()
                     if path.is_dir() and path.name not in EXCLUDED_CRATES)
    for source in sources:
        if not source.is_dir():
            raise ValueError(f"dossier source absent : {source}")
        for path in sorted(source.rglob("*.rs")):
            block_comment = False
            for number, line in enumerate(path.read_text().splitlines(), 1):
                # Les commentaires seuls ne sont pas du code. Les cas complexes restent
                # volontairement visibles au reviewer (ceci n'est pas un parseur Rust).
                stripped = line.strip()
                if block_comment:
                    block_comment = "*/" not in stripped
                    continue
                if stripped.startswith("/*"):
                    block_comment = "*/" not in stripped
                    continue
                if stripped.startswith("//"):
                    continue
                code = stripped.split("//", 1)[0].rstrip()
                if PATTERN.search(code):
                    key = (path.relative_to(root).as_posix(), code)
                    found[key] += 1
                    locations.setdefault(key, []).append(number)
    return found, locations


def load_allowances(path):
    allowed = Counter()
    for entry in json.loads(path.read_text()):
        key = (entry["path"], entry["code"])
        count = entry["count"]
        if key in allowed or type(count) is not int or count < 1 or not entry["reason"].strip():
            raise ValueError(f"exception invalide ou dupliquée : {entry}")
        allowed[key] = count
    return allowed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--strict", action="store_true", help="compatibilité ; toujours bloquant")
    parser.parse_args()
    try:
        found, locations = findings(ROOT)
        allowed = load_allowances(ROOT / "scripts/determinism-allowlist.json")
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1
    failed = False
    for key in sorted(found.keys() | allowed.keys()):
        path, code = key
        if found[key] > allowed[key]:
            failed = True
            lines = ",".join(map(str, locations[key]))
            print(f"ERROR: {path}:{lines}: {code} ({found[key]} occurrence(s), {allowed[key]} autorisée(s))")
        elif found[key] < allowed[key]:
            failed = True
            print(f"ERROR: exception périmée : {path}: {code} ; réduire ou retirer son entrée")
    if failed:
        print("Utiliser une collection ordonnée / le RNG rollback, ou justifier une exception précise.")
        return 1
    print(f"OK: aucun nouveau motif interdit ; {sum(allowed.values())} occurrences auditées dans determinism-allowlist.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
