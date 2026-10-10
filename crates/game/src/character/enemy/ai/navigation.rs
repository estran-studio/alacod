//! Flow Field Navigation System
//!
//! This module provides a shared pathfinding solution for hordes of enemies.
//! Uses Dijkstra over the map bounds, with extra cost near walls.
//! O(1) direction lookups per enemy after computation.
//!
//! Blocking cells: LDtk IntGrid walls, closed doors (door entities that still have a
//! collider) and, depending on the profile, obstacles such as intact windows. The field is
//! rebuilt when the target moves *or* when blocking cells change (door opened, window
//! broken or repaired). Diagonal steps never cut a wall corner.
//!
//! IMPORTANT: Uses BTreeMap/BTreeSet for deterministic iteration order (GGRS rollback).

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use combat::downed::Downed;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::{frame::FrameCount, net_id::GgrsNetId};

use crate::character::player::Player;
use crate::collider::{Collider, ColliderShape};
use map::game::entity::map::door::DoorComponent;

use super::obstacle::{Obstacle, ObstacleType};

/// Grid cell size in fixed-point units (matches LDtk tile size for 1:1 mapping)
pub const GRID_CELL_SIZE: i32 = 16;

/// Grid position for flow field calculations
/// Implements Ord for deterministic BTreeMap/BTreeSet ordering
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert from FixedVec2 world position to grid position
    /// Uses div_euclid for proper floor division with negative coordinates
    pub fn from_fixed(pos: fixed_math::FixedVec2) -> Self {
        Self {
            x: pos.x.to_num::<i32>().div_euclid(GRID_CELL_SIZE),
            y: pos.y.to_num::<i32>().div_euclid(GRID_CELL_SIZE),
        }
    }

    /// Convert to world position (center of cell)
    pub fn to_fixed(self) -> fixed_math::FixedVec2 {
        let half_cell = GRID_CELL_SIZE / 2;
        fixed_math::FixedVec2::new(
            fixed_math::Fixed::from_num(self.x * GRID_CELL_SIZE + half_cell),
            fixed_math::Fixed::from_num(self.y * GRID_CELL_SIZE + half_cell),
        )
    }

    /// Get 4-directional neighbors
    pub fn neighbors_4(&self) -> [GridPos; 4] {
        [
            GridPos::new(self.x + 1, self.y),
            GridPos::new(self.x - 1, self.y),
            GridPos::new(self.x, self.y + 1),
            GridPos::new(self.x, self.y - 1),
        ]
    }

    /// Get 8-directional neighbors (including diagonals)
    pub fn neighbors_8(&self) -> [GridPos; 8] {
        [
            GridPos::new(self.x + 1, self.y),
            GridPos::new(self.x - 1, self.y),
            GridPos::new(self.x, self.y + 1),
            GridPos::new(self.x, self.y - 1),
            GridPos::new(self.x + 1, self.y + 1),
            GridPos::new(self.x - 1, self.y + 1),
            GridPos::new(self.x + 1, self.y - 1),
            GridPos::new(self.x - 1, self.y - 1),
        ]
    }

    /// Manhattan distance to another grid position
    pub fn manhattan_distance(&self, other: &GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

/// Navigation profile determines which obstacles block an enemy
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Default,
    Reflect,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
)]
pub enum NavProfile {
    /// Respects all obstacles (walls, windows, barricades, water, pits)
    #[default]
    Ground,
    /// Ignores breakable obstacles for pathfinding (still attacks them)
    GroundBreaker,
    /// Ignores water and pits (flying enemies)
    Flying,
    /// Ignores everything except solid walls (ghosts)
    Phasing,
}

/// Profil du champ de flux par défaut : celui des zombies et de tout ennemi de gabarit petit
/// qui traverse les obstacles cassables. Lu par le diagnostic de soft-lock pour l'en-tête
/// (`scenario::softlock`, D42).
pub const MOVEMENT_FLOW_PROFILE: NavProfile = NavProfile::GroundBreaker;

/// Gabarit d'un agent pour la navigation (D41) : le dégagement qu'il lui faut autour des murs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum AgentSize {
    /// Corps d'au plus [`SMALL_AGENT_MAX`] de large (zombies, ennemis de 20 px) : le champ
    /// historique, un couloir de deux cases suffit (`FlowFieldCache::is_too_narrow`).
    #[default]
    Small,
    /// Corps plus large (boss) : centre à au moins une case de tout obstacle, couloirs de
    /// trois cases.
    Large,
}

/// Largeur (ou hauteur) de corps au-delà de laquelle un agent est [`AgentSize::Large`].
pub const SMALL_AGENT_MAX: i32 = 20;

impl AgentSize {
    pub fn of(body: &AgentBody) -> Self {
        let max = fixed_math::Fixed::from_num(SMALL_AGENT_MAX);
        if body.left + body.right > max || body.down + body.up > max {
            AgentSize::Large
        } else {
            AgentSize::Small
        }
    }
}

/// Clé d'un champ de flux (D41 + D38) : profil d'obstacles et gabarit de l'agent. Un ennemi
/// suit le champ de **sa** clé ; seules les clés d'ennemis qui se déplacent sont construites.
///
/// `Hash` écrit à la main : une clé de gabarit petit se hache exactement comme son profil seul
/// (`FlowFieldCache` est sous checksum ; avant ce chantier, `layers` était indexé par
/// `NavProfile`) : un cache qui ne porte que le champ historique garde le même checksum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NavKey {
    pub profile: NavProfile,
    pub size: AgentSize,
}

impl std::hash::Hash for NavKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.profile.hash(state);
        if self.size != AgentSize::Small {
            self.size.hash(state);
        }
    }
}

impl NavKey {
    pub const fn new(profile: NavProfile, size: AgentSize) -> Self {
        Self { profile, size }
    }

    /// Clé d'un agent : son profil (`EnemyAiConfig::nav_profile`) et le gabarit de son corps.
    pub fn for_agent(profile: NavProfile, body: &AgentBody) -> Self {
        Self::new(profile, AgentSize::of(body))
    }
}

/// Le champ historique, toujours construit (zombies, ennemis de 20 px qui cassent les
/// fenêtres).
pub const MOVEMENT_FLOW_KEY: NavKey = NavKey::new(MOVEMENT_FLOW_PROFILE, AgentSize::Small);

impl NavProfile {
    /// Returns true if this profile can pass through the given obstacle type
    pub fn can_pass(&self, obstacle_type: ObstacleType) -> bool {
        match self {
            NavProfile::Ground => false,
            NavProfile::GroundBreaker => obstacle_type.is_breakable(),
            NavProfile::Flying => matches!(obstacle_type, ObstacleType::Water | ObstacleType::Pit),
            NavProfile::Phasing => obstacle_type != ObstacleType::Wall,
        }
    }
}

/// A single flow field for a specific navigation profile
/// Uses BTreeMap for deterministic iteration (GGRS rollback compatibility)
#[derive(Clone, Debug, Default, Hash)]
pub struct FlowField {
    /// For each cell, the next cell to move to (toward target)
    pub directions: BTreeMap<GridPos, GridPos>,
    /// Cost to reach target from each cell
    pub costs: BTreeMap<GridPos, u32>,
    /// For each cell, the target (index in `FlowFieldCache::target_ids`) it leads to:
    /// the closest one along the path
    pub owners: BTreeMap<GridPos, usize>,
}

impl FlowField {
    /// Get the direction to move from a given position
    pub fn get_direction(&self, pos: GridPos) -> Option<GridPos> {
        self.directions.get(&pos).copied()
    }

    /// Get the world-space direction vector from a given position
    /// Uses actual enemy position (not cell center) for smoother movement
    pub fn get_direction_vector(
        &self,
        pos: fixed_math::FixedVec2,
    ) -> Option<fixed_math::FixedVec2> {
        let grid_pos = GridPos::from_fixed(pos);
        let next_pos = self.get_direction(grid_pos)?;

        // Direction from actual position to next cell center (smoother than cell-to-cell)
        let next_world = next_pos.to_fixed();
        let direction = next_world - pos;

        if direction.length_squared() > fixed_math::FixedWide::ZERO {
            Some(direction.normalize_or_zero())
        } else {
            // At the next cell center - check if there's a further cell to move to
            if let Some(further_pos) = self.get_direction(next_pos) {
                let further_world = further_pos.to_fixed();
                let further_dir = further_world - pos;
                if further_dir.length_squared() > fixed_math::FixedWide::ZERO {
                    return Some(further_dir.normalize_or_zero());
                }
            }
            None
        }
    }

    /// Get alternative directions from neighboring cells (for escaping corners)
    /// Returns directions sorted by cost (lowest cost = closest to target)
    pub fn get_neighbor_directions(
        &self,
        pos: fixed_math::FixedVec2,
    ) -> Vec<fixed_math::FixedVec2> {
        let grid_pos = GridPos::from_fixed(pos);
        let mut directions = Vec::new();

        // Check all 8 neighboring cells for flow field entries
        for neighbor in grid_pos.neighbors_8() {
            if let Some(next_pos) = self.get_direction(neighbor) {
                // Get the cost of this neighbor's path
                let cost = self.costs.get(&neighbor).copied().unwrap_or(u32::MAX);
                let neighbor_world = neighbor.to_fixed();
                let dir = (neighbor_world - pos).normalize_or_zero();
                if dir.length_squared() > fixed_math::FixedWide::ZERO {
                    directions.push((cost, dir));
                }
            }
        }

        // Sort by cost (lowest first = closest to target)
        directions.sort_by_key(|(cost, _)| *cost);
        directions.into_iter().map(|(_, dir)| dir).collect()
    }

    /// T1.4 (`KeepDistance`, `Flee`) : direction de recul — vers la case voisine couverte
    /// de **coût le plus élevé** (la plus loin des cibles par le chemin), si elle est plus
    /// coûteuse que la case courante ; départage par `GridPos` (le plus petit). `None` : aucune
    /// case voisine ne s'éloigne (coin), ou position hors du champ.
    pub fn retreat_direction(&self, pos: fixed_math::FixedVec2) -> Option<fixed_math::FixedVec2> {
        let here = GridPos::from_fixed(pos);
        let here_cost = *self.costs.get(&here)?;
        let mut best: Option<(u32, GridPos)> = None;
        for neighbor in here.neighbors_8() {
            let Some(&cost) = self.costs.get(&neighbor) else {
                continue;
            };
            if cost <= here_cost {
                continue;
            }
            let better = match best {
                None => true,
                Some((best_cost, best_pos)) => {
                    cost > best_cost || (cost == best_cost && neighbor < best_pos)
                }
            };
            if better {
                best = Some((cost, neighbor));
            }
        }
        let (_, cell) = best?;
        let direction = (cell.to_fixed() - pos).normalize_or_zero();
        (direction != fixed_math::FixedVec2::ZERO).then_some(direction)
    }

    /// Find the nearest cell that has flow field coverage
    /// Used when an enemy is outside the flow field to find a path back in
    /// Returns the direction to move toward the nearest covered cell
    pub fn find_nearest_covered_cell(
        &self,
        pos: fixed_math::FixedVec2,
        max_search: i32,
    ) -> Option<fixed_math::FixedVec2> {
        let grid_pos = GridPos::from_fixed(pos);

        // First check immediate neighbors (most common case)
        for neighbor in grid_pos.neighbors_8() {
            if self.directions.contains_key(&neighbor) {
                let neighbor_world = neighbor.to_fixed();
                let dir = (neighbor_world - pos).normalize_or_zero();
                if dir.length_squared() > fixed_math::FixedWide::ZERO {
                    return Some(dir);
                }
            }
        }

        // Expand search in rings up to max_search distance
        let mut best_cell: Option<(GridPos, u32)> = None; // (cell, cost to target)

        for radius in 2..=max_search {
            // Check cells at this manhattan distance ring
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    // Only check cells on the ring perimeter
                    if dx.abs() != radius && dy.abs() != radius {
                        continue;
                    }

                    let check_pos = GridPos::new(grid_pos.x + dx, grid_pos.y + dy);

                    if let Some(&cost) = self.costs.get(&check_pos) {
                        // Found a covered cell - prefer the one with lowest cost (closest to target)
                        match best_cell {
                            None => best_cell = Some((check_pos, cost)),
                            Some((_, best_cost)) if cost < best_cost => {
                                best_cell = Some((check_pos, cost));
                            }
                            _ => {}
                        }
                    }
                }
            }

            // If we found cells at this radius, return direction to best one
            if let Some((cell, _)) = best_cell {
                let cell_world = cell.to_fixed();
                let dir = (cell_world - pos).normalize_or_zero();
                if dir.length_squared() > fixed_math::FixedWide::ZERO {
                    return Some(dir);
                }
            }
        }

        None
    }
}

/// How far an agent's collider extends from its position on each side (the collider
/// may be offset, e.g. toward the feet).
#[derive(Clone, Copy, Debug)]
pub struct AgentBody {
    pub left: fixed_math::Fixed,
    pub right: fixed_math::Fixed,
    pub down: fixed_math::Fixed,
    pub up: fixed_math::Fixed,
}

impl AgentBody {
    pub fn from_collider(collider: &Collider) -> Self {
        let (half_w, half_h) = match &collider.shape {
            ColliderShape::Circle { radius } => (*radius, *radius),
            ColliderShape::Rectangle { width, height } => (
                *width / fixed_math::new(2.0),
                *height / fixed_math::new(2.0),
            ),
        };
        Self {
            left: half_w - collider.offset.x,
            right: half_w + collider.offset.x,
            down: half_h - collider.offset.y,
            up: half_h + collider.offset.y,
        }
    }
}

/// Level grid information for coordinate conversion
#[derive(Clone, Default, Debug, Hash)]
pub struct LevelGridInfo {
    pub offset_x: i32,
    pub offset_y: i32,
    pub height_tiles: i32,
    pub width_tiles: i32,
}

/// Cache of flow fields for different navigation profiles
/// Uses BTreeMap/BTreeSet for deterministic iteration (GGRS rollback compatibility)
#[derive(Resource, Default, Clone, Debug, Hash)]
pub struct FlowFieldCache {
    /// Target position (player) at last calculation
    pub target_pos: GridPos,
    /// Cells of all targets (players), in net_id order: the field leads to the closest one
    pub targets: Vec<GridPos>,
    /// Net ids of the targets, same order as `targets`
    pub target_ids: Vec<usize>,
    /// Frame when last updated
    pub last_update_frame: u32,
    /// Update interval in frames
    pub update_interval: u32,
    /// Champs de flux par clé (profil, gabarit) **canonique** (voir
    /// [`FlowFieldCache::canonical`]) ; [`MOVEMENT_FLOW_KEY`] est toujours présent.
    pub layers: BTreeMap<NavKey, FlowField>,
    /// Blocked cells per obstacle type (for building flow fields)
    pub blocked_cells: BTreeMap<ObstacleType, BTreeSet<GridPos>>,
    /// All permanently blocked cells (walls) - computed from IntGrid + dynamic walls
    pub wall_cells: BTreeSet<GridPos>,
    /// Wall cells directly from LDtk IntGrid (1:1 tile mapping, immutable after load)
    pub intgrid_wall_cells: BTreeSet<GridPos>,
    /// Level grid info for coordinate conversion
    pub level_info: Option<LevelGridInfo>,
    /// Number of wall entities at last rebuild (to detect when walls are added)
    pub last_wall_entity_count: usize,
}

impl FlowFieldCache {
    pub fn new() -> Self {
        Self {
            target_pos: GridPos::default(),
            targets: Vec::new(),
            target_ids: Vec::new(),
            last_update_frame: 0,
            update_interval: 30, // Update every 30 frames (~2 times per second at 60 FPS)
            layers: BTreeMap::new(),
            blocked_cells: BTreeMap::new(),
            wall_cells: BTreeSet::new(),
            intgrid_wall_cells: BTreeSet::new(),
            level_info: None,
            last_wall_entity_count: 0,
        }
    }

    /// Clé du champ réellement construit pour `key` : un profil qui, sur la carte courante,
    /// voit exactement les mêmes obstacles que [`MOVEMENT_FLOW_PROFILE`] (aucune cellule d'un
    /// obstacle que l'un passe et l'autre pas : pas de fenêtre ni de barricade pour `Ground`)
    /// partage son champ. Ainsi un ennemi `Ground` dans une caverne suit le champ historique,
    /// et seules les cartes à obstacles cassables construisent un champ `Ground` à part.
    pub fn canonical(&self, key: NavKey) -> NavKey {
        let same_obstacles = self.blocked_cells.iter().all(|(obstacle_type, cells)| {
            cells.is_empty()
                || key.profile.can_pass(*obstacle_type)
                    == MOVEMENT_FLOW_PROFILE.can_pass(*obstacle_type)
        });
        if same_obstacles {
            NavKey::new(MOVEMENT_FLOW_PROFILE, key.size)
        } else {
            key
        }
    }

    /// Champ de flux à suivre pour un agent de clé `key`.
    pub fn get_flow_field(&self, key: NavKey) -> Option<&FlowField> {
        self.layers.get(&self.canonical(key))
    }

    /// Case bloquée pour un agent de clé `key` : obstacle du profil, et pour un gabarit grand,
    /// toute case voisine (8-voisinage) d'un obstacle (D41 : le centre reste à une case des
    /// murs).
    pub fn is_blocked_for(&self, pos: &GridPos, key: NavKey) -> bool {
        // D48 : règle partagée avec l'accessibilité des points de caverne (`world::nav`).
        world::nav::blocked_for(
            |x, y| self.is_blocked(&GridPos::new(x, y), key.profile),
            pos.x,
            pos.y,
            key.size == AgentSize::Large,
        )
    }

    /// Couloir trop étroit pour un agent de clé `key` : celui d'une case pour un gabarit petit
    /// ([`Self::is_too_narrow`]) ; un gabarit grand l'exclut déjà par [`Self::is_blocked_for`].
    pub fn is_too_narrow_for(&self, pos: &GridPos, key: NavKey) -> bool {
        key.size == AgentSize::Small && self.is_too_narrow(pos, key.profile)
    }

    /// Check if a cell is blocked for a given navigation profile
    pub fn is_blocked(&self, pos: &GridPos, profile: NavProfile) -> bool {
        // Walls always block (except for Phasing which ignores non-Wall obstacles)
        if self.wall_cells.contains(pos) {
            return true;
        }

        // Check other obstacle types based on profile
        for (obstacle_type, cells) in &self.blocked_cells {
            if cells.contains(pos) && !profile.can_pass(*obstacle_type) {
                return true;
            }
        }

        false
    }

    /// Intact obstacle that blocks movement but that `profile` can break (e.g. a window
    /// for [`NavProfile::GroundBreaker`]).
    pub fn is_breakable_obstacle(&self, pos: &GridPos, profile: NavProfile) -> bool {
        self.blocked_cells
            .iter()
            .any(|(obstacle_type, cells)| cells.contains(pos) && profile.can_pass(*obstacle_type))
    }

    /// Net id of the target (player) the flow field leads to from `pos`: the closest one
    /// along the path.
    pub fn nearest_target(&self, key: NavKey, pos: fixed_math::FixedVec2) -> Option<usize> {
        let owner = *self
            .get_flow_field(key)?
            .owners
            .get(&GridPos::from_fixed(pos))?;
        self.target_ids.get(owner).copied()
    }

    /// Cells of the flow field path from `from`, up to `steps` cells ahead.
    pub fn path_ahead(&self, key: NavKey, from: GridPos, steps: usize) -> Vec<GridPos> {
        let Some(field) = self.get_flow_field(key) else {
            return vec![];
        };
        let mut path = Vec::with_capacity(steps);
        let mut current = from;
        for _ in 0..steps {
            match field.get_direction(current) {
                Some(next) if next != current => {
                    path.push(next);
                    current = next;
                }
                _ => break,
            }
        }
        path
    }

    /// A cell blocked on two opposite sides is a 1-cell corridor: an agent wider than a
    /// cell (zombies are 20 px, cells 16 px) cannot stand in it.
    pub fn is_too_narrow(&self, pos: &GridPos, profile: NavProfile) -> bool {
        world::nav::too_narrow(
            |x, y| self.is_blocked(&GridPos::new(x, y), profile),
            pos.x,
            pos.y,
        )
    }

    /// Point to steer toward in a cell: its center, pushed away from each adjacent blocked
    /// cell by at least half a cell, and more if the agent's body sticks out further on that
    /// side (+1 unit). In a 2-cell opening (door, window) this aims at the middle of the
    /// passage; along a wall it keeps the agent (and its larger sprite) off the wall.
    /// A blocked diagonal neighbor (wall corner, edge of an opening) pushes away by half a
    /// cell on both axes: the agent is centered *before* entering an opening, instead of
    /// entering at an angle and clipping its edge. The diagonal push of an axis is skipped when
    /// the orthogonal neighbor on the side it pushes toward is blocked (D55).
    pub fn steering_point(
        &self,
        cell: GridPos,
        profile: NavProfile,
        body: &AgentBody,
    ) -> fixed_math::FixedVec2 {
        let blocked =
            |dx: i32, dy: i32| self.is_blocked(&GridPos::new(cell.x + dx, cell.y + dy), profile);
        let half_cell = fixed_math::Fixed::from_num(GRID_CELL_SIZE / 2);
        let push =
            |extent: fixed_math::Fixed| (extent - half_cell + fixed_math::FIXED_ONE).max(half_cell);
        let mut point = cell.to_fixed();
        if blocked(-1, 0) {
            point.x += push(body.left);
        }
        if blocked(1, 0) {
            point.x -= push(body.right);
        }
        if blocked(0, -1) {
            point.y += push(body.down);
        }
        if blocked(0, 1) {
            point.y -= push(body.up);
        }
        for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            if blocked(dx, dy) && !blocked(dx, 0) && !blocked(0, dy) {
                // D55 : la poussée d'un axe qui rapproche d'un mur orthogonal est annulée
                // (le mur de ce côté a déjà repoussé le point de tout le corps) ; sinon les
                // deux poussées se compensent et le point retombe dans le mur : l'ennemi
                // accroche le coin sans pouvoir glisser.
                if !blocked(-dx, 0) {
                    point.x -= half_cell * fixed_math::Fixed::from_num(dx);
                }
                if !blocked(0, -dy) {
                    point.y -= half_cell * fixed_math::Fixed::from_num(dy);
                }
            }
        }
        point
    }

    /// Direction to follow the flow field from `pos`, aiming at the next cell's steering
    /// point (see [`Self::steering_point`]). `None` outside the field.
    pub fn flow_direction(
        &self,
        key: NavKey,
        pos: fixed_math::FixedVec2,
        body: &AgentBody,
    ) -> Option<fixed_math::FixedVec2> {
        let field = self.get_flow_field(key)?;
        let profile = key.profile;
        let next = field.get_direction(GridPos::from_fixed(pos))?;
        let direction = self.steering_point(next, profile, body) - pos;
        if direction.length_squared() > fixed_math::FixedWide::ZERO {
            return Some(direction.normalize_or_zero());
        }
        // Already on the steering point: aim at the following cell
        let further = field.get_direction(next)?;
        let direction = self.steering_point(further, profile, body) - pos;
        (direction.length_squared() > fixed_math::FixedWide::ZERO)
            .then(|| direction.normalize_or_zero())
    }

    /// Load wall cells directly from LDtk IntGrid data
    /// This provides perfect 1:1 mapping between LDtk tiles and flow field cells
    pub fn load_intgrid_walls(
        &mut self,
        grid: &[Vec<bool>],
        level_offset: bevy::math::Vec2,
        level_height: usize,
        level_width: usize,
    ) {
        let level_info = LevelGridInfo {
            offset_x: level_offset.x as i32,
            offset_y: level_offset.y as i32,
            height_tiles: level_height as i32,
            width_tiles: level_width as i32,
        };

        // Convert each IntGrid wall tile to FlowField GridPos
        for (ldtk_y, row) in grid.iter().enumerate() {
            for (ldtk_x, &is_wall) in row.iter().enumerate() {
                if is_wall {
                    let grid_pos = ldtk_grid_to_flowfield(ldtk_x, ldtk_y, &level_info);
                    self.intgrid_wall_cells.insert(grid_pos);
                }
            }
        }

        self.level_info = Some(level_info);
        info!(
            "FlowField: loaded {} IntGrid wall cells from {}x{} level at ({}, {})",
            self.intgrid_wall_cells.len(),
            level_width,
            level_height,
            level_offset.x,
            level_offset.y
        );
    }

    /// Remplace les cases murées IntGrid (T1.6, terrain destructible) : seule exception à
    /// « immutable after load ». Appelée après une destruction de terrain dans une caverne ;
    /// `rebuild_blocked_cells` et la détection d'obstacles du prochain pas reconstruisent
    /// alors le flow field en entier.
    pub fn reload_walls(&mut self, cells: BTreeSet<GridPos>) {
        self.intgrid_wall_cells = cells;
    }
}

/// Convert LDtk grid position to FlowField GridPos
/// Handles Y-flip (LDtk Y=0 at top, Bevy Y=0 at bottom) and level offset
pub fn ldtk_grid_to_flowfield(ldtk_x: usize, ldtk_y: usize, info: &LevelGridInfo) -> GridPos {
    // Compute world pixel center of this tile
    // LDtk tile at (ldtk_x, ldtk_y) covers pixels [ldtk_y*16, (ldtk_y+1)*16), center at ldtk_y*16 + 8
    let ldtk_center_x_pixels = (ldtk_x as i32 * GRID_CELL_SIZE) + (GRID_CELL_SIZE / 2);
    let ldtk_center_y_pixels = (ldtk_y as i32 * GRID_CELL_SIZE) + (GRID_CELL_SIZE / 2);

    // Convert X to world pixels (no flip needed)
    let world_center_x = info.offset_x + ldtk_center_x_pixels;

    // Convert Y to world pixels with Y-flip
    // LDtk: Y=0 at top, increases downward
    // Bevy: Y=0 at bottom, increases upward
    let level_height_pixels = info.height_tiles * GRID_CELL_SIZE;
    let world_center_y = info.offset_y + level_height_pixels - ldtk_center_y_pixels;

    // Convert world pixel centers to grid cells
    let world_grid_x = world_center_x.div_euclid(GRID_CELL_SIZE);
    let world_grid_y = world_center_y.div_euclid(GRID_CELL_SIZE);

    GridPos::new(world_grid_x, world_grid_y)
}

/// Configuration for flow field updates
#[derive(Resource, Clone)]
pub struct FlowFieldConfig {
    /// How often to recalculate flow fields (in frames)
    pub update_interval: u32,
    /// Maximum search radius from target
    pub max_search_radius: i32,
    /// Use 8-directional movement (vs 4-directional)
    pub use_8_directions: bool,
    /// Cost of an orthogonal step
    pub straight_cost: u32,
    /// Cost of a diagonal step (~straight × √2)
    pub diagonal_cost: u32,
    /// Extra cost to enter a cell 1 and 2 cells away from a blocked cell. Paths keep away
    /// from walls when there is room (sprites are larger than colliders and would overlap
    /// walls), but still go through doors and windows when they are the only way.
    pub wall_penalty: [u32; 2],
    /// Extra cost to go through an intact obstacle the profile can break (window): the
    /// time to break it. Enemies take an open way if it is not much longer.
    pub breakable_penalty: u32,
}

impl Default for FlowFieldConfig {
    fn default() -> Self {
        Self {
            update_interval: 30,    // Update every 0.5s at 60fps
            max_search_radius: 50,  // 50 cells * 16 units = 800 units radius
            use_8_directions: true, // 8 directions for smoother diagonal movement
            straight_cost: 10,
            diagonal_cost: 14,
            wall_penalty: [30, 10],
            breakable_penalty: 60,
        }
    }
}

/// System to update the global flow field cache
pub fn update_flow_field_system(
    frame: Res<FrameCount>,
    config: Res<FlowFieldConfig>,
    player_query: Query<(&GgrsNetId, &fixed_math::FixedTransform3D, Has<Downed>), With<Player>>,
    // Portes fermées : une porte ouverte n'a plus de collider
    door_query: Query<(&GgrsNetId, &fixed_math::FixedTransform3D, &Collider), With<DoorComponent>>,
    obstacle_query: Query<(&fixed_math::FixedTransform3D, &Collider, &Obstacle), With<Rollback>>,
    // D41 + D38 : clés des ennemis qui se déplacent (un ennemi immobile ne suit aucun champ)
    agents: Query<(&super::state::EnemyAiConfig, &Collider), With<crate::character::enemy::Enemy>>,
    mut cache: ResMut<FlowFieldCache>,
) {
    // Check if we need to update (rate limit)
    if frame.frame < cache.last_update_frame + cache.update_interval {
        return;
    }
    cache.last_update_frame = frame.frame;

    // Wait for walls to be initialized (done by LDTK loader)
    // Prefer IntGrid data, fall back to wall entity count for backward compatibility
    if cache.intgrid_wall_cells.is_empty() && cache.last_wall_entity_count == 0 {
        return;
    }

    // Every player is a target: each cell leads to the closest one along the path.
    // GGRS CRITICAL: players sorted by net_id (deterministic target order)
    let mut players: Vec<_> = player_query.iter().collect();
    if players.is_empty() {
        return; // No players, nothing to do
    }
    // À terre (T1.3, chantier B6) : tant qu'au moins un joueur est encore debout, le flow
    // field (et donc le ciblage ennemi qui s'appuie dessus, `enemy_target_selection`,
    // `update_enemy_targets`) ignore les joueurs à terre — voir la doc de
    // `combat::downed::Downed`. Si personne n'est débout (tous à terre/morts), pas de
    // filtrage : mieux vaut un flow field qui mène quand même quelque part qu'aucun.
    let any_standing = players.iter().any(|(_, _, downed)| !downed);
    if any_standing {
        players.retain(|(_, _, downed)| !downed);
    }
    players.sort_unstable_by_key(|(net_id, ..)| net_id.0);
    let targets: Vec<GridPos> = players
        .iter()
        .map(|(_, transform, _)| GridPos::from_fixed(transform.translation.truncate()))
        .collect();
    let target_ids: Vec<usize> = players.iter().map(|(net_id, ..)| net_id.0).collect();
    let target_pos = targets[0];

    // Rebuild blocked cells first: an opened door or a broken window changes the field
    // even when the target does not move
    let previous_walls = cache.wall_cells.clone();
    let previous_blocked = cache.blocked_cells.clone();
    rebuild_blocked_cells(&mut cache, &door_query, &obstacle_query);
    let obstacles_changed =
        cache.wall_cells != previous_walls || cache.blocked_cells != previous_blocked;

    // Clés à construire (canoniques, voir `FlowFieldCache::canonical`) : le champ historique,
    // plus celles des ennemis présents qui se déplacent. Avec le seul contenu de gabarit petit
    // et sans obstacle qui distingue les profils, il n'y a que [`MOVEMENT_FLOW_KEY`] : même
    // cache et même checksum qu'avant ce chantier.
    let mut keys: BTreeSet<NavKey> = BTreeSet::from([MOVEMENT_FLOW_KEY]);
    for (ai, collider) in &agents {
        if !ai.stationary {
            let key = NavKey::for_agent(ai.nav_profile(), &AgentBody::from_collider(collider));
            keys.insert(cache.canonical(key));
        }
    }
    let keys_changed = !keys.iter().eq(cache.layers.keys());

    if targets == cache.targets
        && target_ids == cache.target_ids
        && !obstacles_changed
        && !keys_changed
        && !cache.layers.is_empty()
    {
        return;
    }

    cache.target_pos = target_pos;
    cache.targets = targets.clone();
    cache.target_ids = target_ids;

    let layers: BTreeMap<NavKey, FlowField> = keys
        .iter()
        .map(|key| (*key, build_flow_field(&targets, *key, &cache, &config)))
        .collect();

    // Log flow field stats only on significant rebuilds
    trace!(
        "FlowField: targets={}, first=({},{}), fields={}, walls={}",
        targets.len(),
        target_pos.x,
        target_pos.y,
        layers.len(),
        cache.wall_cells.len()
    );

    cache.layers = layers;
}

/// Rebuild the blocked cell cache from IntGrid data and current obstacle positions
fn rebuild_blocked_cells(
    cache: &mut FlowFieldCache,
    door_query: &Query<(&GgrsNetId, &fixed_math::FixedTransform3D, &Collider), With<DoorComponent>>,
    obstacle_query: &Query<(&fixed_math::FixedTransform3D, &Collider, &Obstacle), With<Rollback>>,
) {
    cache.blocked_cells.clear();

    // Start with IntGrid wall cells as the source of truth (perfect 1:1 LDtk tile mapping)
    // We no longer iterate wall colliders since IntGrid has all static walls.
    // Wall colliders are only used for physics, not pathfinding.
    cache.wall_cells = cache.intgrid_wall_cells.clone();

    // Door tiles are free in the IntGrid: a door blocks as long as it has a collider
    // (closed, or never openable when not interactable)
    for (_, transform, collider) in utils::order_iter!(door_query) {
        cache.wall_cells.extend(get_collider_cells(
            transform.translation.truncate(),
            collider,
        ));
    }

    let mut window_cells_removed = 0;
    // Process obstacles - windows create HOLES in walls
    for (transform, collider, obstacle) in obstacle_query.iter() {
        let pos = transform.translation.truncate();

        // Windows create passages through walls - remove only the CENTER cell
        // Using center cell prevents accidentally removing adjacent wall cells
        // when the window collider extends slightly beyond its tile
        if obstacle.obstacle_type == ObstacleType::Window {
            let center_cell = GridPos::from_fixed(pos);
            if cache.wall_cells.remove(&center_cell) {
                window_cells_removed += 1;
            }
            // An intact window blocks all the cells of its collider (the whole opening),
            // for profiles that cannot break it; GroundBreaker pays breakable_penalty
            if obstacle.blocks_movement {
                cache
                    .blocked_cells
                    .entry(obstacle.obstacle_type)
                    .or_default()
                    .extend(get_collider_cells(pos, collider));
            }
            continue;
        }

        // For other obstacles, use full collider bounds
        let cells = get_collider_cells(pos, collider);
        if !obstacle.blocks_movement {
            continue;
        }
        cache
            .blocked_cells
            .entry(obstacle.obstacle_type)
            .or_default()
            .extend(cells);
    }

    // Log only at trace level to avoid spam
    trace!(
        "FlowField: {} wall cells, {} window holes",
        cache.wall_cells.len(),
        window_cells_removed
    );
}

/// Get all grid cells occupied by a collider (precise, no padding)
/// Uses exact boundary calculation for proper 1:1 tile alignment
pub fn get_collider_cells(pos: fixed_math::FixedVec2, collider: &Collider) -> Vec<GridPos> {
    let mut cells = Vec::new();
    let offset = fixed_math::FixedVec2::new(collider.offset.x, collider.offset.y);
    let center = pos + offset;
    let center_x = center.x.to_num::<i32>();
    let center_y = center.y.to_num::<i32>();

    match &collider.shape {
        ColliderShape::Circle { radius } => {
            // For circles, use bounding box with proper division
            let r = radius.to_num::<i32>();
            let min_x = (center_x - r).div_euclid(GRID_CELL_SIZE);
            let max_x = (center_x + r - 1).div_euclid(GRID_CELL_SIZE);
            let min_y = (center_y - r).div_euclid(GRID_CELL_SIZE);
            let max_y = (center_y + r - 1).div_euclid(GRID_CELL_SIZE);

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    cells.push(GridPos::new(x, y));
                }
            }
        }
        ColliderShape::Rectangle { width, height } => {
            // Calculate exact bounds without padding
            let half_w = width.to_num::<i32>() / 2;
            let half_h = height.to_num::<i32>() / 2;

            // Use div_euclid for proper floor division
            // Subtract 1 from max to avoid including next cell when exactly on boundary
            let min_x = (center_x - half_w).div_euclid(GRID_CELL_SIZE);
            let max_x = (center_x + half_w - 1).div_euclid(GRID_CELL_SIZE);
            let min_y = (center_y - half_h).div_euclid(GRID_CELL_SIZE);
            let max_y = (center_y + half_h - 1).div_euclid(GRID_CELL_SIZE);

            // Ensure at least one cell (for very small colliders)
            let max_x = max_x.max(min_x);
            let max_y = max_y.max(min_y);

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    cells.push(GridPos::new(x, y));
                }
            }
        }
    }

    cells
}

/// Build a flow field with Dijkstra from the target. Step costs favor cells away from
/// walls (see [`FlowFieldConfig::wall_penalty`]). Deterministic: the heap is ordered by
/// (cost, cell), and cells are only improved by a strictly lower cost.
fn build_flow_field(
    targets: &[GridPos],
    key: NavKey,
    cache: &FlowFieldCache,
    config: &FlowFieldConfig,
) -> FlowField {
    let profile = key.profile;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let mut flow_field = FlowField::default();

    // Search inside the map (walls bounding box + margin) instead of a radius around the
    // target: every spawner of the map must be covered
    let margin = config.max_search_radius.min(8);
    let bounds = cache
        .wall_cells
        .iter()
        .fold(None, |acc: Option<(i32, i32, i32, i32)>, p| {
            Some(match acc {
                None => (p.x, p.x, p.y, p.y),
                Some((x0, x1, y0, y1)) => (x0.min(p.x), x1.max(p.x), y0.min(p.y), y1.max(p.y)),
            })
        });
    let (min_x, max_x, min_y, max_y) = match bounds {
        Some((x0, x1, y0, y1)) => (
            targets.iter().map(|t| t.x).fold(x0, i32::min) - margin,
            targets.iter().map(|t| t.x).fold(x1, i32::max) + margin,
            targets.iter().map(|t| t.y).fold(y0, i32::min) - margin,
            targets.iter().map(|t| t.y).fold(y1, i32::max) + margin,
        ),
        None => {
            let first = targets.first().copied().unwrap_or_default();
            (
                first.x - config.max_search_radius,
                first.x + config.max_search_radius,
                first.y - config.max_search_radius,
                first.y + config.max_search_radius,
            )
        }
    };
    let in_bounds = |p: &GridPos| p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y;

    // Cache geometry once per build. Dijkstra used to repeat BTreeSet lookups for
    // every edge (especially the nine-cell clearance of large actors).
    // The halo preserves tests at the boundary; nothing is retained between frames.
    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;
    let index = |p: GridPos| (p.y - min_y) as usize * width + (p.x - min_x) as usize;
    let halo_width = width + 2;
    let raw_index =
        |x: i32, y: i32| (y - min_y + 1) as usize * halo_width + (x - min_x + 1) as usize;
    let mut raw = vec![false; halo_width * (height + 2)];
    for x in min_x - 1..=max_x + 1 {
        for y in min_y - 1..=max_y + 1 {
            raw[raw_index(x, y)] = cache.is_blocked(&GridPos::new(x, y), profile);
        }
    }
    let blocked = |x, y| raw[raw_index(x, y)];
    let mut clearance = vec![false; width * height];
    let mut narrow = vec![false; width * height];
    let mut breakable = vec![false; width * height];
    let mut wall_distance = vec![u8::MAX; width * height];
    let mut frontier: VecDeque<(GridPos, u8)> = VecDeque::new();
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let p = GridPos::new(x, y);
            let i = index(p);
            clearance[i] = world::nav::blocked_for(blocked, x, y, key.size == AgentSize::Large);
            narrow[i] = key.size == AgentSize::Small && world::nav::too_narrow(blocked, x, y);
            breakable[i] = cache.is_breakable_obstacle(&p, profile);
            if clearance[i] {
                wall_distance[i] = 0;
                frontier.push_back((p, 0));
            }
        }
    }
    while let Some((p, d)) = frontier.pop_front() {
        if d >= 2 {
            continue;
        }
        for n in p.neighbors_8() {
            if in_bounds(&n) && wall_distance[index(n)] == u8::MAX {
                wall_distance[index(n)] = d + 1;
                frontier.push_back((n, d + 1));
            }
        }
    }
    let penalty = |p: GridPos| match wall_distance[index(p)] {
        1 => config.wall_penalty[0],
        2 => config.wall_penalty[1],
        _ => 0,
    };

    // Every target is a source of cost 0 (a cell shared by two targets goes to the first)
    let mut heap: BinaryHeap<Reverse<(u32, GridPos)>> = BinaryHeap::new();
    for (index, target) in targets.iter().enumerate() {
        if flow_field.costs.contains_key(target) {
            continue;
        }
        flow_field.directions.insert(*target, *target);
        flow_field.costs.insert(*target, 0);
        flow_field.owners.insert(*target, index);
        heap.push(Reverse((0, *target)));
    }

    while let Some(Reverse((cost, current))) = heap.pop() {
        if flow_field
            .costs
            .get(&current)
            .is_some_and(|best| cost > *best)
        {
            continue; // stale heap entry
        }

        // Orthogonal neighbors are the first four entries, in the historical order.
        let neighbors = current.neighbors_8();
        let count = if config.use_8_directions { 8 } else { 4 };
        for &neighbor in &neighbors[..count] {
            if !in_bounds(&neighbor) {
                continue;
            }

            // Check if blocked for this profile, or too narrow for an agent
            if clearance[index(neighbor)] || narrow[index(neighbor)] {
                continue;
            }

            // A diagonal step must not cut a corner: both orthogonal cells must be free,
            // otherwise the enemy collider hits the wall corner and gets stuck
            let (dx, dy) = (neighbor.x - current.x, neighbor.y - current.y);
            let diagonal = dx != 0 && dy != 0;
            if diagonal
                && [
                    GridPos::new(current.x + dx, current.y),
                    GridPos::new(current.x, current.y + dy),
                ]
                .iter()
                .any(|p| {
                    if in_bounds(p) {
                        clearance[index(*p)]
                    } else {
                        cache.is_blocked_for(p, key)
                    }
                })
            {
                continue;
            }

            // D55 : un pas orthogonal entre deux coins de mur opposés (un en haut d'un côté,
            // un en bas de l'autre) ne laisse qu'une case (16 px) de passage à l'endroit où le
            // corps le franchit : infranchissable pour un corps de plus de 16 px.
            if !diagonal
                && key.size == AgentSize::Small
                && world::nav::chicane_step(blocked, current.x, current.y, dx, dy)
            {
                continue;
            }

            let step = if diagonal {
                config.diagonal_cost
            } else {
                config.straight_cost
            };
            let breakable = if breakable[index(neighbor)] {
                config.breakable_penalty
            } else {
                0
            };
            let new_cost = cost + step + penalty(neighbor) + breakable;
            if flow_field
                .costs
                .get(&neighbor)
                .is_some_and(|best| new_cost >= *best)
            {
                continue;
            }

            // Direction points TOWARD target (so we store 'current' as the next step)
            flow_field.directions.insert(neighbor, current);
            flow_field.costs.insert(neighbor, new_cost);
            let owner = flow_field.owners[&current];
            flow_field.owners.insert(neighbor, owner);
            heap.push(Reverse((new_cost, neighbor)));
        }
    }

    flow_field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_pos_conversion() {
        let world_pos = fixed_math::FixedVec2::new(
            fixed_math::Fixed::from_num(45),
            fixed_math::Fixed::from_num(65),
        );
        let grid_pos = GridPos::from_fixed(world_pos);
        assert_eq!(grid_pos.x, 2); // 45 / 16 = 2
        assert_eq!(grid_pos.y, 4); // 65 / 16 = 4
    }

    #[test]
    fn test_nav_profile_can_pass() {
        assert!(!NavProfile::Ground.can_pass(ObstacleType::Window));
        assert!(NavProfile::GroundBreaker.can_pass(ObstacleType::Window));
        assert!(NavProfile::Flying.can_pass(ObstacleType::Water));
        assert!(!NavProfile::Flying.can_pass(ObstacleType::Wall));
        assert!(NavProfile::Phasing.can_pass(ObstacleType::Window));
        assert!(!NavProfile::Phasing.can_pass(ObstacleType::Wall));
    }
}

/// D55 : un coin en diagonale ne repousse pas le point visé vers un mur orthogonal.
#[cfg(test)]
mod steering_corner_tests {
    use super::*;

    fn body_rat() -> AgentBody {
        AgentBody::from_collider(&Collider {
            shape: ColliderShape::Rectangle {
                width: fixed_math::new(20.0),
                height: fixed_math::new(20.0),
            },
            offset: fixed_math::FixedVec3::new(
                fixed_math::FIXED_ZERO,
                fixed_math::new(-6.0),
                fixed_math::FIXED_ZERO,
            ),
        })
    }

    /// Graine 19 de throne : mur sous la case (11,37), coin bloqué en haut à gauche (10,38).
    /// Le point visé doit garder le collider (bas à `y - 16`) au-dessus du mur (haut à 592).
    #[test]
    fn le_point_vise_ne_retombe_pas_dans_le_mur() {
        let mut cache = FlowFieldCache::default();
        for x in 11..=12 {
            cache.wall_cells.insert(GridPos::new(x, 36));
        }
        cache.wall_cells.insert(GridPos::new(10, 38));
        let body = body_rat();
        let point = cache.steering_point(GridPos::new(11, 37), NavProfile::Ground, &body);
        assert!(point.y - body.down >= fixed_math::new(592.0), "{point:?}");
        // La poussée en x par le coin reste appliquée.
        assert_eq!(point.x, fixed_math::new(192.0));
    }

    /// Le pas (10,37) → (11,37) de la graine 19 est une chicane ; un pas libre ne l'est pas.
    #[test]
    fn chicane_entre_deux_coins_opposes() {
        let walls: std::collections::BTreeSet<(i32, i32)> =
            [(10, 38), (11, 36), (12, 36)].into_iter().collect();
        let blocked = |x, y| walls.contains(&(x, y));
        assert!(world::nav::chicane_step(blocked, 10, 37, 1, 0));
        assert!(world::nav::chicane_step(blocked, 11, 37, -1, 0));
        assert!(!world::nav::chicane_step(blocked, 11, 37, 1, 0));
        assert!(!world::nav::chicane_step(blocked, 10, 37, 0, 1));
    }

    /// Sans mur orthogonal, le coin diagonal repousse toujours sur les deux axes.
    #[test]
    fn le_coin_seul_repousse_sur_les_deux_axes() {
        let mut cache = FlowFieldCache::default();
        cache.wall_cells.insert(GridPos::new(10, 38));
        let point = cache.steering_point(GridPos::new(11, 37), NavProfile::Ground, &body_rat());
        assert_eq!(point.x, fixed_math::new(192.0));
        assert_eq!(point.y, fixed_math::new(592.0));
    }
}

#[cfg(test)]
mod retreat_tests {
    use super::*;

    /// Couloir horizontal : coût croissant vers la droite (cible à gauche) ; le recul va à
    /// droite, et rien quand on est déjà au bout.
    #[test]
    fn recul_vers_le_cout_le_plus_eleve() {
        let mut field = FlowField::default();
        for x in 0..5 {
            field.costs.insert(GridPos::new(x, 0), x as u32 * 10);
        }
        let at = |x: i32| GridPos::new(x, 0).to_fixed();
        let dir = field.retreat_direction(at(2)).unwrap();
        assert!(dir.x > fixed_math::new(0.9), "{dir:?}");
        assert_eq!(field.retreat_direction(at(4)), None);
        // Hors du champ : rien.
        assert_eq!(
            field.retreat_direction(GridPos::new(10, 10).to_fixed()),
            None
        );
    }
}

/// D41 + D38 : clés de champ (profil, gabarit).
#[cfg(test)]
mod nav_key_tests {
    use super::*;
    use std::hash::{Hash, Hasher};

    fn body(width: f32, height: f32) -> AgentBody {
        AgentBody::from_collider(&Collider {
            shape: ColliderShape::Rectangle {
                width: fixed_math::new(width),
                height: fixed_math::new(height),
            },
            offset: fixed_math::FixedVec3::new(
                fixed_math::FIXED_ZERO,
                fixed_math::new(-6.0),
                fixed_math::FIXED_ZERO,
            ),
        })
    }

    fn hash_of(value: impl Hash) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn gabarit_selon_le_corps() {
        assert_eq!(AgentSize::of(&body(20.0, 20.0)), AgentSize::Small);
        assert_eq!(AgentSize::of(&body(28.0, 28.0)), AgentSize::Large);
        assert_eq!(AgentSize::of(&body(20.0, 24.0)), AgentSize::Large);
    }

    #[test]
    fn cle_petite_hachee_comme_son_profil() {
        // Checksum : un cache qui ne porte que le champ historique ne change pas.
        assert_eq!(hash_of(MOVEMENT_FLOW_KEY), hash_of(MOVEMENT_FLOW_PROFILE));
        let mut old: BTreeMap<NavProfile, u8> = BTreeMap::new();
        old.insert(NavProfile::GroundBreaker, 7);
        let mut new: BTreeMap<NavKey, u8> = BTreeMap::new();
        new.insert(MOVEMENT_FLOW_KEY, 7);
        assert_eq!(hash_of(&old), hash_of(&new));
        assert_ne!(
            hash_of(NavKey::new(NavProfile::GroundBreaker, AgentSize::Large)),
            hash_of(MOVEMENT_FLOW_KEY)
        );
    }

    #[test]
    fn grand_gabarit_reste_a_une_case_des_murs() {
        let mut cache = FlowFieldCache::new();
        cache.wall_cells.insert(GridPos::new(0, 0));
        let large = NavKey::new(NavProfile::GroundBreaker, AgentSize::Large);
        assert!(!cache.is_blocked_for(&GridPos::new(1, 1), MOVEMENT_FLOW_KEY));
        assert!(cache.is_blocked_for(&GridPos::new(1, 1), large));
        assert!(!cache.is_blocked_for(&GridPos::new(2, 0), large));
        // Couloir de trois cases (murs en x = 0 et x = 4) : praticable au centre pour un grand.
        for y in -3..=3 {
            cache.wall_cells.insert(GridPos::new(0, y));
            cache.wall_cells.insert(GridPos::new(4, y));
        }
        assert!(!cache.is_blocked_for(&GridPos::new(2, 0), large));
        assert!(!cache.is_too_narrow_for(&GridPos::new(2, 0), large));
        assert!(cache.is_blocked_for(&GridPos::new(1, 0), large));
    }

    #[test]
    fn ground_partage_le_champ_historique_sans_fenetre() {
        let mut cache = FlowFieldCache::new();
        let ground = NavKey::new(NavProfile::Ground, AgentSize::Small);
        assert_eq!(cache.canonical(ground), MOVEMENT_FLOW_KEY);
        cache
            .blocked_cells
            .entry(ObstacleType::Window)
            .or_default()
            .insert(GridPos::new(3, 3));
        assert_eq!(cache.canonical(ground), ground);
        // Un gabarit grand garde sa taille dans la clé canonique.
        assert_eq!(
            cache.canonical(NavKey::new(NavProfile::GroundBreaker, AgentSize::Large)),
            NavKey::new(NavProfile::GroundBreaker, AgentSize::Large)
        );
    }
}

/// D48 : l'accessibilité des points de caverne (`world::nav::nav_distances`) et le champ de flux
/// atteignent exactement les mêmes cases, pour les deux gabarits, sur les cavernes de `throne`.
#[cfg(test)]
mod cave_nav_tests {
    use super::*;
    use world::{CaveConfig, CellKind};

    fn cave(name: &str) -> CaveConfig {
        let path = format!(
            "{}/../../games/throne/assets/caves/{name}.ron",
            env!("CARGO_MANIFEST_DIR")
        );
        ron::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn champ_de_flux_et_points_de_caverne_meme_accessibilite() {
        for name in ["niveau_1", "niveau_2", "niveau_3"] {
            let config = cave(name);
            for seed in 1..=20u64 {
                let grid = world::generate(seed, &config);
                let mut cache = FlowFieldCache::new();
                for y in 0..grid.height {
                    for x in 0..grid.width {
                        if grid.get(x as i32, y as i32).is_some_and(CellKind::is_solid) {
                            cache.wall_cells.insert(GridPos::new(x as i32, y as i32));
                        }
                    }
                }
                let points = world::points_of_interest(&grid, 4, 0, 1, false);
                let targets: Vec<GridPos> = points
                    .player_spawns
                    .iter()
                    .map(|&(x, y)| GridPos::new(x as i32, y as i32))
                    .collect();
                for size in [AgentSize::Small, AgentSize::Large] {
                    let key = NavKey::new(NavProfile::GroundBreaker, size);
                    let field =
                        build_flow_field(&targets, key, &cache, &FlowFieldConfig::default());
                    let dist = world::nav::nav_distances(
                        &grid,
                        &points.player_spawns,
                        size == AgentSize::Large,
                    );
                    for y in 0..grid.height {
                        for x in 0..grid.width {
                            let in_field =
                                field.costs.contains_key(&GridPos::new(x as i32, y as i32));
                            let reached = dist[(y * grid.width + x) as usize] != u32::MAX;
                            assert_eq!(
                                in_field, reached,
                                "{name} graine {seed} {size:?} case ({x}, {y})"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
fn build_flow_field_legacy(
    targets: &[GridPos],
    key: NavKey,
    cache: &FlowFieldCache,
    config: &FlowFieldConfig,
) -> FlowField {
    let profile = key.profile;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let mut flow_field = FlowField::default();

    // Search inside the map (walls bounding box + margin) instead of a radius around the
    // target: every spawner of the map must be covered
    let margin = config.max_search_radius.min(8);
    let bounds = cache
        .wall_cells
        .iter()
        .fold(None, |acc: Option<(i32, i32, i32, i32)>, p| {
            Some(match acc {
                None => (p.x, p.x, p.y, p.y),
                Some((x0, x1, y0, y1)) => (x0.min(p.x), x1.max(p.x), y0.min(p.y), y1.max(p.y)),
            })
        });
    let (min_x, max_x, min_y, max_y) = match bounds {
        Some((x0, x1, y0, y1)) => (
            targets.iter().map(|t| t.x).fold(x0, i32::min) - margin,
            targets.iter().map(|t| t.x).fold(x1, i32::max) + margin,
            targets.iter().map(|t| t.y).fold(y0, i32::min) - margin,
            targets.iter().map(|t| t.y).fold(y1, i32::max) + margin,
        ),
        None => {
            let first = targets.first().copied().unwrap_or_default();
            (
                first.x - config.max_search_radius,
                first.x + config.max_search_radius,
                first.y - config.max_search_radius,
                first.y + config.max_search_radius,
            )
        }
    };
    let in_bounds = |p: &GridPos| p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y;

    // Distance (in cells, 8-neighborhood) to the nearest blocked cell, up to 2
    let mut wall_distance: BTreeMap<GridPos, u8> = BTreeMap::new();
    let mut frontier: VecDeque<(GridPos, u8)> = VecDeque::new();
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let p = GridPos::new(x, y);
            if cache.is_blocked_for(&p, key) {
                wall_distance.insert(p, 0);
                frontier.push_back((p, 0));
            }
        }
    }
    while let Some((p, d)) = frontier.pop_front() {
        if d >= 2 {
            continue;
        }
        for n in p.neighbors_8() {
            if in_bounds(&n) && !wall_distance.contains_key(&n) {
                wall_distance.insert(n, d + 1);
                frontier.push_back((n, d + 1));
            }
        }
    }
    let penalty = |p: &GridPos| match wall_distance.get(p) {
        Some(1) => config.wall_penalty[0],
        Some(2) => config.wall_penalty[1],
        _ => 0,
    };

    // Every target is a source of cost 0 (a cell shared by two targets goes to the first)
    let mut heap: BinaryHeap<Reverse<(u32, GridPos)>> = BinaryHeap::new();
    for (index, target) in targets.iter().enumerate() {
        if flow_field.costs.contains_key(target) {
            continue;
        }
        flow_field.directions.insert(*target, *target);
        flow_field.costs.insert(*target, 0);
        flow_field.owners.insert(*target, index);
        heap.push(Reverse((0, *target)));
    }

    while let Some(Reverse((cost, current))) = heap.pop() {
        if flow_field
            .costs
            .get(&current)
            .is_some_and(|best| cost > *best)
        {
            continue; // stale heap entry
        }

        let neighbors = if config.use_8_directions {
            current.neighbors_8().to_vec()
        } else {
            current.neighbors_4().to_vec()
        };

        for neighbor in neighbors {
            if !in_bounds(&neighbor) {
                continue;
            }

            // Check if blocked for this profile, or too narrow for an agent
            if cache.is_blocked_for(&neighbor, key) || cache.is_too_narrow_for(&neighbor, key) {
                continue;
            }

            // A diagonal step must not cut a corner: both orthogonal cells must be free,
            // otherwise the enemy collider hits the wall corner and gets stuck
            let (dx, dy) = (neighbor.x - current.x, neighbor.y - current.y);
            let diagonal = dx != 0 && dy != 0;
            if diagonal
                && world::nav::diagonal_cuts_corner(
                    |x, y| cache.is_blocked(&GridPos::new(x, y), profile),
                    current.x,
                    current.y,
                    dx,
                    dy,
                    key.size == AgentSize::Large,
                )
            {
                continue;
            }

            // D55 : un pas orthogonal entre deux coins de mur opposés (un en haut d'un côté,
            // un en bas de l'autre) ne laisse qu'une case (16 px) de passage à l'endroit où le
            // corps le franchit : infranchissable pour un corps de plus de 16 px.
            if !diagonal
                && key.size == AgentSize::Small
                && world::nav::chicane_step(
                    |x, y| cache.is_blocked(&GridPos::new(x, y), profile),
                    current.x,
                    current.y,
                    dx,
                    dy,
                )
            {
                continue;
            }

            let step = if diagonal {
                config.diagonal_cost
            } else {
                config.straight_cost
            };
            let breakable = if cache.is_breakable_obstacle(&neighbor, profile) {
                config.breakable_penalty
            } else {
                0
            };
            let new_cost = cost + step + penalty(&neighbor) + breakable;
            if flow_field
                .costs
                .get(&neighbor)
                .is_some_and(|best| new_cost >= *best)
            {
                continue;
            }

            // Direction points TOWARD target (so we store 'current' as the next step)
            flow_field.directions.insert(neighbor, current);
            flow_field.costs.insert(neighbor, new_cost);
            let owner = flow_field.owners[&current];
            flow_field.owners.insert(neighbor, owner);
            heap.push(Reverse((new_cost, neighbor)));
        }
    }

    flow_field
}

#[cfg(test)]
mod optimized_flow_tests {
    use super::*;
    use std::hash::{Hash, Hasher};

    // Exact state equivalence, not just reachability: direction ties and owners matter
    // for multiplayer target selection and for the rollback checksum.
    #[test]
    fn optimized_geometry_matches_legacy_flow_fields() {
        for seed in 0i32..24 {
            let mut cache = FlowFieldCache::new();
            for x in -2..=12 {
                for y in -2..=10 {
                    if x == -2
                        || x == 12
                        || y == -2
                        || y == 10
                        || (x * 19 + y * 31 + seed * 13).rem_euclid(17) < 3
                    {
                        cache.wall_cells.insert(GridPos::new(x, y));
                    }
                }
            }
            cache.blocked_cells.insert(
                ObstacleType::Window,
                [GridPos::new(5, 4), GridPos::new(5, 5)]
                    .into_iter()
                    .collect(),
            );
            cache.blocked_cells.insert(
                ObstacleType::Water,
                [GridPos::new(3, 6), GridPos::new(4, 6)]
                    .into_iter()
                    .collect(),
            );
            let targets = [GridPos::new(1, 1), GridPos::new(9, 7), GridPos::new(1, 1)];
            for profile in [
                NavProfile::Ground,
                NavProfile::GroundBreaker,
                NavProfile::Flying,
                NavProfile::Phasing,
            ] {
                for size in [AgentSize::Small, AgentSize::Large] {
                    for diagonals in [false, true] {
                        let config = FlowFieldConfig {
                            use_8_directions: diagonals,
                            ..Default::default()
                        };
                        let key = NavKey::new(profile, size);
                        let old = build_flow_field_legacy(&targets, key, &cache, &config);
                        let new = build_flow_field(&targets, key, &cache, &config);
                        assert_eq!(new.costs, old.costs, "seed {seed}, {key:?}");
                        assert_eq!(new.directions, old.directions, "seed {seed}, {key:?}");
                        assert_eq!(new.owners, old.owners, "seed {seed}, {key:?}");
                        let hash = |field: &FlowField| {
                            let mut h = std::collections::hash_map::DefaultHasher::new();
                            field.hash(&mut h);
                            h.finish()
                        };
                        assert_eq!(hash(&new), hash(&old));
                    }
                }
            }
        }
    }
}
