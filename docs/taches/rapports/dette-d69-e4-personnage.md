# dette-d69-e4-personnage — D69 étape 2, E4 : personnage, entrées, plugin, visuels

Worker b0, 2026-10-19, branche `dette-d69-e4-personnage` (depuis `origin/main` `e0061e67`, qui contient E6 et D116). Mac arm64 partagé (charge 10 à 30 : b1 et b3). Fiche : `docs/taches/dette-d69-etape2.md` § E4, recettes `annexe-personnage.md` §1, §4, §5. Refactoring pur : aucune trace de scénario ne bouge, aucun `BLESS`, aucun type rollback créé, retiré ni déplacé.

## 1. Résultat

| À faire (fiche) | État |
|---|---|
| 1. `read_local_inputs` scindé : la simulation lit `Option<Res<DeviceInputs>>` (`BTreeMap<usize, BoxInput>`), `collect_device_inputs` (app, `character/input_devices.rs`) en `ReadInputs`, `.before(read_local_inputs)`, absent en headless ; l'appel à `confirm_input` passe côté app | **fait** : §2 |
| 2. `create_player` sans `ActionState`/`InputMap`/`PointLight2d` ; `attach_local_player_input` (`PreUpdate.before(RunGgrsSystems)`, `Added<LocalPlayer>`), `attach_player_light` (`light/mod.rs`, `lighting`), `AiDriver` sous `harmonium` | **fait** |
| 3. `BaseCharacterGamePlugin` éclaté : ordre des `rollback_and_trace*`/`add_frame_events` et bloc `GgrsSchedule` identiques ; `CharacterAppPlugin` juste après dans `core.rs` ; `skip_device_input` supprimé | **fait** : §3 |
| 4. `Visibility` et `SpatialAudioEmitter` retirés de `character/create.rs` (`Visibility` posée par `attach_character_visuals`) ; `SpatialAudioReceiver` de la caméra intact | **fait** : §5 |
| 5. `spawn_test_wall` → `collider/test_wall.rs` ; appelants migrés | **fait** : `map_ldtk/game/collider.rs`, `examples/character_tester.rs` (une ligne chacun) |
| 6. `ai/debug.rs` → `character/ai_debug.rs`, `health/ui.rs` → `character/health_bar.rs`, `player/control.rs` → `character/control.rs` | **fait** (`git mv`, contenu inchangé hors chemins d'import) |
| 7. Tests `ActionState` de `player/input.rs` → app | **fait** : `input_devices.rs`, le test passe par `collect_device_inputs` puis `read_local_inputs` (comme `ReadInputs`) ; un second test pose le cas sans `DeviceInputs` (headless) |

## 2. Ce qui passe côté application

| Quoi | Où | Notes |
|---|---|---|
| lecture des périphériques (`ActionState`, fenêtre, caméra, `confirm_input`) | `character/input_devices.rs` : `collect_device_inputs` | même formule de `pan` (`Transform` du joueur, `viewport_to_world_2d`) ; remet `DeviceInputs` à vide à chaque appel, ne lit rien hors `InputSource::Devices` |
| pose de `ActionState` + `InputMap` (+ `AiDriver` sous `harmonium`) | `input_devices.rs` : `attach_local_player_input` | `try_insert`, `PreUpdate` avant `RunGgrsSystems` |
| lumière du joueur | `light/mod.rs` : `attach_player_light` (feature `lighting`, `Update`, `Added<Player>`) | posée aussi pour un joueur distant (c'était déjà le cas) |
| leafwing `InputManagerPlugin::<PlayerAction>`, `DeviceInputs`, chargeur `CharacterConfigAssetPlugin`, ressources de debug `FlowFieldDebug`/`EnemyStateDebug` | `character/app_plugin.rs` : `CharacterAppPlugin { headless }` | leafwing, `DeviceInputs` et les deux systèmes ci-dessus **seulement hors headless** (plus de leafwing en headless) |
| `CharacterPresentationPlugin` | `character/app_plugin.rs` (ajouté par `PresentationPlugin`, chemin changé) | contenu inchangé |
| `PlayerAction`/`get_input_map` | `character/control.rs` | lecteurs migrés : `camera/mod.rs`, `ui/mutation_screen.rs` (1 ligne), `input_devices.rs` |
| débug IA (F3/F4/F5), barres de vie | `character/ai_debug.rs`, `character/health_bar.rs` | `feedback.rs`, `room_fog.rs`, `visuals.rs` : chemin de `HealthBar` (1 ligne chacun) |
| mur de test (`Sprite`, lumière) | `collider/test_wall.rs` | code octet pour octet |

La fiche nomme `character/mod.rs` « côté app » pour `CharacterAppPlugin`/`CharacterPresentationPlugin` ; tant que `game` est un seul crate `character/mod.rs` est le fichier de simulation (il doit entrer au cliquet), la moitié application s'appelle donc **`character/app_plugin.rs`** (E9 : `game_app/src/character/mod.rs`), comme `p2p_socket.rs` en E6.

## 3. Ordre et position (preuve)

`make schedule_order_diff BASE=origin/main` : **identique** (4 520 lignes de schedules, 645 types, 5 jeux). L'app de cet outil est headless : `ReadInputs` n'y contient que `read_local_inputs`, comme avant. Hors headless `ReadInputs` gagne `collect_device_inputs` (voulu, avant `read_local_inputs` ; `bots` et `recording` s'ordonnent `.after(read_local_inputs)`, inchangés). `InputManagerPlugin` change de place dans `CoreSetupPlugin` (après `BaseCharacterGamePlugin`, hors headless) : il n'enregistre aucun type rollback.
Deux résolutions à noter : `CharacterConfigAssetPlugin` (E3a) passe du plugin de simulation au plugin application (il charge des assets : sa place), et `StatsPlugin`/`PointerWorldPosition`/`rollback_and_trace*` restent dans le plugin de simulation **à la même place relative** (`CharacterAppPlugin` n'enregistre rien en rollback).

## 4. Sonde `make probe_game_sim --tests` (avant / après)

| | avant (`e0061e67`) | après |
|---|---|---|
| fichiers à fuite / total | 47 / 130 | **45 / 133** |
| erreurs de résolution | 786 | **778** |
| par catégorie | entrée 96, audio 9, fenêtre 10 | entrée 90, audio 7, fenêtre 10 |

Sortent de la liste des fuites (5 fichiers de simulation) : `character/create.rs` (3 erreurs), `character/mod.rs` (3), `character/player/input.rs` (8), `character/player/create.rs` (1), `collider/mod.rs` (1). Entrent (application, 3 nouveaux plus 3 déplacés) : `character/input_devices.rs` (4), `character/app_plugin.rs` (2), `collider/test_wall.rs` (1) ; `character/control.rs` (47), `character/ai_debug.rs` (8), `character/health_bar.rs` (1) gardent leurs erreurs sous un autre nom ; `character/visuals.rs` 17 → 18 (le `Visibility` qu'il pose). Les fuites de la fiche tombent donc bien : `bevy::window`, `leafwing`, `bevy_kira_audio` et `Sprite`/lumière ne sont plus nommés par les cinq fichiers.

**Cliquet** (`scripts/d69-sim-files.txt`, 29 → **35 fichiers**) : + `character/dash.rs`, `character/surface.rs`, `character/enemy/ai/{navigation,retreat,state}.rs`, `collider/mod.rs` (point fixe calculé sur la sonde : le plus grand ensemble sans fuite et fermé). La fiche nomme aussi `character/player/input.rs`, `character/player/create.rs`, `character/mod.rs`, `character/create.rs`, `replay.rs`, `recording.rs`, `ai/*` : ils sont **sans fuite propre** mais ne peuvent pas y entrer, leur fermeture touche des fichiers qui ne sont pas encore séparables :
`character/health/mod.rs` (↔ `character/config.rs` en cycle ; dépend de `economy.rs`, `effects_v3.rs`, `progression.rs`, **`interaction.rs`** qui fuit : `Sprite`, gizmos, E5c), `waves/mod.rs` (↔ `character/enemy/spawning.rs`), `hub.rs` (`interaction.rs`, `shop.rs`, `run_state.rs`), `state_trace.rs` (↔ `run_state.rs`), `progression.rs`. `character/enemy/ai/{pathing,behavior,rules,squad}.rs` dépendent de `character/health/mod.rs` ou `character/config.rs` ; `recording.rs` de `progression.rs` et `state_trace.rs`. Ils entreront quand `interaction.rs` (E5c) et la chaîne `run_state.rs` (E3c/E5) seront libérés. `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` : sortie 0.

## 5. Visibilité, audio : ce qui bouge à l'écran

- `Visibility::default()` du personnage est posée avec les sprites (`attach_character_visuals`, `try_insert`) au lieu de la création. Un personnage dont les assets ne sont pas prêts n'a pas de `Visibility` plus longtemps (il n'a pas non plus de sprite).
- `SpatialAudioEmitter { instances: vec![] }` : composant inerte (`instances` n'est jamais rempli), retiré.
- **Captures : un seul écart, expliqué et reproduit** (§6). Le retrait ne change pas un pixel (expérience A : `Visibility` et émetteur remis dans `create.rs` → mêmes 93 images différentes) ; la cause est une **entité de plus au démarrage**.

## 6. Captures `cmp` (`play_scenario --features render --capture --every 4`, build `headless`)

| scénario | build | images | différentes |
|---|---|---|---|
| `two_players_shooting` | sans `lighting` | 75 | **0** |
| `two_players_shooting` | `lighting` (`--features game/lighting`, lumière des joueurs par `attach_player_light`, murs) | 75 | **0** |
| `gungeon_boss` | sans `lighting` | 175 | **97** (voir ci-dessous) |
| `gungeon_boss` | `lighting` | 175 | **93** |

`gungeon_boss` : les différences sont celles du **halo des objets** (`legibility.rs:320`, `phase = (entity.index_u32() % 16) * 0.4`, respiration de l'alpha et de la taille) : image amplifiée 16×, la différence est un anneau autour du butin (pièce, bottes), jamais ailleurs. Cause établie : `DeviceInputs` est une **ressource nouvelle**, et dans cette version de Bevy une ressource est une entité ; la sonde d'entités (`Query<Entity>` à chaque `First`, 400 frames d'app) donne **+1 dès la frame d'app 1** (757 → 758) et tous les halos qui suivent ont `index + 1` (1186 → 1187, 1192 → 1193, 1198 → 1199), donc une autre phase. Preuve de la cause : la **base + une ressource factice** (`init_resource::<E4Dummy>()` dans `CoreSetupPlugin`) donne **0 image différente de la tête sur 175, 4 captures sur 4**. La capture de `gungeon_boss` de la base est d'ailleurs bimodale d'une exécution à l'autre (base contre base : 0, 1, 0, 1, 0, **95**), la comparaison n'est donc stable que d'un même côté ; la tête seule : 93 à 97 images différentes de la base, 0 contre la base factice. Présentation seule, hors simulation, hors trace.
Chiffres de la boucle d'expériences : 16 captures de la tête contre la base : 93 à chaque fois ; expérience A (`Visibility` + émetteur remis) : 93, 93, 93 ; base factice contre tête : 0.

## 7. Vérifications

| Commande | Résultat |
|---|---|
| `make schedule_order_diff BASE=origin/main` | **OK** : 4 520 lignes, 645 types |
| `make test_scenarios_par SHARDS=3` (sans BLESS) | 802 scénarios, 3 tranches ok, 255 s ; `git status` propre |
| `make test_crates` | sortie 0, 84 `test result: ok` (dont `input_devices::tests`) |
| `make lint` ×5, `make gen GAME=…` ×5 | 0 erreur ; 16 / 104 / 39 / 59 / 213 scénarios générés, tous `ok ok` |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1` | OK / OK |
| `cargo fmt --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools`, `make check_sim_boundary` | verts ; 13 crates, 2 en dette |
| `cargo check -p game --features lighting,ui_hui,debug_ui` ; `cargo check --workspace --all-targets` ; build web `APP_VERSION=e4 cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` | OK ×3 |
| `ALACOD_INPUT=neutral`, devices (défaut), `script:tests/scenarios/p2p_smoke.ron` (zombies, `--seed 1`, 600 frames, `ALACOD_STATE_TRACE`) | traces **identiques** à `origin/main` (599 lignes ×3) |
| remote (`ALACOD_REMOTE=1`, `alacod-remote input Right,Fire --pan 100,0` puis `step 120` puis `state`) | JSON d'état **identique** à `origin/main` (joueur en (453.3, -504.0), vague 1) |
| `p2p-stress.sh` zombies / isaac (3 000 frames) | codes 0 0, 0 `DesyncDetected`, **1 999** / 0 et **1 982** / 0 frames resimulées, traces identiques (2 999 lignes) |

## 8. Essai manuel à faire par William (D4 : clavier + manette)

Ce que la simulation ne peut pas prouver : la lecture réelle des périphériques. Sur un poste avec écran, `cargo run -p zombies` (partie locale, `ALACOD_INPUT` non défini), puis :
1. **Clavier + souris** (touches de `character/control.rs`) : W/A/S/D déplacent, la souris oriente (le tir part vers le curseur), clic gauche tire, R recharge, Maj gauche sprint, C dash, Ctrl gauche modificateur, H interaction, F mêlée, Tab change d'arme, Z change de mode, L lâche l'arme, Espace objet actif, Q blank, E bombe, F12 plante le jeu (diagnostic). Constater : réponse **immédiate**, le personnage tire vers le curseur (pas décalé d'un tour), aucun blocage au premier appui.
2. **Manette** (branchée avant le lancement, puis débranchée et rebranchée en jeu) : croix directionnelle = déplacement, stick gauche = visée (`Pan`), Nord = interaction, Ouest = recharge, Est = mêlée, Sud = valide l'écran de mutation. Constater que ces boutons répondent comme avant.
3. **Premier appui après le chargement** : appuyer sur une direction dès l'apparition du joueur. D4 : l'`ActionState` du joueur local est posé jusqu'à **une frame de rendu** après sa création (`attach_local_player_input`, `PreUpdate`), l'input de cette première frame est neutre ; **tolérance annoncée, à constater** : aucun appui perdu visible, rien de plus qu'un instant.
4. **Écran de mutation** (jeu à progression, p. ex. `throne`) : 1/2/3 choisissent une carte, ←/→ naviguent, Entrée (ou Sud sur manette) valide la carte surlignée (`confirm_input`, lue côté application).
5. **Lâcher d'arme** (L) maintenu : une seule requête (test unitaire `held_drop_sends_one_request_and_rearms_after_release`), à constater à la main.
6. Avec `--features lighting` (défaut de `zombies`) : la lumière autour du joueur apparaît dès son entrée en scène (système `Update`, une frame de rendu après la création).
7. Deux joueurs locaux (`--players localhost localhost`) : chacun répond à ses touches, la caméra suit comme avant.
Ce qui est un défaut : un appui perdu au démarrage de plus d'une frame, une visée qui retarde sur la souris, un joueur dont les touches ne répondent pas.

## 9. Limites

- Manette et clavier **non essayés** (poste sans périphérique pilotable ici) : §8.
- Cliquet : sept fichiers de la fiche libérés mais hors cliquet (§4) ; `interaction.rs` (E5c) et la chaîne `run_state.rs` les débloquent.
- Captures de `gungeon_boss` : non identiques bit à bit à `origin/main` (anneau des halos), pour la cause du §6 ; identiques à la base + une ressource.
- Web : seulement compilé.
