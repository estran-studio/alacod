//! X-1 (`docs/conventions.md` §89) : où un plugin de jeu enregistre ses types rollback, et ce que l'ordre change.
//!
//! `CoreSimPlugin` appelle `GamePlugin::build` **avant** les cinq enregistrements de clôture ; un plugin
//! ajouté après `CoreSetupPlugin` (ici : le `configure` de `run_with`) enregistre **après** eux. Ce test
//! joue le même scénario des deux façons et mesure :
//!
//! - la **trace d'état** (checksum GGRS, synctest à distance 2 : rollbacks réels) est **identique** : les parts
//!   du checksum sont combinées par XOR, l'ordre d'enregistrement n'y entre pas ;
//! - la **liste des types** (`TracedTypes`, ce que compare `make schedule_order_diff`) n'est pas la même : c'est le
//!   seul effet de l'ordre, et la raison de la règle de clôture (un type de plus en queue de liste d'un clone
//!   déplacerait la liste de référence).
//!
//! Un seul plugin installé par processus (`game::extension::install` est global) : ce fichier est son propre
//! processus de test.

use std::sync::Arc;

use bevy::prelude::*;
use game::{
    character::player::Player, extension::GamePlugin, replay::Scenario,
    system_set::RollbackSystemSet,
};
use utils::{
    net_id::GgrsNetId,
    order_mut_iter,
    rollback::{RollbackTraceApp, TracedTypes},
};

/// Compteur d'images posé sur chaque joueur : hash **actif** (un type neutre ne montrerait rien).
#[derive(Component, Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
struct Beat(u32);

fn beat_system(
    mut commands: Commands,
    mut players: Query<(&GgrsNetId, Entity, Option<&mut Beat>), With<Player>>,
) {
    for (_, entity, beat) in order_mut_iter!(players) {
        match beat {
            Some(mut beat) => beat.0 += 1,
            None => {
                commands.entity(entity).insert(Beat(1));
            }
        }
    }
}

fn register(app: &mut App) {
    app.rollback_and_trace::<Beat>().add_systems(
        bevy_ggrs::GgrsSchedule,
        beat_system.in_set(RollbackSystemSet::Run),
    );
}

struct Marker;

impl GamePlugin for Marker {
    fn build(&self, app: &mut App) {
        register(app);
    }
}

fn scenario() -> Scenario {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/ammo_burst.ron"
    ))
    .unwrap();
    let mut scenario = Scenario::from_ron(&source).unwrap();
    scenario.frames = 120;
    scenario.expect.clear();
    scenario
}

fn listed(app: &App) -> Vec<String> {
    app.world()
        .resource::<TracedTypes>()
        .0
        .iter()
        .map(|name| name.rsplit("::").next().unwrap().to_string())
        .collect()
}

#[test]
fn registration_order_changes_the_type_list_but_not_the_trace() {
    game::extension::uninstall();
    let scenario = scenario();
    let config = scenario::runner::PlayConfig {
        follow_handle: None,
    };

    // Après la clôture : le `configure` de `run_with` s'exécute une fois `CoreSetupPlugin` ajouté.
    let mut after = scenario::runner::build_app(&scenario, true, &config);
    register(&mut after);
    let after_types = listed(&after);
    let after_outcome = scenario::run_with(&scenario, register);

    // Avant la clôture : le plugin installé, `CoreSimPlugin` l'appelle lui-même.
    game::extension::install(vec![Arc::new(Marker)]);
    let before = scenario::runner::build_app(&scenario, true, &config);
    let before_types = listed(&before);
    let before_outcome = scenario::run(&scenario);

    assert!(
        after_outcome.failures.is_empty() && before_outcome.failures.is_empty(),
        "{:?} / {:?}",
        after_outcome.failures,
        before_outcome.failures
    );
    // Même état, même checksum à chaque frame, rollbacks du synctest compris.
    assert_eq!(
        before_outcome.trace.len(),
        119,
        "une ligne par frame confirmée (frames - 1)"
    );
    assert_eq!(before_outcome.trace, after_outcome.trace);

    // Le type est tracé dans les deux cas, mais pas à la même place : avant, il précède les cinq de clôture.
    let position = |types: &[String]| types.iter().position(|t| t == "Beat").unwrap();
    assert_eq!(position(&before_types), before_types.len() - 6);
    assert_eq!(position(&after_types), after_types.len() - 1);
    assert_eq!(
        before_types.len(),
        after_types.len(),
        "mêmes types, ordre seul différent"
    );
    game::extension::uninstall();
}
