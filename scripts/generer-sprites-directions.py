#!/usr/bin/env python3
"""Génère la planche de test à 8 directions de `games/testbed` (X-4a, docs/conventions.md §89).

Image originale du projet (dessinée ici, aucune source tierce) : déterministe, placée dans
`games/testbed/assets/sprites/characters/boussole/`, enregistrée en CC0 dans `assets.yaml`.

Format visé (une planche par animation) : une ligne par direction dans l'ordre
`E, NE, N, NW, W, SW, S, SE` (indice = angle / 45°), une colonne par image, cellules de 160x160
(la taille d'un humain de planche « grand sprite », réduit au rendu par `scale`). Chaque cellule
porte une flèche épaisse vers la direction de sa ligne, la couleur de la ligne, l'étiquette du
compas et autant de points que le numéro de l'image : on lit à l'écran la ligne choisie et l'image jouée.

    python3 scripts/generer-sprites-directions.py      # réécrit le PNG (octet pour octet)
"""
import colorsys
import math
import os

from PIL import Image, ImageDraw, ImageFont

OUT = os.path.join(
    os.path.dirname(__file__),
    "..",
    "games",
    "testbed",
    "assets",
    "sprites",
    "characters",
    "boussole",
)
CELL = 160
COLUMNS = 4
LABELS = ["E", "NE", "N", "NW", "W", "SW", "S", "SE"]


def cell(row, col):
    img = Image.new("RGBA", (CELL, CELL), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    r, g, b = (int(255 * c) for c in colorsys.hsv_to_rgb(row / 8.0, 0.75, 0.95))
    dark = (20, 20, 28, 255)
    fill = (r, g, b, 255)
    cx, cy = CELL / 2, 68  # centre du disque ; la bande du bas (y > 128) porte l'étiquette et les points
    # Disque de fond : la silhouette du « personnage ».
    d.ellipse((cx - 60, cy - 60, cx + 60, cy + 60), fill=(235, 235, 240, 255), outline=dark, width=6)
    # Flèche vers la direction de la ligne (y de l'image vers le bas : le nord est en haut).
    theta = math.radians(45 * row)
    ux, uy = math.cos(theta), -math.sin(theta)
    px, py = -uy, ux
    wobble = (0, 4, 0, -4)[col % 4]  # l'image jouée se voit aussi au mouvement de la flèche
    tail, tip, head, half_shaft, half_head = -38, 46 + wobble, 24, 10, 28

    def pt(along, across):
        return (cx + ux * along + px * across, cy + uy * along + py * across)

    arrow = [
        pt(tail, -half_shaft),
        pt(tip - head, -half_shaft),
        pt(tip - head, -half_head),
        pt(tip, 0),
        pt(tip - head, half_head),
        pt(tip - head, half_shaft),
        pt(tail, half_shaft),
    ]
    d.polygon(arrow, fill=fill, outline=dark, width=5)
    # Bande du bas : étiquette du compas à gauche, un point par image écoulée (1 à 4) à droite.
    font = ImageFont.load_default(size=34)
    d.text((40, 146), LABELS[row], font=font, fill=dark, anchor="mm")
    for i in range(col + 1):
        x = 84 + 18 * i
        d.ellipse((x - 6, 140, x + 6, 152), fill=dark)
    return img


def main():
    os.makedirs(OUT, exist_ok=True)
    sheet = Image.new("RGBA", (CELL * COLUMNS, CELL * 8), (0, 0, 0, 0))
    for row in range(8):
        for col in range(COLUMNS):
            sheet.paste(cell(row, col), (col * CELL, row * CELL))
    sheet.save(os.path.join(OUT, "boussole.png"), optimize=False)


if __name__ == "__main__":
    main()
