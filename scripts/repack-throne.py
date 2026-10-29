#!/usr/bin/env python3
"""Surcharge locale de throne (S2-10) : packs payants **non redistribuables**, jamais commités.

Packs lus (archives ouvertes fichier par fichier, rien n'est extrait en bloc) :
  - Aymen B « UPDATE 2_ZombiePack » (https://pixelrogueknight.itch.io/), id `aymen-b-zombiepack` d'`assets.yaml` ;
  - Ittai Manero « Zombie Apocalypse Tileset » (https://ittaimanero.itch.io/zombie-apocalypse-tileset),
    id `ittai-zombie-apocalypse` d'`assets.yaml`.

Écrit, sous `games/throne/assets/` (gitignoré : `sprites/postapo/`, `sprites/*.local.ron`, `ui/*.local.ron`) :
  - `sprites/postapo/` : feuilles et animations (RON) des personnages, de la tourelle, des armes, des projectiles,
    des objets, des coffres, des power-ups, atlas de murs et de sol ;
  - `sprites/sprites.local.ron` : entrées qui remplacent ou ajoutent des ids à `sprites/sprites.ron` ;
  - `ui/hud.local.ron` : `wall_art` (murs et sol) ; `hud.ron` n'est pas touché.
Aucun pixel n'est dessiné : découpe, collage, miroir, mise à l'échelle entière (conventions § 92, § 96).

Usage : scripts/repack-throne.py [ZIP_AYMEN_B] [ZIP_ITTAI]
Défauts : ~/Downloads/UPDATE 2_ZombiePack.zip et ~/Downloads/Zombie Apocalypse Tileset.zip.
"""
import io
import sys
import zipfile
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "games/throne/assets"
OUT = ASSETS / "sprites/postapo"

AYMEN_ROOT = "UPDATE 2_ZombiePack/"
ITTAI_ROOT = "Zombie Apocalypse Tileset/Organized separated sprites/"


class Pack:
    """Archive zip lue membre par membre (jamais extraite)."""

    def __init__(self, path: Path, root: str):
        self.zf = zipfile.ZipFile(path)
        self.root = root

    def load(self, rel: str) -> Image.Image:
        return Image.open(io.BytesIO(self.zf.read(self.root + rel))).convert("RGBA")

    def folder(self, rel: str) -> list[Image.Image]:
        """Images d'un dossier (ordre du nom), pour le pack d'Ittai (une image par fichier)."""
        prefix = self.root + rel
        names = sorted(n for n in self.zf.namelist()
                       if n.startswith(prefix) and n.endswith(".png") and "__MACOSX" not in n)
        if not names:
            raise FileNotFoundError(prefix)
        return [Image.open(io.BytesIO(self.zf.read(n))).convert("RGBA") for n in names]


def blank(w: int, h: int) -> Image.Image:
    return Image.new("RGBA", (w, h))


def strip(im: Image.Image, n: int) -> list[Image.Image]:
    """Découpe une rangée de `n` cases égales."""
    if im.width % n:
        raise ValueError(f"largeur {im.width} non divisible par {n}")
    fw = im.width // n
    return [im.crop((i * fw, 0, (i + 1) * fw, im.height)) for i in range(n)]


def union_bbox(frames: list[Image.Image]) -> tuple[int, int, int, int]:
    boxes = [b for b in (f.getbbox() for f in frames) if b]
    return (min(b[0] for b in boxes), min(b[1] for b in boxes),
            max(b[2] for b in boxes), max(b[3] for b in boxes))


def up(im: Image.Image, k: int) -> Image.Image:
    return im.resize((im.width * k, im.height * k), Image.NEAREST)


def write_sheet_ron(name: str, tile: tuple[int, int], columns: int, rows: int, *, scale: float,
                    animated: bool, anchor: str = "Center", z: float = 1.0, own_colors: bool = False,
                    png: str | None = None) -> str:
    extra = "    own_colors: true,\n" if own_colors else ""
    return f"""(
    path: "sprites/postapo/{png or name}.png",
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
{extra}
)
"""


def write_animation_ron(name: str, anims: dict[str, tuple[int, int]]) -> None:
    body = "\n".join(f'        "{k}": (start: {a}, end: {b}),' for k, (a, b) in anims.items())
    (OUT / f"{name}_animation.ron").write_text(
        f"(\n    frame_duration: 120,\n    animations: {{\n{body}\n    }},\n)\n")


def entry(sprite_id: str, animation: str, sheet: str) -> str:
    return (f'    "{sprite_id}": (animation: "sprites/postapo/{animation}_animation.ron", '
            f'layers: {{ "body": "sprites/postapo/{sheet}_sheet.ron" }}),')


def entry_shadowed(sprite_id: str, animation: str, sheet: str) -> str:
    return (f'    "{sprite_id}": (animation: "sprites/postapo/{animation}_animation.ron", layers: {{ '
            f'"body": "sprites/postapo/{sheet}_sheet.ron", '
            f'"shadow": "sprites/characters/shadow/shadow_sheet.ron" }}),')


# --- personnages et ennemis (Aymen B, cases de 32 × 32) ------------------------------------------

Z = "AnimationSheets/Zombie_Character_32x32/"
SKINS = {  # peau -> (Idle 6, Run 8, Dying_1 8)
    "player": ("AnimationSheets/Player_Character_32x32/Player_idle.png",
               "AnimationSheets/Player_Character_32x32/Player_run.png",
               "AnimationSheets/Player_Character_32x32/Player_Death .png"),
    "bandit": ("AnimationSheets/Bandit_character_32x32/Animations_bandit-idle.png",
               "AnimationSheets/Bandit_character_32x32/Animations_bandit-Run.png",
               "AnimationSheets/Bandit_character_32x32/Animations_bandit-Death.png"),
    "zombie1": (Z + "Zombie1/Zombie_Idle.png", Z + "Zombie1/Zombie_run.png", Z + "Zombie1/Zombie_Death 1.png"),
    "zombie2": (Z + "Zombie2/Zombie 2_Idle.png", Z + "Zombie2/Zombie 2_Run.png", Z + "Zombie2/Zombie 2_Death 1 .png"),
    "zombie3": (Z + "Zombie3/Zombie 3_idle .png", Z + "Zombie3/Zombie 3_run .png", Z + "Zombie3/Zombie 3_death.png"),
    "zombie4": (Z + "Zombie4/Zombie 4_idle.png", Z + "Zombie4/Zombie 4_run.png", Z + "Zombie4/Zombie 4_death 4.png"),
}
# id de personnage de throne -> (peau, échelle de présentation). 9 ennemis sur 5 peaux : la différence
# d'échelle sépare les voisins (la taille de collision, elle, vient du personnage, pas de l'image).
CHARACTERS = {
    "player": ("player", 1.0),
    "rat": ("zombie2", 0.65),
    "chien": ("zombie1", 0.8),
    "rodeur": ("zombie2", 1.0),
    "pillard": ("bandit", 1.0),
    "cracheur": ("zombie3", 1.0),
    "franc_tireur": ("bandit", 0.85),
    "arroseur": ("zombie1", 1.0),
    "brute": ("zombie4", 1.0),
    "buffle": ("zombie4", 1.2),
    "roi_rat": ("zombie4", 1.0),  # l'échelle 1,4 vient de `characters/roi_rat.ron`
}


def skins(aymen: Pack) -> list[str]:
    lines = []
    for skin, (idle, run, dead) in SKINS.items():
        frames = strip(aymen.load(idle), 6) + strip(aymen.load(run), 8) + strip(aymen.load(dead), 8)
        x0, y0, x1, y1 = union_bbox(frames)
        w, h = x1 - x0, y1 - y0
        w += w % 2
        sheet = blank(w * len(frames), h)
        for i, f in enumerate(frames):
            sheet.alpha_composite(f.crop((x0, y0, x0 + w, y1)), (i * w, 0))
        sheet.save(OUT / f"char_{skin}.png")
        write_animation_ron(f"char_{skin}", {"Idle": (0, 5), "Run": (6, 13), "Dying_1": (14, 21)})
        for cid, (s, scale) in CHARACTERS.items():
            if s != skin:
                continue
            (OUT / f"char_{cid}_sheet.ron").write_text(
                write_sheet_ron(f"char_{cid}", (w, h), len(frames), 1, scale=scale, animated=True,
                                png=f"char_{skin}"))
            lines.append(entry_shadowed(cid, f"char_{skin}", f"char_{cid}"))
    return lines


# --- tourelle (Ittai Manero : zombie tourelle), cible et mannequin -------------------------------

def turret(ittai: Pack) -> list[str]:
    idle = ittai.folder("Turret Zombie Animation Frames/")
    x0, y0, x1, y1 = union_bbox(idle)
    w, h = x1 - x0, y1 - y0
    sheet = blank(w * len(idle), h)
    for i, f in enumerate(idle):
        sheet.alpha_composite(f.crop((x0, y0, x1, y1)), (i * w, 0))
    sheet.save(OUT / "char_tourelle.png")
    n = len(idle)
    # Immobile : `Run` rejoue `Idle` ; aucune animation de mort dans le pack (le corps s'efface sans image).
    write_animation_ron("char_tourelle", {"Idle": (0, n - 1), "Run": (0, n - 1)})
    (OUT / "char_tourelle_sheet.ron").write_text(
        write_sheet_ron("char_tourelle", (w, h), n, 1, scale=2.0, animated=True))
    return [entry_shadowed("tourelle", "char_tourelle", "char_tourelle")]


def dummies(ittai: Pack) -> list[str]:
    lines = []
    for cid, folder in (("mannequin", "Scarecrow/"), ("cible", "Shootable Coke Can Animation Frames/")):
        frame = ittai.folder(folder)[0]
        crop = frame.crop(frame.getbbox())
        crop = up(crop, 2)
        crop.save(OUT / f"char_{cid}.png")
        write_animation_ron(f"char_{cid}", {"Idle": (0, 0), "Run": (0, 0)})
        (OUT / f"char_{cid}_sheet.ron").write_text(
            write_sheet_ron(f"char_{cid}", crop.size, 1, 1, scale=1.0, animated=False))
        lines.append(entry_shadowed(cid, f"char_{cid}", f"char_{cid}"))
    return lines


# --- armes portées, icônes d'armes au sol -----------------------------------------------------

W32 = "AnimationSheets/Weaonns_Sprites_32x32/"
HELD = {  # sprite_config de weapons.ron -> arme d'Aymen
    "pistol": W32 + "Pistol_Flicker.png",
    "machine_gun": W32 + "Rifle_Flicker.png",
    "shotgun": W32 + "Pump_Flicker.png",
}
GROUND = {  # id d'arme de throne -> (image du pack, source)
    "revolver": W32 + "Revolver_Flicker.png",
    "mitraillette": W32 + "Rifle_Flicker.png",
    "fusil_a_pompe": W32 + "Pump_Flicker.png",
    "canon_ricochet": W32 + "Sniper_Flicker.png",
    "roquette": W32 + "RocketLauncher_Flicker.png",
    "lance_lames": W32 + "Knife.png",
}


def gun(aymen: Pack, rel: str) -> Image.Image:
    first = strip(aymen.load(rel), 7)[0]
    return first.crop(first.getbbox())


def weapons(aymen: Pack) -> list[str]:
    lines = []
    for name, rel in HELD.items():
        g = gun(aymen, rel)
        sheet = blank(g.width, g.height * 4)
        for r in range(4):
            sheet.alpha_composite(g, (0, r * g.height))
        sheet.save(OUT / f"weapon_{name}.png")
        (OUT / f"weapon_{name}_sheet.ron").write_text(
            write_sheet_ron(f"weapon_{name}", (g.width, g.height), 1, 4, scale=1.0, animated=False,
                            anchor="CenterLeft", z=2.0))
        lines.append(f'    "{name}": (animation: "sprites/characters/player/player_animation.ron", '
                     f'layers: {{ "body": "sprites/postapo/weapon_{name}_sheet.ron" }}),')
    icons = dict(GROUND)
    for wid, rel in icons.items():
        g = gun(aymen, rel)
        g.save(OUT / f"icon_{wid}.png")
        write_animation_ron(f"icon_{wid}", {"Idle": (0, 0)})
        (OUT / f"icon_{wid}_sheet.ron").write_text(
            write_sheet_ron(f"icon_{wid}", g.size, 1, 1, scale=1.0, animated=False, z=0.5))
        lines.append(entry(f"weapon/{wid}", f"icon_{wid}", f"icon_{wid}"))
    # Image générique des armes au sol sans dessin dans le pack : le pistolet (dite telle quelle au rapport).
    lines.append(entry("weapon", "icon_revolver", "icon_revolver"))
    return lines


# --- projectiles, objets, coffres, power-ups -------------------------------------------------

def projectiles(aymen: Pack, ittai: Pack) -> list[str]:
    sheet = aymen.load("SpriteSheets/items & bullets sprites(16x16).png")

    def cell(col: int, row: int) -> Image.Image:
        return sheet.crop((col * 16, row * 16, col * 16 + 16, row * 16 + 16))

    def centred(im: Image.Image, size: int) -> Image.Image:
        out = blank(size, size)
        c = im.crop(im.getbbox())
        out.alpha_composite(c, ((size - c.width) // 2, (size - c.height) // 2))
        return out

    lines = []
    # joueur : lueur orange d'Aymen ; ennemi : crachat du zombie tourelle d'Ittai (vert, jamais confondu)
    spit = ittai.folder("Turret Zombie Vomit Shooting Animation Frames /")[1]
    for sid, src in (("bullet_player", cell(1, 5)), ("bullet_enemy", spit)):
        im = centred(src, 16)
        im.save(OUT / f"{sid}.png")
        write_animation_ron(sid, {"Idle": (0, 0)})
        (OUT / f"{sid}_sheet.ron").write_text(
            write_sheet_ron(sid, (16, 16), 1, 1, scale=1.0, animated=False, z=0.0, own_colors=True))
        lines.append(entry(sid, sid, sid))
    return lines


ITEMS16 = "AnimationSheets/Items_Sprites_16x16/item animations -"
POWERUPS = {  # id de power-up -> fichier d'objet
    "munitions_balles": "ammo-rifle",
    "munitions_obus": "grenades ",
    "munitions_explosifs": "Dynamite",
    "munitions_energie": "battery",
    "munitions_lames": "ammo",
    "rage": "skull ",
    "vitesse": "gas",
}


def objects(aymen: Pack) -> list[str]:
    lines = []
    for pid, name in POWERUPS.items():
        im = strip(aymen.load(ITEMS16 + name + ".png"), 7)[0]
        im = up(im, 2)
        im.save(OUT / f"powerup_{pid}.png")
        write_animation_ron(f"powerup_{pid}", {"Idle": (0, 0)})
        (OUT / f"powerup_{pid}_sheet.ron").write_text(
            write_sheet_ron(f"powerup_{pid}", im.size, 1, 1, scale=1.0, animated=False, z=0.5))
        lines.append(entry(f"powerup/{pid}", f"powerup_{pid}", f"powerup_{pid}"))
    # coffre : fermé = première image de « RedChest », ouvert = dernière image de la planche d'ouverture (32 × 16)
    closed = strip(aymen.load(ITEMS16 + "RedChest.png"), 7)[0]
    opening = strip(aymen.load("AnimationSheets/chest pixel art-Chest_Open_32x16.png"), 6)
    opened = opening[-1]
    cb = closed.getbbox()
    ob = opened.getbbox()
    w = max(cb[2] - cb[0], ob[2] - ob[0])
    h = max(cb[3] - cb[1], ob[3] - ob[1])
    chest = blank(w * 2, h)
    for i, (f, b) in enumerate(((closed, cb), (opened, ob))):
        c = f.crop(b)
        chest.alpha_composite(c, (i * w + (w - c.width) // 2, h - c.height))
    chest = up(chest, 2)
    chest.save(OUT / "chest.png")
    write_animation_ron("chest", {"Idle": (0, 0), "Open": (1, 1)})
    (OUT / "chest_sheet.ron").write_text(
        write_sheet_ron("chest", (w * 2, h * 2), 2, 1, scale=1.0, animated=False, z=0.5))
    lines.append(entry("chest", "chest", "chest"))
    return lines


# --- murs (toit + face) et sol : atlas de wall_art (40 tuiles de 16 px, 8 colonnes) -------------

def wall_atlas(aymen: Pack) -> Image.Image:
    """Même ordre que `map_ldtk::wall_art::wall_tile` : 0..32 dessus (`rebords + 8 × coins`, rebords
    n = 1, e = 2, w = 4), 32..36 face, 36..40 sol. Murs : toit et mur d'intérieur de la planche
    d'intérieur d'Aymen B (région de 96 × 160 px à partir de (0, 32)) ; sol : sable de Wasteland (quatre quarts de l'intérieur d'une même tuile : aucun bord ne fait couture)."""
    interior = aymen.load("TileSheets/Interior_Tileset_32x32.png")
    sand = aymen.load("TileSheets/Wasteland_Tileset_32x32.png")

    def crop(im: Image.Image, x: int, y: int) -> Image.Image:
        return im.crop((x, y, x + 16, y + 16))

    X0, X1, XM = 0, 80, 40          # colonnes : bord ouest, bord est, milieu (région à partir de x = 0)
    Y0, YM = 32, 56                 # rangées : bord nord, milieu du toit
    FACE_Y = 160                    # la face : bas de la région (panneau + plinthe)
    FACE_M, FACE_W, FACE_E = crop(interior, XM, FACE_Y), crop(interior, X0, FACE_Y), crop(interior, X1, FACE_Y)

    def halves(left: Image.Image, right: Image.Image) -> Image.Image:
        t = blank(16, 16)
        t.alpha_composite(left.crop((0, 0, 8, 16)), (0, 0))
        t.alpha_composite(right.crop((8, 0, 16, 16)), (8, 0))
        return t

    def top(rim: int, corners: int) -> Image.Image:
        n, e, w = bool(rim & 1), bool(rim & 2), bool(rim & 4)
        y = Y0 if n else YM
        if w and e:
            t = halves(crop(interior, X0, y), crop(interior, X1, y))
        elif w:
            t = crop(interior, X0, y)
        elif e:
            t = crop(interior, X1, y)
        else:
            t = crop(interior, XM, y)
        t = t.copy()
        # Angle rentrant : le coin du rebord (3 × 3 px) posé sur la tuile.
        if corners & 1:
            t.alpha_composite(crop(interior, X1, Y0).crop((13, 0, 16, 3)), (13, 0))
        if corners & 2:
            t.alpha_composite(crop(interior, X0, Y0).crop((0, 0, 3, 3)), (0, 0))
        return t

    tiles = [top(i % 8, i // 8) for i in range(32)]
    tiles += [FACE_M.copy(), FACE_W.copy(), FACE_E.copy(), halves(FACE_W, FACE_E)]
    for tile, ox, oy in ((9, 4, 4), (9, 12, 4), (9, 4, 12), (9, 12, 12)):
        tiles.append(crop(sand, (tile % 9) * 32 + ox, (tile // 9) * 32 + oy))
    atlas = blank(8 * 16, 5 * 16)
    for i, t in enumerate(tiles):
        atlas.alpha_composite(t, ((i % 8) * 16, (i // 8) * 16))
    return atlas


def hud_overlay() -> None:
    (ASSETS / "ui").mkdir(parents=True, exist_ok=True)
    (ASSETS / "ui/hud.local.ron").write_text(
        "// Surcharge locale de ui/hud.ron (packs Aymen B et Ittai Manero, NON redistribuables) : écrit par\n"
        "// scripts/repack-throne.py, gitignoré. Images dans sprites/postapo/.\n"
        "(\n"
        '    wall_art: Some((layer: "Walls", atlas: "sprites/postapo/atlas_murs.png", tile: 16, columns: 8, '
        'hide_layers: ["LevelConnection", "Walls"], floor: Some((tiles: [36, 37, 38, 39], mask_layer: None))))\n'
        ",\n)\n")


def main() -> None:
    home = Path.home() / "Downloads"
    aymen_zip = Path(sys.argv[1]) if len(sys.argv) > 1 else home / "UPDATE 2_ZombiePack.zip"
    ittai_zip = Path(sys.argv[2]) if len(sys.argv) > 2 else home / "Zombie Apocalypse Tileset.zip"
    aymen = Pack(aymen_zip, AYMEN_ROOT)
    ittai = Pack(ittai_zip, ITTAI_ROOT)
    OUT.mkdir(parents=True, exist_ok=True)
    lines: list[str] = []
    lines += skins(aymen)
    lines += turret(ittai)
    lines += dummies(ittai)
    lines += weapons(aymen)
    lines += projectiles(aymen, ittai)
    lines += objects(aymen)
    wall_atlas(aymen).save(OUT / "atlas_murs.png")
    (ASSETS / "sprites/sprites.local.ron").write_text(
        "// Surcharge locale (packs Aymen B et Ittai Manero, NON redistribuables) : écrit par\n"
        "// scripts/repack-throne.py, gitignoré. Remplace ou ajoute des ids à sprites.ron.\n"
        "({\n" + "\n".join(lines) + "\n})\n")
    hud_overlay()
    print(f"{len(lines)} entrées, {len(list(OUT.glob('*.png')))} images dans {OUT}")


if __name__ == "__main__":
    main()
