# Rapport — m1-v3-bots-softlocks (soft-locks des bots sur `throne`, 200 graines)

Branche partie de `m1-v3-bots-reanimation` (2c1cb21), `origin/main` f80b82b mergé.

## État en cours

Correctifs écrits, testés (tests `bots` 50/50) et rejoués sur les graines en soft-lock (v5,
aa6948f). Mesure « avant » des témoins faite (main f80b82b). Reste, après le merge de D48 (b0) dans
`main` : merger, mesure « après » (graines en soft-lock + témoins 1..20 à 2 et 4 bots), scénarios
figés (familles B et C), suite complète, preuve par inputs des traces qui bougent, livraison.

## Soft-locks (200 graines, 2 bots `prudent`, main 61ac539)

12 soft-locks, 0 desync. Relevé D42 au soft-lock (1 200 frames sans progrès).

| graine | étage | frame | ennemi(s) restant(s) (D42) | bots | famille (après rejeu) |
|---|---|---|---|---|---|
| 23 | 2 | 11367 | roi_rat 56 PV, à 235 px en vue, hors champ de flux | immobiles, **0 munition** | A |
| 43 | 2 | 11682 | roi_rat 252 PV, à 174 px en vue, hors champ de flux | **0 munition** | A + boss coincé (D48) |
| 53 | 1 | 3067 | aucun, portail ouvert, ancre (456, 316) | à 40 px de l'ancre, immobiles | C : ancre dans la roche (D48) |
| 63 | 2 | 5077 | franc_tireur 51 PV à 77 px hors de vue ; tourelle | immobiles | B |
| 76 | 2 | 11401 | roi_rat 315 PV, à 263 px hors de vue, hors champ | j0 seul, **0 munition** | A |
| 81 | 2 | 6491 | aucun, portail ouvert, ancre (512, 352) | j1 seul à 26 px de l'ancre | C |
| 111 | 1 | 3224 | arroseur 54 PV à 178 px hors de vue | immobiles | B |
| 118 | 2 | 5539 | roi_rat 60 PV à 132 px hors de vue ; tourelle | j1 seul, immobile | B |
| 139 | 1 | 3015 | pillard 18 PV à 169 px hors de vue | immobiles | B |
| 149 | 2 | 4185 | franc_tireur 11 PV à 102 px hors de vue ; tourelle | immobiles | B |
| 162 | 2 | 6712 | brute 156 PV à 182 px en vue, hors champ | 12 balles de réserve | A + brute coincée (D48) |
| 200 | 2 | 7448 | franc_tireur 11 PV à 94 px hors de vue ; brute | j0 seul, immobile | B (finit déjà sur f80b82b : réanimation) |

Rejouées sur `main` f80b82b (réanimation incluse) : 11 soft-locks sur 12 (la graine 200 finit).

### Familles, causes établies par rejeu (`--save-scenario`, inputs des bots)

- **B, recul dans la roche** (6 : 63, 111, 118, 139, 149, 200). Pas un poste de tir sans vue
  (hypothèse de départ, fausse) : l'ennemi caché est à moins de `PRUDENT_MIN_DISTANCE` (180 px),
  `prudent` **recule** (Down+Left en boucle, ennemi en haut à droite) et s'enfonce dans la roche,
  en **tirant** sur elle (Fire, Reload en boucle) — même quand l'ennemi est hors de vue.
- **A, à sec** (23, 43, 76, 162). Les chargeurs vidés dans la roche (famille B) épuisent les
  réserves devant le boss ou la tourelle. Graine 43 et 162 : de plus, l'ennemi est coincé hors
  de son propre champ de flux (voir dette).
- **C, portail contre la roche** (53, 81). Sous 48 px, `approach_portal` allait tout droit ; la
  composante qui dégagerait le bot tombe sous la zone morte du pilotage (81 : écart vertical de
  4 px sur 26) ; et aucune case libre pour le corps à moins de 16 px de l'ancre, donc aucune route
  (81 : même de loin, ligne droite dans la roche).

## Correctifs (`crates/bots` seulement)

1. **`prudent` ne recule ni ne tire vers un ennemi caché** (`decide_prudent`) : caché
   (`enemy_visible` faux, mode `Floors` seulement), il va le chercher par le chemin. Test
   `prudent_ni_recul_ni_tir_vers_un_ennemi_cache`.
2. **Tir vers un ennemi immobile caché** (`line_of_fire`) : permis seulement à moins de
   `STILL_TARGET_DISTANCE` (120 px) et si la ligne **brute** (murs sans la marge de 4 px,
   `navigation::walls_clear_with`, `BotView::enemy_shootable`) est libre. Graine 43 : boss coincé
   contre la roche à 85 px, « caché » avec la marge mais touché par les balles (sans ce tir : 0
   dégât, soft-lock). Graine 76 : tourelle derrière la roche ; avec un tir permis sans la ligne
   brute (v2, v3), les chargeurs y repartaient (soft-lock à sec). Test étendu.
3. **Portail** (`input::navigate`, `approach_portal`) : sous la zone de freinage, la ligne droite
   seulement si le corps la parcourt sans toucher d'obstacle (`BotNavigation::body_clear`,
   obstacles élargis du gabarit, bot posé contre un obstacle compris) ; sinon le chemin ; et,
   faute de case libre à moins de `PORTAL_REACH` de l'ancre, la case accessible la plus proche
   (`investigate`). Tests `portail_contre_la_roche_par_le_chemin`,
   `body_clear_refuses_a_body_in_contact`, `portail_par_le_chemin` (ajusté).
4. **À sec, ramasser le butin** (`input::loot_step`, `BotView::loot`) : plus aucune réserve pour
   aucune arme, mode `Floors`, aucun ennemi visible à moins de 150 px : `prudent`/`fonceur` vont au
   `PowerUpPickup` accessible le plus proche (le butin vaut pour tous les joueurs). Test
   `a_sec_prudent_et_fonceur_vont_au_butin`.

### Le ramassage ne se déclenche pas sur la graine 76 : pénurie prouvée

Trace temporaire (binaire `sim-v5dbg`, retirée du code) toutes les 60 frames, graine 76, 2 bots :
dernier butin au sol à **f3180** ; les deux bots sont à sec (aucune réserve) à partir de
**f11280** ; de f3180 au soft-lock (f12661), **0 butin au sol** à chaque relevé. La condition de la
règle est vraie à partir de f11280, il n'y a rien à ramasser : la règle est juste, la **pénurie
de munitions** est un fait de contenu (8 000 frames contre la tourelle de 225 PV et le boss sans
autre source de munitions). Dette proposée ci-dessous.

## Rejeux des graines en soft-lock (2 bots)

| graine | main f80b82b | v1 | v5 (aa6948f) |
|---|---|---|---|
| 23 | soft-lock | étage 3 | étage 3 |
| 43 | soft-lock | soft-lock | étage 3 |
| 53 | soft-lock | soft-lock | soft-lock (ancre dans la roche, D48) |
| 63 | soft-lock | étage 3 | étage 3 |
| 76 | soft-lock | étage 3 | soft-lock (pénurie) |
| 81 | soft-lock | étage 3 | étage 3 |
| 111 | soft-lock | défaite | défaite |
| 118 | soft-lock | étage 3 | étage 3 |
| 139 | soft-lock | étage 3 | étage 3 |
| 149 | soft-lock | étage 3 | étage 3 |
| 162 | soft-lock | défaite | défaite |
| 200 | étage 3 | étage 3 | étage 3 |

## Mesure témoin (graines 1..20)

| | étage 3 | défaites | soft-locks | desync |
|---|---|---|---|---|
| 2 bots, main f80b82b (avant) | 14/20 | 6 | 0 | 0 |
| 4 bots, main f80b82b (avant) | 20/20 | 0 | 0 | 0 |

« Après » : sur la branche mergée avec `main` + D48 (à faire).

## Dettes proposées

- **D48 (b0, ouverte au merge) — ennemi coincé hors de son propre champ de flux.** Graine 43,
  étage 2 : `roi_rat` (gabarit Large) en case (57, 4)–(58, 4), `path_cost` absent (hors du champ
  `GroundBreaker/Large`), à 3 cases du point d'apparition S (60, 4) ; il ne suit plus de chemin.
  Graine 162, étage 2 : `brute` en (9, 4)–(9, 5), hors du champ `GroundBreaker`.
  `spawn_clearance` (D41) garantit un dégagement autour du point d'apparition, pas que la zone où
  le corps se trouve reste couverte par son champ. **Ancre dans la roche** : graine 53, étage 1,
  ancre du portail (456, 316) à l'intérieur du mur 48×64 (x 416–464, y 288–352) ; avec le corps du
  joueur (20×20, décalage y −6), le centre atteignable le plus proche est à 32 px (dessous), 35
  (droite), 52 (dessus) : le rayon de franchissement est de 24 px, portail infranchissable.
- **D50 (orch, ouverte au merge de D48) — pénurie de munitions** (contenu `throne`) : graine 76, étage 2 (voir plus haut) : plus aucun
  butin au sol de f3180 à f12661, bots à sec à f11280 devant une tourelle (225 PV) et le boss.
