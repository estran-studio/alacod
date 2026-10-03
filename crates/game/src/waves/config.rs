//! Wave spawning configuration loaded from RON files.
//!
//! GGRS CRITICAL: All fields use deterministic types (no f32/f64 for game logic).
//!
//! F5 (chantier m0-v11) : les champs numériques sont des [`NumOrExpr`] — littéral
//! (inchangé) ou expression évaluée une fois au lancement (voir
//! `crate::balance::ResolvedWaveConfig` et `docs/conventions.md` §18). Les expressions ne
//! sont **jamais** évaluées ici ni dans l'état rollback : `resolve_balance_system` produit
//! des valeurs concrètes avant la première frame de simulation.

use bevy::{platform::collections::HashMap, prelude::*, reflect::TypePath};
use bevy_fixed::fixed_math;
use content::expr::NumOrExpr;
use serde::{Deserialize, Serialize};

/// Enemy spawn probability tier based on wave number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveTier {
    /// Maximum wave number for this tier (inclusive)
    pub max_wave: NumOrExpr,
    /// Enemy type probabilities (enemy_type_name -> weight)
    /// Weights are relative, not percentages
    pub enemy_probabilities: HashMap<String, NumOrExpr>,
}

/// Main wave configuration loaded from RON file
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct WaveConfig {
    // === Enemy Count Formula ===
    // count = base_enemies + ((wave - 1) * enemies_per_wave) + random(0, max_variance)
    /// Base number of enemies for wave 1
    pub base_enemies: NumOrExpr,
    /// Additional enemies per wave after wave 1
    pub enemies_per_wave: NumOrExpr,
    /// Maximum random variance added to enemy count
    pub max_random_variance: NumOrExpr,

    // === Timing (in frames at 60fps) ===
    /// Minimum delay after all enemies killed before next wave starts
    pub min_wave_delay_frames: NumOrExpr,
    /// Grace period before spawning starts (player prep time)
    pub grace_period_frames: NumOrExpr,

    // === Spawning Constraints ===
    /// Maximum enemies alive at any time
    pub max_concurrent_enemies: NumOrExpr,
    /// Maximum enemies to spawn per batch
    pub spawn_batch_size: NumOrExpr,
    /// Frames between spawn batches
    pub spawn_interval_frames: NumOrExpr,

    // === Spawner Selection ===
    /// Minimum distance from any player for spawner activation
    pub min_player_distance: NumOrExpr,
    /// Maximum distance from nearest player for spawner activation
    pub max_player_distance: NumOrExpr,

    // === Enemy Type Probabilities ===
    /// Wave tiers defining enemy probabilities at different wave ranges
    pub wave_tiers: Vec<WaveTier>,

    // === Scaling ===
    /// Health multiplier increase per wave (e.g., 0.05 = +5% per wave)
    pub health_multiplier_per_wave: NumOrExpr,
    /// Damage multiplier increase per wave (e.g., 0.03 = +3% per wave)
    pub damage_multiplier_per_wave: NumOrExpr,

    /// Vague de victoire (T2.4, chantier F1, `run::modes::RunModeRules for RunMode`) :
    /// `Some(n)` déclare que la partie est gagnée dès que `WaveState::current_wave >= n` ;
    /// `None` (défaut, `#[serde(default)]`) : le mode `Waves` n'a jamais de victoire, comme
    /// avant ce chantier (partie infinie jusqu'à la défaite). Pas encore utilisé par aucun
    /// `wave_config.ron` existant.
    #[serde(default)]
    pub max_wave: Option<NumOrExpr>,
}

impl Default for WaveConfig {
    fn default() -> Self {
        let mut tier_0_probs = HashMap::default();
        tier_0_probs.insert("zombie_full".to_string(), NumOrExpr::Integer(80));
        tier_0_probs.insert("zombie_1".to_string(), NumOrExpr::Integer(20));

        let mut tier_1_probs = HashMap::default();
        tier_1_probs.insert("zombie_full".to_string(), NumOrExpr::Integer(50));
        tier_1_probs.insert("zombie_1".to_string(), NumOrExpr::Integer(35));
        tier_1_probs.insert("zombie_2".to_string(), NumOrExpr::Integer(15));

        let mut tier_2_probs = HashMap::default();
        tier_2_probs.insert("zombie_full".to_string(), NumOrExpr::Integer(30));
        tier_2_probs.insert("zombie_1".to_string(), NumOrExpr::Integer(40));
        tier_2_probs.insert("zombie_2".to_string(), NumOrExpr::Integer(30));

        Self {
            // Enemy count formula
            base_enemies: NumOrExpr::Integer(6),
            enemies_per_wave: NumOrExpr::Integer(4),
            max_random_variance: NumOrExpr::Integer(2),

            // Timing (at 60fps)
            min_wave_delay_frames: NumOrExpr::Integer(600), // 10 seconds
            grace_period_frames: NumOrExpr::Integer(180),   // 3 seconds

            // Spawning constraints
            max_concurrent_enemies: NumOrExpr::Integer(20),
            spawn_batch_size: NumOrExpr::Integer(3),
            spawn_interval_frames: NumOrExpr::Integer(30), // 0.5 seconds between batches

            // Spawner selection
            min_player_distance: NumOrExpr::Literal(fixed_math::new(150.0)),
            max_player_distance: NumOrExpr::Literal(fixed_math::new(600.0)),

            // Enemy probabilities by tier
            wave_tiers: vec![
                WaveTier {
                    max_wave: NumOrExpr::Integer(3),
                    enemy_probabilities: tier_0_probs,
                },
                WaveTier {
                    max_wave: NumOrExpr::Integer(7),
                    enemy_probabilities: tier_1_probs,
                },
                WaveTier {
                    max_wave: NumOrExpr::Integer(999),
                    enemy_probabilities: tier_2_probs,
                },
            ],

            // Scaling
            health_multiplier_per_wave: NumOrExpr::Literal(fixed_math::new(0.05)),
            damage_multiplier_per_wave: NumOrExpr::Literal(fixed_math::new(0.03)),

            max_wave: None,
        }
    }
}
