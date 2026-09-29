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

                if available_levels[ii + i].level_type == LevelType::Spawn {
                    continue;
                }

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
