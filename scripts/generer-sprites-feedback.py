#!/usr/bin/env python3
"""Génère les sprites provisoires du feedback v3 de `games/testbed` (M3-T17, docs/conventions.md §82).

Images originales du projet (dessinées ici, à la main, aucune source tierce) : déterministes,
placées dans `games/testbed/assets/sprites/feedback/`, enregistrées en CC0 dans `assets.yaml`.
Ce sont des **placeholders** de lisibilité (forme, orientation, teinte) à remplacer par la voie V6.

    python3 scripts/generer-sprites-feedback.py      # réécrit les PNG (octet pour octet)

Les images sont en niveaux de blanc/gris : la couleur est posée au rendu (teinte de la larme, de la
mèche, du champion), un sprite sombre ne montrerait aucune teinte.
"""
import math
import os

from PIL import Image, ImageDraw

OUT = os.path.join(os.path.dirname(__file__), "..", "games", "testbed", "assets", "sprites", "feedback")


def save(img, name):
    os.makedirs(OUT, exist_ok=True)
    img.save(os.path.join(OUT, name), optimize=False)


def tear():
    """Halo blanc (64x64) : disque plein, bord doux ; teinté par équipe et par modificateur."""
    size = 64
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()
    c = (size - 1) / 2
    for y in range(size):
        for x in range(size):
            d = math.hypot(x - c, y - c) / c
            if d <= 1.0:
                a = 1.0 if d < 0.45 else max(0.0, 1.0 - (d - 0.45) / 0.55) ** 1.6
                px[x, y] = (255, 255, 255, int(255 * a))
    save(img, "tear.png")


def bomb():
    """Bombe 16x16 : corps rond gris clair, reflet, mèche qui dépasse en haut à droite."""
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse((2, 4, 13, 15), fill=(175, 175, 182, 255), outline=(20, 20, 24, 255))
    d.ellipse((4, 6, 6, 8), fill=(250, 250, 255, 255))
    d.line((9, 4, 11, 1), fill=(150, 110, 60, 255), width=1)
    d.point((12, 0), fill=(255, 255, 255, 255))
    d.point((11, 1), fill=(255, 255, 255, 255))
    save(img, "bomb.png")


def rock():
    """Rocher 16x16 : bloc gris à facettes."""
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.polygon([(1, 12), (2, 5), (6, 2), (11, 2), (14, 6), (15, 12), (12, 15), (4, 15)],
              fill=(128, 122, 114, 255), outline=(60, 56, 52, 255))
    d.polygon([(3, 6), (6, 3), (9, 4), (7, 8)], fill=(170, 164, 154, 255))
    d.line((9, 9, 13, 12), fill=(90, 86, 80, 255))
    save(img, "rock.png")


def companion():
    """Suiveur 16x16 : petit corps rond, deux yeux et un bec à droite (regarde vers +x)."""
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse((2, 3, 12, 13), fill=(235, 235, 235, 255), outline=(60, 60, 70, 255))
    d.polygon([(12, 7), (15, 8), (12, 10)], fill=(255, 190, 60, 255))
    d.rectangle((8, 5, 9, 7), fill=(30, 30, 40, 255))
    d.point((4, 12), fill=(150, 150, 160, 255))
    save(img, "companion.png")


def orbital():
    """Orbital 16x16 : disque avec une pointe à droite (la pointe regarde vers l'extérieur du cercle)."""
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse((1, 3, 11, 13), fill=(225, 235, 245, 255), outline=(50, 60, 90, 255))
    d.polygon([(10, 4), (15, 8), (10, 12)], fill=(255, 255, 255, 255), outline=(50, 60, 90, 255))
    d.ellipse((4, 6, 8, 10), fill=(120, 150, 210, 255))
    save(img, "orbital.png")


def champion():
    """Élite 16x16 : silhouette claire à cornes, que la teinte de la variante colore."""
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rectangle((3, 5, 12, 14), fill=(235, 235, 235, 255), outline=(50, 50, 55, 255))
    d.polygon([(3, 5), (2, 1), (6, 5)], fill=(255, 255, 255, 255), outline=(50, 50, 55, 255))
    d.polygon([(12, 5), (13, 1), (9, 5)], fill=(255, 255, 255, 255), outline=(50, 50, 55, 255))
    d.rectangle((5, 8, 6, 9), fill=(30, 30, 35, 255))
    d.rectangle((9, 8, 10, 9), fill=(30, 30, 35, 255))
    d.line((5, 12, 10, 12), fill=(30, 30, 35, 255))
    save(img, "champion.png")


if __name__ == "__main__":
    tear()
    bomb()
    rock()
    companion()
    orbital()
    champion()
    print("ok :", ", ".join(sorted(os.listdir(OUT))))
