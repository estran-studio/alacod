//! M3-T9 (`docs/conventions.md` §74) : les curses d'étage tirées et exécutées en jeu.
//!
//! - la curse ne dépend que de la graine et de l'étage : même graine, même curse et même trace ;
//!   autre graine, autre tirage ;
//! - le modificateur d'une curse change réellement le déplacement du joueur (pas seulement la stat
//!   résolue que lit l'attente `Stat`) ;
//! - une séquence sans champ `curses` (`deux_niveaux`) ne tire rien : `RunCurses` reste neutre.

use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use game::{
    character::player::Player,
    replay::{Button, Expectation, Segment},
};
use run::RunCurses;
use scenario::{runner::run_with, Scenario};
use utils::frame::FrameCount;

fn load(name: &str) -> Scenario {
    let path = format!(
        "{}/../../tests/scenarios/{name}.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path} : {e}"));
    Scenario::from_ron(&source).expect("scénario valide")
}

fn curse(floor: u32, id: Option<&str>, at_frame: u32) -> Expectation {
    Expectation::Curse {
        floor,
        id: id.map(str::to_string),
        at_frame,
    }
}

#[test]
fn same_seed_same_curse_other_seed_other_curse() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let base = load("curse_drawn_seeded");
    let first = scenario::run(&base);
    let second = scenario::run(&base);
    assert!(first.failures.is_empty(), "{:?}", first.failures);
    assert_eq!(first.trace, second.trace, "même graine : même trace");
    assert_eq!(
        first
            .events
            .iter()
            .filter(|event| event.kind == "curse")
            .map(|event| event.label.as_str())
            .collect::<Vec<_>>(),
        ["étage 1 : curse « darkness »"],
        "le tirage de l'étage 0 est un moment clé, celui sans curse de l'étage 1 n'en est pas un"
    );

    let mut other = base.clone();
    other.map_seed = 2;
    other.expect = vec![
        curse(0, Some("labyrinth"), 5),
        curse(1, Some("darkness"), 240),
    ];
    let other = scenario::run(&other);
    assert!(other.failures.is_empty(), "{:?}", other.failures);
    assert_ne!(first.trace, other.trace, "autre graine : autre tirage");

    // Une graine sans aucune curse (graine 1 : ni à l'étage 0 ni à l'étage 1).
    let mut none = base.clone();
    none.map_seed = 1;
    none.expect = vec![curse(0, None, 5), curse(1, None, 240)];
    let none = scenario::run(&none);
    assert!(none.failures.is_empty(), "{:?}", none.failures);
    assert!(none.events.iter().all(|event| event.kind != "curse"));
}

/// Abscisse du joueur 0 à la frame `at`, lue pendant la partie (bits `f32` dans un atomique).
fn x_at(scenario: &Scenario, at: u32) -> f32 {
    let seen = Arc::new(AtomicU32::new(f32::NAN.to_bits()));
    let sink = Arc::clone(&seen);
    let outcome = run_with(scenario, move |app| {
        app.add_systems(
            Last,
            move |frame: Res<FrameCount>, players: Query<&FixedTransform3D, With<Player>>| {
                if frame.frame == at {
                    if let Some(transform) = players.iter().next() {
                        let x = transform.translation.x.to_num::<f32>();
                        sink.store(x.to_bits(), Ordering::Relaxed);
                    }
                }
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let x = f32::from_bits(seen.load(Ordering::Relaxed));
    assert!(!x.is_nan(), "joueur absent à la frame {at}");
    x
}

#[test]
fn a_modifier_curse_really_slows_the_player() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // Le joueur marche à droite de f10 à f50 sans tirer. Graine 3 : `maudit_maze` tire « maze » à
    // l'étage 0 ; le témoin joue la même graine (donc la même carte et le même point de départ) sur
    // `deux_niveaux`, sans champ `curses`.
    let walk = |floors: &str| {
        let mut scenario = load("curse_modifier");
        scenario.floors = Some(floors.into());
        scenario.frames = 80;
        scenario.players[0].inputs = vec![Segment {
            from: 10,
            to: 50,
            buttons: vec![Button::Right],
            pan: (0, 0),
        }];
        scenario.expect.clear();
        scenario
    };
    let cursed = walk("maudit_maze");
    let free = walk("deux_niveaux");
    let (start_cursed, end_cursed) = (x_at(&cursed, 8), x_at(&cursed, 60));
    let (start_free, end_free) = (x_at(&free, 8), x_at(&free, 60));
    assert_eq!(start_cursed, start_free, "même point de départ");
    let (slow, fast) = (end_cursed - start_cursed, end_free - start_free);
    assert!(fast > 20.0, "le joueur libre a avancé de {fast}");
    let ratio = slow / fast;
    // Le joueur accélère (`MovementConfig::acceleration`) : le rapport des distances n'est pas
    // exactement 0,85 mais reste nettement sous 1.
    assert!(
        (0.70..0.95).contains(&ratio),
        "distance sous maze / libre = {ratio} ({slow} / {fast})"
    );
}

#[test]
fn a_sequence_without_curses_draws_nothing_and_leaves_the_state_neutral() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let neutral = Arc::new(AtomicBool::new(true));
    let sink = Arc::clone(&neutral);
    let mut scenario = load("curse_drawn_seeded");
    scenario.floors = Some("deux_niveaux".into());
    scenario.expect.clear();
    let outcome = run_with(&scenario, move |app| {
        app.add_systems(Last, move |curses: Res<RunCurses>| {
            if *curses != RunCurses::default() {
                sink.store(false, Ordering::Relaxed);
            }
        });
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert!(
        neutral.load(Ordering::Relaxed),
        "aucune entrée ni mémoire d'étage : la ressource reste à sa valeur neutre à chaque frame"
    );
    assert!(outcome.events.iter().all(|event| event.kind != "curse"));
}
