# 200 graines de `throne` à deux bots : pourquoi les bots perdent

m1-analyse-200-throne, 2026-10-05.

## Mesure

- Commande (orchestrateur, machine locale, binaire exact `alacod-sim-61ac539`, main `61ac539`) :
  `alacod-sim --game throne --bots 2 --profiles prudent,prudent --floors run --seeds a..b
  --until-floor 3 --max-frames 15000 --progress --json throne-2-a-b.json`, quatre lots de 50.
- Données : `alacod_tasks/m1-200-throne/` (JSON, journaux) ; rejeux dans `m1-200-throne/rejeux/`.
- Les 39 défaites ont été rejouées avec le même binaire et `--save-scenario` : les 39
  reproduisent le JSON à l'identique (frames, `run_end`, `floor_frames`, `damage_taken`).
  12 d'entre elles (réparties sur la durée passée au troisième étage) ont été rejouées en
  scénario avec la trace détaillée (`ALACOD_DUMP_TRACE`), une à la fois (la trace complète
  tient en mémoire, deux en parallèle ont saturé les 15 Go de la machine) : santé, à terre,
  réserves, chargeurs et dernier tir de chaque arme, ennemis, objets au sol, et les
  `DamageEvent` (source, cible, montant) de chaque frame.
- Les 200 graines à **4 bots** ont été arrêtées par la mémoire de la machine : pas de mesure à
  4 bots dans ce digest.

## Résultat

| Issue | Graines | Part |
|---|---:|---:|
| trois étages finis | 149 | 74,5 % |
| défaite | 39 | 19,5 % |
| soft-lock | 12 | 6 % |
| desync | 0 | 0 % |

**Par étage** (« étage 3 » = index 2 des journaux, caverne `niveau_3` du boss `roi_rat`) :

| Étage | Arrivées | Défaites | Soft-locks | Échec |
|---:|---:|---:|---:|---:|
| 1 | 200 | 0 | 0 | 0 % |
| 2 | 200 | 0 | 3 | 1,5 % |
| 3 | 197 | 39 | 9 | 24 % |

**Comparaison avec les mesures à 20 graines du journal** (`docs/taches.md`, 2 bots : 9/20 avec
l'outil corrigé, puis 7/20 après bots-portail sur l'ancien outil) : sur 200 graines, 74,5 %.
Par blocs de 20 graines, la réussite va de 12/20 à 17/20 (graines 1 à 20 : 13/20) : une mesure
sur 20 graines a une marge de ±3 graines au moins, et les mesures du journal portaient sur des
états antérieurs de `main`. Elles ne se comparent pas directement ; les 200 graines font foi.

## Causes

**Défaites (39).** Toutes au troisième étage, toutes avec un survivant qui finit seul : 37 ont
`deaths` 1/2 (un mort, l'autre à terre à la défaite), 2 ont `deaths` 2/2 (graines 85 et 131,
dont le premier bot est tombé dès le deuxième étage). Sur les 12 rejeux :

- **La première mise à terre** arrive 779 à 1 229 frames après l'entrée au troisième étage
  (médiane 952), alors que les victoires mettent 3 536 frames (médiane) à finir cet étage. À ce
  moment, 2 à 7 ennemis sont encore en vie (médiane 5) et le boss garde 62 à 540 PV sur 540.
- **Qui la cause** (dégâts reçus dans les 300 frames d'avant) : arroseur 35 %, roi_rat 28 %,
  cracheur 15 %, franc_tireur 8 %, tourelle 1 %, mêlée et charge (brute, buffle, pillard) 12 %. Source principale : **tireurs** 7 graines
  (arroseur 6, cracheur 1), **boss** 5 graines. Sur tout l'étage, le roi_rat inflige 36 % des
  dégâts reçus, les tireurs 46 %.
- **Pas de pénurie de munitions** : au moins 432 balles en réserve chez chaque bot à la première
  mise à terre (12/12) ; la piste « bots à sec devant le boss » vue sur des soft-locks ne vaut pas
  pour les défaites.
- **Tir peu efficace** : au troisième étage, 57 à 80 % des balles consommées ne se traduisent pas
  en dégâts (médiane 74 % ; dégâts infligés / balles consommées, mitraillette à 8 par balle),
  compatible avec le tir vers un ennemi hors de vue ou dans la roche relevé par b1. **Une seule
  arme** : la mitraillette tire dans 12/12 ; revolver et lance-lames ne tirent jamais. Armes au
  sol non ramassées (fusil à pompe, revolver, laser, disque) dans 5/12, power-ups non ramassés
  (munitions, rage, vitesse) dans 9/12.
- **Le survivant finit seul** : 54 à 4 828 frames (médiane 545) entre la première mise à terre
  et la défaite ; les bots de `61ac539` ne réaniment pas. À la mise à terre, les deux bots sont
  à 131 à 636 px l'un de l'autre (médiane 330).

**Soft-locks (12)**, classés sans rejeu (traités par b1, m1-v3-bots-softlocks) depuis
`softlock.final_state` : boss vivant 4 (23, 43, 76, 118), ennemi non atteint 6 (brute 162 et
200, tourelle ou franc_tireur 63 et 149, arroseur 111, pillard 139), portail ouvert non pris 2
(53, 81).

**Défaites non rejouées en détail (27)** : même tableau JSON (troisième étage, un mort, survivant
seul) ; la source de la première mise à terre n'est pas départagée entre tireurs et boss.

## Trois correctifs proposés

1. **Bots (b1) — relever le coéquipier.** Preuve : 39/39 défaites finissent avec un survivant
   seul (`deaths`, `run_end`), 54 à 4 828 frames de solitude sur les 12 rejeux, 2 défaites
   (85, 131) où le premier bot tombe dès le deuxième étage et n'est jamais relevé. Effet attendu :
   chaque défaite devient au moins un combat à deux plus long ; la part convertie se mesure en
   rejouant les 200 graines. Coût : fait, la réanimation de b1 est mergée dans `main` (`f80b82b`,
   m1-v3-bots-reanimation) ; reste une relance des 200 graines pour mesurer.
2. **Bots (b1) — tirer juste et changer d'arme.** Preuve : 74 % de balles perdues au troisième
   étage, mitraillette seule dans 12/12, armes et power-ups ignorés au sol ; les tireurs à distance
   causent la première mise à terre dans 7/12. Effet attendu : plus de dégâts par seconde sur les
   tireurs, donc moins de feu reçu avant la mise à terre (5 ennemis vivants en médiane à ce
   moment). Coût : moyen ; le v1 de b1 en cours (pas de tir ni de recul vers un ennemi hors de vue
   en `Floors`) traite la première moitié ; ramasser les armes et passer au fusil à pompe ou aux
   lames contre le boss reste à faire.
3. **Contenu — l'étage 3 est un mur.** Preuve : 39/39 défaites et 9/12 soft-locks au troisième
   étage, 0 défaite aux deux premiers. `niveau_3` place 9 ennemis dont quatre tireurs (arroseur,
   cracheur, franc_tireur, tourelle) et le roi_rat à 540 PV (360 × difficulté 1,5 à l'entrée :
   `1 + floor × 0,25 + floor_minutes × 0,25`, qui s'applique aussi aux dégâts des ennemis). Le
   boss et les tireurs font 82 % des dégâts reçus. Proposition : baisser le terme `floor` de la
   difficulté (0,25 → 0,15 : boss à 468 PV) ou retirer un tireur de `niveau_3` (l'arroseur, 6/12
   premières sources). Effet attendu : non mesuré ici ; à juger après les correctifs 1 et 2, qui
   changent le niveau des bots (un réglage de contenu calé sur des bots qui perdent 74 % de leurs
   balles serait faux). Coût : faible (RON), puis une relance des 200 graines.

## Ce que les 200 graines disent du critère (§9.8)

- **Desync : atteint**, 0 sur 200.
- **Soft-locks : non atteint**, 12 sur 200 (6 %), causes ci-dessus, pris par b1.
- **Défaites** : hors critère, 39 sur 200 (19,5 %), toutes au troisième étage.

## Scénarios figés

`tests/scenarios/throne_defaite_{tireurs,boss,coequipier}.ron` (graines 103, 124, 131) : inputs
enregistrés des bots (le scénario ne dépend pas du code des bots), attentes fixant l'état mesuré à
la première mise à terre (étage, joueur à terre, réserves de balles exactes, ennemis vivants, PV
du boss) puis la défaite. Verts sur la base de la branche (code identique à `61ac539`), traces à
bénir par l'orchestrateur.

## Dette d'outillage

`alacod-sim` n'installe aucun subscriber de log : `RUST_LOG` ne produit rien. Un `--log` (ou
`RUST_LOG` honoré) aurait évité de passer par la trace détaillée des scénarios.

## Annexe : toutes les graines en échec

| Graine | Issue | Étage | Frames | `floor_frames` | Cause | Preuve |
|---:|---|---:|---:|---|---|---|
| 1 | défaite | 3 | 4340 | [481, 1484] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4339, `deaths` 1/2, 2856 frames au 3e étage |
| 2 | défaite | 3 | 4198 | [671, 2025] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4197, `deaths` 1/2, 2173 frames au 3e étage |
| 3 | défaite | 3 | 3425 | [663, 1740] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3424, `deaths` 1/2, 1685 frames au 3e étage |
| 6 | défaite | 3 | 3747 | [1164, 2325] | tireurs | rejeu : 1re mise à terre f3334 (étage 3), sources des 300 frames d'avant : arroseur 52, buffle 12, cracheur 6 ; survivant seul 412 frames ; balles min 672 |
| 15 | défaite | 3 | 3768 | [1142, 2777] | tireurs | rejeu : 1re mise à terre f3713 (étage 3), sources des 300 frames d'avant : arroseur 36, roi_rat 24, cracheur 12 ; survivant seul 54 frames ; balles min 462 |
| 16 | défaite | 3 | 3577 | [1125, 2147] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3576, `deaths` 1/2, 1430 frames au 3e étage |
| 17 | défaite | 3 | 3846 | [750, 1571] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3845, `deaths` 1/2, 2275 frames au 3e étage |
| 23 | soft-lock | 3 | 11367 | [703, 2009] | boss vivant (roi_rat 56 PV) | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 25 | défaite | 3 | 4591 | [932, 2921] | boss | rejeu : 1re mise à terre f3808 (étage 3), sources des 300 frames d'avant : roi_rat 47, franc_tireur 12, tourelle 8 ; survivant seul 782 frames ; balles min 432 |
| 31 | défaite | 3 | 3511 | [732, 2283] | boss | rejeu : 1re mise à terre f3100 (étage 3), sources des 300 frames d'avant : roi_rat 15, cracheur 12, brute 8 ; survivant seul 410 frames ; balles min 492 |
| 42 | défaite | 3 | 2996 | [710, 1684] | tireurs | rejeu : 1re mise à terre f2541 (étage 3), sources des 300 frames d'avant : arroseur 32, roi_rat 31, franc_tireur 6 ; survivant seul 454 frames ; balles min 612 |
| 43 | soft-lock | 3 | 11682 | [1438, 2462] | boss vivant (roi_rat 252 PV) | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 46 | défaite | 3 | 3859 | [661, 2090] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3858, `deaths` 1/2, 1769 frames au 3e étage |
| 47 | défaite | 3 | 3690 | [635, 1523] | tireurs | rejeu : 1re mise à terre f2499 (étage 3), sources des 300 frames d'avant : arroseur 32, franc_tireur 24, brute 16 ; survivant seul 1190 frames ; balles min 552 |
| 52 | défaite | 3 | 4209 | [414, 2427] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4208, `deaths` 1/2, 1782 frames au 3e étage |
| 53 | soft-lock | 2 | 3067 | [755] | portail ouvert non pris | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 54 | défaite | 3 | 3740 | [1055, 2172] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3739, `deaths` 1/2, 1568 frames au 3e étage |
| 59 | défaite | 3 | 4611 | [1335, 3191] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4610, `deaths` 1/2, 1420 frames au 3e étage |
| 62 | défaite | 3 | 3261 | [729, 1757] | boss | rejeu : 1re mise à terre f2986 (étage 3), sources des 300 frames d'avant : roi_rat 23, franc_tireur 18, brute 16 ; survivant seul 274 frames ; balles min 522 |
| 63 | soft-lock | 3 | 5077 | [568, 2123] | ennemi non atteint : franc_tireur + tourelle | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 65 | défaite | 3 | 3569 | [766, 2161] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3568, `deaths` 1/2, 1408 frames au 3e étage |
| 68 | défaite | 3 | 4181 | [982, 2223] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4180, `deaths` 1/2, 1958 frames au 3e étage |
| 74 | défaite | 3 | 3237 | [874, 1955] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3236, `deaths` 1/2, 1282 frames au 3e étage |
| 76 | soft-lock | 3 | 11401 | [575, 1504] | boss vivant (roi_rat 315,5 PV) | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 81 | soft-lock | 3 | 6491 | [1057, 2131] | portail ouvert non pris | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 83 | défaite | 3 | 3026 | [667, 1512] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3025, `deaths` 1/2, 1514 frames au 3e étage |
| 85 | défaite | 3 | 6317 | [784, 2446] | coéquipier non relevé (à terre au 2e étage) | rejeu : 1re mise à terre f1488 (étage 2), sources des 300 frames d'avant : cracheur 36, arroseur 24, buffle 12 ; survivant seul 4828 frames ; balles min 732 |
| 98 | défaite | 3 | 4007 | [657, 1794] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4006, `deaths` 1/2, 2213 frames au 3e étage |
| 103 | défaite | 3 | 2727 | [642, 1627] | tireurs | rejeu : 1re mise à terre f2406 (étage 3), sources des 300 frames d'avant : arroseur 32, cracheur 24, buffle 12 ; survivant seul 320 frames ; balles min 762 |
| 111 | soft-lock | 2 | 3224 | [1159] | ennemi non atteint : arroseur | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 117 | défaite | 3 | 3747 | [1098, 2225] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3746, `deaths` 1/2, 1522 frames au 3e étage |
| 118 | soft-lock | 3 | 5539 | [675, 1592] | boss vivant (roi_rat 60 PV) + tourelle | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 124 | défaite | 3 | 3677 | [909, 2065] | boss | rejeu : 1re mise à terre f3041 (étage 3), sources des 300 frames d'avant : roi_rat 54, cracheur 6 ; survivant seul 635 frames ; balles min 522 |
| 130 | défaite | 3 | 3117 | [677, 1963] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3116, `deaths` 1/2, 1154 frames au 3e étage |
| 131 | défaite | 3 | 3714 | [761, 2576] | coéquipier non relevé (à terre au 2e étage) | rejeu : 1re mise à terre f1370 (étage 2), sources des 300 frames d'avant : arroseur 52, cracheur 12, pillard 9 ; survivant seul 2343 frames ; balles min 762 |
| 133 | défaite | 3 | 3438 | [621, 1663] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3437, `deaths` 1/2, 1775 frames au 3e étage |
| 138 | défaite | 3 | 4032 | [1032, 2242] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4031, `deaths` 1/2, 1790 frames au 3e étage |
| 139 | soft-lock | 2 | 3015 | [873] | ennemi non atteint : pillard | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 144 | défaite | 3 | 4409 | [795, 1830] | boss | rejeu : 1re mise à terre f2799 (étage 3), sources des 300 frames d'avant : roi_rat 16, brute 8 ; survivant seul 1609 frames ; balles min 552 |
| 149 | soft-lock | 3 | 4185 | [772, 1503] | ennemi non atteint : tourelle + franc_tireur | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 156 | défaite | 3 | 4428 | [527, 2184] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4427, `deaths` 1/2, 2244 frames au 3e étage |
| 162 | soft-lock | 3 | 6712 | [961, 2434] | ennemi non atteint : brute | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
| 163 | défaite | 3 | 3288 | [581, 1651] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3287, `deaths` 1/2, 1637 frames au 3e étage |
| 165 | défaite | 3 | 3305 | [592, 1723] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3304, `deaths` 1/2, 1582 frames au 3e étage |
| 173 | défaite | 3 | 3365 | [829, 2105] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3364, `deaths` 1/2, 1260 frames au 3e étage |
| 175 | défaite | 3 | 3898 | [738, 1944] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3897, `deaths` 1/2, 1954 frames au 3e étage |
| 179 | défaite | 3 | 3795 | [503, 1278] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3794, `deaths` 1/2, 2517 frames au 3e étage |
| 181 | défaite | 3 | 3127 | [642, 2085] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3126, `deaths` 1/2, 1042 frames au 3e étage |
| 183 | défaite | 3 | 3380 | [795, 1876] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f3379, `deaths` 1/2, 1504 frames au 3e étage |
| 185 | défaite | 3 | 4479 | [1068, 2113] | mise à terre puis survivant seul (source non rejouée) | JSON : `run_end` Defeat f4478, `deaths` 1/2, 2366 frames au 3e étage |
| 200 | soft-lock | 3 | 7448 | [1121, 2048] | ennemi non atteint : brute + franc_tireur | `softlock.final_state` (ennemis restants, `floor.portal_open`) |
