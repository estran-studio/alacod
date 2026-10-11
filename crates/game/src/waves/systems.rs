//! Wave spawning systems.
//!
//! GGRS CRITICAL: All systems must be deterministic.
//! See CLAUDE.md for GGRS rules.

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_fixed::{
    fixed_math,
    rng::{RngStreams, RollbackRng},
};
use map::game::entity::map::enemy_spawn::EnemySpawnerComponent;
use sim_core::team::Team;
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
};

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
    /// T1.9 : difficulté (santé × difficulté ; 1 sans difficulté activée).
    pub difficulty: crate::clock::DifficultyReader<'w>,
}

use super::{
    state::{WavePhase, WaveState},
    tracking::WaveEnemy,
};

/// Ten seconds without a spawn: distance bounds must not freeze a wave forever.
/// The deadline uses existing rollback state, including the start of each new wave.
const SPAWN_STALL_FRAMES: u32 = 600;

fn spawn_stalled(state: &WaveState, frame: u32) -> bool {
    frame.saturating_sub(state.last_spawn_frame.max(state.phase_start_frame)) >= SPAWN_STALL_FRAMES
}

/// System that manages wave state transitions.
///
/// Runs every frame to check conditions and advance the state machine.
pub fn wave_state_machine_system(
    frame: Res<FrameCount>,
    mut wave_state: ResMut<WaveState>,
    mut rng_streams: ResMut<RngStreams>,
    balance: Res<crate::balance::ResolvedBalance>,
    global_assets: Res<GlobalAsset>,
    wave_enemy_query: Query<Entity, (With<Enemy>, With<WaveEnemy>)>,
) {
    // F5 (chantier m0-v11) : config résolue une fois au lancement (voir `crate::balance`),
    // plus jamais lue depuis l'asset — valeurs identiques pour un contenu littéral.
    // Un jeu sans dossier `Wave` (ex. `testbed`) n'a pas de `wave_config` : la machine
    // reste inerte, comme avant F5 (la résolution ne doit pas réveiller les vagues
    // d'un jeu qui n'en déclare pas).
    if global_assets.wave_config.is_none() {
        return;
    }
    let config = &balance.waves;

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
                rng_streams
                    .get_mut("waves")
                    .next_u32_range(0, config.max_random_variance + 1)
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
                    rng_streams
                        .get_mut("waves")
                        .next_u32_range(0, config.max_random_variance + 1)
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
    mut rng_streams: ResMut<RngStreams>,
    balance: Res<crate::balance::ResolvedBalance>,
    global_assets: Res<GlobalAsset>,

    // Spawner query (from LDTK map)
    // GGRS CRITICAL: GgrsNetId must be first for deterministic sorting
    spawner_query: Query<(
        &GgrsNetId,
        Entity,
        &EnemySpawnerComponent,
        &fixed_math::FixedTransform3D,
    )>,
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

    // Get config (resolved once at run start, F5 — see `crate::balance`)
    // Même garde que la machine : sans dossier `Wave` déclaré, rien à spawner.
    if global_assets.wave_config.is_none() {
        return;
    }
    let config = &balance.waves;

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
    let valid_spawners = select_valid_spawners(
        &spawner_query,
        &player_positions,
        config,
        spawn_stalled(&wave_state, current_frame) || wave_state.spawn_fallback,
    );

    if valid_spawners.is_empty() {
        // No valid spawners - try again next frame
        return;
    }

    let using_fallback = valid_spawners.iter().all(|(_, _, _, transform)| {
        let distance = player_positions
            .iter()
            .map(|p| transform.translation.truncate().distance(p))
            .min()
            .unwrap_or(fixed_math::Fixed::MAX);
        distance < config.min_player_distance || distance > config.max_player_distance
    });
    if using_fallback {
        let (id, _, _, _) = valid_spawners[0];
        wave_state.spawn_fallback = true;
        info!(
            "ggrs{{f={} wave_spawn_fallback wave={} spawner={}}}",
            current_frame, wave_state.current_wave, id.0
        );
    }

    // Calculate batch size
    let available_slots = config
        .max_concurrent_enemies
        .saturating_sub(current_enemies);
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
            rng_streams
                .get_mut("waves")
                .next_u32_range(0, valid_spawners.len() as u32) as usize
        };
        let (_, _, spawner_config, spawner_transform) = &valid_spawners[spawner_idx];

        // Calculate spawn position with offset
        let spawn_pos = calculate_spawn_position(
            spawner_transform.translation,
            spawner_config.spawn_radius,
            rng_streams.get_mut("waves"),
        );

        // Select enemy type based on current wave tier
        let enemy_type = select_enemy_type(&wave_state, config, rng_streams.get_mut("waves"));

        // F5 (chantier m0-v11) : santé max résolue au lancement (`crate::balance`).
        let health_max = balance
            .health_max_by_character
            .get(&enemy_type)
            .copied()
            .unwrap_or_else(|| {
                panic!("équilibrage F5 : pas de santé résolue pour le personnage « {enemy_type} »")
            });
        let health_max = crate::clock::scale(health_max, spawn_assets.difficulty.current());

        // Spawn the enemy and get the entity
        let enemy_entity = spawn_enemy(
            enemy_type.clone(),
            spawn_pos,
            &mut commands,
            &spawn_assets.weapons_asset,
            &spawn_assets.melee_weapons_asset,
            &spawn_assets.characters_asset,
            &global_assets,
            &spawn_assets.collision_settings,
            &mut id_factory,
            Team::Enemies,
            health_max,
            // Graine de run (T1.5 : tirage des variantes), lue sans toucher aux flux.
            rng_streams.run_seed,
            None,
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
    spawner_query: &'a Query<(
        &GgrsNetId,
        Entity,
        &EnemySpawnerComponent,
        &fixed_math::FixedTransform3D,
    )>,
    player_positions: &[fixed_math::FixedVec2],
    config: &crate::balance::ResolvedWaveConfig,
    allow_nearest: bool,
) -> Vec<(
    &'a GgrsNetId,
    Entity,
    &'a EnemySpawnerComponent,
    &'a fixed_math::FixedTransform3D,
)> {
    let mut spawners: Vec<_> = spawner_query.iter().collect();
    spawners.sort_unstable_by_key(|(net_id, _, _, _)| net_id.0);

    let mut valid = Vec::new();
    let mut nearest = None;

    let occupied_defence_room = spawners.iter().any(|(_, _, config, _)| {
        config.activation_area.is_some() && player_positions.iter().any(|&p| config.defends(p))
    });
    for (net_id, entity, spawner_config, transform) in spawners {
        if spawner_config.activation_area.is_some()
            && !player_positions.iter().any(|&p| spawner_config.defends(p))
            && (occupied_defence_room || !allow_nearest)
        {
            continue;
        }
        let spawner_pos = transform.translation.truncate();

        // Find minimum distance to any player
        let min_distance = player_positions
            .iter()
            .map(|p| spawner_pos.distance(p))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(fixed_math::Fixed::MAX);

        // Sorting by net_id above also breaks equal-distance ties deterministically.
        if nearest
            .as_ref()
            .is_none_or(|(distance, _)| min_distance < *distance)
        {
            nearest = Some((min_distance, (net_id, entity, spawner_config, transform)));
        }

        // Check distance bounds
        if (spawner_config.activation_area.is_some() && occupied_defence_room)
            || (min_distance >= config.min_player_distance
                && min_distance <= config.max_player_distance)
        {
            valid.push((net_id, entity, spawner_config, transform));
        }
    }

    if valid.is_empty() && allow_nearest && !player_positions.is_empty() {
        if let Some((_, spawner)) = nearest {
            valid.push(spawner);
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

    let offset =
        fixed_math::FixedVec2::new(fixed_math::cos_fixed(angle), fixed_math::sin_fixed(angle))
            * distance;

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
    config: &crate::balance::ResolvedWaveConfig,
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use fixed_math::{Fixed, FixedTransform3D, FixedVec2};

    fn select(
        In((positions, fallback)): In<(Vec<FixedVec2>, bool)>,
        query: Query<(
            &GgrsNetId,
            Entity,
            &EnemySpawnerComponent,
            &FixedTransform3D,
        )>,
    ) -> Vec<usize> {
        select_valid_spawners(
            &query,
            &positions,
            &crate::balance::resolve_waves(&crate::waves::WaveConfig::default(), 1),
            fallback,
        )
        .iter()
        .map(|s| s.0 .0)
        .collect()
    }

    fn world(spawners: &[(usize, i32)]) -> World {
        let mut world = World::new();
        for &(id, x) in spawners {
            let mut transform = FixedTransform3D::IDENTITY;
            transform.translation.x = Fixed::from_num(x);
            world.spawn((
                GgrsNetId(id, "spawner".into()),
                EnemySpawnerComponent::default(),
                transform,
            ));
        }
        world
    }

    #[test]
    fn distant_or_too_close_spawners_recover_with_net_id_tie_break() {
        for positions in [
            vec![(9, 1000), (3, -1000), (1, 1400)],
            vec![(9, 20), (3, -20)],
        ] {
            let mut w = world(&positions);
            assert!(w
                .run_system_once_with(select, (vec![FixedVec2::ZERO], false))
                .unwrap()
                .is_empty());
            assert_eq!(
                w.run_system_once_with(select, (vec![FixedVec2::ZERO], true))
                    .unwrap(),
                vec![3]
            );
            let mut reversed = positions;
            reversed.reverse();
            assert_eq!(
                world(&reversed)
                    .run_system_once_with(select, (vec![FixedVec2::ZERO], true))
                    .unwrap(),
                vec![3]
            );
        }
    }

    #[test]
    fn exterior_sources_follow_occupied_rooms_even_during_fallback() {
        let mut w = world(&[(1, 80), (2, 300)]);
        for (id, mut config) in w
            .query::<(&GgrsNetId, &mut EnemySpawnerComponent)>()
            .iter_mut(&mut w)
        {
            config.activation_area = Some((
                FixedVec2::new(
                    Fixed::from_num(if id.0 == 1 { -50 } else { 500 }),
                    Fixed::from_num(-50),
                ),
                FixedVec2::new(Fixed::from_num(100), Fixed::from_num(100)),
            ));
        }
        for fallback in [false, true] {
            assert_eq!(
                w.run_system_once_with(select, (vec![FixedVec2::ZERO], fallback))
                    .unwrap(),
                vec![1]
            );
            assert_eq!(
                w.run_system_once_with(
                    select,
                    (
                        vec![FixedVec2::new(Fixed::from_num(550), Fixed::ZERO)],
                        fallback
                    )
                )
                .unwrap(),
                vec![2]
            );
            assert_eq!(
                w.run_system_once_with(
                    select,
                    (
                        vec![
                            FixedVec2::ZERO,
                            FixedVec2::new(Fixed::from_num(550), Fixed::ZERO)
                        ],
                        fallback
                    )
                )
                .unwrap(),
                vec![1, 2]
            );
        }
    }

    #[test]
    fn normal_range_remains_preferred_after_deadline() {
        let mut w = world(&[(1, 20), (9, 300), (3, 400), (4, 1000)]);
        for fallback in [false, true] {
            assert_eq!(
                w.run_system_once_with(select, (vec![FixedVec2::ZERO], fallback))
                    .unwrap(),
                vec![3, 9]
            );
        }
        assert!(w
            .run_system_once_with(select, (vec![], true))
            .unwrap()
            .is_empty());
        assert!(world(&[])
            .run_system_once_with(select, (vec![FixedVec2::ZERO], true))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn deadline_resets_on_spawn_and_new_wave_and_survives_rollback() {
        let state = WaveState {
            phase: WavePhase::Spawning,
            phase_start_frame: 180,
            ..Default::default()
        };
        assert!(!spawn_stalled(&state, 779));
        assert!(spawn_stalled(&state, 780));
        let spawned = WaveState {
            last_spawn_frame: 780,
            ..state.clone()
        };
        assert!(!spawn_stalled(&spawned, 1379));
        assert!(spawn_stalled(&spawned, 1380));
        assert!(!spawn_stalled(&spawned, 779));
        let next = WaveState {
            phase_start_frame: 2000,
            last_spawn_frame: 0,
            ..spawned
        };
        assert!(!spawn_stalled(&next, 2599));
        assert!(spawn_stalled(&next, 2600));
    }
}
