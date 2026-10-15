//! Explosion (`effects::Action::Explode`, M4-T0c, `docs/conventions.md` §62) : forme pure.
//!
//! Contrat seul : la règle qui dit **qui** et **quoi** une explosion touche, sans ECS. Son exécution
//! (dégâts, rochers, mur secret) vient de M4-T4 ; le baril ne l'emploie pas encore (sa trace ne
//! doit pas bouger). Elle vit dans `world` (comme `destroy_terrain`, qu'elle réutilise) : la
//! frontière `SIM` n'y voit que des identifiants et des positions.

use bevy_fixed::fixed_math::{Fixed, FixedVec2, FixedWide};

use crate::destroy::destroy_terrain;
use crate::grid::CellGrid;

/// Ce qu'une explosion touche.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExplosionPlan {
    /// `GgrsNetId.0` des personnages touchés, dans l'ordre croissant des identifiants.
    pub targets: Vec<u32>,
    /// Cases de roche cassées (balayage de `destroy_terrain`), vide si `destroys` est faux.
    pub destroyed: Vec<(u32, u32)>,
}

/// Distance centre à centre strictement inférieure à `radius` (même règle que `destroy_terrain` :
/// carré en `FixedWide`, qui dépasse `Fixed` au-delà de quelques centaines d'unités).
pub fn within_radius(center: FixedVec2, point: FixedVec2, radius: Fixed) -> bool {
    if radius <= Fixed::ZERO {
        return false;
    }
    let r = FixedWide::from_num(radius);
    let dx = FixedWide::from_num(point.x) - FixedWide::from_num(center.x);
    let dy = FixedWide::from_num(point.y) - FixedWide::from_num(center.y);
    dx * dx + dy * dy < r * r
}

/// Cibles d'une explosion en `center` : `candidates` = `(GgrsNetId.0, position)` des personnages
/// à PV (le système les filtre). Rend les identifiants triés, sans doublon. Si `destroys`, casse
/// aussi les rochers de `grid` dans le rayon (la grille est modifiée, vide = rien).
pub fn explosion_plan(
    grid: &mut CellGrid,
    center: FixedVec2,
    radius: Fixed,
    destroys: bool,
    candidates: &[(u32, FixedVec2)],
) -> ExplosionPlan {
    let mut targets: Vec<u32> = candidates
        .iter()
        .filter(|(_, position)| within_radius(center, *position, radius))
        .map(|(id, _)| *id)
        .collect();
    targets.sort_unstable();
    targets.dedup();
    let destroyed = if destroys {
        destroy_terrain(grid, center, radius)
    } else {
        Vec::new()
    };
    ExplosionPlan { targets, destroyed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::CellKind;

    fn at(x: i32, y: i32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    #[test]
    fn rayon_strict_et_ordre_des_identifiants() {
        let mut grid = CellGrid::default();
        let candidates = [
            (9, at(10, 0)),
            (3, at(0, 20)),
            (5, at(0, 21)),
            (1, at(-19, 0)),
        ];
        let plan = explosion_plan(&mut grid, at(0, 0), Fixed::from_num(20), false, &candidates);
        assert_eq!(
            plan.targets,
            vec![1, 9],
            "20 n'est pas < 20 : (0,20) hors rayon"
        );
        assert!(plan.destroyed.is_empty());
    }

    #[test]
    fn rayon_nul_ne_touche_personne() {
        let mut grid = CellGrid::filled(3, 3, CellKind::Rock);
        let plan = explosion_plan(&mut grid, at(24, 24), Fixed::ZERO, true, &[(1, at(24, 24))]);
        assert_eq!(plan, ExplosionPlan::default());
        assert_eq!(grid.count(CellKind::Rock), 9);
    }

    #[test]
    fn destroys_casse_les_rochers_et_pas_sans() {
        let mut grid = CellGrid::filled(3, 3, CellKind::Rock);
        let plan = explosion_plan(&mut grid, at(24, 24), Fixed::from_num(17), false, &[]);
        assert!(plan.destroyed.is_empty());
        assert_eq!(grid.count(CellKind::Rock), 9);
        let plan = explosion_plan(&mut grid, at(24, 24), Fixed::from_num(17), true, &[]);
        assert_eq!(plan.destroyed, vec![(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)]);
        assert_eq!(grid.count(CellKind::Rock), 4);
    }
}
