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
use combat::downed::Downed;
use sim_core::damage::{DamageEvent, DamageKind};
use sim_core::tag::{Tag, Tags};
use sim_core::team::Team;
use std::collections::BTreeMap;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use crate::character::enemy::Enemy;
use crate::character::health::DamageAccumulator;
use crate::character::player::Player;
use crate::frame_events::FrameEvents;

use super::navigation::FlowFieldCache;
use super::obstacle::{Obstacle, ObstacleAttackEvent};
use super::state::{
    AttackTarget, BehaviorRuntime, EnemyAiConfig, EnemyBehaviors, EnemyTarget, MonsterState,
    RangedAttack, RangedAttackState, TargetType,
};
use crate::character::health::Death;
use crate::weapons::WeaponInventory;
use combat::emitter::Emitter;
use combat::projectile::{resolve_pattern, Pattern, PatternLibrary};
use sim_core::stats::StatId;
use stats::StatReader;

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
            // T1.4 : `Targeting::Nearest { ignore }` (vide pour tout le contenu existant).
            Option<&EnemyBehaviors>,
        ),
        // M2-E1 : un ennemi d'une salle dormante saute son tour (`world::RoomDormant`).
        (With<Enemy>, Without<world::RoomDormant>),
    >,
    player_query: Query<
        (&GgrsNetId, &fixed_math::FixedTransform3D, Has<Downed>),
        (With<Player>, Without<Enemy>),
    >,
    player_tags: Query<(&GgrsNetId, Option<&Tags>), With<Player>>,
) {
    // Collect and sort players for deterministic iteration
    let mut players: Vec<_> = player_query.iter().collect();
    // À terre (T1.3, chantier B6) : même règle que `update_flow_field_system`/
    // `update_enemy_targets` — ignorer les joueurs à terre tant qu'un autre est encore
    // debout (voir la doc de `combat::downed::Downed`).
    let any_standing = players.iter().any(|(_, _, downed)| !downed);
    if any_standing {
        players.retain(|(_, _, downed)| !downed);
    }
    let mut players: Vec<_> = players.into_iter().map(|(id, t, _)| (id, t)).collect();
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

    for (_enemy_net_id, enemy_transform, ai_config, mut target, mut state, enemy_behaviors) in
        order_mut_iter!(enemy_query)
    {
        let enemy_pos = enemy_transform.translation.truncate();

        // T1.4 : joueurs ignorés par tag (`Targeting::Nearest { ignore }`) ; liste vide (tout
        // le contenu d'avant T1.4) : aucun filtre, `players` tel quel.
        let ignore = enemy_behaviors
            .map(|behaviors| behaviors.ignore.as_slice())
            .unwrap_or_default();
        let filtered;
        let players: &Vec<_> = if ignore.is_empty() {
            &players
        } else {
            filtered = players
                .iter()
                .filter(|(id, _)| {
                    !player_tags
                        .iter()
                        .find(|(player_id, _)| player_id == id)
                        .is_some_and(|(_, tags)| super::rules::ignored(ignore, tags))
                })
                .copied()
                .collect::<Vec<_>>();
            &filtered
        };

        // Don't retarget if attacking, stunned, or dead
        match *state {
            MonsterState::Attacking { .. } | MonsterState::Dead => continue,
            _ => {}
        }

        // Target the player the flow field leads to (the closest one along the path), so the
        // enemy chases and attacks the same player; fall back to the closest in straight line.
        // Champ du profil de l'ennemi (D38), gabarit petit : le joueur visé dépend des obstacles
        // que l'ennemi passe, pas de sa taille.
        let path_player = flow_field_cache
            .nearest_target(
                super::navigation::NavKey::new(
                    ai_config.nav_profile(),
                    super::navigation::AgentSize::Small,
                ),
                enemy_pos,
            )
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
            // Fenêtre sur le chemin : lue dans le champ qui traverse les cassables (seuls les
            // ennemis qui la cassent la cherchent).
            ahead.extend(flow_field_cache.path_ahead(
                super::navigation::MOVEMENT_FLOW_KEY,
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

/// System to handle enemy attacks
///
/// Tir à distance (T1.2, `EnemyAiConfig::ranged`) : voir [`ranged_attack`], joué avant le
/// corps à corps ; un ennemi qui tire (émetteur posé) ne passe pas par le corps à corps.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn enemy_attack_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &EnemyAiConfig,
            &EnemyTarget,
            &mut MonsterState,
            Option<&mut RangedAttackState>,
            Has<Emitter>,
            Option<&WeaponInventory>,
            // T1.4 : règles et état des behaviors nouveaux (voir `rules`).
            Option<&EnemyBehaviors>,
            Option<&BehaviorRuntime>,
            // T1.3 : `Stun`/`Freeze` (§19) : ni tir ni corps à corps.
            Option<&combat::status::Statuses>,
            // M2-T0c : phase courante d'un boss.
            Option<&behaviors::BossState>,
        ),
        // M2-E1 : un ennemi d'une salle dormante saute son tour (`world::RoomDormant`).
        (With<Enemy>, Without<world::RoomDormant>),
    >,
    player_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            Has<Downed>,
            Has<Death>,
        ),
        (With<Player>, Without<Enemy>),
    >,
    library: Option<Res<PatternLibrary>>,
    stats: StatReader,
    obstacle_query: Query<
        (Entity, &GgrsNetId, &fixed_math::FixedTransform3D, &Obstacle),
        (With<Rollback>, Without<Enemy>, Without<Player>),
    >,
    mut obstacle_events: ResMut<FrameEvents<ObstacleAttackEvent>>,
) {
    for (
        enemy_net_id,
        enemy_entity,
        enemy_transform,
        ai_config,
        target,
        mut state,
        ranged_state,
        has_emitter,
        inventory,
        enemy_behaviors,
        behavior_runtime,
        statuses,
        boss,
    ) in order_mut_iter!(enemy_query)
    {
        if combat::status::incapacitated(statuses) {
            continue;
        }
        let enemy_pos = enemy_transform.translation.truncate();

        // T1.4 : un ennemi à behaviors nouveaux (`BehaviorRuntime`) n'exécute que sa règle
        // retenue (`rules::behavior_select_system`) ; les autres — tout le contenu d'avant
        // T1.4 — suivent exactement le chemin d'origine ci-dessous.
        let runtime_rule: Option<Option<&str>> = behavior_runtime.map(|runtime| {
            runtime
                .selected
                .and_then(|index| {
                    enemy_behaviors?
                        .active(boss.map_or(0, |b| b.phase))
                        .get(index as usize)
                })
                .map(|rule| rule.name())
        });
        let shoot_allowed = runtime_rule.is_none_or(|rule| {
            rule == Some("Shoot")
                || ranged_state
                    .as_ref()
                    .is_some_and(|ranged| ranged.target.is_some())
        });

        // Tir à distance (T1.2) : avant le corps à corps.
        if let (true, Some(ranged), Some(mut ranged_state)) =
            (shoot_allowed, &ai_config.ranged, ranged_state)
        {
            let firing = ranged_attack(
                &mut commands,
                frame.frame,
                RangedShooter {
                    net_id: enemy_net_id,
                    entity: enemy_entity,
                    position: enemy_pos,
                    has_emitter,
                    inventory,
                },
                ranged,
                target,
                &mut ranged_state,
                &mut state,
                &player_query,
                library.as_deref(),
                &stats,
            );
            if firing {
                continue;
            }
        }
        if runtime_rule.is_some_and(|rule| rule != Some("Melee")) {
            continue;
        }

        match target.target_type {
            TargetType::Player => {
                if let Some(ref target_net_id) = target.target {
                    // Find player
                    for (player_net_id, player_transform, _, _) in player_query.iter() {
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

/// Tireur d'une séquence de tir à distance (regroupe les composants lus par
/// [`ranged_attack`]).
pub struct RangedShooter<'a> {
    pub net_id: &'a GgrsNetId,
    pub entity: Entity,
    pub position: fixed_math::FixedVec2,
    /// Un `combat::emitter::Emitter` est posé (séquence en cours, pas encore finie).
    pub has_emitter: bool,
    pub inventory: Option<&'a WeaponInventory>,
}

/// Tir à distance d'un ennemi (T1.2, `docs/conventions.md` §20). Rend `true` si l'ennemi est
/// en train de tirer (le corps à corps est alors sauté cette frame).
///
/// - Séquence en cours (`ranged_state.target`) : interrompue si la cible n'existe plus,
///   est morte, à terre ou hors de `range` (l'émetteur est retiré) ; finie si
///   `emitter_system` a retiré l'émetteur. Dans les deux cas : refroidissement de
///   `cooldown_frames`, retour à `Chasing`.
/// - Sinon, cible `Player` vivante, debout, à moins de `range`, refroidissement écoulé,
///   état `Idle`/`Chasing` : pose un émetteur visant la cible (visée figée), passe
///   `Attacking { target: Player }`. L'ennemi ne bouge plus tant que l'émetteur est posé
///   (`pathing::move_enemies`).
#[allow(clippy::too_many_arguments)]
pub fn ranged_attack(
    commands: &mut Commands,
    frame: u32,
    shooter: RangedShooter,
    ranged: &RangedAttack,
    target: &EnemyTarget,
    ranged_state: &mut RangedAttackState,
    state: &mut MonsterState,
    player_query: &Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            Has<Downed>,
            Has<Death>,
        ),
        (With<Player>, Without<Enemy>),
    >,
    library: Option<&PatternLibrary>,
    stats: &StatReader,
) -> bool {
    // Cible jouable : existe, vivante, debout, à portée.
    let target_in_reach = |net_id: &GgrsNetId| -> Option<fixed_math::FixedVec2> {
        player_query
            .iter()
            .find(|(id, ..)| *id == net_id)
            .filter(|(_, _, downed, dead)| !downed && !dead)
            .map(|(_, transform, ..)| transform.translation.truncate())
            .filter(|pos| shooter.position.distance(pos) < ranged.range)
    };

    if let Some(current) = ranged_state.target.clone() {
        let interrupted = target_in_reach(&current).is_none();
        if shooter.has_emitter && !interrupted {
            return true;
        }
        if shooter.has_emitter {
            commands.entity(shooter.entity).remove::<Emitter>();
        }
        info!(
            "ggrs{{f={} emitter net_id={} {} target={}}}",
            frame,
            shooter.net_id,
            if interrupted { "interrupted" } else { "done" },
            current
        );
        ranged_state.target = None;
        ranged_state.ready_at = frame.saturating_add(ranged.cooldown_frames);
        *state = MonsterState::Chasing;
        return false;
    }

    if frame < ranged_state.ready_at
        || target.target_type != TargetType::Player
        || !matches!(*state, MonsterState::Idle | MonsterState::Chasing)
    {
        return false;
    }
    let Some(target_net_id) = target.target.clone() else {
        return false;
    };
    let Some(target_pos) = target_in_reach(&target_net_id) else {
        return false;
    };

    let Some(weapon) = shooter.inventory.and_then(|inventory| {
        inventory
            .weapons
            .iter()
            .map(|(_, weapon)| weapon)
            .find(|weapon| weapon.config.name == ranged.weapon)
    }) else {
        return false;
    };
    let named = Pattern::Named(ranged.pattern.clone());
    let pattern = match resolve_pattern(library, &named) {
        Ok(pattern) => pattern,
        Err(name) => {
            warn!(
                "ennemi {} : pattern « {} » inconnu (voir `alacod lint`), pas de tir",
                shooter.net_id, name
            );
            return false;
        }
    };
    let aim = target_pos - shooter.position;
    let damage_mult = stats.get(shooter.entity, &StatId::Damage, fixed_math::FIXED_ONE);
    commands.entity(shooter.entity).insert(Emitter::new(
        ranged.pattern.clone(),
        &pattern,
        ranged.weapon.clone(),
        std::sync::Arc::new(weapon.config.projectiles.clone()),
        damage_mult,
        weapon.config.friendly_fire,
        aim,
        frame,
    ));
    ranged_state.target = Some(target_net_id.clone());
    *state = MonsterState::Attacking {
        target: AttackTarget::Player {
            net_id: target_net_id.clone(),
        },
        last_attack_frame: frame,
    };
    info!(
        "ggrs{{f={} emitter net_id={} start pattern={} target={}}}",
        frame, shooter.net_id, ranged.pattern, target_net_id
    );
    true
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
            // T1.2 : une séquence de tir à distance n'est pas un coup de corps à corps.
            Option<&RangedAttackState>,
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

    for (enemy_net_id, team, opt_tags, ai_config, state, ranged_state) in order_iter!(enemy_query) {
        // T1.2 : `Attacking` posé par un tir à distance (séquence en cours) : les dégâts
        // passent par les projectiles, jamais par ce coup direct.
        if ranged_state.is_some_and(|ranged| ranged.target.is_some()) {
            continue;
        }
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
