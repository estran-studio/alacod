//! Behaviors composables (T1.4, chantier D1, `docs/conventions.md` §22) : sélection par
//! priorité et exécution des behaviors **nouveaux** (`KeepDistance`, `Strafe`, `Charge`,
//! `Flee`, `Wander`).
//!
//! Un seul chemin de code : chaque ennemi porte sa liste de règles ([`EnemyBehaviors`],
//! statique), la première applicable gagne ([`behaviors::select`]). Les règles qui existaient
//! avant T1.4 s'exécutent par les systèmes d'origine, inchangés : `Chase` (`move_enemies`),
//! `Melee` (`enemy_attack_system`, l'ennemi continue de suivre son chemin comme avant),
//! `Shoot` (`ranged_attack`, T1.2). Un ennemi qui ne liste que ces règles — tout le contenu
//! d'avant T1.4 — ne porte aucun état nouveau : sa règle retenue est **dérivée** de son état
//! ([`current_rule`]). Les behaviors nouveaux gardent leur état dans [`BehaviorRuntime`]
//! (checksum neutre), mis à jour par [`behavior_select_system`] ; leur déplacement est
//! appliqué par `move_enemies` via [`behavior_motion`].

use behaviors::{select, Behavior, BossState, SelectionContext};
use bevy::prelude::*;
use bevy_fixed::{fixed_math, rng::RngStreams};
use combat::{actors::Health, downed::Downed, emitter::Emitter};
use sim_core::{
    damage::{DamageEvent, DamageKind},
    tag::{Tag, Tags},
    team::Team,
};
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use super::{
    navigation::{FlowFieldCache, NavKey},
    state::{
        BehaviorRuntime, ChargePhase, EnemyAiConfig, EnemyBehaviors, EnemyTarget, MonsterState,
        RangedAttackState, TargetType,
    },
};
use crate::{
    character::{enemy::Enemy, health::Death, player::Player},
    frame_events::FrameEvents,
    weapons::Bullet,
};

/// D38 : pose `combat::weapons::melee::MeleeHold` sur un ennemi tant que sa règle retenue est
/// `Flee` (il ne lance plus d'attaque au corps à corps en fuyant), le retire ensuite. Lu par
/// `enemy_melee_attack_system` à la frame suivante (set `Weapon`, avant `EnemyAI`).
/// `RollbackSystemSet::EnemyAI`, en dernier (après `enemy_attack_system`).
#[allow(clippy::type_complexity)]
pub fn melee_hold_system(
    mut commands: Commands,
    enemies: Query<
        (
            &GgrsNetId,
            Entity,
            &EnemyBehaviors,
            &BehaviorRuntime,
            Has<combat::weapons::melee::MeleeHold>,
            Option<&BossState>,
        ),
        With<Enemy>,
    >,
) {
    for (_, entity, rules, runtime, held, boss) in order_iter!(enemies) {
        let fleeing = runtime
            .selected
            .and_then(|index| {
                rules
                    .active(boss.map_or(0, |b| b.phase))
                    .get(index as usize)
            })
            .is_some_and(|rule| matches!(rule, Behavior::Flee));
        if fleeing && !held {
            commands
                .entity(entity)
                .insert(combat::weapons::melee::MeleeHold);
        } else if !fleeing && held {
            commands
                .entity(entity)
                .remove::<combat::weapons::melee::MeleeHold>();
        }
    }
}

/// Nom du flux RNG de `Wander` : aucun autre système ne le consomme.
pub const BEHAVIORS_RNG_STREAM: &str = "behaviors";
/// `Wander` : nouvelle direction toutes les `WANDER_PERIOD` frames.
pub const WANDER_PERIOD: u32 = 60;
/// `Strafe` : sens alterné toutes les `STRAFE_PERIOD` frames.
pub const STRAFE_PERIOD: u32 = 45;
/// `Charge` : durée maximale de la ruée, et multiplicateur de vitesse.
pub const CHARGE_RUSH_FRAMES: u32 = 60;
pub const CHARGE_SPEED_MULT: i32 = 3;
/// `Perception::Hearing` : un tireur entendu reste connu `HEARING_FRAMES` frames.
pub const HEARING_FRAMES: u32 = 120;

/// Règle retenue d'un ennemi, nom de variante (attente `EnemyState`) : celle de
/// [`BehaviorRuntime`] s'il en porte une, sinon dérivée de l'état des règles d'origine
/// (séquence de tir en cours : `Shoot` ; `Attacking` : `Melee` ; cible connue : `Chase`).
pub fn current_rule(
    rules: &EnemyBehaviors,
    runtime: Option<&BehaviorRuntime>,
    state: &MonsterState,
    target: &EnemyTarget,
    shooting: bool,
    phase: u32,
) -> Option<&'static str> {
    if let Some(runtime) = runtime {
        return runtime
            .selected
            .and_then(|index| rules.active(phase).get(index as usize))
            .map(Behavior::name);
    }
    if shooting && rules.has("Shoot") {
        return Some("Shoot");
    }
    if matches!(state, MonsterState::Attacking { .. }) && rules.has("Melee") {
        return Some("Melee");
    }
    if target.target.is_some() && rules.has("Chase") {
        return Some("Chase");
    }
    None
}

/// Sélection des ennemis à [`BehaviorRuntime`] (behaviors nouveaux), par `GgrsNetId` :
/// faits de la frame, première règle applicable, puis avancée des états (charge, errance,
/// ouïe). `RollbackSystemSet::EnemyAI`, après `enemy_target_selection`.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn behavior_select_system(
    frame: Res<FrameCount>,
    mut rng_streams: ResMut<RngStreams>,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &EnemyAiConfig,
            &EnemyBehaviors,
            &mut EnemyTarget,
            &mut BehaviorRuntime,
            &Health,
            Option<&RangedAttackState>,
            Has<Emitter>,
            // T1.3 : `Stun`/`Freeze` (§19) : aucune règle retenue.
            Option<&combat::status::Statuses>,
            // M2-T0c : phase courante d'un boss (règles de la phase).
            Option<&BossState>,
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
            Option<&Tags>,
        ),
        (With<Player>, Without<Enemy>),
    >,
    bullet_query: Query<(&GgrsNetId, &Bullet, &fixed_math::FixedTransform3D)>,
) {
    let now = frame.frame;
    for (
        net_id,
        transform,
        ai_config,
        rules,
        mut target,
        mut runtime,
        health,
        ranged_state,
        has_emitter,
        statuses,
        boss,
    ) in order_mut_iter!(enemy_query)
    {
        let phase_rules = rules.active(boss.map_or(0, |b| b.phase));
        let position = transform.translation.truncate();

        // Ouïe : un tir de joueur né cette frame à portée rend le tireur connu.
        if let Some(radius) = rules.hearing {
            for (_, bullet, bullet_transform) in order_iter!(bullet_query) {
                if bullet.created_at == now
                    && bullet.source_team == Team::Players
                    && position.distance(&bullet_transform.translation.truncate()) < radius
                {
                    runtime.heard = Some((bullet.source.clone(), now + HEARING_FRAMES));
                    break;
                }
            }
        }
        if let Some((heard_id, until)) = runtime.heard.clone() {
            if now >= until {
                runtime.heard = None;
            } else if target.target.is_none() {
                if let Some((_, player_transform, ..)) =
                    player_query.iter().find(|(id, ..)| **id == heard_id)
                {
                    target.target = Some(heard_id);
                    target.target_type = TargetType::Player;
                    target.last_known_position = Some(player_transform.translation.truncate());
                }
            }
        }

        // Cible joueur connue (vivante, debout, non ignorée).
        let player_target = match (&target.target, target.target_type) {
            (Some(id), TargetType::Player) => player_query
                .iter()
                .find(|(player_id, ..)| *player_id == id)
                .filter(|(_, _, downed, dead, tags)| {
                    !downed && !dead && !ignored(&rules.ignore, *tags)
                })
                .map(|(player_id, player_transform, ..)| {
                    (player_id.clone(), player_transform.translation.truncate())
                }),
            _ => None,
        };
        let target_distance = player_target
            .as_ref()
            .map(|(_, pos)| position.distance(pos));
        let health_ratio = if health.max > fixed_math::FIXED_ZERO {
            health.current / health.max
        } else {
            fixed_math::FIXED_ONE
        };
        let shooting = has_emitter || ranged_state.is_some_and(|ranged| ranged.target.is_some());
        let ctx = SelectionContext {
            target_distance,
            sight: ai_config.aggro_range,
            health_ratio,
            flee_threshold: ai_config.flee_threshold,
            attack_range: ai_config.attack_range,
            melee_in_reach: target_distance.is_some_and(|d| d < ai_config.attack_range),
            shooting,
            shoot_ready: ranged_state.is_some_and(|ranged| now >= ranged.ready_at),
            charging: runtime.charge != ChargePhase::Idle,
            charge_ready: now >= runtime.charge_ready_at,
            previous: runtime.selected.map(|index| index as usize),
        };
        let selected = if combat::status::incapacitated(statuses) {
            None
        } else {
            select(phase_rules, &ctx).map(|index| index as u32)
        };
        if selected != runtime.selected {
            runtime.since_frame = now;
            info!(
                "ggrs{{f={} behavior net_id={} rule={}}}",
                now,
                net_id,
                selected
                    .and_then(|index| phase_rules.get(index as usize))
                    .map(Behavior::name)
                    .unwrap_or("none")
            );
        }
        runtime.selected = selected;
        let rule = selected.and_then(|index| phase_rules.get(index as usize));

        // Charge : télégraphe, ruée, contact, refroidissement.
        match (rule, runtime.charge.clone()) {
            (Some(Behavior::Charge { telegraph }), ChargePhase::Idle) => {
                if let Some((_, target_pos)) = &player_target {
                    runtime.charge = ChargePhase::Telegraph {
                        until: now + telegraph,
                        target: (target_pos.x, target_pos.y),
                    };
                }
            }
            (Some(Behavior::Charge { .. }), ChargePhase::Telegraph { until, target }) => {
                if now >= until {
                    runtime.charge = ChargePhase::Rush {
                        until: now + CHARGE_RUSH_FRAMES,
                        target,
                    };
                }
            }
            (
                Some(Behavior::Charge { .. }),
                ChargePhase::Rush {
                    until,
                    target: frozen,
                },
            ) => {
                let contact = player_target
                    .as_ref()
                    .filter(|(_, pos)| position.distance(pos) <= ai_config.attack_range);
                let reached = position.distance(&fixed_math::FixedVec2::new(frozen.0, frozen.1))
                    < fixed_math::new(4.0);
                if let Some((player_id, _)) = contact {
                    runtime.charge_hit = Some((now, player_id.clone()));
                    info!(
                        "ggrs{{f={} behavior net_id={} charge_hit target={}}}",
                        now, net_id, player_id
                    );
                }
                if contact.is_some() || reached || now >= until {
                    runtime.charge = ChargePhase::Idle;
                    runtime.charge_ready_at = now + ai_config.attack_cooldown_frames;
                }
            }
            (_, ChargePhase::Idle) => {}
            // Une règle plus prioritaire (ex. `Flee`) interrompt la charge.
            (_, _) => {
                runtime.charge = ChargePhase::Idle;
                runtime.charge_ready_at = now + ai_config.attack_cooldown_frames;
            }
        }

        // Errance : nouvelle direction toutes les `WANDER_PERIOD` frames (flux `behaviors`).
        if matches!(rule, Some(Behavior::Wander)) && now >= runtime.wander_next {
            let turn = rng_streams.get_mut(BEHAVIORS_RNG_STREAM).next_fixed();
            let angle = turn.saturating_mul(fixed_math::FIXED_TAU);
            runtime.wander_dir = (fixed_math::cos_fixed(angle), fixed_math::sin_fixed(angle));
            runtime.wander_next = now + WANDER_PERIOD;
        }
    }
}

/// Un joueur porte un des tags ignorés (`Targeting::Nearest { ignore }`).
pub fn ignored(ignore: &[Tag], tags: Option<&Tags>) -> bool {
    !ignore.is_empty() && tags.is_some_and(|tags| ignore.iter().any(|tag| tags.has(tag)))
}

/// Déplacement imposé par un behavior nouveau : direction (unitaire) et multiplicateur de
/// vitesse ; `None` : déplacement d'origine (`Chase`, et pendant `Melee`/`Shoot`).
pub struct BehaviorMotion {
    pub direction: fixed_math::FixedVec2,
    pub speed_mult: fixed_math::Fixed,
}

/// Déplacement du behavior retenu (appelé par `move_enemies` pour les ennemis à
/// [`BehaviorRuntime`]). `target` : position de la cible connue.
pub fn behavior_motion(
    rules: &[Behavior],
    runtime: &BehaviorRuntime,
    now: u32,
    position: fixed_math::FixedVec2,
    target: Option<fixed_math::FixedVec2>,
    flow_field_cache: &FlowFieldCache,
    // D41 + D38 : champ de l'ennemi (profil, gabarit), pour le recul.
    nav_key: NavKey,
) -> Option<BehaviorMotion> {
    let rule = runtime
        .selected
        .and_then(|index| rules.get(index as usize))?;
    let still = || BehaviorMotion {
        direction: fixed_math::FixedVec2::ZERO,
        speed_mult: fixed_math::FIXED_ZERO,
    };
    match rule {
        Behavior::KeepDistance { .. } | Behavior::Flee => {
            let retreat = flow_field_cache
                .get_flow_field(nav_key)
                .and_then(|field| field.retreat_direction(position))
                .or_else(|| target.map(|t| (position - t).normalize_or_zero()));
            Some(match retreat {
                Some(direction) => BehaviorMotion {
                    direction,
                    speed_mult: fixed_math::FIXED_ONE,
                },
                None => still(),
            })
        }
        Behavior::Strafe => {
            let to_target = (target? - position).normalize_or_zero();
            let perpendicular = fixed_math::FixedVec2::new(-to_target.y, to_target.x);
            let flip = (now.saturating_sub(runtime.since_frame) / STRAFE_PERIOD) % 2 == 1;
            Some(BehaviorMotion {
                direction: if flip {
                    fixed_math::FixedVec2::ZERO - perpendicular
                } else {
                    perpendicular
                },
                speed_mult: fixed_math::FIXED_ONE,
            })
        }
        Behavior::Wander => Some(BehaviorMotion {
            direction: fixed_math::FixedVec2::new(runtime.wander_dir.0, runtime.wander_dir.1),
            speed_mult: fixed_math::FIXED_HALF,
        }),
        Behavior::Charge { .. } => Some(match &runtime.charge {
            ChargePhase::Rush { target, .. } => BehaviorMotion {
                direction: (fixed_math::FixedVec2::new(target.0, target.1) - position)
                    .normalize_or_zero(),
                speed_mult: fixed_math::Fixed::from_num(CHARGE_SPEED_MULT),
            },
            _ => still(),
        }),
        _ => None,
    }
}

/// Dégât de contact d'une ruée (`Charge`), décidée à la frame précédente (comme
/// `enemy_attack_damage_translate_system` : un `DamageEvent` émis pendant `EnemyAI` serait
/// perdu). `RollbackSystemSet::CollisionDamage`, avant le résolveur.
pub fn charge_damage_translate_system(
    frame: Res<FrameCount>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    enemy_query: Query<
        (
            &GgrsNetId,
            &Team,
            Option<&Tags>,
            &EnemyAiConfig,
            &BehaviorRuntime,
        ),
        With<Enemy>,
    >,
) {
    let Some(previous_frame) = frame.frame.checked_sub(1) else {
        return;
    };
    for (net_id, team, tags, ai_config, runtime) in order_iter!(enemy_query) {
        let Some((hit_frame, target)) = &runtime.charge_hit else {
            continue;
        };
        if *hit_frame != previous_frame {
            continue;
        }
        let mut tags = tags.cloned().unwrap_or_default();
        tags.insert(Tag::new("melee"));
        tags.insert(Tag::new("charge"));
        damage_events.send(DamageEvent {
            source: net_id.clone(),
            target: target.clone(),
            kind: DamageKind::Physical,
            amount: ai_config.attack_damage,
            frame: frame.frame,
            tags,
            source_team: *team,
            friendly_fire: ai_config.friendly_fire,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use behaviors::Behavior;

    fn rules(list: Vec<Behavior>) -> EnemyBehaviors {
        EnemyBehaviors {
            rules: list,
            ..Default::default()
        }
    }

    #[test]
    fn regle_derivee_sans_etat_nouveau() {
        let zombie = rules(vec![
            Behavior::Melee("zombie_claws".into()),
            Behavior::Chase {
                profile: "Ground".into(),
            },
        ]);
        let mut target = EnemyTarget::default();
        assert_eq!(
            current_rule(&zombie, None, &MonsterState::Idle, &target, false, 0),
            None
        );
        target.target = Some(GgrsNetId(1, String::new()));
        assert_eq!(
            current_rule(&zombie, None, &MonsterState::Chasing, &target, false, 0),
            Some("Chase")
        );
        let attacking = MonsterState::Attacking {
            target: super::super::state::AttackTarget::Player {
                net_id: GgrsNetId(1, String::new()),
            },
            last_attack_frame: 3,
        };
        assert_eq!(
            current_rule(&zombie, None, &attacking, &target, false, 0),
            Some("Melee")
        );
    }

    #[test]
    fn strafe_alterne_toutes_les_45_frames() {
        let drifter = rules(vec![Behavior::Strafe, Behavior::Wander]);
        let runtime = BehaviorRuntime {
            selected: Some(0),
            since_frame: 100,
            ..Default::default()
        };
        let cache = FlowFieldCache::default();
        let at = fixed_math::FixedVec2::ZERO;
        let target = Some(fixed_math::FixedVec2::new(
            fixed_math::new(100.0),
            fixed_math::FIXED_ZERO,
        ));
        let first = behavior_motion(
            &drifter.rules,
            &runtime,
            110,
            at,
            target,
            &cache,
            crate::character::enemy::ai::navigation::MOVEMENT_FLOW_KEY,
        )
        .unwrap();
        let second = behavior_motion(
            &drifter.rules,
            &runtime,
            150,
            at,
            target,
            &cache,
            crate::character::enemy::ai::navigation::MOVEMENT_FLOW_KEY,
        )
        .unwrap();
        // Perpendiculaire à la cible (axe x) : le long de y, sens opposés.
        assert_eq!(first.direction.x, fixed_math::FIXED_ZERO);
        assert!(first.direction.y > fixed_math::FIXED_ZERO);
        assert!(second.direction.y < fixed_math::FIXED_ZERO);
    }

    #[test]
    fn charge_immobile_pendant_le_telegraphe_puis_rapide() {
        let charger = rules(vec![Behavior::Charge { telegraph: 30 }]);
        let cache = FlowFieldCache::default();
        let at = fixed_math::FixedVec2::ZERO;
        let frozen = (fixed_math::new(50.0), fixed_math::FIXED_ZERO);
        let mut runtime = BehaviorRuntime {
            selected: Some(0),
            charge: ChargePhase::Telegraph {
                until: 30,
                target: frozen,
            },
            ..Default::default()
        };
        let telegraph = behavior_motion(
            &charger.rules,
            &runtime,
            10,
            at,
            None,
            &cache,
            crate::character::enemy::ai::navigation::MOVEMENT_FLOW_KEY,
        )
        .unwrap();
        assert_eq!(telegraph.speed_mult, fixed_math::FIXED_ZERO);
        runtime.charge = ChargePhase::Rush {
            until: 90,
            target: frozen,
        };
        let rush = behavior_motion(
            &charger.rules,
            &runtime,
            40,
            at,
            None,
            &cache,
            crate::character::enemy::ai::navigation::MOVEMENT_FLOW_KEY,
        )
        .unwrap();
        assert_eq!(rush.speed_mult, fixed_math::Fixed::from_num(3));
        assert!(rush.direction.x > fixed_math::new(0.99));
    }

    #[test]
    fn errance_meme_graine_memes_directions() {
        let draw = |seed: u32| -> Vec<fixed_math::Fixed> {
            let mut streams = RngStreams::new(seed);
            (0..5)
                .map(|_| streams.get_mut(BEHAVIORS_RNG_STREAM).next_fixed())
                .collect()
        };
        assert_eq!(draw(42), draw(42));
        assert_ne!(draw(42), draw(43));
    }

    #[test]
    fn tags_ignores() {
        let ghost = Tag::new("ghost");
        let mut tags = Tags::new();
        tags.insert(ghost.clone());
        assert!(ignored(std::slice::from_ref(&ghost), Some(&tags)));
        assert!(!ignored(&[], Some(&tags)));
        assert!(!ignored(&[Tag::new("other")], Some(&tags)));
        assert!(!ignored(std::slice::from_ref(&ghost), None));
    }
}
