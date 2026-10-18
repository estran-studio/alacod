//! État logique d'animation d'un personnage : orientation, animation courante, calques actifs.
//!
//! C'est de la **simulation** (rollback, checksum) : l'orientation commande la position de la
//! bouche de l'arme et la direction des coups de mêlée, l'état d'animation est lu par les
//! comportements. Ce qui dessine (feuilles de sprites, timers, `Handle`) reste dans le crate de
//! présentation `animation`, qui lit ces composants. Déménagé de `animation` (dette D69) :
//! `combat` ne dépend plus d'aucun crate de rendu. Les types, leur `Hash` et leur `Debug` sont
//! inchangés : aucune trace ne bouge.
//!
//! [`AnimationStatePlugin`] enregistre les trois composants en rollback. Il remplace
//! l'enregistrement qui vivait dans `animation::D2AnimationPlugin` et s'ajoute au même endroit
//! (avant `GgrsPlugin`, voir `game::core`), donc dans le même ordre d'enregistrement.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use utils::rollback::RollbackTraceApp;

#[derive(Component, Clone, Debug, Hash)]
pub struct ActiveLayers {
    // BTreeMap (pas HashMap) : ce composant est rollback, l'ordre d'itération doit être
    // stable entre clients.
    pub layers: BTreeMap<String, String>,
}

#[derive(Component, Reflect, Default, Clone, Debug, Hash, PartialEq, Eq)]
#[reflect(Component, PartialEq)] // Reflect needed for GGRS state hashing
pub struct AnimationState(pub String);

#[derive(
    Component, Reflect, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, Serialize, Deserialize,
)]
#[reflect(Component, PartialEq)]
pub enum FacingDirection {
    #[default]
    Right, // 0 degrees
    UpRight,   // 45 degrees
    Up,        // 90 degrees
    UpLeft,    // 135 degrees
    Left,      // 180 degrees
    DownLeft,  // 225 degrees
    Down,      // 270 degrees
    DownRight, // 315 degrees
}

impl FacingDirection {
    /// Returns -1 for left-ish directions, 1 for right-ish directions
    pub fn to_int(&self) -> i32 {
        match self {
            FacingDirection::Right | FacingDirection::UpRight | FacingDirection::DownRight => 1,
            FacingDirection::Left | FacingDirection::UpLeft | FacingDirection::DownLeft => -1,
            FacingDirection::Up | FacingDirection::Down => 0,
        }
    }

    /// Returns the angle in radians for this direction
    pub fn to_radians(&self) -> f32 {
        match self {
            FacingDirection::Right => 0.0,
            FacingDirection::UpRight => std::f32::consts::PI / 4.0,
            FacingDirection::Up => std::f32::consts::PI / 2.0,
            FacingDirection::UpLeft => 3.0 * std::f32::consts::PI / 4.0,
            FacingDirection::Left => std::f32::consts::PI,
            FacingDirection::DownLeft => 5.0 * std::f32::consts::PI / 4.0,
            FacingDirection::Down => 3.0 * std::f32::consts::PI / 2.0,
            FacingDirection::DownRight => 7.0 * std::f32::consts::PI / 4.0,
        }
    }

    /// Returns the unit vector for this direction
    pub fn to_vector(&self) -> bevy::math::Vec2 {
        let angle = self.to_radians();
        bevy::math::Vec2::new(angle.cos(), angle.sin())
    }

    /// Determines the facing direction from a 2D vector (using f32 for non-rollback systems)
    pub fn from_vector(vec: bevy::math::Vec2) -> Self {
        if vec.length_squared() < 0.001 {
            return FacingDirection::default();
        }

        let angle = vec.y.atan2(vec.x);
        let normalized_angle = if angle < 0.0 {
            angle + 2.0 * std::f32::consts::PI
        } else {
            angle
        };

        // Divide circle into 8 equal segments (45 degrees each)
        let segment = ((normalized_angle + std::f32::consts::PI / 8.0)
            / (std::f32::consts::PI / 4.0)) as u8
            % 8;

        match segment {
            0 => FacingDirection::Right,
            1 => FacingDirection::UpRight,
            2 => FacingDirection::Up,
            3 => FacingDirection::UpLeft,
            4 => FacingDirection::Left,
            5 => FacingDirection::DownLeft,
            6 => FacingDirection::Down,
            7 => FacingDirection::DownRight,
            _ => FacingDirection::Right,
        }
    }

    /// Determines the facing direction from a fixed-point 2D vector (for deterministic rollback systems)
    pub fn from_fixed_vector(vec: bevy_fixed::fixed_math::FixedVec2) -> Self {
        use bevy_fixed::fixed_math;

        // Un vecteur de plus de ~181 unités ferait déborder `length_squared` et `atan2` (panique) : la direction
        // seule compte, on le réduit d'un facteur 4 au-delà de 128 (inchangé en deçà : traces identiques).
        let vec = if vec.x.abs().max(vec.y.abs()) > fixed_math::new(128.0) {
            let four = fixed_math::new(4.0);
            bevy_fixed::fixed_math::FixedVec2::new(vec.x / four, vec.y / four)
        } else {
            vec
        };
        if vec.length_squared() < fixed_math::new(0.001) {
            return FacingDirection::default();
        }

        let angle = fixed_math::atan2_fixed(vec.y, vec.x);
        let two_pi = fixed_math::new(2.0) * fixed_math::FIXED_PI;
        let normalized_angle = if angle < fixed_math::FIXED_ZERO {
            angle + two_pi
        } else {
            angle
        };

        // Divide circle into 8 equal segments (45 degrees each)
        let pi_over_8 = fixed_math::FIXED_PI / fixed_math::new(8.0);
        let pi_over_4 = fixed_math::FIXED_PI / fixed_math::new(4.0);
        let segment = ((normalized_angle + pi_over_8) / pi_over_4).to_num::<u8>() % 8;

        match segment {
            0 => FacingDirection::Right,
            1 => FacingDirection::UpRight,
            2 => FacingDirection::Up,
            3 => FacingDirection::UpLeft,
            4 => FacingDirection::Left,
            5 => FacingDirection::DownLeft,
            6 => FacingDirection::Down,
            7 => FacingDirection::DownRight,
            _ => FacingDirection::Right,
        }
    }

    /// Check if this direction is primarily horizontal
    pub fn is_horizontal(&self) -> bool {
        matches!(self, FacingDirection::Left | FacingDirection::Right)
    }

    /// Check if sprite should be flipped horizontally
    pub fn should_flip_x(&self) -> bool {
        matches!(
            self,
            FacingDirection::Left | FacingDirection::UpLeft | FacingDirection::DownLeft
        )
    }
}

/// État d'animation logique : fait partie de la simulation (rollback).
#[derive(Bundle)]
pub struct AnimationStateBundle {
    state: AnimationState,
    active_layers: ActiveLayers,
    facing_direction: FacingDirection,
}

impl AnimationStateBundle {
    pub fn new(starting_layers: BTreeMap<String, String>) -> Self {
        Self {
            state: AnimationState("Idle".into()),
            active_layers: ActiveLayers {
                layers: starting_layers,
            },
            facing_direction: FacingDirection::default(),
        }
    }
}

/// Enregistre `AnimationState`, `FacingDirection` et `ActiveLayers` en rollback (checksum et
/// trace), dans cet ordre.
pub struct AnimationStatePlugin;

impl Plugin for AnimationStatePlugin {
    fn build(&self, app: &mut App) {
        app.rollback_and_trace::<AnimationState>()
            .rollback_and_trace::<FacingDirection>()
            .rollback_and_trace::<ActiveLayers>();
    }
}
