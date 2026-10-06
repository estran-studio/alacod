# Rapport — m1-v3-bots-softlocks (soft-locks des bots sur `throne`, 200 graines)

Branche partie de `m1-v3-bots-reanimation` (2c1cb21), `origin/main` ddb9789 (D48) mergé.

## État en cours

Vérifié, livré. Suite complète sur la branche + `main` ddb9789 : attentes de tous les scénarios
vertes ; seules différences, les traces des six scénarios à bots listés plus bas (preuve par
inputs) et la trace nouvelle de `throne_softlock_recul` (à bénir) ; tests des crates verts (seul
échec : `scenarios`, pour ces traces) ; `make lint`, `cargo fmt --check`, interdits,
enregistrement rollback, `make gen` testbed/zombies/throne (aucun fichier modifié), `cargo check
-p throne`, exemples : verts. Target purgé.

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
3. **Portail** — **filet sans témoin depuis D48** (D48 place les ancres par connexité : 53 et
   81 finissent sur `main` ddb9789 ; aucune graine ne reproduit plus la famille C ; gardé,
   couvert par ses tests unitaires) (`input::navigate`, `approach_portal`) : sous la zone de freinage, la ligne droite
   seulement si le corps la parcourt sans toucher d'obstacle (`BotNavigation::body_clear`,
   obstacles élargis du gabarit, bot posé contre un obstacle compris) ; sinon le chemin ; et,
   faute de case libre à moins de `PORTAL_REACH` de l'ancre, la case accessible la plus proche
   (`investigate`). Tests `portail_contre_la_roche_par_le_chemin`,
   `body_clear_refuses_a_body_in_contact`, `portail_par_le_chemin` (ajusté).
4. **À sec, ramasser le butin** (`input::loot_step`, `BotView::loot`) : plus aucune réserve pour
   aucune arme, mode `Floors`, aucun ennemi visible à moins de 150 px : `prudent`/`fonceur` vont au
   `PowerUpPickup` accessible le plus proche (le butin vaut pour tous les joueurs). Test
   `a_sec_prudent_et_fonceur_vont_au_butin`.
5. **Vers un ennemi caché, la route pilotée en vitesse** (`decide::steer`, partagé avec
   `approach_portal`) : après le merge de D48, `throne_quad` (graine 123456, 4 bots) faisait un
   soft-lock nouveau à l'étage 2 (f3807) ; trace temporaire de `navigate` (binaire `sim-navdbg`,
   retirée) : les bots suivaient la route (`chase`) vers une tourelle cachée à 700 px, au signe
   des boutons ; l'élan leur faisait dépasser chaque case visée (à (261, 236), route gauche + haut,
   le bot descend encore jusqu'à y = 199), la route s'inversait à chaque bout d'un couloir et la
   sortie à droite (route (8, 0) en (263, 248)) était traversée sans être prise. Défaut préexistant
   (même cause que l'orbite autour du portail, m1-v3-bots-portail), révélé par la nouvelle
   trajectoire. Pilotée en vitesse : `throne_quad` finit à f2590 (main : f2742), la graine 4 à
   f4162 sans mort (main : f4874). Test `vers_un_ennemi_cache_la_route_est_pilotee_en_vitesse`.

### Le ramassage ne se déclenche pas sur la graine 76 : pénurie prouvée

Trace temporaire (binaire `sim-v5dbg`, retirée du code) toutes les 60 frames, graine 76, 2 bots :
dernier butin au sol à **f3180** ; les deux bots sont à sec (aucune réserve) à partir de
**f11280** ; de f3180 au soft-lock (f12661), **0 butin au sol** à chaque relevé. La condition de la
règle est vraie à partir de f11280, il n'y a rien à ramasser : la règle est juste, la **pénurie
de munitions** est un fait de contenu (8 000 frames contre la tourelle de 225 PV et le boss sans
autre source de munitions). Dette proposée ci-dessous.

## Mesure avant/après

Base commune : `main` ddb9789 (D48 mergé, cartes changées : les mesures sur f80b82b ne sont plus
comparables). « Avant » = `main` ddb9789, « après » = la branche (v7, 24524d6) mergée avec lui ;
`alacod-sim --game throne --bots N --profiles prudent×N --floors run --until-floor 3
--max-frames 15000`.

### Témoins (graines 1..20)

| | étage 3 | défaites | soft-locks | desync |
|---|---|---|---|---|
| 2 bots, avant | 12/20 | 7 (2, 3, 6, 15, 16, 17, 19) | 1 (7) | 0 |
| 2 bots, après | **16/20** | **4** (11, 12, 17, 20) | **0** | 0 |
| 4 bots, avant | 20/20 | 0 | 0 | 0 |
| 4 bots, après | 20/20 | 0 | 0 | 0 |

Défaites : 7 → 4 (3 de moins ; 11, 12 et 20 apparaissent, 2, 3, 6, 15, 16, 19 disparaissent).

### Graines en soft-lock des 200 (2 bots)

| graine | avant (ddb9789) | après (v7) |
|---|---|---|
| 23 | étage 3 (f5536) | étage 3 (f5097) |
| 43 | étage 3 (f5215) | étage 3 (f5183) |
| 53 | étage 3 (f5366) | étage 3 (f4799) |
| 63 | **soft-lock** (f5077) | défaite (f3882) |
| 76 | étage 3 (f4510) | étage 3 (f4681) |
| 81 | étage 3 (f5453) | étage 3 (f6055) |
| 111 | **soft-lock** (f3224) | étage 3 (f4462) |
| 118 | **soft-lock** (f5539) | étage 3 (f5837) |
| 139 | **soft-lock** (f3015) | défaite (f4346) |
| 149 | **soft-lock** (f4185) | étage 3 (f4075) |
| 162 | étage 3 (f4910) | défaite (f2503) |
| 200 | étage 3 (f7550) | étage 3 (f5745) |

Soft-locks 5 → **0**. 43, 53, 162 et 81 finissent déjà sur `main` grâce à D48. Une défaite
ajoutée sur ces graines (162), deux soft-locks devenus défaites (63, 139) : les défaites sont
l'affaire de b0 (m1-analyse-200-throne) ; sur les témoins, elles baissent de 7 à 4.

Rejeux intermédiaires (sur f80b82b, avant D48) : v1 (recul/tir) 2 soft-locks restants (43, 53) ;
v2 (tir vers un ennemi immobile caché) corrige 43 mais revide les chargeurs sur la graine 76 ;
v3 (immobile à moins de 120 px) ; v4 (butin) = v3 ; v5 (ligne brute) : restent 53 (ancre dans la
roche) et 76 (pénurie).

## Scénarios et traces

Scénario figé (un seul, famille B) : `tests/scenarios/throne_softlock_recul.ron` — graine 139,
deux `prudent` : soft-lock à l'étage 1 sur `main` ddb9789 (f3015) ; avec le correctif, étage 1 à
f866, étage 2 à f2237 (`FloorIndex`). Placé dans `tests/scenarios/` (le harnais ne lit que ce
dossier ; `games/throne/assets/scenarios/` de la fiche n'existe pas). Trace nouvelle : bless
orchestrateur. Famille C : pas de scénario, aucune graine ne la reproduit depuis D48.

**Traces qui bougent** (six scénarios à bots ; aucun scénario zombies ni en vagues, `clone_quad`
et `bots_four_mixed` intacts, `equilibrage_*` intacts) — preuve par inputs (`alacod-sim`
`--save-scenario`, `main` ddb9789 contre la branche, même graine, même nombre de bots) :

| scénario | premier input différent | ce qui change | trace diverge à |
|---|---|---|---|
| `throne_floor_1`, `throne_progression`, `throne_solo` (1 bot, 123456) | f21, joueur 0 | `Down, Left, Fire` → `Down, Left` : plus de tir vers l'ennemi caché | ligne 27 (f26) |
| `throne_quad` (4 bots, 123456) | f19, joueur 2 | `Down, Left, Fire` → `Down, Left` | ligne 25 (f24) |
| `throne_three_floors` (2 bots, graine 4) | f3, joueur 0 | `Down, Left, Fire` → `Down, Left` | ligne 9 (f8) |
| `bot_floors_three` (testbed, 2 bots) | f134, joueur 1 | `Down, Right, Fire` (recul) → `Up, Left` (vers l'ennemi caché, sans tir) | ligne 140 (f139) |

Chaque trace diverge 5 frames après le premier input différent. Attentes remesurées
(`ALACOD_EVENTS=1`) :

- `throne_floor_1` : portail franchi à f937 (avant f853), frames 880 → 960.
- `throne_progression` : niveau 1 à f547, étage 1 à f937, `sang_froid` à f1147, niveau 2 à
  f1716, `alerte` à f1837 ; test `hud_text_sur_throne_progression` aligné (Niv. 1 à f550,
  Niv. 2 à f1720).
- `throne_solo` : étage 1 à f937, étage 2 à f3124 (avant f2237), `coriace` à f2316.
- `throne_quad` : étages à f489, f1120, f2590 (avant f697, f1512, f2742) ; les attentes
  `EntityHealth` du boss (NetId 697, 1306) sont remplacées par l'ouverture du portail du niveau
  3 (`Event portal « niveau 2 »` à f2588, boss mort) : les NetId ont changé.
- `throne_three_floors` : étages à f275, f1490, f4162 (avant f663, f1594, f4874) ; personne à
  terre : les attentes `PlayerDowned`/`PlayerRevived` disparaissent (la réanimation reste
  couverte par les tests de `crates/bots`).
- `bot_floors_three` : portails à f56, f484, f892 (avant f765), frames 820 → 920.

## Dettes proposées

- **D48 (b0, mergée dans ddb9789) — ennemi coincé hors de son propre champ de flux.** Graine 43,
  étage 2 : `roi_rat` (gabarit Large) en case (57, 4)–(58, 4), `path_cost` absent (hors du champ
  `GroundBreaker/Large`), à 3 cases du point d'apparition S (60, 4) ; il ne suit plus de chemin.
  Graine 162, étage 2 : `brute` en (9, 4)–(9, 5), hors du champ `GroundBreaker`.
  `spawn_clearance` (D41) garantit un dégagement autour du point d'apparition, pas que la zone où
  le corps se trouve reste couverte par son champ. **Ancre dans la roche** : graine 53, étage 1,
  ancre du portail (456, 316) à l'intérieur du mur 48×64 (x 416–464, y 288–352) ; avec le corps du
  joueur (20×20, décalage y −6), le centre atteignable le plus proche est à 32 px (dessous), 35
  (droite), 52 (dessus) : le rayon de franchissement est de 24 px, portail infranchissable.
- **D50 (ouverte par orch au merge de D48) — pénurie de munitions** (contenu `throne`) : graine 76, étage 2 (voir plus haut) : plus aucun
  butin au sol de f3180 à f12661, bots à sec à f11280 devant une tourelle (225 PV) et le boss.
