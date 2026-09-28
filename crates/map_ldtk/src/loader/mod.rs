pub mod file;

use bevy::prelude::*;
use bevy_ecs_ldtk::assets::{LdtkProjectLoader, LdtkProjectLoaderSettings};
use bevy_ecs_ldtk::prelude::*;

use bevy_fixed::rng::RollbackRng;
use std::sync::{Arc, Mutex};

use map::generation::config::MapGenerationConfig;
use map::generation::map_generation;

use super::generation::{from_map, GeneratedMap};

/// Config de génération transmise au loader LDtk (sérialisée dans ses settings).
///
/// Propre à chaque app (plusieurs parties peuvent tourner dans un même processus,
/// ex. les tests de scénarios). La closure de settings du chargement en garde une
/// copie : un `reload` relit la config courante.
#[derive(Resource, Clone, Default)]
pub struct MapLoaderSettings(Arc<Mutex<serde_json::Map<String, serde_json::Value>>>);

impl MapLoaderSettings {
    fn set(&self, config: &MapGenerationConfig) {
        *self.0.lock().unwrap() = serde_json::to_value(config)
            .expect("Failed to convert struct to value")
            .as_object()
            .expect("Failed to convert value to object")
            .clone();
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

pub fn reload_map(
    asset_server: &Res<AssetServer>,
    settings: &MapLoaderSettings,
    config: &MapGenerationConfig,
) {
    settings.set(config);
    asset_server.reload(config.map_path.clone());
}

pub fn load_map(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    settings: &MapLoaderSettings,
    config: &MapGenerationConfig,
) {
    settings.set(config);

    let shared = settings.clone();
    let ldtk_handle: LdtkProjectHandle = asset_server
        .load_builder()
        .with_settings(move |s: &mut LdtkProjectLoaderSettings| {
            s.data = shared.0.lock().unwrap().clone();
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
    settings: Res<MapLoaderSettings>,
    config: Res<MapGenerationConfig>,
) {
    load_map(&mut commands, &asset_server, &settings, config.as_ref())
}
