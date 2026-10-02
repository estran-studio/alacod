use bevy::log::debug;
use std::rc::Rc;

use bevy::math::{IVec2, Vec2};
use bevy_ecs_ldtk::{
    ldtk::{FieldInstance, FieldValue, LdtkJson, Level, NeighbourLevel, RealEditorValue},
    EntityInstance,
};
use bevy_fixed::rng::RollbackRng;
use serde_json::Value;

use map::{
    game::entity::map::player_spawn::PlayerSpawnConfig,
    generation::{
        entity::{
            character_spawn::CharacterSpawnConfig, door::DoorConfig, enemy_spawn::EnemySpawnConfig,
            location::EntityLocation, soda_location::SodaLocationConfig,
            weapon_location::WeaponLocationConfig, window::WindowConfig,
        },
        room::{Room, RoomConnection},
        IMapGenerator,
    },
};

use crate::map_const::{self, LAYER_ENTITY};

#[derive(Debug, Clone)]
pub struct GeneratedRoom {
    level: Level,
    ldtk: Rc<LdtkJson>,
}

impl GeneratedRoom {
    pub fn create(ldtk_json: Rc<LdtkJson>, room: &Room) -> Self {
        let mut level = ldtk_json
            .levels
            .iter()
            .find(|item| item.identifier == room.level_def.level_id)
            .expect("failed to find level from original")
            .clone();

        level.iid = room.level_iid.clone();
        level.identifier = room.level_iid.clone();
        level.world_x = room.position.0;
        level.world_y = room.position.1;
        level.neighbours.clear();

        if !room.properties.is_empty() {
            for field in level.field_instances.iter_mut() {
                if let Some(value) = room.properties.get(field.identifier.as_str()) {
                    field.value = match value {
                        Value::Bool(b_value) => FieldValue::Bool(*b_value),
                        _ => FieldValue::String(None),
                    };
                    field.real_editor_values = vec![Some(RealEditorValue {
                        // TODO: do the correct mapping
                        id: "V_Bool".to_string(),
                        params: vec![value.clone()],
                    })]
                }
            }
        }
        level
            .layer_instances
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|x| x.identifier == LAYER_ENTITY)
            .unwrap()
            .entity_instances
            .clear();

        GeneratedRoom {
            level,
            ldtk: ldtk_json,
        }
    }
}

pub struct GeneratedMap {
    pub ldtk_json: Rc<LdtkJson>,
    pub generated_rooms: Vec<GeneratedRoom>,
}

pub fn _add_property_entity() {}

pub fn get_new_entity(
    rng: &mut RollbackRng,
    room: &GeneratedRoom,
    original_entity_identifier: &str,
    location: &EntityLocation,
    tile_size: (i32, i32),
    fields: Vec<(&str, FieldValue)>,
) -> EntityInstance {
    let entity = room
        .ldtk
        .defs
        .entities
        .iter()
        .find(|x| x.identifier == original_entity_identifier)
        .unwrap();

    let px = (
        location.position.0 * tile_size.0,
        location.position.1 * tile_size.1,
    );
    let world_px = (px.0 + room.level.world_x, px.1 + room.level.world_y);

    let identifiers = fields
        .iter()
        .map(|x| {
            // Try to find the field in the entity template
            let field_opt = entity.field_defs.iter().find(|fd| fd.identifier == x.0);

            let real_editor_value = match x.1.clone() {
                FieldValue::Int(v) => Some(("V_Int", serde_json::to_value(v).unwrap())),
                FieldValue::Bool(v) => Some(("V_Bool", serde_json::to_value(v).unwrap())),
                FieldValue::String(v) => v
                    .as_ref()
                    .map(|s| ("V_String", serde_json::to_value(s).unwrap())),
                _ => None,
            };

            let real_editor_value = real_editor_value.map(|v| RealEditorValue {
                id: v.0.to_string(),
                params: vec![v.1],
            });

            // If field exists in template, use its definition; otherwise create a synthetic one
            if let Some(field) = field_opt {
                FieldInstance {
                    identifier: field.identifier.clone(),
                    def_uid: field.uid,
                    field_instance_type: field.field_definition_type.clone(),
                    value: x.1.clone(),
                    tile: None,
                    real_editor_values: vec![real_editor_value],
                }
            } else {
                // Create a synthetic field instance for fields not in the template
                let field_type = match x.1 {
                    FieldValue::Int(_) => "Int",
                    FieldValue::Bool(_) => "Bool",
                    FieldValue::String(_) => "String",
                    _ => "String", // Default to String for unknown types
                };

                FieldInstance {
                    identifier: x.0.to_string(),
                    def_uid: 0, // Use 0 for synthetic fields
                    field_instance_type: field_type.to_string(),
                    value: x.1.clone(),
                    tile: None,
                    real_editor_values: vec![real_editor_value],
                }
            }
        })
        .collect();

    EntityInstance {
        identifier: original_entity_identifier.into(),
        def_uid: entity.uid,
        grid: IVec2::new(location.position.0, location.position.1),
        pivot: Vec2::new(entity.pivot_x, entity.pivot_y),
        tags: vec![],
        tile: entity.tile_rect,
        smart_color: entity.color,
        iid: rng.next_uuid(),
        width: location.size.0 * tile_size.0,
        height: location.size.1 * tile_size.1,
        field_instances: identifiers,
        px: IVec2::new(px.0, px.1),
        world_x: Some(world_px.0),
        world_y: Some(world_px.1),
    }
}

impl GeneratedMap {
    pub fn create(ldtk_json: LdtkJson) -> Self {
        GeneratedMap {
            ldtk_json: Rc::new(ldtk_json),
            generated_rooms: vec![],
        }
    }

    pub fn get_generated_map(&self) -> LdtkJson {
        let mut new_map: LdtkJson = (*self.ldtk_json).clone();

        new_map.levels = self
            .generated_rooms
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let mut r = x.level.clone();
                r.identifier = format!("Level_{}", i);
                r
            })
            .collect();

        new_map
    }

    fn add_entity_to_level(
        &mut self,
        rng: &mut RollbackRng,
        location: &EntityLocation,
        entity_type: &str,
        fields: Vec<(&str, FieldValue)>,
    ) {
        let level = self
            .generated_rooms
            .iter_mut()
            .find(|x| x.level.iid == location.level_iid)
            .unwrap();

        let new_entity = get_new_entity(
            rng,
            level,
            entity_type,
            location,
            (
                self.ldtk_json.default_entity_width,
                self.ldtk_json.default_entity_height,
            ),
            fields,
        );

        level
            .level
            .layer_instances
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|x| x.identifier == map_const::LAYER_ENTITY)
            .unwrap()
            .entity_instances
            .push(new_entity);
    }
}

impl IMapGenerator for GeneratedMap {
    fn add_room(
        &mut self,
        _rng: &mut RollbackRng,
        room: &Room,
        connection_used: Option<&RoomConnection>,
        connected_to: Option<&RoomConnection>,
    ) {
        let mut generated_room = GeneratedRoom::create(self.ldtk_json.clone(), room);

        debug!(
            "adding room id={} type={:?} from_level={} position={} \n property={:?}",
            room.level_iid,
            room.level_def.level_type,
            room.level_def.level_id,
            room.position,
            room.properties
        );

        if let Some(connected_to) = connected_to {
            let connection_used = connection_used.unwrap();

            generated_room.level.neighbours.push(NeighbourLevel {
                level_iid: connected_to.level_iid.clone(),
                dir: connection_used.side.to_dir_str().into(),
                ..Default::default()
            });

            // find the other room and me as it's neighbours
            let linked_room = self
                .generated_rooms
                .iter_mut()
                .find(|r| r.level.iid == connected_to.level_iid)
                .unwrap();

            debug!("  connecting my side={:?} index={} with side={:?} index={} of room id={} from_level={} position={}x{}",
               connection_used.side, connection_used.index, connected_to.side, connected_to.index, connected_to.level_iid,
               connected_to.level_id, linked_room.level.world_x, linked_room.level.world_y,
            );

            linked_room.level.neighbours.push(NeighbourLevel {
                dir: connected_to.side.to_dir_str().into(),
                level_iid: room.level_iid.clone(),
                ..Default::default()
            })
        }

        self.generated_rooms.push(generated_room);
    }

    fn add_doors(&mut self, rng: &mut RollbackRng, doors: &Vec<(EntityLocation, DoorConfig)>) {
        debug!("Adding {} doors to map", doors.len());

        for (location, door) in doors.iter() {
            // Determine if door is horizontal or vertical based on size
            // If width > height, it's horizontal; otherwise vertical
            let door_type = if location.size.0 > location.size.1 {
                map_const::ENTITY_DOOR_HORIZONTAL_LOCATION
            } else {
                map_const::ENTITY_DOOR_VERTICAL_LOCATION
            };

            // Log door configuration
            let pairing_info = if let Some((paired_level, (px, py))) = &door.paired_door {
                format!(
                    "paired with door at ({}, {}) in level {}",
                    px, py, paired_level
                )
            } else {
                "unpaired".to_string()
            };

            debug!(
                "  Door at ({}, {}) in level {}: cost={}, electrify={}, interactable={}, {}",
                location.position.0,
                location.position.1,
                location.level_iid,
                door.cost,
                door.electrify,
                door.interactable,
                pairing_info
            );

            let mut fields = vec![
                (
                    map_const::FIELD_PRICE_NAME,
                    FieldValue::Int(Some(door.cost)),
                ),
                (
                    map_const::FIELD_ELECTRIFY_NAME,
                    FieldValue::Bool(door.electrify),
                ),
                (
                    map_const::FIELD_INTERACTABLE_NAME,
                    FieldValue::Bool(door.interactable),
                ),
            ];

            // Add paired door information if it exists
            if let Some((paired_level_iid, (paired_x, paired_y))) = &door.paired_door {
                fields.push((
                    map_const::FIELD_PAIRED_DOOR_X_NAME,
                    FieldValue::Int(Some(*paired_x)),
                ));
                fields.push((
                    map_const::FIELD_PAIRED_DOOR_Y_NAME,
                    FieldValue::Int(Some(*paired_y)),
                ));
                fields.push((
                    map_const::FIELD_PAIRED_DOOR_LEVEL_NAME,
                    FieldValue::String(Some(paired_level_iid.clone())),
                ));
            }

            self.add_entity_to_level(rng, location, door_type, fields);
        }

        debug!("Door generation complete\n");
    }

    fn add_windows(
        &mut self,
        rng: &mut RollbackRng,
        windows: &Vec<(EntityLocation, WindowConfig)>,
    ) {
        for (location, _) in windows.iter() {
            // Determine if window is horizontal or vertical based on size
            // If width > height, it's horizontal; otherwise vertical
            let window_type = if location.size.0 > location.size.1 {
                map_const::ENTITY_WINDOW_HORIZONTAL_LOCATION
            } else {
                map_const::ENTITY_WINDOW_VERTICAL_LOCATION
            };

            self.add_entity_to_level(rng, location, window_type, vec![]);
        }
    }

    fn add_player_spawns(
        &mut self,
        rng: &mut RollbackRng,
        player_spawns: &Vec<(EntityLocation, PlayerSpawnConfig)>,
    ) {
        for (location, spawn) in player_spawns.iter() {
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_PLAYER_SPAWN_LOCATION,
                vec![(
                    map_const::FIELD_PLAYER_SPAWN_INDEX_NAME,
                    FieldValue::Int(Some(spawn.index as i32)),
                )],
            );
        }
    }

    fn add_enemy_spawns(
        &mut self,
        rng: &mut RollbackRng,
        enemy_spawns: &Vec<(EntityLocation, EnemySpawnConfig)>,
    ) {
        debug!("Adding {} enemy spawns to map", enemy_spawns.len());

        for (location, _spawn) in enemy_spawns.iter() {
            debug!(
                "  Enemy spawn at ({}, {}) in level {}",
                location.position.0, location.position.1, location.level_iid
            );

            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_ZOMBIE_SPAWN_LOCATION,
                vec![],
            );
        }

        debug!("Enemy spawn generation complete\n");
    }

    /// T2.9 (testbed) : contrairement aux portes/fenêtres (config recalculée ou vide),
    /// `character`/`team` sont des valeurs d'auteur : réécrites telles quelles.
    fn add_character_spawns(
        &mut self,
        rng: &mut RollbackRng,
        character_spawns: &Vec<(EntityLocation, CharacterSpawnConfig)>,
    ) {
        for (location, config) in character_spawns.iter() {
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_CHARACTER_SPAWN_LOCATION,
                vec![
                    (
                        map_const::FIELD_CHARACTER_NAME,
                        FieldValue::String(Some(config.character.clone())),
                    ),
                    (
                        map_const::FIELD_TEAM_NAME,
                        FieldValue::String(config.team.clone()),
                    ),
                ],
            );
        }
    }

    /// T2.3, chantier C5 v1 : `weapon`/`price` sont des valeurs d'auteur, réécrites telles
    /// quelles (voir la doc de [`Self::add_character_spawns`]).
    fn add_weapon_locations(
        &mut self,
        rng: &mut RollbackRng,
        weapon_locations: &Vec<(EntityLocation, WeaponLocationConfig)>,
    ) {
        for (location, config) in weapon_locations.iter() {
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_WEAPON_LOCATION,
                vec![
                    (
                        map_const::FIELD_WEAPON_NAME,
                        FieldValue::String(Some(config.weapon.clone())),
                    ),
                    (
                        map_const::FIELD_PRICE_NAME,
                        FieldValue::Int(Some(config.price as i32)),
                    ),
                ],
            );
        }
    }

    /// T2.3, chantier C5 v1 : `perk` est une valeur d'auteur, réécrite telle quelle.
    fn add_soda_locations(
        &mut self,
        rng: &mut RollbackRng,
        soda_locations: &Vec<(EntityLocation, SodaLocationConfig)>,
    ) {
        for (location, config) in soda_locations.iter() {
            self.add_entity_to_level(
                rng,
                location,
                map_const::ENTITY_SODA_LOCATION,
                vec![(
                    map_const::FIELD_PERK_NAME,
                    FieldValue::String(Some(config.perk.clone())),
                )],
            );
        }
    }
}
