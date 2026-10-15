# Correctif de visibilité demandé pendant la revue M1

2026-10-10, base `f53cd1ae`, branche `m1-revue-humaine`.

William : « bloquand , doit etre fixer avant de faire les tests sinon je voie pas les enemies ».
Cette demande autorise le changement de contenu dans un commit distinct de la revue.

## Cause et correction

Les onze ennemis des cavernes avaient des skins aux calques vides, et aucune entrée dans
`games/throne/assets/sprites/sprites.ron`. La présentation ne pouvait donc créer leurs sprites.

Les onze identifiants et tous leurs skins (y compris `rapide` pour rat/chien) utilisent désormais
un corps animé et une ombre. Les images `Zombie.png` et `ZombieHard.png` sont reprises à
l'identique de zombies, avec leur attribution CC BY 4.0 dans `assets.yaml`.
Brute, buffle, tourelle et roi_rat utilisent le deuxième corps ; les autres utilisent le premier.
L'échelle existante du boss est conservée. Les personnages de laboratoire cible/mannequin
ne sont pas concernés. Aucun code moteur, chiffre de gameplay ou comportement n'est modifié.

Ce sont des visuels provisoires : ils rendent les ennemis visibles mais n'offrent pas une
silhouette propre à chaque type. La distinction en combat reste à juger par William.

## Preuves de rendu

Captures à f120 des scénarios générés `enemy_rat_still` et `enemy_roi_rat_still`, réalisées avec
`play_scenario --capture <dossier> --every 120` ; les deux sprites sont visibles :

- [Rat](m1-f53cd1ae-visibilite/rat.png)
- [Roi rat](m1-f53cd1ae-visibilite/roi-rat.png)

Les onze entrées, tous les skins et tous les chemins de feuilles/images ont été vérifiés.
SHA-256 des images, identiques aux fichiers sources de zombies :

```text
Zombie.png      57d222caee9188a2b591003995fec3c61d36d32d8ad32a60b2506fb812f56581
ZombieHard.png  68dc014e778b2f654d2a0a796aa85f4ba8f099a31a9204b1b0be7282a22f750d
```

## Validation et traces

Les calques `ActiveLayers` sont sous checksum : les traces des scénarios qui portent ces
ennemis changent. Réenregistrement délibéré des traces throne, sans modification des scripts
RON de scénarios. Les traces des autres jeux ne sont pas touchées.

- `alacod lint games/throne` : aucune erreur.
- `make gen GAME=throne GEN_BLESS=1`, puis `make gen GAME=throne` : 39 scénarios verts
  (13 armes à distance, 4 de corps à corps, 22 gabarits de personnage ; le digest historique
  en comptait 37).
- Rejeu sans bless : `throne_progression`, `throne_ammo_pickup`, `throne_duo_defaite`,
  `throne_floor_1`, `throne_mutation_choice`, `throne_softlock_recul`, `throne_solo`,
  `throne_quad` et `throne_three_floors` verts (9 scénarios manuscrits).
- Comparaison détaillée avant/après par `scripts/trace-diff.py --ignore ActiveLayers` :
  `throne_solo`, 3699 frames identiques ; `throne_quad`, 3399 frames identiques ;
  `throne_three_floors`, 4999 frames identiques. Aucun autre changement d'état détecté.
- Pour le duo, les 869173 paires de lignes des dumps complets ont d'abord été comparées
  position par position : 844269 paires strictement identiques ont été retirées des deux
  fichiers aux mêmes positions, et les 24904 paires restantes ont été conservées avec tous
  les en-têtes de frames. Le comparateur officiel a ensuite contrôlé ces fichiers réduits
  avec le seul filtre `ActiveLayers`. Cette réduction évite de reparcourir les longues
  ressources identiques, sans soustraire aucune différence à la preuve.
- Les 30 traces modifiées sont exclusivement celles de throne. Leurs frames et nombres
  d'entités restent identiques ; seuls leurs checksums changent.

Dumps complets locaux : `/tmp/alacod-m1-visuals-before/<scenario>.full` et
`/tmp/alacod-m1-visuals-after/<scenario>.full`. Commande de preuve pour chaque run :

```sh
python3 scripts/trace-diff.py /tmp/alacod-m1-visuals-before/throne_solo.full \
  /tmp/alacod-m1-visuals-after/throne_solo.full --ignore ActiveLayers
```

Le budget de quad était déjà sous 40 fps pendant la référence avant correctif sur ce MacBook
chargé. Les mesures avec dumps et compilations simultanées ne constituent pas une validation
de performance au calme. Les attentes et les invariants restent les critères du correctif.

## Reprise de la revue

La confirmation de William que le rendu permet la revue reste à recueillir sur le jeu.
Ce correctif ne constitue pas le verdict de sortie de M1 ; les runs humains et les décisions
de la fiche restent à réaliser.
