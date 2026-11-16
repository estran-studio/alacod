//! P-1d (`docs/conventions.md` § 122) : l'invariant `animation_vivante` du runner, éprouvé sur un monde minimal.
//! Rouge : un personnage dont l'état de la simulation change (coup de mêlée) et qui garde le clip de son attente, ou dont le
//! clip ne bouge plus, ou dont la pose n'est pas résolue. Vert : un clip distinct de l'attente dont la position avance.

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::weapons::melee::MeleeAttackState;
use game::model3d::playback::ModelPlayback;
use scenario::invariants::{InvariantQueries, Invariants};

const IDLE: &str = "idle";
const STRIKE: &str = "strike";

/// Joue `frames` frames d'un personnage dont `clip_of(frame)` donne (clip, position) et qui frappe de la frame 10 à la fin ;
/// rend les violations de `animation_vivante`.
fn violations(frames: u32, clip_of: impl Fn(u32) -> (&'static str, f32)) -> Vec<String> {
    let mut world = World::new();
    let entity = world
        .spawn((
            ModelPlayback::playing("modele", IDLE, 2.0, 0.0),
            FixedTransform3D::default(),
            MeleeAttackState::default(),
        ))
        .id();
    let mut queries = InvariantQueries::new(&mut world);
    let config = Invariants {
        sante_bornee: false,
        net_ids_uniques: false,
        joueur_hors_mur: false,
        animation_vivante: true,
    };
    let mut out = Vec::new();
    for frame in 1..=frames {
        let (clip, time) = clip_of(frame);
        *world.get_mut::<ModelPlayback>(entity).unwrap() = ModelPlayback::playing("modele", clip, 2.0, time);
        world.get_mut::<MeleeAttackState>(entity).unwrap().is_attacking = frame >= 10;
        out.extend(queries.check(&mut world, &config, frame));
    }
    out
}

/// Le temps d'un clip qui boucle sur 2 s à 60 frames par seconde.
fn looping(frame: u32) -> f32 {
    (frame as f32 / 60.0) % 2.0
}

#[test]
fn a_character_that_strikes_with_a_strike_clip_that_advances_passes() {
    let out = violations(60, |f| if f < 10 { (IDLE, looping(f)) } else { (STRIKE, looping(f)) });
    assert!(out.is_empty(), "{out:#?}");
}

/// Le défaut des fantassins de 1837 : la simulation change d'état, le clip reste celui de l'attente.
#[test]
fn a_state_change_that_keeps_the_idle_clip_fails() {
    let out = violations(60, |f| (IDLE, looping(f)));
    assert_eq!(out.len(), 1, "{out:#?}");
    assert!(
        out[0].contains("animation_vivante") && out[0].contains("coup de mêlée") && out[0].contains("celui de l'attente"),
        "{out:#?}"
    );
}

#[test]
fn a_clip_that_stops_advancing_fails() {
    let out = violations(80, |f| if f < 10 { (IDLE, looping(f)) } else { (STRIKE, 0.5) });
    assert_eq!(out.len(), 1, "{out:#?}");
    assert!(out[0].contains("figé"), "{out:#?}");
}
