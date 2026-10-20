# dette-d69-plan-etape2 : plan exécutable de l'étape 2 de D69 (`game_sim` / `game_app`)

Branche `dette-d69-plan-etape2` (worker b3), créée depuis `09b68187`, avancée (fast-forward, sans conflit) sur `8633a560` (fusion
de `m2-p2p-deux-machines`, qui a modifié `jjrs/p2p.rs` et `state_trace.rs`). **Analyse, outillage et cinq retouches sur
place ; aucun fichier déplacé, aucune trace ne bouge.** Suite : fiche prête à dispatcher
`docs/taches/dette-d69-etape2.md` (une section par étape). Contexte : étape 1 `docs/taches/rapports/dette-d69-combat-sim.md`,
D69 dans `docs/taches/dettes.md`, frontière de portage console `CLAUDE.md` et `docs/plan-engine.md` §7.

## 0. Résultats

| | |
|---|---|
| `game` | **114 fichiers, 47 863 lignes** : SIM 53 fichiers (24 914 l.), MIXTE 28 (13 275 l.), APP 33 (9 674 l.) |
| Compilé en ECS seul (sonde §1) | **59 fichiers sur 114 portent au moins une fuite** (781 erreurs de résolution avant correction des suivantes) ; 55 sans erreur directe |
| Fichiers séparables **aujourd'hui** (sans fuite **et** ne dépendant que d'autres fichiers séparables) | **16 fichiers, 2 405 lignes (5 %)** : `scripts/d69-sim-files.txt` |
| Fuite dominante dans la simulation | `Assets<…>`/`Handle`/`GlobalAsset` : 176 erreurs de résolution dans 41 fichiers, dont **12 fichiers de simulation, 27 erreurs** (`champions`, `character/{create,enemy/create,enemy/spawning,health/mod,ai/pathing}`, `effects_runtime`, `familiars`, `loot`, `progression`, `shop`, `waves/systems`). Les 303 erreurs « ui » sont dans des fichiers APP purs (`ui/hud.rs` 154) |
| `map_ldtk` | 32 fichiers (1 mort), 8 946 lignes ; ≈ 41 % de logique de simulation (16 systèmes `GgrsSchedule`) mais liée à `bevy_ecs_ldtk` ; décision : APP d'abord (E9/E10), `map_ldtk_sim` ensuite (E11) |
| Graphe interne | un seul bloc fortement connexe de **69 fichiers sur 114** (à cause de `core.rs`, `character/mod.rs`, `weapons.rs`, `global_asset.rs`) ; **18 dépendances remontantes sim → app** à casser (§4), toutes tracées fichier:ligne |
| Plan | **11 étapes, 18 livraisons** (E1, E2, E3a-d, E4, E5a-c, E6, E7, E8, E9, E10, E11a-c) ; phase A (E1-E8) **sans aucun déplacement de fichier**, phase B (E9, E10) = les seuls déplacements, mécaniques |
| Estimation | ≈ 28 jours-agent au total, **chemin critique ≈ 12 jours** avec trois voies parallèles (§6) ; [INFERENCE : tailles mesurées, durées estimées, voir §6] |
| Preuve « ordre inchangé » | outil livré : `make schedule_order_diff` (ordre exact des systèmes de `ReadInputs`, `GgrsSchedule`, `AdvanceWorld`, `SaveWorld`, `LoadWorld` + ordre des types rollback, noms sans chemin de module) ; sensibilité **prouvée** (§7) |
| Retouches faites | 5, sur place, sans trace bougée (§10) : la sonde passe de 62 à 59 fichiers à fuite |

Deux écarts avec l'énoncé, à valider (§9) : **(1)** `scenario` ne peut pas ne dépendre que de la simulation (il joue les cartes avec
`LdtkRoguePlugin`, charge `AssetPlugin` et construit l'app avec `DefaultPlugins` même en headless) ; il reste `APP`, dépend de
`game_sim` **et** de `game_app` (sans la feature de rendu). `bots` en revanche ne lit plus que `game_sim` (et `map_ldtk::game::rooms` : `CurrentRooms` et trois fonctions) dès E3 + E10, et devient `SIM`
après E11. **(2)** `game_app` ne réexporte rien de `game_sim` :
les appelants changent de chemin (E9, E10), coupure franche comme demandé.

## 1. Méthode et outils (reproductibles)

Trois mesures automatiques et une lecture du code ; aucun chiffre du rapport n'est une estimation à la main sauf mention `[INFERENCE]`.

1. **Sonde de compilation** `scripts/probe-game-sim.py` (`make probe_game_sim`). Copie l'arbre de travail dans un dossier
   jetable, réécrit `crates/game/Cargo.toml` (bevy `default-features = false` + `serialize`, `bevy_color`, `bevy_state` ; sans
   `bevy_kira_audio`, `bevy_matchbox`, `leafwing-input-manager`, `bevy_common_assets`, `bevy_hui`, `bevy_light_2d`,
   `bevy-inspector-egui`, `animation`, `telemetry`, `harmonium_bevy`, `tracing-web`), lance `cargo check -p game` et range les erreurs de
   résolution **par fichier**. Un fichier sans erreur compile en ECS seul. Le dépôt n'est jamais modifié ; 18 s en incrémental.
   `--sim-list` ajoute le cliquet de la fiche : les fichiers listés doivent rester sans fuite **et** ne dépendre (`use crate::…`)
   que de fichiers listés (test négatif joué : un fichier hors fermeture, un fichier à fuite et un fichier inexistant sont refusés).
   Constats qui fondent le plan : `bevy_state` suffit (`in_state`, `OnEnter`, `NextState`, `States` compilent), `Time`,
   `TimeUpdateStrategy`, `Vec2`, `Color` sont disponibles sans les features par défaut ; il manque seulement ce que le §2 liste.
   Limite : rustc s'arrête après la résolution des noms ; relancer après chaque correction (les erreurs de typage cachées
   derrière apparaissent alors). `--tests` compile aussi les tests inline : **61 fichiers/813 erreurs** (au lieu de 59/781), les deux fichiers de plus étant `effects_v3.rs:557` (`Assets` dans un test) et `jjrs/p2p_tests.rs:151` (`bevy_matchbox::prelude::PeerId`, même type que `matchbox_protocol::PeerId`), plus 32 erreurs de tests dans des fichiers déjà à fuite (`world_sprites`, `room_fog`, `player/input`, `effects_runtime`, `powerups`). Les fichiers compilés seulement avec `ui_hui`, `lighting`, `debug_ui`, `harmonium` ne sont pas vus
   (`ui/game_over.rs`, `ui/mutation_hui.rs`, `ui/screens/hui.rs`, `ui/weapon_hud.rs`, parties de `ui/mod.rs`) : APP par la feature.
2. **Graphe de dépendances** : références `use crate::…`/`super::…`/`crate::…` résolues au fichier le plus long qui porte le
   chemin de module (596 arêtes fichier → fichier, hors `#[cfg(test)]`). Un `pub use` de `mod.rs` compte sur ce `mod.rs`.
3. **Lectures externes** : `use game::…;` et chemins `game::…` de `bots`, `scenario`, `map_ldtk`, `games/*`, `examples`, `templates`
   (annexe `annexe-lectures-externes.md`, 320 chemins distincts).
4. **Lecture du code par cinq agents** (personnage, présentation, session, gameplay, `map_ldtk`) : une ligne par fichier avec les
   types qui le rendent APP (fichier:ligne), les systèmes `GgrsSchedule`, les types rollback, la recette de coupe. Annexes
   `dette-d69-plan-etape2/annexe-*.md`. **La sonde fait foi pour la classe** ; deux erreurs d'agent relevées et corrigées dans les annexes
   (`SpatialAudioEmitter` « sans récepteur » : le récepteur existe côté caméra mais l'émetteur reste inerte, `instances` n'est jamais
   rempli ; `YELLOW` de `collider/mod.rs` « inutilisé » : utilisé sous la feature `lighting`, donc conservé).

Le script `scripts/check-sim-boundary.sh` reste la preuve de fin : ses crates `PENDING` passent `SIM`/`APP` aux étapes E9 et E11.

## 2. Inventaire de `game`

Classes : **SIM** = logique de simulation, compile en ECS seul une fois les fuites listées fermées ; **MIXTE** = contient les deux
et doit être coupé en deux fichiers (recette dans l'annexe du module et dans la fiche) ; **APP** = présentation, entrées, réseau,
E/S. « SIM à fuite » (16 fichiers, 10 172 lignes, hors retouches : 13 après) = SIM dont une signature lit `Assets<…>`/`GlobalAsset`/`Handle` :
aucune coupe de fichier, seulement le remplacement par une ressource de simulation (E3).

### 2.1 Par module

| module | fichiers | lignes | SIM | MIXTE | APP | fichiers à fuite (sonde) | blocs `GgrsSchedule` | enregistrements rollback | lectures hors `game` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `ui/` | 14 | 7 306 | 2 | 3 | 9 | 7 |  |  | s3 m6 |
| `character/enemy/ai` | 11 | 6 771 | 9 | 1 | 1 | 2 |  |  | b10 s47 m11 |
| `feedback` | 1 | 2 354 | 0 | 1 | 0 | 1 |  |  | s3 |
| `character (racine)` | 8 | 2 312 | 5 | 3 | 0 | 4 | 1 | 32 | b2 s20 m4 e1 |
| `jjrs/` | 7 | 1 783 | 2 | 3 | 2 | 3 |  |  | s2 m2 e3 |
| `effects_runtime` | 1 | 1 430 | 1 | 0 | 0 | 1 | 2 | 5 | s5 |
| `character/player` | 5 | 1 346 | 2 | 2 | 1 | 3 |  |  | b27 s31 m4 e1 |
| `interaction` | 1 | 1 314 | 0 | 1 | 0 | 1 | 1 | 6 | b5 s1 m14 |
| `waves/` | 6 | 1 232 | 3 | 2 | 1 | 4 | 2 | 2 | b1 s15 g6 e2 t1 |
| `powerups` | 1 | 1 032 | 0 | 1 | 0 | 1 | 2 | 2 | b1 s6 |
| `shop` | 1 | 986 | 1 | 0 | 0 | 1 | 3 | 2 | s2 m7 |
| `familiars` | 1 | 984 | 1 | 0 | 0 | 1 | 1 |  |  |
| `effects_v3` | 1 | 919 | 1 | 0 | 0 | 0 | 1 | 2 | s1 |
| `state_trace` | 1 | 912 | 1 | 0 | 0 | 0 |  |  | s4 |
| `replay` | 1 | 896 | 1 | 0 | 0 | 0 |  |  | b8 s125 |
| `room_fog` | 1 | 879 | 0 | 1 | 0 | 1 | 1 |  | m6 |
| `legibility` | 1 | 863 | 0 | 0 | 1 | 1 |  |  | s2 m1 |
| `run_state` | 1 | 805 | 1 | 0 | 0 | 0 | 1 |  | s5 m6 |
| `feedback_v3` | 1 | 774 | 0 | 1 | 0 | 1 |  |  | s1 |
| `world_sprites` | 1 | 764 | 0 | 0 | 1 | 1 |  |  | m1 |
| `camera/` | 2 | 748 | 0 | 0 | 2 | 2 |  |  | s2 |
| `profile` | 1 | 748 | 1 | 0 | 0 | 0 | 1 | 1 | s3 m3 |
| `character/health` | 2 | 744 | 1 | 0 | 1 | 2 |  |  | b3 s32 m5 |
| `items` | 1 | 741 | 0 | 1 | 0 | 1 | 3 | 5 | b1 s5 m3 |
| `character/enemy` | 3 | 637 | 3 | 0 | 0 | 2 |  |  | b3 s20 m5 e1 |
| `bombs` | 1 | 624 | 1 | 0 | 0 | 0 | 1 | 1 | s4 m5 |
| `champions` | 1 | 549 | 1 | 0 | 0 | 1 | 1 | 1 |  |
| `core` | 1 | 545 | 0 | 1 | 0 | 1 | 5 | 5 | b1 s9 m4 g12 e4 t2 |
| `recording` | 1 | 537 | 1 | 0 | 0 | 1 |  |  | b1 s3 |
| `progression` | 1 | 521 | 1 | 0 | 0 | 1 | 1 | 4 | s13 m3 |
| `loot` | 1 | 520 | 1 | 0 | 0 | 1 | 2 | 4 | b3 s7 m21 |
| `curses` | 1 | 404 | 1 | 0 | 0 | 0 | 1 |  | m5 |
| `args/` | 3 | 399 | 0 | 1 | 2 | 1 |  |  | s3 g8 e1 t1 |
| `balance` | 1 | 391 | 0 | 1 | 0 | 1 |  |  | b1 s1 m3 e2 |
| `clock` | 1 | 361 | 1 | 0 | 0 | 0 |  |  | s2 m4 |
| `economy` | 1 | 329 | 0 | 1 | 0 | 1 | 1 | 3 | b1 m2 |
| `remote` | 1 | 325 | 0 | 0 | 1 | 1 |  |  |  |
| `collider/` | 2 | 313 | 0 | 1 | 1 | 2 | 1 | 4 | b9 s22 m27 e2 |
| `global_asset` | 1 | 271 | 0 | 0 | 1 | 1 |  |  | b1 s3 m3 e1 |
| `audio/` | 2 | 270 | 0 | 0 | 2 | 1 |  |  |  |
| `boss` | 1 | 264 | 1 | 0 | 0 | 0 | 1 | 4 | m3 |
| `props` | 1 | 260 | 1 | 0 | 0 | 0 | 1 |  |  |
| `hub` | 1 | 216 | 1 | 0 | 0 | 0 | 1 | 2 | s13 m9 g1 |
| `cave_assets` | 1 | 207 | 0 | 0 | 1 | 1 |  |  | m3 |
| `web/` | 2 | 201 | 0 | 0 | 2 | 1 |  |  | e1 |
| `gen_test` | 1 | 167 | 1 | 0 | 0 | 0 |  |  | s15 |
| `patterns` | 1 | 127 | 1 | 0 | 0 | 0 |  |  |  |
| `weapons` | 1 | 126 | 0 | 1 | 0 | 1 |  |  | b13 s46 m5 e2 |
| `performance` | 1 | 102 | 1 | 0 | 0 | 0 | 2 |  |  |
| `content_hot_reload` | 1 | 87 | 0 | 0 | 1 | 1 |  |  | g5 e1 t1 |
| `logs` | 1 | 86 | 0 | 0 | 1 | 0 |  |  |  |
| `rollback` | 1 | 78 | 1 | 0 | 0 | 0 |  |  |  |
| `frame` | 1 | 76 | 0 | 1 | 0 | 1 |  |  |  |
| `powerup_visuals` | 1 | 75 | 0 | 0 | 1 | 1 |  |  |  |
| `lib` | 1 | 64 | 0 | 1 | 0 | 0 |  |  | s2 m4 g5 e2 t1 |
| `statuses` | 1 | 49 | 1 | 0 | 0 | 0 |  |  |  |
| `light/` | 1 | 22 | 0 | 0 | 1 | 0 |  |  |  |
| `system_set` | 1 | 9 | 1 | 0 | 0 | 0 |  |  | s14 m1 |
| `frame_events` | 1 | 7 | 1 | 0 | 0 | 0 |  |  |  |
| `collision_grid` | 1 | 1 | 1 | 0 | 0 | 0 |  |  | s1 |
| **total** | **114** | **47 863** | **53** | **28** | **33** | **59** | **37** | **85** | b91 s493 m180 g37 e24 t6 |

Colonnes : « blocs `GgrsSchedule` » = appels `add_systems(GgrsSchedule, …)` du fichier (37 dans `game`) ; « enregistrements rollback » = `rollback_and_trace*`, `add_frame_events`, `rollback_component*`
(85 dans `game`, dont 32 dans `character/mod.rs` et 5 dans `core.rs`) ; lectures : `b` `bots`, `s` `scenario`, `m` `map_ldtk`, `g` `games/*`,
`e` `examples`, `t` `templates`.

Trois faits d'inventaire à retenir :

- **Aucun plugin de présentation n'enregistre de type rollback ni d'événement de frame** (relu `build` par `build`, annexe session §1.2) :
  l'ordre d'enregistrement rollback est entièrement dans des plugins de simulation. Les deux seuls systèmes de présentation dans
  `GgrsSchedule` sont `room_fog::capture_vision` (observation, lit l'état, écrit `VisionHistory` non rollback) et les deux
  `performance::begin/end_simulation` (mesure, `ALACOD_PERF_CSV`), tous deux ajoutés **seulement hors headless** : ils n'existent
  pas dans les traces.
- **`CoreSetupPlugin::build` est le squelette de l'ordre** : 56 éléments numérotés dans l'annexe session §1.1, dont 5 enregistrements
  rollback de clôture (`GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`, `RngStreams`) qui passent **après**
  tous les plugins de simulation et doivent le rester.
- **Le headless existant est déjà la preuve de séparabilité de l'ordre** : `!headless` seul ajoute `PresentationPlugin`, donc les traces
  ne dépendent pas de la position de ce bloc.

### 2.2 Par fichier

| fichier | lignes | classe | fuites (sonde, après retouches) | lecteurs hors `game` | enregistre | destination et étape |
|---|---:|---|---|---|---|---|
| `args/cli.rs` | 43 | APP |  |  |  | app — E9 |
| `args/mod.rs` | 278 | MIXTE | 2 (crate d'E/S 2) | s3 g8 e1 t1 |  | sim : `GameArgs`, `GameArgsPlugin`, `*_from_env`, `LaunchSeed`, `DebugAiConfig` — app : `args/mod.rs` : `get_args`, `BaseArgsPlugin`, `TelemetryConfigPlugin` (E6) |
| `args/web.rs` | 78 | APP |  |  |  | app — E9 |
| `audio/adaptive.rs` | 205 | APP |  |  |  | app — E9 |
| `audio/mod.rs` | 65 | APP | 3 (audio 3) |  |  | app — E9 |
| `balance.rs` | 391 | MIXTE | 4 (asset 4) | b1 s1 m3 e2 |  | sim : types résolus + `resolve_*` purs — app : `balance_assets.rs` : `resolve_balance_system` (E3c) |
| `bombs.rs` | 624 | SIM |  | s4 m5 | 1 bloc Ggrs, 1 rollback | sim |
| `boss.rs` | 264 | SIM |  | m3 | 1 bloc Ggrs, 4 rollback | sim |
| `camera/mod.rs` | 657 | APP | 25 (asset 9, sprite/caméra 7, entrée 5) | s1 |  | app — E1 (`CameraFollowOverride`), E9 |
| `camera/ui.rs` | 91 | APP | 16 (ui 10, sprite/caméra 5, asset 1) | s1 |  | app — E9 |
| `cave_assets.rs` | 207 | APP | 1 (asset 1) | m3 |  | app — E9 |
| `champions.rs` | 549 | SIM | 5 (asset 5) |  | 1 bloc Ggrs, 1 rollback | sim — E3a,E3b |
| `character/config.rs` | 245 | MIXTE | 2 (asset 2) | b2 s6 m2 e1 |  | sim : `CharacterConfig` (sans `Asset`), `CharacterConfigKey`, `CharacterConfigs` — app : `character/config_asset.rs` : `CharacterConfigAsset`, chargeur, synchro (E3a) |
| `character/create.rs` | 243 | SIM | 4 (audio 2, asset 1, sprite/caméra 1) |  |  | sim — E3a,E4 (Visibility) |
| `character/dash.rs` | 52 | SIM |  | s3 |  | sim |
| `character/enemy/ai/behavior.rs` | 1025 | SIM |  |  |  | sim |
| `character/enemy/ai/charm.rs` | 96 | SIM |  |  |  | sim |
| `character/enemy/ai/debug.rs` | 332 | APP | 8 (entrée 6, gizmos 2) |  |  | app — E9 |
| `character/enemy/ai/mod.rs` | 32 | MIXTE |  | b2 s23 m4 |  | sim : tout sauf `debug` — app : (`ai/debug.rs` est APP : `character/ai_debug.rs`) |
| `character/enemy/ai/navigation.rs` | 1961 | SIM |  | b8 s10 m7 |  | sim |
| `character/enemy/ai/obstacle.rs` | 280 | SIM |  |  |  | sim — (fait : dérive `Asset`) |
| `character/enemy/ai/pathing.rs` | 1097 | SIM | 1 (asset 1) |  |  | sim — E3a |
| `character/enemy/ai/retreat.rs` | 149 | SIM |  |  |  | sim |
| `character/enemy/ai/rules.rs` | 712 | SIM |  | s5 |  | sim |
| `character/enemy/ai/squad.rs` | 267 | SIM |  |  |  | sim |
| `character/enemy/ai/state.rs` | 820 | SIM |  | s9 |  | sim — (fait : dérive `Asset`) |
| `character/enemy/create.rs` | 288 | SIM | 3 (asset 3) | m2 |  | sim — E3a,E3b |
| `character/enemy/mod.rs` | 4 | SIM |  | b3 s19 m2 |  | sim |
| `character/enemy/spawning.rs` | 345 | SIM | 3 (asset 3) | s1 m1 e1 |  | sim — E3a,E3b |
| `character/health/mod.rs` | 700 | SIM | 2 (asset 2) | b3 s32 m5 |  | sim — E3a |
| `character/health/ui.rs` | 44 | APP | 1 (sprite/caméra 1) |  |  | app — E9 |
| `character/mod.rs` | 330 | MIXTE | 4 (entrée 2, asset 1, crate d'E/S 1) | s1 | 1 bloc Ggrs, 32 rollback | sim : `BaseCharacterSimPlugin`, bloc `GgrsSchedule`, rollback dans l’ordre actuel — app : `character/mod.rs` app : `CharacterAppPlugin` (loader RON, leafwing, ressources debug) + `CharacterPresentationPlugin` (E4) |
| `character/movement.rs` | 305 | SIM |  | s4 |  | sim |
| `character/player/control.rs` | 108 | APP | 47 (entrée 47) |  |  | app — E9 |
| `character/player/create.rs` | 228 | MIXTE | 4 (asset 3, entrée 1) | m1 e1 |  | sim : `create_player` sans `ActionState`/`InputMap`/`PointLight2d` — app : `character/input_devices.rs` : `attach_local_player_input` ; `light/mod.rs` : `attach_player_light` (E4) |
| `character/player/input.rs` | 998 | MIXTE | 6 (fenêtre 2, entrée 2, sprite/caméra 1) | b22 s3 |  | sim : `read_local_inputs`, `BoxInput`, `InputSource`, `ScriptedInputs`, `DeviceInputs` — app : `character/input_devices.rs` : `collect_device_inputs` (leafwing, fenêtre, caméra) (E4) |
| `character/player/jjrs.rs` | 1 | SIM |  | b2 s2 |  | sim |
| `character/player/mod.rs` | 11 | SIM |  | b3 s26 m3 |  | sim |
| `character/surface.rs` | 184 | SIM |  | s1 |  | sim |
| `character/variant.rs` | 466 | SIM |  | s5 m2 |  | sim |
| `character/visuals.rs` | 487 | MIXTE | 19 (asset 9, sprite/caméra 9, crate d'E/S 1) |  |  | sim : `character/appearance.rs` : `CharacterAppearance` (E1) — app : `character/visuals.rs` (reste du fichier) |
| `clock.rs` | 361 | SIM |  | s2 m4 |  | sim |
| `collider/debug.rs` | 176 | APP | 5 (entrée 4, gizmos 1) |  |  | app — E9 |
| `collider/mod.rs` | 137 | MIXTE | 1 (sprite/caméra 1) | b9 s22 m27 e2 | 1 bloc Ggrs, 4 rollback | sim : `BaseColliderSimPlugin`, réexports `combat` — app : `collider/test_wall.rs` : `spawn_test_wall` (E4) |
| `collision_grid.rs` | 1 | SIM |  | s1 |  | sim |
| `content_hot_reload.rs` | 87 | APP | 7 (asset 6, autre 1) | g5 e1 t1 |  | app — E9 |
| `core.rs` | 545 | MIXTE | 11 (fenêtre 5, asset 3, rendu/remote 1) | b1 s9 m4 g12 e4 t2 | 5 blocs Ggrs, 5 rollback | sim : `session.rs` : `AppState`, `OnlineState`, `SIM_FPS`, `CoreSetupConfig`, `is_headless` (E2) ; `core_sim.rs` : `CoreSimPlugin` (E7) — app : `core.rs` : `CoreSetupPlugin`, `get_default_plugin`, `PresentationPlugin`, `GameInfo`, `CoreAppPlugin` (E7) |
| `curses.rs` | 404 | SIM |  | m5 | 1 bloc Ggrs | sim |
| `economy.rs` | 329 | MIXTE | 3 (asset 3) | b1 m2 | 1 bloc Ggrs, 3 rollback | sim : `EconomyConfig`/`PerksConfig` sans `Asset`, `EconomyPlugin` — app : `economy_assets.rs` : enveloppes + `RonAssetPlugin` (E3c) |
| `effects_runtime.rs` | 1430 | SIM | 1 (asset 1) | s5 | 2 blocs Ggrs, 5 rollback | sim — E3b |
| `effects_v3.rs` | 919 | SIM |  | s1 | 1 bloc Ggrs, 2 rollback | sim |
| `familiars.rs` | 984 | SIM | 2 (asset 2) |  | 1 bloc Ggrs | sim — E3a |
| `feedback.rs` | 2354 | MIXTE | 39 (asset 13, ui 11, sprite/caméra 7) | s3 |  | sim : `feedback_log.rs` : `FeedbackLog`, `FeedbackKind`, `record_*`, `pattern_label` (E5b) — app : `feedback.rs` (rendu, sons, chargeur `FeedbackConfig`) |
| `feedback_v3.rs` | 774 | MIXTE | 2 (asset 2) | s1 |  | sim : journal v3 + `VariantTints` (E5b) — app : rendu v3 éventuel (à trancher en E5b) |
| `frame.rs` | 76 | MIXTE | 11 (ui 10, asset 1) |  |  | sim : `frame_counter.rs` : `increase_frame_system` (E1) — app : `frame.rs` : compteur de debug UI |
| `frame_events.rs` | 7 | SIM |  |  |  | sim |
| `gen_test.rs` | 167 | SIM |  | s15 |  | sim |
| `global_asset.rs` | 271 | APP | 17 (asset 16, crate d'E/S 1) | b1 s3 m3 e1 |  | app — E3d, E9 |
| `hub.rs` | 216 | SIM |  | s13 m9 g1 | 1 bloc Ggrs, 2 rollback | sim |
| `interaction.rs` | 1314 | MIXTE | 7 (sprite/caméra 4, asset 2, gizmos 1) | b5 s1 m14 | 1 bloc Ggrs, 6 rollback | sim : lignes 1-1097 (`InteractionPlugin`) — app : `interaction_view.rs` : lignes 1095-1315 (E5c) |
| `items.rs` | 741 | MIXTE | 1 (gizmos 1) | b1 s5 m3 | 3 blocs Ggrs, 5 rollback | sim : lignes 1-631 + `EffectAssets` sur registres — app : `items_view.rs` : `BlankPresentationPlugin` (lignes 632-699) (E5c) |
| `jjrs/allumette.rs` | 473 | APP |  |  |  | app — E9 |
| `jjrs/browser.rs` | 75 | APP |  |  |  | app — E9 |
| `jjrs/local.rs` | 175 | SIM |  |  |  | sim — (fait : imports morts) |
| `jjrs/mod.rs` | 226 | MIXTE | 4 (crate d'E/S 4) | s1 m2 e3 |  | sim : tout sauf `log_ggrs_events` télémétrie — app : `jjrs/telemetry.rs` : `forward_desync_telemetry` (E6) |
| `jjrs/p2p.rs` | 551 | MIXTE | 2 (réseau 2) | s1 |  | sim : `PreviousSessionPeers`, `live_players`, `session_builder`, `abandon_unsynchronized_session`… — app : `jjrs/p2p.rs` app : sockets matchbox, `wait_for_players` (E6) |
| `jjrs/p2p_tests.rs` | 182 | SIM |  |  |  | sim |
| `jjrs/restart.rs` | 101 | MIXTE | 1 (réseau 1) |  |  | sim : tout sauf `drop_matchbox_socket` — app : `jjrs/p2p.rs` app : `drop_matchbox_socket` (E6) |
| `legibility.rs` | 863 | APP | 39 (sprite/caméra 19, asset 9, ui 7) | s2 m1 |  | app — E1 (`WorldLabel`), E9 |
| `lib.rs` | 64 | MIXTE |  |  |  | sim : `lib.rs` de `game_sim` (liste des modules sim) — app : `lib.rs` de `game_app` (E9) |
| `light/mod.rs` | 22 | APP |  |  |  | app — E9 |
| `logs.rs` | 86 | APP |  |  |  | app — E9 |
| `loot.rs` | 520 | SIM | 3 (asset 3) | b3 s7 m21 | 2 blocs Ggrs, 4 rollback | sim — E3b |
| `patterns.rs` | 127 | SIM |  |  |  | sim |
| `performance.rs` | 102 | SIM |  |  | 2 blocs Ggrs | sim |
| `powerup_visuals.rs` | 75 | APP | 10 (ui 4, sprite/caméra 3, asset 2) |  |  | app — E9 |
| `powerups.rs` | 1032 | MIXTE | 5 (asset 5) | b1 s6 | 2 blocs Ggrs, 2 rollback | sim : `PowerUpsConfig` sans `Asset`, 4 systèmes, `PowerUpsTable` — app : `powerups_assets.rs` : `PowerUpsConfigAsset` + loader (E3c) |
| `profile.rs` | 748 | SIM |  | s3 m3 | 1 bloc Ggrs, 1 rollback | sim |
| `progression.rs` | 521 | SIM | 1 (asset 1) | s13 m3 | 1 bloc Ggrs, 4 rollback | sim — E3b |
| `props.rs` | 260 | SIM |  |  | 1 bloc Ggrs | sim |
| `recording.rs` | 537 | SIM | 1 (fenêtre 1) | b1 s3 |  | sim — E6 (`window::ExitSystems`) |
| `remote.rs` | 325 | APP | 4 (rendu/remote 4) |  |  | app — E9 |
| `replay.rs` | 896 | SIM |  | b8 s125 |  | sim |
| `rollback.rs` | 78 | SIM |  |  |  | sim |
| `room_fog.rs` | 879 | MIXTE | 16 (sprite/caméra 11, asset 3, rendu/remote 1) | m6 | 1 bloc Ggrs | sim : `fog_map.rs` : `FogSettings`, `FogMap`, `FogRoom`, `FogDoor` (E1) — app : `room_fog.rs` : `capture_vision`, `FogView`, rendu |
| `run_state.rs` | 805 | SIM |  | s5 m6 | 1 bloc Ggrs | sim |
| `shop.rs` | 986 | SIM | 2 (asset 2) | s2 m7 | 3 blocs Ggrs, 2 rollback | sim — E3b,E3c |
| `state_trace.rs` | 912 | SIM |  | s4 |  | sim |
| `statuses.rs` | 49 | SIM |  |  |  | sim |
| `system_set.rs` | 9 | SIM |  | s14 m1 |  | sim |
| `ui/disconnected.rs` | 153 | APP | 40 (ui 38, autre 2) |  |  | app — E9 |
| `ui/floor_transition.rs` | 176 | APP | 7 (ui 6, autre 1) |  |  | app — E9 |
| `ui/game_over.rs` | 200 | APP |  |  |  | app — E9 |
| `ui/hud.rs` | 2251 | APP | 208 (ui 154, asset 24, sprite/caméra 23) | m3 |  | app — E9 |
| `ui/hud_model.rs` | 1478 | SIM |  | s1 m3 |  | sim |
| `ui/lobby.rs` | 320 | APP | 62 (ui 53, entrée 3, autre 3) |  |  | app — E9 |
| `ui/mod.rs` | 30 | MIXTE |  |  |  | sim : `GameUiModelPlugin` (mutation, hud_model, screens) — app : `GameUiPlugin` : `LobbyUiPlugin`, `DisconnectedUiPlugin` (E5a) |
| `ui/mutation_hui.rs` | 251 | APP |  |  |  | app — E9 |
| `ui/mutation_screen.rs` | 417 | MIXTE | 1 (entrée 1) |  |  | sim : modèle (`MutationScreenView`, `confirm_input`, `MutationNavInput`) — app : `ui/mutation_input.rs` : `read_mutation_nav` (leafwing) (E5a) |
| `ui/screens/hui.rs` | 539 | APP |  |  |  | app — E9 |
| `ui/screens/mod.rs` | 340 | MIXTE | 15 (entrée 15) | s2 |  | sim : `Screens`, `UiStep`, `ScreenScript`, `ScreensPlugin` — app : `ui/screens/input.rs` : `read_nav_keys` (E5a) |
| `ui/screens/model.rs` | 721 | SIM |  |  |  | sim |
| `ui/weapon_hud.rs` | 142 | APP |  |  |  | app — E9 |
| `ui/weapon_visuals.rs` | 288 | APP | 19 (sprite/caméra 9, asset 8, crate d'E/S 2) |  |  | app — E9 |
| `waves/config.rs` | 135 | MIXTE | 1 (asset 1) | s1 |  | sim : `WaveConfig` sans `Asset` — app : `waves/assets.rs` : `WaveConfigAsset` (E3c) |
| `waves/debug.rs` | 106 | APP | 16 (ui 10, sprite/caméra 3, entrée 2) |  |  | app — E9 |
| `waves/mod.rs` | 115 | MIXTE | 1 (asset 1) | b1 s12 g6 e2 t1 | 2 blocs Ggrs, 2 rollback | sim : `WaveSystemPlugin` sans lignes 65 et 68 — app : `waves/assets.rs` : `WaveAssetsPlugin` ; `waves/debug.rs` : `WaveDebugPlugin` (E3c, E5c) |
| `waves/state.rs` | 161 | SIM |  |  |  | sim |
| `waves/systems.rs` | 697 | SIM | 3 (asset 3) | s2 |  | sim — E3a,E3b,E3c |
| `waves/tracking.rs` | 18 | SIM |  |  |  | sim |
| `weapons.rs` | 126 | MIXTE | 13 (asset 7, sprite/caméra 6) | b13 s46 m5 e2 |  | sim : réexport `combat::weapons`, `WeaponRegistry` — app : `weapon_assets.rs` : `WeaponAssetsPlugin`, enveloppes, `weapon_inventory_system`, `hide_new_melee_weapons` (E3b, E5c) |
| `web/logs.rs` | 173 | APP | 3 (entrée 2, crate d'E/S 1) |  |  | app — E9 |
| `web/mod.rs` | 28 | APP |  | e1 |  | app — E9 |
| `world_sprites.rs` | 764 | APP | 37 (sprite/caméra 27, asset 9, crate d'E/S 1) | m1 |  | app — E1 (`SpriteRequest`), E9 |

Sonde complète après retouches : `annexe-sonde.txt`. La colonne « destination » donne la moitié qui reste en `game_sim` et la moitié
qui part en `game_app` pour un fichier MIXTE ; les noms des fichiers nouveaux sont **imposés** par la fiche (chaque coupe crée le
fichier de l'autre moitié à son chemin final, de sorte que E9 se réduise à des `git mv`).

### 2.3 Limites de l'inventaire

- La classe d'un fichier APP parce qu'il fait des E/S ou du réseau (`jjrs/allumette.rs` : `ureq`, Ed25519 ; `args/cli.rs` : `clap` ;
  `logs.rs` : fichiers) est un choix de frontière, pas une fuite détectée par la sonde (ces crates ne sont pas dans `FORBIDDEN`) :
  7 fichiers sont dans ce cas (`args/cli.rs`, `args/web.rs`, `logs.rs`, `jjrs/allumette.rs`, `jjrs/browser.rs`, `web/mod.rs`,
  `audio/adaptive.rs`). Plan §7 : un port console remplace réseau et E/S, jamais la simulation.
- Les chiffres de lignes sont `wc -l` de `crates/game/src` ; les tests inline (`#[cfg(test)]`) y sont compris. Leur effet sur la
  frontière est mesuré par `probe-game-sim.py --tests` (§1).
- Fichiers invisibles de la sonde (compilés sous feature) : 5, listés au §1.

## 3. Inventaire de `map_ldtk`

| fichier | lignes | classe | `bevy_ecs_ldtk` (réf.) | `Sprite`/tuiles (réf.) | blocs `GgrsSchedule` | `Assets`/`GlobalAsset` (réf.) | `game::` (réf.) | remarque |
|---|---:|---|---:|---:|---:|---:|---:|---|
| `game/cave.rs` | 177 | MIXTE | 2 | 1 | 2 |  | 2 | 1 système Ggrs ; `Sprite` en `Update` (E11a) |
| `game/collider.rs` | 434 | MIXTE | 10 | 1 |  | 1 | 7 | `spawn_level_walls(&LdtkProject)` (E11b) |
| `game/entity/character_spawn.rs` | 52 | APP | 2 | 2 |  |  |  |  |
| `game/entity/door.rs` | 103 | MIXTE | 7 | 4 |  |  |  |  |
| `game/entity/enemy_spawn.rs` | 44 | APP | 2 |  |  |  |  |  |
| `game/entity/hub_station.rs` | 39 | MIXTE | 2 | 2 |  |  |  |  |
| `game/entity/mod.rs` | 9 | SIM |  |  |  |  |  |  |
| `game/entity/player_spawn.rs` | 31 | APP | 2 |  |  |  |  |  |
| `game/entity/prop.rs` | 74 | MIXTE | 8 | 4 |  |  |  |  |
| `game/entity/soda_location.rs` | 37 | APP | 2 | 2 |  |  |  |  |
| `game/entity/weapon_location.rs` | 47 | APP | 2 |  |  |  |  |  |
| `game/entity/window.rs` | 39 | APP | 5 | 2 |  |  | 1 |  |
| `game/floors.rs` | 855 | MIXTE | 9 | 5 | 3 | 2 | 23 | `floor_transition_system` (Ggrs) lit `GlobalTransform` et `Assets<LdtkProject>` (E11b) |
| `game/local.rs` | 669 | MIXTE |  | 2 |  | 14 | 24 | 5 `spawn_*_when_map_loaded` ; `Assets<CharacterConfig/Weapons…>` (E3 puis E11b) |
| `game/mod.rs` | 9 | SIM |  |  |  |  |  |  |
| `game/plugin.rs` | 648 | MIXTE | 11 | 1 | 1 |  | 20 | collecte (app) + `spawn_map_items` (sim) + `Sprite` des barreaux (E11a) |
| `game/rooms.rs` | 1528 | MIXTE |  |  | 2 | 9 | 65 | 12 systèmes Ggrs ; `update_hud_room_geometry` à sortir (E11a) ; lu par `bots` (`CurrentRooms`) |
| `game/system/add_level_components.rs` | 89 | MIXTE | 4 |  |  | 1 | 1 | composant de salle posé sur le niveau LDtk |
| `game/system/mod.rs` | 1 | SIM |  |  |  |  |  |  |
| `game/utility.rs` | 46 | APP | 10 |  |  | 1 |  | chargement des niveaux |
| `generation/building.rs` | 911 | APP* | 1 | 1 |  |  | 1 |  |
| `generation/cave.rs` | 265 | APP* | 2 |  |  |  |  |  |
| `generation/from.rs` | 425 | APP* | 6 |  |  |  |  |  |
| `generation/mod.rs` | 724 | APP* |  |  |  |  |  |  |
| `generation/to.rs` | 736 | APP* | 1 |  |  |  | 1 |  |
| `lib.rs` | 10 | MIXTE |  |  |  |  |  | constante `RENDER_ENABLED` (97 gardes dans `scenario/tests`) |
| `loader/file.rs` | 13 | APP | 1 |  |  |  |  | lecture de fichier .ldtk |
| `loader/mod.rs` | 411 | MIXTE | 10 |  |  | 5 | 11 | chargeur d’assets + génération en callback ; tables du registre (≈ 190 l. sim) |
| `map_const.rs` | 57 | SIM |  |  |  |  |  | constantes de noms de couches/champs |
| `mod.rs` | 6 | MORT |  |  |  |  |  | non compilé (aucun `mod mod;`) : supprimer |
| `plugins.rs` | 196 | APP | 1 | 1 |  |  | 1 | `LdtkPlugin`, `register_ldtk_entity` ×12, tuiles |
| `wall_art.rs` | 261 | MIXTE | 4 | 12 |  | 7 | 1 | auto-tuiles des murs (lit `HudConfig`) |

Totaux : MIXTE 13 fichiers (5 298 l.), `generation/*` 5 fichiers (3 061 l., logique pure **liée au modèle JSON de `bevy_ecs_ldtk`**,
donc APP par le graphe de crates), APP 9 fichiers (505 l.), SIM (déclarations de modules) 4 fichiers (76 l.), mort 1 (`src/mod.rs`,
6 l., jamais déclaré). Détails et recettes : `annexe-map_ldtk.md`.

Faits qui décident :

- Le sens est déjà bon : **`game` n'importe rien de `map_ldtk`** (0 `use`, 10 doc-comments), `map_ldtk → game` seulement. Aucun cycle de crate.
- `bevy_ecs_ldtk` (fork `estran-studio`, branche `transform_ldtk_project`) dépend de `bevy_sprite` sans option : **aucun crate SIM ne peut nommer un type
  de ce crate, même le modèle JSON `ldtk::*`**. La génération (`generation/*`, 3 061 lignes) tourne dans le callback du chargeur d'assets :
  c'est du chargement, elle reste côté application. Extraire le modèle JSON dans le fork est un chantier externe pour un gain nul.
- Les systèmes `GgrsSchedule` du crate (12 dans `rooms.rs`, 3 chaînés dans `floors.rs`, 1 dans `cave.rs`) lisent des données `bevy_ecs_ldtk`
  **immuables après le chargement** (`GlobalTransform` des entités de niveau, `Assets<LdtkProject>`, IntGrid) : un instantané pur pris à
  `LdtkMapLoadingEvent` redonne les mêmes valeurs, et `LdtkMapEntityLoadingRegistry` fait déjà ce travail pour portes, fenêtres, spawners.
- Aucun type rollback enregistré dans `map_ldtk` : pas de risque de parité des types vides, seulement l'ordre des systèmes.
- `scenario` joue les cartes par `LdtkRoguePlugin` en headless : il reste APP. `bots` n'exige de `map_ldtk` que `CurrentRooms`, `door_near`,
  `key_lock_cost`, `sorted_rooms`.
- La coupe `map_ldtk_sim` est bloquée par `game`, pas par `bevy_ecs_ldtk` : `spawn_enemy` et `create_player` prennent encore
  `Assets<WeaponsConfig/MeleeWeaponsConfig/CharacterConfig>` (`rooms.rs:1282-1349`, `local.rs:40-143`). D'où l'ordre : **E3 avant E11**.

Décision : deux temps (E9/E10 classent `map_ldtk` entier en `APP` à coût logique nul : 176 chemins `game::` à réécrire ; E11a-c extraient
`map_ldtk_sim`, ≈ 3 700 lignes déplacées, ≈ 600 réécrites, trace identique attendue).

## 4. Graphe de dépendances et cycles

Le graphe fichier → fichier de `game` contient **un seul bloc fortement connexe de 69 fichiers** (et un cycle de 2 :
`navigation.rs` ↔ `state.rs`) : `core.rs` importe presque tout, `character/mod.rs` enregistre presque tout, `weapons.rs`, `global_asset.rs`
et `character/config.rs` sont lus par presque tout. Un tel bloc ne se coupe pas par fichier ; il se coupe par **rôle** (donnée, enveloppe
d'asset, plugin, présentation). Les cycles **internes** à la simulation (`effects_runtime` ↔ `effects_v3` ↔ `items`, `loot` ↔ `shop`,
`economy` ↔ `balance`, `character/health` ↔ `interaction`/`economy`/`effects_v3`/`progression`/`clock`, `run_state` ↔ `hub` ↔ `jjrs`) restent dans
`game_sim`, sans conséquence : ils interdisent seulement de scinder la simulation en plusieurs crates. Seul compte le sens **app → sim**.

Dépendances **remontantes sim → app** (ce qui empêche la séparation), avec le geste qui la casse :

| # | de (sim) | vers (app) | casse par | étape |
|---|---|---|---|---|
| R1 | 13 fichiers sim : `champions.rs:58`, `character/{create,enemy/create,enemy/spawning,player/create}.rs`, `effects_runtime.rs:55,132`, `familiars.rs`, `loot.rs`, `progression.rs:430`, `shop.rs:261`, `waves/systems.rs`, `balance.rs:300`, `powerups.rs:171` | `global_asset::GlobalAsset` (`Handle<…>` ×10, `AssetServer`) ; `interaction.rs:682` corrigé (§10) | ressource de simulation `ContentData`/registres remplie par l'app à la sortie de `AppState::Loading` | E3d |
| R2 | ≈ 15 fichiers sim + `bots/src/input.rs:246` + `scenario/runner.rs:674,1195` | `weapons::{WeaponsConfig, MeleeWeaponsConfig}` (enveloppes `Asset`, `Assets<…>`) | `WeaponRegistry` (tables `BTreeMap` sans `Asset`) publié par l'app, rechargement à chaud inclus | E3b |
| R3 | `character/health/mod.rs:166,377`, `ai/pathing.rs:213`, `player/input.rs:432`, `interaction.rs:628,816`, `familiars.rs`, `champions.rs` | `character/config.rs` : `#[derive(Asset)]` + `CharacterConfigHandles { Handle }` | `CharacterConfigs` + `CharacterConfigKey(String)` (remplace le `Handle` porté par l'entité) | E3a |
| R4 | 32 fichiers (`clock.rs`, `hub.rs`, `run_state.rs`, `state_trace.rs`, `jjrs/*`, `ui/hud_model.rs`, `performance.rs`…) | `core.rs` : `AppState`, `OnlineState`, `SIM_FPS`, `is_headless`, `CoreSetupConfig` (fichier qui importe `ui`, `camera`, `audio`, `light`) | module de simulation `session` ; `core.rs` devient app | E2, E7 |
| R5 | `replay.rs` (23 refs), `recording.rs`, `interaction.rs:98,1056` | `character/player/input.rs` : leafwing, `PrimaryWindow`, `Camera`, `ui::mutation_screen` (`:232,337`) | `read_local_inputs` + `DeviceInputs` en sim, `collect_device_inputs` en app ; `InteractionInput` en `interaction.rs` | E1, E4 |
| R6 | `replay.rs:148` (`Scenario::ui: Vec<UiStep>`), `scenario/runner.rs:437` | `ui/screens/mod.rs` (`UiStep`, `ScreenScript`, importe `ButtonInput`, `Gamepad`) | `UiStep`, `UiAction`, `ScreenId`, `ScreenScript` dans `ui/screens/model.rs` | E1 |
| R7 | `jjrs/mod.rs:166-167,211-227`, `args/mod.rs:221,248` | crate `telemetry` (APP : bevy complet, `ureq`) ; `jjrs/p2p.rs`, `jjrs/restart.rs` : `MatchboxSocket` | message sim `DesyncObserved` + système app ; `TelemetryConfigPlugin` app ; séparation `wait_for_players`/`system_after_map_loaded` en deux fonctions | E6 |
| R8 | `curses.rs:222`, `profile.rs:449`, `core.rs:377` | `frame.rs` (UI de debug : `AssetServer`, `Text`, `Node`) | `increase_frame_system` en `frame_counter.rs` | E1 |
| R9 | `ui/hud_model.rs:28,79`, `ui/mutation_screen.rs:28,193` | `camera/mod.rs:26` : `CameraFollowOverride` | type déplacé dans `ui/hud_model.rs` | E1 |
| R10 | `interaction.rs:1105` (`world_sprites::DoorSprite`), `items.rs:663` (`feedback::FeedbackConfigLoaded`), `interaction.rs:1109-1162` (`Visibility`, `Sprite`, `Gizmos`) | `world_sprites.rs`, `feedback.rs`, caméra | coupure des lignes de présentation vers `interaction_view.rs`, `items_view.rs` | E5c |
| R11 | `run_state.rs:482-488` (wasm) | `jjrs/browser.rs` | fonction de crochet `cfg(wasm32)` côté app, appelée par un message sim | E6 |
| R12 | `recording.rs:297` | `bevy::window::ExitSystems` | ordonner sur le set de simulation `ExitRequests` déjà présent ; la contrainte `ExitSystems` côté app | E6 |
| R13 | `scenario/runner.rs:306,437,3556-3640` | `get_default_plugin()` (DefaultPlugins, `WindowPlugin`, `RenderPlugin`), `Window`, `RenderTarget`, `LabelsEnabled` | `scenario` reste APP ; chemin de capture isolé derrière la feature `render` | E9, E10 |
| R14 | `map_ldtk/rooms.rs:147,221-233`, `loader/mod.rs:169` | `ui::hud_model::{HudModelSet, HudRoomGeometry}` ; `room_fog::FogMap` ; `cave_assets` (chemins) | `update_hud_room_geometry` en plugin app ; `Fog*` en `fog_map.rs` ; helpers de chemins séparés | E1, E11a |
| R15 | `balance.rs:29,32`, `clock.rs:27,154` | `core::OnlineState`, `jjrs::GggrsSessionConfiguration` ; `resolve_balance_system` lit les enveloppes | `SessionPlayers(u32)` écrit par l'app ; `resolve_balance_system` en app | E3c |
| R16 | `waves/mod.rs:39,65,68`, `economy.rs:41`, `powerups.rs:50` | `bevy_common_assets`, `WaveDebugPlugin` | `…AssetsPlugin` app avant le plugin sim | E3c, E5c |
| R17 | `character/mod.rs:14,17,85,87,106-118` | `RonAssetPlugin`, `InputManagerPlugin`, `ai::debug` (gizmos) | `BaseCharacterSimPlugin` / `CharacterAppPlugin` | E4 |
| R18 | `core.rs ↔ ui/camera/audio/light` | `CoreSetupPlugin` mêle les deux | `CoreSimPlugin` / `CoreAppPlugin`, `CoreSetupPlugin` en app | E7 |

**Fermeture.** Parmi les 53 fichiers classés SIM, seuls **16** se ferment aujourd'hui sur eux-mêmes (liste ci-dessus) : les 37 autres
importent un fichier MIXTE (`collider/mod.rs` 11 lecteurs, `weapons.rs` 15, `character/config.rs`, `core.rs`, `interaction.rs`,
`items.rs`, `balance.rs`, `jjrs/mod.rs`…) ou APP (`global_asset.rs`). C'est pourquoi l'ordre des étapes suit les **hubs** (config
des personnages, armes, `core`) et non les modules.

## 5. Cible

### 5.1 Crates et dépendances

```
avant                                      après (E10, E11)
 games/*, examples, templates                games/*, examples, templates ──► game_app, map_ldtk (+ game_sim)
 scenario ──► game, map_ldtk, bots           scenario (APP) ──► game_sim, game_app, map_ldtk, bots
 bots ──► game, map_ldtk                     bots ──► game_sim            (SIM après E11 : map_ldtk_sim)
 map_ldtk ──► game                           map_ldtk (APP) ──► game_sim, game_app       (E11 : + map_ldtk_sim ◄ game_sim)
 game ──► sim crates, bevy complet,          game_app (APP) ──► game_sim, animation, telemetry, bevy complet, kira,
          kira, matchbox, leafwing, …                          matchbox, leafwing, common_assets, light2d, hui, egui, clap, ureq, …
                                             game_sim (SIM) ──► sim crates, bevy ECS (+ bevy_state, bevy_color, serialize),
                                                              bevy_ggrs, ggrs, matchbox_protocol, serde, ron… (liste exacte : le `cargo check` de E9)
```

`game_sim` garde l'arborescence de modules de `game` : un fichier de simulation ne change **pas** de chemin de module (zéro modification de contenu
à E9, seulement `git mv`). `game_app` a ses propres modules (`core`, `ui`, `camera`, `character`, `collider`, `jjrs`, `args`…, qui existent
aussi côté sim : deux crates, aucun conflit). Features : `ui_hui`, `lighting`, `debug_ui`, `harmonium`, `dev-dylib` passent à `game_app`
(`Cargo.toml` racine, `games/*`, `scenario`, `templates/starter`, `scripts/dev-features.sh`, `scripts/scenario-video`).

### 5.2 Ce qui doit changer ailleurs que dans `game` (surfaces de cutover relevées par `grep`)

`Cargo.toml` racine (`game = { path }`, features `render`, `lighting`, `native` : lignes 12-18, 69), `crates/{bots,map_ldtk,scenario}/Cargo.toml`,
`games/*/Cargo.toml` (5), `templates/starter/Cargo.toml` (`{{engine:game}}` → deux dépendances ; `alacod new` résout `{{engine:<crate>}}` par nom de crate,
`crates/content/src/bin/alacod.rs:404`), `scripts/check-sim-boundary.sh` (listes), `scripts/check-forbidden.py` (`CRATES`), `scripts/determinism-allowlist.json`
(**35 entrées sur 8 fichiers** de `crates/game/src/` : `balance`, `character/config`, `character/player/input`, `global_asset`, `performance`, `ui/weapon_visuals`, `waves/config`, `weapons`, à suivre les fichiers qui se coupent), `scripts/dev-features.sh`, `scripts/scenario-video:49`, `Makefile` (commentaire
`crates/game/build.rs`, `APP_VERSION`), `CLAUDE.md` et `docs/conventions.md` (193 mentions `crates/game/…` dans `docs/` et `CLAUDE.md`, réparties sur 70 fichiers de
`docs/taches/` qui sont des rapports historiques : **ne pas les réécrire**, mettre à jour seulement `CLAUDE.md`, `conventions.md`, `etat-engine.md`, `plan-engine.md`).
`crates/game/build.rs` pose `APP_VERSION` (`GameInfo`, chemin d'assets wasm) : il suit `game_app` ; `game_sim` n'en a pas besoin.

## 6. Plan en étapes

Détail exécutable (fichiers, gestes, preuves, conflits attendus) : **`docs/taches/dette-d69-etape2.md`**, une section par livraison. Résumé :

| étape | contenu | fichiers touchés (≈) | estimation | risque | dépend de |
|---|---|---:|---|---|---|
| E0 | outils (sonde, ordre des systèmes, `make`) et 5 retouches : **fait dans cette tâche** | 9 | fait | nul | — |
| E1 | types de donnée sortis de leurs fichiers de présentation : `CameraFollowOverride`, `WorldLabel`, `SpriteRequest`, `UiStep`/`UiAction`/`ScreenId`/`ScreenScript`, `Fog*`, `CharacterAppearance`, `InteractionInput`, `increase_frame_system`, `WaveDebugEnabled` | 25 | 1 j | bas : `CharacterAppearance` est rollback (mêmes derives, même `Hash` manuel) | — |
| E2 | `AppState`, `OnlineState`, `SIM_FPS`, `CoreSetupConfig`, `is_headless` hors de `core.rs` (module `session`) | 45 | 0,5 j | bas (renommage de chemin, 32 fichiers de `game` + 31 lecteurs externes) | — |
| E3a | `CharacterConfigs` + `CharacterConfigKey` à la place de `Handle<CharacterConfig>` ; enveloppe `CharacterConfigAsset` | 20 | 1,5 j | **moyen** : moment du remplissage (avant `RunGgrsSystems`) | E1 |
| E3b | `WeaponRegistry` à la place de `Assets<WeaponsConfig/MeleeWeaponsConfig>` ; rechargement à chaud ; surcharges de scénario | 22 + `bots` + `scenario` | 2 j | **moyen** : rechargement à chaud et `apply_weapon_overrides` | E3a |
| E3c | `PowerUpsTable`, `ResolvedBalance` étendu (`shells_drop`, `shop_prices`, `waves_enabled`, `SessionPlayers`), enveloppes `Economy/Perks/PowerUps/WaveConfigAsset`, `resolve_balance_system` en app | 14 | 2 j | moyen : surcharges de `scenario` avant `OnEnter(GameLoading)` | E3b |
| E3d | `GlobalAsset` réduit aux `Handle` de sprites/animations ; plus aucun lecteur sim ; `ContentData` | 18 | 1 j | bas | E3c |
| E4 | personnage : `read_local_inputs`/`collect_device_inputs`, `create_player` sans `ActionState`/`PointLight2d`, `BaseCharacterSimPlugin`/`CharacterAppPlugin`, `spawn_test_wall` en app, `Visibility` des spawns | 15 | 2 j | moyen : une frame d'`ActionState` tardive hors headless | E1, E3a |
| E5a | écrans, HUD, mutation : modèles en sim, lecture clavier/manette en app ; `GameUiPlugin` éclaté **sans** changer ce qui tourne en headless p2p | 10 | 1,5 j | moyen : `LobbyUiPlugin` tourne aujourd'hui hors `PresentationPlugin` | E1, E2 |
| E5b | `feedback`/`feedback_v3` : journal en sim, rendu et chargeur en app, `VariantTints` | 6 | 1,5 j | bas | E3a |
| E5c | `interaction_view`, `items_view`, `WaveDebugPlugin`, `weapon_assets`, `room_fog` | 12 | 2 j | bas (`Update` seul) | E3b, E3c |
| E6 | `jjrs/p2p.rs` (sockets), `jjrs/mod.rs` (télémétrie), `args`, `restart`, `recording`, `run_state` wasm, `frame.rs` | 12 | 2 j | **moyen** : p2p réel obligatoire | E2 |
| E7 | `CoreSetupPlugin` en `CoreSimPlugin` + `CoreAppPlugin` | 3 | 1,5 j | **élevé** : ordre des enregistrements | E3d, E4, E5, E6 |
| E8 | porte de sortie de la phase A : liste `scripts/d69-sim-files.txt` complète, sonde `--tests`, fermeture, `cargo check` ECS seul du ratchet | 2 | 0,5 j | bas | E7 |
| E9 | **`game_app` créé, les fichiers APP déplacés (`git mv`), `game` devient SIM** (`check-sim-boundary` : `game` passe en `SIM`) | ≈ 60 déplacés (contenu : `crate::x` → `game::x`) + ≈ 40 lecteurs | 2 j, **fenêtre de gel** | moyen : conflits de fusion | E8 |
| E10 | `game` → `game_sim` (renommage mécanique), `bots`/`scenario`/`map_ldtk`/`games` | ≈ 90 | 0,5 j | bas | E9 |
| E11a-c | `map_ldtk_sim` ; `bots` passe `SIM` ; `map_ldtk` : `PENDING` → `APP` dès E9, `map_ldtk_sim` → `SIM` ici | ≈ 25 | 1 + 2 + 2,5 j | moyen (E11b : instantané des `GlobalTransform`) | E3, E10 |

**Voies parallèles.** L1 : E1 → E3a → E3b → E3c → E3d. L2 : E2 → E6. L3 (après E1/E3a) : E4, E5a, E5b ; E5c après E3b/E3c. Jonction : E7 ← L1, L2, L3. Chemin critique :
E1 (1) → E3a (1,5) → E3b (2) → E3c (2) → E3d (1) → E7 (1,5) → E8 (0,5) → E9 (2) → E10 (0,5) = **12 jours**, E11 (5,5) en parallèle de ce qui suit.
Total ≈ 28 jours-agent. [INFERENCE] : aucune durée de référence mesurée pour ce type de coupe ; estimation d'après le nombre de fichiers et de sites à réécrire ; ordre de grandeur seulement. Un jour = un agent, preuves de la fiche comprises
(`SHARDS=3` : 4 à 7 minutes sur ce poste, `test_crates`, ordre des systèmes ≈ 2 × 4 minutes de build, p2p ≈ 10 minutes).

**Conflits attendus avec les branches actives** (b0 `crates/combat` projectiles, b1 données isaac et scénarios, b2 `jjrs/p2p.rs`, b6 `tests/budgets.ron`) :
b2 a **déjà fusionné** (`8633a560`) : E6 peut partir. b6 : aucun fichier commun. b0 : `WeaponAsset` vit dans `combat` ; E3b ne change que les lecteurs
(`game`, `bots`, `scenario`), pas les types, donc pas de conflit tant que b0 n'ajoute pas de champ lu par `Assets<WeaponsConfig>`. b1 : ajoute scénarios et
données ; ses fichiers de `game` éventuels (`items.rs`, `familiars.rs`, `loot.rs`, `shop.rs`) sont aussi des sites d'E3 : **fusionner b1 avant E3b** ou rebaser
(les changements d'E3 sont des signatures de systèmes, courtes). **E9 est la seule étape qui demande un gel** de `crates/game/` : annoncer, fusionner tout
ce qui touche `crates/game/src`, exécuter E9 le même jour, les autres branches rebasent avec la détection de renommage de git (les fichiers de simulation
ne changent **pas** de contenu, ils changent de dossier ; seuls ~70 fichiers app voient `crate::x` devenir `game_sim::x`).

## 7. Preuves, étape par étape

Porte commune de chaque livraison de la phase A (aucune ne change une trace) :

1. `make check_sim_boundary` : inchangé jusqu'à E9 (`game` reste `PENDING`, affiché) ;
2. **`make schedule_order_diff BASE=<commit de base>`** : sortie 0 = ordre des systèmes (`ReadInputs`, `GgrsSchedule`, `AdvanceWorld`,
   `SaveWorld`, `LoadWorld`) et ordre d'enregistrement des types rollback **identiques octet pour octet** pour les cinq jeux ;
3. `make test_scenarios_par SHARDS=3` **sans BLESS** : aucun `.trace` réécrit (`git status` propre) ;
4. `make test_crates` ; `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` (+ `--tests` en E8) : la liste ne se réduit jamais,
   l'étape y ajoute les fichiers qu'elle libère ;
5. `cargo fmt --all -- --check`, `scripts/check-forbidden.sh`, `scripts/check-rollback-registration.sh`, `make test_tools`, `make lint` ×5, `make gen GAME=…` ×5
   (aucune trace générée différente) ;
6. fuzz `ALACOD_FUZZ=0:8` ; `ALACOD_FUZZ_BOMBS=1` pour E3, E4 ;
7. selon l'étape : build web `APP_VERSION=… cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` (E1-E2, E6),
   `cargo check -p game --features lighting,ui_hui,debug_ui` (E4, E5), `scripts/p2p-stress.sh` zombies **et** isaac (E3, E6, E7), captures
   `play_scenario --capture` avant/après comparées par `cmp` comme à l'étape 1 (E4, E5, E7 : `tears_shot_speed`, `two_players_shooting`, `gungeon_boss`,
   un scénario `isaac_hud_items`, un scénario à brouillard).

### 7.1 L'outil d'ordre : ce qu'il prouve et sa sensibilité

`crates/scenario/tests/schedule_order.rs` (test `#[ignore]`, outil) construit l'app headless d'un scénario par jeu (zombies, gungeon, isaac, throne,
testbed) **sans jouer de frame**, initialise les schedules du rollback et écrit, pour chacun, l'ordre exécutable des systèmes (celui de
l'exécuteur monothread, `ApplyDeferred` compris) et `utils::rollback::TracedTypes` (ordre d'enregistrement). Noms normalisés (`a::b::f<c::D>` → `f<D>`) : déplacer
un système d'un crate à l'autre ne change rien à la sortie ; ajouter, retirer, réordonner un système ou un type, oui. `scripts/schedule-order-diff.sh [BASE]` construit la
base dans un worktree jetable, copie le test s'il manque, compare. Mesures du 2026-10-18 :

- baseline : 899 systèmes (`ReadInputs` 6, `GgrsSchedule` 212, `AdvanceWorld` 10, `SaveWorld` 532, `LoadWorld` 139) et 129 types rollback par jeu ; **les cinq jeux sont identiques** entre eux ;
- déterminisme de l'outil : deux exécutions successives, sorties identiques ;
- **sensibilité** : permuter `PropsPlugin` et `BombsPlugin` dans `core.rs` (changement d'ordre sans aucun effet de jeu) est détecté (`clear_frame_events<…>` et sept systèmes décalés, diff non vide) ; `core.rs` restauré.

Conséquence pour E7 : l'outil exige l'ordre **exact** des plugins de simulation ; les plugins app ne doivent pas s'intercaler dans `GgrsSchedule` (aucun ne le fait hors
`room_fog` et `performance`, absents du headless).

**Limite : pas de mode présentation.** L'outil ne prouve que les schedules du rollback (ce qui compte pour les traces). L'ordre de `PreUpdate`/`Update`/`PostUpdate`
de la présentation n'est pas couvert : essai joué (`build_app(…, headless: false)` depuis le test) = panique `on macOS, EventLoop must be created on the main thread` (winit,
le harnais de test n'est pas le thread principal) ; l'option abandonnée n'est pas livrée. Pour E7 il reste deux preuves : le raisonnement (la présentation n'écrit aucun
état de simulation, rapport §2.1) et les captures `cmp` ; si l'on veut un dump, le faire dans un binaire `harness = false` (thread principal) ou avec `WinitPlugin` désactivé.

### 7.2 Ordre d'enregistrement rollback et parité des types vides

Règle de la fiche : **aucune** étape ne crée, ne retire, ne renomme ni ne `neutral`-ise un type rollback. Les types qui changent de fichier (`CharacterAppearance`,
E1) gardent leurs derives et leur `Hash` manuel et leur enregistrement reste à la même place dans `BaseCharacterSimPlugin`. L'ordre est celui de `core.rs` (annexe session §1.1) : bloc
simulation (items 6 → 33), enregistrements de clôture (38) **dernier**. E7 scinde `build` en gardant la suite exacte des lignes SIM ; la sortie `*.rollback.txt` de l'outil est la
preuve, les `.trace` (checksum XOR + dump `ALACOD_STATE_TRACE_FULL` pour la parité) la confirment. Risque nommé : un type rollback d'un plugin déplacé en app (aucun aujourd'hui, vérifié).

## 8. Migration des lectures `bots` / `scenario` / `map_ldtk` / `games`

Détail chemin par chemin : `annexe-lectures-externes.md`. Chiffres : `bots` 91 occurrences (89 vers la simulation, 2 à **migrer** : `global_asset::GlobalAsset`,
`weapons::WeaponsConfig`), `scenario` 491 (478 sim, 13 app : `args`, `core::CoreSetupPlugin`, `camera::ui`, `legibility`), `map_ldtk` 176, `games` 32, `examples` 22, `templates` 5.

Règle : **aucun réexport de compatibilité**. Chaque lecteur est réécrit vers le chemin final dans la livraison qui le fait disparaître :

| livraison | lecteurs réécrits | nature |
|---|---|---|
| E2 | `core::{AppState, OnlineState, SIM_FPS, CoreSetupConfig}` : scenario 15, map_ldtk 6, bots 1, games 12, examples 4, templates 2 | `core` → `session` |
| E3a | `CharacterConfigHandles` (bots 1, scenario 1), `Assets<CharacterConfig>` (`bots/src/input.rs`, `scenario/tests/expectations.rs:1767`, `extra_health.rs:255`, `map_ldtk/local.rs`) | vers `CharacterConfigs` |
| E3b | `Assets<WeaponsConfig/MeleeWeaponsConfig>` (`bots/src/input.rs:246`, `scenario/runner.rs:674,1195,728-800`) | vers `WeaponRegistry` ; surcharges de scénario écrites **sur le registre** |
| E3c/E3d | `GlobalAsset` (bots 1, scenario 3, map_ldtk 3, examples 1), `Assets<EconomyConfig>` (`map_ldtk/rooms.rs:451`) | vers `ResolvedBalance`/`ContentData` |
| E4 | `character::skip_device_input` (`scenario/runner.rs:531`) | supprimé (plus de leafwing en headless) |
| E9 | app : `core::CoreSetupPlugin` et `args::BaseArgsPlugin` (games ×5, examples, templates), `camera::ui`, `legibility` (scenario), `content_hot_reload::GameRoot` (games ×5) | `game::` → `game_app::` |
| E10 | tout le reste : ≈ 560 occurrences, **sed mécanique** `game::` → `game_sim::` (les `use game::{…}` mixtes se scindent) | renommage |

`bots` : `Cargo.toml` `game` → `game_sim` (E10) ; classé `SIM` dans `check-sim-boundary.sh` à E11 (reste `map_ldtk`). `scenario` : `game_sim` + `game_app` ; sa feature `render`
(`map_ldtk/render`, `game_app/ui_hui`) est inchangée ; `scenario/src/runner.rs::capture` (fenêtre, `RenderTarget`, `IsDefaultUiCamera`) reste dans le crate, derrière la feature.
`map_ldtk` : `game_sim` + `game_app` (ses 176 lectures se répartissent : 160 sim, 16 app).

## 9. Risques et décisions à prendre

| # | risque ou décision | mesure ou position |
|---|---|---|
| D1 | **`scenario` ne peut pas être « simulation seule »** : `get_default_plugin()` (DefaultPlugins) même en headless, `AssetPlugin` et `Assets<…>` lus par les systèmes de spawn, `LdtkRoguePlugin`. | Position : `scenario` reste `APP` (déjà classé ainsi), dépend de `game_sim` + `game_app` sans rendu. Aller plus loin = chargeur de contenu `std::fs` pour la simulation (une sortie de `Loading` propre au runner, un remplissage direct des registres depuis `content::Registry`) **et** une génération de niveaux sans `bevy_ecs_ldtk` ; ≈ 5 à 8 j de plus et un risque trace réel (frame de démarrage). Non proposé ; à trancher par William. |
| D2 | Moment du remplissage des registres de simulation (E3) : la simulation lit aujourd'hui l'asset **à chaque frame** pour `powerup_pickup_detect_system`, `loot_drop_on_death_system`, `shell_drop_on_death_system`, `weapon_drop_on_death_system` ; les autres lectures sont sur événement. | Un système app `PreUpdate.before(RunGgrsSystems)` publie le registre quand l'asset change (`is_changed`/`AssetEvent`) ; `OnEnter(GameLoading)` en plus pour `ResolvedBalance` (précédent `resolve_balance_system`). Preuve : traces identiques ; test dédié « le premier tick voit le registre » (E3a). |
| D3 | Rechargement à chaud et surcharges de scénario (`apply_weapon_overrides`, `apply_wave_overrides`, `apply_powerup_drop_chance_override`, `runner.rs:671-800`) mutent aujourd'hui les `Assets`. | Les surcharges se posent **sur le registre** (donnée de simulation), après la synchro ; le rechargement à chaud régénère le registre sur `AssetEvent` (dev seulement). |
| D4 | `ActionState`/`InputMap` attachés par un système `Added<LocalPlayer>` (E4) : une frame de plus avant le premier input clavier d'un joueur local, hors headless. | `PreUpdate.before(RunGgrsSystems)` ; capture + essai manuel ; aucun scénario ni bot ne passe par là (`ScriptedInputs`). |
| D5 | `GameUiPlugin` : `LobbyUiPlugin` et `DisconnectedUiPlugin` tournent aujourd'hui **aussi en headless p2p** (hors `PresentationPlugin`). | Les rattacher à `CoreAppPlugin`, pas à `PresentationPlugin` (E5a, E7) ; `p2p-stress` le prouve. |
| D6 | `CoreAppPlugin` ajouté **après** le bloc simulation (aujourd'hui `PresentationPlugin` est avant) : l'ordre d'insertion des systèmes `Update`/`PostUpdate` de présentation et leurs ambiguïtés, que l'outil d'ordre ne couvre pas (§7.1). | Les systèmes de présentation s'ordonnent entre eux par `.before/.after` explicites (annexe présentation §3) ; captures `cmp` avant/après ; éventuellement un binaire `harness = false` qui dumpe `Update` (thread principal). |
| D7 | Tests inline qui construisent `Assets<…>`/`GlobalAsset`/`Sprite` dans des fichiers sim (`effects_runtime.rs:1170,1217,1295`, `effects_v3.rs:557`, `powerups.rs:603-620,851,921`, `player/input.rs:860-894`) ou nomment `bevy_matchbox` (`jjrs/p2p_tests.rs:151`) : `cargo test -p game_sim` ne compilerait plus (mesuré par `probe-game-sim.py --tests`). | Réécrits avec les registres dans E3/E4 ; `probe-game-sim.py --tests` en E8. |
| D8 | `build.rs` de `game` (`APP_VERSION`), `GameInfo`, chemin d'assets wasm. | Suivent `game_app` (E7/E9) ; `game_sim` sans `build.rs`. |
| D9 | E9 est une fenêtre de gel de `crates/game/`. | §6 : fusionner d'abord ; E9 + E10 le même jour ; rebase par détection de renommage. |
| D10 | Les estimations (28 j, 12 j critique) sont des ordres de grandeur. | Revoir après E1-E3a (premières mesures réelles). |

Questions pour William (non bloquantes pour E1-E8) : D1 (périmètre de « scenario simulation seule ») ; faut-il garder le crate `telemetry` en `APP` (oui par défaut) ; nom final
`game_app` confirmé ?

## 10. Retouches faites dans cette tâche (aucun fichier déplacé, aucune trace ne bouge)

| fichier | retouche | fuite fermée |
|---|---|---|
| `crates/game/src/jjrs/local.rs`, `jjrs/mod.rs` | 18 imports morts retirés (signalés par `unused_imports`) | `animation::SpriteSheetConfig` et `bevy_matchbox` (deux crates de présentation/réseau) ; `GlobalAsset`, `WeaponsConfig`, `create_player`… ; `jjrs/local.rs` ne fuit plus |
| `character/enemy/ai/obstacle.rs:195`, `state.rs:351` | `Asset, TypePath` retirés de `ObstacleConfig` et `EnemyAiConfigRon` (aucun chargeur, aucun lecteur) | deux `derive Asset` : les deux fichiers ne fuient plus |
| `character/movement.rs`, `player/input.rs`, `ai/pathing.rs` | `FIXED_TIMESTEP` déplacé dans `movement.rs` (feuille de simulation) | dépendance de `ai/pathing.rs` vers le fichier MIXTE `player/input.rs` (R5) |
| `interaction.rs:682` + import | paramètre `Res<GlobalAsset>` jamais lu retiré de `handle_weapon_pickup_interaction` | un lecteur de `GlobalAsset` (R1) |
| `scripts/`, `Makefile`, `crates/scenario/tests/schedule_order.rs` | outils : `probe-game-sim.py`, `d69-sim-files.txt`, `schedule-order-diff.sh`, test d'ordre ; cibles `probe_game_sim`, `schedule_order_diff` | — |

Mesure de la sonde : **62 → 59 fichiers à fuite, 789 → 781 erreurs** (`jjrs/local.rs`, `obstacle.rs`, `state.rs` sortent de la liste). Écartées volontairement : `SpatialAudioEmitter`
(suppression d'un composant, effet inerte mais pas « trivial » : E4), `Visibility` des spawns (E4), `YELLOW` de `collider/mod.rs` (sous `lighting`), `recording.rs:297` (ordre de `Last`, E6).

## 11. Vérifications

| Contrôle | Résultat |
|---|---|
| `make check_sim_boundary` | OK : 13 crates SIM (graphe seul et `cargo check --no-default-features` ensemble), 2 en dette D69 (`map_ldtk`, `game`), inchangé |
| `make schedule_order_diff BASE=HEAD` (base `8633a560`, arbre de travail avec les retouches) | **OK : ordre des systèmes et des types rollback identiques** (4 520 lignes de schedules, 645 types, cinq jeux) ; rejoué après la dernière modification de l'outil : identique aussi à la baseline prise sur `09b68187` avant tout changement |
| `make test_scenarios_par SHARDS=3` (sans BLESS) | **664 scénarios**, 3 tranches ok, 391 s ; aucun `.trace` modifié (`git status` : 0 fichier `.trace`) |
| `make test_crates` | 61 binaires de test, 830 tests réussis, 0 échec (somme des lignes `test result`, sortie filtrée) |
| `cargo check -p game --features lighting,ui_hui,debug_ui` | sans erreur (les imports retirés ne servaient à aucune feature) |
| build web : `APP_VERSION=v0.0.0 cargo check -p zombies --target wasm32-unknown-unknown --no-default-features --features render` | sortie 0 (`jjrs/mod.rs`, `jjrs/local.rs` ont des branches `cfg(wasm32)`) |
| `cargo fmt --all -- --check`, `scripts/check-forbidden.sh` (38 occurrences auditées), `scripts/check-rollback-registration.sh`, `make test_tools` (25) | OK |
| sonde `make probe_game_sim` | 62 fichiers/789 erreurs avant les retouches, **59/781 après** ; `--tests` : 61/813 ; `--sim-list scripts/d69-sim-files.txt` : OK (16 fichiers) ; test négatif (fichier hors fermeture, fichier à fuite, fichier inexistant) : sortie 1 avec les trois messages |
| sensibilité de l'outil d'ordre | permutation `PropsPlugin`/`BombsPlugin` dans `core.rs` détectée, `core.rs` restauré (`git status` ne le montre pas) |

Hors de ce que cette tâche a modifié : `make lint` ×5, `make gen` ×5, fuzz, p2p-stress et captures n'ont **pas** été rejoués (les retouches ne touchent ni données ni scénarios ni réseau actif ; les préparer est le travail de la fiche).

Hygiène disque (à savoir pour la suite) : le disque du Mac était plein pendant la tâche (`No space left on device` en plein build, wasm et `--features` à refaire) ; 233 binaires de test **périmés** (générations d'avant le 2026-10-18, doublons exacts de noms plus récents, 51 Go comptés par taille) supprimés de `target/headless/deps` (`df` : de 1,1 Go à 8,9 Go disponibles) ; `scripts/target-sweep.sh` ne les voit pas (il ne traite que `build/` et `incremental/`), `cargo clean` jamais lancé.

## 12. Non fait / limites

- Aucun fichier n'est déplacé, aucun type rollback créé ni retiré, aucun `BLESS`.
- Les recettes des annexes sont des propositions lues dans le code ; chaque étape les revalide par la sonde et l'outil d'ordre. Les numéros de ligne datent de `09b68187`/`8633a560`.
- L'outil d'ordre ne couvre pas la présentation (§7.1) ; les captures `cmp` de la fiche restent à jouer par E4, E5, E7.
- Estimations de durée : ordre de grandeur (D10).
- `make check_sim_boundary` n'est pas modifié : `game` et `map_ldtk` restent `PENDING` jusqu'à E9 et E11.
