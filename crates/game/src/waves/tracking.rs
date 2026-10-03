//! Wave enemy tracking components.
//!
//! GGRS CRITICAL: Components inherit rollback from entity registration.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for enemies spawned by the wave system.
///
/// This allows the wave system to:
/// - Track which enemies belong to the current wave
/// - Detect when all wave enemies are killed
/// - Apply wave-specific modifiers
#[derive(Component, Debug, Clone, Hash, Serialize, Deserialize, Default, Reflect)]
pub struct WaveEnemy {
    /// Wave number when this enemy was spawned
    pub spawned_wave: u32,
}
