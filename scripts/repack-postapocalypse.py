#!/usr/bin/env python3
"""Repack TheLazyStone Post-Apocalypse sheets into alacod sprite sheets.

The pack exports one PNG per (direction, state) with non-uniform frame cells
(width varies per sheet, sheets are tight-cropped). Our SpriteSheetConfig
needs one uniform grid per sheet, so this script slices the *side* sheets
(the engine renders side view + flip_x) and pastes every frame bottom-centred
into a common cell, concatenated as Idle | Run | Dying_1 | Attack in a single
row. It also crops one frame of each gun overlay (hands + weapon) into the
rotating weapon-visual sheet (4 identical rows: weapon sprite_config indexes
0..3 must stay valid).

Source pack is NOT redistributable (see games/*/assets/assets.yaml,
id thelazystone-postapocalypse): generated PNGs land in
games/<game>/assets/sprites/postapo/, which is gitignored. Never commit them.

Usage:
    python3 -m venv /tmp/postapo-venv && /tmp/postapo-venv/bin/pip install pillow
    /tmp/postapo-venv/bin/python scripts/repack-postapocalypse.py [PACK_DIR] [games...]

PACK_DIR defaults to ~/Downloads/PostApocalypse_AssetPack_v1.1.2,
games default to throne and zombies.
"""

import re
import sys
from pathlib import Path

from PIL import Image

# (output name, scale, [(state name in our AnimationMapConfig, pack rel path)])
ENTITIES = {
    "player": (2.0, [
        ("Idle", "Character/Main/Idle/Character_side_idle-Sheet6.png"),
        ("Run", "Character/Main/Run/Character_side_run-Sheet6.png"),
        ("Dying_1", "Character/Main/Death/Character_side_death1-Sheet6.png"),
    ]),
    "zombie_small": (2.0, [
        ("Idle", "Enemies/Zombie_Small/Zombie_Small_Side_Idle-Sheet6.png"),
        ("Run", "Enemies/Zombie_Small/Zombie_Small_Side_Walk-Sheet6.png"),
        ("Dying_1", "Enemies/Zombie_Small/Zombie_Small_Side_First-Death-Sheet6.png"),
        ("Attack", "Enemies/Zombie_Small/Zombie_Small_Side_First-Attack-Sheet4.png"),
    ]),
    "zombie_axe": (1.6, [
        ("Idle", "Enemies/Zombie_Axe/Zombie_Axe_Side_Idle-Sheet6.png"),
        ("Run", "Enemies/Zombie_Axe/Zombie_Axe_Side_Walk-Sheet8.png"),
        ("Dying_1", "Enemies/Zombie_Axe/Zombie_Axe_Side_First-Death-Sheet6.png"),
        ("Attack", "Enemies/Zombie_Axe/Zombie_Axe_Side_First-Attack-Sheet7.png"),
    ]),
    "zombie_big": (1.4, [
        ("Idle", "Enemies/Zombie_Big/Zombie_Big_Side_Idle-Sheet6.png"),
        ("Run", "Enemies/Zombie_Big/Zombie_Big_Side_Walk-Sheet8.png"),
        ("Dying_1", "Enemies/Zombie_Big/Zombie_Big_Side_First-Death-Sheet7.png"),
        ("Attack", "Enemies/Zombie_Big/Zombie_Big_Side_First-Attack-Sheet8.png"),
    ]),
}

# Rotating weapon visuals: one cropped frame of the hands+gun overlay.
WEAPONS = {
    "pistol": "Character/Guns/Pistol/Pistol_side_idle-and-run-Sheet6.png",
    "machine_gun": "Character/Guns/Gun/Gun_side_idle-and-run-Sheet6.png",
    "shotgun": "Character/Guns/Shotgun/Shotgun_side_idle-and-run-Sheet6.png",
}


def sheet_frames(pack: Path, rel: str) -> list[Image.Image]:
    m = re.search(r"Sheet(\d+)\.png$", rel)
    if not m:
        raise ValueError(f"pas de SheetN dans {rel}")
    n = int(m.group(1))
    im = Image.open(pack / rel).convert("RGBA")
    if im.width % n:
        raise ValueError(f"{rel}: largeur {im.width} non divisible par {n}")
    fw = im.width // n
    return [im.crop((i * fw, 0, (i + 1) * fw, im.height)) for i in range(n)]


def repack(pack: Path, out_dir: Path, name: str, scale: float, states) -> None:
    sliced = [(state, sheet_frames(pack, rel)) for state, rel in states]
    cell_w = max(f.width for _, frames in sliced for f in frames)
    cell_h = max(f.height for _, frames in sliced for f in frames)
    total = sum(len(frames) for _, frames in sliced)
    sheet = Image.new("RGBA", (cell_w * total, cell_h))
    anims, idx = [], 0
    for state, frames in sliced:
        anims.append(f'        "{state}": (\n            start: {idx},\n            end: {idx + len(frames) - 1},\n        ),')
        for f in frames:
            sheet.alpha_composite(f, (idx * cell_w + (cell_w - f.width) // 2, cell_h - f.height))
            idx += 1
    sheet.save(out_dir / f"{name}.png")
    (out_dir / f"{name}_sheet.ron").write_text(f"""(
    path: "sprites/postapo/{name}.png",
    tile_size: ({cell_w}, {cell_h}),
    columns: {total},
    rows: 1,

    animated: true,
    name: "body",

    scale: {scale},

    anchor: Center,

    offset_x: 0.0,
    offset_y: 0.0,
    offset_z: 1.0,

)
""")
    (out_dir / f"{name}_animation.ron").write_text("""(
    frame_duration: 120,
    animations: {
""" + "\n".join(anims) + """
    },
)
""")


def weapon(pack: Path, out_dir: Path, name: str, rel: str) -> None:
    frame = sheet_frames(pack, rel)[0]
    bbox = frame.getbbox()
    crop = frame.crop(bbox) if bbox else frame
    rows = 4  # sprite_config.index vaut 0..3 selon l'arme (weapons.ron)
    sheet = Image.new("RGBA", (crop.width, crop.height * rows))
    for r in range(rows):
        sheet.alpha_composite(crop, (0, r * crop.height))
    sheet.save(out_dir / f"weapon_{name}.png")
    (out_dir / f"weapon_{name}_sheet.ron").write_text(f"""(
    path: "sprites/postapo/weapon_{name}.png",
    tile_size: ({crop.width}, {crop.height}),
    columns: 1,
    rows: {rows},

    name: "body",
    animated: false,

    scale: 1.4,

    anchor: CenterLeft,

    offset_x: 0.0,
    offset_y: 0.0,
    offset_z: 2.0,

)
""")


def main() -> None:
    pack = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / "Downloads/PostApocalypse_AssetPack_v1.1.2"
    games = sys.argv[2:] or ["throne", "zombies"]
    if not pack.is_dir():
        sys.exit(f"pack introuvable : {pack}")
    repo = Path(__file__).resolve().parent.parent
    for game in games:
        out_dir = repo / f"games/{game}/assets/sprites/postapo"
        out_dir.mkdir(parents=True, exist_ok=True)
        for name, (scale, states) in ENTITIES.items():
            repack(pack, out_dir, name, scale, states)
        for name, rel in WEAPONS.items():
            weapon(pack, out_dir, name, rel)
        print(f"{game}: {len(ENTITIES)} personnages + {len(WEAPONS)} armes -> {out_dir}")


if __name__ == "__main__":
    main()
