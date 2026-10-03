pub mod file;

use bevy::prelude::*;
use bevy_ecs_ldtk::assets::{LdtkProjectLoader, LdtkProjectLoaderSettings};
use bevy_ecs_ldtk::prelude::*;

use bevy_fixed::rng::RollbackRng;
use std::sync::{Arc, Mutex};

use map::generation::config::MapGenerationConfig;
use map::generation::map_generation;

use super::generation::{from_map, GeneratedMap};
use crate::game::floors::{FloorPlan, FloorWorld};

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
) -> Entity {
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

    commands
        .spawn(LdtkWorldBundle {
            ldtk_handle,
            level_set,
            ..Default::default()
        })
        .id()
}

/// Mode `Floors` (T1.8) : comme [`load_map`], mais la config de génération est figée dans
/// la closure de settings de *ce* chargement (copie, pas l'`Arc` partagé de
/// [`MapLoaderSettings`]) : plusieurs cartes se chargent dans la même frame, chacune avec sa
/// propre config, quel que soit le moment où le chargeur lit ses settings.
pub fn load_map_snapshot(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    config: &MapGenerationConfig,
) -> Entity {
    let data = serde_json::to_value(config)
        .expect("Failed to convert struct to value")
        .as_object()
        .expect("Failed to convert value to object")
        .clone();
    let ldtk_handle: LdtkProjectHandle = asset_server
        .load_builder()
        .with_settings(move |s: &mut LdtkProjectLoaderSettings| {
            s.data = data.clone();
        })
        .load(config.map_path.clone())
        .into();

    commands
        .spawn(LdtkWorldBundle {
            ldtk_handle,
            level_set: LevelSet::default(),
            ..Default::default()
        })
        .id()
}

/// Charge la carte de la partie : une seule (`MapGenerationConfig`), ou, en mode `Floors`
/// (T1.8, [`FloorPlan`] présent), un monde LDtk par carte distincte de la séquence. Chaque
/// monde porte [`FloorWorld`] (son emplacement dans la séquence), `FloorWorld(0)` pour la
/// carte unique : les systèmes de chargement filtrent par emplacement.
pub fn setup_generated_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<MapLoaderSettings>,
    config: Res<MapGenerationConfig>,
    plan: Option<Res<FloorPlan>>,
) {
    let Some(plan) = plan else {
        let world = load_map(&mut commands, &asset_server, &settings, config.as_ref());
        commands.entity(world).insert(FloorWorld(0));
        return;
    };
    for slot in plan.distinct_slots() {
        let floor_config = MapGenerationConfig {
            map_path: plan.levels[slot].clone(),
            seed: config.seed,
            max_width: config.max_width,
            max_heigth: config.max_heigth,
            max_room: config.max_room,
            mode: config.mode,
        };
        let world = load_map_snapshot(&mut commands, &asset_server, &floor_config);
        commands.entity(world).insert(FloorWorld(slot));
    }
}
