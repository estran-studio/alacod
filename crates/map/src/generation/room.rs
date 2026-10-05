use std::{collections::HashMap, rc::Rc};

use bevy_fixed::rng::RollbackRng;
use serde_json::Value;

use super::{
    config::MapGenerationConfig,
    context::{AvailableLevel, Connection, Side},
    entity::location::EntityLocations,
    position::Position,
};

#[derive(Debug, Clone, Default)]
pub enum ConnectionTo {
    Room((String, usize)),
    #[default]
    DeadEnd,
    OutSide,
}

#[derive(Debug, Clone)]
pub struct RoomConnection {
    pub index: usize,
    pub level_iid: String,
    pub level_id: String,
    pub side: Side,
    pub to: Option<ConnectionTo>,
}

/*
 * Room is an instance of a level in a map that is being generated
 */
#[derive(Debug, Clone)]
pub struct Room {
    pub level_iid: String,
    pub position: Position,
    pub connections: Vec<RoomConnection>,
    pub entity_locations: EntityLocations,

    pub level_def: Rc<AvailableLevel>,

    pub properties: HashMap<String, Value>,
}

impl Room {
    pub fn create(
        rng: &mut RollbackRng,
        level: Rc<AvailableLevel>,
        position: Position,
        properties: HashMap<String, Value>,
    ) -> Self {
        let level_iid: String = rng.next_uuid();

        let connections: Vec<_> = level
            .connections
            .iter()
            .map(|x| RoomConnection {
                index: x.index,
                to: None,
                level_id: x.level_id.clone(),
                level_iid: level_iid.clone(),
                side: x.side,
            })
            .collect();

        Self {
            level_iid,
            position,
            entity_locations: level.entity_locations.clone(),
            connections,
            properties,
            level_def: level,
        }
    }

    fn set_connection(
        &mut self,
        my_connection_index: usize,
        their_room: &mut Room,
        their_connection_index: usize,
    ) {
        let my_connection = self.connections.get_mut(my_connection_index).unwrap();
        if my_connection.to.is_some() {
            panic!("connection is already used");
        }
        my_connection.to = Some(ConnectionTo::Room((
            their_room.level_iid.clone(),
            their_connection_index,
        )));
    }

    pub fn set_connection_between(
        &mut self,
        my_connection_index: usize,
        their_room: &mut Room,
        their_connection_index: usize,
    ) {
        // throw if one is already link
        self.set_connection(my_connection_index, their_room, their_connection_index);
        their_room.set_connection(their_connection_index, self, my_connection_index);
    }

    /// D46 : deux salles se chevauchent si leurs rectangles partagent une surface ; deux salles
    /// **adjacentes** (bords qui se touchent, le cas de toute connexion) ne se chevauchent pas
    /// (`<=` ; avec `<`, toute salle voisine aurait été déclarée en chevauchement).
    pub fn is_overlapping(&self, other: &Room) -> bool {
        // find if we are overlapping
        let left_of_other = self.position.0 + self.level_def.level_size_p.0 <= other.position.0;
        let left_of_self = other.position.0 + other.level_def.level_size_p.0 <= self.position.0;

        // Check if one square is above the other
        let above_other = self.position.1 + self.level_def.level_size_p.1 <= other.position.1;
        let above_self = other.position.1 + other.level_def.level_size_p.1 <= self.position.1;

        // If neither square is to the left or above the other, they overlap
        !(left_of_other || left_of_self || above_other || above_self)
    }

    // check if top-left corner is outside or not
    pub fn is_outside(&self, config: &MapGenerationConfig) -> bool {
        let position = &self.position;

        (position.0 > config.max_width || position.0 < -config.max_width)
            || (position.1 > config.max_heigth || position.1 < -config.max_heigth)
    }

    pub fn get_connecting_room_position(
        &self,
        my_connection: &Connection,
        their_level: &AvailableLevel,
        their_connection: usize,
        tile_size: &(i32, i32),
    ) -> Position {
        let my_position = &self.position;

        let their_connection = their_level.connections.get(their_connection).unwrap();

        let offset = my_connection.starting_at - their_connection.starting_at;

        // calculate the pixel offset
        match my_connection.side {
            Side::N | Side::S => {
                let offset_pixel = (offset as i32) * tile_size.0;

                Position(
                    my_position.0 + (their_connection.side.get_factor() * (offset_pixel)),
                    my_position.1
                        + (-their_connection.side.get_factor() * their_level.level_size_p.1),
                )
            }
            Side::W | Side::E => {
                let offset_pixel = (offset as i32) * tile_size.1;

                Position(
                    my_position.0
                        + (-their_connection.side.get_factor() * their_level.level_size_p.0),
                    my_position.1 + (their_connection.side.get_factor() * (offset_pixel)),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::context::LevelType;

    fn room(x: i32, y: i32, w: i32, h: i32) -> Room {
        let level = AvailableLevel {
            level_id: "L".into(),
            level_size: (w as usize / 16, h as usize / 16),
            level_size_p: (w, h),
            level_type: LevelType::Normal,
            connections: vec![],
            entity_locations: EntityLocations {
                doors: vec![],
                sodas: vec![],
                player_spawns: vec![],
                zombie_spawns: vec![],
                crates: vec![],
                weapons: vec![],
                windows: vec![],
                character_spawns: vec![],
            },
        };
        Room {
            level_iid: "iid".into(),
            position: Position(x, y),
            connections: vec![],
            entity_locations: level.entity_locations.clone(),
            level_def: Rc::new(level),
            properties: HashMap::new(),
        }
    }

    /// D46 : des salles voisines (bord commun) ne se chevauchent pas ; une surface commune, si.
    #[test]
    fn chevauchement_exclut_les_salles_adjacentes() {
        let a = room(0, 0, 160, 160);
        assert!(
            !a.is_overlapping(&room(160, 0, 160, 160)),
            "voisine à l'est"
        );
        assert!(
            !a.is_overlapping(&room(-160, 0, 160, 160)),
            "voisine à l'ouest"
        );
        assert!(!a.is_overlapping(&room(0, 160, 160, 160)), "voisine au sud");
        assert!(
            !a.is_overlapping(&room(0, -160, 160, 160)),
            "voisine au nord"
        );
        assert!(!a.is_overlapping(&room(160, 160, 160, 160)), "coin");
        assert!(
            a.is_overlapping(&room(144, 0, 160, 160)),
            "une colonne commune"
        );
        assert!(a.is_overlapping(&room(32, 32, 32, 32)), "incluse");
        assert!(room(32, 32, 32, 32).is_overlapping(&a), "symétrie");
        assert!(a.is_overlapping(&a.clone()), "même place");
    }
}
