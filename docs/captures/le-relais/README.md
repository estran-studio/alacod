# Le Relais — petit bâtiment à défendre

William choisit le 2026-10-09 un bâtiment compact à sécuriser, avec les zombies
qui arrivent de l'extérieur. Carte de travail :
`games/zombies/assets/maps/le_relais_prototype.ldtk`. Elle garde les assets actuels
et ne remplace pas encore la carte de départ du jeu.

![Quatre orientations du bâtiment](salles-prototype.svg)

## Disposition

Chaque plan contient cinq pièces : accueil, atelier, infirmerie, radio et réserve.
Les passages intérieurs sont courts ; quatre portes relient les quatre grandes
zones en boucle et une cinquième donne accès à la réserve. Une sixième porte
permet de sortir dans la cour. Les portes utilisent les prix du générateur actuel
(750 points dans ce niveau unique). La radio et l'extraction ne sont pas encore
interactives ; les noms indiquent leur rôle prévu dans la conception.

- Quatre départs joueurs **dans l'accueil**, avec dégagement pour leurs corps.
- Huit spawners **uniquement dehors**, avec une marge pour l'offset de spawn.
- Huit fenêtres barricadées dans les murs extérieurs : tirs au travers,
  réparation, destruction puis entrée des zombies ; elles bloquent les joueurs.
- Pistolet dans l'accueil, fusil à pompe dans l'atelier, juggernog à l'infirmerie,
  mitraillette dans la réserve.
- Une bande extérieure continue permet aux zombies de contourner le bâtiment
  jusqu'à une fenêtre donnant vers une pièce accessible.

LDtk 1.5.3, grille 16 px, emprise 544 × 480 px (34 × 30 cellules), bâtiment
352 × 288 px extérieur compris. La cour et les cinq pièces forment **un niveau
LDtk unique** : les pièces sont matérialisées par les murs et portes intérieurs.
Le générateur choisit parmi quatre gabarits de départ correspondant aux quatre
réflexions du plan. Ce prototype vérifie la défense du bâtiment ; la composition
procédurale des pièces et les permutations d'équipements restent à construire
avec les interfaces de M2. Quatre orientations ne prouvent pas une variété
stratégique suffisante pour la démo finale.

Le premier prototype assemblait cinq gabarits indépendants avec des spawns
intérieurs. Son pilote était bloqué : le mode vagues sélectionne par distance,
sans filtre par salle, et pouvait choisir un zombie derrière des portes fermées.
Le nouveau plan rend les spawners accessibles depuis l'accueil par l'extérieur
et une fenêtre cassable, même avec toutes les portes fermées. Le moteur n'a
pas été modifié ; ce problème reste pertinent pour d'autres configurations.

## Vérifications

- Lint zombies : aucune erreur.
- Tous les départs joueurs et spawners sont dégagés des murs.
- Flood-fill sur une grille de 8 px avec corps de 20 × 20 px : les huit spawners
  sont reliés à l'accueil, portes fermées et fenêtres considérées franchissables
  par les zombies, pour chacune des quatre orientations.
- Génération native sur graines 1..20 : un bâtiment par carte, quatre plans
  distincts, chacun tiré cinq fois. Les translations du monde sont ignorées.
- Pilote graine 1, quatre acheteurs, jusqu'à l'entrée en vague 5 : **7236 frames**,
  **27 kills**, **5 portes ouvertes**, **15 dégâts**, **0 mort**, aucune failure
  d'invariant, aucun desync ni soft-lock rapporté. Les combats de V5 ne sont pas
  inclus dans cette mesure. La campagne complète est terminée : **20/20** graines
  atteignent V5, **0 mort, mise à terre, desync ou failure d’invariant**,
  **134 dégâts cumulés**. Aucun arrêt pour soft-lock n’est rapporté.
- Entrée V5 : médiane **6940 frames** (≈ 116 s), plage **6455–18713**.
  La graine 11 est un cas à revoir humainement : V3 dure **12754 frames**
  (≈ 213 s entre première apparition et fin), avec une stagnation prolongée
  des kills. Elle termine ensuite ; ce succès ne valide pas la fluidité de
  la navigation dans toutes les situations.

Artefacts détaillés dans `target/metrics/le-relais/` (ignorés par git).
Le ressenti humain du nouveau plan reste à valider.

## Édition

Ouvrir `maps/le_relais_prototype.ldtk` dans LDtk pour examiner les quatre plans.
`WindowHorizontal` mesure 16 × 32 dans les définitions existantes et sert aux
murs verticaux ; `WindowVertical` mesure 32 × 16 et sert aux murs horizontaux.
Les dimensions physiques déterminent l'orientation.

Le script de construction est versionné : `scripts/construire-le-relais.py`.
Il **réécrit** la carte prototype à partir des définitions d'avant_poste ; conserver
les modifications manuelles LDtk avant de le relancer. Son cache de tuiles est
un décor provisoire. Il reste à habiller sols, murs et équipements avec un
ensemble visuel cohérent après validation du parcours.

```sh
python3 scripts/construire-le-relais.py
APP_VERSION=x cargo run --example map_generation --profile headless --no-default-features -- \
  games/zombies/assets/maps/le_relais_prototype.ldtk /tmp/le-relais-graine-1.ldtk 1
```

## Relevé de la campagne

Quatre acheteurs, paramètres S5 B, graines 1..20, jusqu’à **l’entrée** en V5,
plafond 20000 frames. Les essais sont répartis en quatre lots indépendants et
un pilote ; leurs temps muraux ne constituent pas un benchmark.

| Graine | Frame V5 | Kills | Portes ouvertes | Dégâts |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 7236 | 27 | 5 | 15 |
| 2 | 7323 | 30 | 6 | 0 |
| 3 | 6550 | 28 | 6 | 15 |
| 4 | 6909 | 29 | 6 | 8 |
| 5 | 6971 | 28 | 6 | 35 |
| 6 | 6631 | 29 | 6 | 0 |
| 7 | 7256 | 29 | 6 | 5 |
| 8 | 7232 | 26 | 4 | 0 |
| 9 | 6795 | 28 | 6 | 0 |
| 10 | 7014 | 29 | 6 | 5 |
| 11 | 18713 | 28 | 6 | 8 |
| 12 | 7055 | 29 | 6 | 0 |
| 13 | 6875 | 29 | 6 | 3 |
| 14 | 6775 | 26 | 6 | 0 |
| 15 | 6455 | 27 | 6 | 5 |
| 16 | 6870 | 29 | 6 | 35 |
| 17 | 7430 | 31 | 6 | 0 |
| 18 | 7132 | 31 | 6 | 0 |
| 19 | 6639 | 27 | 6 | 0 |
| 20 | 6796 | 30 | 6 | 0 |

Commande de mesure (les lots réels couvrent 1, 2..6, 7..11, 12..16, 17..20) :

```sh
APP_VERSION=x target/headless/alacod-sim --game zombies --bots 4 \
  --map maps/le_relais_prototype.ldtk --seeds 1..20 --until-wave 5 \
  --max-frames 20000 --progress --json /tmp/le-relais-mesures.json
```

La suite historique complète n’a pas été relancée pour cette passe de contenu :
le manifeste et les cartes de ses scénarios ne changent pas. Aucun bless de
trace, aucun changement Rust moteur, aucune nouvelle texture externe.
