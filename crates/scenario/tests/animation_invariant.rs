//! P-1d (`docs/conventions.md` § 122) : l'invariant `animation_vivante` du runner, éprouvé sur un monde minimal.
//! Rouge : un personnage dont l'état de la simulation change (coup de mêlée) et qui garde le clip de son attente, ou dont le
//! clip ne bouge plus, ou dont la pose n'est pas résolue. Vert : un clip distinct de l'attente dont la position avance, une
//! dernière image tenue exprès (`held`), une marche lue à la vitesse de la pose (`moving`) quelle que soit la vitesse.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedTransform3D};
use combat::weapons::melee::MeleeAttackState;
use game::model3d::playback::ModelPlayback;
use scenario::invariants::{InvariantQueries, Invariants};

const IDLE: &str = "idle";
const STRIKE: &str = "strike";
const WALK: &str = "walk";

/// Une frame du monde minimal : la pose décidée, la position x (unités du monde), le coup de mêlée en cours.
struct Frame {
    playback: ModelPlayback,
    x: f32,
    attacking: bool,
}

/// Joue `frames` frames d'un personnage dont `frame_of(frame)` donne l'état ; rend les violations de `animation_vivante`.
fn run(frames: u32, frame_of: impl Fn(u32) -> Frame) -> Vec<String> {
    let mut world = World::new();
    let entity = world
        .spawn((
            ModelPlayback::playing("modele", IDLE, 2.0, 0.0),
            FixedTransform3D::IDENTITY,
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
        let state = frame_of(frame);
        *world.get_mut::<ModelPlayback>(entity).unwrap() = state.playback;
        world
            .get_mut::<FixedTransform3D>(entity)
            .unwrap()
            .translation
            .x = fixed_math::new(state.x);
        world
            .get_mut::<MeleeAttackState>(entity)
            .unwrap()
            .is_attacking = state.attacking;
        out.extend(queries.check(&mut world, &config, frame));
    }
    out
}

/// Un personnage immobile dont `clip_of(frame)` donne (clip, position) et qui frappe de la frame 10 à la fin.
fn violations(frames: u32, clip_of: impl Fn(u32) -> (&'static str, f32)) -> Vec<String> {
    run(frames, |frame| {
        let (clip, time) = clip_of(frame);
        Frame {
            playback: ModelPlayback::playing("modele", clip, 2.0, time),
            x: 0.0,
            attacking: frame >= 10,
        }
    })
}

/// Le temps d'un clip qui boucle sur 2 s à 60 frames par seconde.
fn looping(frame: u32) -> f32 {
    (frame as f32 / 60.0) % 2.0
}

/// Revue P-1d (1) : la dernière image du tir tenue exprès jusqu'à la fin de la séquence (`held`) n'est pas un lecteur bloqué.
#[test]
fn a_last_frame_held_on_purpose_is_not_a_stuck_clip() {
    let out = run(80, |f| {
        let mut playback = if f < 10 {
            ModelPlayback::playing("modele", IDLE, 2.0, looping(f))
        } else {
            ModelPlayback::playing("modele", STRIKE, 0.5, 0.4995)
        };
        playback.held = f >= 10;
        Frame {
            playback,
            x: 0.0,
            attacking: f >= 10,
        }
    });
    assert!(out.is_empty(), "{out:#?}");
}

/// Revue P-1d (2) : une marche lente (0,1 unité par frame) puis rapide joue le même clip de marche ; la marche est celle que la
/// pose a lue (`moving`), pas un second seuil sur la position, qui apprenait la marche lente comme l'attente.
#[test]
fn a_walk_slower_than_a_position_threshold_is_still_a_walk() {
    let out = run(60, |f| {
        let mut playback = ModelPlayback::playing("modele", WALK, 1.0, looping(f) % 1.0);
        playback.moving = true;
        let x = if f <= 30 {
            0.1 * f as f32
        } else {
            3.0 + 0.3 * (f - 30) as f32
        };
        Frame {
            playback,
            x,
            attacking: false,
        }
    });
    assert!(out.is_empty(), "{out:#?}");
}

#[test]
fn a_character_that_strikes_with_a_strike_clip_that_advances_passes() {
    let out = violations(60, |f| {
        if f < 10 {
            (IDLE, looping(f))
        } else {
            (STRIKE, looping(f))
        }
    });
    assert!(out.is_empty(), "{out:#?}");
}

/// Le défaut des fantassins de 1837 : la simulation change d'état, le clip reste celui de l'attente.
#[test]
fn a_state_change_that_keeps_the_idle_clip_fails() {
    let out = violations(60, |f| (IDLE, looping(f)));
    assert_eq!(out.len(), 1, "{out:#?}");
    assert!(
        out[0].contains("animation_vivante")
            && out[0].contains("coup de mêlée")
            && out[0].contains("celui de l'attente"),
        "{out:#?}"
    );
}

#[test]
fn a_clip_that_stops_advancing_fails() {
    let out = violations(80, |f| {
        if f < 10 {
            (IDLE, looping(f))
        } else {
            (STRIKE, 0.5)
        }
    });
    assert_eq!(out.len(), 1, "{out:#?}");
    assert!(out[0].contains("figé"), "{out:#?}");
}
