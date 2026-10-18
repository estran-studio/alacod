//! Carte de brouillard générée (données pures) : grille de visibilité, salles et portes.
//!
//! `map_ldtk` la produit à la génération et la pose en ressource ; `room_fog` (présentation) la lit.
//! Aucune dépendance de rendu : ce fichier est de la simulation.

use std::collections::BTreeSet;

use bevy::prelude::{Resource, Vec2};
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FogSettings {
    pub enabled: bool,
    pub view_tiles: i32,
    pub window_tiles: i32,
    pub transition_seconds: f32,
    pub edge_pixels: f32,
    pub explored_opacity: f32,
}
impl Default for FogSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            view_tiles: 20,
            window_tiles: 8,
            transition_seconds: 0.2,
            edge_pixels: 8.0,
            explored_opacity: 0.72,
        }
    }
}

/// Generated map metadata. Grid coordinates are LDtk top-down, world origin is bottom-left.
/// Stable room ids and door endpoints are retained independently of spawner defence areas.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct FogMap {
    pub width: usize,
    pub height: usize,
    pub tile_size: i32,
    pub origin: [i32; 2],
    pub walls: Vec<bool>,
    pub rooms: Vec<FogRoom>,
    pub doors: Vec<FogDoor>,
    pub windows: Vec<[i32; 4]>,
    pub settings: FogSettings,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FogRoom {
    pub id: String,
    pub rect: [i32; 4],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FogDoor {
    pub rooms: [String; 2],
    pub rect: [i32; 4],
}

impl FogMap {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.width == 0
            || self.height == 0
            || self.width > 1024
            || self.height > 1024
            || self.tile_size <= 0
            || self.walls.len() != self.width * self.height
        {
            return Err("invalid visibility grid");
        }
        let ids: BTreeSet<_> = self.rooms.iter().map(|r| r.id.as_str()).collect();
        if ids.len() != self.rooms.len()
            || self.doors.iter().any(|d| {
                d.rooms[0] == d.rooms[1] || d.rooms.iter().any(|id| !ids.contains(id.as_str()))
            })
        {
            return Err("invalid room connections");
        }
        let s = &self.settings;
        if !(1..=128).contains(&s.view_tiles)
            || !(1..=s.view_tiles).contains(&s.window_tiles)
            || !s.transition_seconds.is_finite()
            || !(0.0..=5.0).contains(&s.transition_seconds)
            || !s.edge_pixels.is_finite()
            || !(0.0..=16.0).contains(&s.edge_pixels)
            || !s.explored_opacity.is_finite()
            || !(0.0..=1.0).contains(&s.explored_opacity)
        {
            return Err("invalid fog settings");
        }
        Ok(())
    }
    pub fn index(&self, p: (i32, i32)) -> Option<usize> {
        (p.0 >= 0 && p.1 >= 0 && p.0 < self.width as i32 && p.1 < self.height as i32)
            .then(|| p.1 as usize * self.width + p.0 as usize)
    }
    pub fn world_cell(&self, p: Vec2) -> (i32, i32) {
        let x = ((p.x - self.origin[0] as f32) / self.tile_size as f32).floor() as i32;
        let y = ((p.y - self.origin[1] as f32) / self.tile_size as f32).floor() as i32;
        (x, self.height as i32 - 1 - y)
    }
    pub fn fixed_cell(&self, p: FixedVec2) -> (i32, i32) {
        let size = Fixed::from_num(self.tile_size);
        let x = ((p.x - Fixed::from_num(self.origin[0])) / size)
            .floor()
            .to_num::<i32>();
        let y = ((p.y - Fixed::from_num(self.origin[1])) / size)
            .floor()
            .to_num::<i32>();
        (x, self.height as i32 - 1 - y)
    }
}
