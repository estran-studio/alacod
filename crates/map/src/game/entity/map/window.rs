use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::generation::entity::window::WindowConfig;

#[derive(Default, Component, Reflect)]
pub struct WindowComponent {
    pub config: WindowConfig,
}

/// Component that tracks window health and repair state
/// This is a rollback component for deterministic gameplay
#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct WindowHealth {
    /// Current health (0-3)
    pub current: u8,
    /// Maximum health (typically 3)
    pub max: u8,
    /// Frame at which the window can be repaired again (for timeout)
    pub can_repair_after_frame: Option<u32>,
}

impl Default for WindowHealth {
    fn default() -> Self {
        Self {
            current: 0, // Start with no health for testing
            max: 3,
            can_repair_after_frame: None,
        }
    }
}
