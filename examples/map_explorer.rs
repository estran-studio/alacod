

use animation::SpriteSheetConfig;
use bevy::{color::palettes::{css::TURQUOISE, tailwind::{ORANGE_300, PURPLE_300}}, platform::collections::HashMap, prelude::*};
use bevy_fixed::fixed_math;
use game::{
    args::BaseArgsPlugin, character::{config::CharacterConfig, enemy::spawning::EnemySpawnerState, player::create::create_player}, collider::{spawn_test_wall, CollisionSettings}, core::{AppState, CoreSetupConfig, CoreSetupPlugin}, global_asset::GlobalAsset, jjrs::{GggrsSessionConfiguration, GggrsSessionConfigurationState, GgrsSessionBuilding}, waves::{WaveDebugEnabled, WaveModeEnabled}, weapons::{melee::MeleeWeaponsConfig, WeaponsConfig}
};
use map::{game::entity::map::{enemy_spawn::EnemySpawnerComponent, player_spawn::PlayerSpawnConfig}, generation::{config::MapGenerationConfig, position}};
use map_ldtk::{game::plugin::LdtkMapLoadingEvent, plugins::LdtkRoguePlugin};
use utils::net_id::GgrsNetIdFactory;

fn main() {
    let game_config = CoreSetupConfig {
        app_name: "zrl-character_tester".into(),
    };

    let core_plugin = CoreSetupPlugin(game_config);

    App::new()
        // Configure the bevy default plugins from our core_plugin configuration
        // if you don't need special overwrite
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        // Core systems and components
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Enable wave-based spawning mode (CoD Zombies style)
        .insert_resource(WaveModeEnabled(true))
        // Enable wave debug UI (toggle with F3)
        .insert_resource(WaveDebugEnabled(true))
        .add_systems(OnEnter(AppState::LobbyLocal), system_configure_map)
        .add_systems(OnEnter(AppState::LobbyOnline), system_configure_map)
        .add_systems(Update, (
            system_wait_for_map_loaded.run_if(on_message::<LdtkMapLoadingEvent>),
        ))
        .run();
}

fn system_configure_map(
    mut commands: Commands,

    mut ggrs_state: ResMut<GggrsSessionConfigurationState>,
) {
    let config = get_config("exemples/test_map.ldtk".into(), 123456);
    commands.insert_resource(config);
    ggrs_state.ready = true;
}


// Wait for spawner to be loaded and create the players
fn system_wait_for_map_loaded(
    mut commands: Commands,

    collision_settings: Res<CollisionSettings>,
    global_assets: Res<GlobalAsset>,
    character_asset: Res<Assets<CharacterConfig>>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<Assets<MeleeWeaponsConfig>>,

    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    sprint_sheet_assets: Res<Assets<SpriteSheetConfig>>,

    mut id_provider: ResMut<GgrsNetIdFactory>,

    // Get my players
    ggrs_session_building: Res<GgrsSessionBuilding>,

    player_spawn: Query<(Entity, &GlobalTransform, &PlayerSpawnConfig)>,
) {


    let spawn: HashMap<usize, _> = player_spawn.iter().map(|s| (s.2.index, s) ).collect();

    info!("Map is loaded with all rollback entity {}", spawn.len());

    if spawn.len() == 0 {
        return;
    }

    for ggrs_player in ggrs_session_building.players.iter() {
        let i = ggrs_player.handle;
        let is_local = ggrs_player.is_local;
        let name = ggrs_player.name.clone();
        let pubkey = ggrs_player.pubkey.clone();

        let (_, transform, _) = spawn.get(&i).unwrap();

        if transform.translation().x == 0.0 && transform.translation().y == 0.0 {
            return;
        }

        println!("Found my spawn at {:?} ", transform.translation());

        create_player(
            &mut commands,
            &global_assets,
            &weapons_asset,
            &melee_weapons_asset,
            &character_asset,
            &collision_settings,
            &asset_server,
            &mut texture_atlas_layouts,
            &sprint_sheet_assets,
            fixed_math::vec3_to_fixed(transform.translation()),
            is_local,
            i,
            name,
            pubkey,
            &mut id_provider,
        );
    }
}

fn get_config(map_path: String, seed: i32) -> MapGenerationConfig {

    MapGenerationConfig {
        seed,
        map_path,
        max_width: 1000,
        max_heigth: 1000,
        ..Default::default()
    }
}
