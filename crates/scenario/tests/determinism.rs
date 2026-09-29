//! Le filet du déterminisme (plan §9.6). En session synctest, GGRS recharge un état, rejoue les
//! dernières frames et compare les checksums des états enregistrés par `rollback_and_trace`.
//! Un état qui n'est pas rollback mais qui influence la simulation diverge à la resimulation :
//! le runner doit le signaler.

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::GgrsSchedule;
use game::{
    character::{health::Health, player::Player},
    system_set::RollbackSystemSet,
};
use scenario::{run_with, Scenario};

fn idle() -> Scenario {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/scenarios/idle.ron");
    let source = std::fs::read_to_string(path).expect("idle.ron");
    Scenario::from_ron(&source).expect("scénario valide")
}

fn failures_synctest(failures: &[String]) -> Vec<&String> {
    failures.iter().filter(|f| f.contains("synctest mismatch")).collect()
}

#[test]
fn synctest_passe_sur_un_scenario_sain() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("test ignoré : compilé avec le rendu des tilemaps (utiliser --no-default-features)");
        return;
    }
    let outcome = run_with(&idle(), |_| {});
    let mismatches = failures_synctest(&outcome.failures);
    assert!(mismatches.is_empty(), "aucun mismatch attendu, trouvé : {mismatches:?}");
}

/// Ressource volontairement absente du rollback : après un rechargement d'état, elle continue
/// de compter au lieu de revenir en arrière. C'est le bug type (état oublié) que le filet doit voir.
#[derive(Resource, Default)]
struct Compteur(u32);

/// Dans la simulation, la santé des joueurs dépend du compteur : à la resimulation, le compteur a
/// d'autres valeurs, la santé diverge, et son checksum (enregistré par `rollback_and_trace`) aussi.
/// La santé est une fonction directe du compteur (pas une condition périodique : le synctest rejoue
/// `check_distance + 1` frames par appel, et une période égale rendrait la divergence invisible).
fn user_du_compteur(mut compteur: ResMut<Compteur>, mut joueurs: Query<&mut Health, With<Player>>) {
    compteur.0 += 1;
    let valeur = fixed_math::Fixed::from_num((compteur.0 % 50) as i32 + 1);
    for mut health in &mut joueurs {
        health.current = valeur;
    }
}

#[test]
fn synctest_detecte_un_etat_hors_rollback() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("test ignoré : compilé avec le rendu des tilemaps (utiliser --no-default-features)");
        return;
    }
    let outcome = run_with(&idle(), |app| {
        app.init_resource::<Compteur>();
        app.add_systems(GgrsSchedule, user_du_compteur.in_set(RollbackSystemSet::Movement));
    });
    let mismatches = failures_synctest(&outcome.failures);
    assert!(
        !mismatches.is_empty(),
        "un mismatch de synctest était attendu ; failures : {:?}",
        outcome.failures
    );
}
