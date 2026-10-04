//! Grille de cellules du monde (T1.0b) : état rollback du terrain d'une caverne.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use serde::{Deserialize, Serialize};

/// Côté d'une cellule en unités monde : la même que `GRID_CELL_SIZE` de la navigation et que
/// les tuiles LDtk (`docs/conventions.md` §1).
pub const CELL_SIZE: i32 = 16;

/// Nature d'une cellule. `Wall` est indestructible (bordure), `Rock` se détruit
/// (`DestroyTerrain`) et devient `Floor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CellKind {
    #[default]
    Floor,
    Wall,
    Rock,
}

impl CellKind {
    /// Bloque le passage (et devient un mur physique).
    pub fn is_solid(self) -> bool {
        !matches!(self, CellKind::Floor)
    }

    fn glyph(self) -> char {
        match self {
            CellKind::Floor => '.',
            CellKind::Wall => '#',
            CellKind::Rock => 'r',
        }
    }
}

/// Terrain d'une caverne, ressource rollback + checksum + trace (enregistrée en
/// `rollback_and_trace_resource_neutral` : la grille vide par défaut contribue 0 au checksum,
/// une carte LDtk ordinaire la laisse vide).
///
/// Coordonnées : cellule `(x, y)` = carré monde `[16x, 16x + 16) × [16y, 16y + 16)`, origine
/// en (0, 0), **+y vers le haut** comme le monde et `GridPos` de la navigation (la rangée 0 est
/// en bas, contrairement aux rangées LDtk).
#[derive(Resource, Clone, Default, PartialEq, Eq, Hash)]
pub struct CellGrid {
    pub width: u32,
    pub height: u32,
    /// Rangée par rangée en partant du bas : index `y * width + x`.
    pub cells: Vec<CellKind>,
}

impl CellGrid {
    pub fn filled(width: u32, height: u32, kind: CellKind) -> Self {
        Self {
            width,
            height,
            cells: vec![kind; (width * height) as usize],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    /// `None` hors de la grille.
    pub fn get(&self, x: i32, y: i32) -> Option<CellKind> {
        self.in_bounds(x, y)
            .then(|| self.cells[self.index(x as u32, y as u32)])
    }

    pub fn set(&mut self, x: u32, y: u32, kind: CellKind) {
        let i = self.index(x, y);
        self.cells[i] = kind;
    }

    /// Centre monde de la cellule.
    pub fn cell_center(x: u32, y: u32) -> FixedVec2 {
        let half = CELL_SIZE / 2;
        FixedVec2::new(
            Fixed::from_num(x as i32 * CELL_SIZE + half),
            Fixed::from_num(y as i32 * CELL_SIZE + half),
        )
    }

    /// Cellule contenant une position monde (peut être hors de la grille).
    pub fn cell_of(position: FixedVec2) -> (i32, i32) {
        (
            position.x.to_num::<i32>().div_euclid(CELL_SIZE),
            position.y.to_num::<i32>().div_euclid(CELL_SIZE),
        )
    }

    pub fn count(&self, kind: CellKind) -> usize {
        self.cells.iter().filter(|c| **c == kind).count()
    }

    /// Rangées de haut en bas, une lettre par cellule (`.` sol, `#` mur, `r` roche).
    pub fn rows_top_down(&self) -> Vec<String> {
        (0..self.height)
            .rev()
            .map(|y| {
                (0..self.width)
                    .map(|x| self.cells[self.index(x, y)].glyph())
                    .collect()
            })
            .collect()
    }
}

/// Compact dans la trace détaillée : dimensions et rangées (une ligne par rangée, haut en bas),
/// au lieu d'une liste de milliers de variantes.
impl std::fmt::Debug for CellGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "CellGrid(vide)");
        }
        write!(f, "CellGrid({}x{}", self.width, self.height)?;
        for row in self.rows_top_down() {
            write!(f, " {row}")?;
        }
        write!(f, ")")
    }
}

/// Marqueur du **monde** (posé sur l'entité de niveau LDtk de la caverne, jamais une entité par
/// cellule) : la carte courante est une caverne dont les murs suivent [`CellGrid`].
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Destructible;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordonnees_y_vers_le_haut() {
        let mut grid = CellGrid::filled(3, 2, CellKind::Floor);
        grid.set(0, 1, CellKind::Rock);
        assert_eq!(grid.get(0, 1), Some(CellKind::Rock));
        assert_eq!(grid.get(3, 0), None);
        assert_eq!(grid.get(0, -1), None);
        assert_eq!(grid.rows_top_down(), vec!["r..", "..."]);
        assert_eq!(
            CellGrid::cell_center(2, 1),
            FixedVec2::new(Fixed::from_num(40), Fixed::from_num(24))
        );
        assert_eq!(
            CellGrid::cell_of(FixedVec2::new(Fixed::from_num(-1), Fixed::from_num(31))),
            (-1, 1)
        );
    }

    #[test]
    fn debug_compact() {
        let grid = CellGrid::filled(2, 1, CellKind::Wall);
        assert_eq!(format!("{grid:?}"), "CellGrid(2x1 ##)");
        assert_eq!(format!("{:?}", CellGrid::default()), "CellGrid(vide)");
    }
}
