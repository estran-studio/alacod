//! Destruction de terrain (`effects::Action::DestroyTerrain`, T1.6) : forme pure.

use bevy_fixed::fixed_math::{Fixed, FixedVec2, FixedWide};

use crate::grid::{CellGrid, CellKind, CELL_SIZE};

/// Toute case `Rock` dont le centre est à **strictement** moins de `radius` de `center`
/// devient `Floor` ; `Wall` et `Floor` ne changent jamais. Rend les cases détruites, dans
/// l'ordre de balayage (rangée par rangée depuis le bas). Distances en `FixedWide` : le carré
/// d'une distance de plusieurs centaines d'unités dépasse `Fixed`.
pub fn destroy_terrain(grid: &mut CellGrid, center: FixedVec2, radius: Fixed) -> Vec<(u32, u32)> {
    let mut destroyed = Vec::new();
    if grid.is_empty() || radius <= Fixed::ZERO {
        return destroyed;
    }
    let r = FixedWide::from_num(radius);
    let r2 = r * r;
    // Boîte englobante en cases, bornée à la grille
    let reach = radius.to_num::<i32>() / CELL_SIZE + 1;
    let (cx, cy) = CellGrid::cell_of(center);
    let x0 = (cx - reach).max(0);
    let y0 = (cy - reach).max(0);
    let x1 = (cx + reach).min(grid.width as i32 - 1);
    let y1 = (cy + reach).min(grid.height as i32 - 1);
    for y in y0..=y1 {
        for x in x0..=x1 {
            if grid.get(x, y) != Some(CellKind::Rock) {
                continue;
            }
            let c = CellGrid::cell_center(x as u32, y as u32);
            let dx = FixedWide::from_num(c.x) - FixedWide::from_num(center.x);
            let dy = FixedWide::from_num(c.y) - FixedWide::from_num(center.y);
            if dx * dx + dy * dy < r2 {
                grid.set(x as u32, y as u32, CellKind::Floor);
                destroyed.push((x as u32, y as u32));
            }
        }
    }
    destroyed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i32, y: i32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    #[test]
    fn rayon_strict_et_mur_intact() {
        // 5x5 : bordure Wall, intérieur Rock
        let mut grid = CellGrid::filled(5, 5, CellKind::Rock);
        for i in 0..5 {
            grid.set(i, 0, CellKind::Wall);
            grid.set(i, 4, CellKind::Wall);
            grid.set(0, i, CellKind::Wall);
            grid.set(4, i, CellKind::Wall);
        }
        // Centre de la case (2, 2) = (40, 40) ; voisins cardinaux à 16, diagonaux à 22.6
        let destroyed = destroy_terrain(&mut grid, at(40, 40), Fixed::from_num(16));
        assert_eq!(
            destroyed,
            vec![(2, 2)],
            "16 n'est pas < 16 : voisins intacts"
        );
        let destroyed = destroy_terrain(&mut grid, at(40, 40), Fixed::from_num(17));
        assert_eq!(destroyed, vec![(2, 1), (1, 2), (3, 2), (2, 3)]);
        // Un grand rayon vide l'intérieur mais jamais la bordure
        destroy_terrain(&mut grid, at(40, 40), Fixed::from_num(500));
        assert_eq!(grid.count(CellKind::Rock), 0);
        assert_eq!(grid.count(CellKind::Wall), 16);
        assert_eq!(grid.count(CellKind::Floor), 9);
    }

    #[test]
    fn hors_grille_et_grille_vide() {
        let mut grid = CellGrid::filled(3, 3, CellKind::Rock);
        assert!(destroy_terrain(&mut grid, at(-200, -200), Fixed::from_num(40)).is_empty());
        let mut empty = CellGrid::default();
        assert!(destroy_terrain(&mut empty, at(0, 0), Fixed::from_num(40)).is_empty());
    }
}
