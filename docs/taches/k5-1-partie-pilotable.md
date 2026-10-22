# k5-1-partie-pilotable — mode éditeur, commandes de partie, `Registry` hors lobby (K5, V3 avec V1)

Lire d'abord `docs/taches/README.md` (§1 variante cloud lignes 69-92, §4 vérification, §7 rapport), `CLAUDE.md` en entier (règles GGRS,
« Jouer et enregistrer »), `docs/editeur-vocal.md` (le guide d'usage, à tenir à jour) et, pour le contexte seulement,
`docs/taches/plan-editeur-vocal.md` (décisions D1 à D7). Tout ce qu'il faut est dans cette fiche. Branche `k5-1-partie-pilotable`
depuis `k5-0-1-editeur` (K5-0 livré : `make editor`, `alacod-remote discover`, `docs/editeur-vocal.md`). Estimation : 1 j. Section de
conventions réservée : la prochaine libre de `docs/conventions.md` (titre « Éditeur par l'IA »). **Aucun système n'est ajouté à
`GgrsSchedule`** : si une trace bouge, arrêter et le dire.

## Point de départ : la branche `k5-1-wip`

Un premier agent a écrit le code Rust puis a été interrompu. Il est sur `k5-1-wip` (commit `5d67b4a7`, « wip k5-1 »). Partir de là :
`git checkout -b k5-1-partie-pilotable k5-1-wip`. Contenu (tout compile : `cargo check -p game --profile headless --no-default-features
$(./scripts/dev-features.sh features) --tests` passait ; **rien n'a été exécuté**, aucun test de scénario écrit, aucune doc) :
- `crates/game/src/editor_mode.rs` (nouveau, `pub mod` dans `lib.rs`) : `EditorMode` (ressource), `EditorMode::from_env()` /
  `enabled_by_env()`, `EditorRefusal {Online, CheckDistance(usize)}` (Display nomme la cause), `hot_edit_allowed(online, check_distance)`,
  3 tests unitaires.
- `args/mod.rs` : `play_check_distance_from_env` renvoie 0 en mode éditeur même avec `ALACOD_STATE_TRACE` (+ `info!`) ; `ALACOD_CHECK_DISTANCE`
  explicite reste prioritaire.
- `character/player/input.rs` : dans `read_local_inputs`, branche `Devices`, `RemoteInputs::held` prend la main sur le clavier par handle.
- `remote.rs` : `from_env` accepte `ALACOD_EDITOR=1` ; `RemoteControlPlugin::build` (pas de pause, pas d'`InputSource::Remote` en mode
  éditeur) ; `alacod/input` vide = relâcher ; `alacod/restart`, `set_modifier`, `clear_modifiers`, `reload_content` ; `alacod/state` enrichi
  (`editor`, `online`, `check_distance`, `game`, `map`, `seed`, `run.step`, `config` par ennemi) ; `pub fn apply_set_modifier`, `pub fn parse_stat`.
- `content_hot_reload.rs` réécrit : plus de `run_if(in_lobby)`, remplace `Registry` **et** `GameManifest`, `ReloadReport`,
  `pub fn reload_registry_now(world)`, `pub fn content_fingerprint(root)`, scrutation 1 s par `Instant` (natif), 2 tests unitaires (non lancés).
- `core.rs` : `EditorMode` insérée avant le remote ; `scripts/alacod-remote` : `restart`, `modifier`, `unmodify`, `reload`, `brief` (1re ligne).
À faire d'abord : relire ce diff (`git diff k5-0-1-editeur k5-1-wip`) en entier, le comprendre, corriger ce qui ne tient pas.

## À faire

1. **Lancer les tests unitaires** : `cargo test -p game --profile headless --no-default-features $(./scripts/dev-features.sh features) --lib
   editor_mode content_hot_reload`. Corriger (le RON de `temp_game` dans `content_hot_reload.rs` doit passer `GameManifest::parse`).
2. **Garde** (`remote.rs`, `guard_hot_edit`) : lit `OnlineState` (`session.rs:44-49`) et `GggrsSessionConfiguration.connection.check_distance`
   (`jjrs/mod.rs:28`, rempli par `args/mod.rs:272` depuis `GameArgs.check_distance`). Absente → erreur interne, jamais « autorisé ».
3. **Clavier + prise de main** : vérifier à la main (headless, `make editor GAME=zombies HEADLESS=1`) : `alacod-remote input Right` bouge le
   joueur 0, `alacod-remote input ""` (relâcher) le laisse immobile, `alacod-remote state` ne se fige pas, la partie **tourne sans `resume`**.
4. **`restart`** : `alacod-remote restart --seed 7` → le journal montre le retour en `GameLoading` puis `InGame`, `alacod-remote state`
   donne `seed: 7`. `seed` s'écrit dans `MapGenerationConfig.seed` (`map::generation::config`, i32) : c'est la seule lecture de la graine
   au `GameLoading` suivant (`map_ldtk/src/loader/runtime.rs` `setup_generated_map`) et de `RunSeed` (`jjrs/local.rs:121`). `ToLobby` + `seed`
   est refusé (le lobby reconstruit la carte : `map_ldtk/src/game/local.rs` `configure_map`).
5. **Modificateurs** : `alacod-remote modifier --handle 0 MoveSpeed Mul 0.9`, puis `state`/`step 60` : le joueur va 10 % moins loin ;
   `unmodify` rétablit. `Modifiers` : `crates/sim_core/src/modifier.rs` (`push_from`, `remove_by_source`, `ModifierSource::Named`).
6. **`Registry` hors lobby + scrutation** : mesurer le coût de `content_fingerprint` sur `games/isaac` (349 RON, 20 déclarations) : écrire un
   test `#[ignore]` ou un `eprintln!` de durée sur 100 appels, dire la mesure au rapport. Écrire un RON **invalide** pendant que le jeu tourne
   (`echo "(((" > games/zombies/assets/waves/wave_config.ron`) : attendu, `ERROR contenu invalide, registre non rechargé` ou
   `Failed to load asset`, ancien registre gardé ; puis `git checkout --` du fichier.
7. **Test (b) `crates/scenario/tests/editor_reload.rs`** (nouveau, sur le modèle de `character_configs.rs:136` et `run.rs:68-170`) :
   - `if map_ldtk::RENDER_ENABLED { return; }` en tête (comme les autres) ; copier `games/testbed` (4,4 Mo) dans un dossier temporaire
     (`std::env::temp_dir()`), scénario `hit_iframes` (jeu testbed) ; `build_app(&scenario, true, &PlayConfig { follow_handle: None })`, puis
     `app.world_mut().insert_resource(GameRoot(tmp))`, `app.finish(); app.cleanup();`, `app.update()` jusqu'à `InGame` et frame ≥ 10.
   - Table choisie : `combat::status::StatusLibrary` (`statuses["brulure"].damage`, 4 dans `games/testbed/assets/statuses/brulure.ron`),
     dérivée du `Registry` à `OnEnter(GameLoading)` (`core_sim.rs:249-262`, `statuses.rs`).
   - Étape A (scrutation) : réécrire `tmp/assets/statuses/brulure.ron` avec `damage: "9"`, `std::thread::sleep(1200 ms)`, 3 `app.update()` :
     `Registry.statuses["brulure"].damage == 9`, état toujours `InGame`, `StatusLibrary` **encore 4** (table dérivée non recalculée en partie).
   - Étape B (événement d'asset) : réécrire avec `"11"` puis tout de suite `Assets<WeaponsConfig>::get_mut(&GlobalAsset.weapons)` (émet
     `Modified`), 2 `update()` : `Registry` à 11 sans attendre 1 s.
   - Étape C : `app.world_mut().insert_resource(RunRequest::Restart)`, attendre un nouveau `InGame` frais (copier `wait_for_fresh_in_game`
     de `run.rs:122`), `StatusLibrary.statuses["brulure"].damage == 11`.
   - Étape D : fichier invalide dans le dossier temporaire → `Registry` inchangé (lint rouge), pas de panique.
8. **Test (c) `set_modifier`** (même fichier ou `editor_modifier.rs`) : le runner joue en synctest d=2 par défaut
   (`crates/scenario/src/runner.rs:1942`, `check_distance_from_env()`), où une écriture hors `GgrsSchedule` est perdue au rechargement du snapshot.
   Fixer `check_distance = 0` : `app.world_mut().resource_mut::<GggrsSessionConfiguration>().connection.check_distance = 0` avant la première
   frame (**[INFERENCE]** : vérifier que la ressource existe déjà après `build_app`, sinon poser `ALACOD_CHECK_DISTANCE=0` dans un test
   isolé `#[serial]`-like, seul de son fichier). Deux parties du même scénario où le joueur 0 tient `Right` de la frame 10 à 100 (modèle :
   un scénario existant à `buttons: [Right]`, p. ex. `grep -l Right tests/scenarios/*.ron`) : A sans modificateur, B avec
   `game::remote::apply_set_modifier(world, &SetModifierParams { stat: "MoveSpeed", op: Mul, value: "0.5", handle: Some(0), … })` à la frame 20
   (le type est privé : le rendre `pub` avec un constructeur ou des champs `pub`). Attendu : déplacement de B < déplacement de A. Ajouter les
   cas d'erreur : stat inconnue → `Err`, `value` non Fixed → `Err`, handle absent → `Err`.
9. **Test (a)** déjà écrit (`editor_mode.rs`) ; ajouter un test unitaire de `parse_stat` (`MoveSpeed`, `custom:xx`, `Nope`).
10. **Tests de non-régression** : `make test_crates` ; sous-ensemble de scénarios puis suite complète (ci-dessous).
11. **Docs** : `docs/editeur-vocal.md` §1 (partie en marche, clavier), §2 (nouvelles sous-commandes, `alacod/*` : liste, paramètres, erreurs
    de garde), §5 (tableau : `Registry` → « au `restart` », scrutation 1 s, limite `GlobalAsset` : un personnage **ajouté** exige une relance,
    cf. Pièges), §7 ; `CLAUDE.md` : variable `ALACOD_EDITOR` dans « Headless, trace d'état » et méthodes dans « Jouer et enregistrer » ;
    `docs/conventions.md` : nouvelle section « Éditeur par l'IA » (mode éditeur, garde d=0, méthodes, scrutation) ; `scripts/alacod-remote`
    docstring.

## Règles

- Aucun système dans `GgrsSchedule`, aucune ressource rollback nouvelle : `make schedule_order_diff` n'a rien à signaler (le lancer une fois).
- Les écritures hors simulation (`Modifiers`, `Registry`) ne sont licites qu'à `check_distance = 0` en session locale : toutes les commandes
  passent par `hot_edit_allowed`. Ne pas l'assouplir.
- Ne pas toucher à `.claude/settings.json`, `scripts/claude-say.sh`, à la simulation, aux scénarios et à leurs traces. Pas de `BLESS`.
- `f32` interdit dans `GgrsSchedule` ; ici les handlers sont hors simulation mais `value` reste un `Fixed` parsé (`"0.9".parse::<Fixed>()`).
- Logs : jamais `Entity`, utiliser `GgrsNetId`/handle. Pas de `HashMap` dans ce qui touche la simulation. Français partout.
- Jamais de push, de `git stash`, de commande destructive.

## Vérification

Compiler : profil `headless`, `--no-default-features`, `$(./scripts/dev-features.sh features)`, **une seule commande cargo à la fois**
(README §1 cloud). `nproc` = 8 sur la machine de travail précédente ; premier build ≈ 10 min.
1. `make test_crates` (tout le workspace) ; `cargo test -p scenario --profile headless --no-default-features $(./scripts/dev-features.sh
   features) --test editor_reload`.
2. Sous-ensemble sans BLESS : `ALACOD_SCENARIO=idle,four_players_shooting,clone_solo,remote_first_fight,throne_solo,gungeon_full_run,isaac_full_run
   make test_scenarios` (+ tous les scénarios dont le nom commence par `remote`). Puis la suite par tranches en arrière-plan :
   `ALACOD_SCENARIO_SLICE=1/3 make test_scenarios`, `2/3`, `3/3` (1 à 3 h au total). **0 trace ne doit bouger.**
3. Preuve headless : `make editor GAME=zombies HEADLESS=1 > /tmp/editor.log 2>&1 &` ; `scripts/alacod-remote discover` liste
   `alacod/restart`, `set_modifier`, `clear_modifiers`, `reload_content` ; `alacod-remote brief` affiche en 1re ligne
   `editor=True online=False check_distance=0 run.step=…` ; essais des points 3 à 6 ; modifier une valeur de `games/zombies/assets/items/powerups.ron`
   puis `alacod-remote reload` → `errors: []` ; refus : relancer avec `ALACOD_CHECK_DISTANCE=2` → `alacod-remote modifier …` échoue avec
   « check_distance=2 » dans le message. Restaurer les fichiers (`git checkout --`).
4. `cargo fmt --check` ; `scripts/check-forbidden.sh` ; `alacod lint games/zombies` (3 jeux).

## Livraison

Commits (un par étape logique) : `k5-1 : mode éditeur jouable, restart/set_modifier/reload_content, Registry hors lobby` puis `k5-1 : tests,
docs`. Rapport `docs/taches/rapports/k5-1-partie-pilotable.md` : ce qui est fait, sorties de commandes (journal du rechargement, refus),
**coût mesuré de la scrutation sur isaac**, ce qui n'a pas pu être vérifié (rendu, clavier réel, Mac), décisions (seed limité à
`MapGenerationConfig`). Ligne au journal `docs/taches.md` §10 sur le modèle des dernières lignes, et `LIVRÉ k5-1-partie-pilotable <sha>` en fin
de réponse. Jamais de push.

## Pièges

- **`InputSource::Remote` coupe le clavier** : `RemoteControlPlugin::build` l'insère aujourd'hui (`remote.rs`, ~l.93 avant K5-1) et
  `read_local_inputs` (`input.rs:217-238`) ne lit alors que `RemoteInputs`. En mode éditeur on ne l'insère pas ; `character/mod.rs:169`
  pose `InputSource::from_env()` (défaut `Devices`), le plugin remote est ajouté **après** (`core.rs`), donc il écraserait.
- **`RemoteClock` démarre en pause** (`paused: true`) : `drive_clock` fige `TimeUpdateStrategy` ; en mode éditeur `paused: false`.
  `Time<Real>` est figé en pause : la scrutation utilise `Instant`, pas `Time`.
- **`check_distance` vit dans `GggrsSessionConfiguration.connection`** (`jjrs/mod.rs:28`), jamais dans `GameArgs` à l'exécution ; il vaut
  2 dans le runner de scénarios (`runner.rs:1942`) et 0 dans le jeu joué (`args/mod.rs:56`), 2 avec `ALACOD_STATE_TRACE` hors mode éditeur.
- **`Registry` est lu par** (tous hors `GgrsSchedule`, vérifié par grep) : `statuses.rs:44`, `patterns.rs:122`, `progression.rs:105`, `clock.rs:91`
  (tous `OnEnter(GameLoading)`), `map_ldtk/src/game/floors.rs:182` (`compute_floor_plan`), `map_ldtk/src/loader/runtime.rs:362`
  (`setup_generated_map` : surfaces, props, salles, objets, synergies, pools, déblocages, accomplissements, hub), `jjrs/local.rs:67` et
  `jjrs/p2p.rs:228` (démarrage de session), `global_asset.rs:193` (`add_global_asset`, **`Startup` seulement**), écrans (`ui/mutation_*`,
  `ui/screens`). `GameManifest` : mêmes lieux + `recording.rs:308`. Tous lisent la ressource courante : remplacer la ressource suffit.
- **`GlobalAsset` ne relit pas le registre** : un fichier de personnage/arme **ajouté** n'a pas de handle d'asset avant relance du binaire ;
  les fichiers existants se rechargent par leurs assets. À écrire dans la doc, ne pas « corriger ».
- **Quatre assets sentinelles** (`CharacterConfigAsset`, `WeaponsConfig`, `MeleeWeaponsConfig`, `WaveConfigAsset`) étaient les seuls
  déclencheurs du rechargement du `Registry` ; éditer `items/`, `rooms/`, `statuses/`, `patterns/`, `curses/`, `economy.ron`, `perks.ron` ou
  `game.ron` ne déclenchait rien : d'où la scrutation. `AssetEvent::Modified` n'est **pas** émis pour un RON invalide (mesuré : `ERROR Failed
  to load asset … Could not parse RON`, table inchangée).
- **`RunRequest`** (`run_state.rs:202-216`) : ressource posée par le handler, consommée par `apply_run_request_system` (`run_state.rs:473-560`) ;
  `Restart` local reprend `MapGenerationConfig` tel quel (`plan_run_request`, `run_state.rs:~255-285`) et saute le lobby.
- **`apply_launch_seed`** (`map_ldtk/src/game/local.rs:100`) ne sert qu'au `--seed` de la ligne de commande (`Startup`) ; zombies tire sa
  graine dans `choose_local_seed` (`games/zombies/src/main.rs:106`). Pour `restart --seed`, écrire `MapGenerationConfig.seed`.
- **`GgrsNetId.0` est un `usize`** (comparer avec `as u64`). `Fixed` se parse avec `FromStr` (feature `serde-str` de `fixed`).
- **Modificateurs** : `Modifiers` est rollback (`stats/src/plugin.rs`) : écrire hors `GgrsSchedule` n'est sûr qu'à d=0. L'expiration
  `until` est une frame absolue (`FrameCount.frame + until_frames`). `ModifierOp` = `Add | Pct | Mul | Set`.
- Le runner de scénarios n'a pas `GameRoot` : le rechargement y est un no-op sauf si un test la pose.
- `RENDER_ENABLED` : les tests de scénario se désactivent s'ils sont compilés avec le rendu (`--no-default-features` obligatoire).
