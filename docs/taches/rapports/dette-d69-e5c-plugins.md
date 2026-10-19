# dette-d69-e5c-plugins : interaction, objets, vagues, brouillard, armes (D69 étape 2, E5c)

Branche `dette-d69-e5c-plugins`, base `a1edf161` (`origin/main` : E3d, E4, E5a, E6, D84). Fiche `docs/taches/dette-d69-etape2.md` § E5c. Refactoring pur : aucune trace ne bouge, aucun `BLESS`.
Coordination : b2 fait E5b (`feedback*.rs`) en parallèle ; mon diff de `core.rs` tient en trois lignes (voir § 1).

## 1. Coupes faites (en place, aucun fichier déplacé)

| point de la fiche | avant | après |
|---|---|---|
| 1. `interaction.rs` | `update_door_visuals`, `WindowHealthBar`, `update_window_health_bars`, `display_interaction_prompts` (`Visibility`, `Sprite`, `Gizmos`) et leur `add_systems(Update, …run_if(InGame))` dans `InteractionPlugin` | `interaction_view.rs` (**nouveau, app**) : les quatre éléments, corps octet pour octet, plus `InteractionPresentationPlugin` (`Update`, `run_if(in_state(AppState::InGame))`, mêmes trois systèmes dans le même ordre de tuple) ajouté à `PresentationPlugin`. `interaction.rs` : plus de `AppState`, plus de fuite (5 → 0) |
| 2. `items.rs` | `LastBlank`, `BlankUsedAt`, `BLANK_WAVE_FRAMES`, `track_blank_system`, `draw_blank_wave_system` (`Gizmos`), `BlankPresentationPlugin` | `items_view.rs` (**nouveau, app**), même code ; `core.rs` pointe `crate::items_view::BlankPresentationPlugin` **à la même ligne**. `items.rs` : 1 → 0 fuite. `EffectAssets` lisait déjà `WeaponRegistry` (E3b) : rien à faire |
| 3. `waves/mod.rs` | `WaveSystemPlugin::build` ajoutait `WaveDebugPlugin` ; `pub use debug::WaveDebugPlugin` | plus de `pub use` ni d'appel ; `PresentationPlugin` ajoute `crate::waves::debug::WaveDebugPlugin`. `WaveAssetsPlugin` était déjà ajouté par `core.rs` avant `WaveSystemPlugin` (E3c) ; `waves/debug.rs` et `waves/assets.rs` restent app |
| 4. `room_fog.rs` | — | rien à couper : sans `Fog*` depuis E1, déjà tout app, `RoomFogPlugin` déjà dans `PresentationPlugin` (`capture_vision` n'a jamais tourné en headless) ; seul son import de `WindowHealthBar` suit (1 ligne) |
| 5. armes | `weapon_inventory_system` (`Visibility`, marqueur `ActiveWeapon`) et `hide_new_melee_weapons` déjà dans `weapon_assets.rs` (E3b) mais enregistrés par `WeaponAssetsPlugin`, donc **en headless aussi** | enregistrés par `WeaponPresentationPlugin` (`hide_new_melee_weapons` en `PreUpdate.after(RunGgrsSystems)`, `weapon_inventory_system` en `Update`) ; `WeaponAssetsPlugin` garde chargeurs, synchro du registre et `weapons_config_update_system`. `hide_new_melee_weapons` devient `pub`. `ActiveWeapon` n'est lu que par `ui/weapon_visuals.rs` (vérifié par recherche : aucune lecture en simulation, `system_weapon_position` lit `WeaponInventory`) |
| acceptation | `apply_player_overrides.before(game::weapon_assets::weapon_inventory_system)` (`scenario/runner.rs`) | `.before` retiré avec son commentaire (un runner de simulation n'a plus à nommer un système de présentation). Pour que l'ordre ne soit plus requis : `weapon_inventory_system` pose et retire `ActiveWeapon` et `Visibility` par `try_insert`/`try_remove` (une arme despawnée dans la même frame par `apply_player_overrides` ne fait plus paniquer la commande différée) ; comportement identique sur une entité vivante |

Appelants migrés (règle 4, pas d'alias) : `WindowHealthBar` (`room_fog.rs`, `map_ldtk/game/floors.rs`, `map_ldtk/game/plugin.rs`) → `game::interaction_view::WindowHealthBar` ; `docs/conventions.md` (deux mentions) et `CLAUDE.md` (une mention).

`core.rs`, tout le diff : `items::BlankPresentationPlugin` → `items_view::BlankPresentationPlugin`, plus deux lignes ajoutées juste après (`InteractionPresentationPlugin`, `WaveDebugPlugin`). `lib.rs` : deux `pub mod`. Aucun type rollback créé, retiré ni déplacé, aucun
système de `GgrsSchedule` touché.

**Effet voulu en headless** : `update_door_visuals`, `update_window_health_bars`, `display_interaction_prompts`, `WaveDebugPlugin`, `weapon_inventory_system` et `hide_new_melee_weapons` ne tournent plus sans `PresentationPlugin` (ils n'écrivent que `Visibility`, `Sprite`,
gizmos, texte d'UI et `ActiveWeapon`, jamais lus par la simulation). Les traces le confirment (§ 3).

## 2. Sonde avant/après (`make probe_game_sim`, `game` en ECS seul)

| | avant (`a1edf161`) | après |
|---|---|---|
| fichiers du crate | 135 | 137 (`interaction_view.rs`, `items_view.rs`) |
| fichiers à fuite | 44 | 44 |
| erreurs de résolution | 752 | 752 |
| catégorie `gizmos` | 9 | 9 |

Les fuites ont **changé de fichier** : `interaction.rs` (5 : `sprite/caméra` 4, `gizmos` 1) et `items.rs` (1 : `gizmos`) sont libérés ; `interaction_view.rs` (5) et `items_view.rs` (1) les portent (app par construction).

**Cliquet : 35 → 76 fichiers** (`scripts/d69-sim-files.txt`, `probe-game-sim.py --sim-list` **et** `--tests` : sortie 0). La fiche demandait `interaction.rs`, `items.rs`, `weapons.rs` (déjà là depuis E3b) et `waves/mod.rs` ; chacun
fermait la porte de toute la chaîne qui les attendait depuis E3a, d'où 41 ajouts, calculés comme la plus grande fermeture interne de fichiers sans fuite :
`balance.rs`, `bombs.rs`, `boss.rs`, `champions.rs`, `character/{mod,create,config}.rs`, `character/health/mod.rs`, `character/player/{input,create}.rs`, `character/enemy/{create,spawning}.rs`, `character/enemy/ai/{pathing,behavior,rules,squad,charm}.rs`,
`economy.rs`, `effects_runtime.rs`, `effects_v3.rs`, `familiars.rs`, `hub.rs`, `interaction.rs`, `items.rs`, `jjrs/{local,p2p}.rs`, `loot.rs`, `powerups.rs`, `profile.rs`, `progression.rs`, `props.rs`, `recording.rs`, `replay.rs`, `run_state.rs`, `shop.rs`, `state_trace.rs`,
`ui/hud_model.rs`, `ui/mutation_screen.rs`, `ui/screens/mod.rs`, `waves/mod.rs`, `waves/systems.rs`.

**Non ajoutés** bien que sans fuite et fermés (application par nature : ils ne compilent en ECS seul que parce que leur contenu est sous `cfg` ou feature) : `args/{cli,web}.rs`, `audio/adaptive.rs`, `jjrs/{allumette,browser,p2p_socket_tests}.rs`, `light/mod.rs`, `logs.rs`, `performance.rs`,
`ui/{mod,game_over,mutation_hui,weapon_hud}.rs`, `ui/screens/hui.rs`, `web/mod.rs`, `lib.rs` (16). `feedback_v3.rs` reste dehors (dépend de `feedback.rs`, E5b). Commentaire daté dans le fichier.

## 3. Preuves

| vérification | résultat |
|---|---|
| `make check_sim_boundary` | OK (13 crates SIM, 2 en dette D69) |
| `make schedule_order_diff BASE=a1edf161` | **sortie 0** : 4 520 lignes de schedules, 645 types rollback identiques |
| `make test_scenarios_par SHARDS=3` sans BLESS | 3 tranches `ok`, **802 scénarios**, **aucun `.trace` modifié** (`git status` : seulement code et docs) |
| `make test_crates` | `exit 0` |
| `make lint` ×5, `make gen GAME=…` ×5 | 0 erreur ; scénarios générés tous `ok ok` ; aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` / `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` (`--ignored`) | OK / OK |
| `cargo fmt --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` (25 tests) | OK |
| `cargo check -p game --features lighting,ui_hui,debug_ui` ; build web `APP_VERSION=e5c cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` ; `cargo check --workspace --all-targets` | OK ×3 |
| `make probe_game_sim SIM_LIST=… ` et `--tests` | sortie 0, 76 fichiers |
| **p2p-stress zombies** (`p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3573`, `COMPOSE_PROJECT_NAME=b3e5c`) | codes 0 0, 0 `DesyncDetected`, **1 941 frames resimulées** (21 rollbacks, profondeur max 11), 2 999 lignes identiques |
| **p2p-stress isaac** (idem) | codes 0 0, 0 `DesyncDetected`, **1 966 frames resimulées** (21 rollbacks), 2 999 lignes identiques |
| captures `play_scenario --capture --every 2` (rendu réel sur le GPU du Mac, `ALACOD_MUTE=1`), binaire de `a1edf161` (worktree jetable, retiré) contre le code final, comparées image par image par `cmp` | **0 image différente** : `door_open` 165/165 (portes, brouillard de la carte `le_relais_modules`), `window_repair` 550/550 (barre de vie des fenêtres), `blank_clears_bullets` 35/35 (onde du blank), `two_players_shooting` 150/150 (vagues avec overlay `WaveDebugEnabled`, brouillard), `movement_melee` 450/450 (arme de mêlée cachée/montrée), `ammo_burst` 450/450 et `aim_four_way` 80/80 (surcharge d'arme de joueur par `apply_player_overrides` : le `.before` retiré) |

## 4. Limites

- Le p2p demandé par la fiche pour E5c n'est que « porte commune » ; il est rejoué (deux jeux) parce que l'enregistrement des systèmes de présentation a bougé. `docker` n'est pas dans le `PATH` d'un shell non interactif du Mac (`/usr/local/bin`, OrbStack) : la première tentative a échoué
  sur ce point (`docker: command not found`), pas sur le code ; rejouée avec `PATH` complété.
- Les captures n'exercent pas le rechargement à chaud des armes ni un joueur qui change d'arme pendant qu'une surcharge de scénario est appliquée la même frame : le cas panique-avant (`insert` sur une arme despawnée) est couvert par le passage à `try_insert`/`try_remove`, pas par un scénario.
- Conflit probable avec E5b (b2) à la fusion : `items_view.rs` lit `crate::feedback::FeedbackConfigLoaded`, que E5b déplace vers `feedback_log.rs` ; corriger l'import (une ligne) au moment de la fusion.
- Aucune relecture indépendante lancée.
