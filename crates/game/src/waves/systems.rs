//! Wave spawning systems.
//!
//! GGRS CRITICAL: All systems must be deterministic.
//! See CLAUDE.md for GGRS rules.

use animation::SpriteSheetConfig;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_fixed::{fixed_math, rng::RollbackRng};
use map::game::entity::map::enemy_spawn::EnemySpawnerComponent;
use utils::{frame::FrameCount, net_id::{GgrsNetId, GgrsNetIdFactory}};

use crate::{
    character::{
        config::CharacterConfig,
        enemy::{create::spawn_enemy, Enemy},
        health::Death,
        player::Player,
    },
    collider::CollisionSettings,
    global_asset::GlobalAsset,
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};

/// Bundled parameters for enemy spawning (reduces parameter count)
#[derive(SystemParam)]
pub struct SpawnAssets<'w> {
    pub collision_settings: Res<'w, CollisionSettings>,
    pub weapons_asset: Res<'w, Assets<WeaponsConfig>>,
    pub melee_weapons_asset: Res<'w, Assets<MeleeWeaponsConfig>>,
    pub characters_asset: Res<'w, Assets<CharacterConfig>>,
    pub asset_server: Res<'w, AssetServer>,
    pub texture_atlas_layouts: ResMut<'w, Assets<TextureAtlasLayout>>,
    pub spritesheet_assets: Res<'w, Assets<SpriteSheetConfig>>,
}

use super::{
    config::WaveConfig,
    state::{WavePhase, WaveState},
    tracking::WaveEnemy,
};

/// System that manages wave state transitions.
///
/// Runs every frame to check conditions and advance the state machine.
pub fn wave_state_machine_system(
    frame: Res<FrameCount>,
    mut wave_state: ResMut<WaveState>,
    mut rng: ResMut<RollbackRng>,
    wave_config_assets: Res<Assets<WaveConfig>>,
    global_assets: Res<GlobalAsset>,
    wave_enemy_query: Query<Entity, (With<Enemy>, With<WaveEnemy>)>,
) {
    // Get config (handle may not be loaded yet)
    let Some(config) = global_assets
        .wave_config
        .as_ref()
        .and_then(|h| wave_config_assets.get(h))
    else {
        return;
    };

    let current_frame = frame.frame;
    let alive_wave_enemies = wave_enemy_query.iter().count() as u32;

    match wave_state.phase {
        WavePhase::NotStarted => {
            // Start first wave - transition to grace period
            wave_state.phase = WavePhase::GracePeriod;
            wave_state.phase_start_frame = current_frame;
            wave_state.current_wave = 1;

            // Calculate wave 1 enemies
            let variance = if config.max_random_variance > 0 {
                rng.next_u32_range(0, config.max_random_variance + 1)
            } else {
                0
            };
            let enemy_count = config.calculate_enemy_count(1, variance);
            let health_mult = config.calculate_health_multiplier(1);
            let damage_mult = config.calculate_damage_multiplier(1);
            wave_state.prepare_next_wave(enemy_count, health_mult, damage_mult);

            info!(
                "ggrs{{f={} wave_system phase=GracePeriod wave={} enemies={}}}",
                current_frame, wave_state.current_wave, enemy_count
            );
        }

        WavePhase::GracePeriod => {
            let elapsed = current_frame.saturating_sub(wave_state.phase_start_frame);
            if elapsed >= config.grace_period_frames {
                // Grace period over, start spawning
                wave_state.phase = WavePhase::Spawning;
                wave_state.phase_start_frame = current_frame;
                wave_state.wave_start_frame = current_frame;
                wave_state.last_spawn_frame = 0; // Allow immediate first spawn

                info!(
                    "ggrs{{f={} wave_system phase=Spawning wave={}}}",
                    current_frame, wave_state.current_wave
                );
            }
        }

        WavePhase::Spawning => {
            // Check if all enemies have been spawned
            if wave_state.enemies_to_spawn == 0 {
                wave_state.phase = WavePhase::InProgress;
                wave_state.phase_start_frame = current_frame;

                info!(
                    "ggrs{{f={} wave_system phase=InProgress wave={} spawned={}}}",
                    current_frame, wave_state.current_wave, wave_state.enemies_spawned_this_wave
                );
            }
        }

        WavePhase::InProgress => {
            // Check if all wave enemies are dead
            if alive_wave_enemies == 0 && wave_state.enemies_spawned_this_wave > 0 {
                wave_state.phase = WavePhase::WaveComplete;
                wave_state.phase_start_frame = current_frame;

                // If we haven't recorded any kills frame, use current
                if wave_state.last_enemy_killed_frame == 0 {
                    wave_state.last_enemy_killed_frame = current_frame;
                }

                info!(
                    "ggrs{{f={} wave_system phase=WaveComplete wave={} killed={}}}",
                    current_frame, wave_state.current_wave, wave_state.wave_enemies_killed
                );
            }
        }

        WavePhase::WaveComplete => {
            let elapsed = current_frame.saturating_sub(wave_state.last_enemy_killed_frame);
            if elapsed >= config.min_wave_delay_frames {
                // Advance to next wave
                wave_state.current_wave += 1;
                wave_state.phase = WavePhase::GracePeriod;
                wave_state.phase_start_frame = current_frame;

                // Calculate next wave enemies
                let variance = if config.max_random_variance > 0 {
                    rng.next_u32_range(0, config.max_random_variance + 1)
                } else {
                    0
                };
                let enemy_count = config.calculate_enemy_count(wave_state.current_wave, variance);
                let health_mult = config.calculate_health_multiplier(wave_state.current_wave);
                let damage_mult = config.calculate_damage_multiplier(wave_state.current_wave);
                wave_state.prepare_next_wave(enemy_count, health_mult, damage_mult);

                info!(
                    "ggrs{{f={} wave_system phase=GracePeriod wave={} enemies={}}}",
                    current_frame, wave_state.current_wave, enemy_count
                );
            }
        }
    }
}

/// System that spawns enemies during the Spawning phase.
///
/// Uses existing LDTK spawner positions for spawn locations.
pub fn wave_spawning_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut wave_state: ResMut<WaveState>,
    mut rng: ResMut<RollbackRng>,
    wave_config_assets: Res<Assets<WaveConfig>>,
    global_assets: Res<GlobalAsset>,

    // Spawner query (from LDTK map)
    // GGRS CRITICAL: GgrsNetId must be first for deterministic sorting
    spawner_query: Query<(&GgrsNetId, Entity, &EnemySpawnerComponent, &fixed_math::FixedTransform3D)>,
    // Player positions for spawner selection
    player_query: Query<&fixed_math::FixedTransform3D, With<Player>>,
    // Current enemy count
    enemy_query: Query<&fixed_math::FixedTransform3D, With<Enemy>>,

    // Bundled asset dependencies for spawn_enemy
    mut spawn_assets: SpawnAssets,
    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    // Only spawn during Spawning phase
    if wave_state.phase != WavePhase::Spawning {
        return;
    }

    // Get config
    let Some(config) = global_assets
        .wave_config
        .as_ref()
        .and_then(|h| wave_config_assets.get(h))
    else {
        return;
    };

    let current_frame = frame.frame;

    // Check spawn interval
    if wave_state.last_spawn_frame > 0
        && current_frame.saturating_sub(wave_state.last_spawn_frame) < config.spawn_interval_frames
    {
        return;
    }

    // Check concurrent enemy limit
    let current_enemies = enemy_query.iter().count() as u32;
    if current_enemies >= config.max_concurrent_enemies {
        return;
    }

    // No enemies left to spawn
    if wave_state.enemies_to_spawn == 0 {
        return;
    }

    // Get player positions
    let player_positions: Vec<fixed_math::FixedVec2> = player_query
        .iter()
        .map(|t| t.translation.truncate())
        .collect();

    if player_positions.is_empty() {
        return;
    }

    // Select valid spawners based on distance
    let valid_spawners = select_valid_spawners(&spawner_query, &player_positions, config);

    if valid_spawners.is_empty() {
        // No valid spawners - try again next frame
        return;
    }

    // Calculate batch size
    let available_slots = config.max_concurrent_enemies.saturating_sub(current_enemies);
    let batch_size = wave_state
        .enemies_to_spawn
        .min(config.spawn_batch_size)
        .min(available_slots);

    for _ in 0..batch_size {
        if wave_state.enemies_to_spawn == 0 {
            break;
        }

        // Select spawner (random from valid spawners)
        let spawner_idx = if valid_spawners.len() == 1 {
            0
        } else {
            rng.next_u32_range(0, valid_spawners.len() as u32) as usize
        };
        let (_, _, spawner_config, spawner_transform) = &valid_spawners[spawner_idx];

        // Calculate spawn position with offset
        let spawn_pos = calculate_spawn_position(
            spawner_transform.translation,
            spawner_config.spawn_radius,
            &mut rng,
        );

        // Select enemy type based on current wave tier
        let enemy_type = select_enemy_type(&wave_state, config, &mut rng);

        // Spawn the enemy and get the entity
        let enemy_entity = spawn_enemy(
            enemy_type.clone(),
            spawn_pos,
            &mut commands,
            &spawn_assets.weapons_asset,
            &spawn_assets.melee_weapons_asset,
            &spawn_assets.characters_asset,
            &spawn_assets.asset_server,
            &mut spawn_assets.texture_atlas_layouts,
            &spawn_assets.spritesheet_assets,
            &global_assets,
            &spawn_assets.collision_settings,
            &mut id_factory,
        );

        // Add WaveEnemy component to track this enemy for wave completion
        commands.entity(enemy_entity).insert(WaveEnemy {
            spawned_wave: wave_state.current_wave,
        });

        wave_state.enemies_to_spawn -= 1;
        wave_state.enemies_spawned_this_wave += 1;
    }

    wave_state.last_spawn_frame = current_frame;

    trace!(
        "ggrs{{f={} wave_spawning spawned={} remaining={}}}",
        current_frame,
        batch_size,
        wave_state.enemies_to_spawn
    );
}

/// Select spawners that are within valid distance range from players.
fn select_valid_spawners<'a>(
    spawner_query: &'a Query<(&GgrsNetId, Entity, &EnemySpawnerComponent, &fixed_math::FixedTransform3D)>,
    player_positions: &[fixed_math::FixedVec2],
    config: &WaveConfig,
) -> Vec<(&'a GgrsNetId, Entity, &'a EnemySpawnerComponent, &'a fixed_math::FixedTransform3D)> {
    let mut spawners: Vec<_> = spawner_query.iter().collect();
    spawners.sort_unstable_by_key(|(net_id, _, _, _)| net_id.0);

    let mut valid = Vec::new();

    for (net_id, entity, spawner_config, transform) in spawners {
        let spawner_pos = transform.translation.truncate();

        // Find minimum distance to any player
        let min_distance = player_positions
            .iter()
            .map(|p| spawner_pos.distance(p))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(fixed_math::Fixed::MAX);

        // Check distance bounds
        if min_distance >= config.min_player_distance && min_distance <= config.max_player_distance
        {
            valid.push((net_id, entity, spawner_config, transform));
        }
    }

    valid
}

/// Calculate spawn position with random offset from spawner.
fn calculate_spawn_position(
    spawner_pos: fixed_math::FixedVec3,
    spawn_radius: fixed_math::Fixed,
    rng: &mut RollbackRng,
) -> fixed_math::FixedVec3 {
    if spawn_radius <= fixed_math::FIXED_ZERO {
        return spawner_pos;
    }

    let angle = rng.next_fixed() * fixed_math::FIXED_TAU;
    let distance = rng.next_fixed() * spawn_radius;

    let offset = fixed_math::FixedVec2::new(
        fixed_math::cos_fixed(angle),
        fixed_math::sin_fixed(angle),
    ) * distance;

    fixed_math::FixedVec3::new(
        spawner_pos.x.saturating_add(offset.x),
        spawner_pos.y.saturating_add(offset.y),
        spawner_pos.z,
    )
}

/// Select enemy type based on wave tier probabilities.
///
/// GGRS CRITICAL: Sorts probability keys for deterministic weighted selection.
fn select_enemy_type(
    wave_state: &WaveState,
    config: &WaveConfig,
    rng: &mut RollbackRng,
) -> String {
    // Get tier for current wave
    let tier = config.get_tier(wave_state.current_wave);

    let Some(tier) = tier else {
        return "zombie_full".to_string(); // Default fallback
    };

    // Calculate total weight
    let total_weight: u32 = tier.enemy_probabilities.values().sum();
    if total_weight == 0 {
        return "zombie_full".to_string();
    }

    // Random weighted selection
    let roll = rng.next_u32_range(0, total_weight);
    let mut cumulative = 0u32;

    // GGRS CRITICAL: Sort keys for deterministic iteration
    let mut sorted_types: Vec<_> = tier.enemy_probabilities.iter().collect();
    sorted_types.sort_by_key(|(k, _)| *k);

    for (enemy_type, weight) in sorted_types {
        cumulative += weight;
        if roll < cumulative {
            return enemy_type.clone();
        }
    }

    // Fallback (shouldn't reach here)
    tier.enemy_probabilities
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "zombie_full".to_string())
}

/// System to track wave enemy deaths.
///
/// Updates kill counter when wave enemies die.
pub fn wave_enemy_death_tracking_system(
    frame: Res<FrameCount>,
    mut wave_state: ResMut<WaveState>,
    query: Query<(&utils::net_id::GgrsNetId, &WaveEnemy), Added<Death>>,
) {
    for (net_id, wave_enemy) in query.iter() {
        info!(
            "ggrs{{f={} wave_death net_id={} spawned_wave={} current_wave={}}}",
            frame.frame, net_id.0, wave_enemy.spawned_wave, wave_state.current_wave
        );
        // Only count kills from current wave
        if wave_enemy.spawned_wave == wave_state.current_wave {
            wave_state.record_kill(frame.frame);
            info!(
                "ggrs{{f={} wave_kill recorded total={} wave={}}}",
                frame.frame, wave_state.total_enemies_killed, wave_state.wave_enemies_killed
            );
        }
    }
}
