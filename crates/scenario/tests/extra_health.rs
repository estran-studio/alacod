//! M3-T3 (`docs/conventions.md` §68) : l'ordre des couches de défense dans le vrai résolveur
//! (`rollback_resolve_damage_events`) : **bouclier → armure → `ExtraHealth` → PV**, sur la cible
//! `iframes_target` du testbed (5000 PV, 30 frames d'invulnérabilité après un dégât réel). Les
//! `DamageEvent` sont injectés à des frames exactes ; aucun tir du joueur. Le bouclier, l'armure
//! (+100 : moitié des dégâts) et les points de vie d'âme sont posés à la frame 6 (la cible apparaît à f5).

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use combat::actors::{ExtraHealth, Shield};
use game::replay::Expectation;
use scenario::Scenario;
use sim_core::damage::{DamageEvent, DamageKind, FriendlyFire};
use sim_core::modifier::{ModifierOp, ModifierSource, Modifiers};
use sim_core::stats::StatId;

const TARGET_NET_ID: usize = 32;

#[derive(Clone, Copy)]
struct Setup {
    shield: u32,
    armor: bool,
    soul: f32,
}

/// `(PV, points de vie d'âme, bouclier)` de la cible à la fin de `frames`, après avoir injecté
/// `hits` (`(frame, dégât, genre)`).
fn target_state(hits: &[(u32, f32, DamageKind)], setup: Setup, frames: u32) -> (f32, f32, u32) {
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
    let seen = std::sync::Arc::new(std::sync::Mutex::new((0.0f32, 0.0f32, 0u32)));
    let sink = seen.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |frame: Res<utils::frame::FrameCount>,
                   mut commands: Commands,
                   mut targets: Query<
                (
                    Entity,
                    &utils::net_id::GgrsNetId,
                    &game::character::health::Health,
                    &mut Modifiers,
                    Option<&ExtraHealth>,
                    Option<&Shield>,
                ),
                With<game::character::enemy::Enemy>,
            >,
                   mut damage: ResMut<sim_core::frame_events::FrameEvents<DamageEvent>>| {
                let Some((entity, id, health, mut modifiers, extra, shield)) = targets
                    .iter_mut()
                    .find(|(_, id, ..)| id.0 == TARGET_NET_ID)
                else {
                    return;
                };
                // Dernière valeur observée (début de la dernière frame simulée).
                *sink.lock().unwrap() = (
                    health.current.to_num::<f32>(),
                    extra.map_or(0.0, |extra| extra.soul.to_num::<f32>()),
                    shield.map_or(0, |shield| shield.0),
                );
                if frame.frame == 6 {
                    if setup.shield > 0 {
                        commands.entity(entity).insert(Shield(setup.shield));
                    }
                    if setup.soul > 0.0 {
                        commands.entity(entity).insert(ExtraHealth {
                            soul: Fixed::from_num(setup.soul),
                        });
                    }
                    if setup.armor {
                        modifiers.push_from(
                            ModifierSource::Named("test".into()),
                            StatId::Armor,
                            ModifierOp::Add,
                            Fixed::from_num(100),
                            None,
                        );
                    }
                }
                for (at, amount, kind) in &hits {
                    if frame.frame == *at {
                        damage.send(DamageEvent {
                            source: id.clone(),
                            target: id.clone(),
                            kind: kind.clone(),
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
    let value = *seen.lock().unwrap();
    value
}

const NONE: Setup = Setup {
    shield: 0,
    armor: false,
    soul: 0.0,
};

#[test]
fn soul_takes_the_damage_point_by_point_before_health() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let phys = DamageKind::Physical;
    let setup = Setup { soul: 6.0, ..NONE };
    // Un coup de 8 : 6 points d'âme, 2 PV. Un second coup de 8 (hors i-frames) : 8 PV, plus d'âme.
    assert_eq!(
        target_state(&[(12, 8.0, phys.clone())], setup, 20),
        (4998.0, 0.0, 0)
    );
    assert_eq!(
        target_state(&[(12, 8.0, phys.clone()), (50, 8.0, phys)], setup, 60),
        (4990.0, 0.0, 0)
    );
}

#[test]
fn layers_apply_in_the_order_shield_armor_soul_health() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let phys = DamageKind::Physical;
    let setup = Setup {
        shield: 1,
        armor: true,
        soul: 6.0,
    };
    // f12 : le bouclier prend le coup entier (ni armure ni âme ne bougent).
    assert_eq!(
        target_state(&[(12, 8.0, phys.clone())], setup, 20),
        (5000.0, 6.0, 0)
    );
    // f50 (après les 15 frames du bouclier) : l'armure divise par 2 (4), l'âme prend 4 (reste 2).
    assert_eq!(
        target_state(
            &[(12, 8.0, phys.clone()), (50, 8.0, phys.clone())],
            setup,
            60
        ),
        (5000.0, 2.0, 0)
    );
    // f90 : 4 après armure, 2 points d'âme puis 2 PV.
    assert_eq!(
        target_state(
            &[
                (12, 8.0, phys.clone()),
                (50, 8.0, phys.clone()),
                (90, 8.0, phys)
            ],
            setup,
            100
        ),
        (4998.0, 0.0, 0)
    );
}

#[test]
fn a_hit_absorbed_by_soul_still_starts_the_hit_iframes() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let phys = DamageKind::Physical;
    let setup = Setup { soul: 10.0, ..NONE };
    // Les deux premiers coups sont à 10 frames d'écart : le second tombe dans les 30 frames
    // d'invulnérabilité du premier (absorbé par l'âme, mais c'était un coup) et est ignoré.
    // Le troisième (f50) passe : 10 - 3 - 3 = 4.
    assert_eq!(
        target_state(
            &[
                (12, 3.0, phys.clone()),
                (22, 3.0, phys.clone()),
                (50, 3.0, phys)
            ],
            setup,
            60
        ),
        (5000.0, 4.0, 0)
    );
}

/// Remplace le test de M3-T0b (`hit_iframes.rs`), vacuous (bouclier posé avant l'apparition de la cible).
/// Un coup absorbé par le bouclier ne pose **pas** les `hit_iframes` (60 frames pour `isaac`, 30 pour la
/// cible) : il pose seulement les brèves i-frames du bouclier (`SHIELD_IFRAMES`, 15 frames). Le bouclier
/// (1 point, posé à f6) absorbe le coup de f12 ; le coup de f30, 18 frames plus tard, passe. Si le coup
/// absorbé posait les 30 frames de `hit_iframes` (jusqu'à f42), le coup de f30 serait ignoré : 5000.
#[test]
fn a_hit_absorbed_by_the_shield_does_not_start_the_hit_iframes() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let phys = DamageKind::Physical;
    let setup = Setup { shield: 1, ..NONE };
    assert_eq!(
        target_state(&[(12, 8.0, phys.clone()), (30, 8.0, phys)], setup, 40),
        (4992.0, 0.0, 0)
    );
}

#[test]
fn true_damage_skips_shield_and_armor_but_not_soul() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let setup = Setup {
        shield: 1,
        armor: true,
        soul: 3.0,
    };
    // 8 de dégâts vrais : le bouclier (toujours là) et l'armure sont ignorés, l'âme prend 3, les PV 5.
    assert_eq!(
        target_state(&[(12, 8.0, DamageKind::True)], setup, 20),
        (4995.0, 0.0, 1)
    );
}

/// Santé du joueur 0 de `hit_iframes_isaac` à la frame 130, avec ou sans la donnée `hit_iframes`
/// de `pleureur` (effacée dans les assets avant le premier coup si `strip` est vrai).
fn isaac_health_at_130(strip: bool) -> f32 {
    let mut scenario = Scenario::from_ron(include_str!(
        "../../../tests/scenarios/hit_iframes_isaac.ron"
    ))
    .expect("scénario hit_iframes_isaac");
    scenario.expect = vec![Expectation::PlayerAlive {
        handle: 0,
        at_frame: 130,
    }];
    scenario.frames = 131;
    let seen = std::sync::Arc::new(std::sync::Mutex::new(f32::NAN));
    let sink = seen.clone();
    scenario::runner::run_with(&scenario, move |app| {
        app.add_systems(
            bevy_ggrs::GgrsSchedule,
            (move |frame: Res<utils::frame::FrameCount>,
                   mut configs: ResMut<game::character::config::CharacterConfigs>,
                   players: Query<(
                &game::character::player::Player,
                &game::character::health::Health,
            )>| {
                if strip && frame.frame < 2 {
                    for config in configs.0.values_mut() {
                        config.hit_iframes = None;
                    }
                }
                if frame.frame == 130 {
                    if let Some((_, health)) = players.iter().find(|(player, _)| player.handle == 0)
                    {
                        *sink.lock().unwrap() = health.current.to_num::<f32>();
                    }
                }
            })
            .in_set(game::system_set::RollbackSystemSet::CollisionDamage)
            .before(game::character::health::rollback_resolve_damage_events),
        );
    });
    let value = *seen.lock().unwrap();
    value
}

#[test]
fn isaac_without_hit_iframes_loses_more() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // Avec la donnée : 4 PV à f130 (un des deux coups quasi simultanés est absorbé) ; sans : 3.
    assert_eq!(isaac_health_at_130(false), 4.0);
    assert_eq!(isaac_health_at_130(true), 3.0);
}
