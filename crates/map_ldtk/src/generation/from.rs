use bevy::log::debug;
use std::rc::Rc;

use bevy_ecs_ldtk::ldtk::{FieldValue, LayerInstance, LdtkJson, Level};
use bevy_ecs_ldtk::prelude::LdtkFields;

use map::generation::{
    config::MapGenerationConfig,
    context::{
        populate_level_connections, scan_height_side, scan_width_side, AvailableLevel, LevelType,
        MapGenerationContext, Side,
    },
    entity::{
        character_spawn::CharacterSpawnConfig,
        enemy_spawn::EnemySpawnConfig,
        location::{EntityLocation, EntityLocations},
        prop::PropConfig,
        soda_location::SodaLocationConfig,
        weapon_location::WeaponLocationConfig,
    },
    position::Position,
};

use crate::map_const;

fn get_level_field(level: &Level, name: &str) -> Option<FieldValue> {
    level
        .field_instances
        .iter()
        .find(|instance| name == instance.identifier)
        .map(|instance| instance.value.clone())
}

macro_rules! get_entities {
    ($self: expr, $tile_size: expr, $identifier: expr, $type: ident) => {
        $self
            .iter()
            .filter(|x| x.identifier == $identifier)
            .map(|x| {
                let position = Position(x.grid.x, x.grid.y);
                let size = (x.width / $tile_size.0, x.height / $tile_size.1);
                $type {
                    position,
                    size,
                    level_iid: "".to_string(),
                }
            })
            .collect()
    };
}

// Macro to collect entities matching multiple identifiers (e.g., DoorHorizontal, DoorVertical)
macro_rules! get_entities_multi {
    ($self: expr, $tile_size: expr, $identifiers: expr, $type: ident) => {
        $self
            .iter()
            .filter(|x| $identifiers.contains(&x.identifier.as_str()))
            .map(|x| {
                let position = Position(x.grid.x, x.grid.y);
                let size = (x.width / $tile_size.0, x.height / $tile_size.1);
                $type {
                    position,
                    size,
                    level_iid: "".to_string(),
                }
            })
            .collect()
    };
}

/// Activation bounds are authored in level-local LDtk pixels, not grid cells.
fn get_enemy_spawns(
    entities: &[bevy_ecs_ldtk::EntityInstance],
    tile_size: &(i32, i32),
) -> Vec<(EntityLocation, EnemySpawnConfig)> {
    entities
        .iter()
        .filter(|entity| entity.identifier == map_const::ENTITY_ZOMBIE_SPAWN_LOCATION)
        .map(|entity| {
            let names = ["active_x", "active_y", "active_width", "active_height"];
            let has_bounds = entity
                .field_instances
                .iter()
                .any(|field| names.contains(&field.identifier.as_str()));
            let activation_area = has_bounds.then(|| {
                let values = names.map(|name| {
                    *entity.get_int_field(name).unwrap_or_else(|_| {
                        panic!(
                            "ZombieSpawn {}: activation bounds require all four integer fields \
                             active_x, active_y, active_width and active_height (invalid {name})",
                            entity.iid
                        )
                    })
                });
                let [x, y, width, height] = values;
                assert!(
                    width > 0 && height > 0,
                    "ZombieSpawn {}: active_width and active_height must be positive",
                    entity.iid
                );
                (x, y, width, height)
            });
            (
                EntityLocation {
                    position: Position(entity.grid.x, entity.grid.y),
                    size: (entity.width / tile_size.0, entity.height / tile_size.1),
                    level_iid: String::new(),
                },
                EnemySpawnConfig { activation_area },
            )
        })
        .collect()
}

/// T2.9 (testbed) : extrait les entités `CharacterSpawn` avec leurs champs `character`/`team`
/// (valeurs d'auteur, contrairement aux portes dont la config est recalculée plus tard —
/// `get_entities!`/`get_entities_multi!` ne portent que position/taille, pas de champs).
fn get_character_spawns(
    entities: &[bevy_ecs_ldtk::EntityInstance],
    tile_size: &(i32, i32),
) -> Vec<(EntityLocation, CharacterSpawnConfig)> {
    entities
        .iter()
        .filter(|x| x.identifier == map_const::ENTITY_CHARACTER_SPAWN_LOCATION)
        .map(|x| {
            let position = Position(x.grid.x, x.grid.y);
            let size = (x.width / tile_size.0, x.height / tile_size.1);
            let character = x
                .get_string_field(map_const::FIELD_CHARACTER_NAME)
                .ok()
                .cloned()
                .unwrap_or_default();
            let team = x
                .get_string_field(map_const::FIELD_TEAM_NAME)
                .ok()
                .cloned()
                .filter(|s| !s.is_empty());
            let variant = x
                .get_string_field(map_const::FIELD_VARIANT_NAME)
                .ok()
                .cloned()
                .filter(|s| !s.is_empty());
            (
                EntityLocation {
                    position,
                    size,
                    level_iid: "".to_string(),
                },
                CharacterSpawnConfig {
                    character,
                    team,
                    variant,
                },
            )
        })
        .collect()
}

/// T2.3 (chantier C5 v1) : extrait les entités `WeaponLocation` avec leurs champs
/// `weapon`/`price` (valeurs d'auteur, comme [`get_character_spawns`] — voir sa doc).
fn get_weapon_locations(
    entities: &[bevy_ecs_ldtk::EntityInstance],
    tile_size: &(i32, i32),
) -> Vec<(EntityLocation, WeaponLocationConfig)> {
    entities
        .iter()
        .filter(|x| x.identifier == map_const::ENTITY_WEAPON_LOCATION)
        .map(|x| {
            let position = Position(x.grid.x, x.grid.y);
            let size = (x.width / tile_size.0, x.height / tile_size.1);
            let weapon = x
                .get_string_field(map_const::FIELD_WEAPON_NAME)
                .ok()
                .cloned()
                .unwrap_or_default();
            let price = x
                .get_int_field(map_const::FIELD_PRICE_NAME)
                .ok()
                .copied()
                .unwrap_or(0)
                .max(0) as u32;
            (
                EntityLocation {
                    position,
                    size,
                    level_iid: "".to_string(),
                },
                WeaponLocationConfig { weapon, price },
            )
        })
        .collect()
}

/// T2.3 (chantier C5 v1) : extrait les entités `SodaLocation` avec leur champ `perk`.
fn get_soda_locations(
    entities: &[bevy_ecs_ldtk::EntityInstance],
    tile_size: &(i32, i32),
) -> Vec<(EntityLocation, SodaLocationConfig)> {
    entities
        .iter()
        .filter(|x| x.identifier == map_const::ENTITY_SODA_LOCATION)
        .map(|x| {
            let position = Position(x.grid.x, x.grid.y);
            let size = (x.width / tile_size.0, x.height / tile_size.1);
            let perk = x
                .get_string_field(map_const::FIELD_PERK_NAME)
                .ok()
                .cloned()
                .unwrap_or_default();
            (
                EntityLocation {
                    position,
                    size,
                    level_iid: "".to_string(),
                },
                SodaLocationConfig { perk },
            )
        })
        .collect()
}

/// M2-T11 : extrait les entités `Table` et `Barrel` avec leur champ `prop`.
fn get_props(
    entities: &[bevy_ecs_ldtk::EntityInstance],
    tile_size: &(i32, i32),
) -> Vec<(EntityLocation, PropConfig)> {
    entities
        .iter()
        .filter(|x| {
            x.identifier == map_const::ENTITY_TABLE || x.identifier == map_const::ENTITY_BARREL
        })
        .map(|x| {
            let position = Position(x.grid.x, x.grid.y);
            let size = (x.width / tile_size.0, x.height / tile_size.1);
            let prop = x
                .get_string_field(map_const::FIELD_PROP_NAME)
                .ok()
                .cloned()
                .unwrap_or_default();
            (
                EntityLocation {
                    position,
                    size,
                    level_iid: "".to_string(),
                },
                PropConfig {
                    entity: x.identifier.clone(),
                    prop,
                },
            )
        })
        .collect()
}

fn extract_entity_locations(level: &Level, tile_size: &(i32, i32)) -> EntityLocations {
    let entity_layer = level
        .layer_instances
        .as_ref()
        .unwrap()
        .iter()
        .find(|x| x.identifier == map_const::LAYER_ENTITY);

    if let Some(entity_layer) = entity_layer {
        EntityLocations {
            doors: get_entities_multi!(
                entity_layer.entity_instances,
                tile_size,
                &[
                    map_const::ENTITY_DOOR_HORIZONTAL_LOCATION,
                    map_const::ENTITY_DOOR_VERTICAL_LOCATION
                ],
                EntityLocation
            ),
            sodas: get_soda_locations(&entity_layer.entity_instances, tile_size),
            player_spawns: get_entities!(
                entity_layer.entity_instances,
                tile_size,
                map_const::ENTITY_PLAYER_SPAWN_LOCATION,
                EntityLocation
            ),
            zombie_spawns: get_enemy_spawns(&entity_layer.entity_instances, tile_size),
            crates: get_entities!(
                entity_layer.entity_instances,
                tile_size,
                map_const::ENTITY_CRATE_LOCATION,
                EntityLocation
            ),
            weapons: get_weapon_locations(&entity_layer.entity_instances, tile_size),
            windows: get_entities_multi!(
                entity_layer.entity_instances,
                tile_size,
                &[
                    map_const::ENTITY_WINDOW_HORIZONTAL_LOCATION,
                    map_const::ENTITY_WINDOW_VERTICAL_LOCATION
                ],
                EntityLocation
            ),
            character_spawns: get_character_spawns(&entity_layer.entity_instances, tile_size),
            props: get_props(&entity_layer.entity_instances, tile_size),
        }
    } else {
        EntityLocations {
            doors: vec![],
            sodas: vec![],
            props: vec![],
            player_spawns: vec![],
            zombie_spawns: vec![],
            crates: vec![],
            weapons: vec![],
            windows: vec![],
            character_spawns: vec![],
        }
    }
}

fn to_available_level(level: &Level, tile_size: &(i32, i32)) -> AvailableLevel {
    let level_size: (usize, usize) = (
        (level.px_wid / tile_size.0) as usize,
        (level.px_hei / tile_size.1) as usize,
    );

    // identify each level connection
    let connection_layer: &LayerInstance = level
        .layer_instances
        .as_ref()
        .map(|layer_instance| {
            layer_instance
                .iter()
                .find(|item| map_const::LAYER_CONNECTION == item.identifier)
                .ok_or("Failed to find LevelConnetion Layer on level")
        })
        .unwrap_or_else(|| Err("No Layers present"))
        .unwrap();

    let grid: Vec<&[i32]> = connection_layer.int_grid_csv.chunks(level_size.0).collect();

    let level_type = {
        let is_spawn = get_level_field(level, map_const::LEVEL_FIELD_SPAWN).is_some_and(|x| {
            if let FieldValue::Bool(value) = x {
                value
            } else {
                false
            }
        });

        if is_spawn {
            LevelType::Spawn
        } else {
            LevelType::Normal
        }
    };

    // get my entity layer from the level and extract all entity

    let mut available_level = AvailableLevel {
        level_id: level.identifier.clone(),
        connections: vec![],
        level_size,
        level_size_p: (
            level_size.0 as i32 * tile_size.0,
            level_size.1 as i32 * tile_size.1,
        ),
        level_type,
        entity_locations: extract_entity_locations(level, tile_size),
        // M2-T10 : champ de niveau `room_kind` (M2-E1), lu par la grammaire d'étage.
        room_kind: get_level_field(level, "room_kind").and_then(|value| match value {
            FieldValue::String(Some(kind)) if !kind.is_empty() => Some(kind),
            _ => None,
        }),
    };

    let mut connections = vec![];
    let mut index = 0;
    scan_width_side(
        &mut connections,
        &mut index,
        &available_level,
        &level_size,
        &grid,
        0,
        Side::N,
    );
    scan_width_side(
        &mut connections,
        &mut index,
        &available_level,
        &level_size,
        &grid,
        level_size.1 - 1,
        Side::S,
    );

    scan_height_side(
        &mut connections,
        &mut index,
        &available_level,
        &level_size,
        &grid,
        0,
        Side::W,
    );
    scan_height_side(
        &mut connections,
        &mut index,
        &available_level,
        &level_size,
        &grid,
        level_size.0 - 1,
        Side::E,
    );

    available_level.connections = connections;

    available_level
}

pub fn from_map(map_json: &LdtkJson, config: MapGenerationConfig) -> MapGenerationContext {
    if map_json.levels.is_empty() {
        eprintln!("to few level present in the project");
    }

    let tile_size = (
        map_json.default_entity_width,
        map_json.default_entity_height,
    );

    let first_level = map_json.levels.first().unwrap();

    let level_size = (
        first_level.px_wid / tile_size.0,
        first_level.px_hei / tile_size.1,
    );

    debug!("starting level generation with config \nseed={} \ntilse_size={}x{} \nlevel_size={}x{}\nmap_size={}x{}",
        config.seed, tile_size.0, tile_size.1, level_size.0, level_size.1,
        config.max_width, config.max_heigth
    );

    let mut available_levels: Vec<AvailableLevel> = map_json
        .levels
        .iter()
        .map(|item| to_available_level(item, &tile_size))
        .collect();

    populate_level_connections(&mut available_levels, config.aligned_openings);

    let available_levels = available_levels
        .iter()
        .map(|x| Rc::new(x.clone()))
        .collect();

    MapGenerationContext {
        level_size,
        tile_size,
        config,
        available_levels,
    }
}
