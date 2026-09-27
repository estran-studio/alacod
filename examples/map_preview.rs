use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use bevy_ecs_ldtk::prelude::*;

use game::core::AppState;
use map::{game::entity::map::door::DoorComponent, generation::config::MapGenerationConfig};
use map_ldtk::{
    game::utility::load_levels_if_not_present, loader::{get_asset_loader_generation, reload_map, setup_generated_map, MapLoaderSettings}, plugins::LdtkRoguePlugin
};
use utils::{
    camera::tod::{move_camera, setup_camera},
    web::WebPlugin,
};

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let map_generation_config = get_config();
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "zrl-map-preview".to_string(),
            resolution: WindowResolution::new(800, 600),

            resizable: true,
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy-canvas".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    #[cfg(target_arch = "wasm32")]
                    file_path: format!("{}/assets", env!("APP_VERSION")),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(window_plugin),
        )
        .init_state::<AppState>()
        .insert_state(AppState::GameLoading)
        .add_systems(Startup, (setup_camera))
        .add_plugins(LdtkRoguePlugin)
        .insert_resource(map_generation_config)
        .add_systems(
            Update,
            (
                move_camera,
                keyinput,
                toggle_door_visibility,
            ),
        )
        .add_plugins(WebPlugin {})
        .run();
}

fn toggle_door_visibility(
    input: Res<ButtonInput<KeyCode>>, // Use ButtonInput<KeyCode> for newer Bevy versions
    mut door_query: Query<&mut Visibility, With<DoorComponent>>,
) {
    // Check if the desired key is pressed (e.g., H key)
    if input.just_pressed(KeyCode::KeyH) {
        // Toggle visibility for all doors
        for mut visibility in door_query.iter_mut() {
            // Toggle between Visible and Hidden
            *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                Visibility::Hidden => Visibility::Visible,
                Visibility::Inherited => Visibility::Hidden, // Handle inherited case
            };
        }
    }
}

fn keyinput(
    input: Res<ButtonInput<KeyCode>>,
    level_query: Query<Entity, With<LevelIid>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loader_settings: Res<MapLoaderSettings>,
    mut config: ResMut<MapGenerationConfig>,
    mut level_set: Query<&mut LevelSet>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        for level_entity in &level_query {
            // TODO: Use despawn_recursive when bevy_hierarchy is available
            commands.entity(level_entity).despawn();
        }
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        config.seed = since_the_epoch.as_secs() as i32;
        reload_map(&asset_server, &loader_settings, config.as_ref());
        for mut level_set in level_set.iter_mut() {
            level_set.iids.clear();
        }
    }
}

fn get_config() -> MapGenerationConfig {
    let args: Vec<String> = env::args().collect();

    let map_path = if args.len() < 2 {
        "exemples/test_map.ldtk".to_string()
    } else {
        args.get(1).unwrap().clone()
    };

    let seed = if args.len() < 3 {
        12345
    } else {
        args.get(2).unwrap().parse::<i32>().unwrap()
    };

    MapGenerationConfig {
        seed,
        map_path,
        max_width: 1000,
        max_heigth: 1000,
        ..Default::default()
    }
}
