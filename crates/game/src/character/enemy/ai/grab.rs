//! Agrippeur (X-6d, `docs/conventions.md` §126) : exécution du behavior `Grab`
//! (`behaviors::Behavior::Grab`, valeurs en données).
//!
//! Phases ([`GrabPhase`], dans [`BehaviorRuntime`]) : `Idle` → `Crouch` (télégraphe, immobile) → `Leap`
//! (bond droit vers la position de la cible à la fin du télégraphe, mouvement de `rules::behavior_motion`)
//! → `Ride` (accroché : composant [`Riding`] sur l'agrippeur, `pathing::move_enemies` le pose sur son
//! porteur) → `Dazed` (étourdi, immobile, sans attaque) → `Idle` (attente `cooldown` avant un nouveau bond).
//! Un bond sans contact (ou sur un porteur qui a déjà `max_riders` agrippeurs) retombe en `Idle` avec la
//! même attente.
//!
//! Accroché, l'agrippeur pose sur son porteur un modificateur `MoveSpeed` × `slow` (source
//! `grab:<net_id>`, expiré la frame suivante s'il n'est pas renouvelé : un agrippeur tué ne laisse rien) et
//! peut mordre. Il se décroche, dans cet ordre de causes : porteur absent, à terre ou mort ; durée
//! maximale ; coup de mêlée lancé par le porteur cette frame ; coup de mêlée d'un joueur reçu cette frame ;
//! secouage (alternances de direction de l'**input rollback** du porteur, jamais le clavier). Tout est en
//! `Fixed` et en entiers, itéré par `GgrsNetId`, sans RNG.

use behaviors::{Behavior, BossState, GRAB_SHAKE_WINDOW};
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::PlayerInputs;
use combat::{
    actors::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP},
    downed::Downed,
    weapons::melee::MeleeAttackState,
};
use sim_core::{
    damage::DamageEvent,
    grab::Riding,
    modifier::{Modifier, ModifierOp, ModifierSource, Modifiers},
    stats::StatId,
    tag::Tag,
    team::Team,
};
use std::collections::BTreeMap;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use super::state::{BehaviorRuntime, EnemyAiConfig, EnemyBehaviors, EnemyTarget, GrabPhase, TargetType};
use crate::{
    character::{
        enemy::Enemy,
        health::Death,
        player::{jjrs::PeerConfig, Player},
    },
    frame_events::FrameEvents,
};

/// Préfixe de la source du modificateur de vitesse posé sur le porteur : `grab:<net_id de l'agrippeur>`.
pub const GRAB_SOURCE_PREFIX: &str = "grab:";

/// Distance au point visé en deçà de laquelle un bond est arrivé (unités).
const LEAP_ARRIVE: i32 = 4;

/// Valeurs d'une règle `Grab` lues par le système.
struct GrabRule {
    telegraph: u32,
    leap_frames: u32,
    hold: u32,
    slow: fixed_math::Fixed,
    sway: Option<(fixed_math::Fixed, u32)>,
    shake: u32,
    shake_window: u32,
    stun: u32,
    cooldown: u32,
    max_riders: u32,
    bite_every: Option<u32>,
}

fn grab_rule(rules: &[Behavior]) -> Option<GrabRule> {
    rules.iter().find_map(|rule| match rule {
        Behavior::Grab {
            telegraph,
            leap_frames,
            hold,
            slow,
            sway,
            shake,
            shake_window,
            stun,
            cooldown,
            max_riders,
            bite_every,
            ..
        } => Some(GrabRule {
            telegraph: *telegraph,
            leap_frames: *leap_frames,
            hold: *hold,
            slow: *slow,
            sway: sway.as_ref().map(|sway| (sway.speed, sway.period)),
            shake: *shake,
            shake_window: shake_window.unwrap_or(GRAB_SHAKE_WINDOW),
            stun: *stun,
            cooldown: *cooldown,
            max_riders: *max_riders,
            bite_every: *bite_every,
        }),
        _ => None,
    })
}

/// Le modificateur de vitesse de l'agrippeur `rider` (source `grab:<rider>`), sans allouer.
fn is_grab_source(source: &ModifierSource, rider: usize) -> bool {
    matches!(source, ModifierSource::Named(name)
        if name.strip_prefix(GRAB_SOURCE_PREFIX).and_then(|id| id.parse::<usize>().ok()) == Some(rider))
}

/// Pose ou renouvelle (jusqu'à `until`, inclus) le ralentissement de l'agrippeur `rider` sur son porteur.
fn hold_slow(modifiers: &mut Modifiers, rider: usize, slow: fixed_math::Fixed, until: u32) {
    if let Some(existing) = modifiers
        .0
        .iter_mut()
        .find(|m| m.stat == StatId::MoveSpeed && is_grab_source(&m.source, rider))
    {
        existing.until = Some(until);
        return;
    }
    modifiers.push(Modifier {
        stat: StatId::MoveSpeed,
        op: ModifierOp::Mul,
        value: slow,
        source: ModifierSource::Named(format!("{GRAB_SOURCE_PREFIX}{rider}")),
        until: Some(until),
    });
}

/// Sens (-1, 0, 1) tenu sur chaque axe d'après les boutons de déplacement de l'input.
fn axes(buttons: u32) -> (i8, i8) {
    let axis = |plus: u32, minus: u32| -> i8 {
        (buttons & plus != 0) as i8 - (buttons & minus != 0) as i8
    };
    (axis(INPUT_RIGHT, INPUT_LEFT), axis(INPUT_UP, INPUT_DOWN))
}

/// Cause d'un décrochage (journal, trace).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    /// Porteur absent, à terre ou mort.
    Carrier,
    /// Durée maximale accroché écoulée.
    Timeout,
    /// Le porteur a lancé un coup de mêlée.
    CarrierMelee,
    /// Un joueur l'a frappé de mêlée.
    Struck,
    /// Le porteur a secoué.
    Shaken,
}

impl Release {
    pub fn name(self) -> &'static str {
        match self {
            Release::Carrier => "carrier",
            Release::Timeout => "timeout",
            Release::CarrierMelee => "carrier_melee",
            Release::Struck => "struck",
            Release::Shaken => "shaken",
        }
    }
}

/// Phases de `Grab` des ennemis qui le listent, par `GgrsNetId`. `RollbackSystemSet::EnemyAI`, après
/// `behavior_select_system` (qui retient la règle) et avant `move_enemies` (qui exécute le bond et pose
/// l'agrippeur sur son porteur).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn grab_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    damage_events: Res<FrameEvents<DamageEvent>>,
    mut enemy_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &EnemyAiConfig,
            &EnemyBehaviors,
            &mut BehaviorRuntime,
            &EnemyTarget,
            Option<&mut Riding>,
            Option<&BossState>,
        ),
        With<Enemy>,
    >,
    mut player_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &Player,
            &mut Modifiers,
            Option<&MeleeAttackState>,
            Has<Downed>,
            Has<Death>,
        ),
        (With<Player>, Without<Enemy>),
    >,
) {
    let now = frame.frame;
    // Agrippeurs déjà accrochés, par porteur (`net_id` du joueur).
    let mut riders: BTreeMap<usize, u32> = BTreeMap::new();
    for (_, _, _, _, _, _, _, riding, _) in order_iter!(enemy_query) {
        if let Some(riding) = riding {
            *riders.entry(riding.carrier.0).or_default() += 1;
        }
    }
    let melee_tag = Tag::new("melee");

    for (net_id, entity, transform, ai_config, rules, mut runtime, target, mut riding, boss) in
        order_mut_iter!(enemy_query)
    {
        let phase_rules = rules.active(boss.map_or(0, |b| b.phase));
        let Some(rule) = grab_rule(phase_rules) else {
            continue;
        };
        let selected = runtime
            .selected
            .and_then(|index| phase_rules.get(index as usize))
            .is_some_and(|rule| matches!(rule, Behavior::Grab { .. }));
        let position = transform.translation.truncate();

        // Cible joueur connue, vivante et debout : celle du bond.
        let target_player = match (&target.target, target.target_type) {
            (Some(id), TargetType::Player) => player_query
                .iter()
                .find(|(player_id, ..)| *player_id == id)
                .filter(|(.., downed, dead)| !downed && !dead)
                .map(|(player_id, player_transform, ..)| {
                    (player_id.clone(), player_transform.translation.truncate())
                }),
            _ => None,
        };

        // Une règle plus prioritaire ou une incapacité a pris la main : la phase est abandonnée (un
        // agrippeur accroché se laisse tomber, sans étourdissement).
        if !selected {
            if runtime.grab != GrabPhase::Idle {
                if let Some(riding) = riding.as_deref() {
                    release_slow(&mut player_query, &riding.carrier, net_id.0);
                    commands.entity(entity).remove::<Riding>();
                }
                runtime.grab = GrabPhase::Idle;
                runtime.grab_ready_at = now.saturating_add(rule.cooldown);
                info!("ggrs{{f={} grab net_id={} phase=abort}}", now, net_id);
            }
            continue;
        }

        match runtime.grab.clone() {
            GrabPhase::Idle => {
                if target_player.is_some() {
                    runtime.grab = GrabPhase::Crouch {
                        until: now.saturating_add(rule.telegraph),
                    };
                    info!("ggrs{{f={} grab net_id={} phase=crouch}}", now, net_id);
                }
            }
            GrabPhase::Crouch { until } => {
                if now >= until {
                    match &target_player {
                        Some((_, target_pos)) => {
                            runtime.grab = GrabPhase::Leap {
                                until: now.saturating_add(rule.leap_frames),
                                target: (target_pos.x, target_pos.y),
                            };
                            info!(
                                "ggrs{{f={} grab net_id={} phase=leap target=({},{})}}",
                                now,
                                net_id,
                                target_pos.x.to_num::<i32>(),
                                target_pos.y.to_num::<i32>()
                            );
                        }
                        None => {
                            runtime.grab = GrabPhase::Idle;
                            runtime.grab_ready_at = now.saturating_add(rule.cooldown);
                        }
                    }
                }
            }
            GrabPhase::Leap { until, target: aim } => {
                let contact = target_player
                    .as_ref()
                    .filter(|(_, pos)| position.distance(pos) <= ai_config.attack_range);
                let reached = position.distance(&fixed_math::FixedVec2::new(aim.0, aim.1))
                    < fixed_math::Fixed::from_num(LEAP_ARRIVE);
                let free = contact
                    .filter(|(carrier, _)| riders.get(&carrier.0).copied().unwrap_or(0) < rule.max_riders);
                if let Some((carrier, _)) = free {
                    *riders.entry(carrier.0).or_default() += 1;
                    let until = now.saturating_add(rule.hold);
                    commands
                        .entity(entity)
                        .insert(Riding::new(carrier.clone(), now, until, rule.sway));
                    if let Some((_, _, _, mut modifiers, ..)) = player_query
                        .iter_mut()
                        .find(|(player_id, ..)| player_id.0 == carrier.0)
                    {
                        hold_slow(&mut modifiers, net_id.0, rule.slow, now.saturating_add(1));
                    }
                    runtime.grab = GrabPhase::Ride;
                    info!(
                        "ggrs{{f={} grab net_id={} phase=ride carrier={}}}",
                        now, net_id, carrier
                    );
                } else if contact.is_some() || reached || now >= until {
                    runtime.grab = GrabPhase::Idle;
                    runtime.grab_ready_at = now.saturating_add(rule.cooldown);
                    info!(
                        "ggrs{{f={} grab net_id={} phase=miss contact={}}}",
                        now,
                        net_id,
                        contact.is_some()
                    );
                }
            }
            GrabPhase::Ride => {
                // Posé la frame de l'accroche (commande) : absent seulement si l'accroche a été défaite.
                let Some(riding) = riding.as_deref_mut() else {
                    runtime.grab = GrabPhase::Dazed {
                        until: now.saturating_add(rule.stun),
                    };
                    continue;
                };
                let carrier = player_query
                    .iter()
                    .find(|(player_id, ..)| player_id.0 == riding.carrier.0)
                    .map(|(_, _, player, _, melee, downed, dead)| {
                        (
                            player.handle,
                            melee.and_then(|m| m.attack_started_frame) == Some(now),
                            downed || dead,
                        )
                    });
                let release = match carrier {
                    None | Some((_, _, true)) => Some(Release::Carrier),
                    Some(_) if now >= riding.until => Some(Release::Timeout),
                    Some((_, true, _)) => Some(Release::CarrierMelee),
                    Some((handle, false, false)) => {
                        let struck = damage_events.iter().any(|event| {
                            event.target.0 == net_id.0
                                && event.source_team == Team::Players
                                && event.tags.has(&melee_tag)
                        });
                        let (x, y) = axes(inputs[handle].0.buttons);
                        let shakes = riding.shake(x, y, now, rule.shake_window);
                        if struck {
                            Some(Release::Struck)
                        } else if shakes >= rule.shake {
                            Some(Release::Shaken)
                        } else {
                            None
                        }
                    }
                };
                match release {
                    Some(release) => {
                        let carrier_id = riding.carrier.clone();
                        release_slow(&mut player_query, &carrier_id, net_id.0);
                        commands.entity(entity).remove::<Riding>();
                        runtime.grab = GrabPhase::Dazed {
                            until: now.saturating_add(rule.stun),
                        };
                        info!(
                            "ggrs{{f={} grab net_id={} phase=dazed cause={} carrier={}}}",
                            now,
                            net_id,
                            release.name(),
                            carrier_id
                        );
                    }
                    None => {
                        if let Some((_, _, _, mut modifiers, ..)) = player_query
                            .iter_mut()
                            .find(|(player_id, ..)| player_id.0 == riding.carrier.0)
                        {
                            hold_slow(&mut modifiers, net_id.0, rule.slow, now.saturating_add(1));
                        }
                        let ridden = now.saturating_sub(riding.since);
                        if let Some(every) = rule.bite_every.filter(|every| *every > 0) {
                            if ridden > 0 && ridden % every == 0 {
                                runtime.grab_bite = Some((now, riding.carrier.clone()));
                                info!(
                                    "ggrs{{f={} grab net_id={} bite carrier={}}}",
                                    now, net_id, riding.carrier
                                );
                            }
                        }
                    }
                }
            }
            GrabPhase::Dazed { until } => {
                if now >= until {
                    runtime.grab = GrabPhase::Idle;
                    runtime.grab_ready_at = now.saturating_add(rule.cooldown);
                    info!("ggrs{{f={} grab net_id={} phase=recovered}}", now, net_id);
                }
            }
        }
    }
}

/// Retire le ralentissement de l'agrippeur `rider` de son porteur `carrier`.
#[allow(clippy::type_complexity)]
fn release_slow(
    player_query: &mut Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &Player,
            &mut Modifiers,
            Option<&MeleeAttackState>,
            Has<Downed>,
            Has<Death>,
        ),
        (With<Player>, Without<Enemy>),
    >,
    carrier: &GgrsNetId,
    rider: usize,
) {
    if let Some((_, _, _, mut modifiers, ..)) = player_query
        .iter_mut()
        .find(|(player_id, ..)| player_id.0 == carrier.0)
    {
        modifiers.0.retain(|m| !is_grab_source(&m.source, rider));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slow_is_renewed_in_place_then_released() {
        let mut modifiers = Modifiers::default();
        hold_slow(&mut modifiers, 12, fixed_math::new(0.6), 101);
        hold_slow(&mut modifiers, 12, fixed_math::new(0.6), 102);
        hold_slow(&mut modifiers, 13, fixed_math::new(0.6), 102);
        assert_eq!(modifiers.0.len(), 2);
        assert_eq!(modifiers.0[0].until, Some(102));
        assert!(is_grab_source(&modifiers.0[1].source, 13));
        assert!(!is_grab_source(&modifiers.0[1].source, 1));
        modifiers.0.retain(|m| !is_grab_source(&m.source, 12));
        assert_eq!(modifiers.0.len(), 1);
    }

    #[test]
    fn axes_follow_the_movement_buttons() {
        assert_eq!(axes(INPUT_LEFT), (-1, 0));
        assert_eq!(axes(INPUT_RIGHT | INPUT_UP), (1, 1));
        assert_eq!(axes(INPUT_LEFT | INPUT_RIGHT), (0, 0));
        assert_eq!(axes(INPUT_DOWN), (0, -1));
    }
}
