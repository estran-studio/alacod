//! Tests d'observabilité des divergences de déterminisme en synctest GGRS.
//!
//! - `synctest_passe_sur_un_scenario_sain` : vérifie qu'un scénario propre ne génère
//!   aucun mismatch de synctest.
//! - `synctest_detecte_une_mutation_hors_simulation` : injecte une mutation délibérée
//!   hors du schedule GGRS et vérifie qu'un mismatch est détecté.

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use game::character::{health::Health, player::Player};
use scenario::{run_with, Scenario};

fn scenarios_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/scenarios"))
}

#[test]
fn synctest_passe_sur_un_scenario_sain() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("test déterminisme ignoré : compilés avec le rendu");
        return;
    }

    let path = scenarios_dir().join("idle.ron");
    let source = std::fs::read_to_string(path).expect("idle.ron");
    let scenario = Scenario::from_ron(&source).expect("scenario valid");

    let outcome = run_with(&scenario, |_| {});

    // Vérifie qu'aucune failure ne contient "synctest"
    let synctest_failures: Vec<_> = outcome
        .failures
        .iter()
        .filter(|f| f.contains("synctest"))
        .collect();

    assert!(
        synctest_failures.is_empty(),
        "Aucun mismatch de synctest attendu, trouvé : {:?}",
        synctest_failures
    );
}

#[test]
fn synctest_detecte_une_mutation_hors_simulation() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("test déterminisme ignoré : compilés avec le rendu");
        return;
    }

    let path = scenarios_dir().join("idle.ron");
    let source = std::fs::read_to_string(path).expect("idle.ron");
    let scenario = Scenario::from_ron(&source).expect("scenario valid");

    // Applique une mutation délibérée hors du schedule GGRS (dans Update ou Last).
    // Cette mutation doit être détectée comme un desync lors de la resimulation en synctest.
    let outcome = run_with(&scenario, |app| {
        app.add_systems(Update, mutate_player_health);
    });

    // Vérifie qu'au moins une failure contient "synctest mismatch"
    let synctest_failures: Vec<_> = outcome
        .failures
        .iter()
        .filter(|f| f.contains("synctest mismatch"))
        .collect();

    assert!(
        !synctest_failures.is_empty(),
        "Attendu au moins un mismatch de synctest, aucun trouvé. Failures : {:?}",
        outcome.failures
    );
}

/// Système qui mutate la santé d'un joueur à chaque update (hors GgrsSchedule).
/// Cette mutation devrait être détectée comme un desync lors de la resimulation en synctest.
fn mutate_player_health(
    mut query: Query<&mut Health, With<Player>>,
) {
    if let Some(mut health) = query.iter_mut().next() {
        // Réduit la santé de 1 unité à chaque frame (hors simulation rollback)
        health.current -= fixed_math::Fixed::from_num(1);
    }
}
