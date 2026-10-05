pub mod file;

use bevy::prelude::*;
use bevy_ecs_ldtk::assets::{LdtkProjectLoader, LdtkProjectLoaderSettings};
use bevy_ecs_ldtk::prelude::*;

use bevy_fixed::rng::RollbackRng;
use std::sync::{Arc, Mutex};

use map::generation::config::{MapGenerationConfig, MapGenerationMode};
use map::generation::map_generation;

use super::generation::{from_map, GeneratedMap};
use crate::game::floors::{FloorPlan, FloorWorld};
use content::registry::{cave_designation, Registry};
use std::collections::BTreeMap;
use world::CaveConfig;

/// T1.6 : cavernes de la partie par emplacement de monde (`FloorWorld`, 0 pour la carte
/// unique), avec leur graine. Hors rollback, figée au chargement comme `FloorPlan` ; vide si
/// aucune carte n'est une caverne.
#[derive(Resource, Debug, Clone, Default)]
pub struct CaveSlots(pub BTreeMap<usize, (i32, CaveConfig)>);

/// Table des surfaces (T1.7) depuis le registre de contenu : `intgrid_value` → définition.
pub fn surface_table(registry: Option<&Registry>) -> world::SurfaceTable {
    let mut table = world::SurfaceTable::default();
    for surface in registry.iter().flat_map(|r| r.surfaces.values()) {
        table.0.insert(
            surface.intgrid_value,
            world::SurfaceDef {
                name: surface.id.to_string(),
                tags: surface.tags.clone(),
                move_speed: surface.move_speed,
                acceleration: surface.acceleration,
            },
        );
    }
    table
}

/// Config de **chargement** d'une carte de la partie : une désignation `cave:<id>`
/// (`docs/conventions.md` §21) devient le chemin d'asset propre à la caverne
/// (`cave://<dossier>/<id>.ldtk`, servi avec le gabarit du dossier :
/// `game::cave_assets`) et le mode `Cave(config)` ; toute autre carte garde `base.mode`.
/// Un chemin par caverne : avec le gabarit pour chemin commun, l'`AssetServer` rendait la
/// première caverne chargée à tous les étages d'une séquence `Floors`. La ressource `MapGenerationConfig`
/// garde la désignation d'origine (enregistrements rejouables).
pub fn resolve_map_config(
    base: &MapGenerationConfig,
    map: &str,
    registry: Option<&Registry>,
) -> (MapGenerationConfig, Option<CaveConfig>) {
    let cave = cave_designation(map).map(|id| {
        let entry = registry
            .and_then(|r| r.caves.get(&id))
            .unwrap_or_else(|| panic!("caverne « {id} » inconnue (voir `alacod lint`)"));
        (id, entry)
    });
    let config = MapGenerationConfig {
        map_path: cave.as_ref().map_or_else(
            || map.to_string(),
            |(id, c)| game::cave_assets::cave_asset_path(&c.template, id.as_str()),
        ),
        seed: base.seed,
        max_width: base.max_width,
        max_heigth: base.max_heigth,
        max_room: base.max_room,
        mode: cave.as_ref().map_or_else(
            || base.mode.clone(),
            |(_, c)| MapGenerationMode::Cave(c.config.clone()),
        ),
    };
    (config, cave.map(|(_, c)| c.config.clone()))
}

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

            // T1.6 : une caverne réécrit le niveau unique du gabarit, sans assemblage de salles
            if let MapGenerationMode::Cave(cave) = &config.mode {
                let id = game::cave_assets::cave_id_of_asset_path(&config.map_path)
                    .unwrap_or_default();
                return crate::generation::cave::build_cave_ldtk(&map_json, &id, config.seed, cave);
            }

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
    registry: Option<Res<Registry>>,
) {
    // T1.7 : table des surfaces du jeu (valeur IntGrid → définition), hors rollback
    commands.insert_resource(surface_table(registry.as_deref()));
    let mut caves = CaveSlots::default();
    let Some(plan) = plan else {
        let (load, cave) = resolve_map_config(&config, &config.map_path, registry.as_deref());
        if let Some(cave) = cave {
            caves.0.insert(0, (config.seed, cave));
        }
        let world = load_map(&mut commands, &asset_server, &settings, &load);
        commands.entity(world).insert(FloorWorld(0));
        commands.insert_resource(caves);
        return;
    };
    for slot in plan.distinct_slots() {
        let (floor_config, cave) =
            resolve_map_config(&config, &plan.levels[slot], registry.as_deref());
        if let Some(cave) = cave {
            caves.0.insert(slot, (config.seed, cave));
        }
        let world = load_map_snapshot(&mut commands, &asset_server, &floor_config);
        commands.entity(world).insert(FloorWorld(slot));
    }
    commands.insert_resource(caves);
}
