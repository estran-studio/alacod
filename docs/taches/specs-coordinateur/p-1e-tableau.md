# P-1e — tableau avant / après (banc, jeu avant, jeu après, écart restant) — 20 nov 2026

Méthode et vérifications (pas de temps, échelle, intégration) : `p-1e-tableau-avant.md`. Banc : `site/labo/armes/` défauts (`banc-grandeurs.txt`). Jeu avant : 1837 `1a3e639`
(= `f2ac3bf` + docs) sur engine `8f093d2f` (`jeu-avant-grandeurs.txt`, `jeu-avant-clips.txt`). Jeu après : même 1837, engine `98211f6c` (`jeu-apres-grandeurs.txt`,
`jeu-apres-clips.txt`). Mêmes gestes, même pilote (`alacod/input` frame par frame, session GGRS du jeu). Positions relevées à 0,1 u.

| Grandeur | Banc | Jeu avant | Jeu après | Écart restant |
|---|---|---|---|---|
| Latence appui → premier déplacement | 16,7 ms (1 image) | 100 ms (6 frames) | **16,7 ms (1 frame)** | 0 |
| Vitesse de marche, fusil porté | 4,400 m/s | 4,396 m/s (1,758 u/f) | 4,396 m/s | 0,1 % (arrondi fixe/relevé) |
| Accélération (une fois l'input appliqué) | échelon | pleine vitesse en 1 frame | pleine vitesse en 1 frame | 0 |
| Temps pour atteindre la vitesse (depuis l'appui) | 16,7 ms | 100 ms | **16,7 ms** | 0 |
| Freinage : glisse après le relâché | 0 cm | 36,7 cm (8,8 u) | **0 cm** | 0 |
| Freinage : temps d'arrêt | 16,7 ms | 83 ms de glisse | **arrêt à la frame suivante** | 0 |
| Demi-tour : temps pour inverser | 16,7 ms | 100 ms | **16,7 ms** | 0 |
| Demi-tour : dépassement | 0 cm | 36,7 cm | **0 cm** | 0 |
| Marche en joue (× 0,6) | 2,640 m/s | 2,641 m/s | 2,646 m/s (fenêtre 7,62-8,03 s) | 0 |
| Marche en rechargeant (× 0,5) | 2,200 m/s | 2,199 m/s | 2,200 m/s | 0 |
| Vitesse des balles | 16,00 m/s | 16,00 m/s (384 u/s) | 16,00 m/s | 0 |
| Appui → balle | 183 ms (11 images, dont la sienne) | 250 ms (15 frames) | **167 ms (10 frames = 1 + épaulé 9)** | 0 (même compte : l'image de l'appui + 0,15 s) |
| Épaulé / tir / rechargement (simulation) | 0,15 / 0,3 / 1,2 s | 9 / 18 / 72 f | 9 / 18 / 72 f (rechargement de +0,467 à +1,667 s) | 0 |
| Recul du Frère | aucun | aucun | aucun | 0 |
| Gigue : distance, durée, courbe | gigue : 2 m en 1 temps (129/min ≈ 28 f), 1 − (1 − u)² ; armes : 2 m en 0,36 s | 28 f, 2,30 m (jugé « bien ») | 28 f, 2,00 m (hors temps : l'appui tombe ailleurs dans la mesure depuis que le délai est parti), même courbe | aligné sur le banc gigue (`parite-labo.md` § 2) ; le 0,36 s du banc armes non repris (inchangé) |
| Gigue : latence | 1 image | 6 frames | **1 frame** | 0 |
| Épaulé à l'écran | clip `epauler` entier sur 0,15 s | 9 f, 36 % du clip | **9 f, 0 → 89 % du clip** (dernière frame : 8/9) | 0 |
| Tir à l'écran | `tir` étiré sur 0,3 s (18 f) | 34 f | **18 f** | 0 |
| Rechargement à l'écran | entré à 0 % dès la fin du tir, 1,2 s | entré à 23,6 % (+0,82 s), 55 f | **entré à 1,4 % (+0,467 s, frame qui suit le tir), 71 f** | 0 |
| Premier pas de gigue | `gigue-01` | `gigue-02` | **`gigue-01`** | 0 |

## En ligne (inchangé, mesuré ; consigne du coordinateur, point 1)

Deux clients 1837 release, matchbox local, entrées scriptées (`p-1e-p2p/gestes.ron`), trace d'état détaillée ; traces des deux clients identiques dans chaque run. Latence comptée
de la frame de l'input à la première frame où la position change (même compte hors ligne : 5 avant, 1 après ; banc : 1).

| Configuration | Départ | Glisse à l'arrêt | Demi-tour | Rollbacks (client a / b), frames resimulées |
|---|---|---|---|---|
| Hors ligne avant (délai 5) | 5 f (83 ms) | 29,3 cm | 5 f | — |
| Hors ligne après (délai 0) | **1 f (17 ms)** | **0 cm** | **1 f** | — |
| En ligne avant = après (délai 5, boucle locale) | 5 f (83 ms) | 29,3 cm | 5 f | 2 / 0 ; trace en ligne **identique** avant et après le correctif |
| En ligne, délai 5, RTT 80 ms (netem 40 ms ± 5) | 5 f (83 ms) | 29,3 cm | 5 f | 0 / 3 ; 4 / 19 |
| En ligne, délai 2, RTT 80 ms (proposition) | 2 f (33 ms) | 7,3 cm | 2 f | 2 / 0 ; 33 / 0 |
