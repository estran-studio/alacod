#!/usr/bin/env python3
"""Murs, portes et objets du monde de `games/isaac` (fiche m3-salles-murs-isaac, docs/conventions.md §83 et §40).

Même générateur que `scripts/generer-sprites-gungeon.py` (importé : primitives de dessin, encodeur PNG, atlas, murs auto-tuilés, objets du monde) ;
ne change que la **palette** (« sous-sol » : briques brun-rouge, sol de terre claire, distincte du pourpre de gungeon) et ce que isaac pose :
portes de 4 cases (16 x 64 et 64 x 16, états `Idle`, `Open`, `Locked`, `Secret` : le mur fissuré d'une salle secrète), rochers (et rocher à butin),
piques (tuile de 16 px de l'atlas des murs, indice 36), sous, coffre, présentoir, bombe posée, objets au sol (`item/<id>`, d'après `objets/*.ron`),
portail. Images **originales du projet** (primitives, aucune source tierce), CC0, déterministes. Les personnages gardent leurs sprites Kenney.

    python3 scripts/generer-sprites-isaac.py            # réécrit PNG, feuilles, animations et le bloc `// >>> généré` de sprites.ron
    python3 scripts/generer-sprites-isaac.py --check    # échoue si un fichier généré diffère du disque
    python3 scripts/generer-sprites-isaac.py --preview /tmp/apercu.png   # planche de contrôle (x2)

Sorties, sous `games/isaac/assets/sprites/lisible/` : atlas_monde.png (32 px), atlas_portail.png (64 px), atlas_porte_v.png (16 x 64),
atlas_porte_h.png (64 x 16), atlas_murs.png (tuiles de 16 px : 36 de mur + 1 de piques), feuilles `sheet_*.ron` et `anim/<id>.ron`.
"""
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("generer_sprites_gungeon", os.path.join(HERE, "generer-sprites-gungeon.py"))
g = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(g)

ROOT = os.path.join(HERE, "..")
ASSETS = os.path.join(ROOT, "games", "isaac", "assets")
OUT = os.path.join(ASSETS, "sprites", "lisible")
SPRITES_RON = os.path.join(ASSETS, "sprites", "sprites.ron")
BEGIN = "    // >>> généré par scripts/generer-sprites-isaac.py (ne pas éditer à la main)"
END = "    // <<< fin du bloc généré"
# Les fonctions de dessin de gungeon lisent ces noms dans leur module : on les remplace avant tout appel.
g.ASSETS = ASSETS
Img, rgb, shade = g.Img, g.rgb, g.shade

# Palette « sous-sol » : briques brun-rouge sur un sol de terre claire (`#7B6A5F`, `make_gabarits_grands.py`).
g.STONE = rgb("#4e3228")        # face du mur : briques
g.STONE_HI = rgb("#8a5a47")     # arête haute de la face
g.MORTAR = rgb("#2e1c15")       # joints
g.STONE_DARK = rgb("#1f130e")   # pied de mur
g.CAP = rgb("#34231d")          # dessus du mur
g.CAP_HI = rgb("#6b4a3c")       # liseré clair des bords qui touchent le sol
g.CAP_EDGE = rgb("#120a07")     # contour sombre
STONE, STONE_HI, MORTAR, STONE_DARK, CAP = g.STONE, g.STONE_HI, g.MORTAR, g.STONE_DARK, g.CAP
BLACK, GOLD, CLEAR = g.BLACK, g.GOLD, g.CLEAR

OPENING = 4                      # cases d'une ouverture (64 px)
DOOR_LEN = OPENING * 16
SPIKES_TILE = 36                 # indice des piques dans `atlas_murs.png` (après les 36 tuiles de mur)
HEADER = "scripts/generer-sprites-isaac.py"


# ---------------------------------------------------------------------------------------------
# Murs et piques (tuiles de 16 px)
# ---------------------------------------------------------------------------------------------
def spikes_tile():
    """Piques : plaque de métal sombre, quatre pointes rouges (une case de 16 px ; les cases voisines se raccordent)."""
    im = Img(16)
    im.rect(0, 0, 16, 16, rgb("#4a4650"))
    im.rect(0, 0, 16, 1, rgb("#6a6572"))
    im.rect(0, 15, 16, 16, rgb("#2a2730"))
    for gx in range(2):
        for gy in range(2):
            x, y = 1 + gx * 8, 1 + gy * 8
            im.poly([(x, y + 6.5), (x + 3, y), (x + 6, y + 6.5)], rgb("#e0453a"))
            im.rect(x + 2, y + 2, x + 3, y + 5, rgb("#ffd6d0"))
            im.rect(x, y + 6, x + 6, y + 7, rgb("#8f241c"))
    return im


def wall_atlas():
    tiles = [g.wall_cap(i % 8, i // 8) for i in range(32)]
    tiles += [g.wall_face(bool(i & 1), bool(i & 2)) for i in range(4)]
    assert len(tiles) == SPIKES_TILE == g.WALL_TILES
    tiles.append(spikes_tile())
    rows = (len(tiles) + g.WALL_COLUMNS - 1) // g.WALL_COLUMNS
    assert rows == (g.WALL_TILES + g.WALL_COLUMNS - 1) // g.WALL_COLUMNS, "la feuille déborde les lignes que lit `wall_art`"
    im = Img(g.WALL_COLUMNS * g.WALL_TILE, rows * g.WALL_TILE)
    for i, tile in enumerate(tiles):
        im.blit(tile, (i % g.WALL_COLUMNS) * g.WALL_TILE, (i // g.WALL_COLUMNS) * g.WALL_TILE)
    return im


# ---------------------------------------------------------------------------------------------
# Portes de quatre cases (vertical : 16 x 64 ; horizontal : transposée)
# ---------------------------------------------------------------------------------------------
def door_v(state):
    """Porte dans un mur vertical. Jambages de pierre aux deux bouts (couleurs de `wall_face`) ; `Idle` : vantail de planches cerclé de fer ;
    `Open` : seuls les jambages, le sol reste visible ; `Locked` : planches, chaînes croisées et cadenas rouge ; `Secret` : tronçon de mur de
    briques fissuré (une bombe l'ouvre)."""
    n = DOOR_LEN
    im = Img(16, n)
    wood, iron = rgb("#7a4f2a"), rgb("#8d949c")

    def jambs():
        for y0, y1 in ((0, 4), (n - 4, n)):
            im.rect(0, y0, 16, y1, STONE)
            im.rect(0, y0, 16, y0 + 1, STONE_HI)
            im.rect(0, y1 - 1, 16, y1, STONE_DARK)
            im.rect(7, y0 + 1, 8, y1 - 1, MORTAR)
        im.outline()

    if state == "Open":
        jambs()
        return im
    if state == "Secret":
        im.rect(0, 0, 16, n, STONE)
        for i, y in enumerate(range(8, n, 8)):  # assises de briques, comme le mur
            im.rect(0, y, 16, y + 1, MORTAR)
            for x in range(4 if i % 2 else 0, 16, 8):
                im.rect(x, y - 7, x + 1, y, MORTAR)
        im.rect(0, 0, 16, 2, STONE_HI)
        im.rect(0, n - 2, 16, n, STONE_DARK)
        crack = rgb("#ffe9a8")
        pts = [(6, 3), (10, 14), (5, 24), (11, 34), (6, 45), (10, 56), (8, 61)]
        for a, b in zip(pts, pts[1:]):
            im.line(a[0], a[1], b[0], b[1], crack, 1.3)
        for y in (14, 34, 56):  # éclats autour de la fissure
            im.set(12, y + 1, crack)
            im.set(3, y - 3, crack)
        im.outline()
        return im
    im.rect(2, 4, 14, n - 4, wood)
    for x in (5, 9):
        im.rect(x, 4, x + 1, n - 4, shade(wood, 0.6))
    im.rect(2, 4, 14, 6, shade(wood, 1.3))
    for y in (12, n // 2 - 2, n - 16):
        im.rect(2, y, 14, y + 3, iron)
        im.rect(2, y, 14, y + 1, shade(iron, 1.4))
        im.rect(3, y + 1, 4, y + 2, BLACK)
        im.rect(12, y + 1, 13, y + 2, BLACK)
    if state == "Locked":
        im.line(2, 6, 14, n - 6, rgb("#aab3bd"), 1.4)
        im.line(14, 6, 2, n - 6, rgb("#aab3bd"), 1.4)
        c = n // 2
        im.rect(4, c - 6, 12, c + 6, rgb("#d62f3a"))
        im.rect(6, c - 11, 10, c - 5, rgb("#aab3bd"))
        im.rect(7, c - 10, 9, c - 5, CLEAR)
        im.rect(7, c - 2, 9, c + 3, BLACK)
    else:
        im.rect(11, n // 2 - 5, 13, n // 2 - 1, GOLD)
    jambs()
    return im


# ---------------------------------------------------------------------------------------------
# Monde (32 x 32)
# ---------------------------------------------------------------------------------------------
def rock(loot=False):
    """Rocher : bloc gris à facettes, **18 x 16 px au centre** (un rocher occupe une case de 16 unités). À butin : veine dorée."""
    im = Img(32)
    c = rgb("#8f8c88")
    im.poly([(7, 24), (8, 14), (12, 9), (19, 9), (24, 14), (25, 24), (21, 26), (11, 26)], c)
    im.poly([(10, 14), (13, 10), (18, 11), (15, 17)], shade(c, 1.35))
    im.poly([(15, 17), (19, 10), (23, 15), (23, 22)], shade(c, 1.12))
    im.line(14, 18, 19, 25, shade(c, 0.5))
    if loot:
        gold = rgb("#f4c430")
        im.line(10, 21, 15, 14, gold, 1.6)
        im.line(15, 14, 22, 19, gold, 1.6)
        im.rect(14, 15, 16, 17, rgb("#fff3a8"))
    return im.outline()


def world_sprites():
    """[(id, {animation: Img})] du monde de isaac. Les ids `item/<id>` sont ceux des objets du jeu (`objets/*.ron`)."""
    out = [
        ("shell", {"Idle": g.coin()}),
        ("chest", {"Idle": g.chest(False), "Open": g.chest(True)}),
        ("shelf", {"Idle": g.shelf()}),
        ("rock", {"Idle": rock()}),
        ("rock_loot", {"Idle": rock(True)}),
        ("bomb", {"Idle": g.bomb()}),  # bombe posée (entité `Bomb`, mèche qui clignote)
        ("item/passive", {"Idle": g.card("?", rgb("#d8dde3"), rgb("#7f8c9a"))}),
        ("item/active", {"Idle": g.active_gem("?", rgb("#c39bd3"), rgb("#6c3483"))}),
        ("item/consumable", {"Idle": g.potion("?", rgb("#58d68d"))}),
    ]
    heart = {
        "coeur": "#e63946", "coeur_ame": "#74b9ff", "eclat_ame": "#a9d6ff", "festin": "#ff9f43", "pain": "#e0a458", "ration": "#ff7f50",
    }
    for iid, name, kind, rarity in g.read_items():
        letter, base = g.initial(name), g.hue(iid)
        border = rgb(g.RARITY_BORDER.get(rarity, "#bfc9d4"))
        if iid in heart:
            out.append(("item/" + iid, {"Idle": g.heart(rgb(heart[iid]))}))
        elif iid == "key":
            out.append(("item/key", {"Idle": g.key()}))
        elif iid == "bomb":
            out.append(("item/bomb", {"Idle": g.bomb()}))
        elif iid == "blank":
            out.append(("item/blank", {"Idle": g.blank()}))
        elif kind == "Active":
            out.append(("item/" + iid, {"Idle": g.active_gem(letter, base, border)}))
        elif kind == "Consumable":
            out.append(("item/" + iid, {"Idle": g.potion(letter, base)}))
        else:
            out.append(("item/" + iid, {"Idle": g.card(letter, base, border)}))
    return out


# ---------------------------------------------------------------------------------------------
# Sorties
# ---------------------------------------------------------------------------------------------
def regen(text):
    return text.replace("scripts/generer-sprites-gungeon.py", HEADER)


def build():
    """{chemin relatif à OUT: octets} de tout ce que le script écrit (sans sprites.ron), et les atlas (hors murs)."""
    files = {}
    monde = g.Atlas("atlas_monde", 32, 8, 1.0, 0.0, 3.4)  # au-dessus des surfaces de la carte (IntGrid)
    for sid, anims in world_sprites():
        monde.add(sid, anims)
    portal = g.Atlas("atlas_portail", 64, 1, 1.0, 0.0, 5.0)
    portal.add("portal", {"Idle": g.portal()})
    states = ("Idle", "Open", "Locked", "Secret")
    porte_v = g.Atlas("atlas_porte_v", (16, DOOR_LEN), 1, 1.0, 0.0, 3.4)
    porte_v.add("door_v", {s: door_v(s) for s in states})
    porte_h = g.Atlas("atlas_porte_h", (DOOR_LEN, 16), 1, 1.0, 0.0, 3.4)
    porte_h.add("door_h", {s: g.transpose(door_v(s)) for s in states})
    atlases = [monde, portal, porte_v, porte_h]
    for a in atlases:
        im, rows = a.image()
        files[a.name + ".png"] = g.png_bytes(im)
        files["sheet_" + a.name.split("_", 1)[1] + ".ron"] = regen(a.sheet_ron(rows)).encode()
        for sid, idx in a.entries:
            files[f"anim/{g.file_name(sid)}.ron"] = regen(g.anim_ron(sid, idx)).encode()
    files["atlas_murs.png"] = g.png_bytes(wall_atlas())
    return files, atlases


def sprites_block(atlases):
    lines = [BEGIN]
    for a in atlases:
        for sid, _ in a.entries:
            lines.append(
                f'    "{sid}": (animation: "sprites/lisible/anim/{g.file_name(sid)}.ron", '
                f'layers: {{ "body": "sprites/lisible/{g.SHEET_OF[a.name]}.ron" }}),'
            )
    lines.append(END)
    return "\n".join(lines) + "\n"


def splice(text, block):
    if BEGIN in text:
        head, rest = text.split(BEGIN, 1)
        _, tail = rest.split(END + "\n", 1)
        return head + block + tail
    marker = '    "pistol": ('  # première fois : avant l'arme à distance
    assert marker in text, "sprites.ron : repère « pistol » introuvable"
    head, tail = text.split(marker, 1)
    return head + block + marker + tail


def preview(path, atlases):
    cols, cell = 12, 140
    cells = [(sid, a.tiles[min(idx.values())]) for a in atlases for sid, idx in a.entries]
    rows = (len(cells) + cols - 1) // cols
    im = Img(cols * cell, rows * cell + 160)
    im.rect(0, 0, im.w, im.h, rgb("#7B6A5F"))
    for i, (sid, t) in enumerate(cells):
        ox, oy = (i % cols) * cell + 6, (i // cols) * cell + 6
        s = 2 if max(t.w, t.h) <= 32 else 1
        for y in range(t.h):
            for x in range(t.w):
                c = t.get(x, y)
                if c[3]:
                    bg = im.get(ox + x * s, oy + y * s)
                    al = c[3] / 255.0
                    col = tuple(int(c[k] * al + bg[k] * (1 - al)) for k in range(3)) + (255,)
                    im.rect(ox + x * s, oy + y * s, ox + (x + 1) * s, oy + (y + 1) * s, col)
        im.text(sid.split("/")[-1][:14], ox, oy + 128, g.WHITE, 1)
    wa = wall_atlas()
    for y in range(wa.h):
        for x in range(wa.w):
            c = wa.get(x, y)
            if c[3]:
                im.rect(8 + x * 2, rows * cell + 8 + y * 2, 8 + x * 2 + 2, rows * cell + 8 + y * 2 + 2, c)
    open(path, "wb").write(g.png_bytes(im))


def main(argv):
    files, atlases = build()
    block = sprites_block(atlases)
    text = open(SPRITES_RON, encoding="utf-8").read()
    new_text = splice(text, block)
    if "--preview" in argv:
        preview(argv[argv.index("--preview") + 1], atlases)
        return 0
    if "--check" in argv:
        bad = []
        for rel, data in sorted(files.items()):
            p = os.path.join(OUT, rel)
            if not os.path.exists(p) or open(p, "rb").read() != data:
                bad.append(rel)
        for folder, _, names in os.walk(OUT):
            for name in names:
                rel = os.path.relpath(os.path.join(folder, name), OUT)
                if rel not in files:
                    bad.append(rel + " (orphelin)")
        if new_text != text:
            bad.append("sprites.ron")
        if bad:
            print("générés périmés :", ", ".join(bad))
            return 1
        print("ok : sprites de isaac à jour (%d fichiers)" % (len(files) + 1))
        return 0
    for rel, data in files.items():
        p = os.path.join(OUT, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        open(p, "wb").write(data)
    for folder, _, names in os.walk(OUT):
        for name in names:
            rel = os.path.relpath(os.path.join(folder, name), OUT)
            if rel not in files:
                os.remove(os.path.join(folder, name))
    if new_text != text:
        open(SPRITES_RON, "w", encoding="utf-8").write(new_text)
    print("ok : %d fichiers dans %s" % (len(files), os.path.relpath(OUT, ROOT)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
