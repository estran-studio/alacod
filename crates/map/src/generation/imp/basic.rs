use std::rc::Rc;

use crate::{
    game::entity::map::player_spawn::PlayerSpawnConfig,
    generation::{
        context::{AvailableLevel, LevelType, MapGenerationContext, MapGenerationData},
        entity::{
            character_spawn::CharacterSpawnConfig, door::DoorConfig, enemy_spawn::EnemySpawnConfig,
            location::EntityLocation, soda_location::SodaLocationConfig,
            weapon_location::WeaponLocationConfig, window::WindowConfig,
        },
        position::Position,
        room::{ConnectionTo, RoomConnection},
        IMapGeneration, Room, LEVEL_PROPERTIES_SPAWN_NAME,
    },
};

use bevy_fixed::rng::RollbackRng;
use serde_json::Value;
use utils::map;

// private struct to store data during the map generation
struct Map {
    // index of the last room that we iterate on
    last_generated_room_index: Option<usize>,
    // list of the room i have generated
    rooms: Vec<Room>,
    // index of all the item in the rooms vector that still possively have open connection
    rooms_possible: Vec<usize>,
    // Track the depth/distance from spawn for each room (by index)
    room_depths: Vec<usize>,
}

pub struct BasicMapGeneration {
    context: MapGenerationContext,
    data: MapGenerationData,
    map: Map,
}

impl BasicMapGeneration {
    pub fn create(context: MapGenerationContext) -> Self {
        BasicMapGeneration {
            data: MapGenerationData::default(),
            context,
            map: Map {
                last_generated_room_index: None,
                rooms: vec![],
                rooms_possible: vec![],
                room_depths: vec![],
            },
        }
    }
}

// Helper function to check if a door is positioned at a specific connection
fn is_door_at_connection(
    door_location: &EntityLocation,
    room: &Room,
    connection: &RoomConnection,
) -> bool {
    use crate::generation::context::Side;

    let room_size_tiles = room.level_def.level_size;
    let door_pos = &door_location.position;
    let door_size = &door_location.size;

    // Get the connection definition to know where it is
    let connection_def = match room
        .level_def
        .connections
        .iter()
        .find(|c| c.index == connection.index)
    {
        Some(def) => def,
        None => return false,
    };

    // Check if door is at the edge where the connection is
    match connection_def.side {
        Side::N => {
            // North side: door should be at y = 0 (top edge)
            door_pos.1 == 0
                && door_pos.0 >= connection_def.starting_at as i32
                && door_pos.0 < (connection_def.starting_at + connection_def.size) as i32
        }
        Side::S => {
            // South side: door should be at y = room_size - door_height (bottom edge)
            door_pos.1 + door_size.1 >= room_size_tiles.1 as i32
                && door_pos.0 >= connection_def.starting_at as i32
                && door_pos.0 < (connection_def.starting_at + connection_def.size) as i32
        }
        Side::W => {
            // West side: door should be at x = 0 (left edge)
            door_pos.0 == 0
                && door_pos.1 >= connection_def.starting_at as i32
                && door_pos.1 < (connection_def.starting_at + connection_def.size) as i32
        }
        Side::E => {
            // East side: door should be at x = room_size - door_width (right edge)
            door_pos.0 + door_size.0 >= room_size_tiles.0 as i32
                && door_pos.1 >= connection_def.starting_at as i32
                && door_pos.1 < (connection_def.starting_at + connection_def.size) as i32
        }
    }
}

impl BasicMapGeneration {
    fn get_next_room_recursize(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Option<(Room, RoomConnection, RoomConnection)> {
        if self.context.config.max_room > 0 && self.map.rooms.len() >= self.context.config.max_room
        {
            println!(
                "max room stopping generation {} {}",
                self.map.rooms.len(),
                self.context.config.max_room
            );
            return None;
        }

        loop {
            if self.map.last_generated_room_index.is_none() {
                println!("no room mark to continue generation");
                return None;
            }

            let previous_room_index = self.map.last_generated_room_index.unwrap();

            let previous_room = self.map.rooms.get_mut(previous_room_index).unwrap();
            let previous_room_def = previous_room.level_def.clone();

            let free_connection_len = previous_room
                .connections
                .iter()
                .filter(|i| i.to.is_none())
                .count();

            if free_connection_len == 0 {
                if let Some(index) = self
                    .map
                    .rooms_possible
                    .iter()
                    .position(|&x| x == previous_room_index)
                {
                    self.map.rooms_possible.remove(index);
                }

                if self.map.rooms_possible.is_empty() {
                    println!("no more room_possible stopping generation");
                    return None;
                }

                self.map.last_generated_room_index = self
                    .map
                    .rooms_possible
                    .get(rng.next_u32_range(0, self.map.rooms_possible.len() as u32) as usize)
                    .copied();

                continue;
            } else {
                // get the connection def
                let connection_def = {
                    let connection = previous_room
                        .connections
                        .iter()
                        .filter(|i| i.to.is_none())
                        .skip(rng.next_u32_range(0, free_connection_len as u32) as usize)
                        .last()
                        .unwrap();

                    previous_room_def.connections.get(connection.index).unwrap()
                };

                if connection_def.compatiable_levels.is_empty() {
                    previous_room
                        .connections
                        .get_mut(connection_def.index)
                        .unwrap()
                        .to = Some(ConnectionTo::DeadEnd);
                    println!("no compatible levels marking as DeadEnd");
                    continue;
                } else {
                    let compatible_level = connection_def
                        .compatiable_levels
                        .iter()
                        .skip(
                            rng.next_u32_range(0, connection_def.compatiable_levels.len() as u32)
                                as usize,
                        )
                        .last()
                        .unwrap();

                    let compatible_level_def = self
                        .context
                        .available_levels
                        .iter()
                        .find(|l| l.level_id == compatible_level.0)
                        .unwrap();

                    let level_connection = compatible_level_def
                        .connections
                        .get(compatible_level.1)
                        .unwrap();

                    let my_position = previous_room.get_connecting_room_position(
                        connection_def,
                        compatible_level_def,
                        compatible_level.1,
                        &self.context.tile_size,
                    );

                    let mut new_room = Room::create(
                        rng,
                        compatible_level_def.clone(),
                        my_position,
                        map!(LEVEL_PROPERTIES_SPAWN_NAME => Value::Bool(false)),
                    );

                    if new_room.is_outside(&self.context.config) {
                        previous_room
                            .connections
                            .get_mut(connection_def.index)
                            .unwrap()
                            .to = Some(ConnectionTo::OutSide);

                        continue;
                    } else {
                        new_room.set_connection_between(
                            level_connection.index,
                            previous_room,
                            connection_def.index,
                        );

                        let new_room_level_connection = new_room
                            .connections
                            .get(level_connection.index)
                            .unwrap()
                            .clone();

                        return Some((
                            new_room,
                            new_room_level_connection,
                            previous_room
                                .connections
                                .get(connection_def.index)
                                .unwrap()
                                .clone(),
                        ));
                    }
                }
            }
        }
    }
}

impl IMapGeneration for BasicMapGeneration {
    fn get_spawning_room(&mut self, rng: &mut RollbackRng) -> Room {
        let spawning_levels: Vec<&Rc<AvailableLevel>> = self
            .context
            .available_levels
            .iter()
            .filter(|i| i.level_type == LevelType::Spawn)
            .collect();

        let spawning_room_def = spawning_levels
            .iter()
            .skip(rng.next_u32_range_inclusive(0, (spawning_levels.len() - 1) as u32) as usize)
            .last();

        if spawning_room_def.is_none() {
            panic!("no spawning room found");
        }

        let spawning_room_def = (*spawning_room_def.unwrap()).clone();

        let x: i32 = rng.next_i32_range_inclusive(
            -self.context.config.max_width,
            self.context.config.max_width - spawning_room_def.level_size_p.0,
        );
        let y: i32 = rng.next_i32_range_inclusive(
            -self.context.config.max_heigth,
            self.context.config.max_heigth - spawning_room_def.level_size_p.1,
        );

        let spawning_room_def = Room::create(
            rng,
            spawning_room_def.clone(),
            Position(x, y),
            map!(LEVEL_PROPERTIES_SPAWN_NAME => Value::Bool(true)),
        );
        self.map.rooms.push(spawning_room_def.clone());
        self.map.room_depths.push(0); // Spawn room is at depth 0
        self.map.last_generated_room_index = Some(0);

        spawning_room_def
    }

    fn get_next_room(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Option<(Room, RoomConnection, RoomConnection)> {
        let room = self.get_next_room_recursize(rng);
        if let Some(room) = room.as_ref() {
            // Calculate depth: parent room's depth + 1
            let parent_depth = self.map.room_depths[self.map.last_generated_room_index.unwrap()];
            let new_depth = parent_depth + 1;

            self.map.rooms.push(room.0.clone());
            self.map.room_depths.push(new_depth);
            let index = self.map.rooms.len() - 1;
            self.map.rooms_possible.push(index);
        }

        room
    }

    fn get_doors(
        &mut self,
        _rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, crate::generation::entity::door::DoorConfig)> {
        // First pass: collect all doors with their room info
        let mut all_doors: Vec<(usize, EntityLocation, DoorConfig)> = vec![];

        for (room_index, room) in self.map.rooms.iter().enumerate() {
            let room_depth = self.map.room_depths[room_index];
            let base_cost = 750;
            let cost_per_depth = 250;
            let door_cost = base_cost + (room_depth * cost_per_depth) as i32;

            for door_location in &room.entity_locations.doors {
                all_doors.push((
                    room_index,
                    EntityLocation {
                        position: door_location.position,
                        size: door_location.size,
                        level_iid: room.level_iid.clone(),
                    },
                    DoorConfig {
                        cost: door_cost,
                        electrify: room_depth > 0,
                        interactable: true, // Default to interactable, will update based on connection
                        paired_door: None,
                    },
                ));
            }
        }

        // Second pass: identify doors at connections and pair them
        // We'll build a list of pairings to apply after iteration
        let mut pairings: Vec<(usize, usize)> = vec![]; // (door_index, other_door_index)
        let mut non_interactable: Vec<usize> = vec![]; // door indices that should not be interactable

        for (room_index, room) in self.map.rooms.iter().enumerate() {
            for connection in &room.connections {
                // Only process connections that lead to another room
                if let Some(ConnectionTo::Room((other_level_iid, other_connection_index))) =
                    &connection.to
                {
                    // Find the other room by level_iid
                    if let Some((other_room_index, other_room)) = self
                        .map
                        .rooms
                        .iter()
                        .enumerate()
                        .find(|(_, r)| &r.level_iid == other_level_iid)
                    {
                        // Find doors in current room that are at this connection
                        for (door_index, (door_room_index, door_loc, door_config)) in
                            all_doors.iter().enumerate()
                        {
                            if *door_room_index == room_index && door_config.paired_door.is_none() {
                                // Check if this door is at the current connection
                                if is_door_at_connection(door_loc, room, connection) {
                                    // Find the corresponding door on the other side
                                    for (
                                        other_door_index,
                                        (other_door_room_index, other_door_loc, other_door_config),
                                    ) in all_doors.iter().enumerate()
                                    {
                                        if *other_door_room_index == other_room_index
                                            && other_door_config.paired_door.is_none()
                                        {
                                            if let Some(other_connection) =
                                                other_room.connections.get(*other_connection_index)
                                            {
                                                if is_door_at_connection(
                                                    other_door_loc,
                                                    other_room,
                                                    other_connection,
                                                ) {
                                                    // Record this pairing
                                                    pairings.push((door_index, other_door_index));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if connection.to.is_some() {
                    // This is a dead-end or outside connection
                    // Mark doors at this connection as non-interactable
                    for (door_index, (door_room_index, door_loc, _)) in all_doors.iter().enumerate()
                    {
                        if *door_room_index == room_index {
                            if is_door_at_connection(door_loc, room, connection) {
                                non_interactable.push(door_index);
                            }
                        }
                    }
                }
            }
        }

        // Apply pairings
        for (door_index, other_door_index) in pairings {
            let other_loc = &all_doors[other_door_index].1;
            all_doors[door_index].2.paired_door = Some((
                other_loc.level_iid.clone(),
                (other_loc.position.0, other_loc.position.1),
            ));

            let door_loc = &all_doors[door_index].1;
            all_doors[other_door_index].2.paired_door = Some((
                door_loc.level_iid.clone(),
                (door_loc.position.0, door_loc.position.1),
            ));
        }

        // Apply non-interactable flags
        for door_index in non_interactable {
            all_doors[door_index].2.interactable = false;
        }

        // Return all doors with their updated configs
        all_doors
            .into_iter()
            .map(|(_, loc, config)| (loc, config))
            .collect()
    }

    fn get_windows(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(
        EntityLocation,
        crate::generation::entity::window::WindowConfig,
    )> {
        self.map
            .rooms
            .iter()
            .flat_map(|x| {
                x.entity_locations
                    .windows
                    .iter()
                    .map(|y| {
                        (
                            EntityLocation {
                                position: y.position,
                                size: y.size,
                                level_iid: x.level_iid.clone(),
                            },
                            WindowConfig {},
                        )
                    })
                    .collect::<Vec<(
                        EntityLocation,
                        crate::generation::entity::window::WindowConfig,
                    )>>()
            })
            .collect()
    }

    fn get_player_spawn(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, PlayerSpawnConfig)> {
        self.map
            .rooms
            .iter()
            .filter(|x| {
                let property = x.properties.get(LEVEL_PROPERTIES_SPAWN_NAME);
                if let Some(property) = property {
                    if let Value::Bool(b) = property {
                        return *b;
                    }
                }
                false
            })
            .flat_map(|roor| {
                roor.entity_locations
                    .player_spawns
                    .iter()
                    .enumerate()
                    .map(|(i, y)| {
                        (
                            EntityLocation {
                                position: y.position,
                                size: y.size,
                                level_iid: roor.level_iid.clone(),
                            },
                            PlayerSpawnConfig { index: i },
                        )
                    })
                    .collect::<Vec<(EntityLocation, PlayerSpawnConfig)>>()
            })
            .collect()
    }

    fn get_enemy_spawns(
        &mut self,
        rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, EnemySpawnConfig)> {
        self.map
            .rooms
            .iter()
            .flat_map(|room| {
                room.entity_locations
                    .zombie_spawns
                    .iter()
                    .map(|location| {
                        (
                            EntityLocation {
                                position: location.position,
                                size: location.size,
                                level_iid: room.level_iid.clone(),
                            },
                            EnemySpawnConfig {},
                        )
                    })
                    .collect::<Vec<(EntityLocation, EnemySpawnConfig)>>()
            })
            .collect()
    }

    fn get_character_spawns(
        &mut self,
        _rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, CharacterSpawnConfig)> {
        self.map
            .rooms
            .iter()
            .flat_map(|room| {
                room.entity_locations
                    .character_spawns
                    .iter()
                    .map(|(location, config)| {
                        (
                            EntityLocation {
                                position: location.position,
                                size: location.size,
                                level_iid: room.level_iid.clone(),
                            },
                            config.clone(),
                        )
                    })
                    .collect::<Vec<(EntityLocation, CharacterSpawnConfig)>>()
            })
            .collect()
    }

    /// T2.3, chantier C5 v1 : mêmes règles que [`Self::get_character_spawns`] ci-dessus.
    fn get_weapon_locations(
        &mut self,
        _rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, WeaponLocationConfig)> {
        self.map
            .rooms
            .iter()
            .flat_map(|room| {
                room.entity_locations
                    .weapons
                    .iter()
                    .map(|(location, config)| {
                        (
                            EntityLocation {
                                position: location.position,
                                size: location.size,
                                level_iid: room.level_iid.clone(),
                            },
                            config.clone(),
                        )
                    })
                    .collect::<Vec<(EntityLocation, WeaponLocationConfig)>>()
            })
            .collect()
    }

    /// T2.3, chantier C5 v1 : mêmes règles que [`Self::get_character_spawns`] ci-dessus.
    fn get_soda_locations(
        &mut self,
        _rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, SodaLocationConfig)> {
        self.map
            .rooms
            .iter()
            .flat_map(|room| {
                room.entity_locations
                    .sodas
                    .iter()
                    .map(|(location, config)| {
                        (
                            EntityLocation {
                                position: location.position,
                                size: location.size,
                                level_iid: room.level_iid.clone(),
                            },
                            config.clone(),
                        )
                    })
                    .collect::<Vec<(EntityLocation, SodaLocationConfig)>>()
            })
            .collect()
    }
}
