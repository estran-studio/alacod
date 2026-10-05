//! Générateur de cavernes (T1.6) : automate cellulaire déterministe, fonction pure.
//!
//! 1. Remplissage : bordure `Wall`, intérieur `Rock` avec la probabilité `fill_ratio`, sinon
//!    `Floor` (`RollbackRng`, jamais `rand`).
//! 2. `iterations` passes de la règle (voisinage de Moore, 8 cases, hors grille = solide) : une
//!    case `Floor` devient `Rock` si au moins `birth` voisins sont solides, une case `Rock`
//!    le reste si au moins `survive` voisins le sont, sinon devient `Floor`. La bordure reste
//!    `Wall`.
//! 3. Connexité : composantes 4-connexes de `Floor` ; la plus grande est gardée (égalité : la
//!    première rencontrée en balayant rangée par rangée depuis le bas), les autres deviennent
//!    `Rock`.
//! 4. Si le ratio de sol (sol / toutes les cases) sort de `[min_floor_ratio, 0.9]`, nouvel
//!    essai avec la suite du même RNG (au plus [`MAX_ATTEMPTS`]).

use std::collections::VecDeque;

use bevy_fixed::fixed_math::Fixed;
use bevy_fixed::rng::RollbackRng;
use serde::{Deserialize, Serialize};

use crate::grid::{CellGrid, CellKind};

/// Ratio de sol maximal accepté (au-delà, la caverne n'a presque plus de roche).
pub const MAX_FLOOR_RATIO: f64 = 0.9;
/// Nombre d'essais avant d'abandonner la contrainte de ratio (le dernier essai est rendu).
pub const MAX_ATTEMPTS: u32 = 64;
/// Écart minimal (distance de Tchebychev, en cases) entre deux `ZombieSpawn`.
pub const ZOMBIE_SPAWN_SPACING: u32 = 8;

/// Contenu d'une caverne, kind RON `Cave` (`caves/<nom>.ron`, `docs/conventions.md` §21).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaveConfig {
    /// Dimensions en cases (bordure `Wall` comprise).
    pub width: u32,
    pub height: u32,
    /// Probabilité qu'une case intérieure commence en `Rock` (`Fixed` en chaîne).
    pub fill_ratio: Fixed,
    pub iterations: u32,
    /// Seuils de la règle (voisins solides sur 8).
    pub birth: u32,
    pub survive: u32,
    /// Ratio de sol minimal après connexité (`Fixed` en chaîne).
    pub min_floor_ratio: Fixed,
    /// Nombre de `ZombieSpawn` placés.
    pub enemy_spawns: u32,
    /// Personnages de laboratoire (`CharacterSpawn`, équipe `enemies`) posés sur les points
    /// `ZombieSpawn`, à tour de rôle (vide par défaut) : un jeu sans vagues (testbed) peuple
    /// ainsi une caverne (bench `bench_cave`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characters: Vec<String>,
    /// Dégagement des points `ZombieSpawn`, en cases autour du point (D41, m1-d41-spawns-degages) :
    /// 1 par défaut (les 8 voisines, [`is_open`]). **Calculé** par le registre de contenu
    /// (`content::registry`) depuis le plus grand corps en jeu de `characters` ; un agent qui
    /// déborde d'une case et demie (plus de 24 px du centre de la case) en demande 2.
    #[serde(default = "default_spawn_clearance", skip_serializing_if = "is_default_spawn_clearance")]
    pub spawn_clearance: u32,
}

fn default_spawn_clearance() -> u32 {
    1
}

fn is_default_spawn_clearance(value: &u32) -> bool {
    *value == 1
}

/// Graine 64 bits repliée sur l'état 32 bits de `RollbackRng`.
fn rng_for(seed: u64) -> RollbackRng {
    RollbackRng::new((seed ^ (seed >> 32)) as u32)
}

/// Génère la grille d'une caverne. Même graine et même config ⇒ même grille.
pub fn generate(seed: u64, config: &CaveConfig) -> CellGrid {
    let mut rng = rng_for(seed);
    let mut grid = attempt(&mut rng, config);
    for _ in 1..MAX_ATTEMPTS {
        if ratio_ok(&grid, config) {
            break;
        }
        grid = attempt(&mut rng, config);
    }
    grid
}

/// Ratio de sol de la grille (sol / toutes les cases).
pub fn floor_ratio(grid: &CellGrid) -> f64 {
    if grid.cells.is_empty() {
        return 0.0;
    }
    grid.count(CellKind::Floor) as f64 / grid.cells.len() as f64
}

fn ratio_ok(grid: &CellGrid, config: &CaveConfig) -> bool {
    // Comparaison exacte en entiers : sol * 2^16 >= min * total (min en Fixed U16)
    let floor = grid.count(CellKind::Floor) as u64;
    let total = grid.cells.len() as u64;
    let min_bits = config.min_floor_ratio.to_bits().max(0) as u64;
    floor << 16 >= min_bits * total && floor * 10 <= total * 9
}

fn is_border(config: &CaveConfig, x: u32, y: u32) -> bool {
    x == 0 || y == 0 || x + 1 == config.width || y + 1 == config.height
}

fn attempt(rng: &mut RollbackRng, config: &CaveConfig) -> CellGrid {
    let (w, h) = (config.width, config.height);
    let mut grid = CellGrid::filled(w, h, CellKind::Floor);
    for y in 0..h {
        for x in 0..w {
            let kind = if is_border(config, x, y) {
                CellKind::Wall
            } else if rng.next_fixed() < config.fill_ratio {
                CellKind::Rock
            } else {
                CellKind::Floor
            };
            grid.set(x, y, kind);
        }
    }
    for _ in 0..config.iterations {
        let previous = grid.clone();
        for y in 1..h.saturating_sub(1) {
            for x in 1..w.saturating_sub(1) {
                let solid = solid_neighbours(&previous, x as i32, y as i32);
                let kind = match previous.get(x as i32, y as i32) {
                    Some(CellKind::Rock) if solid >= config.survive => CellKind::Rock,
                    Some(CellKind::Floor) if solid >= config.birth => CellKind::Rock,
                    _ => CellKind::Floor,
                };
                grid.set(x, y, kind);
            }
        }
    }
    keep_largest_floor_component(&mut grid);
    grid
}

fn solid_neighbours(grid: &CellGrid, x: i32, y: i32) -> u32 {
    let mut n = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) && grid.get(x + dx, y + dy).is_none_or(CellKind::is_solid) {
                n += 1;
            }
        }
    }
    n
}

const NEIGHBOURS_4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Distances de grille 4-connexes sur le sol depuis `start` (`u32::MAX` : inaccessible).
pub fn floor_distances(grid: &CellGrid, start: (u32, u32)) -> Vec<u32> {
    let mut dist = vec![u32::MAX; grid.cells.len()];
    let index = |x: u32, y: u32| (y * grid.width + x) as usize;
    if grid.get(start.0 as i32, start.1 as i32) != Some(CellKind::Floor) {
        return dist;
    }
    dist[index(start.0, start.1)] = 0;
    let mut queue = VecDeque::from([start]);
    while let Some((x, y)) = queue.pop_front() {
        let d = dist[index(x, y)];
        for (dx, dy) in NEIGHBOURS_4 {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if grid.get(nx, ny) == Some(CellKind::Floor) {
                let i = index(nx as u32, ny as u32);
                if dist[i] == u32::MAX {
                    dist[i] = d + 1;
                    queue.push_back((nx as u32, ny as u32));
                }
            }
        }
    }
    dist
}

/// Composantes 4-connexes de sol, dans l'ordre de leur première case (balayage depuis le bas).
pub fn floor_components(grid: &CellGrid) -> Vec<Vec<(u32, u32)>> {
    let mut seen = vec![false; grid.cells.len()];
    let mut components = Vec::new();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let i = (y * grid.width + x) as usize;
            if seen[i] || grid.cells[i] != CellKind::Floor {
                continue;
            }
            seen[i] = true;
            let mut component = vec![(x, y)];
            let mut queue = VecDeque::from([(x, y)]);
            while let Some((cx, cy)) = queue.pop_front() {
                for (dx, dy) in NEIGHBOURS_4 {
                    let (nx, ny) = (cx as i32 + dx, cy as i32 + dy);
                    if grid.get(nx, ny) == Some(CellKind::Floor) {
                        let j = (ny as u32 * grid.width + nx as u32) as usize;
                        if !seen[j] {
                            seen[j] = true;
                            component.push((nx as u32, ny as u32));
                            queue.push_back((nx as u32, ny as u32));
                        }
                    }
                }
            }
            components.push(component);
        }
    }
    components
}

fn keep_largest_floor_component(grid: &mut CellGrid) {
    let components = floor_components(grid);
    let Some(largest) = components
        .iter()
        .enumerate()
        .max_by_key(|(i, c)| (c.len(), std::cmp::Reverse(*i)))
        .map(|(i, _)| i)
    else {
        return;
    };
    for (i, component) in components.iter().enumerate() {
        if i != largest {
            for &(x, y) in component {
                grid.set(x, y, CellKind::Rock);
            }
        }
    }
}

/// Points d'intérêt déterministes d'une caverne, en cases de grille.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CavePoints {
    /// Cases `Floor` les plus proches du centre, dans l'ordre (index de `PlayerSpawn`).
    pub player_spawns: Vec<(u32, u32)>,
    /// Cases `Floor` les plus éloignées (distance de grille) du premier `PlayerSpawn`,
    /// espacées d'au moins [`ZOMBIE_SPAWN_SPACING`] cases.
    pub zombie_spawns: Vec<(u32, u32)>,
}

/// Case de sol dont les 8 voisines sont aussi du sol : un corps de personnage (20 × 20, décalé
/// vers le bas) y apparaît sans chevaucher de mur, sinon chaque déplacement serait refusé.
pub fn is_open(grid: &CellGrid, x: u32, y: u32) -> bool {
    is_open_within(grid, x, y, 1)
}

/// Case de sol dont toutes les cases à `radius` cases ou moins (distance de Tchebychev) sont du
/// sol : [`is_open`] pour `radius` 1 ; un corps plus grand en demande davantage (D41).
pub fn is_open_within(grid: &CellGrid, x: u32, y: u32, radius: u32) -> bool {
    let r = radius as i32;
    (-r..=r).all(|dy| {
        (-r..=r).all(|dx| grid.get(x as i32 + dx, y as i32 + dy) == Some(CellKind::Floor))
    })
}

/// Les `players` cases **dégagées** ([`is_open`]) les plus proches du centre (distance
/// euclidienne au carré, puis y, puis x), puis les `ZombieSpawn`, dégagées de
/// `enemy_clearance` cases ([`is_open_within`], D41 : `CaveConfig::spawn_clearance`) ; avec 1,
/// exactement les points d'avant.
pub fn points_of_interest(
    grid: &CellGrid,
    players: usize,
    enemy_spawns: u32,
    enemy_clearance: u32,
) -> CavePoints {
    let (w, h) = (grid.width as i64, grid.height as i64);
    let mut floor: Vec<(u32, u32)> = (0..grid.height)
        .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
        .filter(|&(x, y)| is_open(grid, x, y))
        .collect();
    // Centre en coordonnées doublées : pas de demi-case
    floor.sort_by_key(|&(x, y)| {
        let dx = 2 * x as i64 - (w - 1);
        let dy = 2 * y as i64 - (h - 1);
        (dx * dx + dy * dy, y, x)
    });
    let player_spawns: Vec<(u32, u32)> = floor.iter().take(players).copied().collect();

    let mut zombie_spawns: Vec<(u32, u32)> = Vec::new();
    if let Some(&origin) = player_spawns.first() {
        let dist = floor_distances(grid, origin);
        let mut candidates: Vec<(u32, u32, u32)> = floor
            .iter()
            .map(|&(x, y)| (dist[(y * grid.width + x) as usize], x, y))
            .filter(|(d, _, _)| *d != u32::MAX)
            .filter(|(_, x, y)| enemy_clearance <= 1 || is_open_within(grid, *x, *y, enemy_clearance))
            .collect();
        candidates.sort_by_key(|&(d, x, y)| (std::cmp::Reverse(d), y, x));
        for (_, x, y) in candidates {
            if zombie_spawns.len() as u32 >= enemy_spawns {
                break;
            }
            let spaced = zombie_spawns
                .iter()
                .all(|&(zx, zy)| zx.abs_diff(x).max(zy.abs_diff(y)) >= ZOMBIE_SPAWN_SPACING);
            if spaced {
                zombie_spawns.push((x, y));
            }
        }
    }
    CavePoints {
        player_spawns,
        zombie_spawns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn petite() -> CaveConfig {
        CaveConfig {
            width: 48,
            height: 32,
            fill_ratio: Fixed::from_num(0.45),
            iterations: 4,
            birth: 5,
            survive: 4,
            min_floor_ratio: Fixed::from_num(0.3),
            enemy_spawns: 4,
            characters: vec![],
            spawn_clearance: 1,
        }
    }

    #[test]
    fn mille_graines_connexes_bordees_et_dans_le_ratio() {
        let config = petite();
        let min = config.min_floor_ratio.to_num::<f64>();
        for seed in 0..1000u64 {
            let grid = generate(seed, &config);
            assert_eq!((grid.width, grid.height), (48, 32));
            assert_eq!(
                floor_components(&grid).len(),
                1,
                "graine {seed} : sol non connexe"
            );
            for y in 0..grid.height {
                for x in 0..grid.width {
                    if is_border(&config, x, y) {
                        assert_eq!(grid.get(x as i32, y as i32), Some(CellKind::Wall));
                    } else {
                        assert_ne!(grid.get(x as i32, y as i32), Some(CellKind::Wall));
                    }
                }
            }
            let ratio = floor_ratio(&grid);
            assert!(
                (min..=MAX_FLOOR_RATIO).contains(&ratio),
                "graine {seed} : ratio {ratio}"
            );
            let points = points_of_interest(&grid, 4, config.enemy_spawns, config.spawn_clearance);
            assert_eq!(points.player_spawns.len(), 4, "graine {seed}");
            assert!(!points.zombie_spawns.is_empty(), "graine {seed}");
            for (x, y) in points.player_spawns.iter().chain(&points.zombie_spawns) {
                assert!(
                    is_open(&grid, *x, *y),
                    "graine {seed} : ({x}, {y}) pas dégagée"
                );
            }
        }
    }

    #[test]
    fn meme_graine_meme_grille_graines_differentes_grilles_differentes() {
        let config = petite();
        assert_eq!(generate(7, &config), generate(7, &config));
        let grids: std::collections::BTreeSet<Vec<String>> = (0..100u64)
            .map(|seed| generate(seed, &config).rows_top_down())
            .collect();
        assert_eq!(grids.len(), 100);
    }

    #[test]
    fn zombie_spawns_espaces_et_loin_du_joueur() {
        let config = petite();
        let grid = generate(3, &config);
        let points = points_of_interest(&grid, 4, config.enemy_spawns, config.spawn_clearance);
        let dist = floor_distances(&grid, points.player_spawns[0]);
        let far = points
            .zombie_spawns
            .iter()
            .map(|&(x, y)| dist[(y * grid.width + x) as usize])
            .min()
            .unwrap();
        // La plus grande distance parmi les cases dégagées (candidates)
        let max_dist = (0..grid.height)
            .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
            .filter(|&(x, y)| is_open(&grid, x, y))
            .map(|(x, y)| dist[(y * grid.width + x) as usize])
            .filter(|d| *d != u32::MAX)
            .max()
            .unwrap();
        assert_eq!(
            dist[(points.zombie_spawns[0].1 * grid.width + points.zombie_spawns[0].0) as usize],
            max_dist
        );
        assert!(far > 0);
        for (i, a) in points.zombie_spawns.iter().enumerate() {
            for b in &points.zombie_spawns[i + 1..] {
                assert!(a.0.abs_diff(b.0).max(a.1.abs_diff(b.1)) >= ZOMBIE_SPAWN_SPACING);
            }
        }
    }

    #[test]
    fn config_ron() {
        let config: CaveConfig = ron::from_str(
            r#"(width: 48, height: 32, fill_ratio: "0.45", iterations: 4, birth: 5, survive: 4,
                min_floor_ratio: "0.3", enemy_spawns: 4)"#,
        )
        .unwrap();
        assert_eq!(config, petite());
    }

    /// D41 : un dégagement de 2 cases ne garde que des points entourés de 5 × 5 cases de sol ;
    /// le dégagement 1 (défaut) donne exactement les points d'avant (`is_open`).
    #[test]
    fn degagement_des_points_ennemis() {
        let config = petite();
        let grid = generate(123456, &config);
        let base = points_of_interest(&grid, 4, config.enemy_spawns, 1);
        let large = points_of_interest(&grid, 4, config.enemy_spawns, 2);
        assert_eq!(base.player_spawns, large.player_spawns, "joueurs : un seul dégagement");
        assert!(!large.zombie_spawns.is_empty());
        for &(x, y) in &large.zombie_spawns {
            assert!(is_open_within(&grid, x, y, 2), "({x}, {y})");
        }
        assert!(base
            .zombie_spawns
            .iter()
            .all(|&(x, y)| is_open(&grid, x, y)));
        // Le dégagement 2 exclut au moins un point que le dégagement 1 accepte (petite caverne
        // dense), sinon le test ne prouve rien.
        let all_open_1: Vec<_> = (0..grid.height)
            .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
            .filter(|&(x, y)| is_open(&grid, x, y))
            .collect();
        assert!(all_open_1
            .iter()
            .any(|&(x, y)| !is_open_within(&grid, x, y, 2)));
    }

    #[test]
    fn degagement_absent_du_ron_vaut_un() {
        let config: CaveConfig = ron::from_str(
            r#"(width: 48, height: 32, fill_ratio: "0.45", iterations: 4, birth: 5,
                survive: 4, min_floor_ratio: "0.3", enemy_spawns: 4)"#,
        )
        .unwrap();
        assert_eq!(config.spawn_clearance, 1);
    }
}
