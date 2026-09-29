//! Wave state machine for tracking wave progression.
//!
//! GGRS CRITICAL: This resource must be registered for rollback.

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use serde::{Deserialize, Serialize};

// Note: We don't derive Reflect for WaveState because Fixed doesn't implement Reflect.
// The rollback system uses Clone, which is sufficient.

/// Current phase of the wave system
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Default, Reflect)]
pub enum WavePhase {
    /// Initial state before first wave
    #[default]
    NotStarted,
    /// Grace period between waves (player prep time)
    GracePeriod,
    /// Actively spawning enemies
    Spawning,
    /// All enemies spawned, waiting for kills
    InProgress,
    /// All wave enemies killed, waiting for delay
    WaveComplete,
}

/// Wave system state resource
///
/// GGRS CRITICAL: Must be registered with `.rollback_and_trace_resource::<WaveState>()`
#[derive(Resource, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct WaveState {
    /// Current phase of the wave
    pub phase: WavePhase,
    /// Current wave number (1-indexed)
    pub current_wave: u32,

    // === Spawning Tracking ===

    /// Remaining enemies to spawn this wave
    pub enemies_to_spawn: u32,
    /// Enemies already spawned this wave
    pub enemies_spawned_this_wave: u32,
    /// Total enemies for current wave (before spawning starts)
    pub wave_enemy_count: u32,

    // === Statistics ===

    /// Total enemies killed across all waves
    pub total_enemies_killed: u32,
    /// Enemies killed in current wave
    pub wave_enemies_killed: u32,

    // === Timing (frame-based for determinism) ===

    /// Frame when current wave started
    pub wave_start_frame: u32,
    /// Frame of last spawn batch
    pub last_spawn_frame: u32,
    /// Frame when last enemy was killed
    pub last_enemy_killed_frame: u32,
    /// Frame when current phase started
    pub phase_start_frame: u32,

    // === Current Wave Modifiers ===

    /// Health multiplier for current wave (1.0 = 100%)
    pub current_health_multiplier: fixed_math::Fixed,
    /// Damage multiplier for current wave (1.0 = 100%)
    pub current_damage_multiplier: fixed_math::Fixed,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            phase: WavePhase::NotStarted,
            current_wave: 0,
            enemies_to_spawn: 0,
            enemies_spawned_this_wave: 0,
            wave_enemy_count: 0,
            total_enemies_killed: 0,
            wave_enemies_killed: 0,
            wave_start_frame: 0,
            last_spawn_frame: 0,
            last_enemy_killed_frame: 0,
            phase_start_frame: 0,
            current_health_multiplier: fixed_math::FIXED_ONE,
            current_damage_multiplier: fixed_math::FIXED_ONE,
        }
    }
}

impl WaveState {
    /// Check if wave system is active (any phase except NotStarted)
    pub fn is_active(&self) -> bool {
        self.phase != WavePhase::NotStarted
    }

    /// Check if currently spawning enemies
    pub fn is_spawning(&self) -> bool {
        self.phase == WavePhase::Spawning
    }

    /// Start the wave system (transition from NotStarted to GracePeriod)
    pub fn start(&mut self, frame: u32) {
        if self.phase == WavePhase::NotStarted {
            self.phase = WavePhase::GracePeriod;
            self.phase_start_frame = frame;
            self.current_wave = 1;
        }
    }

    /// Record an enemy kill
    pub fn record_kill(&mut self, frame: u32) {
        self.total_enemies_killed += 1;
        self.wave_enemies_killed += 1;
        self.last_enemy_killed_frame = frame;
    }

    /// Prepare for next wave
    pub fn prepare_next_wave(&mut self, enemy_count: u32, health_mult: fixed_math::Fixed, damage_mult: fixed_math::Fixed) {
        self.wave_enemy_count = enemy_count;
        self.enemies_to_spawn = enemy_count;
        self.enemies_spawned_this_wave = 0;
        self.wave_enemies_killed = 0;
        self.current_health_multiplier = health_mult;
        self.current_damage_multiplier = damage_mult;
    }
}
