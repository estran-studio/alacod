//! M3-T2 (`docs/conventions.md` §67) : `HitLanded` dans le vrai résolveur de dégâts
//! (`rollback_resolve_damage_events`), sur la cible `iframes_target` du testbed (5000 PV, 30 frames
//! d'invulnérabilité après un dégât réel). Les `DamageEvent` sont injectés à des frames exactes.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use game::effects_v3::HitLanded;
use scenario::Scenario;
use sim_core::damage::{DamageEvent, DamageKind, FriendlyFire};
use sim_core::frame_events::FrameEvents;
use utils::net_id::GgrsNetId;

const TARGET_NET_ID: usize = 32;
const OTHER_NET_ID: usize = 9999;

/// Coups portés `(frame, source, cible)` observés après résolution, pour des dégâts injectés
/// `(frame, source, montant)` sur la cible.
fn hits_landed(damage: &[(u32, usize, f32)], frames: u32) -> Vec<(u32, usize, usize)> {
    if map_ldtk::RENDER_ENABLED {
        return Vec::new();
    }
    let mut scenario = Scenario::from_ron(include_str!("../../../tests/scenarios/hit_iframes.ron"))
        .expect("scénario hit_iframes");
    scenario.players[0].inputs.clear();
    scenario.frames = frames;
    scenario.expect.clear();
    let damage = damage.to_vec();
    // synctest : une frame est simulée plusieurs fois ; un coup se compte une fois (ensemble).
    let seen = Arc::new(Mutex::new(std::collections::BTreeSet::new()));
    let sink = seen.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (
                (move |frame: Res<utils::frame::FrameCount>,
                       targets: Query<&GgrsNetId, With<game::character::enemy::Enemy>>,
                       mut events: ResMut<FrameEvents<DamageEvent>>| {
                    if !targets.iter().any(|id| id.0 == TARGET_NET_ID) {
                        return;
                    }
                    for (at, source, amount) in &damage {
                        if frame.frame == *at {
                            events.send(DamageEvent {
                                source: GgrsNetId(*source, "t".into()),
                                target: GgrsNetId(TARGET_NET_ID, "t".into()),
                                kind: DamageKind::Physical,
                                amount: Fixed::from_num(*amount),
                                frame: frame.frame,
                                tags: Default::default(),
                                source_team: sim_core::team::Team::Players,
                                friendly_fire: FriendlyFire::Never,
                            });
                        }
                    }
                })
                .after(game::character::enemy::ai::rules::charge_damage_translate_system)
                .after(game::bombs::explosion_system)
                .before(game::character::health::rollback_resolve_damage_events)
                .in_set(game::system_set::RollbackSystemSet::CollisionDamage),
                (move |landed: Res<FrameEvents<HitLanded>>| {
                    for hit in landed.iter() {
                        sink.lock()
                            .unwrap()
                            .insert((hit.frame, hit.source, hit.target));
                    }
                })
                .after(game::character::health::rollback_resolve_damage_events)
                .in_set(game::system_set::RollbackSystemSet::CollisionDamage),
            ),
        );
    });
    let value: Vec<_> = seen.lock().unwrap().iter().copied().collect();
    value
}

#[test]
fn a_real_hit_from_another_entity_lands() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    assert_eq!(
        hits_landed(&[(12, OTHER_NET_ID, 8.0)], 20),
        vec![(12, OTHER_NET_ID, TARGET_NET_ID)]
    );
}

#[test]
fn zero_damage_is_not_a_landed_hit() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    assert!(hits_landed(&[(12, OTHER_NET_ID, 0.0)], 20).is_empty());
}

#[test]
fn self_inflicted_damage_is_not_a_landed_hit() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    assert!(hits_landed(&[(12, TARGET_NET_ID, 8.0)], 20).is_empty());
}

#[test]
fn a_hit_dropped_by_the_iframes_is_not_a_landed_hit() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // f22 tombe dans les 30 frames d'invulnérabilité du coup de f12 : seul f12 porte ; f50 porte.
    assert_eq!(
        hits_landed(
            &[
                (12, OTHER_NET_ID, 8.0),
                (22, OTHER_NET_ID, 8.0),
                (50, OTHER_NET_ID, 8.0)
            ],
            60
        ),
        vec![
            (12, OTHER_NET_ID, TARGET_NET_ID),
            (50, OTHER_NET_ID, TARGET_NET_ID)
        ]
    );
}
