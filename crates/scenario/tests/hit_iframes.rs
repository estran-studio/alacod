//! M3-T0b (`docs/conventions.md` §61) : `CharacterConfig.hit_iframes` dans le vrai résolveur
//! (`rollback_resolve_damage_events`), sur la cible `iframes_target` du testbed (5000 PV, 30
//! frames d'invulnérabilité après un dégât réel). Les `DamageEvent` sont injectés à des frames
//! exactes ; aucun tir du joueur.

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use game::replay::Expectation;
use scenario::Scenario;
use sim_core::damage::{DamageEvent, DamageKind, FriendlyFire};

const TARGET_NET_ID: usize = 32;

/// Vie de la cible à `frame` après avoir injecté `hits` (`(frame, dégât)`).
fn target_health(hits: &[(u32, f32)], frames: u32) -> f32 {
    if map_ldtk::RENDER_ENABLED {
        return f32::NAN;
    }
    let mut scenario = Scenario::from_ron(include_str!("../../../tests/scenarios/hit_iframes.ron"))
        .expect("scénario hit_iframes");
    scenario.players[0].inputs.clear();
    scenario.frames = frames;
    scenario.expect = vec![Expectation::EntityHealth {
        net_id: TARGET_NET_ID,
        min: None,
        max: None,
        at_frame: frames,
    }];
    let hits = hits.to_vec();
    let app_health = std::sync::Arc::new(std::sync::Mutex::new(0.0f32));
    let sink = app_health.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |frame: Res<utils::frame::FrameCount>,
                   targets: Query<
                (&utils::net_id::GgrsNetId, &game::character::health::Health),
                With<game::character::enemy::Enemy>,
            >,
                   mut damage: ResMut<sim_core::frame_events::FrameEvents<DamageEvent>>| {
                let Some((id, health)) = targets.iter().find(|(id, _)| id.0 == TARGET_NET_ID) else {
                    return;
                };
                // Dernière valeur observée (début de la dernière frame simulée).
                *sink.lock().unwrap() = health.current.to_num::<f32>();
                for (at, amount) in &hits {
                    if frame.frame == *at {
                        damage.send(DamageEvent {
                            source: id.clone(),
                            target: id.clone(),
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
        );
    });
    let value = *app_health.lock().unwrap();
    value
}

#[test]
fn second_hit_inside_the_iframes_is_dropped() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // Deux coups de 8 à 10 frames d'écart : un seul est appliqué (5000 - 8) ; un coup après
    // l'invulnérabilité (30 frames) l'est de nouveau.
    assert_eq!(target_health(&[(12, 8.0), (22, 8.0)], 40), 4992.0);
    assert_eq!(
        target_health(&[(12, 8.0), (22, 8.0), (50, 8.0)], 60),
        4984.0
    );
}

#[test]
fn zero_damage_does_not_start_the_iframes() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // Un coup sans dégât final ne pose rien : le coup suivant passe.
    assert_eq!(target_health(&[(12, 0.0), (14, 8.0)], 30), 4992.0);
}
