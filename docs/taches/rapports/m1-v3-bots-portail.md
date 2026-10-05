# Rapport — m1-v3-bots-portail (bots : portail et cible fixe hors de vue)

Branche partie de `m1-restart-p2p` (d770e9a). Reproductions faites sur la branche de b0
`m1-integration-scenarios` **3b7a2df** (correctif « une caverne = un asset », tourelle éloignée),
mergée temporairement et jamais livrée ; correctifs reportés sur cette branche par cherry-pick.

## État en cours

Correctifs écrits et vérifiés sur les graines du signalement. Reste : merge d'`origin/main` (avec
l'intégration de b0), mesure 1/2/4 bots sur ce contenu, suite complète (traces
`bot_floors_three`/`throne_*` : rebénies seulement si elles bougent, avec preuve).

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
