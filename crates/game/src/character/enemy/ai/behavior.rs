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
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use crate::character::enemy::Enemy;
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
/// Target: the closest player in aggro range or, beyond it, within `sight_range` and in clear
/// line of sight (D66), unless an intact breakable obstacle (window)
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
            // Suivi Q12 : `Charm` (§19) : l'ennemi charmé cible l'ennemi hostile le plus proche.
            Option<&combat::status::Statuses>,
            &Team,
            // D176 : corps de l'ennemi, pour savoir s'il bute sur une fenêtre au point visé.
            Option<&crate::collider::Collider>,
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
    let breakables: Vec<Breakable> = utils::order_iter!(obstacle_query)
        .into_iter()
        .filter(|(_, _, obstacle, _)| obstacle.blocks_movement && obstacle.breakable)
        .map(|(net_id, transform, _, collider)| Breakable {
            net_id: net_id.clone(),
            translation: transform.translation,
            cells: super::navigation::get_collider_cells(
                transform.translation.truncate(),
                collider,
            ),
            collider: collider.clone(),
        })
        .collect();

    // Suivi Q12 : les ennemis qu'un charmé peut viser (vivants, non charmés), seulement quand l'un
    // d'eux est charmé (aucun calcul ni allocation sinon).
    let any_charmed = enemy_query
        .iter()
        .any(|item| combat::status::charmed(item.6));
    let hostile_enemies: Vec<(GgrsNetId, fixed_math::FixedVec2)> = if any_charmed {
        enemy_query
            .iter()
            .filter(|(_, _, _, _, state, _, statuses, team, _)| {
                **state != MonsterState::Dead
                    && !combat::status::charmed(*statuses)
                    && **team == Team::Enemies
            })
            .map(|(net_id, transform, ..)| (net_id.clone(), transform.translation.truncate()))
            .collect()
    } else {
        Vec::new()
    };

    for (
        enemy_net_id,
        enemy_transform,
        ai_config,
        mut target,
        mut state,
        enemy_behaviors,
        statuses,
        _team,
        enemy_collider,
    ) in order_mut_iter!(enemy_query)
    {
        let enemy_pos = enemy_transform.translation.truncate();

        // Suivi Q12 : un ennemi charmé ne vise que les ennemis hostiles ; le statut expiré, il oublie
        // sa cible `Enemy` et redevient hostile (le chemin d'origine ci-dessous reprend la main).
        if combat::status::charmed(statuses) {
            charm_target_selection(
                enemy_net_id,
                enemy_pos,
                &mut target,
                &mut state,
                &hostile_enemies,
            );
            continue;
        }
        if target.target_type == TargetType::Enemy {
            target.target = None;
            target.target_type = TargetType::None;
            target.last_known_position = None;
            if matches!(
                *state,
                MonsterState::Attacking {
                    target: AttackTarget::Enemy { .. },
                    ..
                }
            ) {
                *state = MonsterState::Chasing;
            }
        }

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

        // Un joueur est perçu à moins de `aggro_range` (rayon seul) ou, à moins de `sight_range`
        // (D66, défaut `aggro_range` : rien ne change), en ligne de vue dégagée. L'ennemi déjà
        // en chasse d'un joueur le garde tant qu'il est à portée de vue, même derrière un coin :
        // sans cela, le champ de flux qui contourne un mur lui ferait perdre sa cible en route.
        // Aucun état nouveau : la cible mémorisée est celle de `EnemyTarget`, déjà rollback.
        let sight_range = ai_config.effective_sight_range();
        let alerted = (*state == MonsterState::Chasing && target.target_type == TargetType::Player)
            .then(|| target.target.clone())
            .flatten();
        let perceives =
            |id: &GgrsNetId, pos: &fixed_math::FixedVec2, distance: fixed_math::Fixed| {
                distance < ai_config.aggro_range
                    || (distance < sight_range
                        && (alerted.as_ref() == Some(id)
                            || flow_field_cache.line_of_sight(enemy_pos, *pos)))
            };

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
            .filter(|(id, distance, pos)| perceives(id, pos, *distance));

        // Find closest player deterministically
        let mut closest_player: Option<(GgrsNetId, fixed_math::Fixed, fixed_math::FixedVec2)> =
            path_player;

        let has_path_player = closest_player.is_some();
        for (player_net_id, player_transform) in players.iter().filter(|_| !has_path_player) {
            let player_pos = player_transform.translation.truncate();
            let distance = enemy_pos.distance(&player_pos);

            if perceives(player_net_id, &player_pos, distance) {
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
            let blocking = window_on_the_way(
                &flow_field_cache,
                enemy_transform.translation,
                enemy_collider,
                ai_config.nav_profile(),
                ai_config.attack_range,
                &ahead,
                &breakables,
            );
            if let Some(window) = blocking {
                target.target = Some(window.net_id.clone());
                target.target_type = TargetType::Obstacle;
                target.last_known_position = Some(window.translation.truncate());
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

/// Obstacle intact et cassable (fenêtre) vu par [`enemy_target_selection`].
struct Breakable {
    net_id: GgrsNetId,
    translation: fixed_math::FixedVec3,
    /// Cases couvertes par son collider.
    cells: Vec<super::navigation::GridPos>,
    collider: crate::collider::Collider,
}

/// Fenêtre intacte à casser avant de passer (premier `GgrsNetId` d'abord), à moins de
/// `attack_range` : l'une des cases de `ahead` (case de l'ennemi puis celles du chemin) est sous
/// son collider, ou, D176, le corps de l'ennemi posé au point visé de sa case suivante
/// ([`FlowFieldCache::steering_point`]) la chevauche. Le second cas est celui d'une ouverture plus
/// large que la fenêtre : le chemin longe la fenêtre par une case libre, trop étroite pour le corps
/// tant que la fenêtre tient ; sans lui, l'ennemi poussait contre son bord sans jamais la frapper.
fn window_on_the_way<'a>(
    cache: &FlowFieldCache,
    enemy_translation: fixed_math::FixedVec3,
    enemy_collider: Option<&crate::collider::Collider>,
    profile: super::navigation::NavProfile,
    attack_range: fixed_math::Fixed,
    ahead: &[super::navigation::GridPos],
    breakables: &'a [Breakable],
) -> Option<&'a Breakable> {
    let enemy_pos = enemy_translation.truncate();
    let in_range = |window: &&Breakable| {
        enemy_pos.distance(&window.translation.truncate()) < attack_range
    };
    breakables
        .iter()
        .filter(in_range)
        .find(|window| ahead.iter().any(|cell| window.cells.contains(cell)))
        .or_else(|| {
            let (next, enemy_collider) = (ahead.get(1)?, enemy_collider?);
            let body = super::navigation::AgentBody::from_collider(enemy_collider);
            let point = cache.steering_point(*next, profile, &body);
            let at_point = fixed_math::FixedVec3::new(point.x, point.y, enemy_translation.z);
            breakables.iter().filter(in_range).find(|window| {
                crate::collider::is_colliding(
                    &at_point,
                    enemy_collider,
                    &window.translation,
                    &window.collider,
                )
            })
        })
}

/// Suivi Q12 : attaque d'un ennemi charmé. Une salve commencée contre un joueur est interrompue ;
/// seule une cible `TargetType::Enemy` encore présente est frappée, au corps à corps, avec la même
/// portée et la même cadence que contre un joueur (`MonsterState::Attacking { Enemy }`, dégât traduit
/// par [`enemy_attack_damage_translate_system`]) ; toute autre attaque est abandonnée.
#[allow(clippy::too_many_arguments)]
fn charm_attack(
    commands: &mut Commands,
    frame: u32,
    (enemy_net_id, enemy_entity, enemy_pos): (&GgrsNetId, Entity, fixed_math::FixedVec2),
    ai_config: &EnemyAiConfig,
    target: &EnemyTarget,
    state: &mut MonsterState,
    ranged_state: Option<Mut<RangedAttackState>>,
    has_emitter: bool,
    hostile: &[(GgrsNetId, fixed_math::FixedVec2)],
    // Peur et charme cumulés : l'ennemi fuit (`Fear` l'emporte, `pathing`), il ne frappe pas non plus.
    feared: bool,
) {
    if has_emitter {
        commands.entity(enemy_entity).remove::<Emitter>();
    }
    if let Some(mut ranged) = ranged_state {
        if ranged.target.is_some() {
            ranged.target = None;
        }
    }
    if feared {
        if matches!(*state, MonsterState::Attacking { .. }) {
            *state = MonsterState::Chasing;
        }
        return;
    }
    let hit = (target.target_type == TargetType::Enemy)
        .then(|| target.target.as_ref())
        .flatten()
        .and_then(|target_id| {
            hostile
                .iter()
                .find(|(id, _)| id == target_id)
                .map(|(id, pos)| (id, *pos))
        });
    let Some((target_id, target_pos)) = hit else {
        if matches!(*state, MonsterState::Attacking { .. }) {
            *state = MonsterState::Chasing;
        }
        return;
    };
    if enemy_pos.distance(&target_pos) >= ai_config.attack_range {
        if matches!(*state, MonsterState::Attacking { .. }) {
            *state = MonsterState::Chasing;
        }
        return;
    }
    let should_attack = match &*state {
        MonsterState::Attacking {
            last_attack_frame, ..
        } => frame >= *last_attack_frame + ai_config.attack_cooldown_frames,
        MonsterState::Chasing => true,
        _ => false,
    };
    if should_attack {
        *state = MonsterState::Attacking {
            target: AttackTarget::Enemy {
                net_id: target_id.clone(),
            },
            last_attack_frame: frame,
        };
        info!(
            "ggrs{{f={} charm_attack net_id={} target={}}}",
            frame, enemy_net_id.0, target_id.0
        );
    }
}

/// Suivi Q12 : cible d'un ennemi charmé (`charm::nearest_target`, égalité : plus petit `GgrsNetId`).
/// Une attaque en cours contre un ennemi encore présent va à son terme ; toute autre attaque est
/// abandonnée (jamais de coup sur un joueur ou une fenêtre pendant le charme).
fn charm_target_selection(
    own_id: &GgrsNetId,
    position: fixed_math::FixedVec2,
    target: &mut EnemyTarget,
    state: &mut MonsterState,
    hostile: &[(GgrsNetId, fixed_math::FixedVec2)],
) {
    if *state == MonsterState::Dead {
        return;
    }
    if let MonsterState::Attacking {
        target: AttackTarget::Enemy { net_id },
        ..
    } = &*state
    {
        if hostile.iter().any(|(id, _)| id == net_id) {
            return;
        }
    }
    if matches!(*state, MonsterState::Attacking { .. }) {
        *state = MonsterState::Chasing;
    }
    let nearest = super::charm::nearest_target(
        position,
        hostile
            .iter()
            .filter(|(id, _)| id.0 != own_id.0)
            .map(|(id, pos)| (id.0, *pos)),
    );
    match nearest {
        Some((id, pos)) => {
            target.target = hostile
                .iter()
                .find(|(candidate, _)| candidate.0 == id)
                .map(|(candidate, _)| candidate.clone());
            target.target_type = TargetType::Enemy;
            target.last_known_position = Some(pos);
            *state = MonsterState::Chasing;
        }
        None => {
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
            // Suivi Q12 : équipe (un charmé ne vise que les `Enemies`).
            &Team,
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
    // Suivi Q12 : cibles possibles des ennemis charmés (vivants, non charmés), seulement si l'un
    // d'eux est charmé : aucun calcul ni allocation sinon.
    let any_charmed = enemy_query
        .iter()
        .any(|item| combat::status::charmed(item.11));
    let hostile_enemies: Vec<(GgrsNetId, fixed_math::FixedVec2)> = if any_charmed {
        enemy_query
            .iter()
            .filter(|item| {
                *item.5 != MonsterState::Dead
                    && !combat::status::charmed(item.11)
                    && *item.13 == Team::Enemies
            })
            .map(|item| (item.0.clone(), item.2.translation.truncate()))
            .collect()
    } else {
        Vec::new()
    };

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
        _team,
    ) in order_mut_iter!(enemy_query)
    {
        if combat::status::incapacitated(statuses) {
            continue;
        }
        let enemy_pos = enemy_transform.translation.truncate();

        // Suivi Q12 : un ennemi charmé n'attaque que sa cible ennemie (corps à corps), jamais un joueur.
        if combat::status::charmed(statuses) {
            charm_attack(
                &mut commands,
                frame.frame,
                (enemy_net_id, enemy_entity, enemy_pos),
                ai_config,
                target,
                &mut state,
                ranged_state,
                has_emitter,
                &hostile_enemies,
                combat::status::feared(statuses),
            );
            continue;
        }

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

        // M2-T7 : une règle `Shoot` propre à la phase courante d'un boss remplace la table de
        // tir de la config ; sans elle, `ai_config.ranged` (chemin d'avant).
        let phase_ranged = boss
            .zip(enemy_behaviors)
            .and_then(|(boss, rules)| rules.phase_shoot(boss.phase));
        let ranged_config = phase_ranged.as_ref().or(ai_config.ranged.as_ref());

        // Tir à distance (T1.2) : avant le corps à corps.
        if let (true, Some(ranged), Some(mut ranged_state)) =
            (shoot_allowed, ranged_config, ranged_state)
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
        // M3-T1 : sous `Fear` l'ennemi fuit (`Flee` imposé) : aucune attaque au corps à corps.
        if combat::status::feared(statuses) {
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
            // Suivi Q12 : une cible `Enemy` n'existe que pour un charmé (branche ci-dessus) ; effacée par
            // `enemy_target_selection` dès la fin du charme, jamais lue ici.
            TargetType::None | TargetType::Enemy => {
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
/// tourné, plus tôt, cette même frame).
///
/// D28 : le dégât direct est émis à chaque attaque. Avant, il n'était émis que si la cible
/// portait déjà un `DamageAccumulator` (reliquat d'avant T1.1 : l'écriture se faisait avec une
/// `Query<&mut DamageAccumulator>` sans `Option`, or le composant est retiré dès qu'il est
/// appliqué), donc presque jamais, et seule la hitbox de mêlée touchait le joueur. Les ennemis
/// font donc plus mal ; `attack_damage` (donnée du personnage) sert au rééquilibrage.
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
            // D29 : dégâts de la vague d'apparition.
            Option<&combat::weapons::melee::WaveDamageScale>,
        ),
        With<Enemy>,
    >,
) {
    let Some(previous_frame) = frame.frame.checked_sub(1) else {
        return;
    };

    for (enemy_net_id, team, opt_tags, ai_config, state, ranged_state, wave_scale) in
        order_iter!(enemy_query)
    {
        // T1.2 : `Attacking` posé par un tir à distance (séquence en cours) : les dégâts
        // passent par les projectiles, jamais par ce coup direct.
        if ranged_state.is_some_and(|ranged| ranged.target.is_some()) {
            continue;
        }
        // Suivi Q12 : un coup de charmé sur un ennemi est un coup de l'équipe `Players`
        // (adversaire de sa cible : `team_allows_hit` le laisse passer, jamais un allié du joueur).
        let (target_net_id, last_attack_frame, source_team) = match state {
            MonsterState::Attacking {
                target: AttackTarget::Player { net_id },
                last_attack_frame,
            } => (net_id, last_attack_frame, *team),
            MonsterState::Attacking {
                target: AttackTarget::Enemy { net_id },
                last_attack_frame,
            } => (net_id, last_attack_frame, Team::Players),
            _ => continue,
        };
        if *last_attack_frame != previous_frame {
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
            amount: wave_scale.map_or(ai_config.attack_damage, |scale| {
                ai_config.attack_damage.saturating_mul(scale.0)
            }),
            frame: frame.frame,
            tags,
            source_team,
            friendly_fire: ai_config.friendly_fire,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: usize) -> GgrsNetId {
        GgrsNetId(n, "test".into())
    }

    fn at(x: f32, y: f32) -> fixed_math::FixedVec2 {
        fixed_math::FixedVec2::new(fixed_math::new(x), fixed_math::new(y))
    }

    /// Suivi Q12 : un charmé vise l'ennemi hostile le plus proche (jamais lui-même) et abandonne une
    /// attaque en cours contre un joueur.
    #[test]
    fn charme_vise_l_ennemi_le_plus_proche_et_lache_le_joueur() {
        let hostile = [(id(20), at(100.0, 0.0)), (id(16), at(40.0, 0.0))];
        let mut target = EnemyTarget {
            target: Some(id(1)),
            target_type: TargetType::Player,
            last_known_position: Some(at(-50.0, 0.0)),
        };
        let mut state = MonsterState::Attacking {
            target: AttackTarget::Player { net_id: id(1) },
            last_attack_frame: 10,
        };
        charm_target_selection(&id(14), at(0.0, 0.0), &mut target, &mut state, &hostile);
        assert_eq!(target.target, Some(id(16)));
        assert_eq!(target.target_type, TargetType::Enemy);
        assert_eq!(target.last_known_position, Some(at(40.0, 0.0)));
        assert_eq!(state, MonsterState::Chasing);
    }

    /// Une attaque en cours contre un ennemi encore présent n'est pas interrompue ; sa cible disparue,
    /// le charmé en cherche une autre, et sans ennemi hostile il se met en attente.
    #[test]
    fn charme_finit_son_coup_puis_cherche_ou_attend() {
        let attacking = MonsterState::Attacking {
            target: AttackTarget::Enemy { net_id: id(16) },
            last_attack_frame: 10,
        };
        let mut target = EnemyTarget::default();
        let mut state = attacking.clone();
        charm_target_selection(
            &id(14),
            at(0.0, 0.0),
            &mut target,
            &mut state,
            &[(id(16), at(500.0, 0.0)), (id(17), at(10.0, 0.0))],
        );
        assert_eq!(
            state, attacking,
            "la cible 16 est encore là, même si 17 est plus proche"
        );
        assert_eq!(target.target, None);
        assert_eq!(target.target_type, TargetType::None);

        charm_target_selection(
            &id(14),
            at(0.0, 0.0),
            &mut target,
            &mut state,
            &[(id(17), at(10.0, 0.0))],
        );
        assert_eq!(target.target, Some(id(17)));
        assert_eq!(state, MonsterState::Chasing);

        charm_target_selection(&id(14), at(0.0, 0.0), &mut target, &mut state, &[]);
        assert_eq!(target.target, None);
        assert_eq!(target.target_type, TargetType::None);
        assert_eq!(state, MonsterState::Idle);
    }

    /// Un ennemi mort ne choisit plus de cible.
    #[test]
    fn charme_mort_ne_change_rien() {
        let mut target = EnemyTarget::default();
        let mut state = MonsterState::Dead;
        charm_target_selection(
            &id(14),
            at(0.0, 0.0),
            &mut target,
            &mut state,
            &[(id(16), at(10.0, 0.0))],
        );
        assert_eq!(state, MonsterState::Dead);
        assert_eq!(target.target, None);
        assert_eq!(target.target_type, TargetType::None);
    }

    fn rect(width: f32, height: f32, offset_y: f32) -> crate::collider::Collider {
        crate::collider::Collider {
            shape: crate::collider::ColliderShape::Rectangle {
                width: fixed_math::new(width),
                height: fixed_math::new(height),
            },
            offset: fixed_math::FixedVec3::new(
                fixed_math::FIXED_ZERO,
                fixed_math::new(offset_y),
                fixed_math::FIXED_ZERO,
            ),
        }
    }

    /// D176, cas de la fenêtre intacte (contrefactuel de b2 sur `avant_poste`, ouverture élargie à
    /// quatre cases x112..176 sur y48..64, fenêtre 32 × 16 en (144, 56) sur les cases (8,3) et (9,3)) :
    /// le chemin d'un `zombie_1` en (160, 44) passe par la case libre (10,3), dont le point visé
    /// (160, 56) met son corps (20 × 20, décalage y −6) sur le bord de la fenêtre. Aucune case du
    /// chemin n'est sous la fenêtre : avant D176 il poussait contre elle sans la frapper. Plus loin
    /// de l'ouverture, le même chemin ne vise aucune fenêtre.
    #[test]
    fn fenetre_au_bord_du_point_vise_est_a_casser() {
        use super::super::navigation::{GridPos, NavProfile};
        use crate::character::enemy::ai::ObstacleType;
        let mut cache = FlowFieldCache::default();
        for x in (0..7).chain(11..16) {
            cache.wall_cells.insert(GridPos::new(x, 3));
        }
        let window = Breakable {
            net_id: id(54),
            translation: at(144.0, 56.0).extend(),
            cells: vec![GridPos::new(8, 3), GridPos::new(9, 3)],
            collider: rect(32.0, 16.0, 0.0),
        };
        cache
            .blocked_cells
            .entry(ObstacleType::Window)
            .or_default()
            .extend(window.cells.iter().copied());
        let zombie = rect(20.0, 20.0, -6.0);
        let breakables = [window];
        let ahead = [GridPos::new(10, 2), GridPos::new(10, 3), GridPos::new(10, 4)];
        let found = window_on_the_way(
            &cache,
            at(160.0, 44.0).extend(),
            Some(&zombie),
            NavProfile::GroundBreaker,
            fixed_math::new(40.0),
            &ahead,
            &breakables,
        );
        assert_eq!(found.map(|w| w.net_id.0), Some(54));
        // Sans corps connu, seul le critère des cases compte : rien à casser sur ce chemin.
        let without_body = window_on_the_way(
            &cache,
            at(160.0, 44.0).extend(),
            None,
            NavProfile::GroundBreaker,
            fixed_math::new(40.0),
            &ahead,
            &breakables,
        );
        assert!(without_body.is_none());
        // Un zombie qui longe le mur sous l'ouverture (case suivante (11,2), point visé loin de la
        // fenêtre) ne la vise pas.
        let along = [GridPos::new(10, 2), GridPos::new(11, 2), GridPos::new(12, 2)];
        let passing = window_on_the_way(
            &cache,
            at(160.0, 40.0).extend(),
            Some(&zombie),
            NavProfile::GroundBreaker,
            fixed_math::new(40.0),
            &along,
            &breakables,
        );
        assert!(passing.is_none());
    }
}
