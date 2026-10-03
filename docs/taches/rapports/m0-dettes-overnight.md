# Lot de dettes « overnight » (D3, D13, D19, D21, D22, D23, D24) — livraison

Tête de branche vérifiée : `fee5345`, soit le merge de `main` `a39956a` (docs seulement)
sur les commits de travail. Le SHA livré ajoute ensuite le commit de ce rapport, purement
documentaire.

- Branche : `m0-dettes-overnight`, depuis `main` `0942abb`.
- Périmètre : les sept lignes D3, D13, D19, D21, D22, D23, D24 de `docs/taches/dettes.md`.
- Date : 2026-10-03. Agent : Claude (Claude Code, session cloud : 4 cœurs, 15 Go).

**Vérification standard verte, hors p2p et bench (non faisables ici).**
- **Aucune trace modifiée** : les 62 traces sont identiques, sans aucun bless.
- **Aucun scénario `.ron` ajouté** : tous les nouveaux tests sont des tests unitaires ou des
  fixtures de lint.
- Ni merge dans `main` ni modification de `docs/taches.md`. Dans `dettes.md`, seules les
  sept lignes du lot ont changé.

## 1. Fait (un commit par dette, puis un commit de doc)

| Commit | Dette | Fichiers |
|---|---|---|
| `71240ee` | D23 | `scripts/scenario-video`, `Makefile` |
| `4aab39b` | D24 | `crates/scenario/tests/scenarios.rs`, `scripts/scenario-metrics.py` |
| `9c7bb62` | D21 | `crates/scenario/src/events.rs`, `scripts/scenario-review.html` |
| `9de34ca` | D22 | `crates/game/src/ui/hud.rs` |
| `2844a65` | D19 | `crates/game/src/recording.rs`, `crates/scenario/src/runner.rs`, `crates/game/src/replay.rs` |
| `f13f58a` | D13 | `crates/game/src/run_state.rs`, `crates/game/src/ui/lobby.rs`, `crates/game/src/core.rs` |
| `b3c74f0` | D3 | `crates/content/src/{registry,lint}.rs`, 4 fixtures, `crates/game/src/global_asset.rs`, `crates/game/src/ui/weapon_visuals.rs`, `games/*/assets/**` |
| `9e8466d` | doc | `CLAUDE.md`, `docs/conventions.md`, `docs/taches/dettes.md` |

### D23 : `SCENARIO=a,b` et `SCENARIO="a b"`

Choix : **les deux syntaxes sont acceptées**, ce qui laisse la doc existante juste.
- `scenario_names` découpe chaque argument sur les virgules et les espaces.
- Un nom sans `tests/scenarios/<nom>.ron` arrête le script avant toute compilation.
- `DRY_RUN=1` liste les scénarios sans compiler ni capturer ; `make videos` saute alors le
  montage.

### D24 : métriques dans le target actif

La vraie cause : le Makefile exporte `CARGO_TARGET_DIR=./target`, un chemin **relatif**, et
`cargo test` lance le binaire depuis `crates/scenario/`.

Choix : **le test retrouve lui-même le target**, quelle que soit la variable.
- Il prend le premier ancêtre de son exécutable qui porte le `CACHEDIR.TAG` posé par cargo.
- À défaut, il lit `CARGO_TARGET_DIR` (un chemin relatif se lit depuis la racine du
  workspace), sinon `<workspace>/target`.
- `scenario-metrics.py` applique la même règle pour un chemin relatif.

### D21 : moments clés

- **Fin de partie** : `victory`, `defeat` et `abandon`, à la frame où `Run.step` devient
  `Ended`. `defeat` garde sa catégorie et son libellé, puisque l'attente `Defeat` les lit.
- **Power-ups** : `powerup` à l'apparition et à l'expiration ; `powerup_pickup` au ramassage,
  avec le joueur, lu dans `FrameEvents<PowerUpPickedUp>`.
- **Armes murales** (`WeaponPickup` avec `price: Some`) : exclues du suivi des armes au sol.
  Leur achat est déjà le moment clé `purchase`.
- La logique est dans deux fonctions pures testées. La page de revue colore les pastilles
  `victory`, `defeat` et `powerup*`.

### D22 : sources du joueur pour un joueur mort

Choix : **rien n'est affiché**, pas même le préfixe, pour `health`, `ammo`, `weapon` et
`currency`. C'est la même règle que pour les sources T2.12. Le texte est calculé par une
fonction pure (`player_source_text`), testée.

La barre de vie n'est pas touchée (hors périmètre) : elle garde sa dernière largeur.

### D19 : réglages du scénario dans l'enregistreur

- `RecordedSettings` regroupe `game`, `weapon_overrides`, `wave_overrides`, `powerups` et
  `powerup_drop_chance_override`.
- `InputRecorder::new(settings)` les reçoit et `to_scenario` les reporte.
- Le runner installe `InputRecorder::new(RecordedSettings::from_scenario(scenario))` avant la
  première frame et ne patche plus `recorded`.
- Une partie jouée utilise `RecordedSettings::default()`, soit le comportement d'avant.

### D13 : résumé à l'abandon, lobby local qui attend

- **Résumé à l'abandon** : quitter vers le lobby une partie en cours la termine en `Abandon`
  **avec son résumé**. Le calcul `run_summary` est partagé avec `finalize_run_summary_system` :
  mêmes champs, et le chemin défaite/victoire reste identique bit pour bit.
- **Lobby qui attend** : un retour au lobby **local** pose `LocalLobbyHold { summary }`, hors
  rollback.
  - `setup_ggrs_local` ne démarre pas tant qu'elle existe (condition dans `core.rs`).
  - L'écran du lobby (`ui/lobby.rs`) affiche le résumé et la retire sur Entrée ou sur
    « Nouvelle partie ».
  - Premier lancement inchangé : démarrage immédiat.
- La décision prise sur une requête est une fonction pure, `plan_run_request`.

### D3 : table des feuilles de sprites (kind `SpriteSheet`)

**Ce qui change :**
- Plus aucun chemin de sprite dans le code. `GlobalAsset` charge la table
  `sprites/sprites.ron`, déclarée dans `game.ron` des deux jeux.
- Un id par entrée : `asset_name_ref` d'un personnage, `sprite_config.name` d'une arme, ou
  `slash` (effet de mêlée, désormais optionnel).

**Lint, quatre règles, chacune avec sa fixture et son test :**
- `sprite_sheet_missing_file` : fichier d'animation ou feuille d'un calque absent ;
- `sprite_sheet_missing_image` : image (`path`) d'une feuille absente ;
- `sprite_sheet_duplicate_key` : id répété dans la table ;
- `weapon_sprite_unknown` : `sprite_config.name` d'une arme absent de la table.

**Rangement par entité** (`git mv`, avec historique) :
- Destinations : `sprites/characters/{player,shadow}/`, `sprites/weapons/`,
  `sprites/enemies/zombie/`, `sprites/effects/slash/`.
- Contenu inchangé, sauf la ligne `path:` de chaque feuille. Les configs d'animation
  (`*_animation.ron`) sont des renommages purs.
- `assets.yaml` (licences, voie V6) : chemins mis à jour, et tous les fichiers listés
  existent.

**Décisions :**
- **Les configs de contenu ne bougent pas.** `player_config.ron`, `weapons.ron` et
  `zombie_*_config.ron` restent sous `ZombieShooter/Sprites/**`, pour limiter les conflits avec
  les branches parallèles qui peuvent les éditer. Les feuilles n'y sont plus mélangées.
  `ZombieShooter/Sprites/Obj/` garde quatre images que rien ne référence.
- **Testbed :** ses feuilles d'armes citaient `ZombieShooter/Sprites/Obj/Weapons.png`, absent
  du testbed sur `origin/main`, donc l'arme était invisible. J'ai copié `Weapons.png` depuis
  `zombies` (même pack). L'arme est désormais visible : c'est le seul changement visible de D3.
- **Le `rifle` garde `sprite_config.name: "shotgun"`.** `Weapon` est un composant rollback
  haché qui contient `sprite_config`, donc renommer changerait des traces. Il n'existe pas non
  plus d'image propre au rifle. Reste une dette, ci-dessous.

## 2. Vérifié (dans ce clone)

| Commande | Résultat |
|---|---|
| `make test_scenarios` au départ (référence) | **62 verts**, `3 passed; 0 failed; 7 ignored` (14 min 21 s), aucun `.trace` modifié |
| `make test_scenarios` sur la branche (toutes les dettes) | **62 verts**, `3 passed; 0 failed; 7 ignored` (11 min 24 s) ; `git status` : aucun `.trace` |
| tests des dix crates (§4), après `cargo fmt` | **304 réussis, 0 échec, 8 ignorés** (16 min 14 s), dont les 62 scénarios ; aucun `.trace` modifié |
| `cargo test -p content` | 64 unitaires et 40 fixtures (36 + 4 nouvelles) ; lint de `zombies` et `testbed` sans erreur |
| `cargo test -p game --lib` | 29 tests, dont les 7 nouveaux (D13 ×4, D19, D22, lobby) |
| `make lint` | zombies (4 personnages, 4 armes, 6 de mêlée, 1 vague, 3 cartes) et testbed : « aucune erreur » |
| `cargo fmt --all -- --check` | rien à afficher, après un `cargo fmt` |
| `./scripts/check-forbidden.sh` | 4 avertissements, ceux qui existaient déjà |
| `./scripts/check-rollback-registration.sh` | OK |
| `make gen GAME=zombies` | 10 armes `ok ok`, aucun fichier modifié |
| D23, `DRY_RUN=1` | virgules, espaces et mélange listent les bons noms ; nom inconnu : code 1 ; sans argument : 52 scénarios |
| D23, rendu réel (Xvfb + lavapipe) | `testbed_corridor_idle` et `testbed_window` : deux vidéos et le montage, avec `SCENARIO=a,b` comme avec `SCENARIO="a b"` |
| D24 | sans `CARGO_TARGET_DIR` (cargo direct) et avec `./target` (make) : `target/metrics/latest.json`, plus de `crates/scenario/target/` ; script lancé depuis la racine et depuis `crates/scenario/` : il lit les métriques |
| D21, `ALACOD_EVENTS=1` | voir le détail sous ce tableau |
| D3 et D22, captures `play_scenario --capture` | voir le détail sous ce tableau |

Détail D21, sur trois scénarios :
- `clone_solo` : `victory` f3017, `powerup_pickup` max_ammo f401 et nuke f651, aucun `drop`
  à f0 ;
- `downed_all_lose` : `defeat` f1505, inchangé ;
- `powerup_drop_on_kill` : `powerup … au sol` f383.

Détail des captures (D3 et D22) :
- `clone_solo` (f650, f1300) et `bench_horde` (f400, f800) : joueur avec ses calques, ombre,
  arme en main et zombies visibles ;
- `testbed_dummy_shoot` (f60) : l'arme du joueur est visible ;
- `idle` à f1120 (joueur mort) : plus de « $ » ni de « ? | ? » ;
- aucune erreur de chargement d'asset dans les journaux.

Référence de départ : ma correction de D24 (fichier de test uniquement) a été compilée
**pendant** la première compilation de la référence, car je l'ai écrite juste après l'avoir
lancée. La référence était donc « `main` + test D24 ». Cela explique « 3 passed » au lieu de 2
et n'a aucun effet sur la simulation.

Le merge de `main` (`a39956a` : `docs/taches.md` et une fiche) arrive après ces vérifications.
Il ne touche aucun code : rien n'a été relancé.

## 3. Non fait, non vérifié

- **p2p à deux clients** et **bench strict** : non faisables dans le cloud. Aucune ligne de
  simulation n'a changé : les 62 traces sont identiques. À rejouer par l'orchestrateur.
- **D13 jamais joué avec une fenêtre** : ni clic sur « Lobby » en cours de partie, ni écran
  d'attente, ni Entrée pour relancer.
  - Couvert : les tests unitaires, et le test d'intégration `run.rs` (`ToLobby` →
    `LobbyLocal`), qui passe par le nouveau chemin (abandon, résumé, `LocalLobbyHold`) sans
    vérifier le résumé.
  - Aucun bouton ni aucune touche ne quitte une partie **en cours** : seul l'écran de fin
    (partie déjà terminée) pose `ToLobby`. L'abandon avec résumé ne passe donc aujourd'hui que
    par le test `run.rs` ou une `RunRequest` insérée directement.
  - À voir par l'orchestrateur : `make zombies`, jouer jusqu'à la défaite, cliquer « Lobby ».
    Il doit voir « Défaite : vague N - K kills - P points » et « Entrée : nouvelle partie »,
    le jeu ne doit pas relancer tout seul, et Entrée doit relancer.
- **D22 sur la vidéo de comparaison** : vérifié sur une capture d'`idle` à f1120, pas sur la
  vidéo de T3.2. À f1120, il faut voir sous « Vague 1 » aucun « $ », et en bas à droite aucun
  « ? | ? ».
- **D21 dans la page de revue** : les couleurs des pastilles sont écrites, mais la page n'a pas
  été ouverte dans un navigateur.
- **D3, contrôle partiel** : vérifié sur des captures, pas sur les vidéos de tous les scénarios.
  Le testbed n'a pas été capturé en vidéo.
- **Nombre de tests sur `main`** : non mesuré. 304 = celui de `main` + 14 nouveaux (4 fixtures,
  7 tests `game`, 2 tests `events`, 1 test D24), soit 290 attendus sur `main`.

## 4. Dettes et questions ouvertes

- **Rifle sans sprite propre** : il faut une image (voie V6) puis `sprite_config.name:
  "rifle"`. Les traces des scénarios qui utilisent le rifle changent alors (`weapon_rifle`,
  `shop_tour`… : preuve et bless par l'orchestrateur), car `Weapon` hache `sprite_config`.
  Autre option : sortir `sprite_config` du hash de `Weapon`, ce qui changerait toutes les
  traces une fois.
- **Configs de contenu encore sous `ZombieShooter/Sprites/**`** (`player_config.ron`,
  `weapons.ron`, `zombie_*_config.ron`) et images inutilisées sous `Obj/` : à ranger
  (`characters/`, `weapons/`, `enemies/`) quand aucune branche parallèle ne les édite.
- **Licence de la copie testbed** : `games/testbed/assets/sprites/weapons/Weapons.png` vient du
  pack blackHUNTERdev, enregistré dans `games/zombies/assets/assets.yaml`. Le testbed n'a pas de
  registre de licences.
- **Barre de vie** d'un joueur mort : elle garde sa dernière largeur (D22 ne touche que les
  textes).
- **Quitter une partie en cours** : aucune commande en jeu ne pose `RunRequest::ToLobby`
  pendant la partie (un menu pause manque), donc `Abandon` n'est pas atteignable à la main.
- **Abandon en p2p** : `Run` est modifié hors `GgrsSchedule` (déjà le cas avant D13 pour
  `step`, désormais aussi `summary`), juste avant la destruction de la session. Sans effet sur
  les traces, mais à garder en tête si l'abandon devient un jour un événement synchronisé.
- **Disque du cloud** : il a débordé une fois pendant les tests des dix crates (rendu + tests),
  ce qui a fait planter le compilateur. Après nettoyage d'artefacts recompilables, tout a été
  relancé et vert. La compilation avec rendu et les tests de toutes les crates ne tiennent pas
  ensemble dans environ 38 Go.
