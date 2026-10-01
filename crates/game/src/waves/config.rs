//! Wave spawning configuration loaded from RON files.
//!
//! GGRS CRITICAL: All fields use deterministic types (no f32/f64 for game logic).

use bevy::{platform::collections::HashMap, prelude::*, reflect::TypePath};
use bevy_fixed::fixed_math;
use serde::{Deserialize, Serialize};

/// Enemy spawn probability tier based on wave number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveTier {
    /// Maximum wave number for this tier (inclusive)
    pub max_wave: u32,
    /// Enemy type probabilities (enemy_type_name -> weight)
    /// Weights are relative, not percentages
    pub enemy_probabilities: HashMap<String, u32>,
}

/// Main wave configuration loaded from RON file
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct WaveConfig {
    // === Enemy Count Formula ===
    // count = base_enemies + ((wave - 1) * enemies_per_wave) + random(0, max_variance)
    /// Base number of enemies for wave 1
    pub base_enemies: u32,
    /// Additional enemies per wave after wave 1
    pub enemies_per_wave: u32,
    /// Maximum random variance added to enemy count
    pub max_random_variance: u32,

    // === Timing (in frames at 60fps) ===
    /// Minimum delay after all enemies killed before next wave starts
    pub min_wave_delay_frames: u32,
    /// Grace period before spawning starts (player prep time)
    pub grace_period_frames: u32,

    // === Spawning Constraints ===
    /// Maximum enemies alive at any time
    pub max_concurrent_enemies: u32,
    /// Maximum enemies to spawn per batch
    pub spawn_batch_size: u32,
    /// Frames between spawn batches
    pub spawn_interval_frames: u32,

    // === Spawner Selection ===
    /// Minimum distance from any player for spawner activation
    pub min_player_distance: fixed_math::Fixed,
    /// Maximum distance from nearest player for spawner activation
    pub max_player_distance: fixed_math::Fixed,

    // === Enemy Type Probabilities ===
    /// Wave tiers defining enemy probabilities at different wave ranges
    pub wave_tiers: Vec<WaveTier>,

    // === Scaling ===
    /// Health multiplier increase per wave (e.g., 0.05 = +5% per wave)
    pub health_multiplier_per_wave: fixed_math::Fixed,
    /// Damage multiplier increase per wave (e.g., 0.03 = +3% per wave)
    pub damage_multiplier_per_wave: fixed_math::Fixed,

    /// Vague de victoire (T2.4, chantier F1, `run::modes::RunModeRules for RunMode`) :
    /// `Some(n)` déclare que la partie est gagnée dès que `WaveState::current_wave >= n` ;
    /// `None` (défaut, `#[serde(default)]`) : le mode `Waves` n'a jamais de victoire, comme
    /// avant ce chantier (partie infinie jusqu'à la défaite). Pas encore utilisé par aucun
    /// `wave_config.ron` existant.
    #[serde(default)]
    pub max_wave: Option<u32>,
}

impl Default for WaveConfig {
    fn default() -> Self {
        let mut tier_0_probs = HashMap::default();
        tier_0_probs.insert("zombie_full".to_string(), 80);
        tier_0_probs.insert("zombie_1".to_string(), 20);

        let mut tier_1_probs = HashMap::default();
        tier_1_probs.insert("zombie_full".to_string(), 50);
        tier_1_probs.insert("zombie_1".to_string(), 35);
        tier_1_probs.insert("zombie_2".to_string(), 15);

        let mut tier_2_probs = HashMap::default();
        tier_2_probs.insert("zombie_full".to_string(), 30);
        tier_2_probs.insert("zombie_1".to_string(), 40);
        tier_2_probs.insert("zombie_2".to_string(), 30);

        Self {
            // Enemy count formula
            base_enemies: 6,
            enemies_per_wave: 4,
            max_random_variance: 2,

            // Timing (at 60fps)
            min_wave_delay_frames: 600, // 10 seconds
            grace_period_frames: 180,   // 3 seconds

            // Spawning constraints
            max_concurrent_enemies: 20,
            spawn_batch_size: 3,
            spawn_interval_frames: 30, // 0.5 seconds between batches

            // Spawner selection
            min_player_distance: fixed_math::new(150.0),
            max_player_distance: fixed_math::new(600.0),

            // Enemy probabilities by tier
            wave_tiers: vec![
                WaveTier {
                    max_wave: 3,
                    enemy_probabilities: tier_0_probs,
                },
                WaveTier {
                    max_wave: 7,
                    enemy_probabilities: tier_1_probs,
                },
                WaveTier {
                    max_wave: 999,
                    enemy_probabilities: tier_2_probs,
                },
            ],

            // Scaling
            health_multiplier_per_wave: fixed_math::new(0.05),
            damage_multiplier_per_wave: fixed_math::new(0.03),

            max_wave: None,
        }
    }
}

impl WaveConfig {
    /// Get the wave tier for a given wave number
    pub fn get_tier(&self, wave: u32) -> Option<&WaveTier> {
        self.wave_tiers.iter().find(|t| wave <= t.max_wave)
    }

    /// Calculate the total enemy count for a wave
    pub fn calculate_enemy_count(&self, wave: u32, variance: u32) -> u32 {
        let base = self.base_enemies;
        let scaling = wave.saturating_sub(1) * self.enemies_per_wave;
        let clamped_variance = variance.min(self.max_random_variance);
        base + scaling + clamped_variance
    }

    /// Calculate health multiplier for a wave (as Fixed, 1.0 = 100%)
    pub fn calculate_health_multiplier(&self, wave: u32) -> fixed_math::Fixed {
        let wave_bonus =
            self.health_multiplier_per_wave * fixed_math::Fixed::from_num(wave.saturating_sub(1));
        fixed_math::FIXED_ONE + wave_bonus
    }

    /// Calculate damage multiplier for a wave (as Fixed, 1.0 = 100%)
    pub fn calculate_damage_multiplier(&self, wave: u32) -> fixed_math::Fixed {
        let wave_bonus =
            self.damage_multiplier_per_wave * fixed_math::Fixed::from_num(wave.saturating_sub(1));
        fixed_math::FIXED_ONE + wave_bonus
    }
}
