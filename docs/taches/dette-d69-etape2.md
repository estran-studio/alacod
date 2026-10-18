# dette-d69-etape2 — découper `game` en `game_sim` et `game_app` (D69, étape 2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (règles GGRS, « Simulation et présentation », checklist de parité des types vides), le
**rapport `docs/taches/rapports/dette-d69-plan-etape2.md`** (inventaire, graphe, cycles R1-R18, décisions D1-D10) et ses annexes
`docs/taches/rapports/dette-d69-plan-etape2/annexe-*.md` (recettes de coupe fichier par fichier), l'étape 1
`docs/taches/rapports/dette-d69-combat-sim.md` (la recette : donnée de simulation inchangée, enveloppe `Asset` et visuels dérivés de
l'état côté application, `attach_bullet_visuals` en `PreUpdate` après `RunGgrsSystems`), `scripts/check-sim-boundary.sh`.
Une branche par livraison, `dette-d69-<id>` depuis `origin/main`. Estimations : voir chaque section (ordre de grandeur, rapport §6).

## But

Prouver par la compilation que la simulation de `game` (≈ 70 % des lignes) est de la simulation ECS seule, séparée de la
présentation, des entrées, du réseau et des chargeurs d'assets : `game_sim` (classe `SIM`, `make check_sim_boundary`) et `game_app`
(classe `APP`). C'est la frontière de portage console. **Aucune trace de scénario ne bouge, aucun `BLESS`.** Phase A (E1-E8) : couper
**sur place**, sans déplacer de fichier. Phase B (E9-E10) : les seuls déplacements, mécaniques. Phase C (E11) : `map_ldtk_sim`.

## Règles communes (valent pour chaque section)

1. **Rien n'est déplacé avant E9.** Une coupe crée le fichier de l'autre moitié **à son chemin final** (noms imposés ci-dessous : carte de
   destination au rapport §2.2), de sorte que E9 soit un `git mv` par fichier. Un fichier de simulation garde son chemin de module :
   `game_sim` aura la même arborescence que `game`.
2. **Un type ne change de fichier que s'il le doit**, et alors **octet pour octet** : mêmes derives, même `Hash` manuel, même `Debug`
   (CLAUDE.md : un champ ou une variante de plus déplace les traces). Un type rollback ne change **jamais** de plugin ni de position dans
   `CoreSetupPlugin::build` (annexe session §1.1).
3. **Aucun type rollback créé, retiré, renommé, passé en `neutral`/`no_checksum`.** Aucun nouveau `HashMap`/`HashSet`/`f32` dans la simulation.
   Un registre de simulation est un `BTreeMap`. Un système nouveau n'entre dans `GgrsSchedule` que s'il y est déjà (aucune étape n'en ajoute).
4. **Chaque appelant est migré vers le chemin final dans la même livraison** (`bots`, `scenario`, `map_ldtk`, `games`, `examples`,
   `templates`). Pas de `pub use` de compatibilité, pas d'alias : un module coupé n'est jamais réexporté sous son ancien nom. (Les réexports
   qui existent déjà et ne sont pas des shims de migration, `weapons.rs`→`combat::weapons`, `system_set.rs`, `rollback.rs`, restent.)
5. **Les visuels se dérivent de l'état** : système app `Update`, ou `PreUpdate.after(bevy_ggrs::RunGgrsSystems)` avec le filtre
   `With<X>, Without<Visuel>` (et **non** `Added<X>` : voir la correction de revue de l'étape 1, §4). Jamais d'événement émis par la simulation.
6. **Porte de livraison** (toutes les sections, sauf mention) :
   - `make check_sim_boundary` ;
   - `make schedule_order_diff BASE=<commit de base de la branche>` : sortie 0 (ordre des systèmes et des types rollback identique, 5 jeux) ;
   - `make test_scenarios_par SHARDS=3` **sans BLESS** : aucun `.trace` modifié (`git status` propre hors code et docs) ;
   - `make test_crates` ;
   - `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` : sortie 0 ; **ajouter à la liste les fichiers que l'étape libère** (la liste ne
     se réduit jamais) ;
   - `cargo fmt --all -- --check`, `scripts/check-forbidden.sh`, `scripts/check-rollback-registration.sh`, `make test_tools`,
     `make lint` ×5 et `make gen GAME=…` ×5 sans trace différente ;
   - fuzz `ALACOD_FUZZ=0:8` (+ `ALACOD_FUZZ_BOMBS=1` pour E3 et E4).
   Les **preuves supplémentaires** sont dans chaque section : p2p réel (`scripts/p2p-stress.sh zombies …` puis `isaac`, `MATCHBOX_PORT` et
   `COMPOSE_PROJECT_NAME` propres), build web (`APP_VERSION=… cargo check -p zombies --target wasm32-unknown-unknown --no-default-features
   --features render`), `cargo check -p game --features lighting,ui_hui,debug_ui`, captures `play_scenario --capture` avant/après comparées par `cmp`
   (comme l'étape 1 : `tears_shot_speed`, `two_players_shooting`, `gungeon_boss`, `isaac_hud_items`, un scénario à brouillard).
7. **Rapport** par livraison (`docs/taches/rapports/dette-d69-<id>.md`) : sonde avant/après (fichiers à fuite, erreurs), sortie de
   `schedule_order_diff`, tableau des vérifications, limites. Mettre à jour D69 dans `dettes.md` (une ligne de suivi) et `etat-engine.md` au passage de
   `game` en `SIM` (E9). `scripts/d69-sim-files.txt` est le cliquet : l'étape y ajoute ses fichiers.
8. **Outils disponibles** (livrés par `dette-d69-plan-etape2`) : `scripts/probe-game-sim.py` (`make probe_game_sim`, `--sim-list`, `--tests`, `--verbose`,
   `--json`), `scripts/schedule-order-diff.sh` (`make schedule_order_diff`, `ALACOD_SCHEDULE_OUT=<dossier>` pour garder les deux sorties), `crates/scenario/tests/schedule_order.rs`.

Retouches déjà faites par `dette-d69-plan-etape2` (ne pas les refaire) : imports morts de `jjrs/local.rs` et `jjrs/mod.rs`, dérives `Asset, TypePath` de
`ObstacleConfig`/`EnemyAiConfigRon`, `FIXED_TIMESTEP` dans `character/movement.rs`, paramètre `GlobalAsset` inutile de `handle_weapon_pickup_interaction`.

## Ordre et voies

```
L1 : E1 ─► E3a ─► E3b ─► E3c ─► E3d ──┐
L2 : E2 ─► E6 ────────────────────────┤
L3 :      E4 (E1,E3a)  E5a (E1,E2)  E5b (E3a)  E5c (E3b,E3c) ─► E7 ─► E8 ─► E9 ─► E10 ─► E11a ─► E11b ─► E11c
```

E1 et E2 partent ensemble. Fusionner **b1** (données isaac) avant E3b ou rebaser. **E9 demande un gel de `crates/game/`** (voir sa section).

---

## E1 — types de donnée sortis de leurs fichiers de présentation (`dette-d69-e1-types-donnees`)

Estimation 1 j. Prérequis : aucun. Casse R6, R8, R9, R14 (en partie), R5 (en partie).

### But
Faire sortir des fichiers APP ou MIXTE les petits types de **donnée** que la simulation, `scenario` ou `map_ldtk` lisent, sans changer une valeur ni un chemin autre que le leur.

### À faire
Déplacer, **octet pour octet**, vers un fichier de simulation (nouveau ou existant), et migrer chaque appelant (`game`, `bots`, `scenario`, `map_ldtk`, `games`, `examples`) :

| type | de | vers | appelants notables |
|---|---|---|---|
| `CameraFollowOverride(usize)` | `camera/mod.rs:26` | `ui/hud_model.rs` | `ui/hud_model.rs:28,79`, `ui/mutation_screen.rs:28,193`, `camera/mod.rs:223`, `scenario/runner.rs:527` |
| `WorldLabel(String)` et `SpriteRequest(String)` | `legibility.rs:427`, `world_sprites.rs:71` | `visual_requests.rs` (nouveau) | posés par `map_ldtk/game/floors.rs:769-770` |
| `UiStep`, `UiAction`, `ScreenId`, `ScreenScript` | `ui/screens/mod.rs:50-58`, `ui/screens/model.rs` | `ui/screens/model.rs` | `replay.rs:148` (`Scenario::ui`), `scenario/runner.rs:437` |
| `FogSettings`, `FogMap`, `FogRoom`, `FogDoor` | `room_fog.rs:20-66` | `fog_map.rs` (nouveau) | `map_ldtk/game/collider.rs:73,82`, `generation/building.rs:446,709` |
| `CharacterAppearance` (**rollback**, `Hash` manuel excluant la couleur) | `character/visuals.rs:19-37` | `character/appearance.rs` (nouveau) | `character/create.rs:24,105`, `character/mod.rs:142` ; son enregistrement reste **à la même place** dans `BaseCharacterGamePlugin` |
| `InteractionInput` | `character/player/input.rs:42` | `interaction.rs` | `interaction.rs:98,1056`, `character/player/create.rs`, `bots` |
| `increase_frame_system` | `frame.rs:12-14` | `frame_counter.rs` (nouveau) | `core.rs:377`, `curses.rs:222`, `profile.rs:449` |
| `WaveDebugEnabled` | `waves/debug.rs:11-13` | `waves/state.rs` | `scenario/runner.rs:327,3569`, `games/zombies`, `examples` ; son `init_resource` passe dans `WaveSystemPlugin` |

### Hors périmètre
`SIM_FPS`/`AppState` (E2), toute coupe de plugin, tout `Asset`.

### Acceptation
Porte commune. Ajouter à `scripts/d69-sim-files.txt` : `visual_requests.rs`, `fog_map.rs`, `frame_counter.rs`, `character/appearance.rs`, `ui/screens/model.rs` (déjà), et vérifier
que `replay.rs` **ne dépend plus d'aucun fichier app hors `character/player/input.rs`** (R6). Build web et `cargo check` des features (`lighting`, `ui_hui`, `debug_ui`) : le déplacement de types ne doit rien casser.
Rollback : `CharacterAppearance` apparaît dans `*.rollback.txt` à la **même position** ; sinon l'étape est rejetée.

---

## E2 — états et constantes de session hors de `core.rs` (`dette-d69-e2-session`)

Estimation 0,5 j. Prérequis : aucun. Casse R4 (la moitié « types »).

### But
`AppState`, `OnlineState`, `SIM_FPS`, `CoreSetupConfig`, `is_headless` et `single_threaded_rollback_schedules` sont lus par 32 fichiers de `game` et 40 lecteurs externes
mais définis dans `core.rs`, qui importe `ui`, `camera`, `audio`, `light`. Les poser dans un module de simulation.

### À faire
1. Créer `session.rs` (module `game::session`, plus tard `game_sim::session`) avec : `AppState` (`core.rs:74-83`), `OnlineState` (`:85-91`), `SIM_FPS` et `is_headless` (`:539-545`), `CoreSetupConfig` (`:54-72`), `single_threaded_rollback_schedules` (`:404-424`). Mêmes derives. `GameInfo` **reste** dans `core.rs` (app : `env!("APP_VERSION")`).
2. Migrer les 32 fichiers de `game` (`balance`, `clock`, `hub`, `run_state`, `state_trace`, `performance`, `jjrs/{mod,local,p2p}`, `ui/{hud_model,hud,game_over,screens/mod,mutation_screen}`, `room_fog`, `interaction`, `items`…) et les lecteurs externes (`scenario` 15, `map_ldtk` 6, `bots` 1, `games` 12, `examples` 4, `templates` 2) : liste exacte dans `annexe-lectures-externes.md` (`core::AppState`, `core::OnlineState`, `core::SIM_FPS`, `core::CoreSetupConfig`).
3. `core.rs` garde `CoreSetupPlugin`, `get_default_plugin`, `PresentationPlugin`, `GameInfo`, `debug_ui_enabled`. `CoreSetupPlugin` et `PresentationPlugin` ne bougent pas d'une ligne dans `build`.

### Acceptation
Porte commune ; build web ; `session.rs` dans la liste du cliquet. `grep -rn "core::\(AppState\|OnlineState\|SIM_FPS\|CoreSetupConfig\)"` ne renvoie plus rien hors `core.rs`.

---

## E3a — configuration des personnages en ressource de simulation (`dette-d69-e3a-character-configs`)

Estimation 1,5 j. Prérequis : E1. Casse R3. **Étape la plus délicate de E3 pour le moment du remplissage (D2).**

### But
Aujourd'hui `CharacterConfig` est à la fois la donnée et un `Asset`, et chaque personnage porte `CharacterConfigHandles { config: Handle<CharacterConfig> }` ; quatre systèmes de simulation
lisent `Res<Assets<CharacterConfig>>` : `apply_inputs` (`player/input.rs:432,477`), `rollback_resolve_damage_events` et `rollback_apply_accumulated_damage` (`health/mod.rs:166,377`), `move_enemies`
(`ai/pathing.rs:213,314`), plus `interaction.rs:628,816`, `familiars.rs`, `champions.rs`, `waves/systems.rs`, les créateurs (`create.rs:30-66`, `enemy/create.rs:69-92`, `enemy/spawning.rs:58-63`,
`player/create.rs:35-42,110-114`), `bots/src/input.rs:45,174,247`, `map_ldtk/game/{local,rooms}.rs`, `scenario/tests/{expectations.rs:1767,extra_health.rs:255,scenarios.rs:1567}`.

### À faire
1. `character/config.rs` : `CharacterConfig` perd `Asset, TypePath` (reste `Deserialize, Debug, Clone`) ; ajouter `CharacterConfigKey(pub String)` (`Component`, remplace `CharacterConfigHandles` : la clé est l'`asset_name_ref` que `create.rs:63-67` clone déjà) et la ressource `CharacterConfigs(BTreeMap<String, CharacterConfig>)` (clés = celles de `GlobalAsset::character_configs`). `Hash`/`Debug` de ce qui est rollback inchangés (`CharacterConfigHandles` n'est **pas** rollback : à vérifier avant de remplacer).
2. Nouveau `character/config_asset.rs` (app) : `CharacterConfigAsset(pub CharacterConfig)` (`#[derive(Asset, TypePath, Deserialize)] #[serde(transparent)]`, tous les `.ron` existants se chargent sans modification), `RonAssetPlugin::<CharacterConfigAsset>` (déplacé de `character/mod.rs:85`), et le système de synchro **`PreUpdate.before(bevy_ggrs::RunGgrsSystems)`** qui remplit `CharacterConfigs` à partir des assets chargés (et au rechargement à chaud : `content_hot_reload.rs:18` suit l'enveloppe).
3. Remplacer chaque lecture `Assets<CharacterConfig>` + `CharacterConfigHandles` par `Res<CharacterConfigs>` + `CharacterConfigKey` (sites ci-dessus). `GlobalAsset::character_configs` reste, côté app, la source de la synchro.
4. `scenario/src/generate.rs` parse `CharacterConfig` en RON directement : inchangé.
5. **Test dédié** : « le premier tick de simulation d'un personnage créé voit son entrée de `CharacterConfigs` » (replis existants : `input.rs:477 continue`, `pathing.rs:314-318` vitesse de repli, `health/mod.rs:306,458` défauts : ils masqueraient un registre vide).

### Hors périmètre
Armes (E3b), `GlobalAsset` (E3d), `Visibility`/`ActionState` des spawns (E4).

### Acceptation
Porte commune + **p2p-stress** zombies et isaac (premier tick, rollback) ; fuzz `ALACOD_FUZZ_BOMBS=1`. Les 4 systèmes ne contiennent plus `Assets`. Ajouter au cliquet les fichiers libérés (`character/health/mod.rs`, `ai/pathing.rs` si E3b n'y touche plus…).

---

## E3b — registre d'armes de simulation (`dette-d69-e3b-weapon-registry`)

Estimation 2 j. Prérequis : E3a (même mécanique) et **b1 fusionnée**. Casse R2.

### But
Les enveloppes `WeaponsConfig`/`MeleeWeaponsConfig` (étape 1, `weapons.rs:25-31`) sont lues par ≈ 15 fichiers de simulation : `champions.rs:61,146-147,355`, `effects_runtime.rs:56,133`, `effects_v3.rs:550,557`, `loot.rs:42,117,218,271`, `progression.rs:431`, `shop.rs:46,403`, `character/enemy/{create.rs:13-15,69-70,spawning.rs:12,61-62}`, `character/player/create.rs:17-19,40-41`, `waves/systems.rs:27,34-35`, `bots/src/input.rs:246`, `scenario/runner.rs:674,1195`.

### À faire
1. `weapons.rs` (sim) : `WeaponRegistry { weapons: BTreeMap<String, WeaponAsset>, melee: BTreeMap<String, MeleeWeaponAsset> }` (les tables d'enveloppe actuelles sont des `HashMap` bevy, seulement lues par `.get` : copie en `BTreeMap`, aucune itération non déterministe). Remplace `EffectAssets.weapons`, `SpawnAssets`, `DivisionAssets`.
2. `weapon_assets.rs` (app, nouveau) : `WeaponAssetsPlugin`, enveloppes, rechargement à chaud (`weapons_config_update_system`, `weapons.rs:108-110`), `weapon_inventory_system`, `hide_new_melee_weapons`, **et** la synchro qui publie `WeaponRegistry` (`PreUpdate.before(RunGgrsSystems)`, régénéré sur `AssetEvent<WeaponsConfig>`).
3. `scenario/runner.rs::apply_weapon_overrides` (`:671`) et les lecteurs de `Assets<WeaponsConfig>` (`:728-800,912,1195`, `tests/*`) visent le **registre**, après la synchro ; `bots/src/input.rs:246` lit `Res<WeaponRegistry>`. L'ordre `.before(weapon_inventory_system)` de `runner.rs:430-433` disparaît si la fonction passe côté app : vérifier que `apply_player_overrides` ne lit jamais l'ancien `WeaponInventory` (le `ApplyDeferred` automatique suffisait).
4. `powerups.rs`, `economy.rs`, `waves` ne sont pas concernés ici (E3c).

### Acceptation
Porte commune + p2p-stress ; scénarios `weapon_*`, `shotgun_*`, `reload_*` sans BLESS ; fuzz. Le rechargement à chaud (`native`) doit régénérer le registre : essai manuel documenté (modifier `weapons.ron`, voir l'arme changer) car aucun scénario ne le joue. Cliquet : `effects_runtime.rs`, `loot.rs`, `progression.rs`, `champions.rs`, `character/enemy/{create,spawning}.rs` si E3a est fusionnée.

---

## E3c — économie, power-ups, vagues, équilibrage en données de simulation (`dette-d69-e3c-balance`)

Estimation 2 j. Prérequis : E3b. Casse R15, R16.

### À faire
1. `economy.rs`, `powerups.rs`, `waves/config.rs` : `EconomyConfig`, `PerksConfig`, `PowerUpsConfig`, `WaveConfig` perdent `Asset, TypePath` (`economy.rs:79,168`, `powerups.rs:97`, `waves/config.rs:27`) ; les enveloppes `#[serde(transparent)]` vont en `economy_assets.rs`, `powerups_assets.rs`, `waves/assets.rs` (app) avec leurs `RonAssetPlugin` (`economy.rs:315-316`, `powerups.rs:569`, `waves/mod.rs:65`), ajoutés **juste avant** le plugin de simulation correspondant (l'ordre des types rollback `economy.rs:318-322` est dans `EconomyPlugin`, inchangé ; `PowerUpsPlugin` reste à la position 165 de `CoreSetupPlugin::build`).
2. `PowerUpsTable(Option<PowerUpsConfig>)` résolu à `OnEnter(GameLoading)` après `resolve_balance_system` ; `powerup_pickup_detect_system` (`powerups.rs:169-192`) et `loot_drop_on_death_system` (`:492-513`) — qui lisent l'asset **à chaque frame** — lisent la table.
3. `ResolvedBalance` gagne `economy.shells_drop`, `economy.shop_prices` (lus par `shop.rs:258-278` et `map_ldtk/rooms.rs:451,460-465`) et `waves_enabled: bool` (remplace `global_assets.wave_config.is_none()` de `waves/systems.rs:72,218`) ; `clock.rs:95-96,151-156` et `balance.rs:29-32,310` lisent `SessionPlayers(u32)` (ressource de simulation écrite par l'app avant `OnEnter(GameLoading)`) au lieu de `OnlineState` + `GggrsSessionConfiguration`.
4. `balance.rs` : types résolus et `resolve_*` purs (`:37-285`) restent ; `resolve_balance_system` (`:292-352`) passe dans `balance_assets.rs` (app), mêmes noms exportés, même `OnEnter(GameLoading)`.
5. `scenario/runner.rs` : `apply_wave_overrides`/`apply_powerup_drop_chance_override` visent l'enveloppe **avant** la résolution (déjà vrai : la valeur lue est celle de l'asset à ce moment) ou la table après elle ; documenter lequel.

### Acceptation
Porte commune ; scénarios power-ups, boutique, vagues, `wave_*`, `powerup_*` sans BLESS ; **la parité** `ResolvedBalance` identique champ à champ (test unitaire de comparaison enveloppe/table). Les tests inline de `powerups.rs:603-620,851,921` sont réécrits sur la table.

---

## E3d — `GlobalAsset` réduit à la présentation (`dette-d69-e3d-global-asset`)

Estimation 1 j. Prérequis : E3c. Casse R1.

### À faire
1. `GlobalAsset` (`global_asset.rs:30-58`) ne garde que les `Handle` de feuilles de sprites, animations et effets (consommés par `character/visuals.rs`, `ui/weapon_visuals.rs`, `world_sprites.rs`, `feedback*.rs`, `powerup_visuals.rs`) ; les `Handle<…Config>` des enveloppes restent pour la synchro. `loading_asset_system` publie les registres avant `NextState::set` (sortie de `AppState::Loading`).
2. Ressource de simulation `ContentData` seulement si un lecteur subsiste après E3a-c ; sinon aucune. Vérifier par `grep -rn "GlobalAsset" crates/game/src` : seuls des fichiers APP.
3. Lecteurs externes : `bots/src/input.rs:51,245,294`, `scenario/runner.rs:38,673,680,727,738,783,794,911,1193,1221,1333,1349`, `map_ldtk/game/{local.rs:17,42,139,379,402,rooms.rs:55,450,615,1284}`, `examples/character_tester.rs:17,80` migrent vers les registres.

### Acceptation
Porte commune + p2p-stress ; `GlobalAsset` n'apparaît plus dans aucun fichier du cliquet ; **la sonde ne signale plus aucune erreur `asset` dans un fichier classé SIM** (27 erreurs dans 12 fichiers aujourd'hui ; les 149 autres sont dans des fichiers MIXTE, 61, ou APP, 88, qui se coupent ou se déplacent).

---

## E4 — personnage : entrées, plugin, visuels (`dette-d69-e4-personnage`)

Estimation 2 j. Prérequis : E1, E3a. Casse R5, R17 ; annexe `annexe-personnage.md` §1, §4, §5.

### À faire
1. `character/player/input.rs` : scinder `read_local_inputs` (`:219-387`). `game` garde `read_local_inputs` (même nom : `bots` et `recording` s'ordonnent `.after`), qui perd `q_window`, `q_camera`, `ActionState` et l'appel à `ui::mutation_screen::confirm_input` (`:232,337`), gagne `Option<Res<DeviceInputs>>` (`DeviceInputs(pub BTreeMap<usize, BoxInput>)`) ; la branche `Devices` (`:258-361`) devient la lecture de cette ressource, les branches Remote/Scripted/Neutral restent (`:236-258`). `character/input_devices.rs` (app, nouveau) : `collect_device_inputs` (leafwing, `PrimaryWindow`, `Camera`, `confirm_input`), `ReadInputs`, **`.before(read_local_inputs)`**, absent en headless. `FIXED_TIMESTEP`, `InteractionInput` déjà sortis.
2. `character/player/create.rs` : `create_player` ne pose plus `ActionState`/`InputMap` (`:85-91`) ni `PointLight2d` (`:167-186`) ; `attach_local_player_input(Query<Entity, Added<LocalPlayer>>)` (`input_devices.rs`) en `PreUpdate.before(RunGgrsSystems)` ; `attach_player_light(Query<Entity, Added<Player>>)` (`light/mod.rs`, sous `feature = "lighting"`) ; `AiDriver` (harmonium) sous le même `cfg`.
3. `character/mod.rs` : `BaseCharacterGamePlugin::build` éclaté. Garder dans le plugin de simulation **exactement** l'ordre actuel des `rollback_and_trace*`/`add_frame_events` et le bloc `GgrsSchedule` (`:207-308`) ; `character/mod.rs` côté app : `CharacterAppPlugin` (`RonAssetPlugin::<CharacterConfigAsset>` s'il n'y est pas déjà, `InputManagerPlugin::<PlayerAction>`, ressources de debug `FlowFieldDebug`/`EnemyStateDebug` lues de `DebugAiConfig` : `:106-118`) et `CharacterPresentationPlugin`. Ajouter `CharacterAppPlugin` **juste après** le plugin de simulation dans `core.rs:144` ; `skip_device_input` (`:76-79`, appelant unique `scenario/runner.rs:531`) disparaît (plus de leafwing en headless).
4. `character/visuals.rs` : sans `CharacterAppearance` (E1), reste app. `character/create.rs` : retirer `Visibility::default()` (`:103`, l'insérer dans `attach_character_visuals`, `visuals.rs:78-81`) et `SpatialAudioEmitter` (`:3,166` : composant inerte, `instances` jamais rempli ; **attention** `SpatialAudioReceiver` existe côté caméra, `camera/mod.rs:630,648`, le laisser).
5. `collider/mod.rs` : `spawn_test_wall` (`:25-108`, `Sprite` + lumière) → `collider/test_wall.rs` (app) ; appelants `jjrs/local.rs`, `map_ldtk/game/collider.rs:6`, `examples/character_tester.rs:15,139-166` migrés ; `collider/debug.rs` reste app.
6. `character/enemy/ai/mod.rs` : retirer `pub mod debug` et son `pub use` (`:8,18-21`) ; `ai/debug.rs` → `character/ai_debug.rs`. `character/health/ui.rs` → `character/health_bar.rs`, `character/player/control.rs` → `character/control.rs` (APP purs : renommés en E9 mais les **lecteurs** `camera/mod.rs:14`, `ui/mutation_screen.rs:29`… sont déjà côté app).
7. Tests inline de `player/input.rs:860-894` (`ActionState`) → app.

### Acceptation
Porte commune ; `cargo check -p game --features lighting,ui_hui,debug_ui` ; captures `cmp` (`two_players_shooting`, `gungeon_boss`, un scénario à lumière si `lighting`) ; `ALACOD_INPUT=neutral`, `script:`, `remote` inchangés ; essai manuel clavier + manette (D4 : une frame d'`ActionState` tardive tolérée, à constater). Cliquet : `character/player/input.rs`, `character/player/create.rs`, `character/mod.rs`, `character/create.rs`, `collider/mod.rs`, `replay.rs`, `recording.rs` (si E6 fait), `ai/*`.

---

## E5a — écrans, HUD, mutation (`dette-d69-e5a-ecrans`)

Estimation 1,5 j. Prérequis : E1, E2. `annexe-presentation.md` §1.1.

### À faire
1. `ui/screens/mod.rs` : modèle (`Screens`, `UiActionQueue` rendu `pub`, `InventorySource`, `MerchantSource`, `update_screens`, `ScreensPlugin` sans clavier) reste ; `read_nav_keys` (`:83-130`) → `ui/screens/input.rs` (app), ordre `read_nav_keys.before(update_screens).run_if(in_state(AppState::InGame))` à la place du `.chain()` de `:63-67`.
2. `ui/mutation_screen.rs` : modèle (`:35-172,250-260`, `MutationScreenModelPlugin`) reste ; `read_mutation_nav` (`:223-237`, `ActionState<PlayerAction>`) → `ui/mutation_input.rs` (app), écrit la ressource de simulation `MutationNavInput { handle, held: u32, step: i32 }` que `update_mutation_screen_view` lit ; `.before(update_mutation_screen_view)`.
3. `ui/mod.rs` : `GameUiModelPlugin` (mutation, `hud_model`, `screens`) **à la place exacte** de `core.rs:184` ; `GameUiPlugin` (`LobbyUiPlugin`, `DisconnectedUiPlugin`) : **hors `PresentationPlugin`** (ils tournent aujourd'hui aussi en headless p2p : `LobbyOnline`) — le rattacher à `CoreAppPlugin` en E7, en attendant le laisser appelé par `core.rs`.
4. `ui/hud_model.rs` : `CameraFollowOverride` (E1) et `SIM_FPS` (E2) importés de modules de simulation.

### Acceptation
Porte commune ; scénarios `HudText`/`ScreenText` (`isaac_hud_items`, `isaac_banner_blind`, `m2_ecrans_*`) ; `p2p-stress` (le lobby doit encore s'afficher en headless) ; capture `cmp` d'un écran de pause. Cliquet : `ui/hud_model.rs`, `ui/screens/mod.rs`, `ui/mutation_screen.rs`.

---

## E5b — journal de feedback en simulation (`dette-d69-e5b-feedback`)

Estimation 1,5 j. Prérequis : E3a. `annexe-presentation.md` §1.3.

### À faire
1. `feedback.rs` : le journal (`FeedbackKind`, `FeedbackCue`, `FeedbackLog`, `FEEDBACK_HISTORY_LEN`, `PatternKind`/`pattern_label`/`fan_rays`, `TelegraphShape`, `record_feedback_cues` `:475-657`, `record_t19_cues` `:658-784`, `FeedbackConfigLoaded` `:48`, fonctions pures d'enveloppe `:331-364`, `pub use content::feedback::*`) → `feedback_log.rs` (sim, nouveau) ; rendu, sons, `FeedbackConfig` (asset) et son chargeur restent dans `feedback.rs` (app). Lecteurs : `scenario/src/events.rs:209-251`, `scenario/runner.rs` (3), `feedback_v3.rs`.
2. `feedback_v3.rs` : `variant_tint` lit `GlobalAsset` + `Assets<CharacterConfig>` (`:29-37,320-335,388-389,507`) → `VariantTints(BTreeMap<(config, variant), (f32, f32, f32)>)` ressource publiée par l'app depuis `CharacterConfigs` (E3a) ; le journal v3 (`record_v3_cues` `:451-607`, `cue_label` `:610`) reste simulation.
3. `FeedbackLogPlugin` (`core.rs:183`) à la même place, sans `RonAssetPlugin` (`feedback.rs:441` → `FeedbackAssetsPlugin` app).

### Acceptation
Porte commune ; `Event(kind: "feedback")` des scénarios (`status_charm*`, `transformation_*`) ; captures `cmp`. Cliquet : `feedback_log.rs`, `feedback_v3.rs`.

---

## E5c — interaction, objets, vagues, brouillard, armes : plugins coupés (`dette-d69-e5c-plugins`)

Estimation 2 j. Prérequis : E3b, E3c. R10, R16.

### À faire
1. `interaction.rs` : lignes 1095-1315 (`update_door_visuals`, `WindowHealthBar`, `update_window_health_bars`, `display_interaction_prompts`) → `interaction_view.rs` (app), `InteractionPresentationPlugin` (`Update`, `run_if(in_state(AppState::InGame))`, copie exacte de `:1083-1092`) ajouté à `PresentationPlugin`.
2. `items.rs:632-699` (`BlankPresentationPlugin`, `track_blank_system`, `draw_blank_wave_system` : `Gizmos`) → `items_view.rs` (app) ; `EffectAssets` lit `WeaponRegistry` (E3b).
3. `waves/mod.rs:65,68` : `WaveAssetsPlugin` et `WaveDebugPlugin` (app) ; `waves/debug.rs` reste app.
4. `room_fog.rs` : sans `Fog*` (E1), tout app (`capture_vision` compris : il n'a jamais tourné en headless).
5. `weapons.rs` : `weapon_inventory_system` (`Visibility` des armes portées) et `hide_new_melee_weapons` côté `weapon_assets.rs` (E3b).

### Acceptation
Porte commune ; captures `cmp` (portes, fenêtres abîmées, blank, vagues avec overlay, brouillard) ; le `.before(weapon_inventory_system)` de `scenario/runner.rs:430-433` est retiré avec la fonction. Cliquet : `interaction.rs`, `items.rs`, `weapons.rs`, `waves/mod.rs`.

---

## E6 — session, réseau, arguments, télémétrie (`dette-d69-e6-session-reseau`)

Estimation 2 j. Prérequis : E2 (b2 a fusionné `8633a560`). R7, R11, R12 ; `annexe-session.md` §2.2-§2.5, §5.

### À faire
1. `jjrs/p2p.rs` : côté sim `PreviousSessionPeers`, `live_players`, `SYNC_TIMEOUT_*`, `SessionSyncWatch`, `sync_timed_out`, `reset_sync_watch`, `abandon_unsynchronized_session`, `MAX_PREDICTION_WINDOW`, `session_builder` (+ `p2p_tests.rs`) ; côté app (`jjrs/p2p.rs` app) `default_ice_server`, `start_matchbox_socket`, `open_matchbox_socket`, `wait_for_players`. **Couper `wait_for_players` et `system_after_map_loaded`** en deux fonctions : l'appariement joueurs socket ↔ `GggrsSessionConfiguration` (`:255-319`) devient une fonction de simulation sans socket qui prend la liste des pairs ; l'app lit `socket.players()`. Valeurs et frame d'émission identiques ; relire les modifications de b2 (`8633a560`, D87 `p2p-silences`, 65 lignes).
2. `jjrs/restart.rs` : `drop_matchbox_socket` (3 lignes, `:75-79`) → app ; son `OnExit(InGame)` (`core.rs:320-323`) suit.
3. `jjrs/mod.rs` : `log_ggrs_events` sans télémétrie (`:166-167,211-227`) ; nouveau message de simulation `DesyncObserved { frame, local_checksum, remote_checksum, addr_debug }` (même idiome que `SessionLostEvent`) écrit au même instant ; `jjrs/telemetry.rs` (app) : `forward_desync_telemetry`.
4. `args/mod.rs` : `GameArgsPlugin` (sim, lu par `scenario`) ne pose plus `TelemetryConfig` (`:248-252`) ; `TelemetryConfigPlugin(GameArgs)` (app, ajouté par `BaseArgsPlugin` juste après `GameArgsPlugin`). `scenario/runner.rs:307` ajoute `GameArgsPlugin` seul : plus de `TelemetryConfig` dans les scénarios (vérifier que `telemetry::TelemetryPlugin` n'est jamais ajouté là).
5. `recording.rs:297` : `.after(bevy::window::ExitSystems)` → `.after(state_trace::ExitRequests)` (déjà `:296`) ; l'app ajoute `configure_sets(Last, bevy::window::ExitSystems.before(ExitRequests))` si l'ordre doit être conservé.
6. `run_state.rs:482-488` (wasm) : message de simulation consommé par un système app qui appelle `jjrs::browser::{session, alacod_return_to_lobby}`.
7. `frame.rs` : sans `increase_frame_system` (E1), tout app (UI de debug, `GameInfo`).
8. `jjrs/p2p_tests.rs:151` : `bevy_matchbox::prelude::PeerId` → `matchbox_protocol::PeerId` (même type ; ajouter la dépendance de données à `game`, déjà dans `combat`) ; vérifié par `make probe_game_sim ARGS=--tests` (`python3 scripts/probe-game-sim.py --tests`).

### Acceptation
Porte commune ; **p2p** : `scripts/p2p-stress.sh` zombies et isaac (≥ 1 000 frames resimulées, 0 desync), `p2p-restart.sh`, `p2p-lobby.sh`, `p2p-disconnect.sh`, `p2p-sync-timeout.sh` ; build web (`cfg(wasm32)` touché : R11, `jjrs/browser.rs`) ; `scripts/check-p2p-traces.py` sur les traces. Cliquet : `jjrs/restart.rs`, `jjrs/local.rs`, `jjrs/p2p_tests.rs`, `recording.rs`, `run_state.rs`, `profile.rs`, `curses.rs`.

---

## E7 — `CoreSetupPlugin` en deux plugins (`dette-d69-e7-core`)

Estimation 1,5 j. Prérequis : E3d, E4, E5a-c, E6. **Étape la plus sensible à l'ordre.** `annexe-session.md` §1.1 (56 éléments numérotés) et §1.3.

### À faire
1. `core_sim.rs` (nouveau, devient `game_sim::core`) : `CoreSimPlugin(CoreSetupConfig)`, dont `build` reprend **dans l'ordre exact** les éléments SIM du §1.1 de l'annexe session : 0, 2, 4, 6, 8, parties SIM de 9-10, 11-15, 16-18, 19-29, 30-32, 33-40 (parties SIM de 9-10, 16-18 et 30-32 ; **l'ordre numérique est l'ordre de `build`** : `WaveSystemPlugin` (32) reste après 19-29), 43, 45 (sim), 46 (sim), 47 (sim), 48 (sim), 49 (sim), 50-52, 55, et `finish` (`single_threaded_rollback_schedules`). **L'élément 38 (les cinq enregistrements de clôture `GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`, `RngStreams`) reste le dernier enregistrement rollback.** `ProfilePlugin::build` lit `ProfileSettings` et `BaseCharacterGamePlugin` lit `DebugAiConfig` : ils restent dans ce bloc, après les `insert_resource` de l'appelant (`scenario/runner.rs:306-318`).
2. `core.rs` (app) : `CoreAppPlugin(CoreSetupConfig)` (éléments APP : 1, 3, 5, 7, parties APP de 9-10/16-18/30-32, 41, 42, 44, 45 `wait_for_players`, 46 `drop_matchbox_socket`, 47 socket, 49 télémétrie, 53, 54), `get_default_plugin`, `PresentationPlugin` (ordre interne `:507-529` **conservé**), `GameInfo`, `debug_ui_enabled`, `GameUiPlugin` (Lobby, Disconnected) **hors `PresentationPlugin`**.
3. `CoreSetupPlugin` garde son nom (lu par `games/*`, `examples`, `templates`, `scenario`) : `build` = `app.add_plugins(CoreSimPlugin(cfg.clone()))` **puis** `app.add_plugins(CoreAppPlugin(cfg))` ; `finish` inchangé. Les plugins APP viennent donc **après** tous les plugins de simulation (aucun n'enregistre de rollback, rapport §2.1).
4. Si `Plugin::build` de `CoreSimPlugin` doit être réutilisable sans `CoreAppPlugin` (E11, `bots`), prévoir un test qui construit `CoreSimPlugin` seul sur `MinimalPlugins` et initialise les schedules (c'est ce que fait déjà `schedule_order.rs` pour le plugin complet).

### Acceptation
Porte commune **avec `make schedule_order_diff` strictement vide** (c'est la preuve de l'étape) ; **l'outil ne couvre pas l'ordre de `Update`/`PostUpdate` de la présentation** : le justifier (les systèmes app s'ordonnent entre eux par `.before/.after`, le bloc `PresentationPlugin` garde son ordre interne) et le contrôler par captures `cmp` avant/après (option : binaire `harness = false` qui dumpe ces schedules, winit exige le thread principal); p2p-stress ; captures `cmp` ; build web. Aucun fichier `core*.rs` ne doit importer `ui`, `camera`, `audio`, `light` côté sim.

---

## E8 — porte de sortie de la phase A (`dette-d69-e8-porte`)

Estimation 0,5 j. Prérequis : E7.

### À faire
1. `scripts/d69-sim-files.txt` = **tous** les fichiers classés SIM après coupes (53 au départ, plus les moitiés simulation des 28 MIXTE ; les fichiers APP et les moitiés app n'y sont pas). Vérifier par `make probe_game_sim SIM_LIST=scripts/d69-sim-files.txt` et **`--tests`** (tests inline compris : D7).
2. La fermeture : aucun fichier de la liste ne dépend d'un fichier hors liste (le script le vérifie).
3. `cargo check` ECS seul simulé : la sonde sur la liste est **vide d'erreur** pour tous les fichiers de la liste. Écrire le tableau « fichier → destination » final dans le rapport (c'est la carte de E9) et la liste des `git mv`.
4. Mettre à jour `scripts/determinism-allowlist.json` **dans E9** (chemins), pas ici.

### Acceptation
Sonde : 0 fichier de la liste en erreur ; le nombre de fichiers à fuite restants = exactement les fichiers app (≈ 60). Si un fichier sim fuit encore, retour à l'étape qui l'a manqué.

---

## E9 — `game_app` : déplacement des fichiers de présentation (`dette-d69-e9-game-app`)

Estimation 2 j, **fenêtre de gel de `crates/game/`**. Prérequis : E8 ; annoncer le gel, fusionner toute branche qui touche `crates/game/src`, exécuter E9 et E10 le même jour. Réutilisable par `git` : les fichiers de simulation ne sont **pas** touchés.

### À faire
1. Créer `crates/game_app` (`Cargo.toml` : dépend de `game` + `animation`, `telemetry`, `bevy` (features complètes), `bevy_kira_audio`, `bevy_matchbox`, `leafwing-input-manager`, `bevy_common_assets`, `bevy_light_2d`/`bevy_hui`/`bevy-inspector-egui` (optionnels), `clap`, `chrono`, `tracing-subscriber`/`-appender`/`-web`, `ureq`, `ed25519-dalek`, `base64`, `getrandom`, `web-sys`, `wasm-bindgen`, `console_error_panic_hook`, `harmonium_bevy` (optionnel) ; features `ui_hui`, `lighting`, `debug_ui`, `harmonium`, `dev-dylib` ; `build.rs` qui pose `APP_VERSION`). `crates/game/Cargo.toml` : bevy `default-features = false` + `serialize`, `bevy_color`, `bevy_state`, **plus aucune** de ces dépendances ; `game` ne dépend plus de `animation`, `telemetry`.
2. `git mv` de la carte de destination (rapport §2.2, colonne « destination ») : fichiers APP (`ui/{hud,lobby,disconnected,game_over,floor_transition,mutation_hui,weapon_hud,weapon_visuals,screens/hui,screens/input,mutation_input}.rs`, `camera/`, `audio/`, `light/`, `web/`, `remote.rs`, `logs.rs`, `performance.rs`, `legibility.rs`, `world_sprites.rs`, `powerup_visuals.rs`, `global_asset.rs`, `cave_assets.rs`, `content_hot_reload.rs`, `character/{visuals,health_bar,control,ai_debug,input_devices,config_asset}.rs`, `collider/{debug,test_wall}.rs`, `jjrs/{allumette,browser,telemetry}.rs`, `args/{cli,web}.rs`, `*_assets.rs`, `*_view.rs`, `core.rs` (app), …) vers `crates/game_app/src/` **au même chemin relatif** ; `lib.rs` des deux crates. Réécrire `crate::<module de simulation>` → `game::<module>` dans les fichiers déplacés.
3. Lecteurs externes app : `core::CoreSetupPlugin`, `args::BaseArgsPlugin`, `content_hot_reload::GameRoot` (games ×5, examples, templates), `camera::ui`, `legibility` (scenario) → `game_app::…` ; `Cargo.toml` racine (features `render`, `lighting`, `native`), `games/*`, `scenario` (`render = ["map_ldtk/render", "game_app/ui_hui"]`, `dev-dylib`), `templates/starter` (`{{engine:game_app}}` + `{{engine:game}}`), `map_ldtk`, `scripts/dev-features.sh`, `scripts/scenario-video:49`, `scripts/check-forbidden.py` (`CRATES`), `scripts/determinism-allowlist.json` (8 fichiers de `game` : `balance`, `character/config`, `character/player/input`, `global_asset`, `performance`, `ui/weapon_visuals`, `waves/config`, `weapons` : chemins suivis).
4. `scripts/check-sim-boundary.sh` : `game` de `PENDING` à `SIM` ; `game_app` en `APP` ; `map_ldtk` de `PENDING` à `APP`. Le script doit échouer si un crate de `crates/` n'est pas classé (déjà le cas).
5. Documentation : `CLAUDE.md` (section « Frontière simulation/présentation », liste `SIM`/`PENDING`), `docs/etat-engine.md` (D69), `docs/conventions.md` §4, `dettes.md` D69.

### Acceptation
`make check_sim_boundary` : **`game` ECS seul** (graphe seul : aucun crate de présentation) et `cargo check --no-default-features` des 14 crates ; porte commune (traces identiques, **dump identique**) ; build web ; features ; p2p-stress ; captures `cmp`. Le chiffre qui clôt D69 pour `game` : **14 crates SIM** (13 + `game`).

---

## E10 — `game` → `game_sim` (`dette-d69-e10-renommage`)

Estimation 0,5 j. Prérequis : E9, même jour.

### À faire
`git mv crates/game crates/game_sim` ; `name = "game_sim"` ; `game::` → `game_sim::` dans `bots` (89 occurrences), `scenario` (478), `map_ldtk` (160), `games` (16), `examples` (14), `templates` (2) ; `game_app` : `game::` → `game_sim::` ; `Cargo.toml` racine et crates (`game = { path }` → `game_sim`), `Cargo.lock`, `Makefile` (commentaires), `scripts/` (`check-sim-boundary.sh`, `check-forbidden.py`, `determinism-allowlist.json`, `check-rollback-registration.sh`), `CLAUDE.md`, `docs/conventions.md`, `etat-engine.md`, `plan-engine.md`. Les rapports historiques de `docs/taches/` ne sont **pas** réécrits. Supprimer `scripts/probe-game-sim.py` et la cible `probe_game_sim` (la frontière est dans `check-sim-boundary.sh` : `game_sim` en `SIM`), garder `schedule-order-diff.sh` (chemin de la sortie inchangé) ; `d69-sim-files.txt` supprimé.

### Acceptation
Porte commune ; `grep -rn "\bgame::" crates games examples templates scripts` : zéro hors commentaires historiques ; `cargo test -p game_sim --no-default-features` passe (tests ECS seul).

---

## E11a — `map_ldtk` : présentations en place (`dette-d69-e11a-map-ldtk-presentation`)

Estimation 1 j. Prérequis : E10. `annexe-map_ldtk.md` §5, §8.

### À faire
`rooms.rs:147,221-233` : `update_hud_room_geometry` → plugin app (`RoomsHudPlugin`), même ordre `.before(HudModelSet)` ; `floors.rs:704-776` (`Visibility`, `Sprite`, `SpriteRequest`, `WorldLabel`) et `cave.rs:150-177` (`Sprite`) → `FloorsPresentationPlugin`, `CavePresentationPlugin` ; supprimer le paramètre `window_bars` de `plugin.rs:622-635` (doublon de `floor_window_bars_system`, `floors.rs:721-748`, à rendre inconditionnel) ; supprimer `src/mod.rs` (non compilé) ; `prelude::*` → listes explicites (surface `bevy_ecs_ldtk` lisible).

### Acceptation
Porte commune ; captures `cmp` (barreaux de fenêtre, étiquettes d'étage, cavernes).

---

## E11b — `map_ldtk` : instantané pur des niveaux (`dette-d69-e11b-floor-levels`)

Estimation 2 j. Prérequis : E11a, E3. Risque : instantané de `GlobalTransform` f32.

### À faire
Introduire `FloorLevels`/`LevelSpawnRequest` (données pures) remplies par l'app à `LdtkMapLoadingEvent` ; `FloorSlots` → composant `FloorWorld` sur les entités de niveau ; `floor_transition_system` (`GgrsSchedule`) et `spawn_level_walls`, `surface_grid_of_levels`, `add_room_component_to_ldtk_level`, `fill_cave_grid_on_load`, les 5 `spawn_*_when_map_loaded` lisent le DTO ; séparer la collecte (app) de `spawn_map_items` (sim) dans `plugin.rs`. Ordre d'enregistrement de `LdtkMapLoadingPlugin::build` (`plugin.rs:85-154`) conservé.

### Acceptation
Porte commune : **traces identiques** (preuve que l'instantané redonne les mêmes `Vec3`) ; `cargo test -p map_ldtk` ; p2p-stress ; scénarios `floors_*`, `cave_*`, `key_door_*`, `grid_*`.

---

## E11c — `map_ldtk_sim` et `bots` en simulation (`dette-d69-e11c-map-ldtk-sim`)

Estimation 2,5 j. Prérequis : E11b.

### À faire
Créer `crates/map_ldtk_sim` (≈ 3 700 lignes déplacées : `rooms`, `floors`, `cave`, `collider`, plugin sim, `local`, tables du `loader`, composants d'entités, `map_const`) dépendant de `game_sim`, `bevy` ECS seul ; `map_ldtk` (APP) dépend de `map_ldtk_sim` ; `bots` et `scenario/src/{events.rs,bin/alacod-sim.rs}` pointent `map_ldtk_sim` pour `CurrentRooms`, `door_near`, `key_lock_cost`, `sorted_rooms` ; `check-sim-boundary.sh` : `map_ldtk_sim` et **`bots`** en `SIM`. `game_sim` doit héberger ou recevoir `FogMap` (E1) et les helpers de `cave_assets` (chemins).

### Acceptation
Porte commune ; `make check_sim_boundary` : 16 crates SIM (+ `map_ldtk_sim`, `bots`) ; l'ordre d'`add_plugins` de `LdtkMapLoadingPlugin` prouvé par le dump (bloc `GgrsSchedule` inchangé).

---

## Suites hors périmètre

- D1 (rapport §9) : `scenario` strictement « simulation seule » (chargeur de contenu `std::fs` + génération sans `bevy_ecs_ldtk`) : non proposé.
- Retirer `crates/animation` et `crates/telemetry` de la classe `APP` historique : ils y sont déjà.
- Après E10 : mettre à jour `docs/plan-engine.md` §7 (frontière de portage console : `game_app` est ce qu'un port remplace).
