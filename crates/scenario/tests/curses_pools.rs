//! M3-T0e (`docs/conventions.md` §64) : `RunCurses` et `PoolState` sont des ressources rollback
//! neutres ; M3-T9 remplit `RunCurses` en jeu (§74), M3-T8 remplira `PoolState`. Ce test les remplit depuis un système
//! de `GgrsSchedule` du test et prouve : (1) le synctest (sauvegarde puis restauration à chaque
//! frame) ne trouve aucune divergence, donc elles sont bien dans les snapshots et le checksum ;
//! (2) les attentes `Curse` et `PoolSeen` les lisent ; (3) une relance les remet à zéro.

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;
use game::{
    core::AppState, replay::Expectation, run_state::RunRequest, system_set::RollbackSystemSet,
};
use items::PoolState;
use run::RunCurses;
use scenario::{
    runner::{build_app, PlayConfig},
    Scenario,
};
use utils::frame::FrameCount;

fn arena() -> Scenario {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/testbed_arena_idle.ron"
    );
    let source = std::fs::read_to_string(path).expect("testbed_arena_idle.ron");
    Scenario::from_ron(&source).expect("scénario valide")
}

/// Frame de la simulation à laquelle le test écrit dans les ressources.
const WRITE_AT: u32 = 30;

/// Étage 0 maudit (« darkness ») à `WRITE_AT`, étage 1 (« maze ») à `2 * WRITE_AT` ; `a` sort du
/// pool `p` à `WRITE_AT`.
fn write_state(
    frame: Res<FrameCount>,
    mut curses: ResMut<RunCurses>,
    mut pools: ResMut<PoolState>,
) {
    if frame.frame == WRITE_AT {
        curses.set(0, "darkness");
        pools.record("p", "a");
    }
    if frame.frame == 2 * WRITE_AT {
        curses.set(1, "maze");
    }
}

fn curse(floor: u32, id: Option<&str>, at_frame: u32) -> Expectation {
    Expectation::Curse {
        floor,
        id: id.map(str::to_string),
        at_frame,
    }
}

fn seen(item: &str, seen: bool, at_frame: u32) -> Expectation {
    Expectation::PoolSeen {
        pool: "p".into(),
        item: item.into(),
        seen,
        at_frame,
    }
}

#[test]
fn expectations_read_the_resources_and_synctest_finds_no_divergence() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = arena();
    scenario.frames = 100;
    scenario.expect = vec![
        curse(0, None, WRITE_AT - 5),
        curse(0, Some("darkness"), WRITE_AT + 5),
        curse(1, None, 2 * WRITE_AT - 5),
        curse(1, Some("maze"), 2 * WRITE_AT + 5),
        // L'étage 0 garde sa curse quand l'étage 1 reçoit la sienne.
        curse(0, Some("darkness"), 2 * WRITE_AT + 5),
        seen("a", false, WRITE_AT - 5),
        seen("a", true, WRITE_AT + 5),
        seen("b", false, WRITE_AT + 5),
    ];
    let outcome = scenario::run_with(&scenario, |app| {
        app.add_systems(
            GgrsSchedule,
            write_state
                .after(RollbackSystemSet::Run)
                .before(RollbackSystemSet::FrameCounter),
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn a_wrong_expectation_fails_with_the_observed_value() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = arena();
    scenario.frames = 60;
    scenario.expect = vec![
        curse(0, Some("blind"), WRITE_AT + 5),
        curse(1, Some("maze"), WRITE_AT + 5),
        seen("a", false, WRITE_AT + 5),
        seen("b", true, WRITE_AT + 5),
    ];
    let outcome = scenario::run_with(&scenario, |app| {
        app.add_systems(
            GgrsSchedule,
            write_state
                .after(RollbackSystemSet::Run)
                .before(RollbackSystemSet::FrameCounter),
        );
    });
    let failures = outcome.failures.join("\n");
    for needle in [
        "curse de l'étage 0 : Some(\"darkness\") (attendu Some(\"blind\"))",
        "curse de l'étage 1 : None (attendu Some(\"maze\"))",
        "« a » sorti = true (attendu false)",
        "« b » sorti = false (attendu true)",
    ] {
        assert!(
            failures.contains(needle),
            "{needle} absent de :\n{failures}"
        );
    }
}

#[test]
fn a_restart_clears_curses_and_pool_state() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = arena();
    scenario.frames = 100;
    scenario.expect = vec![];
    let mut app = build_app(
        &scenario,
        true,
        &PlayConfig {
            follow_handle: None,
        },
    );
    app.add_systems(
        GgrsSchedule,
        write_state
            .after(RollbackSystemSet::Run)
            .before(RollbackSystemSet::FrameCounter),
    );
    app.finish();
    app.cleanup();

    let frame = |app: &App| app.world().resource::<FrameCount>().frame;
    for _ in 0..20_000 {
        app.update();
        if frame(&app) > WRITE_AT + 5 {
            break;
        }
    }
    assert_eq!(app.world().resource::<RunCurses>().get(0), Some("darkness"));
    assert!(app.world().resource::<PoolState>().has_seen("p", "a"));

    app.world_mut().insert_resource(RunRequest::Restart);
    let state = |app: &App| app.world().resource::<State<AppState>>().get().clone();
    let mut left = false;
    for _ in 0..20_000 {
        app.update();
        if state(&app) != AppState::InGame {
            left = true;
        } else if left {
            break;
        }
    }
    assert!(left, "jamais quitté InGame");
    assert!(
        frame(&app) <= WRITE_AT,
        "le test lit l'état avant la réécriture de la nouvelle partie (frame {})",
        frame(&app)
    );
    assert_eq!(*app.world().resource::<RunCurses>(), RunCurses::default());
    assert_eq!(*app.world().resource::<PoolState>(), PoolState::default());
}
