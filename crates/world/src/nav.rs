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
    blocked(x, y)
        || (large
            && NEIGHBOURS_8
                .iter()
                .any(|&(dx, dy)| blocked(x + dx, y + dy)))
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
            dist[index(nx, ny)] = d + 1;
            queue.push_back((nx, ny));
        }
    }
    dist
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
        assert_eq!(large[i(2, 2)], u32::MAX, "passage de deux cases fermé au gabarit grand");
        assert_ne!(large[i(8, 2)], u32::MAX);
    }
}
