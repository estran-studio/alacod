use bevy::prelude::*;
use bevy_fixed::{fixed_math, rng::RngStreams};
use map::game::entity::map::{
    enemy_spawn::EnemySpawnerComponent, level_id::LevelId, room::RoomBounds,
};
use sim_core::team::Team;

use crate::{
    character::{config::CharacterConfig, player::Player},
    collider::CollisionSettings,
    global_asset::GlobalAsset,
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};

use super::{create::spawn_enemy, Enemy};

/// Maximum distance from player for spawners to activate
/// Should be within flow field range (50 cells * 16 = 800 units)
const MAX_SPAWN_DISTANCE: f32 = 700.0;

#[derive(Component, Debug, Reflect, Clone, Hash)]
#[reflect]
pub struct EnemySpawnerState {
    pub cooldown_remaining: u32,
    pub last_spawn_frame: u32,
    pub active: bool,
}

impl Default for EnemySpawnerState {
    fn default() -> Self {
        Self {
            cooldown_remaining: 0,
            last_spawn_frame: 0,
            active: true,
        }
    }
}

pub fn enemy_spawn_from_spawners_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut rng_streams: ResMut<RngStreams>,
    mut spawner_query: Query<(
        &GgrsNetId,
        Entity,
        &EnemySpawnerComponent,
        &mut EnemySpawnerState,
        &fixed_math::FixedTransform3D,
        Option<&LevelId>,
    )>,
    room_query: Query<(&RoomBounds, &LevelId)>,
    enemy_query: Query<&fixed_math::FixedTransform3D, With<Enemy>>,
    player_query: Query<(&GgrsNetId, &fixed_math::FixedTransform3D), With<Player>>,
    global_assets: Res<GlobalAsset>,
    collision_settings: Res<CollisionSettings>,
    weapons_asset: Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: Res<Assets<MeleeWeaponsConfig>>,
    characters_asset: Res<Assets<CharacterConfig>>,
    balance: Res<crate::balance::ResolvedBalance>,

    mut id_factory: ResMut<GgrsNetIdFactory>,
) {
    // --- Step 0: Decrement cooldowns for all spawners first (deterministic order) ---
    // This ensures cooldowns are always decremented, even if spawner isn't selected
    let mut spawner_data: Vec<_> = spawner_query.iter_mut().collect();
    spawner_data.sort_by_key(|(net_id, ..)| net_id.0);
    for (_, _, _, mut state, _, _) in spawner_data {
        if state.cooldown_remaining > 0 {
            state.cooldown_remaining -= 1;
        }
    }

    // --- Step 1: Build player-to-room mapping ---
    // For each player, find which room (LevelId) they are in
    let mut players_with_rooms: Vec<(usize, fixed_math::FixedVec2, String)> = Vec::new();

    for (net_id, transform) in order_iter!(player_query) {
        let player_pos = transform.translation.truncate();

        // Find which room this player is in
        for (room_bounds, level_id) in room_query.iter() {
            if room_bounds.contains(player_pos) {
                players_with_rooms.push((net_id.0, player_pos, level_id.0.clone()));
                break; // Player can only be in one room
            }
        }
    }

    if players_with_rooms.is_empty() {
        info!(
            "ggrs{{f={} spawner_skip reason=no_players_in_room}}",
            frame.frame
        );
        return;
    }

    if frame.frame % 60 == 0 {
        debug!(
            "Frame {}: Players with rooms: {:?}",
            frame.frame, players_with_rooms
        );
    }

    // --- Step 2: Check enemy count limit ---
    let current_enemies_count = enemy_query.iter().count();
    let global_max_enemies = 10;

    if current_enemies_count >= global_max_enemies {
        info!(
            "ggrs{{f={} spawner_skip reason=max_enemies count={} max={}}}",
            frame.frame, current_enemies_count, global_max_enemies
        );
        return;
    }

    // --- Step 3: Find the best spawner (closest to a player in the SAME room) ---
    let max_spawn_dist = fixed_math::new(MAX_SPAWN_DISTANCE);

    // Candidate: (spawner_net_id, entity, distance_to_player, level_id)
    let mut best_spawner: Option<(usize, Entity, fixed_math::Fixed, String)> = None;

    // Collect spawner data in deterministic order
    let spawner_data: Vec<_> = spawner_query.iter().collect();
    let mut sorted_spawners: Vec<_> = spawner_data.into_iter().collect();
    sorted_spawners.sort_by_key(|(net_id, ..)| net_id.0);

    for (net_id, entity, config, state, spawner_transform, level_id) in sorted_spawners {
        // Skip inactive or on cooldown
        if !state.active || state.cooldown_remaining > 0 {
            continue;
        }

        let spawner_pos = spawner_transform.translation.truncate();
        let spawner_level = match level_id {
            Some(id) => id.0.as_str(),
            None => {
                if frame.frame % 60 == 0 {
                    debug!(
                        "Frame {}: Spawner {} has no LevelId, skipped",
                        frame.frame, net_id.0
                    );
                }
                continue;
            }
        };

        // Find the closest player that is in the SAME room as this spawner
        let mut min_distance_to_same_room_player: Option<fixed_math::Fixed> = None;

        for (player_net_id, player_pos, player_room) in &players_with_rooms {
            // Only consider players in the same room
            if player_room != spawner_level {
                continue;
            }

            let distance = spawner_pos.distance(player_pos);

            // Deterministic tie-breaking: prefer lower player net_id
            let should_update = match min_distance_to_same_room_player {
                None => true,
                Some(current_min) => distance < current_min,
            };

            if should_update {
                min_distance_to_same_room_player = Some(distance);
            }
        }

        // Skip if no players in the same room
        let distance_to_player = match min_distance_to_same_room_player {
            Some(d) => d,
            None => {
                if frame.frame % 60 == 0 {
                    debug!(
                        "Frame {}: Spawner {} (Level: {}) has no players in same room, skipped",
                        frame.frame, net_id.0, spawner_level
                    );
                }
                continue;
            }
        };

        // Check min distance (player too close)
        if distance_to_player < config.min_spawn_distance {
            if frame.frame % 60 == 0 {
                debug!(
                    "Frame {}: Spawner {} (Level: {}) skipped: player too close ({:?} < {:?})",
                    frame.frame,
                    net_id.0,
                    spawner_level,
                    distance_to_player,
                    config.min_spawn_distance
                );
            }
            continue;
        }

        // Check max distance (player too far / outside flow field range)
        if distance_to_player > max_spawn_dist {
            if frame.frame % 60 == 0 {
                debug!(
                    "Frame {}: Spawner {} (Level: {}) skipped: player too far ({:?} > {:?})",
                    frame.frame, net_id.0, spawner_level, distance_to_player, max_spawn_dist
                );
            }
            continue;
        }

        if frame.frame % 60 == 0 {
            debug!(
                "Frame {}: Spawner {} (Level: {}) is valid candidate, distance: {:?}",
                frame.frame, net_id.0, spawner_level, distance_to_player
            );
        }

        // Update best spawner if this one is closer (with deterministic tie-breaking by net_id)
        let should_update = match &best_spawner {
            None => true,
            Some((best_net_id, _, best_dist, _)) => {
                distance_to_player < *best_dist
                    || (distance_to_player == *best_dist && net_id.0 < *best_net_id)
            }
        };

        if should_update {
            best_spawner = Some((
                net_id.0,
                entity,
                distance_to_player,
                spawner_level.to_string(),
            ));
        }
    }

    // --- Step 4: Spawn from the best spawner ---
    let Some((best_net_id, best_entity, best_distance, best_level)) = best_spawner else {
        info!(
            "ggrs{{f={} spawner_skip reason=no_valid_spawner}}",
            frame.frame
        );
        return;
    };

    if frame.frame % 60 == 0 {
        debug!(
            "Frame {}: Selected spawner {} (Level: {}) at distance {:?}",
            frame.frame, best_net_id, best_level, best_distance
        );
    }

    // GGRS trace log for spawner selection
    info!(
        "ggrs{{f={} spawner_select room={} spawner={} dist={}}}",
        frame.frame, best_level, best_net_id, best_distance
    );

    // Get mutable access to spawn from the selected spawner
    if let Ok((spawner_net_id, _entity, config, mut state, spawner_transform, _)) =
        spawner_query.get_mut(best_entity)
    {
        let spawner_pos = spawner_transform.translation.truncate();

        let final_spawn_pos = if config.spawn_radius > fixed_math::FIXED_ZERO {
            let angle_rand = rng_streams.get_mut("waves").next_fixed();
            let angle = angle_rand * fixed_math::FIXED_TAU;

            let distance_rand = rng_streams.get_mut("waves").next_fixed();
            let distance = distance_rand * config.spawn_radius;

            let offset = fixed_math::FixedVec2::new(
                fixed_math::cos_fixed(angle),
                fixed_math::sin_fixed(angle),
            ) * distance;

            fixed_math::FixedVec3::new(
                spawner_pos.x.saturating_add(offset.x),
                spawner_pos.y.saturating_add(offset.y),
                fixed_math::FIXED_ZERO,
            )
        } else {
            spawner_transform.translation
        };

        let type_index =
            (rng_streams.get_mut("waves").next_u32() as usize) % config.enemy_types.len();
        let enemy_type_name = config.enemy_types[type_index].clone();

        debug!(
            "Frame {}: Spawning enemy (type: {}) from spawner {} (Level: {}) at {:?}",
            frame.frame, enemy_type_name, spawner_net_id.0, best_level, final_spawn_pos
        );

        // GGRS trace log for diff_log comparison between clients
        info!(
            "ggrs{{f={} enemy_spawn room={} spawner={} dist={} type={} pos=({},{})}}",
            frame.frame,
            best_level,
            spawner_net_id.0,
            best_distance,
            enemy_type_name,
            final_spawn_pos.x,
            final_spawn_pos.y
        );

        // F5 (chantier m0-v11) : santé max résolue au lancement (`crate::balance`).
        let health_max = balance
            .health_max_by_character
            .get(&enemy_type_name)
            .copied()
            .unwrap_or_else(|| {
                panic!("équilibrage F5 : pas de santé résolue pour le personnage « {enemy_type_name} »")
            });

        let _ = spawn_enemy(
            enemy_type_name,
            final_spawn_pos,
            &mut commands,
            &weapons_asset,
            &melee_weapons_asset,
            &characters_asset,
            &global_assets,
            &collision_settings,
            &mut id_factory,
            Team::Enemies,
            health_max,
            // Graine de run (T1.5 : tirage des variantes), lue sans toucher aux flux.
            rng_streams.run_seed,
            None,
        );

        state.cooldown_remaining = config.max_cooldown;
        state.last_spawn_frame = frame.frame;
    }
}
