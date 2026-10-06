# m0-200-graines-test-map — critère M0 §9.8 rejoué sur `test_map` (local)

## État en cours

- Lancé le 2026-10-05 en local (orch : la session cloud n'a jamais livré), binaire
  `alacod_tasks/m1-200-throne/alacod-sim-f80b82b` (aucune compilation), deux lots de 100 graines
  en parallèle, sorties dans `alacod_tasks/m0-200-test-map/`. Branche depuis origin/main `ddb9789`.
- **Pause** (contre-ordre d'orch, priorité aux mesures de b1) : les deux lots sont arrêtés après
  une graine chacun. Graine 1 : vague 5, f7291, 0 mort, 52 kills ; graine 101 : vague 5, f7632,
  0 mort, 53 kills (≈ 12 fps, 10 min par graine sous contention). Journaux gardés
  (`sim-*.log.partie1`). `alacod-sim` n'écrit son JSON qu'en fin de lot : les JSON de 1 et 101
  sont perdus. Reprise au signal d'orch : plages `2..100` et `102..200`, et 1 et 101 rejouées
  en même temps (20 min) pour avoir leur JSON, puis fusion.

## Préparation (texte seul) : volet « contenu `niveau_3` », tâche suivante

Pour après la remesure des 200 graines de throne (correctifs de bots de b1). Rien n'est fait ni
mesuré ici ; chiffres tirés du contenu et des analyses déjà livrées.

**Ce qu'on sait.**
- Toutes les défaites des 200 graines (39/39) sont au troisième étage ; la première mise à terre
  vient des tireurs (7/12, l'arroseur seul 6/12) ou du boss (5/12) ; le boss et les tireurs font
  82 % des dégâts reçus à l'étage (digest m1-200-graines-throne).
- `niveau_3` : 9 ennemis, PV de base 910 (roi_rat 360, tourelle 150, brute 120, buffle 90,
  arroseur 50, franc_tireur 50, cracheur 40, rat 30, chien 20). Difficulté
  `1 + floor × 0,25 + floor_minutes × 0,25` : ×1,5 à l'entrée (1 365 PV), puis +0,25 par minute
  passée dans l'étage, sur les PV **et** les dégâts des ennemis.
- D50 (graine 76, b1) : aucun butin de munitions de f3180 à f12661, deux bots à sec dès f11280
  devant une tourelle et le boss → soft-lock. Butin : `drop_chance` 0,25, munitions 96/146 du
  poids, balles 36/146 : une recharge de balles pour ≈ 16 kills en moyenne (0,25 × 36/146 ≈ 6 %),
  et `niveau_3` n'a que 9 ennemis. À l'inverse, les 12 défaites rejouées n'avaient pas de pénurie
  (≥ 432 balles) : D50 concerne les étages qui durent (soft-locks, combats lents), pas les
  défaites rapides.
- Les bots perdent 74 % de leurs balles au troisième étage (avant correctifs) : tout réglage de
  contenu mesuré avant les correctifs de bots serait calé sur un niveau de jeu faux.

**Trois leviers, à mesurer un par un** (20 graines témoins fixes : 25, 76, 103 et 17 graines
d'échec et de réussite des 200 ; puis 200 graines pour le retenu) :
1. **D50, munitions garanties** (contenu, `items/powerups.ron` + règle de butin) : butin pondéré
   par les réserves des joueurs (poids des munitions ×k quand la réserve d'un type est sous un
   seuil) ou une munition garantie tous les N ennemis tués. Effet attendu : plus de soft-lock « à
   sec » (graine 76) ; peu d'effet sur les défaites rapides. Traces : toutes celles de throne avec
   butin changent si la règle touche le tirage ; une règle « seulement sous un seuil » ne change
   que les runs où une réserve tombe sous le seuil.
2. **Retirer l'arroseur de `niveau_3`** (contenu, `caves/niveau_3.ron`, `characters`) : première
   source de dégâts avant la mise à terre (35 %), −75 PV à l'entrée ; `enemy_spawns` 9 → 8 garde
   le boss (point 0) et la tourelle. Traces : seulement les runs qui atteignent le troisième étage.
3. **Pente de difficulté 0,25 → 0,15 par étage** (`difficulty.ron`) : ×1,3 au lieu de ×1,5 à
   l'entrée du troisième étage (boss 468 PV au lieu de 540, dégâts reçus −13 %), et ×1,15 au
   deuxième. Plus large que (2) : touche aussi le deuxième étage et toutes les traces throne.

**Recommandation** : D50 d'abord (un soft-lock prouvé, indépendant du niveau des bots), puis (2)
plutôt que (3) si les défaites au boss restent au-dessus de ~10 % après les correctifs de bots :
(2) vise la cause mesurée (tireurs) sans changer les deux premiers étages ni toutes les traces.
- **Référence de la mesure** : binaire `alacod-sim-f80b82b` (`main` `f80b82b`), donc **avant D51
  et m1-v3-bots-armes** (41 commits de retard sur `main` `ed8a274` au 2026-10-06). b1 a mesuré
  `zombies` inchangé après D51 (20/20 sur 20 graines) : le résultat reste valable pour M0, mais
  il porte sur `f80b82b`.
- **Reprises** : la session s'est fermée dans la nuit du 5 au 6 (simulation arrêtée avec elle,
  pas par la mémoire) ; reprise le 2026-10-06 à 11 h 15 en une seule file : 157 graines déjà
  jouées (1 à 100, 102 à 150), reste 151 à 200 puis 101 (pour son JSON). Point provisoire : seule
  la graine 100 échoue (soft-lock vague 1).
