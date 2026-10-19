# dette-d69-e7-core : `CoreSetupPlugin` en deux plugins (D69 étape 2, E7)

Branche `dette-d69-e7-core` (worker b2), base `origin/main` `f80c1ea3` (E1 à E6 fusionnées, dont E5b). Fiche `docs/taches/dette-d69-etape2.md` § E7 ;
annexe `docs/taches/rapports/dette-d69-plan-etape2/annexe-session.md` § 1.1 (56 éléments numérotés) et § 1.3. Refactoring pur : **aucune trace ne bouge, aucun `BLESS`**.

## 1. Ce qui change

| | simulation : `crates/game/src/core_sim.rs` (nouveau) | application : `crates/game/src/core.rs` |
|---|---|---|
| plugin | `CoreSimPlugin(CoreSetupConfig)` : `build` + `finish` (`single_threaded_rollback_schedules`) | `CoreAppPlugin(CoreSetupConfig)` ; `CoreSetupPlugin` (garde son nom : `build` = `CoreSimPlugin(cfg.clone())` **puis** `CoreAppPlugin(cfg)`) |
| autres | – | `get_default_plugin`, `PresentationPlugin` (corps **inchangé**), `GameInfo`, `debug_ui_enabled` |
| test | `core_sim_seul_sur_minimal_plugins` (§ 4) | – |

`core.rs` passe de 527 à 315 lignes, `core_sim.rs` fait 359 lignes dont le test. Aucun appelant n'a changé de chemin (`games/*`, `examples`, `templates`, `scenario` lisent toujours `game::core::CoreSetupPlugin`).

### Répartition des 56 éléments de l'annexe § 1.1 (état de `origin/main`, après E3-E6)

| `CoreSimPlugin::build`, dans l'ordre de l'ancien `build` | `CoreAppPlugin::build`, dans l'ordre de l'ancien `build` |
|---|---|
| 0 `TimeUpdateStrategy` (headless) ; 2 `AnimationStatePlugin` ; 4 `GgrsPlugin` ; 6 `BaseWeaponGamePlugin` ; 8 `(Combat, Behaviors, Effects)` ; 9 `BaseColliderGamePlugin` ; 10 `BaseCharacterGamePlugin` | 1 `WebPlugin` ; 3 `D2AnimationPlugin` ; 5 `PresentationPlugin` (si pas headless) ; 7 `WeaponAssetsPlugin` ; 10 `CharacterAppPlugin` |
| 11-15 `RunPlugin`, `WorldPlugin`, `EffectsRuntimePlugin`, `EffectsV3Plugin`, `ProgressionPlugin` | 16 `EconomyAssetsPlugin` ; 18 `PowerUpsAssetsPlugin` ; 30 `FeedbackAssetsPlugin` |
| 16 `EconomyPlugin` ; 17 `InteractionPlugin` ; 18 `PowerUpsPlugin` ; 19-29 `ItemsPlugin` … `ProfilePlugin` | 31 `GameUiPlugin` (lobby, déconnexion) puis `GameUiInputPlugin` (**rattachement de E5a**) ; 32 `WaveAssetsPlugin` |
| 30 `FeedbackLogPlugin` ; 31 les trois modèles (`MutationScreenModelPlugin`, `HudModelPlugin`, `ScreensPlugin`) ; 32 `WaveSystemPlugin` ; 33 `RunStatePlugin` | 34 `GameInfo` ; 41 `Startup : add_global_asset` ; 42 `Update : loading_asset_system` |
| 34 `GggrsSessionConfigurationState`, `GgrsNetIdFactory`, `FrameCount` ; 35 `RngStreams` ; 36 `SessionLostEvent` et `DesyncObserved` ; 37 `init_state` | 44 `OnEnter(LobbyOnline)` : allumette, `start_matchbox_socket` |
| **38 les cinq enregistrements de clôture, derniers enregistrements rollback** ; 39 `configure_sets` ; 40 profilage | 45 `wait_for_players` ; wasm `return_to_host_lobby` |
| 43 `PostUpdate : sync_bevy_transforms_from_fixed` | 46 `LobbyConnectWatch`, `observe_session_peers` (`.before(abandon_unsynchronized_session)`), `drop_matchbox_socket` |
| 45 `setup_ggrs_local` ; 46 ressources de session, `reset_sync_watch`, `abandon_unsynchronized_session`, `reset_online_games`, `count_online_game` | 47 `OnEnter(GameStarting)` : `build_p2p_session` |
| 47 `start_online_run`, `system_after_map_loaded_local` ; 48 `resolve_{pattern_library,progression,status_library,clocks}` ; 49 `log_ggrs_events` | 48 `publish_session_players` (`.before(resolve_clocks_system)`), `resolve_balance_system`, `resolve_powerups_table_system` |
| 50-52 `StateTracePlugin`, `RecordingPlugin`, `SnapshotProfilePlugin` ; 55 `clock_system`, `increase_frame_system`, `debug_desync_system` | 49 `forward_desync_telemetry` (`.after(log_ggrs_events)`) ; `ExitSystems.before(ExitRequests)` ; 53 `ContentHotReloadPlugin` ; 54 `RemoteControlPlugin` |

Les trois `.chain()`/`.after` qui reliaient un système d'application à un système de simulation (`(observe_session_peers, abandon_unsynchronized_session)`, `(log_ggrs_events, forward_desync_telemetry)`,
`resolve_clocks_system.after(publish_session_players)`) deviennent des arêtes `.before`/`.after` posées **depuis l'application vers la simulation** (jamais l'inverse), avec la même condition d'état.
Les lectures de ressources de `build` (`ProfilePlugin` : `ProfileSettings` ; `CharacterAppPlugin` : `DebugAiConfig`) restent après les `insert_resource` de l'appelant (`scenario/runner.rs:306-318`).

### Écarts à la fiche (tous justifiés)

1. **`GameUiModelPlugin` supprimé** (E5a l'avait créé dans `ui/mod.rs`, fichier application). La fiche demande que `core_sim.rs` n'importe rien d'`ui` : `CoreSimPlugin` ajoute donc directement `MutationScreenModelPlugin`,
   `HudModelPlugin` et `ScreensPlugin` (même ordre qu'avant), trois plugins qui vivent dans des fichiers déjà au cliquet (`ui/mutation_screen.rs`, `ui/hud_model.rs`, `ui/screens/mod.rs`). `ui/mod.rs` ne garde que `GameUiPlugin` et `GameUiInputPlugin`.
2. **`finish`** : porté par `CoreSimPlugin` (fiche, point 1). `CoreSetupPlugin` n'en a plus : bevy appelle `finish` de chaque plugin imbriqué (`SubApp::finish` parcourt `plugin_registry`, où l'ajout imbriqué est inscrit), au même rang
   qu'avant (juste après le plugin englobant, avant les plugins qu'il ajoute). Un second appel serait redondant. L'exécuteur monothread n'a pas d'accesseur dans bevy 0.19 : `finish` n'est donc pas assertable ; le coût du rollback (`rollback_ms_worst`) le trahirait.
3. **`schedule_order.rs`** gagne l'option `ALACOD_SCHEDULE_ALL=1` (§ 3.2) ; la sortie par défaut, celle de `make schedule_order_diff`, ne change pas d'un octet.

## 2. Sonde `make probe_game_sim`, cliquet

| | avant (`f80c1ea3`) | après |
|---|---|---|
| fichiers du crate | 138 | 139 (`core_sim.rs`) |
| fichiers à fuite | **44** | **44** |
| erreurs de résolution | **754** | **754** |
| `core.rs` | 12 (fenêtre 6, asset 3, rendu/remote 1, crate d'E/S 1 ; la sonde n'affiche que les quatre premières catégories) | 12 (idem : application) |
| `core_sim.rs` | – | **0** (absent de la liste des fuites) |

Cliquet : `core_sim.rs` **entre**, et avec lui `feedback_log.rs` et `feedback_v3.rs` (refusés à E5b : fermeture à fuite ; leurs dépendances, `boss.rs`, `bombs.rs`, `familiars.rs`, `character/config.rs`, `character/health/mod.rs`, sont au cliquet depuis E5c). **76 → 79 fichiers**, sonde `OK : les 79 fichiers … compilent en ECS seul et n'en appellent aucun autre`. Premier essai avec `core_sim.rs` seul : refusé (« dépend de fichiers hors liste : `feedback_log.rs` »), d'où l'ajout des deux fichiers du journal, calculé et vérifié par la sonde.
`core.rs` reste application (`get_default_plugin` : `DefaultPlugins`, `WinitPlugin`, `RenderPlugin`).

## 3. Preuves d'ordre

### 3.1 `make schedule_order_diff BASE=f80c1ea3` : **sortie 0, strictement vide**

`OK : ordre des systèmes et des types rollback identiques (4520 lignes de schedules, 645 types)`, pour les cinq jeux (zombies, gungeon, isaac, throne, testbed) : `ReadInputs`, `GgrsSchedule`, `AdvanceWorld`, `SaveWorld`, `LoadWorld` octet pour octet,
types rollback dans le même ordre, les cinq types de clôture en dernier (`GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`, `RngStreams` : positions 124 à 128). Mêmes chiffres qu'à E5b.

### 3.2 Ce que l'outil ne couvre pas : `Update`, `PostUpdate`, `OnEnter`… et la présentation

`make schedule_order_diff` ne regarde que les cinq schedules du rollback, en headless (donc sans `PresentationPlugin`). Pour le reste, j'ai étendu `crates/scenario/tests/schedule_order.rs`
(option `ALACOD_SCHEDULE_ALL=1`, hors sortie par défaut) : il écrit **tous** les schedules dans l'ordre exécutable **et, pour chacun, les paires de systèmes non ordonnés qui se disputent une donnée**
(`ScheduleGraph::conflicting_systems`), dans l'ordre où elles s'exécutent. Ce sont les seules paires dont le résultat peut dépendre de l'ordre d'insertion des plugins. Deux sorties : `<jeu>.all.txt` (headless, 5 jeux) et
`presentation.all.txt` (`CoreSetupPlugin` avec `headless: false` sur des plugins par défaut headless : `PresentationPlugin` à sa place exacte, sans fenêtre ni GPU ; ce dump a été produit sur `f80c1ea3` et sur la branche).

**Constats** (mêmes systèmes dans chaque schedule des deux côtés : aucun ajouté, retiré ni doublé ; mêmes schedules) :

- **Constat structurel** : le tri topologique de bevy ne suit pas l'ordre d'insertion. Un plugin déplacé retourne aussi des paires qu'il ne touche pas (`apply_run_request_system` / `setup_ggrs_local`, deux systèmes de simulation dont l'ordre
  d'insertion relatif est conservé, passent de « premier » à « second »). Une preuve « mêmes paires dans le même ordre » est donc inatteignable en dehors du rollback, pour E7 comme pour toute étape qui ajoute ou déplace un plugin (E4, E5, E6 l'ont fait sans l'outil).
- **`GgrsSchedule`, `SaveWorld`, `LoadWorld`, `AdvanceWorld`, `ReadInputs`, `Startup`, `PreUpdate`, `First`, `Last`, `StateTransition`** : aucune paire en conflit retournée, en headless comme avec présentation. Dans `GgrsSchedule`, bevy refuse toute ambiguïté (`ambiguity_detection: Error`) :
  l'ordre exécutable peut changer (c'est le cas **avec** présentation : l'ordre de `clear_frame_events` et de quelques systèmes à dépendances croisées, `capture_vision` de `RoomFogPlugin` de 172 à 164, déjà différent entre headless et présentation sur `f80c1ea3`), pas le résultat.
- **Headless, 20 paires retournées** (`Update` 18, `OnEnter(GameStarting)` 1, `OnEnter(LobbyOnline)` 1), examinées par lecture du code, sauf le chargement de carte (dernière ligne) :

| paire (avant → après) | verdict |
|---|---|
| `OnEnter(GameStarting)` : `start_online_run` / `system_after_map_loaded_local` | exclusives : chacune sort si `OnlineState` n'est pas le sien |
| `OnEnter(LobbyOnline)` : `reset_online_games` / `start_matchbox_socket` | `reset_online_games` ne lit de `OnlineRestart` que son existence ; `start_matchbox_socket` n'écrit que `restart.since` : aucune dépendance |
| `loading_asset_system` (état `Loading`) / `apply_run_request_system`, `setup_ggrs_local` (`LobbyLocal`), `wait_for_players` (`LobbyOnline`), `transition_to_game_starting` (écrit `GameStarting` sur fin de chargement de carte, en `GameLoading`) | états exclusifs ; aucune `RunRequest` n'existe en `Loading` |
| `loading_asset_system` / `apply_weapon_overrides` (scénarios) | `loading_asset_system` ne lit de `WeaponRegistry` que « vide ou non » (garde de sortie de `Loading`) ; la surcharge ne vide ni ne remplit : aucune dépendance |
| `apply_run_request_system` / `setup_ggrs_local`, `wait_for_players` | une `RunRequest` est consommée en `InGame` ; ces deux systèmes ne tournent qu'en lobby |
| `apply_run_request_system` / `handle_disconnect_event` (UI) | un abandon et un `SessionLostEvent` dans le même update : l'écran « déconnecté » peut passer d'absent à présent (présentation seule, aucun état de simulation) |
| `abandon_unsynchronized_session` / `log_ggrs_events`, `observe_session_peers` / `log_ggrs_events` | `Session` lue des deux côtés (`abandon` : `current_state()` ; `log` : vidage des événements) : indépendants |
| `observe_session_peers`, `wait_for_players` / `update_lobby_ui`, `update_lobby_status` | l'interface du lobby lit `LobbyConnectWatch` : au plus une frame de décalage de texte |
| `apply_player_overrides` / `apply_weapon_overrides`, `create_wall_colliders_from_ldtk`, `wait_for_all_map_rollback_entity` ; `spawn_characters_when_map_loaded` / `init_floor_state_when_map_loaded` ; `transition_to_game_starting` / `wait_for_all_map_rollback_entity` | chargement de carte et surcharges de scénario (`map_ldtk`, `scenario`), jamais touchés ici : retournés par le tri ; **non jugés un à un** : la preuve est la suite entière sans BLESS (§ 4), qui joue les 807 scénarios |

- **Avec présentation, 138 paires retournées** : `Update` 126 (107 entre deux systèmes de présentation, 14 entre un système de présentation et un système d'animation ou de lobby, 5 sans présentation), `PostUpdate` 10 (6 présentation/présentation, 4
  présentation/`sync_bevy_transforms_from_fixed`), `OnEnter` 2. Les quatre paires `sync_bevy_transforms_from_fixed` ↔ `animate_damage_numbers`, `apply_boss_zoom`, `update_camera_shake_offset` (et `animate_debris`/`compute_slices_on_asset_event`) :
  `sync_bevy_transforms_from_fixed` ne touche que les entités qui portent `FixedTransform3D` ; la caméra et les chiffres de dégâts n'en ont pas : **mêmes composants, entités disjointes**, aucun effet. Les autres paires sont des systèmes de rendu
  (`sync_labels`, `skin_bullets`, `attach_world_sprites`, `update_hud_values`, `camera_control_system` …) qui se disputent `Sprite`/`Visibility`/`Transform` sur des entités différentes.
  **Aucun effet sur la simulation** ; la seule preuve possible sur l'aspect est la capture comparée (§ 5, **non faite**).

`PresentationPlugin` et `get_default_plugin` : corps inchangés (`diff` vide entre `f80c1ea3:crates/game/src/core.rs` lignes 415-478 et 480-527 et les lignes 203-266 et 268-315 de la nouvelle version).

## 4. Vérifications (arbre de la branche, résultats réels)

Machine : VPS Debian 8 cœurs, partagée (un runner CI y tourne : charge 3 à 7). **Le binaire `rustup` de `~/.cargo/bin` a disparu à 14:05** (les liens `cargo`, `rustc` pointent dans le vide ; autre processus que le mien) : les essais p2p, qui
appellent `cargo` depuis un script, ont été lancés avec `PATH=~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH` ; rien n'a été réinstallé.

| vérification | résultat |
|---|---|
| `make schedule_order_diff BASE=f80c1ea3` | **sortie 0** (§ 3.1) |
| `make check_sim_boundary` | OK : 13 crates SIM, 2 en dette D69 |
| `make test_scenarios_par SHARDS=3` sans BLESS | 3 tranches `ok`, **807 scénarios** fusionnés (les 802 de E5b + 5 de `m3-t18`), 1 862 s ; **aucun `.trace` modifié** (`git status` : seuls le code et les docs de la livraison) |
| `make test_crates` | `exit 0` : 84 suites, **1 304 tests passés** (1 303 + `core_sim_seul_sur_minimal_plugins`), 0 échec, 19 ignorés (+ `dump_presentation_order`) |
| `cargo fmt --all -- --check`, `scripts/check-forbidden.sh`, `scripts/check-rollback-registration.sh`, `make test_tools` (25 tests) | OK ×4 |
| `make lint` ×5 | 0 erreur (zombies, testbed, throne, gungeon, isaac ; avertissements existants : testbed 115, gungeon 1) |
| `make gen GAME=…` ×5 | **431 scénarios générés joués, tous `ok ok`** (zombies 16, testbed 104, throne 39, gungeon 59, isaac 213), aucune trace différente |
| fuzz `ALACOD_FUZZ=0:8` (`--test fuzz_inputs -- --ignored`) | OK (459 s) |
| `cargo check -p game --features lighting,ui_hui,debug_ui` | OK |
| build web `APP_VERSION=e7 cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` (les blocs `cfg(wasm32)` de `CoreAppPlugin` : `return_to_host_lobby`, `start_matchbox_socket`) | OK |
| `cargo check --workspace --all-targets` | OK |
| `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` | sortie 0 ; 79 fichiers (§ 2) |
| **p2p-stress zombies** (`p2p_smoke.ron`, 3 000 frames, `MATCHBOX_PORT=3566`, `COMPOSE_PROJECT_NAME=b2`) | **1 639 frames resimulées** (client 0 : 20 rollbacks, profondeur max 11 ; client 1 : 1), codes 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| **p2p-stress isaac** (`ALACOD_SKIP_HUB=1`, idem) | **1 581 frames resimulées** (18 rollbacks, profondeur max 11), codes 0 0, 0 `DesyncDetected`, 2 999 lignes identiques |
| `p2p-restart.sh zombies` | OK : traces identiques partie par partie, partie locale 2 identique à la 1 (599 lignes) |
| `p2p-lobby.sh zombies` | OK : 5/5 essais verts consécutifs dans la bande 100-400 ms, 1 pair obsolète ignoré par essai, 0 abandon |
| `p2p-disconnect.sh zombies` | invariants OK (survivant terminé, 0 `DesyncDetected`, 4 499 lignes) |
| `p2p-sync-timeout.sh zombies` | OK : abandons `session_sync_timeout` puis `session_peer_left`, retrouvailles, 299 lignes identiques |

Les quatre derniers essais sont ceux de l'ordre de `Update` : lobby (`wait_for_players`, `observe_session_peers`, `abandon_unsynchronized_session`, `handle_disconnect_event`, `apply_run_request_system`, `build_p2p_session`, `forward_desync_telemetry`), qui est précisément ce que § 3.2 retourne.

## 5. Captures `cmp` (non faites ici)

`play_scenario --capture` ouvre un contexte `bevy_winit` : sans affichage ni GPU il échoue (D94 : « Failed to build event loop », `xvfb-run` absent sur ce VPS). **Non tenté ici, non contourné.** À rejouer sur le Mac, `cmp` image par image entre `origin/main`
(`f80c1ea3`) et la branche. Les scénarios couvrent les systèmes retournés de § 3.2 (HUD, brouillard, chiffres de dégâts, secousse et zoom de boss, fondu d'étage et portail, pause, télégraphes, blank) :

```bash
# Sur le Mac, depuis le dépôt : BASE = f80c1ea3 (origin/main), E7 = la branche dette-d69-e7-core
git worktree add --detach ../alacod-base f80c1ea3
git worktree add ../alacod-e7 dette-d69-e7-core
SCENARIOS="two_players_shooting ui_pause_menu gungeon_boss isaac_hud_items tears_shot_speed bot_floors_three \
  champion_splits blank_clears_bullets status_charm"
for tree in alacod-base alacod-e7; do
  ( cd ../$tree && for s in $SCENARIOS; do
      cargo run -q -p scenario --features render --bin play_scenario -- tests/scenarios/$s.ron \
        --capture /tmp/e7/$tree/$s --every 2 ; done )
done
for s in $SCENARIOS; do
  for f in /tmp/e7/alacod-base/$s/*.png; do cmp -s "$f" "/tmp/e7/alacod-e7/$s/$(basename "$f")" || echo "DIFFÈRE $s $(basename "$f")"; done
done
```

`two_players_shooting` joue la carte du Relais avec brouillard actif ; `bot_floors_three` traverse des étages (fondu `draw_floor_fade`, portails) ; `ui_pause_menu` ouvre l'écran de pause (`GameUiInputPlugin`, maintenant dans `CoreAppPlugin`).
Pour le dump d'ordre avec présentation sur le Mac (mêmes fichiers que § 3.2, sans fenêtre) :
`ALACOD_SCHEDULE_ALL=1 ALACOD_SCHEDULE_DUMP=/tmp/e7/head cargo test -q -p scenario --profile headless --test schedule_order -- --include-ignored` (idem dans un worktree de `f80c1ea3` avec ce fichier de test copié, puis `diff -r`).

## 6. Limites

- **Aucune preuve visuelle** (§ 5). Ce qui la remplace : la simulation lit les mêmes états aux mêmes frames (807 scénarios sans BLESS, p2p) ; le corps de `PresentationPlugin` n'a pas changé d'une ligne ; les paires de présentation retournées sont toutes entre systèmes qui se disputent un type de composant sur des
  entités différentes (§ 3.2), jugées par lecture du code pour les quatre qui touchent `sync_bevy_transforms_from_fixed`, **non lues une à une pour les 107 paires présentation/présentation** : elles ne se prouvent que par les captures.
- Les features `ui_hui`, `lighting` et `debug_ui` n'ont pas de dump d'ordre (seulement `cargo check`) : leurs systèmes n'existent pas dans le dump de présentation.
- L'ordre des plugins `D2AnimationPlugin`, `WeaponAssetsPlugin`, `*AssetsPlugin` et de `PresentationPlugin` par rapport aux plugins de simulation change (ils passent après) ; aucun n'enregistre de type rollback (`*.rollback.txt` identique, § 3.1) ; l'app se construit en headless et avec `PresentationPlugin` (dump du § 3.2).
- `CoreSetupPlugin::finish` n'existe plus (§ 1, écart 2).
- `GameUiModelPlugin` n'existe plus (§ 1, écart 1) ; `docs/conventions.md` § 56 et `CLAUDE.md` § « Simulation et présentation » sont à jour.
