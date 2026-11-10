//! X-6b (`docs/conventions.md` §119) : option `zigzag` de `Charge` et behavior `ReturnHome`, mesurés image
//! par image sur les scénarios `zigzag_charge_retour` et `zigzag_temoin_droit` (testbed, joueur 0 immobile
//! en (424, 696), rôdeur placé à son point d'apparition (542, 696)).

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use map_ldtk::game::profile::ScriptedPlacement;
use scenario::Scenario;
use sim_core::animation_state::FacingDirection;

const SPAWN: (f32, f32) = (542.0, 696.0);
const AXIS_Y: f32 = 696.0;

/// Position et orientation du rôdeur placé (`Placed(0)`) à chaque frame de `frames`.
fn track(ron: &str, frames: u32) -> Vec<(u32, f32, f32, FacingDirection)> {
    let mut scenario = Scenario::from_ron(ron).expect("scénario");
    scenario.expect.clear();
    let mut samples = Vec::new();
    scenario::runner::run_visiting(&scenario, frames, |world, frame| {
        let mut query = world.query::<(&ScriptedPlacement, &FixedTransform3D, &FacingDirection)>();
        if let Some((_, transform, facing)) = query.iter(world).find(|(p, ..)| p.0 == 0) {
            samples.push((
                frame,
                transform.translation.x.to_num::<f32>(),
                transform.translation.y.to_num::<f32>(),
                *facing,
            ));
        }
    });
    samples
}

fn zigzag() -> Vec<(u32, f32, f32, FacingDirection)> {
    track(
        include_str!("../../../tests/scenarios/zigzag_charge_retour.ron"),
        200,
    )
}

/// Écart latéral maximal pendant la première ruée (f22 à f48) : `> 0` avec l'option, nul sans.
#[test]
fn zigzag_deviates_sideways_and_the_straight_charge_does_not() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let deviation = |samples: &[(u32, f32, f32, FacingDirection)]| {
        samples
            .iter()
            .filter(|(frame, ..)| (22..=48).contains(frame))
            .map(|(_, _, y, _)| (y - AXIS_Y).abs())
            .fold(0.0f32, f32::max)
    };
    let with = deviation(&zigzag());
    let without = deviation(&track(
        include_str!("../../../tests/scenarios/zigzag_temoin_droit.ron"),
        60,
    ));
    assert!(with > 6.0, "écart latéral avec zigzag : {with}");
    assert!(without < 0.01, "écart latéral sans zigzag : {without}");
    // L'onde change de signe (amplitude nominale 10, période 24).
    let samples = zigzag();
    let signs: Vec<bool> = samples
        .iter()
        .filter(|(frame, ..)| (22..=48).contains(frame))
        .filter(|(_, _, y, _)| (y - AXIS_Y).abs() > 3.0)
        .map(|(_, _, y, _)| *y > AXIS_Y)
        .collect();
    assert!(signs.contains(&true) && signs.contains(&false), "{signs:?}");
}

/// Après la morsure : il recule en regardant la cible (le joueur est à l'ouest, il va vers l'est et
/// regarde à l'ouest), s'arrête à moins de 4 unités de son point d'apparition, y attend sans bouger
/// puis repart charger.
#[test]
fn retreat_faces_the_target_stops_at_the_spawn_point_then_recharges() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let samples = zigzag();
    let at = |frame: u32| *samples.iter().find(|(f, ..)| *f == frame).expect("frame");
    // Recul : x croît, orientation vers le joueur (ouest) et non le sens du déplacement.
    for frame in 55..85 {
        let (_, x, _, facing) = at(frame);
        let (_, previous, ..) = at(frame - 1);
        assert!(
            x > previous,
            "f{frame} : le rôdeur recule (x {previous} → {x})"
        );
        assert_eq!(facing, FacingDirection::Left, "f{frame}");
    }
    // Arrivée à moins d'une case (4 unités) du point d'apparition, puis immobile pendant l'attente.
    let (_, x, y, _) = at(95);
    assert!(
        (x - SPAWN.0).hypot(y - SPAWN.1) < 4.0,
        "arrivée en ({x}, {y})"
    );
    for frame in 95..=125 {
        let (_, wx, wy, _) = at(frame);
        assert_eq!((wx, wy), (x, y), "f{frame} : attente immobile");
    }
    // L'attente finie, la charge repart (le télégraphe, immobile, puis la ruée vers l'ouest).
    let (_, rush_x, ..) = at(160);
    assert!(rush_x < SPAWN.0 - 20.0, "seconde ruée : x = {rush_x}");
}
