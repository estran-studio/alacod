#!/usr/bin/env python3
"""Compare deux dumps de trace complets (outil de preuve permanent, T1.2).

Usage:
    scripts/trace-diff.py <a.full> <b.full> [--ignore Nom1,Nom2,...]

Un dump (`ALACOD_DUMP_TRACE=<dossier>`, voir `crates/game/src/state_trace.rs`,
`StateTraceRecorderPlugin::dump`, et `crates/scenario/tests/scenarios.rs`) contient, pour
chaque frame simulée, une ligne d'en-tête :

    <frame> <checksum GGRS 128 bits en hex> <nb d'entités rollback>

puis une ligne par ressource tracée (`<chemin::du::Type>=<Debug>`) et une ligne par entité
rollback triée par GgrsNetId (`GgrsNetId(id, "nom") <chemin::du::Type>=<Debug> ...`, un
tracer par composant présent sur cette entité), toutes les frames séparées par une ligne
vide. Un dump complet (`idle`/`shoot_around`, 1500 frames avec une centaine d'entités) pèse
facilement plusieurs centaines de Mo : ce script lit les deux fichiers en flux, une frame à
la fois des deux côtés, sans jamais les charger entièrement en mémoire.

Ce script retire toujours le checksum de l'en-tête (il change dès qu'un composant rollback
est ajouté ou retiré, ce n'est justement pas ce qu'on veut comparer) et neutralise toute
valeur `Entity` brute embarquée dans un Debug (`{index}v{generation}`, ex. `2659v0` dans
`WeaponInventory.weapons: Vec<(Entity, Weapon)>`) : sa valeur dépend du nombre exact
d'entités déjà créées dans CE process au moment du spawn, non déterministe d'un lancement à
l'autre même à code strictement identique (déjà exclue du `Checksum` GGRS pour cette raison,
voir le `Hash` manuel de `WeaponInventory`). Avec `--ignore`,
retire en plus les paires `<Nom>=<valeur>` dont le dernier segment du nom (après `::`) est
dans la liste — sur les lignes de ressource et d'entité (une ligne d'entité peut porter
plusieurs composants, `--ignore` ne retire que ceux nommés, pas toute la ligne). Un nom
générique (`FrameEvents<run::currency::CurrencyEvent>`, une ressource `FrameEvents<T>`
entière) se filtre par son nom complet tel qu'il apparaît dans le dump : le suffixe `<...>`
est gardé entier (pas coupé à son propre `::` interne), voir `short_name`.

Affiche la première frame qui diffère (numéro, lignes présentes d'un seul côté) et sort
avec le code 1. Code 0 si les deux dumps sont identiques une fois checksums et noms ignorés
retirés.

Exemple (T1.2, avant de blesser les traces : prouver qu'aucune valeur de stat ne diverge
des constantes qu'elle remplace) :

    scripts/trace-diff.py dumps/main/idle.full dumps/branch/idle.full --ignore Stats,Modifiers
"""

import argparse
import itertools
import re
import sys
from pathlib import Path

HEADER_RE = re.compile(r"^(\d+) ([0-9a-f]{32}) (\d+)$")

# `Entity` (bevy_ecs) s'affiche `{index}v{generation}` (ex. `2659v0`) : cette valeur brute
# dépend de l'ordre et du nombre exact de `spawn()` faits dans CE process avant ce point (le
# minutage du chargement des assets, en tâche de fond, peut le faire varier d'un lancement à
# l'autre pour EXACTEMENT le même code : voir le commentaire du `Hash` manuel de
# `game::weapons::WeaponInventory`, qui exclut déjà l'`Entity` du checksum pour cette
# raison). Une comparaison entre deux process (deux binaires, ou même le même binaire
# relancé) doit donc toujours neutraliser ces valeurs, pas seulement filtrer par
# `--ignore` : elles ne sont pas spécifiques à un composant, elles apparaissent nichées
# dans le Debug de n'importe quel composant qui porte une `Entity` (ex.
# `WeaponInventory.weapons: Vec<(Entity, Weapon)>`).
ENTITY_RE = re.compile(r"\b\d+v\d+\b")

# Une clé de tracer est un chemin de type Rust (identifiants séparés par `::`), jamais
# d'espace ni de `=` : sert à retrouver le début de chaque paire "clé=valeur" sur une ligne
# qui peut en contenir plusieurs (une ligne d'entité empile un tracer par composant
# présent). Le contenu d'une valeur (Debug d'un composant quelconque) peut lui contenir des
# espaces, des deux-points, des virgules... jamais un `=` suivi de ce motif de clé, en
# pratique (Debug dérivé utilise `champ: valeur`, jamais `champ=valeur`).
#
# Suffixe générique optionnel (`<...>`, T2.3 : `FrameEvents<run::currency::CurrencyEvent>`,
# le nom de type que `std::any::type_name` produit pour toute ressource `FrameEvents<T>`,
# T1.1+) : un seul niveau (pas de `<`/`>`/`=`/espace à l'intérieur), suffisant pour tous les
# types réellement tracés par `StateTracers` aujourd'hui (leur paramètre générique est
# toujours un type non générique). `short_name` en dessous garde ce suffixe entier plutôt que
# de le couper au `::` qu'il contient.
KEY_RE = re.compile(
    r"(?:(?<=\s)|^)([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*(?:<[^<>=\s]*>)?)="
)


def short_name(key: str) -> str:
    """Dernier segment du nom pour le filtrage `--ignore` : le suffixe générique (s'il y en
    a un) est gardé entier plutôt que coupé au `::` qu'il contient lui-même, pour que
    `--ignore FrameEvents<run::currency::CurrencyEvent>` matche exactement le nom tel qu'il
    apparaît dans le dump (voir la doc de `KEY_RE`)."""
    if "<" in key:
        head, _, rest = key.partition("<")
        return head.rsplit("::", 1)[-1] + "<" + rest
    return key.rsplit("::", 1)[-1]

# Combien de frames tenir en mémoire des deux côtés pour donner du contexte autour d'une
# différence trouvée en flux (voir `iter_frames`/la boucle de comparaison dans `main`).
CONTEXT_FRAMES = 1


def strip_ignored(line: str, ignored: set) -> str:
    """Retire les paires `clé=valeur` dont le dernier segment de la clé est dans `ignored`.
    Le préfixe avant la première clé (rien pour une ressource, l'entité `GgrsNetId(...)`
    pour une ligne de composants) est toujours conservé."""
    matches = list(KEY_RE.finditer(line))
    if not matches:
        return line

    prefix = line[: matches[0].start()].rstrip()
    kept = []
    for i, m in enumerate(matches):
        key = m.group(1)
        short = short_name(key)
        if short in ignored:
            continue
        end = matches[i + 1].start() if i + 1 < len(matches) else len(line)
        kept.append(line[m.start() : end].rstrip())

    if not kept:
        return prefix
    if prefix:
        return prefix + " " + " ".join(kept)
    return " ".join(kept)


def iter_frames(path: Path, ignored: set):
    """Générateur de `(numéro_de_frame, [lignes nettoyées])`, une frame à la fois : ne
    charge jamais tout le fichier en mémoire (voir la doc du module)."""
    current_frame = None
    current_lines = []
    with path.open("r") as f:
        for raw in f:
            line = raw.rstrip("\n")
            header = HEADER_RE.match(line)
            if header:
                if current_frame is not None:
                    yield current_frame, current_lines
                current_frame = int(header.group(1))
                entity_count = header.group(3)
                current_lines = [f"{current_frame} <checksum> {entity_count}"]
                continue
            if not line.strip():
                continue
            if current_frame is None:
                continue  # contenu avant la première ligne d'en-tête : ignoré
            cleaned = strip_ignored(ENTITY_RE.sub("<entity>", line), ignored)
            if cleaned.strip():
                current_lines.append(cleaned)
    if current_frame is not None:
        yield current_frame, current_lines


def report_diff(label_a: str, label_b: str, lines_a: list, lines_b: list, limit: int = 20):
    set_a, set_b = set(lines_a), set(lines_b)
    only_a = [l for l in lines_a if l not in set_b]
    only_b = [l for l in lines_b if l not in set_a]
    print(f"  seulement dans {label_a} :")
    for l in only_a[:limit]:
        print(f"    {l}")
    if len(only_a) > limit:
        print(f"    ... ({len(only_a) - limit} de plus)")
    print(f"  seulement dans {label_b} :")
    for l in only_b[:limit]:
        print(f"    {l}")
    if len(only_b) > limit:
        print(f"    ... ({len(only_b) - limit} de plus)")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("a", type=Path, help="premier dump (.full)")
    parser.add_argument("b", type=Path, help="second dump (.full)")
    parser.add_argument(
        "--ignore",
        default="",
        help="noms de composants/ressources à ignorer, séparés par des virgules (ex. Stats,Modifiers)",
    )
    args = parser.parse_args()

    for path in (args.a, args.b):
        if not path.is_file():
            print(f"erreur : {path} n'existe pas ou n'est pas un fichier", file=sys.stderr)
            return 2

    ignored = {name.strip() for name in args.ignore.split(",") if name.strip()}

    sentinel = object()
    count = 0
    for item_a, item_b in itertools.zip_longest(
        iter_frames(args.a, ignored), iter_frames(args.b, ignored), fillvalue=sentinel
    ):
        if item_a is sentinel or item_b is sentinel:
            longer, shorter, extra = (
                (args.a, args.b, item_a) if item_b is sentinel else (args.b, args.a, item_b)
            )
            frame, _ = extra
            print(f"{longer} a plus de frames que {shorter} (à partir de la frame {frame})")
            return 1

        frame_a, lines_a = item_a
        frame_b, lines_b = item_b
        if frame_a != frame_b:
            print(
                f"désalignement de frame : {args.a} en est à {frame_a}, {args.b} à {frame_b} "
                f"(après {count} frames comparées)"
            )
            return 1

        count += 1
        if lines_a != lines_b:
            print(f"première frame qui diffère : {frame_a}")
            report_diff(str(args.a), str(args.b), lines_a, lines_b)
            return 1

    if count == 0:
        print("aucune frame trouvée dans l'un des deux dumps", file=sys.stderr)
        return 2

    ignored_desc = ", ".join(sorted(ignored)) if ignored else "rien"
    print(f"identique ({count} frames comparées, ignoré : {ignored_desc})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
