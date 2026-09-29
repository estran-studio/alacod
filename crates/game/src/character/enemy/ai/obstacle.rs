//! Generic Obstacle System
//!
//! Replaces hardcoded Window/Wall logic with a flexible obstacle system
//! that supports different types of blocking entities with various properties.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::frame_events::FrameEvents;

/// Type of obstacle - determines default behavior and appearance
/// GGRS: PartialOrd + Ord required for BTreeMap in FlowFieldCache
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect, Serialize, Deserialize, Default)]
pub enum ObstacleType {
    /// Solid wall - never breakable, always blocks
    #[default]
    Wall,
    /// Window - breakable, allows attack through, blocks movement
    Window,
    /// Barricade - breakable, blocks attack, blocks movement
    Barricade,
    /// Water - blocks ground units, not flying
    Water,
    /// Pit/hole - blocks ground units, not flying
    Pit,
    /// Low cover - can shoot over, blocks movement for ground
    LowCover,
}

impl ObstacleType {
    /// Returns true if this obstacle type is typically breakable
    pub fn is_breakable(&self) -> bool {
        matches!(self, ObstacleType::Window | ObstacleType::Barricade)
    }

    /// Returns true if attacks can pass through this obstacle
    pub fn allows_attack_through(&self) -> bool {
        matches!(self, ObstacleType::Window | ObstacleType::LowCover)
    }

    /// Returns true if this blocks ground movement by default
    pub fn blocks_ground(&self) -> bool {
        !matches!(self, ObstacleType::LowCover)
    }

    /// Returns true if this blocks flying movement
    pub fn blocks_flying(&self) -> bool {
        matches!(self, ObstacleType::Wall)
    }
}

/// Generic obstacle component that replaces hardcoded Window behavior
#[derive(Component, Clone, Debug, Hash, Reflect, Serialize, Deserialize)]
pub struct Obstacle {
    /// Type of obstacle (determines default behaviors)
    pub obstacle_type: ObstacleType,
    /// Whether this obstacle blocks movement
    pub blocks_movement: bool,
    /// Whether attacks can pass through this obstacle
    pub allows_attack_through: bool,
    /// Whether this obstacle can be destroyed
    pub breakable: bool,
    /// Current health (if breakable)
    pub health: Option<u32>,
    /// Maximum health (if breakable)
    pub max_health: Option<u32>,
}

impl Default for Obstacle {
    fn default() -> Self {
        Self {
            obstacle_type: ObstacleType::Wall,
            blocks_movement: true,
            allows_attack_through: false,
            breakable: false,
            health: None,
            max_health: None,
        }
    }
}

impl Obstacle {
    /// Create a new obstacle with default properties for the given type
    pub fn new(obstacle_type: ObstacleType) -> Self {
        match obstacle_type {
            ObstacleType::Wall => Self {
                obstacle_type,
                blocks_movement: true,
                allows_attack_through: false,
                breakable: false,
                health: None,
                max_health: None,
            },
            ObstacleType::Window => Self {
                obstacle_type,
                blocks_movement: true,
                allows_attack_through: true,
                breakable: true,
                health: Some(3),
                max_health: Some(3),
            },
            ObstacleType::Barricade => Self {
                obstacle_type,
                blocks_movement: true,
                allows_attack_through: false,
                breakable: true,
                health: Some(5),
                max_health: Some(5),
            },
            ObstacleType::Water => Self {
                obstacle_type,
                blocks_movement: true,
                allows_attack_through: true,
                breakable: false,
                health: None,
                max_health: None,
            },
            ObstacleType::Pit => Self {
                obstacle_type,
                blocks_movement: true,
                allows_attack_through: true,
                breakable: false,
                health: None,
                max_health: None,
            },
            ObstacleType::LowCover => Self {
                obstacle_type,
                blocks_movement: true,
                allows_attack_through: true,
                breakable: true,
                health: Some(2),
                max_health: Some(2),
            },
        }
    }

    /// Create a window obstacle
    pub fn window() -> Self {
        Self::new(ObstacleType::Window)
    }

    /// Create a barricade obstacle
    pub fn barricade() -> Self {
        Self::new(ObstacleType::Barricade)
    }

    /// Create with custom health
    pub fn with_health(mut self, health: u32) -> Self {
        self.health = Some(health);
        self.max_health = Some(health);
        self.breakable = true;
        self
    }

    /// Apply damage to this obstacle, returns true if destroyed
    pub fn take_damage(&mut self, damage: u32) -> bool {
        if let Some(ref mut health) = self.health {
            *health = health.saturating_sub(damage);
            if *health == 0 {
                self.blocks_movement = false;
                return true;
            }
        }
        false
    }

    /// Check if this obstacle is destroyed
    pub fn is_destroyed(&self) -> bool {
        self.health == Some(0)
    }

    /// Check if this obstacle is intact (has health or is not breakable)
    pub fn is_intact(&self) -> bool {
        match self.health {
            Some(h) => h > 0,
            None => true, // Non-breakable obstacles are always "intact"
        }
    }
}

/// RON configuration for obstacle definitions
#[derive(Clone, Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct ObstacleConfig {
    pub obstacle_type: ObstacleType,
    pub blocks_movement: Option<bool>,
    pub allows_attack_through: Option<bool>,
    pub breakable: Option<bool>,
    pub health: Option<u32>,
}

impl From<&ObstacleConfig> for Obstacle {
    fn from(config: &ObstacleConfig) -> Self {
        let mut obstacle = Obstacle::new(config.obstacle_type);

        if let Some(blocks) = config.blocks_movement {
            obstacle.blocks_movement = blocks;
        }
        if let Some(allows) = config.allows_attack_through {
            obstacle.allows_attack_through = allows;
        }
        if let Some(breakable) = config.breakable {
            obstacle.breakable = breakable;
        }
        if let Some(health) = config.health {
            obstacle.health = Some(health);
            obstacle.max_health = Some(health);
        }

        obstacle
    }
}

/// Attaque d'un obstacle, émise par l'IA et appliquée dans la même frame
/// par [`process_obstacle_damage`] (voir [`FrameEvents`]).
#[derive(Clone, Debug)]
pub struct ObstacleAttackEvent {
    pub attacker: Entity,
    pub obstacle: Entity,
    pub damage: u32,
}

/// Hash manuel : `attacker`/`obstacle` sont des `Entity` (différents d'un client à
/// l'autre) sans `GgrsNetId` compagnon sur cet événement, contrairement à
/// `InteractionEvent`/`MeleeHitbox` ; ils sont donc exclus faute de mieux. Seul `damage`
/// contribue au checksum : une divergence sur QUEL obstacle est visé, à dégâts égaux,
/// resterait invisible (dette, voir le rapport de migration).
impl std::hash::Hash for ObstacleAttackEvent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.damage.hash(state);
    }
}

/// System to process obstacle damage
/// Also syncs with WindowHealth for legacy compatibility
///
/// A destroyed obstacle keeps its collider: `blocks_movement` becomes false, so enemies
/// walk through it (their collision ignores obstacles that no longer block), while
/// players are still stopped by a broken window. Repairing it blocks enemies again.
pub fn process_obstacle_damage(
    frame: Res<utils::frame::FrameCount>,
    attack_events: Res<FrameEvents<ObstacleAttackEvent>>,
    mut obstacle_query: Query<(&utils::net_id::GgrsNetId, &mut Obstacle, Option<&mut map::game::entity::map::window::WindowHealth>)>,
) {
    for event in attack_events.iter() {
        if let Ok((net_id, mut obstacle, window_health_opt)) = obstacle_query.get_mut(event.obstacle) {
            let destroyed = obstacle.take_damage(event.damage);

            // Sync with WindowHealth if present (legacy compatibility)
            if let Some(mut window_health) = window_health_opt {
                window_health.current = window_health.current.saturating_sub(event.damage as u8);
            }

            if destroyed {
                info!("ggrs{{f={} obstacle_destroyed net_id={}}}", frame.frame, net_id.0);
            }
        }
    }
}
