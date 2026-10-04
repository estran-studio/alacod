# Rapport — m1-v3-bots-pathfinding : `prudent` et `fonceur` naviguent par le champ (suite T1.14, voie V3)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport). **Base** : branche partie de
`61b4f58` (LIVRÉ throne phase 2 : contient T1.10, throne p1 et p2, et `origin/m1-v1e-horloges`
`e0c9fd6`) ; `origin/main` `e26a3e3` inclus. Fiche locale `docs/taches/m1-v3-bots-pathfinding.md`
(non poussée). Worktree `alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude
Code (b1) ; date : 2026-10-04.

## État en cours

- **Fait** : navigation de `prudent`/`fonceur` en mode `Floors`, critère throne 20/20, testbed
  20/20, §24.
- **Chiffres** : throne 17/20 → **20/20** ; une seule trace existante change
  (`bot_floors_three`) ; 4 traces `throne_*` réévaluées (non bénies).
- **Prochaine étape** : LIVRÉ, attente de la vérification de l'orchestrateur.

## Fait

- `crates/bots/src/navigation.rs` : `DirectNavigation` (cache séparé du `BotNavigation` de
  `chasseur`/`acheteur`), `walls_clear` (ligne de vue sur les seuls `Wall`, sans champ).
- `BotView` : `enemy_visible`, `route`. `read_bot_inputs` (`ReadInputs`, hors rollback) calcule,
  **en mode `Floors` seulement**, la ligne de vue vers l'ennemi le plus proche et, quand une règle
  s'en sert, le pas suivant : `chase` (postes de tir de l'ennemi le plus proche par le chemin),
  sinon `investigate` (point accessible le plus proche), ou `approach` du portail (rayon 16) ;
  composantes de moins de 2 px annulées.
- `decide` : `prudent` — esquive puis recul sous 180 inchangés ; ennemi caché ou au-delà de 320 →
  `route` ; visible dans la bande → immobile ; portail au-delà de 48 → `route`, puis approche
  freinée. `fonceur` — ennemi caché → `route`, portail → `route`. Sans route : ligne droite en
  repli. Tir inchangé.
- Tests : `ennemi_cache_suivre_le_chemin`, `portail_par_le_chemin` (41 tests `bots`).
- `throne` : horloge d'étage ramenée à `alerte` 15 s / `renfort` 30 s puis toutes les 15 s (les
  étages durent maintenant de 10 à 25 s : à 20 s l'alerte ne sonnait plus) ; scénarios `throne_*`
  recalés.
- §24 v1 : paragraphe « Navigation de `prudent`/`fonceur` ».

## Décision prise en cours : navigation en mode `Floors` seulement

Première version (navigation partout) : `clone_quad` régressait (vague 4 au lieu de 5 à f4372,
10 kills au lieu de 13) et `bots_four_mixed`, `bots_two_fonceurs`, `bot_prudent_dodge` changeaient
de trace. En vagues, un zombie dehors est « caché » derrière les murs jusqu'à sa fenêtre : les
bots quittaient la salle pour aller le chercher. En `Floors`, il faut au contraire trouver chaque
ennemi puis le portail. Hors `Floors`, `enemy_visible` vaut `true` et `route` `None` : décision
**identique au bit près à T1.14** (les inputs ne dépendent que de la vue).

## Traces

Une seule trace existante change, **`bot_floors_three`** (testbed, mode `Floors`, deux `prudent`) :
la navigation y tourne, passages à f135, f390, f792 (f135, f388, f796 en ligne droite), attentes
vertes ; à bénir. **`bots_four_mixed` et `clone_quad` : traces identiques** (mode vagues, inputs
de `prudent` inchangés hors `Floors` ; ils ne sont pas figés en `Scripted`, c'est le filtre de
mode qui les protège). `bots_two_fonceurs`, `bot_prudent_dodge`, `bot_prudent_nododge` (inputs
`Scripted`) : identiques. `throne_floor_1`, `throne_three_floors`, `throne_progression` : sans
trace bénie encore (livraisons p1/p2), attentes recalées sur la navigation ;
`throne_mutation_choice` (inputs `Scripted`) inchangé.

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=2`, profil headless.

- **Throne** (`alacod-sim --game throne --bots 2 --profiles prudent,prudent --floors run --seeds
  1..20 --until-floor 3 --max-frames 12000`) : **20/20**, 0 desync, 0 mort, 72 points de dégâts,
  de 1 759 à 3 543 frames (17/20 avant : graines 5, 12, 20 résolues). Simulation de 78 à
  118 fps par graine (154 sans navigation : coût du champ 8 px par bot quand il sert). Étape
  intermédiaire : sans zone morte ni calcul paresseux, 19/20 (graine 2 : bots plaqués en diagonale
  contre un coin de mur), corrigé.
- **Testbed** (`--game testbed --floors trois_niveaux --seeds 1..20 --until-floor 3`) : **20/20**,
  0 mort, 30 points de dégâts, de 524 à 1 889 frames.
- `make test_scenarios` (sans bless) : traces existantes identiques sauf `bot_floors_three` ;
  autres échecs uniquement « pas de trace de référence » (4 `throne_*`, 7 de T1.10, 3 de T1.9).
  `bench_horde` 33,5 fps sous charge (plancher 38 ; aucun bot dans les benchs).
- `cargo test` des quatorze crates : **487 réussis, 1 échec, 9 ignorés** (l'échec : `scenarios`,
  pour les raisons ci-dessus) ; `bots.rs`, `hunter_doors.rs` verts.
- `make lint` (trois jeux), `cargo fmt --all -- --check` vide (après `cargo fmt`, bots seulement),
  `check-forbidden.sh` 4 occurrences préexistantes, `check-rollback-registration.sh` OK, `make gen`
  testbed et zombies sans modification (throne : rouge, attend T1.13), `cargo check -p throne`
  et exemples code 0.

## Non fait / dettes

- Hors `Floors`, `prudent`/`fonceur` restent en ligne droite (voulu : vagues).
- Coût : environ ×0,6 sur la vitesse de simulation des runs de bots en `Floors` ; un cache de champ
  par cible stable serait la suite si ça gêne les métriques.
- Esquive des zombies au contact, bots en jeu fenêtré : hors périmètre.
