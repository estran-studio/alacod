#!/usr/bin/env python3
"""Machines à perk de zombies (S1-8 suite, décision de William du 29 oct) : images CC0 du pack « Pixel Art
Vending Machines » de karsiori, **commitées** (CC0) ; écrit `games/zombies/assets/sprites/props/`.

Usage : scripts/repack-distributeurs.py <« Pixel Art Vending Machines Pack » (dossier extrait ou .zip)>

Une feuille par perk (`perk_machine_<perk>_sheet.ron`, une case) et une animation commune
(`perk_machine_animation.ron`, `Idle`). La couleur de la machine suit celle de l'icône du perk de `hud.ron`
(juggernog rouge, speed_cola vert, double_tap orange, stamin_up jaune-olive). L'image est réduite à 48 px de haut
(les 16 px d'une case du jeu, les personnages en font 20 à 32) par filtre Lanczos, sans autre retouche.
Les entrées de `sprites.ron` : `perk_machine/<perk>` (voir `world_sprites::perk_machine_candidates`).
"""
import io
import sys
import zipfile
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "games/zombies/assets/sprites/props"
HEIGHT = 48
# perk -> fichier du pack (variante avec vitre)
PERKS = {
    "juggernog": "Machine 3/Vending Machine 3.1.png",
    "speed_cola": "Machine 3/Vending Machine 3.2.png",
    "double_tap": "Machine 1/Vending Machine 1.1.png",
    "stamin_up": "Machine 3/Vending Machine 3.3.png",
}
ROOT_NAME = "Pixel Art Vending Machines Pack"


def open_member(src: Path, member: str) -> Image.Image:
    if src.is_file():
        with zipfile.ZipFile(src) as zf:
            return Image.open(io.BytesIO(zf.read(f"{ROOT_NAME}/{member}"))).convert("RGBA")
    base = src if (src / "Machine 1").is_dir() else src / ROOT_NAME
    return Image.open(base / member).convert("RGBA")


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    src = Path(sys.argv[1]).expanduser()
    OUT.mkdir(parents=True, exist_ok=True)
    for perk, member in PERKS.items():
        im = open_member(src, member)
        w = round(im.width * HEIGHT / im.height)
        im = im.resize((w, HEIGHT), Image.LANCZOS)
        im.save(OUT / f"perk_machine_{perk}.png")
        # Posée sur le centre de la case : le bas de l'image affleure le bas de la case (16 px).
        (OUT / f"perk_machine_{perk}_sheet.ron").write_text(f"""(
    path: "sprites/props/perk_machine_{perk}.png",
    tile_size: ({w}, {HEIGHT}),
    columns: 1,
    rows: 1,

    animated: false,
    name: "body",

    scale: 1.0,

    anchor: Center,

    offset_x: 0.0,
    offset_y: {HEIGHT / 2 - 8:.1f},
    offset_z: 0.5,

)
""")
        print(perk, im.size)
    (OUT / "perk_machine_animation.ron").write_text("""(
    frame_duration: 120,
    animations: {
        "Idle": (start: 0, end: 0),
    },
)
""")


if __name__ == "__main__":
    main()
