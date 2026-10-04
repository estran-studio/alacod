//! T1.6 : une caverne (`cave:petite` du testbed) se charge par le chemin LDtk ordinaire, et
//! l'attente `CellState` lit `world::CellGrid`.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use bevy_ggrs::Rollback;
use game::character::player::Player;
use game::collider::Wall;
use game::replay::{Expectation, Scenario};
use scenario::{run, run_with};
use world::{CaveConfig, CellGrid, CellKind};

const SEED: i32 = 123456;

fn petite() -> CaveConfig {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../games/testbed/assets/caves/petite.ron"
    );
    ron::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn cave_scenario(frames: u32, expect: Vec<Expectation>) -> Scenario {
    let mut scenario = Scenario::from_ron(&format!(
        r#"(game: "testbed", map: "cave:petite", map_seed: {SEED}, frames: {frames},
            players: [()])"#
    ))
    .expect("scénario de caverne");
    scenario.expect = expect;
    scenario
}

/// Une case de chaque nature dans la grille générée pour `SEED`.
fn sample(grid: &CellGrid, kind: CellKind) -> (i32, i32) {
    (0..grid.height as i32)
        .flat_map(|y| (0..grid.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| grid.get(x, y) == Some(kind))
        .unwrap()
}

#[test]
fn cell_state_suit_la_grille_generee() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let grid = world::generate(SEED as u32 as u64, &petite());
    let expect = [CellKind::Wall, CellKind::Rock, CellKind::Floor]
        .into_iter()
        .map(|kind| {
            let (x, y) = sample(&grid, kind);
            Expectation::CellState {
                x,
                y,
                kind,
                at_frame: 10,
            }
        })
        .collect();
    let outcome = run(&cave_scenario(20, expect));
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);

    // Mauvaise nature et case hors grille : échecs
    let (x, y) = sample(&grid, CellKind::Rock);
    let outcome = run(&cave_scenario(
        20,
        vec![
            Expectation::CellState {
                x,
                y,
                kind: CellKind::Floor,
                at_frame: 10,
            },
            Expectation::CellState {
                x: -1,
                y: 0,
                kind: CellKind::Wall,
                at_frame: 10,
            },
        ],
    ));
    assert_eq!(outcome.failures.len(), 2, "{:?}", outcome.failures);
}

/// Les murs physiques et la grille viennent des mêmes cases ; le joueur apparaît sur le sol.
#[test]
fn caverne_chargee_par_le_chemin_ldtk() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // (murs rollback, grille, case du joueur), relevés à chaque frame
    type Seen = Arc<Mutex<(usize, CellGrid, Option<(i32, i32)>)>>;
    let seen: Seen = Arc::default();
    let probe = seen.clone();
    let outcome = run_with(&cave_scenario(20, vec![]), move |app| {
        app.add_systems(
            Last,
            move |walls: Query<(), (With<Wall>, With<Rollback>)>,
                  grid: Res<CellGrid>,
                  players: Query<&FixedTransform3D, With<Player>>| {
                let mut seen = probe.lock().unwrap();
                seen.0 = walls.iter().count();
                seen.1 = grid.clone();
                seen.2 = players
                    .iter()
                    .next()
                    .map(|t| CellGrid::cell_of(t.translation.truncate()));
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let (walls, grid, player) = seen.lock().unwrap().clone();
    assert_eq!(grid, world::generate(SEED as u32 as u64, &petite()));
    assert!(walls > 0, "aucun mur de caverne");
    let (x, y) = player.expect("joueur absent");
    assert_eq!(
        grid.get(x, y),
        Some(CellKind::Floor),
        "joueur en ({x}, {y})"
    );
}

/// Une demande de destruction (sans projectile) creuse un cratère de roche, recrée les murs et
/// reste déterministe en synctest (rejeux à chaque frame : murs recréés et navigation
/// resynchronisée depuis `CellGrid`, état rollback).
#[test]
fn destruction_creuse_et_reconstruit_les_murs_en_synctest() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use bevy_ggrs::GgrsSchedule;
    use sim_core::frame_events::FrameEvents;
    use sim_core::system_set::RollbackSystemSet;
    use utils::frame::FrameCount;
    use world::DestroyTerrainRequest;

    const AT: u32 = 30;
    let grid = world::generate(SEED as u32 as u64, &petite());
    // Une case de roche voisine du sol : le cratère de rayon 40 la contient
    let (x, y) = (1..grid.height as i32 - 1)
        .flat_map(|y| (1..grid.width as i32 - 1).map(move |x| (x, y)))
        .find(|&(x, y)| {
            grid.get(x, y) == Some(CellKind::Rock) && grid.get(x + 1, y) == Some(CellKind::Floor)
        })
        .unwrap();
    let center = CellGrid::cell_center(x as u32, y as u32);
    let mut after = grid.clone();
    let destroyed = world::destroy_terrain(
        &mut after,
        center,
        bevy_fixed::fixed_math::Fixed::from_num(40),
    );
    assert!(destroyed.len() > 1);

    let mut expect = vec![
        Expectation::CellState {
            x,
            y,
            kind: CellKind::Rock,
            at_frame: AT - 1,
        },
        Expectation::CellState {
            x,
            y,
            kind: CellKind::Floor,
            at_frame: AT + 5,
        },
    ];
    // Les bordures restent des murs
    expect.push(Expectation::CellState {
        x: 0,
        y,
        kind: CellKind::Wall,
        at_frame: AT + 5,
    });
    let seen: Arc<Mutex<(usize, CellGrid)>> = Arc::default();
    let probe = seen.clone();
    let outcome = run_with(&cave_scenario(AT + 60, expect), move |app| {
        app.add_systems(
            GgrsSchedule,
            (move |frame: Res<FrameCount>,
                   mut requests: ResMut<FrameEvents<DestroyTerrainRequest>>| {
                if frame.frame == AT {
                    requests.send(DestroyTerrainRequest::new(
                        center,
                        bevy_fixed::fixed_math::Fixed::from_num(40),
                    ));
                }
            })
            .after(RollbackSystemSet::Projectiles)
            .before(RollbackSystemSet::World),
        )
        .add_systems(
            Last,
            move |walls: Query<(), (With<Wall>, With<Rollback>)>, grid: Res<CellGrid>| {
                *probe.lock().unwrap() = (walls.iter().count(), grid.clone());
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let (walls, final_grid) = seen.lock().unwrap().clone();
    assert_eq!(final_grid, after, "grille finale = grille creusée");
    assert!(walls > 0);
}

/// Mode `Floors` : une caverne est un niveau valide d'une séquence (`floors/caverne.ron` du
/// testbed : `floor_a`, `cave:petite`, `floor_b`). Mêmes inputs que `portal_next_floor` : le
/// follower de `floor_a` meurt, le joueur entre dans le portail et passe dans la caverne
/// (`CellGrid` remplie au passage) ; sans ennemi, le portail de la caverne s'ouvre aussitôt et
/// mène à `floor_b`, où `CellGrid` redevient vide.
#[test]
fn caverne_dans_une_sequence_floors() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = Scenario::from_ron(&format!(
        r#"(game: "testbed", floors: Some("caverne"), map_seed: {SEED}, frames: 300,
            powerup_drop_chance_override: "0.0",
            players: [(inputs: [
                (from: 0, to: 200, buttons: [Fire], pan: (-64, 48)),
                (from: 200, to: 230, buttons: [Right, Down], pan: (0, 0)),
            ])])"#
    ))
    .expect("scénario Floors");
    scenario.expect = vec![
        Expectation::FloorIndex {
            index: 0,
            at_frame: 220,
        },
        Expectation::FloorIndex {
            index: 2,
            at_frame: 299,
        },
    ];
    // Niveau courant -> la grille a-t-elle été vue remplie / vide pendant ce niveau
    type Seen = Arc<Mutex<std::collections::BTreeMap<u32, (bool, bool)>>>;
    let seen: Seen = Arc::default();
    let probe = seen.clone();
    let outcome = run_with(&scenario, move |app| {
        app.add_systems(
            Last,
            move |grid: Res<CellGrid>, floor: Res<run::FloorState>| {
                let mut seen = probe.lock().unwrap();
                let entry = seen.entry(floor.index).or_default();
                if grid.is_empty() {
                    entry.1 = true;
                } else {
                    entry.0 = true;
                }
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(
        seen.get(&1),
        Some(&(true, false)),
        "caverne : grille remplie ({seen:?})"
    );
    assert_eq!(
        seen.get(&2),
        Some(&(false, true)),
        "floor_b : grille vide ({seen:?})"
    );
}
