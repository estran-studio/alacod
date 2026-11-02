#!/usr/bin/env python3
"""Images publiques de throne (voie t, polissage-throne-vitrine) : CC0 et CC BY, **commitées**.

Sources lues dans `~/orca/workspaces/alacod/packs/` (téléchargées d'OpenGameArt, licences relues sur la page de
chaque contribution, déclarées dans `games/throne/assets/assets.yaml` : `knekko-futuristic-guns`, `letargicdev-portals`,
`redshrike-rat-king`) :
  - `oga-knekko/*.png` (CC0) : 13 armes, une image par arme (au sol `weapon/<id>`, portée `held/<id>`) ;
  - `oga-portails/portalRings2.png` (CC0) : portail de l'étage (`portal`) ;
  - `oga-redshrike/rats.png` (CC BY 4.0, « Art by Redshrike, for OpenGameArt.org ») : roi rat (`roi_rat`).
Écrit sous `games/throne/assets/sprites/vitrine/` et affiche les entrées à reporter dans `sprites/sprites.ron`.
Aucun pixel dessiné : découpe, réduction par moyenne (BOX) puis alpha binaire, collage, retrait du fond vert de la feuille du rat.

Usage : scripts/repack-throne-cc0.py [DOSSIER_DES_PACKS]   (défaut : ~/orca/workspaces/alacod/packs)
"""
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "games/throne/assets/sprites/vitrine"
PACKS = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / "orca/workspaces/alacod/packs"

# id d'arme de throne -> (feuille de Knekko, case grossière (x0, y0, x1, y1), longueur voulue en pixels après réduction)
GUNS = {
    "revolver": ("pistols", (0, 0, 37, 31), 14),
    "mitraillette": ("machine_guns", (0, 0, 69, 37), 24),
    "fusil_a_pompe": ("shotguns_3", (0, 0, 101, 32), 28),
    "canon_ricochet": ("sniper_0", (0, 0, 131, 28), 34),
    "lance_grenades": ("neonpunk", (43, 0, 127, 36), 26),
    "roquette": ("military", (103, 66, 205, 100), 30),
    "mortier": ("military", (103, 36, 205, 66), 34),
    "laser": ("military", (0, 34, 103, 68), 30),
    "plasma": ("military", (0, 68, 103, 103), 30),
    "traqueur": ("neonpunk", (43, 34, 80, 60), 14),
    "lance_lames": ("melee_0", (0, 70, 42, 88), 16),
    "disque": ("neonpunk", (84, 28, 153, 58), 24),
    "arsenal": ("military", (0, 0, 103, 34), 32),
}


def trimmed(im: Image.Image, box: tuple[int, int, int, int]) -> Image.Image:
    part = im.crop(box)
    return part.crop(part.getbbox())


def shrink(im: Image.Image, length: int) -> Image.Image:
    """Réduit par moyenne (BOX) à `length` pixels de long, puis alpha binaire (pixels nets)."""
    if im.width <= length:
        return im
    k = length / im.width
    out = im.resize((length, max(1, round(im.height * k))), Image.BOX)
    px = out.load()
    for y in range(out.height):
        for x in range(out.width):
            r, g, b, a = px[x, y]
            px[x, y] = (r, g, b, 255) if a >= 110 else (0, 0, 0, 0)
    return out.crop(out.getbbox())


def sheet_ron(path: str, tile: tuple[int, int], rows: int, *, scale: float, anchor: str, z: float,
              animated: bool = False, columns: int = 1) -> str:
    return f"""(
    path: "sprites/vitrine/{path}",
    tile_size: ({tile[0]}, {tile[1]}),
    columns: {columns},
    rows: {rows},

    animated: {str(animated).lower()},
    name: "body",

    scale: {scale},

    anchor: {anchor},

    offset_x: 0.0,
    offset_y: 0.0,
    offset_z: {z},

)
"""


def animation_ron(anims: dict[str, tuple[int, int]], frame_duration: int = 120) -> str:
    body = "\n".join(f'        "{k}": (start: {a}, end: {b}),' for k, (a, b) in anims.items())
    return f"(\n    frame_duration: {frame_duration},\n    animations: {{\n{body}\n    }},\n)\n"


def guns() -> list[str]:
    lines = []
    for wid, (sheet, box, length) in GUNS.items():
        g = shrink(trimmed(Image.open(PACKS / "oga-knekko" / f"{sheet}.png").convert("RGBA"), box), length)
        # Quatre lignes identiques : `sprite_config.index` des armes de weapons.ron vaut 0, 1 ou 3.
        stacked = Image.new("RGBA", (g.width, g.height * 4), (0, 0, 0, 0))
        for r in range(4):
            stacked.alpha_composite(g, (0, r * g.height))
        stacked.save(OUT / f"arme_{wid}.png")
        (OUT / f"arme_{wid}_sol_sheet.ron").write_text(
            sheet_ron(f"arme_{wid}.png", g.size, 4, scale=1.5, anchor="Center", z=0.5))
        (OUT / f"arme_{wid}_main_sheet.ron").write_text(
            sheet_ron(f"arme_{wid}.png", g.size, 4, scale=1.0, anchor="CenterLeft", z=2.0))
        lines.append(f'    "weapon/{wid}": (animation: "sprites/vitrine/arme_animation.ron", '
                     f'layers: {{ "body": "sprites/vitrine/arme_{wid}_sol_sheet.ron" }}),')
        lines.append(f'    "held/{wid}": (animation: "sprites/characters/player/player_animation.ron", '
                     f'layers: {{ "body": "sprites/vitrine/arme_{wid}_main_sheet.ron" }}),')
        print(f"  {wid:15s} {g.width:3d} x {g.height:2d}")
    (OUT / "arme_animation.ron").write_text(animation_ron({"Idle": (0, 0)}))
    return lines


def portal() -> list[str]:
    strip = Image.open(PACKS / "oga-portails/portalRings2.png").convert("RGBA")
    strip.save(OUT / "portail.png")
    (OUT / "portail_sheet.ron").write_text(
        sheet_ron("portail.png", (32, 32), 1, columns=5, scale=2.0, anchor="Center", z=5.0))
    # Fenêtre fixe (les images du monde ne s'animent pas) : la case 2, anneaux nets.
    (OUT / "portail_animation.ron").write_text(animation_ron({"Idle": (2, 2)}))
    return ['    "portal": (animation: "sprites/vitrine/portail_animation.ron", '
            'layers: { "body": "sprites/vitrine/portail_sheet.ron" }),']


def rat_king() -> list[str]:
    src = Image.open(PACKS / "oga-redshrike/rats.png").convert("RGB")
    cells = [(2, 3), (21, 3), (40, 3), (2, 24), (21, 24), (40, 24)]  # face (3) puis côté droit (3), 16 x 18 chacune
    strip = Image.new("RGBA", (16 * len(cells), 18), (0, 0, 0, 0))
    for i, (x, y) in enumerate(cells):
        cell = src.crop((x, y, x + 16, y + 18)).convert("RGBA")
        px = cell.load()
        for yy in range(cell.height):
            for xx in range(cell.width):
                if px[xx, yy][:3] == (0, 128, 0):
                    px[xx, yy] = (0, 0, 0, 0)
        strip.alpha_composite(cell, (16 * i, 0))
    strip.save(OUT / "roi_rat.png")
    (OUT / "roi_rat_sheet.ron").write_text(
        sheet_ron("roi_rat.png", (16, 18), 1, columns=len(cells), scale=2.0, anchor="Center", z=1.0, animated=True))
    (OUT / "roi_rat_animation.ron").write_text(animation_ron({"Idle": (0, 2), "Run": (3, 5)}, 150))
    return ['    "roi_rat": (animation: "sprites/vitrine/roi_rat_animation.ron", layers: { '
            '"body": "sprites/vitrine/roi_rat_sheet.ron", '
            '"shadow": "sprites/characters/shadow/shadow_sheet.ron" }),']


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    print("armes :")
    lines = guns() + portal() + rat_king()
    print(f"{len(lines)} entrées pour sprites/sprites.ron :")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
