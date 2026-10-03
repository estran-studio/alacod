//! Wave-based enemy spawning system for Call of Duty Zombies-like gameplay.
//!
//! # Overview
//!
//! This module implements a wave-based spawning system with:
//! - Hybrid wave triggers (kill-based + minimum delay)
//! - Formula-based scaling from RON configuration
//! - Integration with existing LDTK spawners
//! - Full GGRS rollback compatibility
//!
//! # Wave Flow
//!
//! ```text
//! NotStarted → GracePeriod → Spawning → InProgress → WaveComplete
//!                   ↑                                      │
//!                   └──────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! Add the `WaveSystemPlugin` to your app and enable wave mode:
//!
//! ```rust,ignore
//! app.add_plugins(WaveSystemPlugin);
//! app.insert_resource(WaveModeEnabled(true));
//! ```
//!
//! # Configuration
//!
//! Configure via `waves/wave_config.ron` dans les assets du jeu. See `config::WaveConfig` for options.

pub mod config;
pub mod debug;
pub mod state;
pub mod systems;
pub mod tracking;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ggrs::GgrsSchedule;

use crate::character::enemy::spawning::enemy_spawn_from_spawners_system;
use crate::character::health::{rollback_apply_accumulated_damage, rollback_apply_death};
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;

pub use config::WaveConfig;
pub use debug::{WaveDebugEnabled, WaveDebugPlugin};
pub use state::{WavePhase, WaveState};
pub use tracking::WaveEnemy;

/// Resource to enable/disable wave spawning mode.
///
/// When enabled, the wave system handles all enemy spawning.
/// When disabled, the original spawner system is used.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WaveModeEnabled(pub bool);

/// Plugin that adds the wave spawning system.
pub struct WaveSystemPlugin;

impl Plugin for WaveSystemPlugin {
    fn build(&self, app: &mut App) {
        // RON asset plugin for wave configuration
        app.add_plugins(RonAssetPlugin::<WaveConfig>::new(&["ron"]));

        // Debug UI plugin
        app.add_plugins(WaveDebugPlugin);

        // Resources
        app.init_resource::<WaveState>();
        app.init_resource::<WaveModeEnabled>();

        // Register types for reflection (useful for debugging)
        app.register_type::<WavePhase>();
        app.register_type::<WaveEnemy>();

        // Rollback registration
        app.rollback_and_trace_resource::<WaveState>();
        app.rollback_and_trace::<WaveEnemy>();
        app.rollback_and_trace::<tracking::SpawnFallback>();

        // Wave state and spawning systems run in EnemySpawning set
        // Use ambiguous_with to mark intentional ambiguity with old spawner
        // (they have mutually exclusive run conditions so never actually conflict)
        app.add_systems(
            GgrsSchedule,
            (
                systems::wave_state_machine_system
                    .run_if(wave_mode_enabled)
                    .ambiguous_with(enemy_spawn_from_spawners_system),
                systems::wave_spawning_system
                    .run_if(wave_mode_enabled)
                    .after(systems::wave_state_machine_system)
                    .ambiguous_with(enemy_spawn_from_spawners_system),
            )
                .in_set(RollbackSystemSet::EnemySpawning),
        );

        // Death tracking runs in DeathManagement AFTER damage is applied (Death added)
        // but BEFORE rollback_apply_death (entity despawned)
        app.add_systems(
            GgrsSchedule,
            systems::wave_enemy_death_tracking_system
                .run_if(wave_mode_enabled)
                .after(rollback_apply_accumulated_damage)
                .before(rollback_apply_death)
                .in_set(RollbackSystemSet::DeathManagement),
        );
    }
}

/// Run condition: wave mode is enabled
fn wave_mode_enabled(wave_mode: Res<WaveModeEnabled>) -> bool {
    wave_mode.0
}
