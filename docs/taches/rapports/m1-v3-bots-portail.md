# Rapport — m1-v3-bots-portail (bots : portail et cible fixe hors de vue)

Branche partie de `m1-restart-p2p` (d770e9a). Reproductions faites sur la branche de b0
`m1-integration-scenarios` **3b7a2df** (correctif « une caverne = un asset », tourelle éloignée),
mergée temporairement et jamais livrée ; correctifs reportés sur cette branche par cherry-pick.

## État

Livrée : `origin/main` b22d4ab (intégration de b0 : cave://, boss, calibrage) mergé ; mesure 1/2/4
bots sur ce contenu ; six scénarios à bots en mode `Floors` remesurés (attentes vertes, traces à
bénir par orch, preuve ci-dessous) ; aucune autre trace ne bouge.

## Diagnostic et correctifs (`crates/bots`, navigation seulement en mode `Floors`)

1. **(a) Le bot seul ne prend pas le portail ouvert** (solo, graines 3 et 16). L'ancre du portail
   n'est pas dans la roche (graine 3 : (384,260), case (24,16) atteignable, zone ouverte ; pas de
   dette `run::floors`). Log de route : le bot **orbitait** autour du portail à pleine vitesse
   (≈ 140–150) : les boutons ne donnent que le signe de chaque axe, le corps garde son élan, il
   dépassait chaque point visé (sous 48 px en T1.14, puis à 50–190 px en suivant la route).
   Correctif : approche du portail **pilotée en vitesse** (`decide::approach_portal`) : vitesse
   voulue = direction (route au-delà de 48 px, portail en deçà) × min(120, 2 × distance), chaque axe
   pressé selon l'écart vitesse voulue − vitesse actuelle (zone morte 12) : vitesse transverse
   annulée, arrivée freinée par le bouton opposé. (Un premier correctif — freinage dès 112 px — ne
   faisait que déplacer l'orbite : graine 7 à 2 bots.)
2. **(b) Cible fixe hors de vue jamais achevée** (2 bots, graine 123456 : tourelle dans un
   recoin). Deux causes :
   - la ligne de vue était un segment fin : en frôlant un coin de roche elle passait pour
     « visible » à 295 px alors que les balles s'y arrêtaient (PV de la tourelle figés) →
     **ligne de tir avec marge** `SHOT_MARGIN` = 4 px (`navigation::walls_clear` et les postes de
     tir `BotNavigation::visible`) ;
   - même avec une vraie ligne, `prudent` tirait de 300 px (dispersion) et vidait ses munitions,
     ou touchait si rarement que la partie n'avançait plus (graine 7 : ennemi coincé, 15 PV en 600
     frames) → **ennemi immobile** (`MoveSpeed` de base nulle : tourelle ; ou `Velocity::main`
     nulle : ennemi coincé) : `prudent` s'en rapproche par le chemin jusqu'à 120 px et ne recule
     pas devant lui (`BotView::enemy_still`).
3. Diagnostic : l'instantané du `SoftlockDump` porte l'état du portail (index, ouvert, ancre et sa
   case) et le dernier input de chaque joueur.

Tests unitaires : `prudent_freine_pres_du_portail`, `prudent_annule_la_vitesse_transverse`,
`prudent_se_rapproche_d_un_ennemi_immobile`, `ligne_de_tir_avec_marge`.

## Graines du signalement (base 3b7a2df de b0, avec les correctifs)

| cas | avant | après |
|---|---|---|
| solo graine 3 | soft-lock étage 0, portail ouvert | portails des étages 0 et 1 franchis (f701, f2523), mort à l'étage 2 |
| solo graine 16 | soft-lock étage 0, portail ouvert | portail franchi (f769), mort à l'étage 1 |
| 2 bots graine 123456 | soft-lock f4569, tourelle à 1 PV | étage 3 à f3810 |
| 2 bots graine 7 | orbite autour du portail (étage 0) | étages 0 et 1 franchis ; soft-lock étage 2 : munitions épuisées |

## Mesure sur main (b22d4ab), 20 graines, `--floors run --until-floor 3 --max-frames 15000`

« Avant » = `alacod-sim` d'`origin/main` ; « après » = cette branche mergée ; mêmes assets.

| bots | étage 3 avant | étage 3 après | soft-locks avant → après | tous morts avant → après |
|---|---|---|---|---|
| 1 | 0/20 | 1/20 | 6 → 4 | 14 → 15 |
| 2 | 3/20 | 6/20 | 16 → 11 | 1 → 3 |
| 4 | 11/20 | **18/20** | 9 → 2 | 0 → 0 |

Aucun soft-lock « portail ouvert » après ; aucun « sans munitions » dominant (les réglages de b0
l'ont traité). Seul, le bot meurt (difficulté) ; plus aucun blocage à l'étage 0.

Soft-locks restants à 2 bots (11) : surtout le **dernier joueur vivant à terre** (graines 6, 15, 16,
17, 19 — le joueur à terre saigne 1800 frames, voir D43) et des **combats lents** (PV de l'ennemi
qui baissent : 169 → 33, 444 → 300… mais pas de kill en 600 frames, graines 4, 10, 14, 17, 20,
voir D44) ; graine 13 : ennemis inactifs dans des coins et un bot au chargeur presque vide.

## Scénarios et traces (preuve)

`make test_scenarios` : exactement six scénarios changent, tous à bots `prudent` en mode
`Floors` ; aucun autre. Leurs attentes, mesurées avec l'ancien bot, ont été **remesurées**
(`ALACOD_EVENTS=1`, `ALACOD_DUMP_TRACE` pour les NetId du boss) et les commentaires « Mesuré » mis à
jour ; elles passent toutes, seule la trace diffère.

Preuve que seule la décision des bots change : les correctifs vivent dans `crates/bots`
(`ReadInputs`, hors rollback, aucune règle de simulation). Inputs enregistrés
(`alacod-sim --save-scenario`, graine 123456, binaires avant/après) : la première différence d'input
précède de 5 frames la première ligne de trace qui diffère, dans chaque configuration :

| scénario(s) | 1re différence d'input | 1re ligne de trace différente | cause |
|---|---|---|---|
| `throne_floor_1`, `throne_progression`, `throne_solo` (1 bot) | f403 : « Fire » → « Down, Left, Fire » | f408 | ennemi immobile : approche au lieu de garder la bande |
| `throne_three_floors` (2 bots) | f128, joueur 1 : « Down, Left » → « Up, Left » | f133 | idem (pas de recul devant l'immobile) |
| `throne_quad` (4 bots) | f128, les quatre joueurs | f133 | idem |
| `bot_floors_three` (testbed, 2 bots) | f0 : « Down, Right, Fire » → « Fire » | f5 | ennemi immobile au départ : plus de recul |

Nouvelles valeurs mesurées (graine 123456) :
- `bot_floors_three` : portails à f56, f484, f765 (avant f135, f390, f792).
- `throne_floor_1` : portail ouvert f648, franchi f853 (avant f808 : vitesse de croisière 120 et
  arrivée freinée, contre l'ancienne pleine vitesse) ; `frames` 830 → 880.
- `throne_progression` : étage 1 f853, niveau 2 f1651, alerte f1753.
- `throne_solo` : étage 1 f853, étage 2 f2237 (avant f2816), **vivant** à f3699 (avant : mort et
  défaite à f3640) ; attentes `PlayerDead`/`Defeat` retirées.
- `throne_three_floors` (2 bots) : étages f480, f1097, **f2640** (avant f5109) ; `frames` 5200 → 2700
  (la boucle sur le niveau 3 les tue ensuite, f4185 et f4642) ; joueur 1 à terre à f2385.
- `throne_quad` (4 bots) : étages f697, f1512, f3761 (avant f662, f1380, f3313) ; boss NetId 697
  (540 PV) à f1511, mort à f2935 ; nouveau boss NetId 1492 à f3760 ; joueur 2 à terre à f2303 ;
  `frames` 3400 → 3800.

## Dettes (à reporter par orch)

- **D43** — dernier joueur vivant à terre sans personne pour le relever : la partie ne finit ni
  gagnée ni perdue pendant le saignement. Mesuré dans `throne` (Floors) : un joueur à terre saigne
  1800 frames (`Downed { since_frame: 3505, bleedout_at_frame: 5305 }`, graine 7 de b0), donc la
  défaite arrive, mais après la fenêtre du détecteur de soft-lock (600 frames) : la plupart de ces
  cas sont des faux positifs de D44. Reste à décider la règle (« à terre et seul → défaite
  immédiate » ?) et à mesurer zombies à 2 joueurs (même situation).
- **D44** — heuristique de soft-lock d'`alacod-sim` : « 600 frames sans kill » signale un combat
  lent (PV de l'ennemi qui baissent) et un saignement en cours ; compter aussi les dégâts infligés
  et ne pas conclure pendant un saignement.

## Vérifié

`CARGO_BUILD_JOBS=2`, profil `headless` : tests de `bots` (44 + nouveaux : portail piloté en
vitesse, vitesse transverse, ennemi immobile, ligne de tir avec marge) ; les six scénarios : attentes
vertes, trace seule différente ; suite complète : voir ci-dessous.
