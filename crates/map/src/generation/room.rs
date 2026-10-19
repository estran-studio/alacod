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

    /// Emprise d'une prise dans le monde : côté, coordonnée du bord (x pour E/W, y pour N/S,
    /// bord **extérieur** du rectangle : `x + largeur` pour E) et intervalle `[début, fin)` en
    /// pixels le long de ce bord. Deux prises de pièces voisines se font face quand leurs côtés
    /// sont opposés et ces trois valeurs égales (fermeture des boucles).
    pub fn opening_span(&self, index: usize, tile_size: &(i32, i32)) -> (Side, i32, i32, i32) {
        let def = &self.level_def.connections[index];
        let (w, h) = self.level_def.level_size_p;
        match def.side {
            Side::N | Side::S => {
                let edge = if def.side == Side::N {
                    self.position.1
                } else {
                    self.position.1 + h
                };
                let start = self.position.0 + def.starting_at as i32 * tile_size.0;
                (def.side, edge, start, start + def.size as i32 * tile_size.0)
            }
            Side::W | Side::E => {
                let edge = if def.side == Side::W {
                    self.position.0
                } else {
                    self.position.0 + w
                };
                let start = self.position.1 + def.starting_at as i32 * tile_size.1;
                (def.side, edge, start, start + def.size as i32 * tile_size.1)
            }
        }
    }

    /// Relie une prise à une pièce déjà posée, même si elle avait été fermée (`DeadEnd`,
    /// `OutSide`) faute de pièce à y poser : fermeture des boucles.
    pub fn link_opening(&mut self, index: usize, their_iid: &str, their_index: usize) {
        self.connections[index].to = Some(ConnectionTo::Room((their_iid.to_string(), their_index)));
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

    /// Position d'une pièce à coller sur ma prise `my_connection` par sa prise
    /// `their_connection` : bord à bord (le bord opposé de l'autre pièce touche le mien), les
    /// deux prises exactement en face (même emprise le long du bord, `opening_span`).
    ///
    /// 2026-10-17 : avant, la coordonnée perpendiculaire était `± leur taille` pour les côtés
    /// S et E (juste seulement pour des pièces de même taille, le cas de toutes les cartes
    /// d'alors) et le décalage le long du bord changeait de signe selon le côté (sans effet
    /// tant que `aligned_openings` imposait un décalage nul).
    pub fn get_connecting_room_position(
        &self,
        my_connection: &Connection,
        their_level: &AvailableLevel,
        their_connection: usize,
        tile_size: &(i32, i32),
    ) -> Position {
        let my_position = &self.position;
        let (my_w, my_h) = self.level_def.level_size_p;
        let (their_w, their_h) = their_level.level_size_p;
        let their_connection = their_level.connections.get(their_connection).unwrap();

        // Le long du bord : ma prise commence à `my.start`, la leur à `their.start` (en cases
        // depuis le coin haut-gauche de chaque pièce) ; elles coïncident si l'autre pièce est
        // décalée de la différence.
        let offset = my_connection.starting_at as i32 - their_connection.starting_at as i32;

        match my_connection.side {
            Side::N => Position(
                my_position.0 + offset * tile_size.0,
                my_position.1 - their_h,
            ),
            Side::S => Position(my_position.0 + offset * tile_size.0, my_position.1 + my_h),
            Side::W => Position(
                my_position.0 - their_w,
                my_position.1 + offset * tile_size.1,
            ),
            Side::E => Position(my_position.0 + my_w, my_position.1 + offset * tile_size.1),
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
                props: vec![],
                player_spawns: vec![],
                zombie_spawns: vec![],
                crates: vec![],
                weapons: vec![],
                windows: vec![],
                character_spawns: vec![],
            },
            room_kind: None,
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

    fn level_with(w: i32, h: i32, side: Side, starting_at: usize) -> AvailableLevel {
        let mut level = (*room(0, 0, w, h).level_def).clone();
        level.connections = vec![Connection {
            index: 0,
            size: 2,
            side,
            starting_at,
            level_id: "L".into(),
            compatiable_levels: vec![],
        }];
        level
    }

    /// Une pièce collée par une prise a cette prise exactement en face de la mienne, quelles
    /// que soient les tailles et les décalages (`aligned_openings: false`).
    #[test]
    fn collage_bord_a_bord_prises_en_face() {
        let tile = (16, 16);
        for (my_side, my_start, their_side, their_start) in [
            (Side::N, 12, Side::S, 3),
            (Side::S, 12, Side::N, 3),
            (Side::W, 8, Side::E, 2),
            (Side::E, 8, Side::W, 2),
        ] {
            let mut me = room(100, 200, 416, 304);
            let my_level = level_with(416, 304, my_side, my_start);
            me.level_def = Rc::new(my_level);
            let theirs = level_with(160, 112, their_side, their_start);
            let my_connection = me.level_def.connections[0].clone();
            let position = me.get_connecting_room_position(&my_connection, &theirs, 0, &tile);
            let mut placed = room(position.0, position.1, 160, 112);
            placed.level_def = Rc::new(theirs);
            let (side_a, edge_a, start_a, end_a) = me.opening_span(0, &tile);
            let (side_b, edge_b, start_b, end_b) = placed.opening_span(0, &tile);
            assert!(side_a.is_opposite(side_b));
            assert_eq!(
                (edge_a, start_a, end_a),
                (edge_b, start_b, end_b),
                "{my_side:?}"
            );
            assert!(!me.is_overlapping(&placed), "{my_side:?}");
        }
    }
}
