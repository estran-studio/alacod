//! m1-integration-scenarios : en mode `Floors`, chaque étage de caverne est **sa** caverne, et
//! le terrain (`CellGrid`, source de la destruction et de la navigation) correspond aux murs
//! réellement chargés (un mur LDtk ⇔ une cellule solide).
//!
//! Avant le correctif « une caverne = un asset », les trois cavernes de `throne` partageaient le
//! chemin d'asset `caves/gabarit.ldtk` : l'`AssetServer` rendait le premier chargement
//! (`niveau_1`) aux trois étages, alors que `CellGrid` était régénérée depuis la vraie config
//! (`niveau_2`, `niveau_3`) : murs et terrain ne correspondaient plus.

use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use bevy_ggrs::Rollback;
use combat::collider::{Collider, ColliderShape};
use game::collider::Wall;
use run::FloorState;
use scenario::{runner, Scenario};
use utils::frame::FrameCount;
use world::{CellGrid, CELL_SIZE};

fn throne_three_floors() -> Scenario {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/throne_three_floors.ron"
    );
    let source = std::fs::read_to_string(path).expect("throne_three_floors.ron");
    Scenario::from_ron(&source).expect("scénario valide")
}

/// Cellules couvertes par les murs rollback (rectangles fusionnés), en coordonnées de grille
/// (origine du niveau en (0, 0) monde, `docs/conventions.md` §21).
fn wall_cells(world: &mut World) -> BTreeSet<(i32, i32)> {
    let mut cells = BTreeSet::new();
    let mut q =
        world.query_filtered::<(&FixedTransform3D, &Collider), (With<Wall>, With<Rollback>)>();
    for (transform, collider) in q.iter(world) {
        let ColliderShape::Rectangle { width, height } = collider.shape else {
            continue;
        };
        let cx = (transform.translation.x + collider.offset.x).to_num::<f32>();
        let cy = (transform.translation.y + collider.offset.y).to_num::<f32>();
        let (w, h) = (width.to_num::<f32>(), height.to_num::<f32>());
        let size = CELL_SIZE as f32;
        let x0 = ((cx - w / 2.0) / size).round() as i32;
        let x1 = ((cx + w / 2.0) / size).round() as i32;
        let y0 = ((cy - h / 2.0) / size).round() as i32;
        let y1 = ((cy + h / 2.0) / size).round() as i32;
        for x in x0..x1 {
            for y in y0..y1 {
                cells.insert((x, y));
            }
        }
    }
    cells
}

fn grid_solid_cells(grid: &CellGrid) -> BTreeSet<(i32, i32)> {
    let mut cells = BTreeSet::new();
    for y in 0..grid.height as i32 {
        for x in 0..grid.width as i32 {
            if grid.get(x, y).is_some_and(|cell| cell.is_solid()) {
                cells.insert((x, y));
            }
        }
    }
    cells
}

/// État d'un étage : (index, largeur × hauteur de la grille, murs absents de la grille, cases
/// solides de la grille sans mur).
fn floor_coherence(app: &mut App) -> (u32, (u32, u32), usize, usize) {
    let world = app.world_mut();
    let index = world.resource::<FloorState>().index;
    let walls = wall_cells(world);
    let grid = world.resource::<CellGrid>();
    let solid = grid_solid_cells(grid);
    (
        index,
        (grid.width, grid.height),
        walls.difference(&solid).count(),
        solid.difference(&walls).count(),
    )
}

/// Avance jusqu'à la première frame où `FloorState::index` vaut `index`.
fn step_until_floor(app: &mut App, index: u32, max_frames: u32) {
    for _ in 0..max_frames * 4 {
        let previous = app.world().resource::<FrameCount>().frame;
        app.update();
        if app.world().resource::<FrameCount>().frame != previous
            && app.world().resource::<FloorState>().index == index
        {
            return;
        }
    }
    panic!("étage {index} jamais atteint en {max_frames} frames");
}

#[test]
fn chaque_etage_charge_sa_caverne_et_son_terrain() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let scenario = throne_three_floors();
    let mut app = runner::run_until(&scenario, 1);
    let mut seen = Vec::new();
    seen.push(floor_coherence(&mut app));
    for index in 1..=2 {
        step_until_floor(&mut app, index, scenario.frames);
        seen.push(floor_coherence(&mut app));
    }
    eprintln!("(étage, grille, murs hors grille, grille sans mur) : {seen:?}");
    // Dimensions de `caves/niveau_{1,2,3}.ron`.
    let sizes: Vec<_> = seen.iter().map(|(_, size, ..)| *size).collect();
    assert_eq!(sizes, [(48, 32), (56, 40), (64, 44)], "{seen:?}");
    for (index, _, walls_outside, grid_without_wall) in &seen {
        assert_eq!(
            (*walls_outside, *grid_without_wall),
            (0, 0),
            "étage {index} : murs et terrain ne correspondent pas ({seen:?})"
        );
    }
}
