use bevy::log::debug;
use std::rc::Rc;

use crate::{
    game::entity::map::player_spawn::PlayerSpawnConfig,
    generation::{
        context::{AvailableLevel, LevelType, MapGenerationContext, MapGenerationData, Side},
        entity::{
            character_spawn::CharacterSpawnConfig, door::DoorConfig, enemy_spawn::EnemySpawnConfig,
            location::EntityLocation, prop::PropConfig, soda_location::SodaLocationConfig,
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
pub(super) struct Map {
    // index of the last room that we iterate on
    pub(super) last_generated_room_index: Option<usize>,
    // list of the room i have generated
    pub(super) rooms: Vec<Room>,
    // index of all the item in the rooms vector that still possively have open connection
    pub(super) rooms_possible: Vec<usize>,
    // Track the depth/distance from spawn for each room (by index)
    pub(super) room_depths: Vec<usize>,
}

pub struct BasicMapGeneration {
    pub(super) context: MapGenerationContext,
    data: MapGenerationData,
    pub(super) map: Map,
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
            debug!(
                "max room stopping generation {} {}",
                self.map.rooms.len(),
                self.context.config.max_room
            );
            return None;
        }

        loop {
            if self.map.last_generated_room_index.is_none() {
                debug!("no room mark to continue generation");
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
                    debug!("no more room_possible stopping generation");
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
                    // D45 : `.nth(r)` (et non `.skip(r).last()`, qui rendait toujours la dernière
                    // connexion libre quel que soit le tirage).
                    let connection = previous_room
                        .connections
                        .iter()
                        .filter(|i| i.to.is_none())
                        .nth(rng.next_u32_range(0, free_connection_len as u32) as usize)
                        .unwrap();

                    previous_room_def.connections.get(connection.index).unwrap()
                };

                // D10 : le gabarit de départ (`Spawn`) n'est jamais réutilisé comme salle suivante
                // (la carte n'a qu'un point d'apparition). Une connexion sans autre gabarit
                // compatible est fermée comme sans gabarit du tout.
                let candidates: Vec<&(String, usize)> = connection_def
                    .compatiable_levels
                    .iter()
                    .filter(|(level_id, _)| {
                        self.context
                            .available_levels
                            .iter()
                            .find(|l| &l.level_id == level_id)
                            .is_none_or(|l| l.level_type != LevelType::Spawn)
                    })
                    .collect();

                if candidates.is_empty() {
                    previous_room
                        .connections
                        .get_mut(connection_def.index)
                        .unwrap()
                        .to = Some(ConnectionTo::DeadEnd);
                    debug!("no compatible levels marking as DeadEnd");
                    continue;
                } else {
                    // D45 : `.nth(r)`, même correction (toujours le dernier gabarit compatible avant).
                    let compatible_level = candidates
                        .get(rng.next_u32_range(0, candidates.len() as u32) as usize)
                        .copied()
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

                    // D46 : une salle qui chevaucherait une salle déjà placée ferme la connexion
                    // (comme une sortie de carte) au lieu d'être posée par-dessus.
                    let overlapping = self
                        .map
                        .rooms
                        .iter()
                        .any(|placed| new_room.is_overlapping(placed));
                    let previous_room = self.map.rooms.get_mut(previous_room_index).unwrap();
                    if overlapping {
                        previous_room
                            .connections
                            .get_mut(connection_def.index)
                            .unwrap()
                            .to = Some(ConnectionTo::DeadEnd);
                        continue;
                    }
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

/// Rounds `v` to the nearest multiple of `size` while staying inside `[min, max]`.
///
/// The bounds are themselves pulled to the multiples contained in the interval
/// whenever possible, so the clamp does not break the grid alignment; if the
/// interval is narrower than one tile (degenerate config), falls back to a plain
/// clamp.
fn snap_to_tile_grid(v: i32, min: i32, max: i32, size: i32) -> i32 {
    let snapped = (v + size / 2).div_euclid(size) * size;
    let min_aligned = (min + size - 1).div_euclid(size) * size;
    let max_aligned = max.div_euclid(size) * size;
    if min_aligned <= max_aligned {
        snapped.clamp(min_aligned, max_aligned)
    } else {
        snapped.clamp(min, max)
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

        // D45 : `.nth(r)` (toujours le dernier gabarit de départ avant).
        let spawning_room_def = spawning_levels
            .iter()
            .nth(rng.next_u32_range_inclusive(0, (spawning_levels.len() - 1) as u32) as usize);

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

        // Snap the spawn room position to the tile grid: every other room derives
        // from it through multiples of the tile size (get_connecting_room_position),
        // so an arbitrary position would shift the whole world by the same fractional
        // residue. Merged collision rectangles (physics) would then straddle flow
        // field cells without being registered in them: navigation routes enemies
        // through physically blocked cells and 2px slivers catch the 20px bodies,
        // softlocking the wave (validation seeds 16/17, m0-v7).
        let x = snap_to_tile_grid(
            x,
            -self.context.config.max_width,
            self.context.config.max_width - spawning_room_def.level_size_p.0,
            self.context.tile_size.0,
        );
        let y = snap_to_tile_grid(
            y,
            -self.context.config.max_heigth,
            self.context.config.max_heigth - spawning_room_def.level_size_p.1,
            self.context.tile_size.1,
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

    fn get_links(&mut self) -> Vec<(RoomConnection, RoomConnection)> {
        if !self.context.config.close_loops {
            return Vec::new();
        }
        let tile_size = self.context.tile_size;
        let mut linked = Vec::new();
        for i in 0..self.map.rooms.len() {
            for j in (i + 1)..self.map.rooms.len() {
                for a in 0..self.map.rooms[i].connections.len() {
                    if matches!(
                        self.map.rooms[i].connections[a].to,
                        Some(ConnectionTo::Room(_))
                    ) {
                        continue;
                    }
                    let (side_a, edge_a, start_a, end_a) =
                        self.map.rooms[i].opening_span(a, &tile_size);
                    let facing = (0..self.map.rooms[j].connections.len()).find(|&b| {
                        if matches!(
                            self.map.rooms[j].connections[b].to,
                            Some(ConnectionTo::Room(_))
                        ) {
                            return false;
                        }
                        let (side_b, edge_b, start_b, end_b) =
                            self.map.rooms[j].opening_span(b, &tile_size);
                        // Deux prises libres se font face si elles sont sur des côtés opposés, au
                        // même bord (salles bord contre bord) et à la même emprise. Deux salles
                        // séparées par du vide ne sont jamais reliées : la porte donnerait sur le
                        // vide (étages `Graph` à pose décalée, fusion de main 2026-10-19).
                        side_a.is_opposite(side_b)
                            && edge_a == edge_b
                            && start_a == start_b
                            && end_a == end_b
                    });
                    let Some(b) = facing else {
                        continue;
                    };
                    let iid_i = self.map.rooms[i].level_iid.clone();
                    let iid_j = self.map.rooms[j].level_iid.clone();
                    self.map.rooms[i].link_opening(a, &iid_j, b);
                    self.map.rooms[j].link_opening(b, &iid_i, a);
                    debug!(
                        "loop closed between {} ({:?}) and {} ({:?})",
                        iid_i,
                        side_a,
                        iid_j,
                        side_a.get_opposite()
                    );
                    linked.push((
                        self.map.rooms[i].connections[a].clone(),
                        self.map.rooms[j].connections[b].clone(),
                    ));
                }
            }
        }
        linked
    }

    fn unused_openings(&self) -> Vec<(String, Side, usize, usize)> {
        if !self.context.config.wall_unused_openings {
            return Vec::new();
        }
        self.map
            .rooms
            .iter()
            .flat_map(|room| {
                room.connections
                    .iter()
                    .filter(|c| !matches!(c.to, Some(ConnectionTo::Room(_))))
                    .map(|c| {
                        let def = &room.level_def.connections[c.index];
                        (room.level_iid.clone(), def.side, def.starting_at, def.size)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
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
        let mut removed: Vec<usize> = vec![]; // door indices at walled openings

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
                } else {
                    // Prise sans voisine (fermée, ou jamais tentée) : sa porte est retirée quand
                    // la prise est murée (`wall_unused_openings`), non interactive sinon.
                    for (door_index, (door_room_index, door_loc, _)) in all_doors.iter().enumerate()
                    {
                        if *door_room_index == room_index
                            && is_door_at_connection(door_loc, room, connection)
                        {
                            if self.context.config.wall_unused_openings {
                                removed.push(door_index);
                            } else if connection.to.is_some() {
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
            .enumerate()
            .filter(|(i, _)| !removed.contains(i))
            .map(|(_, (_, loc, config))| (loc, config))
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
        _rng: &mut RollbackRng,
    ) -> Vec<(EntityLocation, EnemySpawnConfig)> {
        self.map
            .rooms
            .iter()
            .flat_map(|room| {
                room.entity_locations
                    .zombie_spawns
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

    /// M2-T11 : mêmes règles que [`Self::get_character_spawns`] ci-dessus.
    fn get_props(&mut self, _rng: &mut RollbackRng) -> Vec<(EntityLocation, PropConfig)> {
        self.map
            .rooms
            .iter()
            .flat_map(|room| {
                room.entity_locations
                    .props
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
                    .collect::<Vec<(EntityLocation, PropConfig)>>()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        config::MapGenerationConfig,
        context::{Connection, Side},
        entity::location::EntityLocations,
    };

    fn level(id: &str, openings: &[(Side, usize)]) -> AvailableLevel {
        AvailableLevel {
            level_id: id.into(),
            level_size: (10, 10),
            level_size_p: (160, 160),
            level_type: LevelType::Normal,
            connections: openings
                .iter()
                .enumerate()
                .map(|(index, (side, starting_at))| Connection {
                    index,
                    size: 2,
                    side: *side,
                    starting_at: *starting_at,
                    level_id: id.into(),
                    compatiable_levels: vec![],
                })
                .collect(),
            entity_locations: EntityLocations {
                doors: openings
                    .iter()
                    .map(|(side, starting_at)| {
                        let start = *starting_at as i32;
                        let (position, size) = match side {
                            Side::N => (Position(start, 0), (2, 1)),
                            Side::S => (Position(start, 9), (2, 1)),
                            Side::W => (Position(0, start), (1, 2)),
                            Side::E => (Position(9, start), (1, 2)),
                        };
                        EntityLocation {
                            position,
                            size,
                            level_iid: id.into(),
                        }
                    })
                    .collect(),
                sodas: vec![],
                props: vec![],
                player_spawns: vec![],
                zombie_spawns: vec![],
                crates: vec![],
                weapons: vec![],
                windows: vec![],
                character_spawns: vec![],
            },
            room_kind: None,
        }
    }

    fn generator(rooms: Vec<(AvailableLevel, Position)>) -> BasicMapGeneration {
        let mut rng = RollbackRng::new(1);
        let context = MapGenerationContext {
            tile_size: (16, 16),
            level_size: (10, 10),
            available_levels: vec![],
            config: MapGenerationConfig::default(),
        };
        let mut generator = BasicMapGeneration::create(context);
        for (level, position) in rooms {
            generator.map.rooms.push(Room::create(
                &mut rng,
                Rc::new(level),
                position,
                Default::default(),
            ));
            generator.map.room_depths.push(0);
        }
        generator
    }

    /// Deux pièces dont les prises libres se font face à travers du vide ne sont pas reliées :
    /// seule une prise bord contre bord ferme une boucle.
    #[test]
    fn fermeture_des_boucles_exige_le_contact() {
        // A (0,0) avec E@4 ; B (176,0) avec W@4 : une tuile de vide entre les deux.
        let mut gap = generator(vec![
            (level("A", &[(Side::E, 4)]), Position(0, 0)),
            (level("B", &[(Side::W, 4)]), Position(176, 0)),
        ]);
        assert!(gap.get_links().is_empty());
        assert_eq!(
            gap.unused_openings().len(),
            2,
            "les deux prises sont murées"
        );
    }

    /// Une boucle fermée ouvre le passage : la porte de chaque prise reliée est appariée
    /// (interactable) comme une connexion d'arbre, jamais retirée.
    #[test]
    fn boucle_fermee_ouvre_les_portes_apparies() {
        let mut generator = generator(vec![
            (level("A", &[(Side::E, 4)]), Position(0, 0)),
            (level("B", &[(Side::W, 4)]), Position(160, 0)),
        ]);
        assert_eq!(generator.get_links().len(), 1);
        let mut rng = RollbackRng::new(1);
        let doors = generator.get_doors(&mut rng);
        // Les deux prises ont chacune une porte, appariée et interactive (pas retirée).
        assert_eq!(doors.len(), 2, "deux portes aux prises de la boucle");
        assert!(
            doors
                .iter()
                .all(|(_, config)| config.interactable && config.paired_door.is_some()),
            "chaque porte de boucle est appariée et interactive : {:?}",
            doors
                .iter()
                .map(|(_, c)| (c.interactable, c.paired_door.is_some()))
                .collect::<Vec<_>>()
        );
    }

    /// Deux pièces posées bord à bord dont les prises libres se font face sont reliées ; une
    /// prise qui fait face à un mur plein ou à une prise décalée reste libre et sera murée.
    #[test]
    fn fermeture_des_boucles_prises_en_face() {
        // A (0,0) avec E@4 ; B (160,0) avec W@4 (en face) et S@4 ; C (160,160) avec N@6
        // (décalée : pas en face de B.S) et W@2 ; D (0,160) avec E@2 (en face de C.W).
        let mut generator = generator(vec![
            (level("A", &[(Side::E, 4)]), Position(0, 0)),
            (level("B", &[(Side::W, 4), (Side::S, 4)]), Position(160, 0)),
            (
                level("C", &[(Side::N, 6), (Side::W, 2)]),
                Position(160, 160),
            ),
            (level("D", &[(Side::E, 2)]), Position(0, 160)),
        ]);
        let linked = generator.get_links();
        let pairs: Vec<(String, String)> = linked
            .iter()
            .map(|(a, b)| (a.level_id.clone(), b.level_id.clone()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("A".to_string(), "B".to_string()),
                ("C".to_string(), "D".to_string())
            ]
        );
        let unused: Vec<(String, Side)> = generator
            .unused_openings()
            .into_iter()
            .map(|(iid, side, _, _)| {
                let room = generator
                    .map
                    .rooms
                    .iter()
                    .find(|r| r.level_iid == iid)
                    .unwrap();
                (room.level_def.level_id.clone(), side)
            })
            .collect();
        assert_eq!(
            unused,
            vec![("B".to_string(), Side::S), ("C".to_string(), Side::N)]
        );
    }

    /// Les deux passes sont idempotentes et ne relient jamais une prise déjà reliée.
    #[test]
    fn fermeture_des_boucles_ignore_les_prises_reliees() {
        let mut generator = generator(vec![
            (level("A", &[(Side::E, 4)]), Position(0, 0)),
            (level("B", &[(Side::W, 4)]), Position(160, 0)),
        ]);
        assert_eq!(generator.get_links().len(), 1);
        assert!(generator.get_links().is_empty());
        assert!(generator.unused_openings().is_empty());
    }
}
