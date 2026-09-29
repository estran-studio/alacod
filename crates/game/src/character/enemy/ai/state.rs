//! Monster State Machine
//!
//! Generic state machine for enemy AI, replacing the hardcoded ZombieState.
//! Supports composable behaviors through configuration.

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utils::net_id::GgrsNetId;

use super::navigation::NavProfile;
use super::obstacle::ObstacleType;

/// Generic monster state - replaces ZombieState
#[derive(Component, Clone, Debug, Hash, PartialEq, Eq, Reflect, Serialize, Deserialize, Default)]
pub enum MonsterState {
    /// Waiting or wandering randomly
    #[default]
    Idle,
    /// Following the flow field toward target
    Chasing,
    /// Attacking a target (player or obstacle)
    Attacking {
        target: AttackTarget,
        last_attack_frame: u32,
    },
    /// Stunned or knocked back (recovering)
    Stunned {
        recover_at_frame: u32,
    },
    /// Special state for breaking through obstacles
    Breaching {
        obstacle: GgrsNetId,
        start_frame: u32,
    },
    /// Fleeing from target (low health behavior)
    Fleeing,
    /// Dead but not yet despawned
    Dead,
}

/// Target for attacks
#[derive(Clone, Debug, Hash, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum AttackTarget {
    /// Attacking a player
    Player { net_id: GgrsNetId },
    /// Attacking an obstacle (window, barricade, etc.)
    Obstacle { net_id: GgrsNetId },
    /// Attacking at a position (for area attacks)
    Position { x: i32, y: i32 },
}

/// Movement type for enemies
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Reflect, Serialize, Deserialize)]
pub enum MovementType {
    /// Standard ground movement
    #[default]
    Ground,
    /// Flying - ignores water, pits
    Flying,
    /// Phasing - passes through most obstacles
    Phasing,
}

impl From<MovementType> for NavProfile {
    fn from(movement: MovementType) -> Self {
        match movement {
            MovementType::Ground => NavProfile::Ground,
            MovementType::Flying => NavProfile::Flying,
            MovementType::Phasing => NavProfile::Phasing,
        }
    }
}

/// AI configuration for an enemy - loaded from RON
#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct EnemyAiConfig {
    /// Movement type determines pathfinding behavior
    pub movement_type: MovementType,
    /// Range at which enemy becomes aggressive
    pub aggro_range: fixed_math::Fixed,
    /// Range at which enemy can attack
    pub attack_range: fixed_math::Fixed,
    /// Frames between attacks
    pub attack_cooldown_frames: u32,
    /// Obstacle types this enemy can break
    pub can_break: Vec<ObstacleType>,
    /// Obstacle types this enemy can attack through
    pub attack_through: Vec<ObstacleType>,
    /// Obstacle types this enemy ignores for pathfinding
    pub ignores: Vec<ObstacleType>,
    /// Whether to use GroundBreaker profile (path through breakables)
    pub path_through_breakables: bool,
    /// Optional flee threshold (0.0-1.0 health percentage)
    pub flee_threshold: Option<fixed_math::Fixed>,
    /// Damage dealt per attack
    pub attack_damage: fixed_math::Fixed,
}

impl Default for EnemyAiConfig {
    fn default() -> Self {
        Self {
            movement_type: MovementType::Ground,
            aggro_range: fixed_math::new(300.0),
            attack_range: fixed_math::new(35.0),
            attack_cooldown_frames: 60,
            can_break: vec![ObstacleType::Window],
            attack_through: vec![ObstacleType::Window],
            ignores: vec![],
            path_through_breakables: false,
            flee_threshold: None,
            attack_damage: fixed_math::new(10.0),
        }
    }
}

impl EnemyAiConfig {
    /// Get the navigation profile for this enemy
    pub fn nav_profile(&self) -> NavProfile {
        if self.path_through_breakables {
            NavProfile::GroundBreaker
        } else {
            self.movement_type.into()
        }
    }

    /// Check if this enemy can break a given obstacle type
    pub fn can_break_obstacle(&self, obstacle_type: ObstacleType) -> bool {
        self.can_break.contains(&obstacle_type)
    }

    /// Check if this enemy can attack through a given obstacle type
    pub fn can_attack_through_obstacle(&self, obstacle_type: ObstacleType) -> bool {
        self.attack_through.contains(&obstacle_type)
    }

    /// Check if this enemy ignores a given obstacle type
    pub fn ignores_obstacle(&self, obstacle_type: ObstacleType) -> bool {
        self.ignores.contains(&obstacle_type)
    }

    /// Create a standard zombie configuration
    pub fn zombie() -> Self {
        Self {
            movement_type: MovementType::Ground,
            aggro_range: fixed_math::new(500.0),
            attack_range: fixed_math::new(40.0),
            attack_cooldown_frames: 60,
            can_break: vec![ObstacleType::Window, ObstacleType::Barricade],
            attack_through: vec![ObstacleType::Window],
            ignores: vec![],
            path_through_breakables: true, // Zombies path through breakables (windows)
            flee_threshold: None,
            attack_damage: fixed_math::new(10.0),
        }
    }

    /// Create a flying enemy configuration
    pub fn flying() -> Self {
        Self {
            movement_type: MovementType::Flying,
            aggro_range: fixed_math::new(400.0),
            attack_range: fixed_math::new(50.0),
            attack_cooldown_frames: 45,
            can_break: vec![],
            attack_through: vec![],
            ignores: vec![ObstacleType::Water, ObstacleType::Pit],
            path_through_breakables: false,
            flee_threshold: None,
            attack_damage: fixed_math::new(8.0),
        }
    }

    /// Create a ghost configuration
    pub fn ghost() -> Self {
        Self {
            movement_type: MovementType::Phasing,
            aggro_range: fixed_math::new(350.0),
            attack_range: fixed_math::new(30.0),
            attack_cooldown_frames: 90,
            can_break: vec![],
            attack_through: vec![
                ObstacleType::Window,
                ObstacleType::Barricade,
                ObstacleType::LowCover,
            ],
            ignores: vec![
                ObstacleType::Window,
                ObstacleType::Barricade,
                ObstacleType::Water,
                ObstacleType::Pit,
                ObstacleType::LowCover,
            ],
            path_through_breakables: false,
            flee_threshold: None,
            attack_damage: fixed_math::new(15.0),
        }
    }

    /// Create a tank zombie that smashes through obstacles
    pub fn tank() -> Self {
        Self {
            movement_type: MovementType::Ground,
            aggro_range: fixed_math::new(400.0),
            attack_range: fixed_math::new(50.0),
            attack_cooldown_frames: 90,
            can_break: vec![
                ObstacleType::Window,
                ObstacleType::Barricade,
                ObstacleType::LowCover,
            ],
            attack_through: vec![],
            ignores: vec![],
            path_through_breakables: true, // Uses GroundBreaker profile
            flee_threshold: None,
            attack_damage: fixed_math::new(25.0),
        }
    }
}

/// RON configuration for enemy AI (for data-driven setup)
#[derive(Clone, Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct EnemyAiConfigRon {
    pub movement_type: Option<MovementType>,
    pub aggro_range: Option<String>, // Fixed as string "300.0"
    pub attack_range: Option<String>,
    pub attack_cooldown_frames: Option<u32>,
    pub can_break: Option<Vec<ObstacleType>>,
    pub attack_through: Option<Vec<ObstacleType>>,
    pub ignores: Option<Vec<ObstacleType>>,
    pub path_through_breakables: Option<bool>,
    pub flee_threshold: Option<String>,
    pub attack_damage: Option<String>,
}

impl From<&EnemyAiConfigRon> for EnemyAiConfig {
    fn from(ron: &EnemyAiConfigRon) -> Self {
        let mut config = EnemyAiConfig::default();

        if let Some(movement) = ron.movement_type {
            config.movement_type = movement;
        }
        if let Some(ref range) = ron.aggro_range {
            if let Ok(val) = range.parse::<f32>() {
                config.aggro_range = fixed_math::new(val);
            } else {
                warn!("Failed to parse aggro_range '{}' from RON config.", range);
            }
        }
        if let Some(ref range) = ron.attack_range {
            if let Ok(val) = range.parse::<f32>() {
                config.attack_range = fixed_math::new(val);
            } else {
                warn!("Failed to parse attack_range '{}' from RON config.", range);
            }
        }
        if let Some(cooldown) = ron.attack_cooldown_frames {
            config.attack_cooldown_frames = cooldown;
        }
        if let Some(ref can_break) = ron.can_break {
            config.can_break = can_break.clone();
        }
        if let Some(ref attack_through) = ron.attack_through {
            config.attack_through = attack_through.clone();
        }
        if let Some(ref ignores) = ron.ignores {
            config.ignores = ignores.clone();
        }
        if let Some(path_through) = ron.path_through_breakables {
            config.path_through_breakables = path_through;
        }
        if let Some(ref threshold) = ron.flee_threshold {
            if let Ok(val) = threshold.parse::<f32>() {
                config.flee_threshold = Some(fixed_math::new(val));
            } else {
                warn!(
                    "Failed to parse flee_threshold '{}' from RON config.",
                    threshold
                );
            }
        }
        if let Some(ref damage) = ron.attack_damage {
            if let Ok(val) = damage.parse::<f32>() {
                config.attack_damage = fixed_math::new(val);
            } else {
                warn!("Failed to parse attack_damage '{}' from RON config.", damage);
            }
        }

        config
    }
}

/// Current target information for an enemy
#[derive(Component, Clone, Debug, Hash, Default, Serialize, Deserialize)]
pub struct EnemyTarget {
    /// The current target entity (if any)
    pub target: Option<GgrsNetId>,
    /// Type of the current target
    pub target_type: TargetType,
    /// Last known position of target
    pub last_known_position: Option<fixed_math::FixedVec2>,
}

/// Type of target being pursued
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Reflect, Serialize, Deserialize)]
pub enum TargetType {
    #[default]
    None,
    Player,
    Obstacle,
}
