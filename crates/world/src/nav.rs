//! Règles de passage de la navigation des ennemis (D48, m1-d48-ennemis-hors-champ) : une seule
//! définition pour le champ de flux de `game` (`FlowFieldCache`, Dijkstra) et pour
//! l'accessibilité des points d'apparition d'une caverne ([`nav_distances`]). `blocked(x, y)` dit
//! si une case est un obstacle pour le profil de l'agent ; `large` est le gabarit grand
//! (`AgentSize::Large` de `game` : corps de plus de 20 px).

use std::collections::VecDeque;

use crate::grid::CellGrid;

/// Voisines en 8-connexité : les quatre orthogonales puis les quatre diagonales (même ordre que
/// `GridPos::neighbors_8` de `game`).
pub const NEIGHBOURS_8: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
];

/// Case bloquée pour l'agent : la case elle-même, ou, pour un gabarit grand (centre à au moins
/// une case de tout obstacle), l'une de ses 8 voisines.
pub fn blocked_for(blocked: impl Fn(i32, i32) -> bool, x: i32, y: i32, large: bool) -> bool {
    blocked(x, y) || (large && NEIGHBOURS_8.iter().any(|&(dx, dy)| blocked(x + dx, y + dy)))
}

/// Couloir d'une case : bloqué des deux côtés sur un axe. Exclu pour un gabarit petit (un
/// couloir de deux cases suffit) ; un gabarit grand l'exclut déjà par [`blocked_for`].
pub fn too_narrow(blocked: impl Fn(i32, i32) -> bool, x: i32, y: i32) -> bool {
    (blocked(x - 1, y) && blocked(x + 1, y)) || (blocked(x, y - 1) && blocked(x, y + 1))
}

/// Case où l'agent ne peut pas entrer : [`blocked_for`], ou [`too_narrow`] pour un gabarit petit.
pub fn impassable(blocked: impl Fn(i32, i32) -> bool, x: i32, y: i32, large: bool) -> bool {
    blocked_for(&blocked, x, y, large) || (!large && too_narrow(&blocked, x, y))
}

/// Un pas diagonal de `(x, y)` vers `(x + dx, y + dy)` coupe un coin : l'une des deux
/// orthogonales est bloquée pour l'agent (son collider heurterait l'angle du mur).
pub fn diagonal_cuts_corner(
    blocked: impl Fn(i32, i32) -> bool,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    large: bool,
) -> bool {
    blocked_for(&blocked, x + dx, y, large) || blocked_for(&blocked, x, y + dy, large)
}

/// D55 : le pas orthogonal `(x, y)` → `(x + dx, y + dy)` traverse une chicane : sur l'axe
/// perpendiculaire, la case de départ est bloquée d'un côté et celle d'arrivée du côté opposé. Les
/// deux coins sont distants d'une seule case (16 px), trop peu pour un corps de plus de 16 px :
/// le collider accroche un coin sans jamais glisser (graine 19 de throne). Pour un gabarit petit
/// (20 px) ; un gabarit grand écarte déjà ces cases par [`blocked_for`].
pub fn chicane_step(blocked: impl Fn(i32, i32) -> bool, x: i32, y: i32, dx: i32, dy: i32) -> bool {
    let (px, py) = (-dy, dx);
    let (tx, ty) = (x + dx, y + dy);
    (blocked(x + px, y + py) && blocked(tx - px, ty - py))
        || (blocked(x - px, y - py) && blocked(tx + px, ty + py))
}

/// Distances de navigation (en pas, 8-connexité) depuis `sources` sur une grille de caverne, avec
/// les règles du champ de flux : roche et bordure bloquent, [`impassable`], pas de coin coupé.
/// Les sources (les points des joueurs) comptent même bloquées, comme les cibles du champ.
/// `u32::MAX` : case que le champ de flux de ce gabarit n'atteint pas.
pub fn nav_distances(grid: &CellGrid, sources: &[(u32, u32)], large: bool) -> Vec<u32> {
    let blocked = |x: i32, y: i32| grid.get(x, y).is_none_or(|kind| kind.is_solid());
    let mut dist = vec![u32::MAX; grid.cells.len()];
    let index = |x: i32, y: i32| (y as u32 * grid.width + x as u32) as usize;
    let mut queue = VecDeque::new();
    for &(x, y) in sources {
        let (x, y) = (x as i32, y as i32);
        if grid.get(x, y).is_some() && dist[index(x, y)] == u32::MAX {
            dist[index(x, y)] = 0;
            queue.push_back((x, y));
        }
    }
    while let Some((x, y)) = queue.pop_front() {
        let d = dist[index(x, y)];
        for (dx, dy) in NEIGHBOURS_8 {
            let (nx, ny) = (x + dx, y + dy);
            if grid.get(nx, ny).is_none() || dist[index(nx, ny)] != u32::MAX {
                continue;
            }
            if impassable(blocked, nx, ny, large) {
                continue;
            }
            if dx != 0 && dy != 0 && diagonal_cuts_corner(blocked, x, y, dx, dy, large) {
                continue;
            }
            if (dx == 0 || dy == 0) && !large && chicane_step(blocked, x, y, dx, dy) {
                continue;
            }
            dist[index(nx, ny)] = d + 1;
            queue.push_back((nx, ny));
        }
    }
    dist
}

/// Case de portail d'un niveau de caverne (D48, graine 53 de throne : barycentre des points des
/// joueurs dans un mur) : `target` si le champ de flux du gabarit l'atteint depuis `sources`, sinon
/// la case de sol atteinte la plus proche (distance euclidienne au carré, puis y, puis x) ; `None`
/// si aucune case n'est atteinte.
pub fn nearest_reached_cell(
    grid: &CellGrid,
    sources: &[(u32, u32)],
    target: (i32, i32),
    large: bool,
) -> Option<(u32, u32)> {
    let dist = nav_distances(grid, sources, large);
    let reached = |x: u32, y: u32| {
        dist[(y * grid.width + x) as usize] != u32::MAX
            && grid
                .get(x as i32, y as i32)
                .is_some_and(|kind| !kind.is_solid())
    };
    if target.0 >= 0 && target.1 >= 0 && grid.get(target.0, target.1).is_some() {
        if reached(target.0 as u32, target.1 as u32) {
            return Some((target.0 as u32, target.1 as u32));
        }
    }
    (0..grid.height)
        .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
        .filter(|&(x, y)| reached(x, y))
        .min_by_key(|&(x, y)| {
            let (dx, dy) = (x as i64 - target.0 as i64, y as i64 - target.1 as i64);
            (dx * dx + dy * dy, y, x)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::CellKind;

    /// Grille depuis des rangées de texte (`#` roche, `.` sol), la première rangée en haut.
    fn grid(rows: &[&str]) -> CellGrid {
        let (w, h) = (rows[0].len() as u32, rows.len() as u32);
        let mut grid = CellGrid::filled(w, h, CellKind::Floor);
        for (row, line) in rows.iter().enumerate() {
            let y = h - 1 - row as u32;
            for (x, c) in line.chars().enumerate() {
                if c == '#' {
                    grid.set(x as u32, y, CellKind::Rock);
                }
            }
        }
        grid
    }

    /// Graine 162 de throne en miniature : une poche reliée au reste par un couloir d'une case
    /// est ouverte (4-connexe) mais hors du champ d'un gabarit petit.
    #[test]
    fn couloir_d_une_case_coupe_la_poche() {
        let g = grid(&[
            "##########",
            "#...##...#",
            "#........#",
            "#...##...#",
            "##########",
        ]);
        let small = nav_distances(&g, &[(7, 2)], false);
        let i = |x: u32, y: u32| (y * g.width + x) as usize;
        assert_eq!(small[i(4, 2)], u32::MAX, "couloir (4, 2) d'une case");
        assert_eq!(small[i(2, 2)], u32::MAX, "poche derrière le couloir");
        assert_ne!(small[i(6, 3)], u32::MAX);
    }

    /// Un gabarit grand n'atteint que les cases dont les 8 voisines sont libres ; un petit passe
    /// un couloir de deux cases.
    #[test]
    fn gabarit_grand_exige_les_huit_voisines() {
        let g = grid(&[
            "###########",
            "#....#....#",
            "#.........#",
            "#.........#",
            "#....#....#",
            "###########",
        ]);
        let i = |x: u32, y: u32| (y * g.width + x) as usize;
        let small = nav_distances(&g, &[(8, 2)], false);
        let large = nav_distances(&g, &[(8, 2)], true);
        assert_ne!(small[i(2, 2)], u32::MAX);
        assert_eq!(
            large[i(2, 2)],
            u32::MAX,
            "passage de deux cases fermé au gabarit grand"
        );
        assert_ne!(large[i(8, 2)], u32::MAX);
    }

    /// Portail : une cible dans la roche est ramenée sur la case atteinte la plus proche ; une
    /// cible atteinte reste telle quelle.
    #[test]
    fn portail_ramene_sur_une_case_atteinte() {
        let g = grid(&[
            "#########",
            "#...#...#",
            "#...#...#",
            "#.......#",
            "#########",
        ]);
        let sources = [(2, 2), (6, 2)];
        // (4, 3) est de la roche ; (3, 3) et (5, 3) à égale distance : la plus petite x gagne.
        assert_eq!(
            nearest_reached_cell(&g, &sources, (4, 3), false),
            Some((3, 3))
        );
        assert_eq!(
            nearest_reached_cell(&g, &sources, (2, 3), false),
            Some((2, 3))
        );
    }
}
