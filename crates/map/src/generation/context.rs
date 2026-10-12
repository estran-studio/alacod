use bevy_fixed::rng::RollbackRng;

use super::{config::MapGenerationConfig, entity::location::EntityLocations};

use std::{fmt::Display, rc::Rc, usize};

// This package does the conversion between ldtk map and my AvailableLevel struct to be used by the algo

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    N,
    S,
    W,
    E,
}

impl Side {
    pub fn get_opposite(&self) -> Self {
        match self {
            Side::N => Side::S,
            Side::S => Side::N,
            Side::W => Side::E,
            Side::E => Side::W,
        }
    }

    pub fn to_dir_str(&self) -> &'static str {
        match self {
            Side::N => "n",
            Side::E => "e",
            Side::S => "s",
            Side::W => "w",
        }
    }

    pub fn get_factor(&self) -> i32 {
        match self {
            Side::N | Side::W => -1,
            Side::S | Side::E => 1,
        }
    }

    pub fn is_opposite(&self, other: Side) -> bool {
        other == self.get_opposite()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LevelType {
    Spawn,
    Normal,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub index: usize,
    pub size: usize,
    pub side: Side,
    pub starting_at: usize,

    //pub level_iid: String,
    pub level_id: String,
    pub compatiable_levels: Vec<(String, usize)>,
    //pub to: Option<ConnectionTo>,
}

impl Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "side={:?} starting_at={} size={}",
            self.size, self.starting_at, self.size
        )
    }
}

impl Connection {
    fn are_matching(&self, other: &Connection) -> bool {
        self.side.is_opposite(other.side)
            && self.starting_at == other.starting_at
            && self.size == other.size
    }
}

#[derive(Debug, Clone)]
pub struct AvailableLevel {
    // identifier of the original level
    pub level_id: String,

    //pub level_iid: String,

    // level size in tile
    pub level_size: (usize, usize),

    // level size in px
    pub level_size_p: (i32, i32),

    // type of level
    pub level_type: LevelType,

    pub connections: Vec<Connection>,

    pub entity_locations: EntityLocations,

    /// M2-T10 : type de salle du gabarit (champ de niveau LDtk `room_kind`, M2-E1,
    /// `docs/conventions.md` §35), `None` si le niveau n'en déclare pas. Lu seulement par la
    /// grammaire d'étage ([`crate::generation::floor`]) ; `Basic` l'ignore.
    pub room_kind: Option<String>,
}

pub type AvailableLevels = Vec<Rc<AvailableLevel>>;

pub fn scan_width_side(
    connections: &mut Vec<Connection>,
    index: &mut usize,
    level: &AvailableLevel,
    level_size: &(usize, usize),
    grid: &Vec<&[i32]>,
    row_index: usize,
    side: Side,
) {
    let mut i = 0;
    while i < level_size.0 {
        if grid[row_index][i] == 1 {
            let mut size = 1;
            while size + i < level_size.0 {
                if grid[row_index][size + i] == 0 {
                    break;
                }
                size += 1;
            }

            connections.push(Connection {
                index: *index,
                size,
                side,
                starting_at: i,
                level_id: level.level_id.clone(),
                compatiable_levels: vec![],
            });

            *index += 1;

            i += size;
        } else {
            i += 1;
        }
    }
}

pub fn scan_height_side(
    connections: &mut Vec<Connection>,
    index: &mut usize,
    level: &AvailableLevel,
    level_size: &(usize, usize),
    grid: &Vec<&[i32]>,
    column_index: usize,
    side: Side,
) {
    let mut i = 0;
    while i < level_size.1 {
        if grid[i][column_index] == 1 {
            let mut size = 1;
            while size + i < level_size.1 {
                if grid[size + i][column_index] == 0 {
                    break;
                }
                size += 1;
            }

            connections.push(Connection {
                index: *index,
                size,
                side,
                starting_at: i,
                level_id: level.level_id.clone(),
                compatiable_levels: vec![],
            });

            *index += 1;

            i += size;
        } else {
            i += 1;
        }
    }
}

pub fn populate_level_connections(available_levels: &mut Vec<AvailableLevel>) {
    let mut to_add_elements: Vec<(usize, usize, (AvailableLevel, usize))> = vec![];

    let mut i = 0;
    while i < available_levels.len() {
        let mut y = 0;

        while y < available_levels[i].connections.len() {
            let level = &available_levels[i];
            let connection = level.connections.get(y).unwrap();

            let mut ii = 1;

            while ii + i < available_levels.len() {
                let mut yy = 0;

                // D47 : plus de saut des gabarits `Spawn` ici. Le `continue` sans `ii += 1`
                // bouclait à l'infini dès qu'un `Spawn` n'était pas premier ; et le saut ne valait
                // que dans un sens (un `Spawn` en tête restait compatible avec tout). Toutes les
                // cartes ont leur unique `Spawn` en tête : l'appariement est inchangé pour elles,
                // et ne dépend plus de l'ordre des gabarits.
                while yy < available_levels[ii + i].connections.len() {
                    let other_level = &available_levels[ii + i];

                    let other_connection = other_level.connections.get(yy).unwrap();

                    if connection.are_matching(other_connection) {
                        //if other_level.level_type != LevelType::Spawn {
                        to_add_elements.push((i, y, (available_levels[ii + i].clone(), yy)));
                        //}

                        // adding otherlevel to add to level
                        //if level.level_type != LevelType::Spawn {
                        to_add_elements.push((i + ii, yy, (available_levels[i].clone(), y)));
                        //}
                    }

                    yy += 1;
                }

                ii += 1;
            }

            y += 1;
        }

        i += 1;
    }

    for to_add in to_add_elements {
        available_levels[to_add.0].connections[to_add.1]
            .compatiable_levels
            .push((to_add.2 .0.level_id.clone(), to_add.2 .1));
    }
}

pub struct MapGenerationContext {
    pub tile_size: (i32, i32),
    pub level_size: (i32, i32),

    pub available_levels: AvailableLevels,

    pub config: MapGenerationConfig,
}

#[derive(Default)]
pub struct MapGenerationData {
    // TODO change for trait to be able to replace for unit test
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locations() -> EntityLocations {
        EntityLocations {
            doors: vec![],
            sodas: vec![],
            player_spawns: vec![],
            zombie_spawns: vec![],
            crates: vec![],
            weapons: vec![],
            windows: vec![],
            character_spawns: vec![],
        }
    }

    fn level(id: &str, level_type: LevelType, sides: &[Side]) -> AvailableLevel {
        AvailableLevel {
            level_id: id.into(),
            level_size: (10, 10),
            level_size_p: (160, 160),
            level_type,
            connections: sides
                .iter()
                .enumerate()
                .map(|(index, side)| Connection {
                    index,
                    size: 2,
                    side: *side,
                    starting_at: 4,
                    level_id: id.into(),
                    compatiable_levels: vec![],
                })
                .collect(),
            entity_locations: locations(),
            room_kind: None,
        }
    }

    fn compat(levels: &[AvailableLevel]) -> Vec<(String, usize, Vec<(String, usize)>)> {
        let mut out: Vec<_> = levels
            .iter()
            .flat_map(|l| {
                l.connections.iter().map(|c| {
                    let mut v = c.compatiable_levels.clone();
                    v.sort();
                    (l.level_id.clone(), c.index, v)
                })
            })
            .collect();
        out.sort();
        out
    }

    /// D47 : un gabarit `Spawn` qui n'est pas premier ne fait plus boucler l'appariement, et
    /// l'appariement ne dépend plus de l'ordre des gabarits.
    #[test]
    fn appariement_independant_de_la_place_du_spawn() {
        let a = || level("A", LevelType::Normal, &[Side::E, Side::S]);
        let s = || level("S", LevelType::Spawn, &[Side::W, Side::N]);
        let b = || level("B", LevelType::Normal, &[Side::W]);

        let mut spawn_premier = vec![s(), a(), b()];
        populate_level_connections(&mut spawn_premier);
        let mut spawn_milieu = vec![a(), s(), b()];
        populate_level_connections(&mut spawn_milieu);
        let mut spawn_dernier = vec![a(), b(), s()];
        populate_level_connections(&mut spawn_dernier);

        let attendu = compat(&spawn_premier);
        assert_eq!(attendu, compat(&spawn_milieu));
        assert_eq!(attendu, compat(&spawn_dernier));
        // A.E (0) s'apparie aux deux W (S.0, B.0) ; A.S (1) à S.N (1).
        assert!(attendu.contains(&("A".into(), 0, vec![("B".into(), 0), ("S".into(), 0)])));
        assert!(attendu.contains(&("A".into(), 1, vec![("S".into(), 1)])));
    }
}
