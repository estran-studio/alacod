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
    /// M2-T13 : carte telle quelle, sans assemblage ni réécriture (le hub, `docs/conventions.md`
    /// §57). Variante ajoutée en dernier.
    Static,
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct MapGenerationConfig {
    pub map_path: String,

    pub seed: i32,

    pub max_width: i32,
    pub max_heigth: i32,

    pub max_room: usize,

    /// Voir `world::MapSettings::aligned_openings`.
    #[serde(default = "default_true")]
    pub aligned_openings: bool,
    /// Voir `world::MapSettings::close_loops`.
    #[serde(default = "default_true")]
    pub close_loops: bool,
    /// Voir `world::MapSettings::wall_unused_openings`.
    #[serde(default = "default_true")]
    pub wall_unused_openings: bool,

    pub mode: MapGenerationMode,
}

fn default_true() -> bool {
    true
}

impl Default for MapGenerationConfig {
    fn default() -> Self {
        Self::from_settings(&world::MapSettings::default())
    }
}

impl MapGenerationConfig {
    /// Config d'une carte à partir des réglages du jeu (`entry.map` de `game.ron`) ; `map_path`
    /// vide, graine 1, mode `Basic`.
    pub fn from_settings(settings: &world::MapSettings) -> Self {
        Self {
            seed: 1,
            max_width: settings.max_width,
            max_heigth: settings.max_height,
            map_path: "".into(),
            max_room: settings.max_rooms,
            aligned_openings: settings.aligned_openings,
            close_loops: settings.close_loops,
            wall_unused_openings: settings.wall_unused_openings,
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
