pub mod file;

use bevy::prelude::*;
use bevy_ecs_ldtk::assets::{LdtkProjectLoader, LdtkProjectLoaderSettings};
use bevy_ecs_ldtk::prelude::*;

use bevy_fixed::rng::RollbackRng;
use once_cell::sync::Lazy;

use map::generation::config::MapGenerationConfig;
use map::generation::map_generation;

use super::generation::{from_map, GeneratedMap};

static mut CONFIG: Lazy<MapGenerationConfig> = Lazy::new(MapGenerationConfig::default);

fn set_global_config(config: &MapGenerationConfig) {
    unsafe {
        //let rf = Lazy::force_mut(&mut CONFIG);
        CONFIG.seed = config.seed;
        CONFIG.max_width = config.max_width;
        CONFIG.max_heigth = config.max_heigth;
        CONFIG.mode = config.mode;
        CONFIG.map_path = config.map_path.clone();
    }
}

pub fn get_asset_loader_generation() -> LdtkProjectLoader {
    LdtkProjectLoader {
        callback: Some(Box::new(|map_json, config| {
            let config: MapGenerationConfig =
                serde_json::from_value(serde_json::Value::Object(config))
                    .expect("Failed to convert value to struct");

            let context = from_map(&map_json, config);
            let mut generator = GeneratedMap::create(map_json);

            map_generation(context, &mut generator).unwrap();

            generator.get_generated_map()
        })),
    }
}

pub fn reload_map(asset_server: &Res<AssetServer>, config: &MapGenerationConfig) {
    set_global_config(config);
    asset_server.reload(config.map_path.clone());
}

pub fn load_map(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    config: &MapGenerationConfig,
) {
    set_global_config(config);

    let ldtk_handle: LdtkProjectHandle = asset_server
        .load_builder()
        .with_settings(|s: &mut LdtkProjectLoaderSettings| unsafe {
            s.data = serde_json::to_value(&*CONFIG)
                .expect("Failed to convert struct to value")
                .as_object()
                .expect("Failed to convert value to object")
                .clone();
        })
        .load(config.map_path.clone())
        .into();

    let level_set = LevelSet::default();

    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        level_set,
        ..Default::default()
    });
}

pub fn setup_generated_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<MapGenerationConfig>,
) {
    load_map(&mut commands, &asset_server, config.as_ref())
}
