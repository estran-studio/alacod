# Rapport — m1-v3-bots-softlocks (soft-locks des bots sur `throne`, 200 graines)

Branche partie de `m1-v3-bots-reanimation` (2c1cb21).

## État en cours

Liste faite depuis les JSON des 200 graines à 2 bots (`main` 61ac539, binaire
`alacod-sim-61ac539`), lecture seule. Hypothèses par famille tirées du code, **pas encore
rejouées** (attente du feu vert d'orch). Reste : rejeu de chaque graine (`--save-scenario`),
correctifs B puis A puis C, tests, mesure avant/après, scénarios figés, suite.

## Soft-locks (200 graines, 2 bots `prudent`, main 61ac539)

12 soft-locks, 0 desync. Relevé D42 au soft-lock (1 200 frames sans progrès) ; « −600 » : case
du joueur 600 frames avant.

| graine | étage | frame | ennemi(s) restant(s) (D42) | bots | famille |
|---|---|---|---|---|---|
| 23 | 2 | 11367 | roi_rat 56 PV, à 235 px en vue, hors champ de flux | j0, j1 (41, 39) immobiles, **0 munition** (chargeur et réserves) | A |
| 43 | 2 | 11682 | roi_rat 252 PV, à 174 px en vue, hors champ de flux | j0 (46, 4), j1 (44, 4), **0 munition** | A |
| 53 | 1 | 3067 | aucun, portail ouvert, ancre (456, 316) | j0 à 40 px, j1 à 40 px de l'ancre, immobiles | C |
| 63 | 2 | 5077 | franc_tireur 51 PV à 77 px hors de vue ; tourelle | j0, j1 (4, 4) immobiles | B |
| 76 | 2 | 11401 | roi_rat 315 PV, à 263 px hors de vue, hors champ | j0 seul (j1 mort), **0 munition** | A |
| 81 | 2 | 6491 | aucun, portail ouvert, ancre (512, 352) | j1 seul à 26 px de l'ancre (rayon 24), immobile | C |
| 111 | 1 | 3224 | arroseur 54 PV à 178 px hors de vue | j0, j1 (37, 2) immobiles | B |
| 118 | 2 | 5539 | roi_rat 60 PV à 132 px hors de vue ; tourelle | j1 seul (61, 40) immobile | B |
| 139 | 1 | 3015 | pillard 18 PV à 169 px hors de vue | j0, j1 (36, 15) immobiles | B |
| 149 | 2 | 4185 | franc_tireur 11 PV à 102 px hors de vue ; tourelle | j0, j1 (37, 5) immobiles | B |
| 162 | 2 | 6712 | brute 156 PV à 182 px en vue | j0, j1 bougent peu, 12 balles de réserve | A ? |
| 200 | 2 | 7448 | franc_tireur 11 PV à 94 px hors de vue ; brute | j0 seul (2, 38) immobile | B |

Familles (hypothèses, à confirmer par rejeu) :

- **B, poste de tir sans vue** (6) : `BotNavigation::chase` mène au « poste de tir » (vue calculée
  de centre de case à centre de case) ; depuis la position réelle, `walls_clear` (marge 4 px) dit
  « hors de vue », le pas restant est annulé par la zone morte, `investigate` n'est pas essayé.
- **A, à sec** (3–4) : plus aucune munition ; le butin (`RefillAmmoOf`) vit 1 800 frames et
  `prudent` ne le cherche pas ; `prudent` tire dès 320 px **même sans ligne de tir** (munitions
  vidées dans la roche).
- **C, portail contre la roche** (2) : l'ancre (barycentre des spawns) touche la roche ; sous
  48 px `approach_portal` va tout droit, sans le chemin.
