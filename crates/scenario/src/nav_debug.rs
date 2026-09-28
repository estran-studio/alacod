//! Diagnostic de la navigation des ennemis : grille ASCII qui superpose ce que voit le
//! pathfinding (flow field) et ce qu'impose la physique (colliders).
//!
//! `ALACOD_NAV=<scénario>:<frame> cargo test -p scenario --profile headless --test scenarios nav_map -- --ignored --nocapture`

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use game::{
    character::{
        enemy::{
            ai::{navigation::GRID_CELL_SIZE, FlowFieldCache, GridPos, NavProfile, Obstacle},
            Enemy,
        },
        player::Player,
    },
    collider::{Collider, ColliderShape, Wall},
};
use map::game::entity::map::{door::DoorComponent, enemy_spawn::EnemySpawnerComponent};

/// Cellules couvertes par un collider (même calcul de bornes que le pathfinding).
fn collider_cells(transform: &FixedTransform3D, collider: &Collider) -> Vec<GridPos> {
    let center_x = (transform.translation.x + collider.offset.x).to_num::<i32>();
    let center_y = (transform.translation.y + collider.offset.y).to_num::<i32>();
    let (half_w, half_h) = match &collider.shape {
        ColliderShape::Circle { radius } => (radius.to_num::<i32>(), radius.to_num::<i32>()),
        ColliderShape::Rectangle { width, height } => (width.to_num::<i32>() / 2, height.to_num::<i32>() / 2),
    };
    let min_x = (center_x - half_w).div_euclid(GRID_CELL_SIZE);
    let max_x = (center_x + half_w - 1).div_euclid(GRID_CELL_SIZE).max(min_x);
    let min_y = (center_y - half_h).div_euclid(GRID_CELL_SIZE);
    let max_y = (center_y + half_h - 1).div_euclid(GRID_CELL_SIZE).max(min_y);
    (min_x..=max_x)
        .flat_map(|x| (min_y..=max_y).map(move |y| GridPos::new(x, y)))
        .collect()
}

/// Direction du flow field en style pavé numérique : 8 haut, 2 bas, 4 gauche, 6 droite,
/// 7 9 1 3 diagonales, 5 cible.
fn arrow(from: GridPos, to: GridPos) -> char {
    match ((to.x - from.x).signum(), (to.y - from.y).signum()) {
        (0, 1) => '8',
        (0, -1) => '2',
        (-1, 0) => '4',
        (1, 0) => '6',
        (-1, 1) => '7',
        (1, 1) => '9',
        (-1, -1) => '1',
        (1, -1) => '3',
        _ => '5',
    }
}

/// Grille ASCII de la navigation, du haut (y max) vers le bas. Légende en tête.
/// Avec `arrows`, les cases atteignables montrent la direction du flow field.
pub fn nav_ascii(world: &mut World, arrows: bool) -> String {
    let cache = world.resource::<FlowFieldCache>().clone();
    let field = cache.get_flow_field(NavProfile::GroundBreaker).cloned().unwrap_or_default();

    let mut physics: BTreeSet<GridPos> = BTreeSet::new();
    let mut q = world.query_filtered::<(&FixedTransform3D, &Collider), (With<Wall>, Without<Obstacle>)>();
    for (t, c) in q.iter(world) {
        physics.extend(collider_cells(t, c));
    }

    let mut marks: BTreeMap<GridPos, char> = BTreeMap::new();
    let mut q = world.query::<(&FixedTransform3D, Option<&Collider>, &Obstacle)>();
    for (t, c, obstacle) in q.iter(world) {
        let cells = c.map(|c| collider_cells(t, c)).unwrap_or_else(|| vec![GridPos::from_fixed(t.translation.truncate())]);
        let ch = if obstacle.blocks_movement { 'W' } else { 'w' };
        for cell in cells {
            marks.insert(cell, ch);
        }
    }
    // Porte ouverte = sans collider (une porte non interactive reste fermée)
    let mut q = world.query_filtered::<(&FixedTransform3D, Option<&Collider>), With<DoorComponent>>();
    for (t, c) in q.iter(world) {
        let cells = c.map(|c| collider_cells(t, c)).unwrap_or_else(|| vec![GridPos::from_fixed(t.translation.truncate())]);
        for cell in cells {
            marks.insert(cell, if c.is_some() { 'D' } else { 'd' });
        }
    }
    let mut q = world.query_filtered::<&FixedTransform3D, With<EnemySpawnerComponent>>();
    for t in q.iter(world) {
        marks.insert(GridPos::from_fixed(t.translation.truncate()), 'S');
    }
    let mut q = world.query_filtered::<&FixedTransform3D, With<Enemy>>();
    for t in q.iter(world) {
        marks.insert(GridPos::from_fixed(t.translation.truncate()), 'Z');
    }
    let mut q = world.query_filtered::<&FixedTransform3D, With<Player>>();
    for t in q.iter(world) {
        marks.insert(GridPos::from_fixed(t.translation.truncate()), 'P');
    }

    let all: Vec<&GridPos> = cache.wall_cells.iter().chain(physics.iter()).chain(marks.keys()).collect();
    if all.is_empty() {
        return "aucune donnée de navigation".into();
    }
    let min_x = all.iter().map(|p| p.x).min().unwrap() - 1;
    let max_x = all.iter().map(|p| p.x).max().unwrap() + 1;
    let min_y = all.iter().map(|p| p.y).min().unwrap() - 1;
    let max_y = all.iter().map(|p| p.y).max().unwrap() + 1;

    let mut out = String::from(
        "légende : # mur (pathfinding + physique)  p mur physique seul  n mur pathfinding seul\n\
         W fenêtre intacte  w fenêtre cassée  D porte fermée  d porte ouverte  S spawner  Z zombie  P joueur\n\
         . atteignable par le flow field (ou direction 8/2/4/6/7/9/1/3)   (espace) libre mais NON atteignable\n",
    );
    out += &format!(
        "x {min_x}..{max_x}, y {min_y}..{max_y} (cellules de {GRID_CELL_SIZE} px), cible {:?}, {} cellules atteignables\n",
        cache.target_pos,
        field.directions.len()
    );
    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let pos = GridPos::new(x, y);
            let nav_wall = cache.wall_cells.contains(&pos);
            let phys_wall = physics.contains(&pos);
            let ch = if let Some(m) = marks.get(&pos) {
                *m
            } else if nav_wall && phys_wall {
                '#'
            } else if phys_wall {
                'p'
            } else if nav_wall {
                'n'
            } else if let Some(next) = field.directions.get(&pos) {
                if arrows { arrow(pos, *next) } else { '.' }
            } else {
                ' '
            };
            out.push(ch);
        }
        out.push('\n');
    }
    out
}
