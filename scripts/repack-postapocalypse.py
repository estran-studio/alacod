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

For `zombies` only it also writes (all under games/zombies/assets, all gitignored): the boss sheet
(`zombie_boss`, same sheet as `zombie_big`), doors and windows (`door_h/v`, `window_h/v`), weapon and power-up ground icons,
player/enemy projectiles, the wall + floor atlas (`sprites/postapo/atlas_murs.png`, 40 tiles of
16 px, layout of `map_ldtk::wall_art::wall_tile`), HUD icons (`sprites/postapo/hud/`) and the
local HUD overlay `ui/hud.local.ron`. Only pack pixels are used (crop, paste, flip, rotate by 90
degrees, integer nearest scaling, alpha compositing): nothing is drawn.

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


# Surcharge locale de sprites/sprites.ron (`content::registry::load_sprite_sheets`) : entités
# post-apo par jeu. Fichier écrit à côté de la table publique, gitignoré comme les PNG ; chaque id
# doit exister dans la table publique (repli des autres postes et de la CI).
ENEMY_SKINS = {
    "throne": {
        "rat": "zombie_small", "chien": "zombie_small", "rodeur": "zombie_small",
        "pillard": "zombie_axe", "cracheur": "zombie_small", "franc_tireur": "zombie_small",
        "arroseur": "zombie_axe", "tourelle": "zombie_big", "brute": "zombie_big",
        "buffle": "zombie_big", "roi_rat": "zombie_big",
    },
    "zombies": {
        "zombie_1": "zombie_small", "zombie_2": "zombie_axe",
        "zombie_full": "zombie_big", "boss_zombie": "zombie_boss",
    },
}
SHADOW = "sprites/characters/shadow/shadow_sheet.ron"


def write_overlay(game: str, path: Path, extra=()) -> None:
    entries = [("player", "sprites/postapo/player_animation.ron",
                {"body": "sprites/postapo/player_sheet.ron", "shadow": SHADOW})]
    for name in WEAPONS:
        entries.append((name, "sprites/characters/player/player_animation.ron",
                        {"body": f"sprites/postapo/weapon_{name}_sheet.ron"}))
    for enemy, skin in ENEMY_SKINS.get(game, {}).items():
        entries.append((enemy, f"sprites/postapo/{skin}_animation.ron",
                        {"body": f"sprites/postapo/{skin}_sheet.ron", "shadow": SHADOW}))
    entries.extend(extra)
    lines = ["// Surcharge locale (pack TheLazyStone, NON redistribuable) : écrit par",
             "// scripts/repack-postapocalypse.py, gitignoré. Remplace les entrées de même id de sprites.ron.",
             "({"]
    for name, animation, layers in entries:
        body = ", ".join(f'"{k}": "{v}"' for k, v in layers.items())
        lines.append(f'    "{name}": (animation: "{animation}", layers: {{ {body} }}),')
    lines.append("})")
    path.write_text("\n".join(lines) + "\n")


# ---------------------------------------------------------------------------------------------
# Zombies seulement (polissage-zombies-visuel) : boss, portes, fenêtres, icônes d'armes et de
# power-ups, projectiles, atlas des murs et du sol, icônes du HUD. Toutes les images sont des
# pixels du pack découpés, collés, retournés, tournés ou agrandis d'un facteur entier : rien n'est
# dessiné. Les cases du pack introuvables restent transparentes (voir les notes de l'appelant).
# ---------------------------------------------------------------------------------------------

# Le boss reprend la feuille de zombie_big à la même échelle : son agrandissement et sa teinte viennent de
# la donnée (`marker` de `ZombieShooter/Sprites/Zombie/boss_zombie_config.ron`, D66), donc les mêmes avec ou
# sans le pack.
ZOMBIES_ONLY_ENTITIES = {"zombie_boss": (1.4, ENTITIES["zombie_big"][1])}

# Atlas des murs : bloc rouge de Roof_TileSet (colonnes 8..10, lignes 0..4) pour le dessus, face de
# Brick-Wall_TileSet (tuiles 15 = bout gauche, 16 = milieu, 17 = bout droit) pour la face.
ROOF_COL0 = 8
ROOF_INTERIOR_ROW = 3
BRICK_END_W, BRICK_MID, BRICK_END_E = 15, 16, 17
# Quatre sols : (fichier, tuile (colonne, ligne), retournement horizontal, vertical).
FLOOR_TILES = [
    ("Tiles/Background_Bleak-Yellow_TileSet.png", (5, 0), False, False),
    ("Tiles/Background_Bleak-Yellow_TileSet.png", (0, 0), False, False),
    ("Tiles/Background_Bleak-Yellow_TileSet.png", (2, 0), False, False),
    ("Tiles/Background_Bleak-Yellow_TileSet.png", (14, 0), False, False),
]

HUD_ICONS = {  # id du HUD -> icône du pack (couleur et lettre : repli, voir hud.local.ron)
    "juggernog": ("UI/Inventory/Objects/Icon_First-Aid-Kit_Red.png", "#c0392b", "J"),
    "speed_cola": ("UI/Inventory/Objects/Icon_Canned-soup.png", "#27ae60", "S"),
    "double_tap": ("UI/Inventory/Objects/Icon_Bullet-box_Red.png", "#e67e22", "D"),
    "stamin_up": ("UI/Inventory/Objects/Icon_Canned-food.png", "#f1c40f", "T"),
}
HUD_EXTRA = {
    "ammo": "UI/Inventory/Objects/Icon_Bullet-box_Green.png",
    "heart_full": "UI/HP/Heart_Full.png",
    "heart_half": "UI/HP/Heart_Half.png",
    "heart_empty": "UI/HP/Heart_Empty.png",
}


def load(pack: Path, rel: str) -> Image.Image:
    return Image.open(pack / rel).convert("RGBA")


def crop16(im: Image.Image, cx: int, cy: int) -> Image.Image:
    return im.crop((cx * 16, cy * 16, cx * 16 + 16, cy * 16 + 16))


def up(im: Image.Image, k: int) -> Image.Image:
    return im.resize((im.width * k, im.height * k), Image.NEAREST)


def blank(w: int, h: int) -> Image.Image:
    return Image.new("RGBA", (w, h))


def put(dst: Image.Image, src: Image.Image, x: int, y: int) -> None:
    dst.alpha_composite(src, (x, y))


def centred(cell: tuple[int, int], im: Image.Image) -> Image.Image:
    out = blank(*cell)
    put(out, im, (cell[0] - im.width) // 2, (cell[1] - im.height) // 2)
    return out


def write_sheet(out_dir: Path, name: str, frames: list[Image.Image], anims: dict, *,
                z: float = 0.5, own_colors: bool = False) -> None:
    """Frames empilées verticalement (une colonne, comme les portes de gungeon), une feuille et une
    animation RON ; `anims` : nom -> (début, fin)."""
    w, h = frames[0].size
    sheet = blank(w, h * len(frames))
    for i, f in enumerate(frames):
        if f.size != (w, h):
            raise ValueError(f"{name}: frame {i} {f.size} != {(w, h)}")
        put(sheet, f, 0, i * h)
    sheet.save(out_dir / f"{name}.png")
    extra = "    own_colors: true,\n" if own_colors else ""
    (out_dir / f"{name}_sheet.ron").write_text(f"""(
    path: "sprites/postapo/{name}.png",
    tile_size: ({w}, {h}),
    columns: 1,
    rows: {len(frames)},

    animated: false,
    name: "body",

    scale: 1.0,

    anchor: Center,

    offset_x: 0.0,
    offset_y: 0.0,
    offset_z: {z},
{extra}
)
""")
    body = "\n".join(f'        "{k}": (start: {s}, end: {e}),' for k, (s, e) in anims.items())
    (out_dir / f"{name}_animation.ron").write_text(f"""(
    frame_duration: 120,
    animations: {{
{body}
    }},
)
""")


# --- atlas des murs et du sol -----------------------------------------------------------------

def wall_atlas(pack: Path) -> list[Image.Image]:
    """40 tuiles de 16 px dans l'ordre de `map_ldtk::wall_art::wall_tile` : 0..32 dessus
    (`rebords + 8 × coins`), 32..36 face, 36..40 sols."""
    roof = load(pack, "Tiles/Roof_TileSet.png")
    brick = load(pack, "Tiles/Brick-Wall_TileSet.png")

    def r(c: int, row: int) -> Image.Image:
        return crop16(roof, ROOF_COL0 + c, row)

    def b(i: int) -> Image.Image:
        return crop16(brick, i % 6, i // 6)

    def halves(left: Image.Image, right: Image.Image) -> Image.Image:
        t = blank(16, 16)
        put(t, left.crop((0, 0, 8, 16)), 0, 0)
        put(t, right.crop((8, 0, 16, 16)), 8, 0)
        return t

    def top(rim: int, corners: int) -> Image.Image:
        n, e, w = bool(rim & 1), bool(rim & 2), bool(rim & 4)
        row = 0 if n else ROOF_INTERIOR_ROW
        if w and e:
            t = halves(r(0, row), r(2, row))
        elif w:
            t = r(0, row).copy()
        elif e:
            t = r(2, row).copy()
        else:
            t = r(1, row).copy()
        # Angle rentrant : le coin du rebord (3×3 px du coin du bloc de toit) posé sur la tuile.
        if corners & 1:
            put(t, r(2, 0).crop((13, 0, 16, 3)), 13, 0)
        if corners & 2:
            put(t, r(0, 0).crop((0, 0, 3, 3)), 0, 0)
        return t

    tiles = [top(i % 8, i // 8) for i in range(32)]
    tiles.append(b(BRICK_MID).copy())                    # 32 : face
    tiles.append(b(BRICK_END_W).copy())                  # 33 : bout ouest
    tiles.append(b(BRICK_END_E).copy())                  # 34 : bout est
    tiles.append(halves(b(BRICK_END_W), b(BRICK_END_E)))  # 35 : pilier d'une case
    for rel, (cx, cy), flip_h, flip_v in FLOOR_TILES:
        t = crop16(load(pack, rel), cx, cy)
        if flip_h:
            t = t.transpose(Image.FLIP_LEFT_RIGHT)
        if flip_v:
            t = t.transpose(Image.FLIP_TOP_BOTTOM)
        tiles.append(t)
    return tiles


def write_atlas(out_dir: Path, tiles: list[Image.Image]) -> None:
    atlas = blank(8 * 16, 5 * 16)
    for i, t in enumerate(tiles):
        put(atlas, t, (i % 8) * 16, (i // 8) * 16)
    atlas.save(out_dir / "atlas_murs.png")


# --- portes et fenêtres ----------------------------------------------------------------------

def doors(pack: Path, out_dir: Path) -> None:
    wood = load(pack, "Objects/Buildable/Wooden/Wooden-wall_Gate_Horizontal.png")          # 16×15
    steel = load(pack, "Objects/Buildable/Reinforced/Reinforced_wooden-wall_Gate_Horizontal.png")
    post = load(pack, "Objects/Buildable/Wooden/Wooden-wall_Vertical_For-gate.png")        # 7×10
    # Cadenas : les pixels qui diffèrent entre la grille grillagée avec et sans serrure.
    lock_img = load(pack, "Tiles/Wire-Fence/Wire-Fence_Gate_Lock.png")
    plain_img = load(pack, "Tiles/Wire-Fence/Wire-Fence_Gate.png")
    diff = [(x, y) for y in range(16) for x in range(16) if lock_img.getpixel((x, y)) != plain_img.getpixel((x, y))]
    x0, x1 = min(x for x, _ in diff), max(x for x, _ in diff)
    y0, y1 = min(y for _, y in diff), max(y for _, y in diff)
    lock = blank(x1 - x0 + 1, y1 - y0 + 1)
    for x, y in diff:
        lock.putpixel((x - x0, y - y0), lock_img.getpixel((x, y)))

    def h_closed(leaf: Image.Image) -> Image.Image:
        t = blank(32, 16)
        put(t, leaf, 0, 1)
        put(t, leaf, 16, 1)
        return t

    def v_closed(leaf: Image.Image) -> Image.Image:
        rot = leaf.transpose(Image.ROTATE_90)  # 15×16 : planches à l'horizontale
        t = blank(16, 32)
        put(t, rot, 0, 0)
        put(t, rot, 0, 16)
        return t

    h_open = blank(32, 16)
    put(h_open, post, 0, 16 - post.height)
    put(h_open, post, 32 - post.width, 16 - post.height)
    v_open = blank(16, 32)
    put(v_open, post, (16 - post.width) // 2, 0)
    put(v_open, post, (16 - post.width) // 2, 32 - post.height)

    h_locked = h_closed(steel)
    put(h_locked, lock, 16 - lock.width // 2, (16 - lock.height) // 2)
    v_locked = v_closed(steel)
    put(v_locked, lock, (16 - lock.width) // 2, 16 - lock.height // 2)

    anims = {"Idle": (0, 0), "Open": (1, 1), "Locked": (2, 2)}
    write_sheet(out_dir, "door_h", [h_closed(wood), h_open, h_locked], anims)
    write_sheet(out_dir, "door_v", [v_closed(wood), v_open, v_locked], anims)


def window_states(pack: Path) -> list[Image.Image]:
    """Fenêtre beige 15×15 : verrouillée (deux planches), une planche, une demi-planche, ouverte.
    Les planches sont les pixels de Window_18 (boarded-up) qui diffèrent de Window_14 (broken) :
    lignes 5..8 = planche haute, 9..11 = planche basse. On repart de Window_18 et on recopie les
    pixels de Window_14 là où une planche est retirée."""
    boarded = load(pack, "Objects/Windows/Window_18_Boarded-up_Beige.png")
    broken = load(pack, "Objects/Windows/Window_14_Broken_Beige.png")

    def without(base: Image.Image, rows: range, x_from: int = 0) -> Image.Image:
        t = base.copy()
        for y in rows:
            for x in range(x_from, 15):
                if boarded.getpixel((x, y)) != broken.getpixel((x, y)):
                    t.putpixel((x, y), broken.getpixel((x, y)))
        return t

    two_thirds = without(boarded, range(5, 9))                     # planche haute arrachée
    one_third = without(two_thirds, range(9, 12), 8)               # moitié droite de la planche basse arrachée
    return [boarded.copy(), two_thirds, one_third, broken.copy()]


def windows(pack: Path, out_dir: Path, atlas: list[Image.Image]) -> None:
    """Fenêtres `window_h` (entité 16 × 32) et `window_v` (32 × 16) : même image, **32 × 32**, la fenêtre du pack
    agrandie d'un facteur entier (×2, 30 × 30) et centrée : à l'échelle du jeu (≈ 2,4 px par unité) une fenêtre
    de 15 px laissait les planches à un pixel et les quatre états indiscernables. L'image dépasse de l'entité
    (les cases voisines sont des murs), comme un zombie de 32 dépasse son collider de 20."""
    states = window_states(pack)
    anims = {"Idle": (0, 0), "Damaged": (1, 1), "Critical": (2, 2), "Broken": (3, 3)}
    frames = []
    for st in states:
        t = blank(32, 32)
        put(t, st.resize((st.width * 2, st.height * 2), Image.NEAREST), 1, 1)
        frames.append(t)
    write_sheet(out_dir, "window_h", frames, anims)
    write_sheet(out_dir, "window_v", frames, anims)


# --- icônes de sol, projectiles ---------------------------------------------------------------

def icons(pack: Path, out_dir: Path) -> list[tuple[str, str]]:
    """Icônes d'armes et de power-ups ; retourne (id de sprite, nom de fichier)."""
    out = []
    weapons = {"pistol": "Icon_Pistol", "machine_gun": "Icon_Gun", "shotgun": "Icon_Shotgun"}
    for key, icon in weapons.items():
        im = load(pack, f"UI/Inventory/Objects/{icon}.png")
        write_sheet(out_dir, f"icon_{key}", [centred((16, 12), im)], {"Idle": (0, 0)})
        out.append((f"weapon/{key}", f"icon_{key}"))
    out.append(("weapon", "icon_pistol"))
    for key, rel in {"max_ammo": "Icon_Bullet-crate_Red", "carpenter": "Icon_Wooden-wall"}.items():
        im = load(pack, f"UI/Inventory/Objects/{rel}.png")
        write_sheet(out_dir, f"powerup_{key}", [centred((16, 16), im)], {"Idle": (0, 0)})
        out.append((f"powerup/{key}", f"powerup_{key}"))
    return out


def bullets(pack: Path, out_dir: Path) -> list[tuple[str, str]]:
    """Projectiles : une seule image par fichier (le moteur charge le PNG entier et l'étire dans un
    carré), tournée vers la droite. Joueur : cartouche du fusil (4×1) ×3 ; ennemi : crachat
    `shot_1` image 0 (7×6, pointe en bas) tourné d'un quart de tour vers la droite, ×2."""
    shot = load(pack, "Character/Guns/Bullets/Gun-bullet_Whole.png")
    write_sheet(out_dir, "bullet_player", [centred((12, 12), up(shot, 3))], {"Idle": (0, 0)}, z=0.0, own_colors=True)
    spit = load(pack, "Enemies/Shot/shot_1-Sheet3.png").crop((0, 0, 7, 6)).transpose(Image.ROTATE_90)
    write_sheet(out_dir, "bullet_enemy", [centred((14, 14), up(spit, 2))], {"Idle": (0, 0)}, z=0.0, own_colors=True)
    return [("bullet_player", "bullet_player"), ("bullet_enemy", "bullet_enemy")]


# --- icônes du HUD et surcharge de ui/hud.ron -------------------------------------------------

def hud_icons(pack: Path, assets: Path) -> None:
    out = assets / "sprites/postapo/hud"
    out.mkdir(parents=True, exist_ok=True)
    sources = {k: v[0] for k, v in HUD_ICONS.items()}
    sources.update(HUD_EXTRA)
    for key, rel in sources.items():
        im = load(pack, rel)
        if max(im.size) < 12:
            im = up(im, 2)
        im.save(out / f"{key}.png")
    lines = [
        "// Surcharge locale de ui/hud.ron (pack TheLazyStone, NON redistribuable) : écrit par",
        "// scripts/repack-postapocalypse.py, gitignoré. Images dans sprites/postapo/.",
        "(",
        '    wall_art: Some((layer: "Walls", atlas: "sprites/postapo/atlas_murs.png", tile: 16, columns: 8, '
        'hide_layers: ["LevelConnection", "Walls", "Sol", "Mobilier"], floor: Some((tiles: [36, 37, 38, 39], mask_layer: Some("Sol"))))),',
        "    icons: {",
    ]
    for key, (_, color, label) in HUD_ICONS.items():
        lines.append(f'        "{key}": (color: "{color}", label: "{label}", image: Some("sprites/postapo/hud/{key}.png")),')
    lines += ["    },", ")"]
    (assets / "ui").mkdir(parents=True, exist_ok=True)
    (assets / "ui/hud.local.ron").write_text("\n".join(lines) + "\n")


def zombies_extras(pack: Path, assets: Path, out_dir: Path) -> list:
    """Tout ce que `zombies` ajoute ; retourne les entrées de sprites.local.ron."""
    for name, (scale, states) in ZOMBIES_ONLY_ENTITIES.items():
        repack(pack, out_dir, name, scale, states)
    atlas = wall_atlas(pack)
    write_atlas(out_dir, atlas)
    doors(pack, out_dir)
    windows(pack, out_dir, atlas)
    entries = []
    for sprite in ("door_v", "door_h", "window_h", "window_v"):
        entries.append((sprite, f"sprites/postapo/{sprite}_animation.ron", {"body": f"sprites/postapo/{sprite}_sheet.ron"}))
    for sprite, stem in icons(pack, out_dir) + bullets(pack, out_dir):
        entries.append((sprite, f"sprites/postapo/{stem}_animation.ron", {"body": f"sprites/postapo/{stem}_sheet.ron"}))
    hud_icons(pack, assets)
    return entries


def main() -> None:
    pack = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / "Downloads/PostApocalypse_AssetPack_v1.1.2"
    games = sys.argv[2:] or ["throne", "zombies"]
    if not pack.is_dir():
        sys.exit(f"pack introuvable : {pack}")
    repo = Path(__file__).resolve().parent.parent
    for game in games:
        assets = repo / f"games/{game}/assets"
        out_dir = assets / "sprites/postapo"
        out_dir.mkdir(parents=True, exist_ok=True)
        for name, (scale, states) in ENTITIES.items():
            repack(pack, out_dir, name, scale, states)
        for name, rel in WEAPONS.items():
            weapon(pack, out_dir, name, rel)
        extra = zombies_extras(pack, assets, out_dir) if game == "zombies" else []
        write_overlay(game, out_dir.parent / "sprites.local.ron", extra)
        print(f"{game}: {len(ENTITIES)} personnages + {len(WEAPONS)} armes -> {out_dir} (+ sprites.local.ron)")


if __name__ == "__main__":
    main()
