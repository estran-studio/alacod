# Rapport — m1-v2-contenu-throne-p2 : `throne`, phase 2 (mutations, progression, horloge, difficulté)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport). **Base** : branche partie de
`c18b8a9` (LIVRÉ throne phase 1, qui contient T1.10 `4f772d8`), avec **`origin/m1-v1e-horloges`
`e0c9fd6` mergée** (T1.9 de b0, pas encore vérifiée par l'orchestrateur ; merge `b1ca5e1`) ;
`origin/main` `e26a3e3` inclus. Si la vérification de T1.9 ou T1.10 impose un correctif, remerger
`origin/main`. Fiche : [m1-v2-contenu-throne](../m1-v2-contenu-throne.md) (phase 2). Worktree
`alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ; date :
2026-10-04.

## État en cours

- **Fait** : phase 2 (progression, huit mutations, horloge d'étage, difficulté, actives par le
  manifeste), trois scénarios nouveaux, deux réévalués, gabarit d'armes préparé pour T1.13, §29.
- **Chiffres** : 17/20 graines finissent la run, 0 desync, 0 mort ; traces existantes identiques.
- **Prochaine étape** : LIVRÉ, attente de la vérification de l'orchestrateur.

## Fait

- **Merge de T1.9** : conflits limités aux champs ajoutés en parallèle par T1.9 et T1.10
  (manifeste, registre, kinds, `Scenario`, littéraux de test, `game.ron` du testbed) : tous
  gardés.
- **Manifeste** : `entry: (..., progression: "run", clocks: ["etage"], difficulty: true)` : actifs
  en partie jouée **et** dans tous les scénarios de `throne`.
- **`progression/run.ron`** : un rad par kill, niveaux 3 / 8 / 15, trois choix, 600 frames,
  `weapon_pool` par niveau (0 : balles ; 1 : obus et lames ; 2 : énergie ; 3 : explosifs), drop
  0,08.
- **Huit mutations** (effets v1 seulement) : `coriace`, `vampire`, `tireur` (pattern `salve`),
  `adrenaline`, `rancune`, `sang_froid`, `chasseur` (cible `champion` = élite `blinde`),
  `irradie` (`GaugeAdd`).
- **Horloge** `clocks/etage.ron` (`Floor` : `alerte` 20 s, `renfort` 40 s puis toutes les 20 s) et
  **difficulté** `1 + floor * 0.25 + floor_minutes * 0.25`.
- **Scénarios** : `throne_progression` (un bot : niveau 1 à f528, rads, `sang_froid` prise d'office
  à f1128, niveau 2, horloge `alerte` à f2125 dans l'étage 1) ; `throne_mutation_choice` (inputs
  enregistrés du même bot par `alacod-sim --save-scenario`, rejoués avec `ChoiceB` de f558 à f562
  → `chasseur` au lieu de `sang_froid`) ; `throne_floor_1` et `throne_three_floors` **réévalués**
  avec le jeu complet (butin compris : `powerup_drop_chance_override` retiré).
- **Gabarit pour T1.13** : `gabarit_armes.ldtk` (copie de l'arène du testbed, `cible` à
  `counts_hits` à +128/−48 du spawn, `mannequin` ailleurs), personnages `cible` et `mannequin` ;
  `generate_template` viendra avec T1.13 (b0 ajoute `mode` au scénario pour forcer `Sandbox`).
- **Docs** : §29 complété (phase 2).

## Traces

Les traces `throne_*` de la phase 1 (bénies au merge de p1) **changent** en p2 : progression,
horloge, difficulté et butin y jouent désormais (attendu, nouveau jeu ; accord de
l'orchestrateur). `throne_floor_1` garde son portail à f925 ; `throne_three_floors` passe à f655,
f1436, f2120. Nouvelles : `throne_progression`, `throne_mutation_choice`.

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=2`, profil headless, sur
l'état fusionné (T1.9 comprise).

- **Critère des 20 graines** (`alacod-sim --game throne --bots 2 --profiles prudent,prudent
  --floors run --seeds 1..20 --until-floor 3 --max-frames 12000`, tout actif) : **17/20**,
  0 desync, 0 mort, 225 points de dégâts au total (difficulté), de 1 654 à 3 811 frames. Écarts :
  - **graine 5** (étage 2) : le `pillard` en fuite reste derrière un mur, comme en phase 1 ;
  - **graines 12** (étage 3) et **20** (étage 2) : plus aucun ennemi, portail ouvert, les deux bots
    coincés dans un recoin en marchant vers le portail en ligne droite.
  Les trois relèvent de la dette de pathfinding de `prudent` (T1.14) : nouvelles positions
  d'arrivée dues à la difficulté et au butin. Essai `fill_ratio` 0,35 : **15/20** (pire, d'autres
  recoins) ; gardé à 0,38.
- Scénarios : `throne_progression`, `throne_mutation_choice`, `throne_floor_1` (f925),
  `throne_three_floors` (f655, f1436, f2120, une mutation chacun) verts sur leurs attentes.
- `make test_scenarios` (sans bless) : **toutes les traces existantes identiques** ; échecs
  uniquement « pas de trace de référence » : les 4 `throne_*`, les 7 de T1.10, les 3 de T1.9
  (`clock_events`, `clock_floor_reset`, `difficulty_scales`). `bench_horde` 32,8 fps sous charge
  (plancher 38).
- `cargo test` des quatorze crates : **485 réussis, 1 échec, 9 ignorés** (l'échec : `scenarios`,
  traces nouvelles).
- `make lint` sans erreur (trois jeux ; throne : 13 personnages, 13 armes, 4 de mêlée, 1 carte) ;
  `cargo fmt --all -- --check` vide ; `check-forbidden.sh` 4 occurrences préexistantes ;
  `check-rollback-registration.sh` OK ; `make gen GAME=testbed` et `GAME=zombies` code 0, aucun
  fichier modifié ; `make gen GAME=throne` rouge comme en phase 1 (attend T1.13) ;
  `cargo check -p throne` et exemples code 0.
- Gabarit `gabarit_armes.ldtk` : lint vert, **pas jouable aujourd'hui** (en mode `Floors`, un
  scénario de throne ignore sa carte : vérifié, il joue `niveau_1`) ; T1.13 ajoute `mode` au
  scénario.

## Non fait / non vérifié

- Horloge sans effet de jeu en v1 (aucune action sur ses événements, `OnEvent` v2) ;
  `renfort` ne fait rien.
- Écran de choix (T1.16), HUD des rads (T1.18) ; jeu fenêtré non lancé ; pas de p2p.
- `test:` des ennemis et scénarios générés de throne (T1.13).

## Dettes / questions ouvertes

- **Pathfinding de `prudent`** : seule cause des 3 graines perdues ; c'est elle qui borne le
  critère des 20 graines sur des cavernes générées.
- Les bots ne choisissent jamais de mutation (toujours la première option, 600 frames après le
  niveau) : un profil de bot qui choisit serait utile aux métriques.
- `FloorEntered` et `OnFloorEntered` (§23) : rien ne les consomme encore dans throne.
