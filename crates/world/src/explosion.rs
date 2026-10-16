//! Explosion (`effects::Action::Explode`, M3-T0c, `docs/conventions.md` §62) : forme pure.
//!
//! Contrat seul : la règle qui dit **qui** et **quoi** une explosion touche, sans ECS. Son exécution
//! (dégâts, rochers, mur secret) vient de M3-T4 ; le baril ne l'emploie pas encore (sa trace ne
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

/// Le rectangle centré en `rect_center` (demi-côtés `half_w`, `half_h`) est-il touché ? Distance du
/// centre de l'explosion au point le plus proche du rectangle, strictement inférieure à `radius`
/// (un centre dans le rectangle donne 0 : touché). Sert aux portes et murs secrets, plus larges
/// qu'une case, dont le centre peut être hors de portée alors que leur bord y est.
pub fn rect_within_radius(
    center: FixedVec2,
    rect_center: FixedVec2,
    half_w: Fixed,
    half_h: Fixed,
    radius: Fixed,
) -> bool {
    let clamp = |value: Fixed, mid: Fixed, half: Fixed| value.clamp(mid - half, mid + half);
    let nearest = FixedVec2::new(
        clamp(center.x, rect_center.x, half_w),
        clamp(center.y, rect_center.y, half_h),
    );
    within_radius(center, nearest, radius)
}

/// Cases d'objets d'une case (`cell`, centre) touchées par une explosion : centre à **strictement**
/// moins de `radius` (même règle que `destroy_terrain` pour une case de roche), rendues dans
/// l'ordre croissant `(y, x)` : la même suite sur tous les clients, quel que soit l'ordre des
/// entités. Sert aux rochers-entités (M3-T4).
pub fn cells_in_radius(
    center: FixedVec2,
    radius: Fixed,
    cells: &[((i32, i32), FixedVec2)],
) -> Vec<(i32, i32)> {
    let mut hit: Vec<(i32, i32)> = cells
        .iter()
        .filter(|(_, at)| within_radius(center, *at, radius))
        .map(|(cell, _)| *cell)
        .collect();
    hit.sort_unstable_by_key(|&(x, y)| (y, x));
    hit.dedup();
    hit
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
    fn rectangle_touche_par_son_bord_le_plus_proche() {
        let rect = at(100, 0);
        let (hw, hh) = (Fixed::from_num(8), Fixed::from_num(16));
        // Bord gauche à x = 92 : à 92 du centre, 91 < 92 touche, 92 non (strict).
        assert!(rect_within_radius(
            at(0, 0),
            rect,
            hw,
            hh,
            Fixed::from_num(93)
        ));
        assert!(!rect_within_radius(
            at(0, 0),
            rect,
            hw,
            hh,
            Fixed::from_num(92)
        ));
        // Dans le rectangle : distance 0.
        assert!(rect_within_radius(
            at(100, 5),
            rect,
            hw,
            hh,
            Fixed::from_num(1)
        ));
        // Au-dessus du bord haut (y = 16) : distance verticale seule.
        assert!(rect_within_radius(
            at(100, 30),
            rect,
            hw,
            hh,
            Fixed::from_num(15)
        ));
        assert!(!rect_within_radius(
            at(100, 30),
            rect,
            hw,
            hh,
            Fixed::from_num(14)
        ));
        assert!(!rect_within_radius(at(0, 0), rect, hw, hh, Fixed::ZERO));
    }

    #[test]
    fn cases_touchees_triees_par_rangee_puis_colonne() {
        // Centres de cases de 16 : (2,1) = (40,24), (0,0) = (8,8), (1,0) = (24,8), (0,1) = (8,24)
        let cells = [
            ((2, 1), at(40, 24)),
            ((1, 0), at(24, 8)),
            ((0, 1), at(8, 24)),
            ((0, 0), at(8, 8)),
        ];
        // Centre de l'explosion (16, 16) : les trois premières à 11,3, (2,1) à 25,3
        let hit = cells_in_radius(at(16, 16), Fixed::from_num(12), &cells);
        assert_eq!(hit, vec![(0, 0), (1, 0), (0, 1)]);
        assert_eq!(
            cells_in_radius(at(16, 16), Fixed::from_num(26), &cells),
            vec![(0, 0), (1, 0), (0, 1), (2, 1)]
        );
        assert!(cells_in_radius(at(16, 16), Fixed::ZERO, &cells).is_empty());
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
