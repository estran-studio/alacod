#!/usr/bin/env python3
"""Langage visuel provisoire de `games/gungeon` (fiche m2-lisibilite-gungeon, docs/conventions.md §83).

Images **originales du projet** (dessinées ici par primitives, aucune source tierce), déterministes :
une silhouette ET une couleur par classe d'entité, un halo pour les ramassables. Ce sont des
**placeholders de lisibilité**, remplaçables fichier par fichier par la voie V6 (assets définitifs) ;
CC0 (voir `assets.yaml`). Aucune dépendance : bibliothèque standard seulement (encodeur PNG intégré).

    python3 scripts/generer-sprites-gungeon.py            # réécrit PNG, feuilles, animations, sprites.ron
    python3 scripts/generer-sprites-gungeon.py --check    # échoue si un fichier généré diffère du disque
    python3 scripts/generer-sprites-gungeon.py --preview /tmp/apercu.png   # planche de contrôle (x3)

Sorties, sous `games/gungeon/assets/sprites/lisible/` :
  atlas_persos.png / sheet_persos.ron   personnages (32 px, un calque `body`)
  atlas_boss.png   / sheet_boss.ron     boss (64 px)
  atlas_monde.png  / sheet_monde.ron    objets du monde, ramassables, halo, pointeur de visée (32 px)
  bullet_player.png, bullet_enemy.png   projectiles (32 px, image seule : teintée au rendu = blanc)
  anim/<id>.ron                         une animation par entrée (`Idle`, `Open`, `Run`, `Dying_1`)
et le bloc `// >>> généré` de `games/gungeon/assets/sprites/sprites.ron`.

Les images sont **claires et peu saturées quand la couleur vient du rendu** (joueur : teinte par
handle ; pointeur : teinte par équipe) : un sprite sombre ne montrerait aucune teinte.
"""
import os
import re
import struct
import sys
import zlib

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
ASSETS = os.path.join(ROOT, "games", "gungeon", "assets")
OUT = os.path.join(ASSETS, "sprites", "lisible")
SPRITES_RON = os.path.join(ASSETS, "sprites", "sprites.ron")
BEGIN = "    // >>> généré par scripts/generer-sprites-gungeon.py (ne pas éditer à la main)"
END = "    // <<< fin du bloc généré"

# ---------------------------------------------------------------------------------------------
# Couleurs
# ---------------------------------------------------------------------------------------------
OUTLINE = (22, 18, 30, 255)
CLEAR = (0, 0, 0, 0)


def rgb(h):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), 255)


def shade(c, f):
    """f < 1 assombrit, f > 1 éclaircit (vers le blanc)."""
    if f <= 1:
        return (int(c[0] * f), int(c[1] * f), int(c[2] * f), c[3])
    t = f - 1
    return (int(c[0] + (255 - c[0]) * t), int(c[1] + (255 - c[1]) * t), int(c[2] + (255 - c[2]) * t), c[3])


WHITE = rgb("#ffffff")
BLACK = rgb("#15121c")
BRASS = rgb("#e0a93b")
RED = rgb("#c0392b")
STEEL = rgb("#7f8c9a")
GOLD = rgb("#f4c430")
WOOD = rgb("#8a5a2b")


# ---------------------------------------------------------------------------------------------
# Toile : primitives remplies (centre de pixel), contour automatique
# ---------------------------------------------------------------------------------------------
class Img:
    def __init__(self, w, h=None):
        self.w, self.h = w, h or w
        self.px = [CLEAR] * (self.w * self.h)

    def set(self, x, y, c):
        if 0 <= x < self.w and 0 <= y < self.h:
            self.px[y * self.w + x] = c

    def get(self, x, y):
        if 0 <= x < self.w and 0 <= y < self.h:
            return self.px[y * self.w + x]
        return CLEAR

    def rect(self, x0, y0, x1, y1, c):
        """Pixels x0 <= x < x1, y0 <= y < y1."""
        for y in range(int(y0), int(y1)):
            for x in range(int(x0), int(x1)):
                self.set(x, y, c)

    def ellipse(self, cx, cy, rx, ry, c):
        for y in range(self.h):
            for x in range(self.w):
                dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
                if dx * dx + dy * dy <= 1.0:
                    self.set(x, y, c)

    def poly(self, pts, c):
        n = len(pts)
        for y in range(self.h):
            for x in range(self.w):
                px, py = x + 0.5, y + 0.5
                inside = False
                j = n - 1
                for i in range(n):
                    xi, yi = pts[i]
                    xj, yj = pts[j]
                    if (yi > py) != (yj > py) and px < (xj - xi) * (py - yi) / (yj - yi) + xi:
                        inside = not inside
                    j = i
                if inside:
                    self.set(x, y, c)

    def line(self, x0, y0, x1, y1, c, width=1.0):
        dx, dy = x1 - x0, y1 - y0
        l2 = dx * dx + dy * dy or 1.0
        for y in range(self.h):
            for x in range(self.w):
                t = max(0.0, min(1.0, ((x + 0.5 - x0) * dx + (y + 0.5 - y0) * dy) / l2))
                qx, qy = x0 + t * dx, y0 + t * dy
                if (x + 0.5 - qx) ** 2 + (y + 0.5 - qy) ** 2 <= (width / 2) ** 2:
                    self.set(x, y, c)

    def outline(self, c=OUTLINE):
        """Contour d'un pixel (4-voisins) autour de tout pixel opaque."""
        add = []
        for y in range(self.h):
            for x in range(self.w):
                if self.get(x, y)[3] == 0:
                    if any(self.get(x + dx, y + dy)[3] > 0 for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                        add.append((x, y))
        for x, y in add:
            self.set(x, y, c)
        return self

    def text(self, s, x, y, c, scale=1):
        for ch in s:
            glyph = FONT.get(ch.upper(), FONT["?"])
            for gy, row in enumerate(glyph):
                for gx, bit in enumerate(row):
                    if bit == "1":
                        self.rect(x + gx * scale, y + gy * scale, x + (gx + 1) * scale, y + (gy + 1) * scale, c)
            x += 4 * scale

    def blit(self, other, ox, oy):
        for y in range(other.h):
            for x in range(other.w):
                c = other.get(x, y)
                if c[3] > 0:
                    self.set(ox + x, oy + y, c)


FONT = {}
_GLYPHS = """
A 010 101 111 101 101
B 110 101 110 101 110
C 011 100 100 100 011
D 110 101 101 101 110
E 111 100 110 100 111
F 111 100 110 100 100
G 011 100 101 101 011
H 101 101 111 101 101
I 111 010 010 010 111
J 001 001 001 101 010
K 101 101 110 101 101
L 100 100 100 100 111
M 101 111 111 101 101
N 110 101 101 101 101
O 010 101 101 101 010
P 110 101 110 100 100
Q 010 101 101 111 011
R 110 101 110 101 101
S 011 100 010 001 110
T 111 010 010 010 010
U 101 101 101 101 111
V 101 101 101 101 010
W 101 101 111 111 101
X 101 101 010 101 101
Y 101 101 010 010 010
Z 111 001 010 100 111
0 111 101 101 101 111
1 010 110 010 010 111
2 110 001 010 100 111
3 110 001 010 001 110
4 101 101 111 001 001
5 111 100 110 001 110
6 011 100 111 101 111
7 111 001 010 010 010
8 111 101 111 101 111
9 111 101 111 001 110
+ 000 010 111 010 000
$ 011 110 010 011 110
? 110 001 010 000 010
! 010 010 010 000 010
"""
for _line in _GLYPHS.strip().splitlines():
    _p = _line.split()
    FONT[_p[0]] = _p[1:]


# ---------------------------------------------------------------------------------------------
# PNG (RGBA 8 bits, déterministe)
# ---------------------------------------------------------------------------------------------
def png_bytes(img):
    raw = bytearray()
    for y in range(img.h):
        raw.append(0)
        for x in range(img.w):
            raw.extend(img.px[y * img.w + x])

    def chunk(tag, data):
        body = tag + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    head = struct.pack(">IIBBBBB", img.w, img.h, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", head) + chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")


# ---------------------------------------------------------------------------------------------
# Personnages (32 x 32, regardent « de face » ; le pointeur de visée dit où ils tirent)
# ---------------------------------------------------------------------------------------------
def eyes(im, cx, y, gap=3, color=WHITE, pupil=BLACK, h=3, w=2):
    """Deux yeux : blanc w x h, pupille dans la colonne de droite (ils regardent un peu vers +x)."""
    for sx in (-1, 1):
        x = cx + sx * gap - w // 2
        im.rect(x, y, x + w, y + h, color)
        im.rect(x + w - 1, y + max(0, h - 2), x + w, y + h, pupil)


def perso_player():
    """Pistolero : grand chapeau, poncho triangulaire. Clair : la teinte par handle fait la couleur."""
    im = Img(32)
    im.poly([(16, 14), (5, 28), (27, 28)], rgb("#f4f4f4"))  # poncho
    im.poly([(16, 14), (10, 28), (22, 28)], rgb("#dcdcdc"))
    im.rect(11, 26, 15, 29, rgb("#9a9a9a"))  # bottes
    im.rect(17, 26, 21, 29, rgb("#9a9a9a"))
    im.ellipse(16, 11, 5.5, 5.5, rgb("#ffffff"))  # tête
    im.rect(11, 10, 21, 13, rgb("#bcbcbc"))  # visière sombre
    im.rect(13, 11, 15, 12, WHITE)
    im.rect(17, 11, 19, 12, WHITE)
    im.ellipse(16, 6.5, 11, 2.2, rgb("#c9c9c9"))  # bord du chapeau
    im.rect(11, 1, 21, 6, rgb("#b0b0b0"))  # calotte
    im.rect(11, 4, 21, 5, rgb("#ffffff"))
    return im.outline()


def perso_bullet_kin():
    """Balle de laiton : tête ronde, corps droit, bague sombre à la base."""
    im = Img(32)
    c = rgb("#e0a93b")
    im.ellipse(16, 11, 8, 8, c)
    im.rect(8, 11, 24, 27, c)
    im.rect(8, 23, 24, 27, shade(c, 0.72))
    im.rect(8, 22, 24, 23, shade(c, 0.55))
    im.rect(10, 8, 12, 14, shade(c, 1.35))  # reflet
    eyes(im, 16, 12, 4)
    im.rect(14, 18, 18, 19, BLACK)
    return im.outline()


def perso_squad():
    """Escouade : petite balle pointue, orange, trois traits blancs (elles viennent par trois)."""
    im = Img(32)
    c = rgb("#ee7b30")
    im.poly([(16, 6), (22, 14), (22, 27), (10, 27), (10, 14)], c)
    im.rect(10, 24, 22, 27, shade(c, 0.7))
    for i in range(3):
        im.rect(12 + i * 3, 17, 13 + i * 3, 21, WHITE)
    eyes(im, 16, 13, 2, h=2, w=1)
    return im.outline()


def perso_shotgun():
    """Cartouche de fusil : cylindre rouge large, culot de laiton, sommet serti."""
    im = Img(32)
    r = rgb("#c0392b")
    im.rect(6, 7, 26, 28, r)
    im.rect(6, 21, 26, 28, BRASS)
    im.rect(6, 20, 26, 21, shade(BRASS, 0.55))
    im.rect(6, 7, 26, 10, shade(r, 0.7))  # sertissage
    for x in range(8, 25, 4):
        im.rect(x, 7, x + 1, 10, shade(r, 0.45))
    im.rect(8, 11, 10, 19, shade(r, 1.3))
    eyes(im, 16, 12, 5, h=4, w=3)
    im.rect(12, 18, 20, 19, BLACK)
    return im.outline()


def perso_lanceur():
    """Lanceur : corps rond violet, tube de mortier sur la tête, mèche."""
    im = Img(32)
    v = rgb("#8e44ad")
    im.ellipse(16, 20, 10, 9, v)
    im.rect(12, 4, 20, 13, rgb("#3b3b4a"))
    im.rect(11, 3, 21, 6, rgb("#5a5a6e"))
    im.rect(14, 4, 18, 5, BLACK)
    im.poly([(24, 8), (27, 4), (26, 9)], rgb("#ffb347"))
    im.line(20, 10, 24, 8, rgb("#c9b37a"))
    im.rect(8, 16, 11, 22, shade(v, 1.35))
    eyes(im, 16, 18, 4, h=3)
    im.rect(14, 24, 18, 25, BLACK)
    return im.outline()


def perso_tourelle():
    """Tourelle (stationnaire) : cube d'acier sur trépied, lentille rouge. Le canon est le pointeur."""
    im = Img(32)
    s = rgb("#7f8c9a")
    im.poly([(8, 28), (13, 20), (19, 20), (24, 28), (21, 28), (16, 23), (11, 28)], shade(s, 0.7))
    im.rect(7, 9, 25, 22, s)
    im.rect(7, 9, 25, 12, shade(s, 1.3))
    im.rect(7, 19, 25, 22, shade(s, 0.65))
    for x, y in ((9, 11), (22, 11), (9, 19), (22, 19)):
        im.rect(x, y, x + 1, y + 1, shade(s, 0.45))
    im.ellipse(16, 15.5, 4.5, 4.5, rgb("#1c1c24"))
    im.ellipse(16, 15.5, 2.5, 2.5, rgb("#ff3b30"))
    im.rect(15, 14, 16, 15, rgb("#ffd0cc"))
    return im.outline()


def perso_veteran():
    """Vétéran : carrure large, casque à crête, chevrons jaunes. Vert sombre."""
    im = Img(32)
    g = rgb("#56803f")
    im.poly([(7, 10), (25, 10), (28, 28), (4, 28)], g)
    im.rect(4, 24, 28, 28, shade(g, 0.7))
    im.poly([(10, 3), (22, 3), (24, 12), (8, 12)], shade(g, 0.8))  # casque
    im.rect(15, 0, 17, 4, rgb("#c0392b"))  # crête
    im.rect(10, 8, 22, 10, BLACK)  # fente des yeux
    im.rect(12, 8, 14, 9, rgb("#ffe27a"))
    im.rect(18, 8, 20, 9, rgb("#ffe27a"))
    for dy in (0, 4):
        im.poly([(11, 15 + dy), (16, 19 + dy), (21, 15 + dy), (21, 17 + dy), (16, 21 + dy), (11, 17 + dy)], rgb("#f4d03f"))
    return im.outline()


def perso_bandana():
    """Bandana : corps turquoise élancé, bandeau rouge aux deux pans qui volent."""
    im = Img(32)
    t = rgb("#17b8a6")
    im.ellipse(16, 17, 7.5, 11, t)
    im.rect(8, 11, 24, 16, rgb("#d62f3a"))  # bandeau
    im.poly([(24, 12), (31, 8), (29, 13)], rgb("#d62f3a"))  # pans
    im.poly([(24, 14), (31, 17), (28, 12)], rgb("#a8222c"))
    im.rect(12, 12, 14, 14, WHITE)
    im.rect(18, 12, 20, 14, WHITE)
    im.rect(13, 13, 14, 14, BLACK)
    im.rect(19, 13, 20, 14, BLACK)
    im.rect(14, 21, 18, 22, BLACK)
    im.rect(10, 20, 12, 27, shade(t, 1.3))
    return im.outline()


def perso_fuyard():
    """Fuyard : petite goutte bleu pâle aux yeux ronds, traits de vitesse derrière, goutte de sueur."""
    im = Img(32)
    b = rgb("#a9d6f5")
    im.poly([(18, 7), (24, 18), (24, 24), (12, 24), (12, 18)], b)
    im.ellipse(18, 20, 6.5, 6, b)
    for i, y in enumerate((14, 19, 24)):
        im.line(3 + i, y, 9, y, WHITE)
    im.ellipse(17, 17, 2.6, 3, WHITE)
    im.ellipse(22, 17, 2.6, 3, WHITE)
    im.rect(17, 17, 19, 20, BLACK)
    im.rect(22, 17, 24, 20, BLACK)
    im.poly([(25, 6), (27, 10), (23, 10)], rgb("#5dade2"))
    return im.outline()


def perso_cible():
    """Cible d'entraînement : disque à anneaux rouge et blanc sur un pied."""
    im = Img(32)
    im.rect(14, 22, 18, 29, WOOD)
    im.rect(9, 27, 23, 30, shade(WOOD, 0.7))
    for r, c in ((11, rgb("#e8e8e8")), (8.5, rgb("#d62f3a")), (6, rgb("#e8e8e8")), (3.5, rgb("#d62f3a")), (1.5, rgb("#e8e8e8"))):
        im.ellipse(16, 13, r, r, c)
    return im.outline()


def perso_gatling():
    """Boss (64 x 64) : forteresse cramoisie, couronne de pointes dorées, yeux jaunes en colère."""
    im = Img(64)
    c = rgb("#8e2a22")
    im.poly([(10, 18), (54, 18), (60, 56), (4, 56)], c)
    im.poly([(10, 18), (54, 18), (50, 26), (14, 26)], shade(c, 1.25))
    im.rect(4, 48, 60, 56, shade(c, 0.65))
    for x in range(8, 57, 12):  # rivets
        im.rect(x, 50, x + 3, 53, shade(c, 1.5))
    for i, x in enumerate(range(12, 53, 8)):  # couronne
        im.poly([(x, 18), (x + 4, 4 if i % 2 == 0 else 9), (x + 8, 18)], GOLD)
    im.poly([(14, 30), (30, 34), (30, 40), (14, 38)], rgb("#ffe14a"))
    im.poly([(50, 30), (34, 34), (34, 40), (50, 38)], rgb("#ffe14a"))
    im.rect(20, 33, 24, 37, BLACK)
    im.rect(40, 33, 44, 37, BLACK)
    im.rect(22, 44, 42, 47, BLACK)
    for x in range(24, 41, 4):
        im.rect(x, 44, x + 2, 47, WHITE)
    return im.outline()


# ---------------------------------------------------------------------------------------------
# Monde et ramassables (32 x 32)
# ---------------------------------------------------------------------------------------------
def halo():
    """Disque blanc à bord doux, teinté par catégorie et pulsé par le rendu."""
    im = Img(32)
    for y in range(32):
        for x in range(32):
            d = (((x + 0.5 - 16) ** 2 + (y + 0.5 - 16) ** 2) ** 0.5) / 16.0
            if d <= 1.0:
                a = 1.0 if d < 0.4 else max(0.0, 1.0 - (d - 0.4) / 0.6) ** 1.5
                im.set(x, y, (255, 255, 255, int(255 * a)))
    return im


def aim():
    """Pointeur de visée : chevron blanc plein, pointe à droite (+x). Teinté par équipe, tourné vers la cible."""
    im = Img(32)
    im.poly([(20, 10), (30, 16), (20, 22), (23, 16)], WHITE)
    im.poly([(10, 10), (20, 16), (10, 22), (13, 16)], shade(WHITE, 0.82))
    return im.outline()


def coin():
    """Douille / sou : pièce jaune à rebord, trait vertical clair."""
    im = Img(32)
    im.ellipse(16, 16, 8, 8, shade(GOLD, 0.75))
    im.ellipse(16, 16, 6.2, 6.2, GOLD)
    im.rect(15, 11, 17, 21, shade(GOLD, 1.5))
    im.rect(13, 13, 19, 14, shade(GOLD, 1.5))
    im.rect(13, 18, 19, 19, shade(GOLD, 1.5))
    im.rect(11, 10, 13, 12, WHITE)
    return im.outline()


def key():
    """Clé : anneau, tige, deux dents. Or clair, en diagonale."""
    im = Img(32)
    k = rgb("#f7dc6f")
    im.ellipse(11, 11, 5, 5, k)
    im.ellipse(11, 11, 2.2, 2.2, CLEAR)
    im.line(14, 14, 25, 25, k, 3)
    im.rect(21, 25, 25, 28, k)
    im.rect(24, 22, 28, 25, k)
    im.line(8, 8, 11, 7, WHITE, 1)
    return im.outline()


def bomb():
    """Bombe : sphère noire, reflet, mèche et étincelle orange."""
    im = Img(32)
    im.ellipse(15, 19, 9, 9, rgb("#2b2b36"))
    im.ellipse(12, 16, 2.5, 2.5, rgb("#8a8a9a"))
    im.line(19, 12, 23, 7, rgb("#c9b37a"), 2)
    im.poly([(22, 3), (27, 6), (24, 8), (27, 11), (22, 8)], rgb("#ff9f1a"))
    im.rect(23, 5, 25, 7, rgb("#fff3a8"))
    return im.outline()


def heart(color=rgb("#e63946")):
    im = Img(32)
    im.ellipse(11.5, 12.5, 6, 6, color)
    im.ellipse(20.5, 12.5, 6, 6, color)
    im.poly([(5.5, 14), (26.5, 14), (16, 27)], color)
    im.rect(9, 9, 12, 11, shade(color, 1.6))
    return im.outline()


def blank():
    """Blank : disque bleu clair, anneau blanc, deux arcs d'onde."""
    im = Img(32)
    im.ellipse(16, 16, 10, 10, rgb("#5dade2"))
    im.ellipse(16, 16, 7.5, 7.5, rgb("#aed6f1"))
    im.ellipse(16, 16, 4.5, 4.5, rgb("#5dade2"))
    im.ellipse(16, 16, 2, 2, WHITE)
    return im.outline()


def card(letter, base, border):
    """Objet passif : carte portrait, bord de couleur, initiale sur fond clair."""
    im = Img(32)
    im.rect(8, 5, 24, 28, border)
    im.rect(10, 7, 22, 26, base)
    im.rect(10, 7, 22, 9, shade(base, 1.3))
    im.text(letter, 12, 12, shade(base, 0.35), 2)
    im.rect(21, 25, 24, 28, shade(border, 0.7))
    return im.outline()


def active_gem(letter, base, border):
    """Objet actif : hexagone saillant (se déclenche à la main), initiale au centre."""
    im = Img(32)
    pts = [(16, 3), (27, 9), (27, 22), (16, 29), (5, 22), (5, 9)]
    im.poly(pts, border)
    im.poly([(16, 6), (24, 11), (24, 20), (16, 26), (8, 20), (8, 11)], base)
    im.poly([(16, 6), (24, 11), (16, 14), (8, 11)], shade(base, 1.35))
    im.text(letter, 12, 13, shade(base, 0.35), 2)
    return im.outline()


def potion(letter, base):
    """Consommable : fiole ronde au goulot bouché, initiale sur le ventre."""
    im = Img(32)
    im.rect(13, 4, 19, 11, rgb("#d9c7a3"))
    im.rect(12, 3, 20, 6, rgb("#8a5a2b"))
    im.ellipse(16, 19, 9, 9, base)
    im.ellipse(12, 16, 2, 3, shade(base, 1.6))
    im.text(letter, 13, 16, shade(base, 0.3), 2)
    return im.outline()


def weapon_icon(kind, color):
    """Arme au sol : silhouette de famille, grande (24 px) et orange par défaut."""
    im = Img(32)
    d = shade(color, 0.6)
    if kind == "pistol":
        im.rect(5, 12, 22, 17, color)
        im.rect(5, 12, 22, 14, shade(color, 1.3))
        im.poly([(7, 17), (13, 17), (11, 27), (6, 27)], d)
        im.rect(22, 13, 27, 15, color)
    elif kind == "revolver":
        im.rect(6, 12, 25, 17, color)
        im.rect(14, 9, 20, 18, shade(color, 1.2))  # barillet vu de côté
        im.rect(15, 10, 16, 17, d)
        im.rect(18, 10, 19, 17, d)
        im.poly([(6, 17), (12, 17), (9, 28), (4, 28)], d)
        im.rect(25, 13, 30, 15, color)
    elif kind == "shotgun":
        im.rect(2, 12, 28, 16, color)
        im.rect(2, 16, 16, 19, shade(color, 0.8))
        im.poly([(2, 12), (2, 22), (9, 19), (9, 12)], d)
        im.rect(28, 11, 30, 17, shade(color, 1.3))
    elif kind == "rifle":
        im.rect(3, 13, 29, 17, color)
        im.rect(14, 17, 18, 23, d)
        im.poly([(3, 12), (3, 22), (10, 17), (10, 12)], d)
        im.rect(18, 10, 24, 13, shade(color, 1.3))  # lunette
        im.rect(28, 12, 31, 14, shade(color, 1.3))
    elif kind == "bow":
        im.line(9, 5, 9, 27, d, 2)
        im.poly([(9, 5), (22, 13), (22, 19), (9, 27), (13, 16)], color)
        im.line(10, 16, 29, 16, WHITE, 1)
        im.poly([(29, 16), (25, 13), (25, 19)], WHITE)
    elif kind == "energy":
        im.rect(4, 13, 20, 19, shade(color, 0.75))
        im.poly([(6, 19), (12, 19), (10, 28), (5, 28)], d)
        im.ellipse(23, 16, 6.5, 6.5, color)
        im.ellipse(23, 16, 3.2, 3.2, WHITE)
        for ang in ((23, 6), (23, 26), (29, 16)):
            im.line(23, 16, ang[0], ang[1], WHITE, 1)
    elif kind == "launcher":
        im.rect(3, 11, 25, 19, color)
        im.rect(3, 13, 25, 15, shade(color, 1.3))
        im.poly([(25, 9), (31, 15), (25, 21)], shade(color, 0.8))
        im.poly([(8, 19), (14, 19), (12, 28), (7, 28)], d)
    elif kind == "flame":
        im.rect(3, 13, 21, 18, color)
        im.poly([(21, 15), (30, 9), (28, 15), (30, 21)], rgb("#ff7a1a"))
        im.poly([(21, 15), (27, 12), (26, 15), (27, 18)], rgb("#ffe14a"))
        im.rect(6, 18, 10, 26, d)
    else:
        im.rect(4, 12, 26, 18, color)
        im.rect(8, 18, 12, 26, d)
    return im.outline()


def chest(open_=False):
    """Coffre à clé : bois, bandes d'or, cadenas. Ouvert : couvercle relevé, intérieur sombre, éclat."""
    im = Img(32)
    wood, band = rgb("#9a6232"), rgb("#f1c40f")
    if not open_:
        im.rect(4, 12, 28, 27, wood)
        im.poly([(4, 12), (7, 5), (25, 5), (28, 12)], shade(wood, 1.15))
        im.rect(4, 17, 28, 19, band)
        im.rect(7, 5, 9, 27, band)
        im.rect(23, 5, 25, 27, band)
        im.rect(13, 15, 19, 23, rgb("#bfc9d4"))  # cadenas
        im.rect(14, 12, 18, 16, rgb("#8794a3"))
        im.rect(15, 18, 17, 21, BLACK)
    else:
        im.rect(4, 15, 28, 27, wood)
        im.rect(5, 16, 27, 20, rgb("#2a1d12"))
        im.poly([(4, 15), (3, 5), (27, 3), (28, 14)], shade(wood, 1.1))
        im.rect(4, 22, 28, 24, band)
        im.rect(9, 13, 11, 15, rgb("#fff3a8"))
        im.rect(21, 12, 23, 14, rgb("#fff3a8"))
        im.rect(15, 11, 17, 13, rgb("#fff3a8"))
    return im.outline()


def shelf():
    """Présentoir : socle de pierre, plateau doré où l'objet est posé, étiquette de prix."""
    im = Img(32)
    stone = rgb("#9aa5b1")
    im.rect(8, 16, 24, 27, stone)
    im.rect(6, 26, 26, 30, shade(stone, 0.7))
    im.rect(5, 12, 27, 17, GOLD)
    im.rect(5, 12, 27, 14, shade(GOLD, 1.4))
    im.rect(11, 20, 21, 25, rgb("#f5f5f0"))  # étiquette
    im.text("$", 14, 20, rgb("#8a5a2b"), 1)
    return im.outline()


def barrel():
    """Baril explosif (cassable) : rouge, cerclages sombres, flamme jaune."""
    im = Img(32)
    r = rgb("#c0392b")
    im.ellipse(16, 16, 11, 13.5, r)
    im.rect(5, 6, 27, 26, r)
    im.ellipse(16, 5.5, 10.5, 3, shade(r, 1.25))
    im.rect(5, 9, 27, 11, shade(r, 0.5))
    im.rect(5, 21, 27, 23, shade(r, 0.5))
    im.poly([(16, 12), (20, 17), (18, 20), (16, 17), (14, 20), (12, 17)], rgb("#ffd23f"))
    im.rect(15, 16, 17, 19, rgb("#fff3a8"))
    return im.outline()


def table():
    """Table (couverture) : plateau de bois vu de dessus et de face, pieds."""
    im = Img(32)
    w = rgb("#a9713a")
    im.rect(3, 8, 29, 20, w)
    im.rect(3, 8, 29, 11, shade(w, 1.3))
    im.rect(3, 18, 29, 20, shade(w, 0.65))
    im.rect(5, 20, 8, 28, shade(w, 0.7))
    im.rect(24, 20, 27, 28, shade(w, 0.7))
    for x in range(7, 27, 5):
        im.rect(x, 13, x + 3, 14, shade(w, 0.8))
    return im.outline()


def rock():
    """Rocher : bloc gris à facettes."""
    im = Img(32)
    g = rgb("#8d8a85")
    # 18 x 16 px au centre : un rocher occupe une case de 16 unités.
    im.poly([(7, 24), (8, 14), (12, 9), (19, 9), (24, 14), (25, 24), (21, 26), (11, 26)], g)
    im.poly([(10, 14), (13, 10), (18, 11), (15, 17)], shade(g, 1.35))
    im.poly([(15, 17), (19, 10), (23, 15), (23, 22)], shade(g, 1.12))
    im.line(14, 18, 19, 25, shade(g, 0.5))
    return im.outline()


def spikes():
    """Piques : plaque de métal, quatre rangées de pointes rouges."""
    im = Img(32)
    im.rect(3, 6, 29, 28, rgb("#59606a"))
    for gx in range(4):
        for gy in range(3):
            x, y = 6 + gx * 6, 9 + gy * 6
            im.poly([(x, y + 5), (x + 2.5, y), (x + 5, y + 5)], rgb("#e74c3c"))
            im.rect(x + 2, y + 1, x + 3, y + 3, rgb("#ffd0cc"))
    return im.outline()


def secret_wall():
    """Mur fissuré : brique sombre, fissure claire (une bombe l'ouvre)."""
    im = Img(32)
    im.rect(2, 2, 30, 30, rgb("#6b5b73"))
    for y in range(2, 30, 7):
        im.rect(2, y, 30, y + 1, rgb("#3a2f40"))
    im.line(14, 3, 19, 12, rgb("#ffe9a8"), 1.2)
    im.line(19, 12, 13, 19, rgb("#ffe9a8"), 1.2)
    im.line(13, 19, 18, 29, rgb("#ffe9a8"), 1.2)
    return im.outline()


def door_v(state):
    """Porte (16 x 32, vue du dessus dans un mur vertical) : fermée = planches pleines ; ouverte = baie
    sombre entre deux montants ; verrouillée = planches, chaînes croisées et cadenas rouge."""
    im = Img(16, 32)
    wood = rgb("#8a5a2b")
    if state == "open":
        # Passage libre : deux montants et un linteau, le sol reste visible entre eux.
        im.rect(2, 1, 5, 31, wood)
        im.rect(11, 1, 14, 31, wood)
        im.rect(2, 1, 14, 4, shade(wood, 1.3))
        for y in range(8, 30, 6):
            im.rect(7, y, 9, y + 2, rgb("#d9c7a3"))
        return im.outline()
    im.rect(2, 1, 14, 31, wood)
    im.rect(5, 2, 6, 30, shade(wood, 0.6))
    im.rect(9, 2, 10, 30, shade(wood, 0.6))
    im.rect(2, 1, 14, 3, shade(wood, 1.3))
    im.rect(2, 8, 14, 10, shade(wood, 0.55))
    im.rect(2, 22, 14, 24, shade(wood, 0.55))
    if state == "locked":
        im.line(2, 2, 14, 30, rgb("#aab3bd"), 1.4)
        im.line(14, 2, 2, 30, rgb("#aab3bd"), 1.4)
        im.rect(4, 11, 12, 21, rgb("#d62f3a"))
        im.rect(6, 7, 10, 12, rgb("#aab3bd"))
        im.rect(7, 8, 9, 12, CLEAR)
        im.rect(7, 14, 9, 18, BLACK)
    else:
        im.rect(11, 14, 13, 17, GOLD)
    return im.outline()


def transpose(im):
    out = Img(im.h, im.w)
    for y in range(im.h):
        for x in range(im.w):
            out.set(y, x, im.get(x, y))
    return out


def portal():
    """Portail (64 x 64) : spirale violette et cyan sur disque sombre."""
    import math

    im = Img(64)
    for y in range(64):
        for x in range(64):
            dx, dy = x + 0.5 - 32, y + 0.5 - 32
            r = math.hypot(dx, dy)
            if r > 30:
                continue
            ang = math.atan2(dy, dx)
            phase = (ang / (2 * math.pi) * 3 + r / 7.0) % 1.0
            if r < 5:
                c = WHITE
            elif phase < 0.34:
                c = rgb("#b57bff")
            elif phase < 0.6:
                c = rgb("#5a2d9c")
            elif phase < 0.8:
                c = rgb("#5be6ff")
            else:
                c = rgb("#2a1458")
            im.set(x, y, c)
    return im.outline(rgb("#e9d8ff"))


def bullet(core, ring):
    """Projectile 32 x 32 : disque plein (le cœur remplit la toile), anneau contrasté, point clair."""
    im = Img(32)
    im.ellipse(16, 16, 15.5, 15.5, ring)
    im.ellipse(16, 16, 11, 11, core)
    im.ellipse(13, 13, 3.5, 3.5, WHITE)
    return im


def bullet_enemy():
    """Ennemi : orbe rouge à anneau sombre — toujours rond, toujours rouge/orange."""
    return bullet(rgb("#ff5a36"), rgb("#7a1410"))


def bullet_player():
    """Allié : losange blanc bleuté (forme différente de l'orbe ennemi : lisible sans la couleur)."""
    im = Img(32)
    im.poly([(16, 0.5), (31.5, 16), (16, 31.5), (0.5, 16)], rgb("#2c7bd6"))
    im.poly([(16, 5), (27, 16), (16, 27), (5, 16)], rgb("#e8f4ff"))
    im.poly([(16, 9), (22, 16), (16, 23), (10, 16)], WHITE)
    return im


# ---------------------------------------------------------------------------------------------
# Registre
# ---------------------------------------------------------------------------------------------
PERSOS = [
    ("player", perso_player),
    ("bullet_kin", perso_bullet_kin),
    ("bullet_kin_squad", perso_squad),
    ("shotgun_kin", perso_shotgun),
    ("lanceur_kin", perso_lanceur),
    ("tourelle_kin", perso_tourelle),
    ("veteran_kin", perso_veteran),
    ("bandana_kin", perso_bandana),
    ("fuyard_kin", perso_fuyard),
    ("cible", perso_cible),
]

WEAPON_KINDS = {
    "pistolet": "pistol", "pistolet_precis": "pistol", "pistolet_jumele": "pistol", "revolver": "revolver",
    "fusil_pompe": "shotgun", "double_canon": "shotgun",
    "mitraillette": "rifle", "carabine_rafale": "rifle", "mitrailleuse_lourde": "rifle", "fusil_precision": "rifle",
    "arc": "bow", "arbalete": "bow",
    "laser": "energy", "boule_energie": "energy", "chercheur": "energy", "rebondisseur": "energy",
    "lance_roquettes": "launcher", "grenadier": "launcher", "lance_flammes": "flame",
}
WEAPON_COLORS = {
    "pistol": "#f39c12", "revolver": "#e67e22", "shotgun": "#d35400", "rifle": "#f5b041",
    "bow": "#d4ac6e", "energy": "#48c9b0", "launcher": "#cb4335", "flame": "#f1948a",
}
# Couleurs d'objet : teinte stable par id (hachage), distincte d'un objet à l'autre.
RARITY_BORDER = {"Common": "#bfc9d4", "Uncommon": "#5dade2", "Rare": "#f1c40f"}


def hue(name):
    h = 0
    for ch in name:
        h = (h * 131 + ord(ch)) % 360
    import colorsys

    r, g, b = colorsys.hsv_to_rgb(h / 360.0, 0.45, 0.95)
    return (int(r * 255), int(g * 255), int(b * 255), 255)


def read_items():
    """(id, nom, genre, rareté) lus dans `objets/*.ron` (même source que le jeu)."""
    items = []
    folder = os.path.join(ASSETS, "objets")
    for fname in sorted(os.listdir(folder)):
        if not fname.endswith(".ron"):
            continue
        text = open(os.path.join(folder, fname), encoding="utf-8").read()
        name = re.search(r'name:\s*"([^"]+)"', text)
        kind = re.search(r"kind:\s*(\w+)", text)
        rarity = re.search(r"rarity:\s*(\w+)", text)
        items.append((fname[:-4], name.group(1) if name else fname[:-4], kind.group(1) if kind else "Passive",
                      rarity.group(1) if rarity else "Common"))
    return items


def initial(name):
    ch = name.strip()[0].upper()
    return {"É": "E", "È": "E", "Ê": "E", "À": "A", "Ç": "C"}.get(ch, ch)


def world_sprites():
    """[(id, {animation: Img})] du monde. Les ids `item/<id>` sont ceux des objets du jeu."""
    out = [
        ("halo", {"Idle": halo()}),
        ("aim", {"Idle": aim()}),
        ("shell", {"Idle": coin()}),
        ("chest", {"Idle": chest(False), "Open": chest(True)}),
        ("shelf", {"Idle": shelf()}),
        ("barrel", {"Idle": barrel()}),
        ("table", {"Idle": table()}),
        ("rock", {"Idle": rock()}),
        ("spikes", {"Idle": spikes()}),
        ("secret_wall", {"Idle": secret_wall()}),
        ("item/key", {"Idle": key()}),
        ("item/bomb", {"Idle": bomb()}),
        ("item/heart", {"Idle": heart()}),
        ("item/blank", {"Idle": blank()}),
        ("item/passive", {"Idle": card("?", rgb("#d8dde3"), rgb("#7f8c9a"))}),
        ("item/active", {"Idle": active_gem("?", rgb("#c39bd3"), rgb("#6c3483"))}),
        ("item/consumable", {"Idle": potion("?", rgb("#58d68d"))}),
        ("weapon", {"Idle": weapon_icon("pistol", rgb(WEAPON_COLORS["pistol"]))}),
    ]
    for wid, kind in sorted(WEAPON_KINDS.items()):
        out.append(("weapon/" + wid, {"Idle": weapon_icon(kind, rgb(WEAPON_COLORS[kind]))}))
    special = {"key", "blank"}
    for iid, name, kind, rarity in read_items():
        if iid in special:
            continue
        letter, base = initial(name), hue(iid)
        border = rgb(RARITY_BORDER.get(rarity, "#bfc9d4"))
        if iid == "ration":
            out.append(("item/" + iid, {"Idle": heart(rgb("#ff7f50"))}))
        elif kind == "Active":
            out.append(("item/" + iid, {"Idle": active_gem(letter, base, border)}))
        elif kind == "Consumable":
            out.append(("item/" + iid, {"Idle": potion(letter, base)}))
        else:
            out.append(("item/" + iid, {"Idle": card(letter, base, border)}))
    return out


# ---------------------------------------------------------------------------------------------
# Sorties
# ---------------------------------------------------------------------------------------------
class Atlas:
    def __init__(self, name, tile, columns, scale, offset_y, offset_z, anchor="Center", animated=False):
        self.animated = animated
        self.tw, self.th = tile if isinstance(tile, tuple) else (tile, tile)
        self.name, self.tile, self.columns = name, self.tw, columns
        self.scale, self.offset_y, self.offset_z, self.anchor = scale, offset_y, offset_z, anchor
        self.tiles = []  # Img
        self.entries = []  # (id, {anim: index})

    def add(self, sid, anims, character=False):
        idx = {}
        for anim, img in anims.items():
            assert (img.w, img.h) == (self.tw, self.th), (sid, img.w, img.h, self.tw, self.th)
            idx[anim] = len(self.tiles)
            self.tiles.append(img)
        if character:  # un personnage a Run et Dying_1 : la même image (pas d'animation dessinée)
            idx["Run"] = idx["Dying_1"] = idx["Idle"]
        self.entries.append((sid, idx))

    def image(self):
        rows = (len(self.tiles) + self.columns - 1) // self.columns
        im = Img(self.columns * self.tw, max(1, rows) * self.th)
        for i, t in enumerate(self.tiles):
            im.blit(t, (i % self.columns) * self.tw, (i // self.columns) * self.th)
        return im, rows

    def sheet_ron(self, rows):
        return (
            "// Généré par scripts/generer-sprites-gungeon.py : ne pas éditer à la main.\n"
            "(\n"
            f'    path: "sprites/lisible/{self.name}.png",\n'
            f"    tile_size: ({self.tw}, {self.th}),\n"
            f"    columns: {self.columns},\n"
            f"    rows: {rows},\n"
            f"    animated: {'true' if self.animated else 'false'},\n"
            '    name: "body",\n'
            f"    scale: {self.scale:.1f},\n"
            f"    anchor: {self.anchor},\n"
            "    offset_x: 0.0,\n"
            f"    offset_y: {self.offset_y:.1f},\n"
            f"    offset_z: {self.offset_z:.1f},\n"
            ")\n"
        )


def anim_ron(sid, idx):
    lines = [f"// Généré par scripts/generer-sprites-gungeon.py : image(s) de `{sid}` dans l'atlas.", "(", "    frame_duration: 120,", "    animations: {"]
    for name in sorted(idx):
        lines.append(f'        "{name}": (start: {idx[name]}, end: {idx[name]}),')
    lines += ["    },", ")", ""]
    return "\n".join(lines)


def file_name(sid):
    return sid.replace("/", "__")


def build():
    """{chemin relatif à OUT: octets} de tout ce que le script écrit (sans sprites.ron)."""
    files = {}
    persos = Atlas("atlas_persos", 32, 8, 1.0, 2.0, 1.0, animated=True)
    for sid, fn in PERSOS:
        persos.add(sid, {"Idle": fn()}, character=True)
    boss = Atlas("atlas_boss", 64, 4, 1.0, 6.0, 1.0, animated=True)
    boss.add("gatling", {"Idle": perso_gatling()}, character=True)
    monde = Atlas("atlas_monde", 32, 8, 1.0, 0.0, 0.5)
    portal_atlas = Atlas("atlas_portail", 64, 1, 1.0, 0.0, 5.0)
    portal_atlas.add("portal", {"Idle": portal()})
    for sid, anims in world_sprites():
        monde.add(sid, anims)
    # Portes : trois états (`Idle` fermée, `Open`, `Locked`) ; vertical 16 x 32, horizontal 32 x 16 (transposée).
    porte_v = Atlas("atlas_porte_v", (16, 32), 1, 1.0, 0.0, 0.6)
    porte_v.add("door_v", {"Idle": door_v("closed"), "Open": door_v("open"), "Locked": door_v("locked")})
    porte_h = Atlas("atlas_porte_h", (32, 16), 1, 1.0, 0.0, 0.6)
    porte_h.add("door_h", {k: transpose(v) for k, v in (("Idle", door_v("closed")), ("Open", door_v("open")), ("Locked", door_v("locked")))})
    atlases = [persos, boss, monde, portal_atlas, porte_v, porte_h]
    for a in atlases:
        im, rows = a.image()
        files[a.name + ".png"] = png_bytes(im)
        files["sheet_" + a.name.split("_", 1)[1] + ".ron"] = a.sheet_ron(rows).encode()
        for sid, idx in a.entries:
            files[f"anim/{file_name(sid)}.ron"] = anim_ron(sid, idx).encode()
    files["bullet_enemy.png"] = png_bytes(bullet_enemy())
    files["bullet_player.png"] = png_bytes(bullet_player())
    files["anim/bullet_enemy.ron"] = anim_ron("bullet_enemy", {"Idle": 0}).encode()
    files["anim/bullet_player.ron"] = anim_ron("bullet_player", {"Idle": 0}).encode()
    for who in ("enemy", "player"):
        files[f"sheet_bullet_{who}.ron"] = (
            "// Généré par scripts/generer-sprites-gungeon.py : ne pas éditer à la main.\n"
            "(\n"
            f'    path: "sprites/lisible/bullet_{who}.png",\n'
            "    tile_size: (32, 32),\n    columns: 1,\n    rows: 1,\n    animated: false,\n"
            '    name: "body",\n    scale: 1.0,\n    anchor: Center,\n    offset_x: 0.0,\n    offset_y: 0.0,\n    offset_z: 0.0,\n)\n'
        ).encode()
    return files, atlases


SHEET_OF = {
    "atlas_persos": "sheet_persos", "atlas_boss": "sheet_boss", "atlas_monde": "sheet_monde",
    "atlas_portail": "sheet_portail", "atlas_porte_v": "sheet_porte_v", "atlas_porte_h": "sheet_porte_h",
}


def sprites_block(atlases):
    lines = [BEGIN]
    for a in atlases:
        for sid, _ in a.entries:
            lines.append(
                f'    "{sid}": (animation: "sprites/lisible/anim/{file_name(sid)}.ron", '
                f'layers: {{ "body": "sprites/lisible/{SHEET_OF[a.name]}.ron" }}),'
            )
    for who in ("player", "enemy"):
        lines.append(
            f'    "bullet_{who}": (animation: "sprites/lisible/anim/bullet_{who}.ron", '
            f'layers: {{ "body": "sprites/lisible/sheet_bullet_{who}.ron" }}),'
        )
    lines.append(END)
    return "\n".join(lines) + "\n"


# Entrées de sprites.ron remplacées par le bloc généré (lot C, Kenney) : retirées à la main une fois.
def splice(text, block):
    if BEGIN in text:
        head, rest = text.split(BEGIN, 1)
        _, tail = rest.split(END + "\n", 1)
        return head + block + tail
    # première fois : le bloc va juste avant les armes à distance.
    marker = "    // Armes à distance"
    assert marker in text, "sprites.ron : repère « Armes à distance » introuvable"
    head, tail = text.split(marker, 1)
    return head + block + marker + tail


def preview(path, atlases):
    cols, cell = 12, 70
    cells = []
    for a in atlases:
        for sid, idx in a.entries:
            cells.append((sid, a.tiles[min(idx.values())]))
    rows = (len(cells) + cols - 1) // cols
    im = Img(cols * cell, rows * cell)
    im.rect(0, 0, im.w, im.h, rgb("#8a8f99"))
    for i, (sid, t) in enumerate(cells):
        ox, oy = (i % cols) * cell + 3, (i // cols) * cell + 3
        s = 2 if max(t.w, t.h) == 32 else 1
        for y in range(t.h):
            for x in range(t.w):
                c = t.get(x, y)
                if c[3]:
                    bg = im.get(ox + x * s, oy + y * s)
                    al = c[3] / 255.0
                    col = tuple(int(c[k] * al + bg[k] * (1 - al)) for k in range(3)) + (255,)
                    im.rect(ox + x * s, oy + y * s, ox + (x + 1) * s, oy + (y + 1) * s, col)
        im.text(sid.split("/")[-1][:14], ox, oy + 62, WHITE, 1)
    open(path, "wb").write(png_bytes(im))


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
        if new_text != text:
            bad.append("sprites.ron")
        if bad:
            print("générés périmés :", ", ".join(bad))
            return 1
        print("ok : sprites de gungeon à jour (%d fichiers)" % (len(files) + 1))
        return 0
    for rel, data in files.items():
        p = os.path.join(OUT, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        open(p, "wb").write(data)
    if new_text != text:
        open(SPRITES_RON, "w", encoding="utf-8").write(new_text)
    print("ok : %d fichiers dans %s" % (len(files), os.path.relpath(OUT, ROOT)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
