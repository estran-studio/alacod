# Rapport — m1-v3-bots-v1 : bots v1 (T1.14, voie V3)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport) ; base `origin/main`
`22d6c73` (T1.7 livrée, pas encore poussée sur main au moment de la livraison : la branche part
de sa tête `de09670`). Fiche : [m1-v3-bots-v1](../m1-v3-bots-v1.md). Branche `m1-v3-bots-v1` ;
worktree `alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ; agent : Claude Code (b1) ; date :
2026-10-04 (nuit).

## État en cours

- **Fait** : tout le périmètre de la fiche, avec deux extensions acceptées par l'orchestrateur
  (gestion d'arme de `prudent`, niveau `floor_d`) et une troisième trouvée par le critère des
  20 graines (approche freinée du portail).
- **Chiffres** : 20/20 graines finissent trois niveaux, 0 desync, 0 softlock, 0 mort ;
  `bot_prudent_dodge` 8 points encaissés contre 56 sans esquive.
- **Prochaine étape** : LIVRÉ, attente de la vérification de l'orchestrateur.

## Fait

- **Vue** (`crates/bots/src/view.rs`) : `ProjectileView { position, velocity, size, distance }`
  — toute `Bullet` (ordinaire ou composable) d'une équipe autre que les joueurs, à moins de
  320 px, les 16 plus proches, triées par `GgrsNetId` (`projectile_views`) ; `body_radius`
  (demi-diagonale du collider) ; `reload`, `switch_weapon`, `trigger_ready` (gestion d'arme) ;
  `velocity` (freinage au portail).
- **Esquive** (`crates/bots/src/dodge.rs`, pure en `Fixed`) : pour chaque projectile qui
  approche, approche minimale sur 30 frames ; menace sous rayon du corps + taille + 8 px ;
  perpendiculaire à la vitesse, du côté qui s'éloigne, somme normalisée.
- **`prudent` v1** : l'esquive remplace le déplacement (tir conservé) ; gestion d'arme
  (recharge, changement d'arme vide, détente relâchée pour `Manual`/`Shotgun`/`Burst`) ;
  approche freinée du portail. `fonceur`, `immobile`, `chasseur`, `acheteur` inchangés.
- **`nearest_enemy_any`** (décision 3) : **pas de champ ajouté**. La requête `enemies` de la vue
  n'a aucune limite de portée : `nearest_enemy` ne vaut `None` que s'il ne reste aucun ennemi,
  et `prudent`/`fonceur` marchaient déjà vers l'ennemi le plus proche, aussi loin soit-il.
- **`alacod-sim`** : `--until-floor <n>` (exige `--floors`, `stop_condition` testée) ;
  `--until-wave` ou `--until-floor` obligatoire ; `SimResult.floor_frames` (moments clés
  `floor`), `damage_taken`, `dodges` (`bots::BotStats`, hors rollback).
- **Runner** : `StopEarly.until_floor` ; soft-lock `Floors` (1 200 frames sans passage ni ennemi
  en moins : arrêt avec `softlock`, instantané « précédent » à 600) ; métrique `damage_taken`.
- **`scripts/scenario-metrics.py`** : colonnes `Niveau`, `Frames/niveau`, `Esquives`, résumé
  « niveaux finis / graines ».
- **Contenu** : `testbed/floor_d.ldtk` (copie de `floor_b`, follower une case plus haut) ;
  `trois_niveaux` = `floor_a`, `floor_d`, `floor_c`.
- **Scénarios** : `bot_prudent_dodge`, `bot_prudent_nododge`, `bot_floors_three`.
- **Docs** : `docs/conventions.md` §24 « v1 (T1.14) », ligne `--until-floor` du §17.

## Diagnostic

- **Blocage de `floor_b`** : son follower apparaît en case LDtk (6, 10), juste au-dessus d'un
  mur (rangée 11) ; le corps (20 × 20, décalé de 6 vers le bas) chevauche le mur dès
  l'apparition, il ne bouge jamais (« sans chemin depuis sa case exacte » dans le dump) et le
  niveau ne peut pas finir ; les bots, sortis de la salle en reculant, tiraient dans le mur.
  `floor_b` reste tel quel (trace de `portal_next_floor`) : **dette** — corriger la position du
  follower de `floor_b` (vers (6, 9)) avec le bless de `portal_next_floor`.
- **Orbite autour du portail** (graine 20 au premier passage) : sans ennemi, le portail ouvert,
  les bots le dépassaient à pleine vitesse et tournaient autour (positions relevées toutes les
  100 frames de f1300 à f2300 dans un rayon de ~100 px autour du centre) sans entrer dans son
  rayon de 24 ; le jeu ne freine que si aucun bouton n'est tenu. Approche freinée (§24 v1).
- **Jumeau `bot_prudent_nododge`** (reproductible) : diff local temporaire, non commité, dans
  `crates/bots/src/decide.rs` :
  ```diff
  -    if let Some(away) = crate::dodge::dodge(view) {
  +    if let Some(away) = crate::dodge::dodge(view).filter(|_| false) { // NODODGE-TEMP
  ```
  puis, depuis `alacod/` (après `source ../env.sh`) :
  ```bash
  cargo run -q -p scenario --profile headless --bin alacod-sim -- --game testbed --bots 1 \
    --profiles prudent --map testbed/arena_tir.ldtk --seeds 123456..123456 --until-wave 99 \
    --max-frames 600 --save-scenario <dossier> --json <fichier>
  ```
  et `git checkout -- crates/bots/src/decide.rs`. Le `seed_123456.ron` produit (inputs
  `Scripted`) est le corps du scénario, avec son en-tête et l'attente `Health` à f599. Mesure :
  56 points encaissés (santé 44 à f599) contre 8 avec l'esquive (santé pleine à f599 après
  régénération, un seul coup à f188).

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=2` ou `3`, profil
headless.

- **Critère des 20 graines** :
  `alacod-sim --game testbed --bots 2 --profiles prudent,prudent --floors trois_niveaux
  --seeds 1..20 --until-floor 3 --max-frames 12000` : **20/20 au niveau 3**, 0 desync,
  0 softlock, 0 mort, 34 points de dégâts au total, de 420 à 1 452 frames (graines 9, 13 et
  16 au-dessus de 1 200, les autres entre 420 et 754).
- `bot_prudent_dodge` : un coup (f188), aucun dégât de f200 à f500, santé 100 à f599 ;
  `bot_prudent_nododge` : santé 44 à f599 ; `bot_floors_three` : portails f135, f388, f796,
  deux bots en vie.
- **Scénarios existants avec bots `prudent`** : la fiche indiquait qu'aucun scénario n'en avait,
  or `bots_four_mixed` et `clone_quad` en ont ; rejoués, **traces identiques** (zombies :
  mitrailleuse automatique, pas de projectile ennemi, pas de portail).
- Tests unitaires `bots` : 39 (dont `dodge` : frontal → pas de côté, à côté / qui s'éloigne /
  trop lent → `None`, deux menaces → somme ; `projectile_views` filtrée et triée ; freinage
  au portail) ; `alacod-sim` : `--until-floor` sans `--floors` refusé.
- `make test_scenarios` (sans bless, tête `5bd48e2`) : **les 91 traces de main identiques** ;
  échecs uniquement « pas de trace de référence » pour les 3 scénarios de T1.14 et les 4 de
  T1.7 (livrée, pas encore bénie sur main). Aucune attente en échec.
- `cargo test -q --profile headless -p scenario -p run -p combat -p game -p content -p map_ldtk
  -p map -p sim_core -p stats -p bots -p effects -p behaviors -p world -p utils --no-fail-fast` :
  **425 réussis, 1 échec, 9 ignorés** (45 blocs) ; l'échec est le test `scenarios`, uniquement
  sur les mêmes traces nouvelles.
- `make gen GAME=testbed` et `GAME=zombies` : code 0, aucun fichier modifié ; `make lint` sans
  erreur (deux jeux, 11 cartes testbed) ; `cargo fmt --all -- --check` vide ;
  `check-forbidden.sh` 4 occurrences préexistantes ; `check-rollback-registration.sh` OK ;
  `cargo build --profile headless --examples` code 0.

## Non fait / non vérifié

- Pathfinding pour `prudent` (le flow field 8 px reste à `chasseur`/`acheteur`) : un bot sorti de
  la salle rejoint encore l'ennemi en ligne droite.
- Esquive des zombies au contact, bots dans le jeu fenêtré, métriques vidéo (hors périmètre).
- `EnemyDistance`/`EnemyContactBefore` (T1.4) pas encore dans main.

## Dettes / questions ouvertes

- Follower de `floor_b` en (6, 10) dans le mur : à remonter en (6, 9) avec bless de
  `portal_next_floor` (`trois_niveaux` utilise déjà `floor_d`).
- Fiche m1-v1e-horloges (`clock_floor_reset`) : `trois_niveaux` passe par `floor_d`.
