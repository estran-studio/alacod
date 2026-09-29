//! Enemy Behavior System
//!
//! Implements the AI behavior loop using the flow field navigation
//! and generic state machine.
//!
//! Enemies chase the closest player along the flow field. An intact window that blocks
//! their path (they collide with it in `move_enemies`) becomes their target: they break
//! it first, then resume the chase.

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use sim_core::damage::{DamageEvent, DamageKind};
use sim_core::tag::{Tag, Tags};
use sim_core::team::Team;
use std::collections::BTreeMap;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use crate::character::enemy::Enemy;
use crate::character::health::DamageAccumulator;
use crate::character::movement::Velocity;
use crate::character::player::Player;
use crate::frame_events::FrameEvents;

use super::navigation::FlowFieldCache;
use super::obstacle::{Obstacle, ObstacleAttackEvent};
use super::state::{AttackTarget, EnemyAiConfig, EnemyTarget, MonsterState, TargetType};

/// System to select targets for enemies based on proximity
///
/// Target: the closest player in aggro range, unless an intact breakable obstacle (window)
/// is on the enemy's flow field path, within attack range: the enemy must break it first.
pub fn enemy_target_selection(
    frame: Res<FrameCount>,
    flow_field_cache: Res<FlowFieldCache>,
    obstacle_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &Obstacle,
            &crate::collider::Collider,
        ),
        (With<Rollback>, Without<Enemy>, Without<Player>),
    >,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &EnemyAiConfig,
            &mut EnemyTarget,
            &mut MonsterState,
        ),
        With<Enemy>,
    >,
    player_query: Query<
        (&GgrsNetId, &fixed_math::FixedTransform3D),
        (With<Player>, Without<Enemy>),
    >,
) {
    // Collect and sort players for deterministic iteration
    let mut players: Vec<_> = player_query.iter().collect();
    players.sort_by_key(|(net_id, _)| net_id.0);

    // Intact breakable obstacles and the cells they cover, by net_id
    let breakables: Vec<_> = utils::order_iter!(obstacle_query)
        .into_iter()
        .filter(|(_, _, obstacle, _)| obstacle.blocks_movement && obstacle.breakable)
        .map(|(net_id, transform, _, collider)| {
            let pos = transform.translation.truncate();
            (
                net_id.clone(),
                pos,
                super::navigation::get_collider_cells(pos, collider),
            )
        })
        .collect();

    for (_enemy_net_id, enemy_transform, ai_config, mut target, mut state) in
        order_mut_iter!(enemy_query)
    {
        let enemy_pos = enemy_transform.translation.truncate();

        // Don't retarget if attacking, stunned, or dead
        match *state {
            MonsterState::Attacking { .. }
            | MonsterState::Stunned { .. }
            | MonsterState::Breaching { .. }
            | MonsterState::Dead => continue,
            _ => {}
        }

        // Target the player the flow field leads to (the closest one along the path), so the
        // enemy chases and attacks the same player; fall back to the closest in straight line
        let path_player = flow_field_cache
            .nearest_target(super::navigation::NavProfile::GroundBreaker, enemy_pos)
            .and_then(|net_id| players.iter().find(|(id, _)| id.0 == net_id))
            .map(|(id, transform)| {
                let pos = transform.translation.truncate();
                ((*id).clone(), enemy_pos.distance(&pos), pos)
            })
            .filter(|(_, distance, _)| *distance < ai_config.aggro_range);

        // Find closest player deterministically
        let mut closest_player: Option<(GgrsNetId, fixed_math::Fixed, fixed_math::FixedVec2)> =
            path_player;

        let has_path_player = closest_player.is_some();
        for (player_net_id, player_transform) in players.iter().filter(|_| !has_path_player) {
            let player_pos = player_transform.translation.truncate();
            let distance = enemy_pos.distance(&player_pos);

            if distance < ai_config.aggro_range {
                match closest_player {
                    None => {
                        closest_player = Some(((*player_net_id).clone(), distance, player_pos));
                    }
                    Some((_, closest_dist, _)) => {
                        // Strict inequality ensures first one (lowest NetId due to sort) is kept on ties
                        if distance < closest_dist {
                            closest_player = Some(((*player_net_id).clone(), distance, player_pos));
                        }
                    }
                }
            }
        }

        // An intact window on the way (current cell or next cells), within attack range:
        // break it before going through
        if closest_player.is_some() {
            let here = super::navigation::GridPos::from_fixed(enemy_pos);
            let mut ahead = vec![here];
            ahead.extend(flow_field_cache.path_ahead(
                super::navigation::NavProfile::GroundBreaker,
                here,
                3,
            ));
            let blocking = breakables.iter().find(|(_, pos, cells)| {
                enemy_pos.distance(pos) < ai_config.attack_range
                    && ahead.iter().any(|cell| cells.contains(cell))
            });
            if let Some((obstacle_id, obstacle_pos, _)) = blocking {
                target.target = Some(obstacle_id.clone());
                target.target_type = TargetType::Obstacle;
                target.last_known_position = Some(*obstacle_pos);
                *state = MonsterState::Chasing;
                continue;
            }
        }

        if let Some((player_id, _, player_pos)) = closest_player {
            target.target = Some(player_id);
            target.target_type = TargetType::Player;
            target.last_known_position = Some(player_pos);
            *state = MonsterState::Chasing;
        } else {
            target.target = None;
            target.target_type = TargetType::None;
            target.last_known_position = None;
            *state = MonsterState::Idle;
        }
    }
}

/// System to move enemies using the flow field
pub fn enemy_movement_system(
    frame: Res<FrameCount>,
    flow_field_cache: Res<FlowFieldCache>,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            Entity,
            &mut fixed_math::FixedTransform3D,
            &mut Velocity,
            &EnemyAiConfig,
            &EnemyTarget,
            &MonsterState,
            &mut animation::FacingDirection,
        ),
        With<Enemy>,
    >,
    player_query: Query<&fixed_math::FixedTransform3D, (With<Player>, Without<Enemy>)>,
) {
    // Collect enemy positions for separation calculation
    let enemy_positions: Vec<(Entity, fixed_math::FixedVec2)> = enemy_query
        .iter()
        .map(|(_, entity, transform, ..)| (entity, transform.translation.truncate()))
        .collect();

    let separation_distance = fixed_math::new(40.0);
    let separation_force = fixed_math::new(2.0);
    let slow_down_distance = fixed_math::new(50.0);
    let optimal_attack_distance = fixed_math::new(30.0);

    for (_net_id, entity, mut transform, mut velocity, ai_config, target, state, mut facing) in
        order_mut_iter!(enemy_query)
    {
        let enemy_pos = transform.translation.truncate();

        // Only move when chasing
        if *state != MonsterState::Chasing {
            velocity.main = fixed_math::FixedVec2::ZERO;
            continue;
        }

        // Get movement direction from flow field
        let nav_profile = ai_config.nav_profile();
        let flow_field = match flow_field_cache.get_flow_field(nav_profile) {
            Some(ff) => ff,
            None => {
                // Fallback: move directly toward last known position
                if let Some(target_pos) = target.last_known_position {
                    let direction = (target_pos - enemy_pos).normalize_or_zero();
                    velocity.main = direction * fixed_math::new(50.0);
                }
                continue;
            }
        };

        // Get direction from flow field
        let direction = match flow_field.get_direction_vector(enemy_pos) {
            Some(dir) => dir,
            None => {
                // Not in flow field, move toward target directly
                if let Some(target_pos) = target.last_known_position {
                    (target_pos - enemy_pos).normalize_or_zero()
                } else {
                    fixed_math::FixedVec2::ZERO
                }
            }
        };

        // Calculate base velocity
        let base_speed = fixed_math::new(80.0); // TODO: Use character config
        let mut desired_velocity = direction * base_speed;

        // Apply separation from other enemies
        let mut separation = fixed_math::FixedVec2::ZERO;
        let mut separation_count = 0u32;

        for (other_entity, other_pos) in &enemy_positions {
            if *other_entity == entity {
                continue;
            }

            let dist = enemy_pos.distance(other_pos);
            if dist < separation_distance && dist > fixed_math::new(0.1) {
                let repulsion = (enemy_pos - *other_pos).normalize_or_zero() / dist;
                separation += repulsion;
                separation_count += 1;
            }
        }

        if separation_count > 0 {
            separation =
                (separation / fixed_math::Fixed::from_num(separation_count)) * separation_force;
        }

        // Slow down when near target
        let distance_to_nearest_player = player_query
            .iter()
            .map(|pt| enemy_pos.distance(&pt.translation.truncate()))
            .fold(
                fixed_math::Fixed::MAX,
                |acc, d| {
                    if d < acc {
                        d
                    } else {
                        acc
                    }
                },
            );

        let speed_factor = if distance_to_nearest_player < optimal_attack_distance {
            fixed_math::FIXED_ZERO
        } else if distance_to_nearest_player < slow_down_distance {
            let range = slow_down_distance - optimal_attack_distance;
            if range > fixed_math::FIXED_ZERO {
                ((distance_to_nearest_player - optimal_attack_distance) / range)
                    .clamp(fixed_math::FIXED_ZERO, fixed_math::FIXED_ONE)
            } else {
                fixed_math::FIXED_ONE
            }
        } else {
            fixed_math::FIXED_ONE
        };

        desired_velocity = desired_velocity * speed_factor;
        velocity.main = desired_velocity + separation;

        // Apply movement
        let total_velocity = velocity.main + velocity.knockback;
        let timestep = fixed_math::new(1.0 / 60.0);

        if total_velocity.length_squared() > fixed_math::new(0.01) {
            transform.translation.x = transform
                .translation
                .x
                .saturating_add(total_velocity.x * timestep);
            transform.translation.y = transform
                .translation
                .y
                .saturating_add(total_velocity.y * timestep);

            // Update facing direction
            if velocity.main.length_squared() > fixed_math::new(0.01) {
                *facing = animation::FacingDirection::from_fixed_vector(velocity.main);
            }
        }
    }
}

/// System to handle enemy attacks
pub fn enemy_attack_system(
    frame: Res<FrameCount>,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &EnemyAiConfig,
            &EnemyTarget,
            &mut MonsterState,
        ),
        With<Enemy>,
    >,
    player_query: Query<
        (&GgrsNetId, &fixed_math::FixedTransform3D),
        (With<Player>, Without<Enemy>),
    >,
    obstacle_query: Query<
        (Entity, &GgrsNetId, &fixed_math::FixedTransform3D, &Obstacle),
        (With<Rollback>, Without<Enemy>, Without<Player>),
    >,
    mut obstacle_events: ResMut<FrameEvents<ObstacleAttackEvent>>,
) {
    for (enemy_net_id, enemy_entity, enemy_transform, ai_config, target, mut state) in
        order_mut_iter!(enemy_query)
    {
        let enemy_pos = enemy_transform.translation.truncate();

        match target.target_type {
            TargetType::Player => {
                if let Some(ref target_net_id) = target.target {
                    // Find player
                    for (player_net_id, player_transform) in player_query.iter() {
                        if player_net_id != target_net_id {
                            continue;
                        }

                        let player_pos = player_transform.translation.truncate();
                        let distance = enemy_pos.distance(&player_pos);

                        if distance < ai_config.attack_range {
                            // Check if we can attack
                            let should_attack = match &*state {
                                MonsterState::Attacking {
                                    last_attack_frame, ..
                                } => {
                                    frame.frame
                                        >= *last_attack_frame + ai_config.attack_cooldown_frames
                                }
                                MonsterState::Chasing => true,
                                _ => false,
                            };

                            if should_attack {
                                // Dégât (T1.1) : pas d'écriture directe ici. `MonsterState`
                                // transitionne vers `Attacking { last_attack_frame: frame.frame,
                                // .. }` ci-dessous ; `enemy_attack_damage_translate_system`
                                // (RollbackSystemSet::CollisionDamage, avant DeathManagement)
                                // relit cette transition à la frame SUIVANTE (elle ne peut pas
                                // émettre un DamageEvent résolu la même frame : `EnemyAI` est
                                // ordonné après `CollisionDamage`, voir sa doc) et émet le
                                // DamageEvent avec `ai_config.attack_damage` — reproduisant
                                // exactement le délai d'une frame d'avant T1.1 (le dégât était
                                // déjà appliqué à la santé une frame après la décision, puisque
                                // `DeathManagement` de cette frame-ci avait déjà eu lieu).
                                *state = MonsterState::Attacking {
                                    target: AttackTarget::Player {
                                        net_id: player_net_id.clone(),
                                    },
                                    last_attack_frame: frame.frame,
                                };

                                info!(
                                    "[{}] Enemy {} attacking player {} (damage: {:?})",
                                    frame.frame,
                                    enemy_net_id,
                                    player_net_id,
                                    ai_config.attack_damage.to_num::<f32>()
                                );
                            }
                        } else {
                            // Out of range, go back to chasing
                            if matches!(*state, MonsterState::Attacking { .. }) {
                                *state = MonsterState::Chasing;
                            }
                        }
                        break;
                    }
                }
            }
            TargetType::Obstacle => {
                if let Some(ref target_net_id) = target.target {
                    // Find obstacle
                    for (obstacle_entity, obstacle_net_id, obstacle_transform, obstacle) in
                        obstacle_query.iter()
                    {
                        if obstacle_net_id != target_net_id {
                            continue;
                        }

                        // Skip if destroyed
                        if !obstacle.is_intact() {
                            *state = MonsterState::Chasing;
                            break;
                        }

                        let obstacle_pos = obstacle_transform.translation.truncate();
                        let distance = enemy_pos.distance(&obstacle_pos);

                        if distance < ai_config.attack_range {
                            let should_attack = match &*state {
                                MonsterState::Attacking {
                                    last_attack_frame, ..
                                } => {
                                    frame.frame
                                        >= *last_attack_frame + ai_config.attack_cooldown_frames
                                }
                                MonsterState::Chasing => true,
                                _ => false,
                            };

                            if should_attack {
                                // Send attack event
                                obstacle_events.send(ObstacleAttackEvent {
                                    attacker: enemy_entity,
                                    obstacle: obstacle_entity,
                                    damage: 1, // TODO: Configure per enemy
                                });

                                *state = MonsterState::Attacking {
                                    target: AttackTarget::Obstacle {
                                        net_id: obstacle_net_id.clone(),
                                    },
                                    last_attack_frame: frame.frame,
                                };

                                info!(
                                    "[{}] Enemy {} attacking obstacle {}",
                                    frame.frame, enemy_net_id, obstacle_net_id
                                );
                            }
                        } else {
                            if matches!(*state, MonsterState::Attacking { .. }) {
                                *state = MonsterState::Chasing;
                            }
                        }
                        break;
                    }
                }
            }
            TargetType::None => {
                // No target, return to idle
                if *state == MonsterState::Chasing {
                    *state = MonsterState::Idle;
                }
            }
        }
    }
}

/// Traduit une attaque d'ennemi décidée à la frame précédente (`MonsterState::Attacking`,
/// mis à jour par [`enemy_attack_system`] ci-dessus, `RollbackSystemSet::EnemyAI`) en
/// `DamageEvent` (T1.1, chantier B1).
///
/// Pourquoi une frame de retard : `RollbackSystemSet::ORDER` place `EnemyAI` **après**
/// `CollisionDamage` (où vit le résolveur unique,
/// `character::health::rollback_resolve_damage_events`) et `DeathManagement`. Un
/// `DamageEvent` émis pendant `EnemyAI` ne pourrait donc jamais être lu la même frame par
/// `CollisionDamage`, déjà passé plus tôt — et serait perdu au nettoyage de
/// `FrameEvents<DamageEvent>` au début de la frame suivante (`RollbackSystemSet::FrameStart`)
/// sans jamais être résolu. Ce système lit donc la transition de la frame **précédente**
/// (`last_attack_frame == frame - 1`) et construit l'événement ici, dans `CollisionDamage`,
/// avant `rollback_resolve_damage_events` : le dégât est appliqué exactement une frame après
/// la décision — comme avant T1.1, où `DamageAccumulator` était déjà écrit dans `EnemyAI`
/// mais n'était consommé par `DeathManagement` que la frame suivante (elle avait déjà
/// tourné, plus tôt, cette même frame). Reproduit donc la trace existante à l'identique
/// (voir le rapport, point 4) : ce n'est pas un choix de gameplay, juste la façon de faire
/// passer ce dégât par `DamageEvent` sans changer son timing.
pub fn enemy_attack_damage_translate_system(
    frame: Res<FrameCount>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    enemy_query: Query<
        (
            &GgrsNetId,
            &Team,
            Option<&Tags>,
            &EnemyAiConfig,
            &MonsterState,
        ),
        With<Enemy>,
    >,
    // Préservation d'un bug pré-existant (voir le rapport, point 4) : ne sert qu'à la
    // vérification ci-dessous, jamais à écrire.
    target_has_accumulator: Query<(&GgrsNetId, Has<DamageAccumulator>), With<Player>>,
) {
    let Some(previous_frame) = frame.frame.checked_sub(1) else {
        return;
    };

    // Bug pré-existant (avant T1.1) préservé à l'identique, PAS un choix de gameplay :
    // `enemy_attack_system` (ci-dessus) écrivait `player_damage_query.get_mut(player_entity)`
    // avec `Query<&mut DamageAccumulator>` (sans `Option`) — cette écriture n'avait donc
    // d'effet QUE si la cible portait déjà un `DamageAccumulator` à cet instant précis. Or
    // `DamageAccumulator` est retiré (`commands.remove`) dès qu'il est appliqué, la même
    // frame, par `rollback_apply_accumulated_damage` (`DeathManagement`, qui tourne avant
    // `EnemyAI` où vivait cette écriture) : en pratique, le dégât direct de
    // `EnemyAiConfig::attack_damage` ne s'appliquait donc (quasiment) jamais — seule la
    // griffe via hitbox (`weapons::melee`, `MeleeWeaponConfig::damage`, ex. 2.5 pour
    // `zombie_claws`) touchait réellement le joueur. Vérifié empiriquement (T1.1, point 4) :
    // sans cette garde, `idle`/`shoot_around`/`remote_first_fight` divergent bien au-delà des
    // lignes `DamageEvent`/`DamageAccumulator`/`HitBy` (le joueur meurt nettement plus tôt,
    // `shoot_around` ne tue même plus aucun zombie avant sa propre mort). Corriger ce bug
    // changerait l'équilibrage (dégâts ennemis ~5× plus fréquents) sans rapport avec ce
    // chantier ; à traiter séparément, délibérément, avec ses propres tests. Un résolveur
    // unique et correct (`character::health::rollback_resolve_damage_events`) ne peut pas
    // reproduire ce comportement lui-même (il utilise `Option<&mut DamageAccumulator>`,
    // correct, pour les deux autres émetteurs) : la garde vit donc ici, à l'émission.
    let mut has_accumulator: BTreeMap<usize, bool> = BTreeMap::new();
    for (net_id, has) in target_has_accumulator.iter() {
        has_accumulator.insert(net_id.0, has);
    }

    for (enemy_net_id, team, opt_tags, ai_config, state) in order_iter!(enemy_query) {
        let MonsterState::Attacking {
            target: AttackTarget::Player {
                net_id: target_net_id,
            },
            last_attack_frame,
        } = state
        else {
            continue;
        };
        if *last_attack_frame != previous_frame {
            continue;
        }
        if !has_accumulator
            .get(&target_net_id.0)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }

        // Tags du dégât (T1.1) : tags du zombie (ex. `zombie`, posé par
        // `CharacterConfig::tags`) union le genre d'attaque `melee` (voir la doc de
        // `sim_core::damage::DamageEvent::tags`).
        let mut tags = opt_tags.cloned().unwrap_or_default();
        tags.insert(Tag::new("melee"));

        damage_events.send(DamageEvent {
            source: enemy_net_id.clone(),
            target: target_net_id.clone(),
            kind: DamageKind::Physical,
            amount: ai_config.attack_damage,
            frame: frame.frame,
            tags,
            source_team: *team,
            friendly_fire: ai_config.friendly_fire,
        });
    }
}

/// System to handle stunned state recovery
pub fn enemy_stun_recovery_system(
    frame: Res<FrameCount>,
    mut enemy_query: Query<(&GgrsNetId, &mut MonsterState), With<Enemy>>,
) {
    for (_net_id, mut state) in order_mut_iter!(enemy_query) {
        if let MonsterState::Stunned { recover_at_frame } = *state {
            if frame.frame >= recover_at_frame {
                *state = MonsterState::Idle;
            }
        }
    }
}

/// Apply stun to an enemy
pub fn apply_stun(state: &mut MonsterState, current_frame: u32, stun_duration: u32) {
    *state = MonsterState::Stunned {
        recover_at_frame: current_frame + stun_duration,
    };
}
