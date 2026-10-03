//! Données communes aux systèmes d'armes et aux personnages, extraites sans changement de valeurs.
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct Player {
    pub handle: PlayerHandle,
    pub color: Color,
    pub name: String,
    pub pubkey: String,
}

/// Hash manuel : seul `handle` contribue au checksum GGRS. `color` est de la présentation
/// (`f32`, pas de `Hash`) ; `name` et `pubkey` sont des données d'identité **propres à
/// chaque client** en p2p (un client ne connaît que son propre nom, les autres joueurs
/// reçoivent un nom de repli) : les hacher faisait diverger les checksums de tous les
/// clients dès la frame 0 (test p2p headless de la CI de nuit, T2.14) sans que la
/// simulation diffère. Elles ne pilotent rien dans la simulation.
impl std::hash::Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Hash)]
#[reflect(Component)]
pub struct Enemy {}
#[derive(Component, Default, Clone, Debug, Hash)]
pub struct SprintState {
    pub is_sprinting: bool,
    pub sprint_factor: fixed_math::Fixed, // Ranges from 0.0 to 1.0 for gradual acceleration
}

#[derive(Component, Default, Clone, Debug, Hash)]
pub struct Velocity {
    pub main: fixed_math::FixedVec2,
    pub knockback: fixed_math::FixedVec2,
}

#[derive(Component, Default, Clone, Debug, Hash)]
pub struct DashState {
    pub is_dashing: bool,
    pub dash_direction: fixed_math::FixedVec2,
    pub dash_frames_remaining: u32,
    pub dash_cooldown_remaining: u32,
    pub dash_distance_per_frame: fixed_math::Fixed, // Distance to move each frame
    pub dash_start_position: fixed_math::FixedVec3, // Starting position for the dash
    pub dash_total_distance: fixed_math::Fixed,     // Total distance for current dash
}

impl DashState {
    pub fn can_dash(&self) -> bool {
        !self.is_dashing && self.dash_cooldown_remaining == 0
    }

    pub fn start_dash(
        &mut self,
        direction: fixed_math::FixedVec2,
        start_position: fixed_math::FixedVec3,
        total_distance: fixed_math::Fixed,
        duration_frames: u32,
    ) {
        // Ensure duration is at least 1 to prevent division by zero
        let safe_duration = duration_frames.max(1);

        self.is_dashing = true;
        self.dash_direction = direction.normalize_or_zero();
        self.dash_frames_remaining = safe_duration;
        self.dash_start_position = start_position;
        self.dash_total_distance = total_distance;
        self.dash_distance_per_frame = total_distance / fixed_math::new(safe_duration as f32);
    }

    pub fn update(&mut self) {
        if self.is_dashing {
            self.dash_frames_remaining = self.dash_frames_remaining.saturating_sub(1);
            if self.dash_frames_remaining == 0 {
                self.is_dashing = false;
            }
        }

        if self.dash_cooldown_remaining > 0 {
            self.dash_cooldown_remaining = self.dash_cooldown_remaining.saturating_sub(1);
        }
    }

    pub fn set_cooldown(&mut self, cooldown_frames: u32) {
        self.dash_cooldown_remaining = cooldown_frames;
    }
}
#[derive(Component, Clone, Debug, Serialize, Default, Deserialize, Hash)]
pub struct Health {
    pub current: fixed_math::Fixed,
    pub max: fixed_math::Fixed,
    pub invulnerable_until_frame: Option<u32>, // Optional invulnerability window
}

impl fmt::Display for Health {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HP: {}/{}", self.current, self.max)?;
        if let Some(frame) = self.invulnerable_until_frame {
            write!(f, " (Invulnerable until frame {})", frame)?;
        }
        Ok(())
    }
}

pub const INPUT_UP: u16 = 1 << 0;
pub const INPUT_DOWN: u16 = 1 << 1;
pub const INPUT_LEFT: u16 = 1 << 2;
pub const INPUT_RIGHT: u16 = 1 << 3;
pub const INPUT_RELOAD: u16 = 1 << 4;
pub const INPUT_SWITCH_WEAPON_MODE: u16 = 1 << 5;
pub const INPUT_SPRINT: u16 = 1 << 6;
pub const INPUT_DASH: u16 = 1 << 7;
pub const INPUT_MODIFIER: u16 = 1 << 8;
pub const INPUT_INTERACTION: u16 = 1 << 9;
pub const INPUT_MELEE_ATTACK: u16 = 1 << 10;
pub const INPUT_FORCE_CRASH: u16 = 1 << 11;
/// Lâche l'arme active au sol (T2.2, chantier B7). Touche `G` (`Devices`, voir
/// `character::player::control::get_input_map`), bouton `DropWeapon` des scénarios (voir
/// `game::replay::Button`).
pub const INPUT_DROP_WEAPON: u16 = 1 << 12;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoxInput {
    pub buttons: u16,
    pub pan_x: i16,
    pub pan_y: i16,

    pub fire: bool,
    pub switch_weapon: bool,
}

/// Component for the weapon sprite's position relative to player
///
/// Lu par `system_weapon_position` (rotation de l'arme, `GgrsSchedule`) mais écrit par
/// `apply_inputs` seulement hors dash : il est donc de l'état rollback, enregistré par
/// `BaseCharacterGamePlugin` (hors checksum, voir là).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
}

pub type BoxConfig = bevy_ggrs::GgrsConfig<BoxInput>;
pub type PeerConfig = bevy_ggrs::GgrsConfig<BoxInput, bevy_matchbox::prelude::PeerId>;
