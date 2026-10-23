# k5-3-apps-de-test — remote et mode éditeur sans variable d'environnement dans les apps de test (K5, V3)

Lire d'abord `docs/taches/README.md` (§1, §4, §7), `CLAUDE.md` (« Headless », « Jouer et enregistrer », « Simulation et présentation »),
`docs/editeur-vocal.md` et la section « Éditeur par l'IA » de `docs/conventions.md` (ajoutées par K5-1). Branche `k5-3-apps-de-test` depuis
`main` **après le merge de `k5-1-partie-pilotable`** (K5-2 n'est pas requis ; sans lui `character_tester` n'a pas de `spawn`). Estimation : 1 j.
Pas de nouvelle règle de simulation : cette tâche ne touche ni `GgrsSchedule` ni les scénarios.

## But

Aujourd'hui (état de `main` après K5-1) le mode éditeur et le remote ne s'allument que par `ALACOD_EDITOR=1` / `ALACOD_REMOTE=1` :
`make editor` les pose, mais `make character_tester` (`Makefile:331-332`) non, et l'exemple `character_tester` ne le sait pas. On veut :
`make character_tester` et `make editor` répondent sur le port remote (15702) **sans que William pose une variable**, et `alacod-remote brief`
affiche `editor=True` pour les deux.

## À faire

1. **`CoreSetupConfig`** (`crates/game/src/session.rs:13-30`) : ajouter `#[serde(default)] pub editor: bool` (« mode éditeur voulu par l'exécutable,
   en plus de `ALACOD_EDITOR=1` »). `from_env` le laisse à `false`. Mettre à jour les **trois** constructions littérales du struct (sinon
   erreur de compilation) : `crates/game/src/core_sim.rs:315`, `crates/scenario/src/runner.rs:288`, `crates/scenario/tests/schedule_order.rs:240`
   (ajouter `editor: false` ou `..Default::default()`). `grep -rn "CoreSetupConfig {" crates games examples templates` pour n'en oublier aucune
   (les `games/*/src/main.rs` utilisent `from_env`).
2. **`crates/game/src/core.rs`** (bloc `#[cfg(not(target_arch = "wasm32"))]` après `ContentHotReloadPlugin`, ~l.196-205 après K5-1) :
   `let editor = self.0.editor || EditorMode::enabled_by_env();` → `if editor { app.insert_resource(EditorMode) }` ; le remote doit démarrer si
   `ALACOD_REMOTE=1` **ou** `editor` : changer `RemoteControlPlugin::from_env()` en `RemoteControlPlugin::new(editor)` qui renvoie `Some` si
   `editor || ALACOD_REMOTE=1` (port : `ALACOD_REMOTE_PORT`, défaut `bevy::remote::http::DEFAULT_PORT`). Garder `from_env()` si d'autres
   appelants l'utilisent (`grep -rn "RemoteControlPlugin" crates games examples`).
3. **`check_distance` à 0 sans variable** : `play_check_distance_from_env` (`crates/game/src/args/mod.rs:56`) ne voit pas `CoreSetupConfig`.
   `BaseArgsPlugin` est ajouté **avant** `CoreSetupPlugin` dans `examples/character_tester.rs` (« MUST be before core_plugin », commentaire du fichier)
   et lit `get_args()` seul. Solution la plus simple : `BaseArgsPlugin` expose un constructeur `BaseArgsPlugin::editor()` (ou un champ) qui force
   `check_distance = 0` quand l'exécutable déclare l'édition ; **ou** l'exemple fait `std::env::set_var("ALACOD_EDITOR", "1")` en première
   ligne de `main()` (avant `get_args()`), simple et local à l'exemple. Choisir l'une, la documenter (rapport) ; dans les deux cas
   `alacod-remote brief` doit afficher `check_distance=0`.
4. **`examples/character_tester.rs`** : `game_config.editor = true;` après `CoreSetupConfig::from_env("zrl-character_tester")` (l.~37). Il insère
   déjà `GameRoot` (l.~53) et le `Registry` mais **pas** `GameManifest` : `alacod/state` doit donc retomber sur le nom du dossier de `GameRoot`
   (`game: "zombies"`, logique déjà écrite en K5-1) ; vérifier. Le rechargement du `Registry` (K5-1) y marche : `alacod-remote reload`.
5. **`Makefile`** : `character_tester` (l.331-332) n'a pas `$(DEV_FEATURES)` alors que `zombies`, `testbed`… l'ont : l'ajouter (une seule
   configuration de features par arborescence : sinon Bevy est compilé deux fois, `scripts/dev-features.sh`). `make editor` (K5-0) pose déjà
   `ALACOD_EDITOR=1 ALACOD_REMOTE=1 ALACOD_LABELS=1` : les **garder** (ils restent valables pour les binaires de jeu qui n'ont pas la config
   `editor`), mais ajouter dans les `games/<jeu>/src/main.rs` (`zombies`, `gungeon`, `isaac`, `throne`, `testbed`) `game_config.editor = true`
   **seulement si** William veut que `cargo run -p zombies` seul démarre en mode éditeur — **ne pas le faire** : un jeu lancé à la main doit
   rester un jeu (décision à confirmer avec William, la noter au rapport).
6. **`map_preview` reste tel quel** (`examples/map_preview.rs`) : pas de `CoreSetupPlugin`, pas de session GGRS, ni `FrameCount`, ni `OnlineState`
   — `alacod/state` y paniquerait (`world.resource::<FrameCount>()`) ; c'est un aperçu statique rechargé à la touche R (`keyinput`, l.~71-90, graine = horloge).
   « Map Tester » = `make editor` + `alacod-remote restart --seed N` (K5-1). Le dire dans `docs/editeur-vocal.md`.
7. **`docs/jouer.md`** (il parle de gungeon : ajouter une section « Éditer en jouant (mode éditeur) » après « Les modes ») : `make editor GAME=zombies`,
   `make character_tester`, ce qu'on y voit (étiquettes F6, partie en marche, clavier), où est `docs/editeur-vocal.md`, la limite local/d=0.
   `docs/editeur-vocal.md` §1 : tableau « app / commande / ce qu'elle apporte » (`make editor`, `make character_tester`, `map_preview`, `map_generation`
   = CLI sans `App`, Music Tester hors périmètre jusqu'à Harmonium).
8. **Web** : `character_tester` est aussi compilé en wasm (`Makefile:376`, `build_character_tester_web`) ; tout le code ajouté reste sous
   `#[cfg(not(target_arch = "wasm32"))]` ou n'utilise que `std::env` (retourne `Err` en wasm). Vérifier avec
   `cargo check -p game --target wasm32-unknown-unknown --no-default-features` **si la cible est installée** ; sinon le dire au rapport.

## Règles

- Présentation/outillage seulement : aucun système de simulation, aucun enregistrement rollback, aucune trace ne bouge.
- `core.rs` est dans la moitié « application » (`CoreAppPlugin`) : rien de rollback ici (CLAUDE.md « Simulation et présentation »).
- Ne pas changer `ALACOD_EDITOR` : il reste honoré partout, la config n'est qu'une autre façon de l'allumer.
- Compilation : profil `headless` ou `dev` avec rendu selon la preuve ; une seule commande cargo à la fois.

## Vérification

1. `make test_crates` (les trois constructions de `CoreSetupConfig` compilent) ; `cargo check --examples --features native --no-default-features
   $(./scripts/dev-features.sh features)` (l'exemple `character_tester` compile).
2. Sous-ensemble sans `BLESS` : `ALACOD_SCENARIO=idle,four_players_shooting,clone_solo,remote_first_fight make test_scenarios` : 0 trace différente
   (aucune simulation touchée) ; `cargo test -p scenario --test schedule_order` (utilise `CoreSetupConfig`).
3. Preuve, **sur une machine avec affichage** (la machine de travail du cloud n'en a pas) : `make character_tester` puis, sans variable,
   `scripts/alacod-remote discover` et `scripts/alacod-remote brief` (1re ligne `editor=True online=False check_distance=0`) ; `make editor GAME=zombies`
   idem. Headless possible pour `make editor` seulement (`HEADLESS=1`) ; pour `character_tester`, `ALACOD_HEADLESS=1 cargo run --example character_tester
   --profile headless --no-default-features --features native $(./scripts/dev-features.sh features) -- --local-port 7000 --players localhost`
   (`CoreSetupConfig::from_env` lit `ALACOD_HEADLESS`). Coller les sorties au rapport ; ce qui n'a pas pu être essayé (fenêtre, clavier) est dit tel quel.

## Livraison

Commit `k5-3 : remote et mode éditeur par défaut dans character_tester` ; rapport `docs/taches/rapports/k5-3-apps-de-test.md` ; journal
`docs/taches.md` §10 ; `LIVRÉ k5-3-apps-de-test <sha>`. Jamais de push.

## Pièges

- **Ordre des plugins** : `EditorMode` doit être insérée **avant** `RemoteControlPlugin` (celui-ci lit `world.contains_resource::<EditorMode>()` dans
  `build` pour décider la pause et l'`InputSource`). Dans `core.rs` : insérer, puis ajouter le plugin remote.
- **`InputSource`** : `character/mod.rs:169` insère `InputSource::from_env()` (défaut `Devices`) ; hors mode éditeur le plugin remote y substitue
  `Remote` (clavier coupé). Si `character_tester` passe à `editor = true`, le clavier reste actif : c'est voulu.
- `character_tester` **ne passe pas** par `LdtkLocalGamePlugin` : pas de `LdtkGameMap`, donc `alacod/restart --seed` n'a que `MapGenerationConfig` si
  elle existe ; l'exemple pose lui-même ses entités à `OnEnter(GameLoading)` (`system_game_loading`) : un `restart` rejoue ce système.
- `Makefile:331` : `$(ARGS)` vient **avant** `--features native` ; ne pas réordonner (les cibles `*_matchbox` suivent le même modèle).
- Le port est fixe (15702) : deux apps de test ne peuvent pas tourner ensemble sans `ALACOD_REMOTE_PORT`.
- `CoreSetupConfig` est `Serialize/Deserialize` (utilisée par le web) : le champ `editor` doit avoir `#[serde(default)]` pour ne pas casser les
  configurations existantes sérialisées.
- Music Tester (Harmonium) : hors périmètre tant qu'Harmonium n'existe pas ; même mode local, même `scripts/alacod-remote`.
