# Annexe — lectures `game::` hors du crate, avec crate cible

> Annexe de `docs/taches/rapports/dette-d69-plan-etape2.md` §8. Mesuré le 2026-10-18 sur `8633a560` par
> lecture des `use game::…;` et des chemins `game::…` (hors commentaires) ; un chemin développé d’un
> `use game::{a::X, b::Y}` compte une occurrence par feuille. **Total bots + scenario : 582 occurrences**
> (425 lignes contenant `game::` : `bots/src` 56, `scenario/src` 202, `scenario/tests` 167 ; le « 262 » du
> rapport de l’étape 1 comptait `bots/src` + `scenario/src`, 258 aujourd’hui). `b` = `bots`, `s` = `scenario`,
> `m` = `map_ldtk`, `g` = `games/*`, `e` = `examples`, `t` = `templates/starter`. Les chemins `game::local`,
> `game::rooms`, `game::entity`, `game::plugin`, `game::utility` rencontrés dans `bots`/`scenario`/`games` sont ceux de
> `map_ldtk::game::…` (autre crate), exclus ici.

Destination : **sim** = reste lisible depuis `game_sim` au même chemin de module (E10 renomme seulement le crate) ;
**app** = devient `game_app::<chemin>` (E9) ; **à migrer** = le chemin disparaît de la simulation (enveloppe `Asset`
ou `Handle`), le lecteur doit passer au registre de simulation (étape indiquée).

| chemin sous `game::` | b | s | m | g | e | t | destination | note |
|---|---:|---:|---:|---:|---:|---:|---|---|
| `args::B` |  |  |  | 5 | 1 | 1 | app |  |
| `args::LaunchSeed` |  |  |  | 3 |  |  | sim |  |
| `args::GameArgs` |  | 1 |  |  |  |  | sim |  |
| `args::GameArgsPlugin` |  | 1 |  |  |  |  | sim |  |
| `args::check_distance_from_env` |  | 1 |  |  |  |  | sim |  |
| `balance::ResolvedBalance` | 1 | 1 | 3 |  | 1 |  | sim |  |
| `balance::resolve_balance_system` |  |  |  |  | 1 |  | sim |  |
| `bombs::explosion_system` |  | 4 |  |  |  |  | sim |  |
| `bombs::RockBroken` |  |  | 1 |  |  |  | sim |  |
| `bombs::bomb_fuse_system` |  |  | 1 |  |  |  | sim |  |
| `bombs::explosion_terrain_system` |  |  | 1 |  |  |  | sim |  |
| `bombs::rock_loot_key` |  |  | 1 |  |  |  | sim |  |
| `bombs::room_of` |  |  | 1 |  |  |  | sim |  |
| `boss::ReinforcementRequest` |  |  | 1 |  |  |  | sim |  |
| `boss::boss_defeated_system` |  |  | 1 |  |  |  | sim |  |
| `boss::boss_phase_system` |  |  | 1 |  |  |  | sim |  |
| `camera::CameraFollowOverride` |  | 1 |  |  |  |  | sim | `CameraFollowOverride` → `ui::hud_model` (E1) |
| `camera::ui::CameraDebugUiEnabled` |  | 1 |  |  |  |  | app |  |
| `cave_assets::cave_asset_path` |  |  | 1 |  |  |  | app |  |
| `cave_assets::cave_id_of_asset_path` |  |  | 1 |  |  |  | app |  |
| `cave_assets::floor_asset_path` |  |  | 1 |  |  |  | app |  |
| `character::player::Player` | 3 | 26 | 3 |  |  |  | sim |  |
| `character::enemy::Enemy` | 3 | 19 | 2 |  |  |  | sim |  |
| `character::health::Health` | 3 | 17 |  |  |  |  | sim |  |
| `character::health::HitCount` |  | 8 |  |  |  |  | sim |  |
| `character::config::CharacterConfig` | 1 | 3 | 2 |  | 1 |  | sim |  |
| `character::health::rollback_resolve_damage_events` |  | 6 |  |  |  |  | sim |  |
| `character::enemy::ai::MonsterState` | 1 | 4 |  |  |  |  | sim |  |
| `character::enemy::ai::Obstacle` |  | 4 | 1 |  |  |  | sim |  |
| `character::enemy::ai::navigation::AgentBody` | 4 | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::navigation::FlowFieldCache` |  | 1 | 4 |  |  |  | sim |  |
| `character::variant::Variant` |  | 5 |  |  |  |  | sim |  |
| `character::enemy::ai::FlowFieldCache` |  | 4 |  |  |  |  | sim |  |
| `character::enemy::ai::GridPos` |  | 4 |  |  |  |  | sim |  |
| `character::enemy::ai::navigation::GridPos` | 2 |  | 2 |  |  |  | sim |  |
| `character::enemy::ai::navigation::MOVEMENT_FLOW_KEY` |  | 4 |  |  |  |  | sim |  |
| `character::enemy::ai::rules::charge_damage_translate_system` |  | 4 |  |  |  |  | sim |  |
| `character::health::Death` |  | 1 | 3 |  |  |  | sim |  |
| `character::player::input::BoxInput` | 4 |  |  |  |  |  | sim |  |
| `character::player::jjrs::PeerConfig` | 2 | 2 |  |  |  |  | sim |  |
| `character::enemy::ai::EnemyAiConfig` | 1 | 2 |  |  |  |  | sim |  |
| `character::player::input::INPUT_INTERACTION` | 3 |  |  |  |  |  | sim |  |
| `character::player::input::INPUT_RIGHT` | 3 |  |  |  |  |  | sim |  |
| `character::player::input::INPUT_UP` | 3 |  |  |  |  |  | sim |  |
| `character::config::CharacterConfigHandles` | 1 | 1 |  |  |  |  | sim | **à migrer** : `CharacterConfigKey` (E3a) |
| `character::config::CharacterTest` |  | 2 |  |  |  |  | sim |  |
| `character::enemy::ai::EnemyTarget` |  | 2 |  |  |  |  | sim |  |
| `character::enemy::ai::NavProfile` |  | 2 |  |  |  |  | sim |  |
| `character::enemy::ai::navigation::FlowField` | 2 |  |  |  |  |  | sim |  |
| `character::enemy::ai::state::EnemyAiConfig` |  | 2 |  |  |  |  | sim |  |
| `character::enemy::ai::state::EnemyBehaviors` |  | 2 |  |  |  |  | sim |  |
| `character::enemy::create::spawn_enemy` |  |  | 2 |  |  |  | sim |  |
| `character::enemy::spawning::EnemySpawnerState` |  |  | 1 |  | 1 |  | sim |  |
| `character::movement::SprintState` |  | 2 |  |  |  |  | sim |  |
| `character::player::create::create_player` |  |  | 1 |  | 1 |  | sim |  |
| `character::player::input::INPUT_DOWN` | 2 |  |  |  |  |  | sim |  |
| `character::player::input::INPUT_LEFT` | 2 |  |  |  |  |  | sim |  |
| `character::player::input::INPUT_RELOAD` | 2 |  |  |  |  |  | sim |  |
| `character::player::input::InputSource` |  | 2 |  |  |  |  | sim |  |
| `character::variant::VariantContext` |  |  | 2 |  |  |  | sim |  |
| `character::dash::D` |  | 1 |  |  |  |  | sim |  |
| `character::dash::DashOverride` |  | 1 |  |  |  |  | sim |  |
| `character::dash::DashState` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::AttackTarget` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::Obstacle::new` |  |  | 1 |  |  |  | sim |  |
| `character::enemy::ai::ObstacleType::Barricade` |  |  | 1 |  |  |  | sim |  |
| `character::enemy::ai::ObstacleType::Wall` |  |  | 1 |  |  |  | sim |  |
| `character::enemy::ai::navigation::AgentBody::from_collider` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::navigation::AgentSize::Small` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::navigation::GRID_CELL_SIZE` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::navigation::NavKey` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::navigation::NavProfile::Ground` |  |  | 1 |  |  |  | sim |  |
| `character::enemy::ai::rules::current_rule` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::state::BehaviorRuntime` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::state::EnemyTarget` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::state::MonsterState` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::state::MovementType::Flying` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::ai::state::RangedAttackState` |  | 1 |  |  |  |  | sim |  |
| `character::enemy::spawning::enemy_spawn_from_spawners_system` |  | 1 |  |  |  |  | sim |  |
| `character::health::new_health` |  |  | 1 |  |  |  | sim |  |
| `character::health::rollback_apply_death` |  |  | 1 |  |  |  | sim |  |
| `character::movement::DashConfig` |  | 1 |  |  |  |  | sim |  |
| `character::movement::Velocity` |  | 1 |  |  |  |  | sim |  |
| `character::player::input::INPUT_BLANK` | 1 |  |  |  |  |  | sim |  |
| `character::player::input::INPUT_BOMB` | 1 |  |  |  |  |  | sim |  |
| `character::player::input::ScriptedInputs` |  | 1 |  |  |  |  | sim |  |
| `character::player::input::read_local_inputs` | 1 |  |  |  |  |  | sim |  |
| `character::skip_device_input` |  | 1 |  |  |  |  | sim | **supprimé** (E4) |
| `character::surface::feet_position` |  | 1 |  |  |  |  | sim |  |
| `clock::DifficultyReader` |  |  | 2 |  |  |  | sim |  |
| `clock::scale` |  |  | 2 |  |  |  | sim |  |
| `clock::ClocksOverride` |  | 1 |  |  |  |  | sim |  |
| `clock::DifficultyOverride` |  | 1 |  |  |  |  | sim |  |
| `collider::Collider` | 4 | 9 | 3 |  |  |  | sim |  |
| `collider::Wall` | 2 | 8 | 3 |  |  |  | sim |  |
| `collider::ColliderShape::Rectangle` |  |  | 6 |  |  |  | sim |  |
| `collider::CollisionSettings` |  |  | 5 |  | 1 |  | sim |  |
| `collider::ColliderShape` | 1 | 3 | 1 |  |  |  | sim |  |
| `collider::CollisionLayer` |  |  | 4 |  |  |  | sim |  |
| `collider::Window` | 2 |  | 1 |  |  |  | sim |  |
| `collider::is_colliding` |  | 2 | 1 |  |  |  | sim |  |
| `collider::ColliderShape::Circle` |  |  | 2 |  |  |  | sim |  |
| `collider::spawn_test_wall` |  |  | 1 |  | 1 |  | app | `collider::test_wall` (E4) |
| `collision_grid::collider_aabb` |  | 1 |  |  |  |  | sim |  |
| `content_hot_reload::GameRoot` |  |  |  | 5 | 1 | 1 | app |  |
| `core::AppState` |  | 5 | 2 |  | 2 |  | sim | → `session` (E2) |
| `core::CoreSetupConfig` |  | 1 |  | 5 | 1 | 1 | sim | → `session` (E2) |
| `core::CoreSetupPlugin` |  | 1 |  | 5 | 1 | 1 | app |  |
| `core::AppState::InGame` | 1 | 1 |  |  |  |  | sim | → `session` (E2) |
| `core::AppState::LobbyLocal` |  |  |  | 1 |  |  | sim | → `session` (E2) |
| `core::OnlineState` |  |  | 1 |  |  |  | sim | → `session` (E2) |
| `core::OnlineState::Online` |  |  | 1 |  |  |  | sim | → `session` (E2) |
| `core::SIM_FPS` |  | 1 |  |  |  |  | sim | → `session` (E2) |
| `core::debug_ui_enabled` |  |  |  | 1 |  |  | app |  |
| `curses::CurseTable` |  |  | 2 |  |  |  | sim |  |
| `curses::CurseTable::for_mode` |  |  | 1 |  |  |  | sim |  |
| `curses::curse_map_seed` |  |  | 1 |  |  |  | sim |  |
| `curses::floor_rooms_for_slot` |  |  | 1 |  |  |  | sim |  |
| `economy::PerkMachine` | 1 |  | 1 |  |  |  | sim |  |
| `economy::EconomyConfig` |  |  | 1 |  |  |  | sim |  |
| `effects_runtime::Gauges` |  | 2 |  |  |  |  | sim |  |
| `effects_runtime::EffectState::default` |  | 1 |  |  |  |  | sim |  |
| `effects_runtime::Effects` |  | 1 |  |  |  |  | sim |  |
| `effects_runtime::apply_effects_system` |  | 1 |  |  |  |  | sim |  |
| `effects_v3::HitLanded` |  | 1 |  |  |  |  | sim |  |
| `feedback::FeedbackKind` |  | 1 |  |  |  |  | sim | `feedback_log` (E5b) |
| `feedback::FeedbackLog` |  | 1 |  |  |  |  | sim | `feedback_log` (E5b) |
| `feedback::pattern_label` |  | 1 |  |  |  |  | sim | `feedback_log` (E5b) |
| `feedback_v3::cue_label` |  | 1 |  |  |  |  | sim |  |
| `gen_test::ChampionTest` |  | 2 |  |  |  |  | sim |  |
| `gen_test::FamiliarTest` |  | 2 |  |  |  |  | sim |  |
| `gen_test::ItemTest` |  | 2 |  |  |  |  | sim |  |
| `gen_test::TearsTarget` |  | 2 |  |  |  |  | sim |  |
| `gen_test::TearsTest` |  | 2 |  |  |  |  | sim |  |
| `gen_test::BossFightTest` |  | 1 |  |  |  |  | sim |  |
| `gen_test::ItemFile` |  | 1 |  |  |  |  | sim |  |
| `gen_test::RoomFile` |  | 1 |  |  |  |  | sim |  |
| `gen_test::RoomTest` |  | 1 |  |  |  |  | sim |  |
| `gen_test::TearsHits` |  | 1 |  |  |  |  | sim |  |
| `global_asset::GlobalAsset` | 1 | 3 | 3 |  | 1 |  | app | **à migrer** : ressource `ContentData`/registres (E3) |
| `hub::HubSession` |  | 7 | 2 | 1 |  |  | sim |  |
| `hub::HubState` |  | 4 |  |  |  |  | sim |  |
| `hub::HubServices` |  |  | 3 |  |  |  | sim |  |
| `hub::HubExit` |  | 1 |  |  |  |  | sim |  |
| `hub::HubStation` |  |  | 1 |  |  |  | sim |  |
| `hub::HubStay` |  | 1 |  |  |  |  | sim |  |
| `hub::apply_to_map_config` |  |  | 1 |  |  |  | sim |  |
| `hub::hub_station_system` |  |  | 1 |  |  |  | sim |  |
| `hub::resolve_session_mode` |  |  | 1 |  |  |  | sim |  |
| `interaction::Interactable` | 4 | 1 | 5 |  |  |  | sim |  |
| `interaction::InteractionType` | 1 |  | 2 |  |  |  | sim |  |
| `interaction::WindowHealthBar` |  |  | 3 |  |  |  | sim |  |
| `interaction::InteractionEvent` |  |  | 1 |  |  |  | sim |  |
| `interaction::InteractionType::Door` |  |  | 1 |  |  |  | sim |  |
| `interaction::InteractionType::Table` |  |  | 1 |  |  |  | sim |  |
| `interaction::InteractionType::Window` |  |  | 1 |  |  |  | sim |  |
| `items::item_consumable_pickup_system` |  | 3 |  |  |  |  | sim |  |
| `items::BlankState` | 1 | 1 |  |  |  |  | sim |  |
| `items::spawn_item_pickup` |  | 1 | 1 |  |  |  | sim |  |
| `items::FloorStartConsumables` |  |  | 1 |  |  |  | sim |  |
| `items::FloorStartConsumables::default` |  |  | 1 |  |  |  | sim |  |
| `jjrs::GggrsSessionConfigurationState` |  |  | 1 |  | 1 |  | sim |  |
| `jjrs::GgrsSessionBuilding` |  |  | 1 |  | 1 |  | sim |  |
| `jjrs::GggrsSessionConfiguration` |  |  |  |  | 1 |  | sim |  |
| `jjrs::PlayerConfig` |  | 1 |  |  |  |  | sim |  |
| `jjrs::p2p::MAX_PREDICTION_WINDOW` |  | 1 |  |  |  |  | sim |  |
| `legibility::LabelsEnabled` |  | 1 |  |  |  |  | app |  |
| `legibility::LegendOpen` |  | 1 |  |  |  |  | app |  |
| `legibility::WorldLabel` |  |  | 1 |  |  |  | app |  |
| `loot::seed_context` |  | 1 | 5 |  |  |  | sim |  |
| `loot::SlotContext` |  | 1 | 4 |  |  |  | sim |  |
| `loot::FloorLootTables` |  | 1 | 3 |  |  |  | sim |  |
| `loot::KEY_ITEM` | 2 |  | 2 |  |  |  | sim |  |
| `loot::assign_slot` |  | 1 | 3 |  |  |  | sim |  |
| `loot::chest_table_id` |  | 1 | 1 |  |  |  | sim |  |
| `loot::spawn_chest` |  | 1 | 1 |  |  |  | sim |  |
| `loot::BOMB_ITEM` | 1 |  |  |  |  |  | sim |  |
| `loot::FloorLootTables::default` |  |  | 1 |  |  |  | sim |  |
| `loot::LootOverrides` |  | 1 |  |  |  |  | sim |  |
| `loot::spawn_drawn_loot` |  |  | 1 |  |  |  | sim |  |
| `powerups::PowerUpPickup` | 1 | 2 |  |  |  |  | sim |  |
| `powerups::PowerUpPickedUp` |  | 1 |  |  |  |  | sim |  |
| `powerups::PowerUpsConfig` |  | 1 |  |  |  |  | sim |  |
| `powerups::powerup_pickup_detect_system` |  | 1 |  |  |  |  | sim |  |
| `powerups::spawn_powerup_pickup` |  | 1 |  |  |  |  | sim |  |
| `profile::HOST_STATS_ENV` |  |  | 1 |  |  |  | sim |  |
| `profile::ProfileSettings` |  | 1 |  |  |  |  | sim |  |
| `profile::RUN_UNLOCKS_ENV` |  |  | 1 |  |  |  | sim |  |
| `profile::RunHistory` |  | 1 |  |  |  |  | sim |  |
| `profile::online_host_profile` |  |  | 1 |  |  |  | sim |  |
| `profile::purchase_unlock` |  | 1 |  |  |  |  | sim |  |
| `progression::Mutations` |  | 4 |  |  |  |  | sim |  |
| `progression::Level` |  | 3 |  |  |  |  | sim |  |
| `progression::MutationChoice` |  | 2 | 1 |  |  |  | sim |  |
| `progression::PendingMutations` |  | 1 | 1 |  |  |  | sim |  |
| `progression::ProgressionTable` |  | 1 | 1 |  |  |  | sim |  |
| `progression::ProgressionOverride` |  | 1 |  |  |  |  | sim |  |
| `progression::progression_system` |  | 1 |  |  |  |  | sim |  |
| `recording::InputRecorder` |  | 1 |  |  |  |  | sim |  |
| `recording::InputRecorder::native` |  | 1 |  |  |  |  | sim |  |
| `recording::RecordedSettings` |  | 1 |  |  |  |  | sim |  |
| `recording::record_local_inputs` | 1 |  |  |  |  |  | sim |  |
| `replay::EntityRef` |  | 17 |  |  |  |  | sim |  |
| `replay::BotProfile` | 4 | 6 |  |  |  |  | sim |  |
| `replay::Expectation` |  | 10 |  |  |  |  | sim |  |
| `replay::Scenario` |  | 10 |  |  |  |  | sim |  |
| `replay::PlayerScript` |  | 6 |  |  |  |  | sim |  |
| `replay::Button` |  | 4 |  |  |  |  | sim |  |
| `replay::Expectation::EntityCount` |  | 4 |  |  |  |  | sim |  |
| `replay::Expectation::Event` |  | 4 |  |  |  |  | sim |  |
| `replay::Expectation::Health` |  | 4 |  |  |  |  | sim |  |
| `replay::ExplorerPolicy` | 1 | 3 |  |  |  |  | sim |  |
| `replay::Segment` |  | 4 |  |  |  |  | sim |  |
| `replay::CharacterPlacement` |  | 3 |  |  |  |  | sim |  |
| `replay::EntityKind::Bullet` |  | 3 |  |  |  |  | sim |  |
| `replay::EntityKind::Enemy` |  | 3 |  |  |  |  | sim |  |
| `replay::BotProfile::Explorateur` |  | 2 |  |  |  |  | sim |  |
| `replay::DistanceTarget` |  | 2 |  |  |  |  | sim |  |
| `replay::EntityKind` |  | 2 |  |  |  |  | sim |  |
| `replay::EntityKind::Player` |  | 2 |  |  |  |  | sim |  |
| `replay::EntityRef::NetId` |  | 2 |  |  |  |  | sim |  |
| `replay::EntityRef::Placed` |  | 2 |  |  |  |  | sim |  |
| `replay::EntityRef::Target` |  | 2 |  |  |  |  | sim |  |
| `replay::Expectation::EntityHealth` |  | 2 |  |  |  |  | sim |  |
| `replay::Expectation::NoDamageBetween` |  | 2 |  |  |  |  | sim |  |
| `replay::Invariants` |  | 2 |  |  |  |  | sim |  |
| `replay::PowerUpPlacement` |  | 2 |  |  |  |  | sim |  |
| `replay::WaveOverride` |  | 2 |  |  |  |  | sim |  |
| `replay::WeaponOverride` |  | 2 |  |  |  |  | sim |  |
| `replay::BotProfile::Fonceur` |  | 1 |  |  |  |  | sim |  |
| `replay::ChestPlacement` |  | 1 |  |  |  |  | sim |  |
| `replay::Detour` | 1 |  |  |  |  |  | sim |  |
| `replay::DevilPolicy` | 1 |  |  |  |  |  | sim |  |
| `replay::DevilPolicy::IfHealthAbove` |  | 1 |  |  |  |  | sim |  |
| `replay::DevilPolicy::Never` |  | 1 |  |  |  |  | sim |  |
| `replay::DistanceTarget::Entity` |  | 1 |  |  |  |  | sim |  |
| `replay::DistanceTarget::Player` |  | 1 |  |  |  |  | sim |  |
| `replay::EntityKind::Barrel` |  | 1 |  |  |  |  | sim |  |
| `replay::EntityKind::Rock` |  | 1 |  |  |  |  | sim |  |
| `replay::EntityKind::Rollback` |  | 1 |  |  |  |  | sim |  |
| `replay::EntityKind::Table` |  | 1 |  |  |  |  | sim |  |
| `replay::Expectation::RunState` |  | 1 |  |  |  |  | sim |  |
| `replay::Expectation::RunSummary` |  | 1 |  |  |  |  | sim |  |
| `replay::HubSetup` |  | 1 |  |  |  |  | sim |  |
| `replay::ItemPlacement` |  | 1 |  |  |  |  | sim |  |
| `replay::ModifierSpec` |  | 1 |  |  |  |  | sim |  |
| `replay::RunStepExpectation` |  | 1 |  |  |  |  | sim |  |
| `replay::RunStepExpectation::Ended` |  | 1 |  |  |  |  | sim |  |
| `replay::ShelfPlacement` |  | 1 |  |  |  |  | sim |  |
| `replay::ShopOrder` | 1 |  |  |  |  |  | sim |  |
| `room_fog::FogMap` |  |  | 3 |  |  |  | sim | `Fog*` (E1) ; reste app |
| `room_fog::FogDoor` |  |  | 1 |  |  |  | sim | `Fog*` (E1) ; reste app |
| `room_fog::FogRoom` |  |  | 1 |  |  |  | sim | `Fog*` (E1) ; reste app |
| `room_fog::FogSettings` |  |  | 1 |  |  |  | sim | `Fog*` (E1) ; reste app |
| `run_state::RunRequest` |  | 4 |  |  |  |  | sim |  |
| `run_state::FloorsOverride` |  | 1 | 1 |  |  |  | sim |  |
| `run_state::finalize_run_summary_system` |  |  | 1 |  |  |  | sim |  |
| `run_state::floor_levels` |  |  | 1 |  |  |  | sim |  |
| `run_state::floors_consumables_per_floor` |  |  | 1 |  |  |  | sim |  |
| `run_state::floors_loot_tables` |  |  | 1 |  |  |  | sim |  |
| `run_state::floors_victory_at_end` |  |  | 1 |  |  |  | sim |  |
| `shop::spawn_shelf` |  | 1 | 1 |  |  |  | sim |  |
| `shop::SHELF_SPACING` |  |  | 1 |  |  |  | sim |  |
| `shop::ShopOverrides` |  | 1 |  |  |  |  | sim |  |
| `shop::assign_shelves` |  |  | 1 |  |  |  | sim |  |
| `shop::draw_shelves` |  |  | 1 |  |  |  | sim |  |
| `shop::shelf_interaction_system` |  |  | 1 |  |  |  | sim |  |
| `shop::shelf_price` |  |  | 1 |  |  |  | sim |  |
| `shop::spawn_shell_pickup` |  |  | 1 |  |  |  | sim |  |
| `state_trace::StateTraceRecorder` |  | 3 |  |  |  |  | sim |  |
| `state_trace::StateTraceRecorderPlugin` |  | 1 |  |  |  |  | sim |  |
| `system_set::RollbackSystemSet::CollisionDamage` |  | 6 |  |  |  |  | sim |  |
| `system_set::RollbackSystemSet::Effects` |  | 4 |  |  |  |  | sim |  |
| `system_set::RollbackSystemSet` |  | 2 | 1 |  |  |  | sim |  |
| `system_set::RollbackSystemSet::DeathManagement` |  | 1 |  |  |  |  | sim |  |
| `system_set::RollbackSystemSet::EnemySpawning` |  | 1 |  |  |  |  | sim |  |
| `ui::hud::HudConfig` |  |  | 1 |  |  |  | app |  |
| `ui::hud::HudConfigHandle` |  |  | 1 |  |  |  | app |  |
| `ui::hud::WallArt` |  |  | 1 |  |  |  | app |  |
| `ui::hud_model::HudModelSet` |  |  | 1 |  |  |  | sim | modèle pur (E5a) |
| `ui::hud_model::HudRoomBounds` |  |  | 1 |  |  |  | sim | modèle pur (E5a) |
| `ui::hud_model::HudRoomGeometry` |  |  | 1 |  |  |  | sim | modèle pur (E5a) |
| `ui::hud_model::HudSnapshot` |  | 1 |  |  |  |  | sim | modèle pur (E5a) |
| `ui::screens::ScreenScript` |  | 1 |  |  |  |  | sim | modèle pur (E5a) |
| `ui::screens::Screens` |  | 1 |  |  |  |  | sim | modèle pur (E5a) |
| `waves::WaveModeEnabled` |  | 1 |  | 5 | 1 | 1 | sim |  |
| `waves::WaveState` | 1 | 5 |  |  |  |  | sim |  |
| `waves::WaveDebugEnabled` |  | 1 |  | 1 | 1 |  | sim |  |
| `waves::WaveEnemy` |  | 1 |  |  |  |  | sim |  |
| `waves::WavePh` |  | 1 |  |  |  |  | sim |  |
| `waves::WavePhase` |  | 1 |  |  |  |  | sim |  |
| `waves::WavePhase::Spawning` |  | 1 |  |  |  |  | sim |  |
| `waves::WavePhase::WaveComplete` |  | 1 |  |  |  |  | sim |  |
| `waves::config::WaveConfig` |  | 1 |  |  |  |  | sim |  |
| `waves::systems::wave_spawning_system` |  | 1 |  |  |  |  | sim |  |
| `waves::systems::wave_state_machine_system` |  | 1 |  |  |  |  | sim |  |
| `weapons::Bullet` |  | 10 |  |  |  |  | sim |  |
| `weapons::WeaponInventory` | 2 | 7 |  |  |  |  | sim |  |
| `weapons::WeaponsConfig` | 1 | 3 | 2 |  | 1 |  | app | **à migrer** : `WeaponRegistry` (E3b) |
| `weapons::WeaponModesState` | 2 | 4 |  |  |  |  | sim |  |
| `weapons::WeaponState` | 2 | 4 |  |  |  |  | sim |  |
| `weapons::melee::MeleeWeaponsConfig` |  | 2 | 2 |  | 1 |  | app | **à migrer** : `WeaponRegistry` (E3b) |
| `weapons::WeaponPickup` | 2 | 2 |  |  |  |  | sim |  |
| `weapons::FiringMode` | 2 |  |  |  |  |  | sim |  |
| `weapons::Weapon` |  | 2 |  |  |  |  | sim |  |
| `weapons::WeaponTest` |  | 2 |  |  |  |  | sim |  |
| `weapons::melee::MeleeAttackState` |  | 2 |  |  |  |  | sim |  |
| `weapons::BulletType` | 1 |  |  |  |  |  | sim |  |
| `weapons::FiringModeConfig` | 1 |  |  |  |  |  | sim |  |
| `weapons::MagBulletConfig::Mag` |  | 1 |  |  |  |  | sim |  |
| `weapons::WeaponAsset` |  | 1 |  |  |  |  | sim |  |
| `weapons::default_mode_ammo_contribution` |  | 1 |  |  |  |  | sim |  |
| `weapons::melee::MeleeWeapon` |  | 1 |  |  |  |  | sim |  |
| `weapons::melee::MeleeWeaponAsset` |  | 1 |  |  |  |  | sim |  |
| `weapons::melee::self` |  | 1 |  |  |  |  | sim |  |
| `weapons::self` |  |  | 1 |  |  |  | sim |  |
| `weapons::spawn_weapon_for_player` |  | 1 |  |  |  |  | sim |  |
| `weapons::weapon_inventory_system` |  | 1 |  |  |  |  | app | `weapon_assets` (E5c) |
| `web::WebPlugin` |  |  |  |  | 1 |  | app |  |
| `world_sprites::SpriteRequest` |  |  | 1 |  |  |  | app |  |

## Totaux par crate lecteur et destination

| lecteur | → sim | → app | total |
|---|---:|---:|---:|
| `bots` | 89 | 2 | 91 |
| `scenario` | 478 | 13 | 491 |
| `map_ldtk` | 160 | 16 | 176 |
| `games` | 16 | 16 | 32 |
| `examples` | 14 | 8 | 22 |
| `templates` | 2 | 3 | 5 |
