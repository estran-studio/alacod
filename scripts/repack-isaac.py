#!/usr/bin/env python3
"""Surcharge locale d'isaac (S2-4, docs/conventions.md § 106) : packs payants **non redistribuables**, jamais commités.

Remplace les tuiles Kenney de gungeon (personnage, ennemis, boss, cibles) par les packs déjà achetés :
  - Aymen B « UPDATE 2_ZombiePack » (id `aymen-b-zombiepack` d'`assets.yaml`) : héros et ennemis en 32 × 32, six peaux ;
  - Ittai Manero « Zombie Apocalypse Tileset » (id `ittai-zombie-apocalypse`) : tourelle, mouche (oiseau), enfant, maigre,
    colosse (gros zombie), assis, épouvantail, canette.
Même mécanique que scripts/repack-throne.py (importé : `Pack`, `strip`, `union_bbox`, `write_sheet_ron`, `write_animation_ron`,
`entry_shadowed`, `SKINS`) ; aucun pixel n'est dessiné : découpe, collage, mise à l'échelle entière (§ 92, § 96).

Écrit, sous `games/isaac/assets/` (gitignoré : `sprites/postapo/`, `sprites/sprites.local.ron`) :
  - `sprites/postapo/char_*` : feuilles et animations (RON) ;
  - `sprites/sprites.local.ron` : une entrée par `asset_name_ref` d'un personnage d'isaac, qui remplace celle de `sprites.ron`.
Le sol, le halo des ramassables et le pointeur de visée ne sont PAS ici : ils sont publics (scripts/generer-sprites-isaac.py).

Usage : scripts/repack-isaac.py [ZIP_AYMEN_B] [ZIP_ITTAI] [--preview apercu.png]
Défauts : ~/Downloads/UPDATE 2_ZombiePack.zip et ~/Downloads/Zombie Apocalypse Tileset.zip.
"""
import importlib.util
import io
import sys
import zipfile
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("repack_throne", HERE / "repack-throne.py")
rt = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(rt)

ROOT = HERE.parent
ASSETS = ROOT / "games/isaac/assets"
OUT = ASSETS / "sprites/postapo"
rt.ASSETS, rt.OUT = ASSETS, OUT
Pack, blank, strip, write_sheet_ron, write_animation_ron, entry_shadowed = (
    rt.Pack, rt.blank, rt.strip, rt.write_sheet_ron, rt.write_animation_ron, rt.entry_shadowed)

# --- Aymen B : id de personnage d'isaac -> (peau, échelle de présentation) ---------------------------
# L'échelle de collision vient de `characters/*.ron`, jamais de l'image ; entre deux ennemis qui partagent une peau,
# l'échelle (et la peau elle-même) est la seule différence : la légende (F7) dit laquelle est laquelle.
AYMEN = {
    "player": ("player", 1.0),
    "suiveur": ("zombie2", 1.0),
    "rampant": ("zombie1", 1.0),
    "cracheur": ("zombie3", 1.0),
    "bourdon": ("zombie3", 0.75),
    "spectre": ("bandit", 1.0),
    "gardien": ("zombie4", 2.2),
}

# --- Ittai : id -> (dossier, images Idle (début, fin), images Run (début, fin), échelle, alignement) -----
# `bottom` : pieds alignés (marcheurs) ; `center` : centrée (oiseau). Aucune animation de mort dans ce pack.
ITTAI = {
    "araignee": ("Kid Zombie Animation Frames/", (0, 0), (0, 8), 2.5, "bottom"),
    "mouche": ("White Bird Flying, Ground Eating + Being Shot Blood Animation Frames/", (0, 2), (0, 2), 3.0, "center"),
    "follet": ("White Bird Flying, Ground Eating + Being Shot Blood Animation Frames/", (0, 2), (0, 2), 4.5, "center"),
    "arroseur": ("Sitting Zombie/", (0, 1), (0, 1), 3.0, "bottom"),
    "tourelle_croix": ("Turret Zombie Animation Frames/", (0, 11), (0, 11), 2.0, "bottom"),
    "colosse": ("Big Zombie Walking Animation Frames/", (0, 0), (0, 8), 3.5, "bottom"),
    "cible_mordante": ("Skinny Walking Zombie Animation/", (0, 0), (0, 8), 2.5, "bottom"),
    "cible_fragile": ("Scarecrow/", (0, 0), (0, 0), 2.5, "bottom"),
    "cible": ("Shootable Coke Can Animation Frames/", (0, 0), (0, 0), 2.5, "bottom"),
}


def aymen_skins(aymen: Pack, used: set[str]) -> dict[str, tuple[int, int]]:
    """Une feuille par peau (Idle 6 | Run 8 | Dying_1 8, cellule recadrée à la boîte commune) ; retourne la taille de cellule."""
    sizes = {}
    for skin, (idle, run, dead) in rt.SKINS.items():
        if skin not in used:
            continue
        frames = strip(aymen.load(idle), 6) + strip(aymen.load(run), 8) + strip(aymen.load(dead), 8)
        x0, y0, x1, y1 = rt.union_bbox(frames)
        w, h = x1 - x0, y1 - y0
        w += w % 2
        sheet = blank(w * len(frames), h)
        for i, f in enumerate(frames):
            sheet.alpha_composite(f.crop((x0, y0, x0 + w, y1)), (i * w, 0))
        sheet.save(OUT / f"char_{skin}.png")
        write_animation_ron(f"char_{skin}", {"Idle": (0, 5), "Run": (6, 13), "Dying_1": (14, 21)})
        sizes[skin] = (w, h)
    return sizes


def ittai_frames(ittai: Pack, folder: str, lo: int, hi: int, align: str) -> Image.Image:
    """Bande horizontale des images `lo..hi` du dossier, cellules de la boîte commune (pieds ou centre alignés)."""
    ims = ittai.folder(folder)[lo:hi + 1]
    boxes = [im.crop(im.getbbox()) for im in ims]
    w, h = max(b.width for b in boxes), max(b.height for b in boxes)
    w += w % 2
    sheet = blank(w * len(boxes), h)
    for i, b in enumerate(boxes):
        x = i * w + (w - b.width) // 2
        y = h - b.height if align == "bottom" else (h - b.height) // 2
        sheet.alpha_composite(b, (x, y))
    return sheet


def ittai_character(ittai: Pack, cid: str) -> str:
    folder, idle, run, scale, align = ITTAI[cid]
    lo, hi = min(idle[0], run[0]), max(idle[1], run[1])
    strip_ = ittai_frames(ittai, folder, lo, hi, align)
    n = hi - lo + 1
    w = strip_.width // n
    strip_.save(OUT / f"char_{cid}.png")
    write_animation_ron(f"char_{cid}", {"Idle": (idle[0] - lo, idle[1] - lo), "Run": (run[0] - lo, run[1] - lo)})
    (OUT / f"char_{cid}_sheet.ron").write_text(
        write_sheet_ron(f"char_{cid}", (w, strip_.height), n, 1, scale=scale, animated=True))
    return entry_shadowed(cid, f"char_{cid}", f"char_{cid}")


def preview(path: Path, aymen: Pack, ittai: Pack) -> None:
    """Planche de contrôle (hors moteur) : première image Idle de chaque personnage, à l'échelle de présentation."""
    cells = []
    for cid, (skin, scale) in AYMEN.items():
        idle = rt.SKINS[skin][0]
        f = strip(aymen.load(idle), 6)[0]
        cells.append((cid, f.crop(f.getbbox()), scale))
    for cid, (folder, idle, run, scale, align) in ITTAI.items():
        f = ittai.folder(folder)[idle[0]]
        cells.append((cid, f.crop(f.getbbox()), scale))
    cell = 150
    sheet = Image.new("RGBA", (cell * 7, cell * ((len(cells) + 6) // 7)), (123, 106, 95, 255))
    for i, (cid, im, scale) in enumerate(cells):
        k = max(1, round(scale * 2))
        im = im.resize((im.width * k, im.height * k), Image.NEAREST)
        sheet.alpha_composite(im, ((i % 7) * cell + (cell - im.width) // 2, (i // 7) * cell + cell - im.height - 6))
    sheet.save(path)


def main() -> None:
    args = [a for a in sys.argv[1:]]
    prev = None
    if "--preview" in args:
        i = args.index("--preview")
        prev = Path(args[i + 1])
        del args[i:i + 2]
    home = Path.home() / "Downloads"
    aymen_zip = Path(args[0]) if len(args) > 0 else home / "UPDATE 2_ZombiePack.zip"
    ittai_zip = Path(args[1]) if len(args) > 1 else home / "Zombie Apocalypse Tileset.zip"
    aymen = Pack(aymen_zip, rt.AYMEN_ROOT)
    ittai = Pack(ittai_zip, rt.ITTAI_ROOT)
    OUT.mkdir(parents=True, exist_ok=True)
    lines = []
    used = {skin for skin, _ in AYMEN.values()}
    sizes = aymen_skins(aymen, used)
    for cid, (skin, scale) in AYMEN.items():
        w, h = sizes[skin]
        (OUT / f"char_{cid}_sheet.ron").write_text(
            write_sheet_ron(f"char_{cid}", (w, h), 22, 1, scale=scale, animated=True, png=f"char_{skin}"))
        lines.append(entry_shadowed(cid, f"char_{skin}", f"char_{cid}"))
    for cid in ITTAI:
        lines.append(ittai_character(ittai, cid))
    (ASSETS / "sprites/sprites.local.ron").write_text(
        "// Surcharge locale (packs Aymen B et Ittai Manero, NON redistribuables) : écrit par\n"
        "// scripts/repack-isaac.py, gitignoré. Remplace les entrées de personnages de sprites.ron.\n"
        "({\n" + "\n".join(lines) + "\n})\n")
    if prev:
        preview(prev, aymen, ittai)
    print(f"{len(lines)} entrées, {len(list(OUT.glob('*.png')))} images dans {OUT}")


if __name__ == "__main__":
    main()
