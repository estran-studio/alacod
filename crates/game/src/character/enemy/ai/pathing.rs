use crate::character::config::{CharacterConfig, CharacterConfigHandles};
use crate::character::enemy::Enemy;
use crate::character::movement::Velocity;
use crate::character::player::input::FIXED_TIMESTEP;
use crate::character::player::Player;
use crate::collider::{is_colliding, Collider, Wall, Window};
use animation::FacingDirection;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use combat::downed::Downed;
use serde::{Deserialize, Serialize};
use sim_core::stats::StatId;
use stats::StatReader;
use std::collections::VecDeque;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use super::obstacle::Obstacle;
use super::state::EnemyAiConfig;

#[derive(Component, Debug, Clone, Hash, Default)]
pub struct EnemyPath {
    // Target to move toward
    pub target_position: fixed_math::FixedVec2,
    // Queue of waypoints (if using pathfinding)
    pub waypoints: VecDeque<fixed_math::FixedVec2>,
    // Path recalculation timer
    pub recalculate_ticks: u32,
    // Path status
    pub path_status: PathStatus,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub enum PathStatus {
    #[default]
    Idle,
    DirectPath,
    CalculatingPath,
    FollowingPath,
    Blocked,
}

/// Tracks consecutive frames of wall-sliding for stuck recovery
#[derive(Component, Clone, Debug, Hash, Default, Serialize, Deserialize)]
pub struct WallSlideTracker {
    /// Number of consecutive frames where movement was partially blocked
    pub consecutive_slide_frames: u8,
}

#[derive(Resource, Clone, Debug, Hash)]
pub struct PathfindingConfig {
    // How often to recalculate paths (in frames)
    pub recalculation_interval: u32,
    // Maximum pathfinding iterations
    pub max_iterations: u32,
    // Maximum path length
    pub max_path_length: usize,
    // Direct path threshold (distance at which to use direct path)
    pub direct_path_threshold: fixed_math::Fixed,
    // Node size for discretization (if using grid-based approach)
    pub node_size: fixed_math::Fixed,
    // Movement speed fallback
    pub movement_speed: fixed_math::Fixed,
    // Waypoint reach distance
    pub waypoint_reach_distance: fixed_math::Fixed,
    // Minimum distance to maintain from player (attack range)
    pub optimal_attack_distance: fixed_math::Fixed,
    // Distance at which to start slowing down
    pub slow_down_distance: fixed_math::Fixed,
    // Separation force between enemies
    pub enemy_separation_force: fixed_math::Fixed,
    // Separation distance between enemies
    pub enemy_separation_distance: fixed_math::Fixed,
}

impl Default for PathfindingConfig {
    fn default() -> Self {
        Self {
            recalculation_interval: 10, // Recalculate every ~166ms (at 60 FPS) for fresher target tracking
            max_iterations: 1000,
            max_path_length: 50,
            direct_path_threshold: fixed_math::new(200.0),
            node_size: fixed_math::new(16.0),
            movement_speed: fixed_math::new(20.0),
            waypoint_reach_distance: fixed_math::new(10.0),
            optimal_attack_distance: fixed_math::new(30.0), // Melee range - get close to players
            slow_down_distance: fixed_math::new(50.0),      // Start slowing down very close
            enemy_separation_force: fixed_math::new(2.0),   // Much stronger separation force
            enemy_separation_distance: fixed_math::new(80.0), // Larger separation distance
        }
    }
}

// System to find closest player and set as target
// Uses EnemyTarget from the new AI state system
pub fn update_enemy_targets(
    player_query: Query<(
        &GgrsNetId,
        &fixed_math::FixedTransform3D,
        &Player,
        Has<Downed>,
    )>,
    mut enemy_query: Query<
        (
            &fixed_math::FixedTransform3D,
            &mut EnemyPath,
            Option<&super::state::EnemyTarget>,
        ),
        With<Enemy>,
    >,
    frame: Res<FrameCount>,
    config: Res<PathfindingConfig>,
) {
    // Get all player positions with their net IDs
    // GGRS CRITICAL: Sort by net_id for deterministic tie-breaking when multiple players at equal distance
    let mut player_positions: Vec<(GgrsNetId, fixed_math::FixedVec2, bool)> = player_query
        .iter()
        .map(|(net_id, fixed_transform, _, downed)| {
            (
                net_id.clone(),
                fixed_transform.translation.truncate(),
                downed,
            )
        })
        .collect();
    // À terre (T1.3, chantier B6) : même règle que `update_flow_field_system` — ignorer les
    // joueurs à terre tant qu'un autre est encore debout (voir la doc de
    // `combat::downed::Downed`).
    let any_standing = player_positions.iter().any(|(_, _, downed)| !downed);
    if any_standing {
        player_positions.retain(|(_, _, downed)| !downed);
    }
    let mut player_positions: Vec<(GgrsNetId, fixed_math::FixedVec2)> = player_positions
        .into_iter()
        .map(|(id, pos, _)| (id, pos))
        .collect();
    player_positions.sort_unstable_by_key(|(net_id, _)| net_id.0);

    // Update each enemy's target
    for (enemy_fixed_transform, mut path, enemy_target_opt) in enemy_query.iter_mut() {
        // Only update periodically to save performance
        if frame.frame % config.recalculation_interval != 0 {
            continue;
        }

        let enemy_pos_v2 = enemy_fixed_transform.translation.truncate();

        // If enemy has a specific target from the AI system, use its last known position
        if let Some(enemy_target) = enemy_target_opt {
            if let Some(target_pos) = enemy_target.last_known_position {
                path.target_position = target_pos;
                path.recalculate_ticks = frame.frame;
                continue;
            }
        }

        // Fallback: find closest player if no target or target position unknown
        if !player_positions.is_empty() {
            // Initialize with the first player
            let mut closest_player_pos_v2 = player_positions[0].1;
            let mut closest_distance_sq = enemy_pos_v2.distance_squared(&closest_player_pos_v2);

            for (_, player_pos_v2) in player_positions.iter().skip(1) {
                let distance_sq = enemy_pos_v2.distance_squared(player_pos_v2);
                if distance_sq < closest_distance_sq {
                    closest_distance_sq = distance_sq;
                    closest_player_pos_v2 = *player_pos_v2;
                }
            }

            path.target_position = closest_player_pos_v2;
            path.recalculate_ticks = frame.frame;
        }
    }
}

pub fn move_enemies(
    frame: Res<FrameCount>,
    wave: Option<Res<crate::waves::WaveState>>,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            Entity,
            &mut fixed_math::FixedTransform3D,
            &mut Velocity,
            &mut EnemyPath,
            &mut FacingDirection,
            &CharacterConfigHandles,
            &Collider,
            &crate::collider::CollisionLayer,
            &mut WallSlideTracker,
            Option<&super::state::EnemyTarget>,
            &EnemyAiConfig,
            Option<&crate::waves::WaveEnemy>,
            // Regroupés en un sous-tuple (limite de 15 éléments d'un tuple de requête Bevy) :
            // T1.2 : un ennemi qui tire (émetteur posé) ne bouge pas ; T1.4 : behaviors
            // nouveaux (déplacement imposé, voir `rules::behavior_motion`).
            (
                Has<combat::emitter::Emitter>,
                Option<&super::state::BehaviorRuntime>,
                Option<&super::state::EnemyBehaviors>,
                // T1.3 : `Stun`/`Freeze` (§19) : immobile.
                Option<&combat::status::Statuses>,
            ),
        ),
        With<Enemy>,
    >,
    player_query: Query<&fixed_math::FixedTransform3D, (With<Player>, Without<Enemy>)>,
    character_configs: Res<Assets<CharacterConfig>>,
    config: Res<PathfindingConfig>,
    collision_settings: Res<crate::collider::CollisionSettings>,
    wall_collider_query: Query<
        (
            &fixed_math::FixedTransform3D,
            &Collider,
            &crate::collider::CollisionLayer,
        ),
        (With<Wall>, Without<Enemy>, Without<Player>),
    >,
    // Windows block enemies while intact (see process_obstacle_damage)
    window_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &Obstacle,
            &Collider,
        ),
        (
            With<Window>,
            With<Rollback>,
            Without<Enemy>,
            Without<Player>,
        ),
    >,
    flow_field_cache: Res<super::navigation::FlowFieldCache>,
    stats: StatReader,
    grids: Res<crate::collision_grid::CollisionGrids>,
) {
    // --- Spatial Grid for Separation (T2.1, chantier B4b) ---
    // Remplace la table de hachage locale (collection non déterministe interdite par
    // `check-forbidden.sh`) par `combat::grid::SpatialGrid` (T1.8). Entrées ponctuelles (AABB
    // dégénérée min==max==position). `enemy_positions` reste un instantané figé au début de
    // l'appel :
    // les mouvements appliqués plus bas, un ennemi à la fois, ne doivent pas se voir entre eux
    // (comportement inchangé, GGRS CRITICAL : `order_iter!` pour un ordre déterministe).
    let enemy_positions: Vec<(GgrsNetId, Entity, fixed_math::FixedVec2)> = order_iter!(enemy_query)
        .iter()
        .map(|(net_id, entity, transform, ..)| {
            ((*net_id).clone(), *entity, transform.translation.truncate())
        })
        .collect();

    // Granularité du bucketing spatial seulement (optimisation de recherche de voisins) :
    // reste la constante partagée `PathfindingConfig`, pas la stat `SeparationDistance` de
    // chaque ennemi (T1.2) — aujourd'hui identiques pour tous les ennemis (aucun `stats:` de
    // RON ne la surcharge encore), donc sans effet sur le résultat ; un futur type d'ennemi
    // avec une `SeparationDistance` très différente resterait correct (le rayon réel de
    // répulsion, lu plus bas, est bien celui de l'ennemi), seulement avec une grille moins
    // optimale pour ce cas précis (limite déjà présente avant T2.1, inchangée).
    let cell_size = config.enemy_separation_distance;
    let mut separation_grid = combat::grid::SpatialGrid::new(cell_size);
    for (net_id, entity, pos) in &enemy_positions {
        separation_grid.insert(
            net_id.clone(),
            *entity,
            combat::grid::Aabb {
                min: *pos,
                max: *pos,
            },
        );
    }

    // Obstacle avoidance constants
    let lookahead_distance = fixed_math::new(30.0); // How far ahead to check for obstacles
    let avoidance_strength = fixed_math::new(0.7); // How strongly to steer away from obstacles

    // Second pass - calculate and apply movement in deterministic order
    for (
        net_id,
        entity,
        mut fixed_transform,
        mut velocity_component,
        mut path,
        mut facing_direction,
        config_handles,
        enemy_collider,
        enemy_collision_layer,
        mut wall_slide_tracker,
        enemy_target_opt,
        ai_config,
        wave_enemy,
        (emitting, behavior_runtime, enemy_behaviors, statuses),
    ) in order_mut_iter!(enemy_query)
    {
        // T2.9 (testbed) : un ennemi stationnaire (`dummy`/`target`/`ally`/`civilian`)
        // ignore la flow field et ne bouge jamais, quelle que soit sa cible. `false` par
        // défaut : aucun ennemi zombie existant n'est concerné. T1.2 : idem pendant un tir
        // à distance (télégraphe compris) ; aucun ennemi existant ne tire.
        if ai_config.stationary || emitting || combat::status::incapacitated(statuses) {
            velocity_component.main = fixed_math::FixedVec2::ZERO;
            continue;
        }

        let enemy_pos_v2 = fixed_transform.translation.truncate();

        // Get character movement config
        let base_movement_speed =
            if let Some(char_config) = character_configs.get(&config_handles.config) {
                char_config.movement.max_speed // This should be fixed_math::Fixed
            } else {
                config.movement_speed // Fallback is also Fixed
            };
        // Stats branchées (T1.2, chantier B2) : les constantes de `PathfindingConfig` sont
        // maintenant des stats par ennemi (`character::create::create_character` les pose
        // depuis ce même `PathfindingConfig::default()` à la création, voir
        // `enemy::create::enemy_stat_defaults`), surchargeables par `CharacterConfig::stats`
        // dans le RON de chaque type d'ennemi. Sans surcharge, valeur identique à avant.
        let movement_speed = stats.get(entity, &StatId::EnemyMoveSpeed, base_movement_speed);
        let separation_distance = stats.get(
            entity,
            &StatId::SeparationDistance,
            config.enemy_separation_distance,
        );
        let separation_force = stats.get(
            entity,
            &StatId::SeparationForce,
            config.enemy_separation_force,
        );
        let optimal_attack_distance = stats.get(
            entity,
            &StatId::OptimalAttackDistance,
            config.optimal_attack_distance,
        );
        let slow_down_distance =
            stats.get(entity, &StatId::SlowDownDistance, config.slow_down_distance);

        // Get the player position from flow field cache
        let player_pos = flow_field_cache.target_pos.to_fixed();

        // Get the enemy's actual target position (could be window, player, etc.)
        let actual_target = if let Some(enemy_target) = enemy_target_opt {
            if let Some(target_pos) = enemy_target.last_known_position {
                // Use enemy's specific target (could be a window)
                target_pos
            } else {
                // Fallback to player position
                player_pos
            }
        } else {
            // No target component, use player position
            player_pos
        };

        // Enemies are wider than a flow field cell and their collider is offset toward the
        // feet: steer toward points pushed away from walls accordingly
        let body = super::navigation::AgentBody::from_collider(enemy_collider);
        // D41 + D38 : champ du profil et du gabarit de l'ennemi.
        let nav_key = super::navigation::NavKey::for_agent(ai_config.nav_profile(), &body);

        // A pushed steering point can leave its intended cell or overlap a wall.
        // Only recover wave actors after ten seconds without spawning/killing; ordinary
        // movement and historical fixtures keep their original steering. All inputs are
        // rollback state or geometry-derived caches, with no hidden timer.
        let recovery_direction = wave
            .as_deref()
            .filter(|state| wave_enemy.is_some() && navigation_recovery_due(state, frame.frame))
            .and_then(|_| {
                use super::navigation::GridPos;
                let field = flow_field_cache.get_flow_field(nav_key)?;
                let current = GridPos::from_fixed(enemy_pos_v2);
                let next = field.get_direction(current)?;
                if next == current {
                    return None;
                }
                let preferred = flow_field_cache.steering_point(next, nav_key.profile, &body);
                let blocked = |point: fixed_math::FixedVec2, include_windows: bool| {
                    let pos =
                        fixed_math::FixedVec3::new(point.x, point.y, fixed_transform.translation.z);
                    let aabb = crate::collision_grid::collider_aabb(&pos, enemy_collider);
                    grids.walls.query_aabb(&aabb).into_iter().any(|entry| {
                        wall_collider_query.get(entry.entity).is_ok_and(
                            |(transform, collider, layer)| {
                                collision_settings.layer_matrix[enemy_collision_layer.0][layer.0]
                                    && is_colliding(
                                        &pos,
                                        enemy_collider,
                                        &transform.translation,
                                        collider,
                                    )
                            },
                        )
                    }) || (include_windows
                        && grids.characters.query_aabb(&aabb).into_iter().any(|entry| {
                            window_query.get(entry.entity).is_ok_and(
                                |(_, transform, obstacle, collider)| {
                                    obstacle.blocks_movement
                                        && is_colliding(
                                            &pos,
                                            enemy_collider,
                                            &transform.translation,
                                            collider,
                                        )
                                },
                            )
                        }))
                };
                if GridPos::from_fixed(preferred) == next && !blocked(preferred, false) {
                    return None;
                }
                recovery_point(enemy_pos_v2, next, preferred, |point| !blocked(point, true))
                    .map(|point| (point - enemy_pos_v2).normalize_or_zero())
            });

        // Calculate direction to actual target using flow field
        let direction_to_target_v2 =
            if let Some(flow_field) = flow_field_cache.get_flow_field(nav_key) {
                // Always use flow field for navigation - it handles pathfinding around walls
                match flow_field_cache.flow_direction(nav_key, enemy_pos_v2, &body) {
                    Some(dir) => dir,
                    None => {
                        // Outside flow field coverage - find nearest covered cell
                        // and move toward it instead of directly toward player
                        // (moving directly toward player often pushes into walls)
                        match flow_field.find_nearest_covered_cell(enemy_pos_v2, 10) {
                            Some(dir) => dir,
                            None => {
                                // No flow field nearby at all - try neighbor directions as fallback
                                let neighbors = flow_field.get_neighbor_directions(enemy_pos_v2);
                                neighbors.into_iter().next().unwrap_or_else(|| {
                                    // Last resort: direct movement (but this should rarely happen)
                                    (actual_target - enemy_pos_v2).normalize_or_zero()
                                })
                            }
                        }
                    }
                }
            } else {
                // No flow field yet, move directly toward target
                (actual_target - enemy_pos_v2).normalize_or_zero()
            };

        // --- General Obstacle Avoidance Steering ---
        // Use FlowField's blocked cells for O(1) lookups instead of O(walls) collision checks
        let direction_to_target_v2 = if let Some(direction) = recovery_direction {
            direction
        } else {
            use super::navigation::GridPos;

            // Fast grid-based check using FlowField's precomputed blocked cells (obstacles du
            // profil de l'ennemi, D38)
            let is_cell_blocked = |test_pos: fixed_math::FixedVec2| -> bool {
                let grid_pos = GridPos::from_fixed(test_pos);
                flow_field_cache.is_blocked(&grid_pos, nav_key.profile)
            };

            // Check if moving forward would hit a blocked cell
            let forward_pos = enemy_pos_v2 + direction_to_target_v2 * lookahead_distance;

            if is_cell_blocked(forward_pos) {
                // Forward is blocked - check left and right to find clear path
                let perpendicular =
                    fixed_math::FixedVec2::new(-direction_to_target_v2.y, direction_to_target_v2.x);

                let left_pos = enemy_pos_v2
                    + (direction_to_target_v2 + perpendicular).normalize_or_zero()
                        * lookahead_distance;
                let right_pos = enemy_pos_v2
                    + (direction_to_target_v2 - perpendicular).normalize_or_zero()
                        * lookahead_distance;

                let left_clear = !is_cell_blocked(left_pos);
                let right_clear = !is_cell_blocked(right_pos);

                if left_clear && !right_clear {
                    // Steer left
                    let blended = direction_to_target_v2
                        * (fixed_math::FIXED_ONE - avoidance_strength)
                        + perpendicular * avoidance_strength;
                    blended.normalize_or_zero()
                } else if right_clear && !left_clear {
                    // Steer right
                    let blended = direction_to_target_v2
                        * (fixed_math::FIXED_ONE - avoidance_strength)
                        - perpendicular * avoidance_strength;
                    blended.normalize_or_zero()
                } else if left_clear && right_clear {
                    // Both clear - pick left (deterministic for GGRS)
                    let blended = direction_to_target_v2
                        * (fixed_math::FIXED_ONE - avoidance_strength)
                        + perpendicular * avoidance_strength;
                    blended.normalize_or_zero()
                } else {
                    // Both blocked - keep original direction, wall sliding will handle it
                    direction_to_target_v2
                }
            } else {
                // Forward is clear, no steering needed
                direction_to_target_v2
            }
        };

        // Calculate distance to nearest player (for attack range check)
        // Initialize with FixedWide::MAX because distance_squared now returns FixedWide
        let mut min_dist_sq_to_player_fw = fixed_math::FixedWide::MAX;

        for player_fixed_transform in player_query.iter() {
            let player_pos_v2 = player_fixed_transform.translation.truncate();
            // enemy_pos_v2.distance_squared(&player_pos_v2) returns FixedWide
            let current_dist_sq_fw = enemy_pos_v2.distance_squared(&player_pos_v2);

            // Compare FixedWide with FixedWide
            if current_dist_sq_fw < min_dist_sq_to_player_fw {
                min_dist_sq_to_player_fw = current_dist_sq_fw;
            }
        }

        // Now min_dist_sq_to_player_fw holds the minimum squared distance as FixedWide.
        // Calculate the actual distance (as Fixed) if a player was found.
        let distance_to_nearest_player: fixed_math::Fixed;
        if min_dist_sq_to_player_fw < fixed_math::FixedWide::MAX {
            // Check against FixedWide::MAX
            // .sqrt() on FixedWide returns FixedWide (assuming FixedSqrt is implemented for FixedWide)
            let dist_fw = min_dist_sq_to_player_fw.sqrt();

            // Convert the FixedWide result of sqrt back to Fixed for use in subsequent game logic
            // This relies on your Fixed::from_num and FixedWide::to_num methods.
            distance_to_nearest_player = fixed_math::Fixed::from_num(dist_fw);
        } else {
            // No players found, or distance was effectively infinite.
            // Set to a very large Fixed value that your game logic can handle.
            distance_to_nearest_player = fixed_math::Fixed::MAX;
        }

        // Calculate separation force (avoid other enemies) using the spatial grid (T2.1).
        // Requête AABB de demi-étendue `cell_size` : couvre exactement les 9 cellules
        // `(cellule ± 1, cellule ± 1)` de l'ancien bucketing (identité mathématique
        // floor((p ± cell_size) / cell_size) == floor(p / cell_size) ± 1, voir le rapport de
        // la tâche), donc le même ensemble de candidats avant le filtre
        // `dist_to_other < separation_distance` qui décide réellement qui compte. Résultats
        // triés par `net_id` (la grille les insère dans cet ordre et `query_aabb` les trie) :
        // la somme fixed-point accumule dans le même ordre croissant qu'avant.
        let mut separation_v2 = fixed_math::FixedVec2::ZERO;
        let mut separation_count: u32 = 0; // Use u32 for count

        let neighborhood = combat::grid::Aabb {
            min: fixed_math::FixedVec2::new(enemy_pos_v2.x - cell_size, enemy_pos_v2.y - cell_size),
            max: fixed_math::FixedVec2::new(enemy_pos_v2.x + cell_size, enemy_pos_v2.y + cell_size),
        };

        for other_entry in separation_grid.query_aabb(&neighborhood) {
            if other_entry.entity == entity {
                continue;
            }
            let other_pos_v2 = other_entry.aabb.min;

            let dist_to_other = enemy_pos_v2.distance(&other_pos_v2);
            // Use small epsilon for distance > 0 check
            if dist_to_other < separation_distance && dist_to_other > fixed_math::new(0.1) {
                let repulsion_v2 = (enemy_pos_v2 - other_pos_v2).normalize_or_zero()
                    / dist_to_other.max(fixed_math::FIXED_ONE);
                separation_v2 += repulsion_v2;
                separation_count += 1;
            }
        }

        if separation_count > 0 {
            // Convert count to Fixed for division
            separation_v2 =
                (separation_v2 / fixed_math::Fixed::from_num(separation_count)) * separation_force;
        }

        // Calculate base velocity using flow field direction
        let base_velocity_v2 = direction_to_target_v2 * movement_speed;

        // Slow down when near player (for attack positioning)
        let speed_factor_fixed = if distance_to_nearest_player < optimal_attack_distance {
            fixed_math::FIXED_ZERO // Stop when in melee range
        } else if distance_to_nearest_player < slow_down_distance {
            let range = slow_down_distance - optimal_attack_distance;
            if range > fixed_math::FIXED_ZERO {
                let t = (distance_to_nearest_player - optimal_attack_distance) / range;
                t.clamp(fixed_math::FIXED_ZERO, fixed_math::FIXED_ONE)
            } else {
                fixed_math::FIXED_ONE
            }
        } else {
            fixed_math::FIXED_ONE // Full speed
        };

        // T1.4 : un behavior nouveau retenu (`KeepDistance`, `Flee`, `Strafe`, `Wander`,
        // `Charge`) impose sa direction et sa vitesse, sans ralentissement près du joueur ;
        // sinon (tous les ennemis d'avant T1.4) le calcul d'origine, inchangé.
        let motion = behavior_runtime
            .zip(enemy_behaviors)
            .and_then(|(runtime, rules)| {
                super::rules::behavior_motion(
                    rules,
                    runtime,
                    frame.frame,
                    enemy_pos_v2,
                    enemy_target_opt.and_then(|target| target.last_known_position),
                    &flow_field_cache,
                    nav_key,
                )
            });
        let desired_move_velocity_v2 = match &motion {
            Some(motion) => motion.direction * movement_speed.saturating_mul(motion.speed_mult),
            None => base_velocity_v2 * speed_factor_fixed,
        };

        // Combine movement and separation for AI intent
        let final_movement_v2 = desired_move_velocity_v2 + separation_v2;
        velocity_component.main = final_movement_v2;

        // GGRS trace for diff_log (see CLAUDE.md section 9)
        trace!(
            "ggrs{{f={} ai_move net_id={} pos=({},{}) dir=({},{}) vel=({},{}) sep={}}}",
            frame.frame,
            net_id.0,
            enemy_pos_v2.x.to_num::<i32>(),
            enemy_pos_v2.y.to_num::<i32>(),
            direction_to_target_v2.x.to_num::<i32>(),
            direction_to_target_v2.y.to_num::<i32>(),
            final_movement_v2.x.to_num::<i32>(),
            final_movement_v2.y.to_num::<i32>(),
            separation_count,
        );

        // Apply movement using both main and knockback velocities
        let total_velocity = velocity_component.main + velocity_component.knockback;
        if total_velocity.length_squared() > fixed_math::new(0.01) {
            let delta_x = total_velocity.x * fixed_math::new(FIXED_TIMESTEP);
            let delta_y = total_velocity.y * fixed_math::new(FIXED_TIMESTEP);

            // Helper to check collision at a position. Grille spatiale (T2.1, chantier B4b) :
            // requête sur `CollisionGrids::walls` au lieu de la boucle sur tous les murs —
            // l'ancienne optimisation `max_check_dist` (sauter les murs à plus de 100 unités)
            // est maintenant inutile, la grille ne renvoie déjà que les murs proches de `pos`.
            let check_wall_collision = |pos: &fixed_math::FixedVec3| -> bool {
                let aabb = crate::collision_grid::collider_aabb(pos, enemy_collider);
                for wall_entry in grids.walls.query_aabb(&aabb) {
                    let Ok((wall_transform, wall_collider, wall_layer)) =
                        wall_collider_query.get(wall_entry.entity)
                    else {
                        continue;
                    };
                    if !collision_settings.layer_matrix[enemy_collision_layer.0][wall_layer.0] {
                        continue;
                    }
                    if is_colliding(
                        pos,
                        enemy_collider,
                        &wall_transform.translation,
                        wall_collider,
                    ) {
                        return true;
                    }
                }
                // Fenêtres intactes bloquent les ennemis : grille dynamique partagée
                // (`CollisionGrids::characters`), filtrée par `blocks_movement` comme avant.
                // Même instantané que les balles/la mêlée cette frame (les fenêtres ne
                // bougent pas et leur `Obstacle`/`Collider` ne changent pas entre `Movement`
                // et `EnemyAI`, voir `crate::collision_grid`) : les entrées joueurs/ennemis,
                // périmées à ce point, ne sont jamais lues ici (`window_query` les rejette).
                grids.characters.query_aabb(&aabb).into_iter().any(|entry| {
                    window_query.get(entry.entity).is_ok_and(
                        |(_, window_transform, obstacle, window_collider)| {
                            obstacle.blocks_movement
                                && is_colliding(
                                    pos,
                                    enemy_collider,
                                    &window_transform.translation,
                                    window_collider,
                                )
                        },
                    )
                })
            };

            // Try full movement (X + Y)
            let full_pos = fixed_math::FixedVec3::new(
                fixed_transform.translation.x.saturating_add(delta_x),
                fixed_transform.translation.y.saturating_add(delta_y),
                fixed_transform.translation.z,
            );

            if !check_wall_collision(&full_pos) {
                fixed_transform.translation = full_pos;
            } else {
                // Full movement blocked - try sliding along walls: X, then Y from the X
                // obtained (D39 : Y testé depuis le X de départ combinait deux mouvements
                // libres séparément dans le coin d'un mur ; même règle que les joueurs).
                let start_x = fixed_transform.translation.x;
                let start_y = fixed_transform.translation.y;
                let (slid, moved_x, moved_y) = crate::collider::slide_axes(
                    fixed_transform.translation,
                    delta_x,
                    delta_y,
                    &check_wall_collision,
                );
                fixed_transform.translation = slid;

                // Detect wall-sliding: tried diagonal but only one axis succeeded
                let was_trying_diagonal = delta_x.abs() > fixed_math::FIXED_ZERO
                    && delta_y.abs() > fixed_math::FIXED_ZERO;
                let is_wall_sliding = was_trying_diagonal && (moved_x != moved_y);

                if is_wall_sliding {
                    wall_slide_tracker.consecutive_slide_frames = wall_slide_tracker
                        .consecutive_slide_frames
                        .saturating_add(1);
                } else if moved_x && moved_y {
                    // Successfully moved in both axes - reset tracker
                    wall_slide_tracker.consecutive_slide_frames = 0;
                }

                // If wall-sliding too long, trigger escape logic
                const MAX_SLIDE_FRAMES: u8 = 10; // ~170ms at 60fps
                if wall_slide_tracker.consecutive_slide_frames >= MAX_SLIDE_FRAMES {
                    // Reset tracker
                    wall_slide_tracker.consecutive_slide_frames = 0;

                    // Try alternative directions (same logic as complete stuck)
                    let move_magnitude = (delta_x.abs() + delta_y.abs()).max(fixed_math::new(1.0));
                    let speed = velocity_component.main.length();

                    // Try flow field neighbor directions first
                    if let Some(flow_field) = flow_field_cache.get_flow_field(nav_key) {
                        let neighbor_dirs = flow_field.get_neighbor_directions(enemy_pos_v2);
                        for dir in neighbor_dirs {
                            // Determine slide axis (which axis succeeded)
                            let slide_axis = if moved_x {
                                fixed_math::FixedVec2::new(
                                    fixed_math::new(1.0),
                                    fixed_math::FIXED_ZERO,
                                )
                            } else {
                                fixed_math::FixedVec2::new(
                                    fixed_math::FIXED_ZERO,
                                    fixed_math::new(1.0),
                                )
                            };

                            // Prefer directions perpendicular to slide axis
                            let perpendicular_component =
                                dir.x * slide_axis.y + dir.y * slide_axis.x;
                            if perpendicular_component.abs() > fixed_math::new(0.3) {
                                let dx = dir.x * move_magnitude;
                                let dy = dir.y * move_magnitude;
                                let test_pos = fixed_math::FixedVec3::new(
                                    fixed_transform.translation.x.saturating_add(dx),
                                    fixed_transform.translation.y.saturating_add(dy),
                                    fixed_transform.translation.z,
                                );
                                if !check_wall_collision(&test_pos) {
                                    fixed_transform.translation = test_pos;
                                    velocity_component.main = dir * speed;
                                    break;
                                }
                            }
                        }
                    }
                }

                // If completely stuck, check if we're blocked by a window and attack it
                // This uses the flow field to find the best escape route toward the target
                if !moved_x && !moved_y {
                    // Reset wall slide tracker when completely stuck
                    wall_slide_tracker.consecutive_slide_frames = 0;

                    // A blocking intact window is attacked by enemy_attack_system: the
                    // enemy targets it in enemy_target_selection (with attack cooldown)

                    let move_magnitude = (delta_x.abs() + delta_y.abs()).max(fixed_math::new(1.0));
                    let speed = velocity_component.main.length();

                    let mut escaped = false;

                    // First, try directions from neighboring flow field cells (sorted by cost)
                    if let Some(flow_field) = flow_field_cache.get_flow_field(nav_key) {
                        let neighbor_dirs = flow_field.get_neighbor_directions(enemy_pos_v2);
                        for dir in neighbor_dirs {
                            let dx = dir.x * move_magnitude;
                            let dy = dir.y * move_magnitude;
                            let test_pos = fixed_math::FixedVec3::new(
                                start_x.saturating_add(dx),
                                start_y.saturating_add(dy),
                                fixed_transform.translation.z,
                            );
                            if !check_wall_collision(&test_pos) {
                                fixed_transform.translation = test_pos;
                                velocity_component.main = dir * speed;
                                escaped = true;
                                break;
                            }
                        }
                    }

                    // If flow field neighbors didn't help, try all 8 cardinal directions
                    if !escaped {
                        let directions: [(fixed_math::Fixed, fixed_math::Fixed); 8] = [
                            (move_magnitude, fixed_math::FIXED_ZERO),  // Right
                            (-move_magnitude, fixed_math::FIXED_ZERO), // Left
                            (fixed_math::FIXED_ZERO, move_magnitude),  // Up
                            (fixed_math::FIXED_ZERO, -move_magnitude), // Down
                            (move_magnitude, move_magnitude),          // Up-Right
                            (-move_magnitude, move_magnitude),         // Up-Left
                            (move_magnitude, -move_magnitude),         // Down-Right
                            (-move_magnitude, -move_magnitude),        // Down-Left
                        ];

                        for (dx, dy) in directions {
                            let test_pos = fixed_math::FixedVec3::new(
                                start_x.saturating_add(dx),
                                start_y.saturating_add(dy),
                                fixed_transform.translation.z,
                            );
                            if !check_wall_collision(&test_pos) {
                                fixed_transform.translation = test_pos;
                                velocity_component.main =
                                    fixed_math::FixedVec2::new(dx, dy).normalize_or_zero() * speed;
                                escaped = true;
                                break;
                            }
                        }
                    }

                    if !escaped {
                        // Truly stuck in all directions - zero velocity
                        velocity_component.main = fixed_math::FixedVec2::ZERO;
                        trace!(
                            "ggrs{{f={} ai_stuck net_id={} pos=({},{})}}",
                            frame.frame,
                            net_id.0,
                            fixed_transform.translation.x.to_num::<i32>(),
                            fixed_transform.translation.y.to_num::<i32>(),
                        );
                    }
                }
            }

            // Update facing direction based on main velocity (not knockback)
            if velocity_component.main.length_squared() > fixed_math::new(0.01) {
                *facing_direction = FacingDirection::from_fixed_vector(velocity_component.main);
            }
        }
    }
}

fn navigation_recovery_due(state: &crate::waves::WaveState, frame: u32) -> bool {
    matches!(
        state.phase,
        crate::waves::WavePhase::Spawning | crate::waves::WavePhase::InProgress
    ) && frame.saturating_sub(
        state
            .last_spawn_frame
            .max(state.last_enemy_killed_frame)
            .max(state.phase_start_frame),
    ) >= 600
}

/// Stable integer samples inside the next tile, then the cardinal steps of a diagonal,
/// then the current tile. A wall corner can make every route from the current position
/// cross the wall, even for an axis-aligned cell move: an intermediate point in the
/// current tile moves the offset body off the corner (for example below the wall's
/// bottom edge) before the segment toward the next tile. Validate the whole straight
/// segment at one-unit intervals, so recovering a waypoint cannot cross a wall/door.
fn recovery_point(
    from: fixed_math::FixedVec2,
    next: super::navigation::GridPos,
    preferred: fixed_math::FixedVec2,
    clear: impl Fn(fixed_math::FixedVec2) -> bool,
) -> Option<fixed_math::FixedVec2> {
    use super::navigation::GRID_CELL_SIZE;
    // D53 : `clear` interroge les grilles de collision (requêtes qui allouent) ; un même point
    // revient (candidat, extrémité de segment, destination) : résultat mémorisé pour l'appel.
    // Le résultat de `recovery_point` est inchangé (même ordre, même prédicat).
    let memo = std::cell::RefCell::new(std::collections::BTreeMap::new());
    let clear = |point: fixed_math::FixedVec2| -> bool {
        let key = (point.x.to_bits(), point.y.to_bits());
        if let Some(known) = memo.borrow().get(&key) {
            return *known;
        }
        let value = clear(point);
        memo.borrow_mut().insert(key, value);
        value
    };
    let mut candidates = Vec::new();
    let current = super::navigation::GridPos::from_fixed(from);
    let mut cells = vec![next];
    if current.x != next.x && current.y != next.y {
        cells.extend([
            super::navigation::GridPos::new(current.x, next.y),
            super::navigation::GridPos::new(next.x, current.y),
        ]);
    }
    cells.push(current);
    for (stage, cell) in cells.into_iter().enumerate() {
        for x in 1..GRID_CELL_SIZE {
            for y in 1..GRID_CELL_SIZE {
                let point = fixed_math::FixedVec2::new(
                    fixed_math::Fixed::from_num(cell.x * GRID_CELL_SIZE + x),
                    fixed_math::Fixed::from_num(cell.y * GRID_CELL_SIZE + y),
                );
                candidates.push((stage, point.distance_squared(&preferred), x, y, point));
            }
        }
    }
    candidates.sort_by_key(|(stage, distance, x, y, _)| (*stage, *distance, *x, *y));
    let segment_clear = |start: fixed_math::FixedVec2, end: fixed_math::FixedVec2| {
        let delta = end - start;
        let steps = delta
            .x
            .abs()
            .max(delta.y.abs())
            .ceil()
            .to_num::<i32>()
            .max(1);
        (1..=steps).all(|step| {
            clear(
                start
                    + delta * fixed_math::Fixed::from_num(step)
                        / fixed_math::Fixed::from_num(steps),
            )
        })
    };
    // D53 : les destinations (points libres de la case suivante) ne servent qu'aux étapes
    // suivantes ; calculées au premier besoin (les candidats de l'étape 0 passent d'abord, et
    // le premier qui convient termine la recherche).
    let destinations = std::cell::OnceCell::new();
    let destinations = || {
        destinations.get_or_init(|| {
            candidates
                .iter()
                .filter(|(stage, _, _, _, point)| *stage == 0 && clear(*point))
                .map(|(_, _, _, _, point)| *point)
                .collect::<Vec<_>>()
        })
    };
    candidates.iter().find_map(|&(stage, _, _, _, point)| {
        (clear(point)
            && segment_clear(from, point)
            && (stage == 0 || destinations().iter().any(|end| segment_clear(point, *end))))
        .then_some(point)
    })
}

#[cfg(test)]
mod recovery_tests {
    use super::super::navigation::GridPos;
    use super::*;

    #[test]
    fn recovery_deadline_uses_the_latest_progress_and_only_active_waves() {
        let mut state = crate::waves::WaveState {
            phase: crate::waves::WavePhase::Spawning,
            phase_start_frame: 100,
            last_spawn_frame: 200,
            last_enemy_killed_frame: 300,
            ..default()
        };
        assert!(!navigation_recovery_due(&state, 899));
        assert!(navigation_recovery_due(&state, 900));
        state.last_spawn_frame = 500;
        assert!(!navigation_recovery_due(&state, 900));
        state.phase = crate::waves::WavePhase::GracePeriod;
        assert!(!navigation_recovery_due(&state, 2000));
    }

    #[test]
    fn boundary_waypoint_returns_inside_next_cell_and_makes_progress() {
        let from = fixed_math::FixedVec2::new(fixed_math::new(448.013), fixed_math::new(-208.003));
        let point = recovery_point(
            from,
            GridPos::new(27, -13),
            fixed_math::FixedVec2::new(fixed_math::new(448.0), fixed_math::new(-208.0)),
            |_| true,
        )
        .unwrap();
        assert_eq!(GridPos::from_fixed(point), GridPos::new(27, -13));
        assert!(point.x < from.x - fixed_math::FIXED_ONE);
    }

    #[test]
    fn window_corner_recovery_checks_offset_body_and_entire_segment() {
        let (body, wall, wall_pos) = corner_fixture();
        let clear =
            |point: fixed_math::FixedVec2| !is_colliding(&point.extend(), &body, &wall_pos, &wall);
        let from = fixed_math::FixedVec2::new(fixed_math::new(28.541), fixed_math::new(-128.344));
        let preferred = fixed_math::FixedVec2::new(fixed_math::new(8.0), fixed_math::new(-128.0));
        assert!(!clear(preferred));
        let point = recovery_point(from, GridPos::new(0, -8), preferred, clear).unwrap();
        assert_eq!(GridPos::from_fixed(point), GridPos::new(1, -8));
        assert!(clear(point));
        let second = recovery_point(point, GridPos::new(0, -8), preferred, clear).unwrap();
        assert_eq!(GridPos::from_fixed(second), GridPos::new(0, -8));
        assert!(clear(second));
        assert!(recovery_point(from, GridPos::new(0, -8), preferred, |_| false).is_none());
    }

    #[test]
    fn axis_aligned_move_recovers_via_the_current_tile_below_a_wall_corner() {
        let (body, wall, wall_pos) = corner_fixture();
        let clear =
            |point: fixed_math::FixedVec2| !is_colliding(&point.extend(), &body, &wall_pos, &wall);
        // Seed 12 : le corps est dans la même rangée que la case cible, le
        // déplacement de case est horizontal et toute route directe rase le coin
        // du mur. La case courante contient un point sous l'arête basse du mur.
        let from = fixed_math::FixedVec2::new(fixed_math::new(28.457), fixed_math::new(-127.506));
        let preferred = fixed_math::FixedVec2::new(fixed_math::new(8.0), fixed_math::new(-128.0));
        assert_eq!(GridPos::from_fixed(from), GridPos::new(1, -8));
        let point = recovery_point(from, GridPos::new(0, -8), preferred, clear).unwrap();
        assert_eq!(GridPos::from_fixed(point), GridPos::new(1, -8));
        assert!(clear(point));
        assert!(point.y > from.y);
        let second = recovery_point(point, GridPos::new(0, -8), preferred, clear).unwrap();
        assert_eq!(GridPos::from_fixed(second), GridPos::new(0, -8));
        assert!(clear(second));
    }

    fn corner_fixture() -> (Collider, Collider, fixed_math::FixedVec3) {
        let body = Collider {
            shape: crate::collider::ColliderShape::Rectangle {
                width: fixed_math::new(20.0),
                height: fixed_math::new(20.0),
            },
            offset: fixed_math::FixedVec3::new(
                fixed_math::FIXED_ZERO,
                fixed_math::new(-6.0),
                fixed_math::FIXED_ZERO,
            ),
        };
        let wall = Collider {
            shape: crate::collider::ColliderShape::Rectangle {
                width: fixed_math::new(16.0),
                height: fixed_math::new(32.0),
            },
            offset: fixed_math::FixedVec3::ZERO,
        };
        let wall_pos = fixed_math::FixedVec3::new(
            fixed_math::new(10.0),
            fixed_math::new(-153.0),
            fixed_math::FIXED_ZERO,
        );
        (body, wall, wall_pos)
    }
}
