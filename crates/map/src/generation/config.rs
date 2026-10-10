use std::ops::RangeInclusive;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MapGenerationMode {
    Basic,
    /// T1.6 : caverne générée par automate cellulaire (`world::cave`), un seul niveau écrit
    /// dans le gabarit LDtk désigné par `map_path` (`map_ldtk::generation::cave`). La graine
    /// est `seed`, comme pour `Basic`.
    Cave(world::CaveConfig),
    /// M2-T10 : étage assemblé par une grammaire (`generation::floor`) : types de salles
    /// requis, boss en cul-de-sac à distance maximale. `Basic` reste inchangé.
    Floor(super::floor::FloorGrammar),
}

#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct MapGenerationConfig {
    pub map_path: String,

    pub seed: i32,

    pub max_width: i32,
    pub max_heigth: i32,

    pub max_room: usize,

    pub mode: MapGenerationMode,
}

impl Default for MapGenerationConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            max_width: 1000,
            max_heigth: 1000,
            map_path: "".into(),
            max_room: 10,
            mode: MapGenerationMode::Basic,
        }
    }
}

impl MapGenerationConfig {
    pub fn get_range_x(&self, my_size: i32) -> RangeInclusive<i32> {
        -self.max_width..=(self.max_width - my_size)
    }

    pub fn get_range_y(&self, my_size: i32) -> RangeInclusive<i32> {
        -self.max_heigth..=(self.max_heigth - my_size)
    }
}
