//! Navigation des nouveaux profils, dérivée des colliders observés dans ReadInputs.
//! Le cache ne constitue pas un état de simulation : ses clés contiennent toute la géométrie,
//! le corps et les cibles. GGRS rejoue les inputs enregistrés, jamais ce calcul. Comme
//! CollisionGrids, il n'entre donc ni dans les snapshots ni dans les traces. Aucun état de
//! décision, compteur ou RNG n'est caché ici ; vider le cache donne exactement le même input.
//!
//! Dijkstra multi-source et FlowField/GridPos partagés avec les zombies. Une grille de 8 px
//! représente les positions réellement libres du corps (offset aux pieds inclus), plutôt que
//! les seules tuiles de 16 px : le centre d'une ouverture de 32 px peut être entre deux tuiles.
//! Les fenêtres, même cassées, restent bloquantes pour le joueur. Les sources sont des postes
//! de tir autour des ennemis, pour pouvoir tuer ceux qui restent derrière ces fenêtres.
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use bevy::prelude::Resource;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use game::character::enemy::ai::navigation::{AgentBody, FlowField, GridPos};
use game::collider::{Collider, ColliderShape};

const CELL: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub min: FixedVec2,
    pub max: FixedVec2,
}
impl Rect {
    pub fn collider(position: FixedVec2, collider: &Collider) -> Self {
        let center = position + collider.offset.truncate();
        let half = match collider.shape {
            ColliderShape::Rectangle { width, height } => {
                FixedVec2::new(width / Fixed::from_num(2), height / Fixed::from_num(2))
            }
            ColliderShape::Circle { radius } => FixedVec2::new(radius, radius),
        };
        Self {
            min: center - half,
            max: center + half,
        }
    }
    fn expanded(self, body: &AgentBody) -> Self {
        let margin = Fixed::from_num(1);
        Self {
            min: self.min - FixedVec2::new(body.right + margin, body.up + margin),
            max: self.max + FixedVec2::new(body.left + margin, body.down + margin),
        }
    }
    pub fn distance(self, point: FixedVec2) -> Fixed {
        point.distance(&FixedVec2::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
        ))
    }
    fn contains(self, point: FixedVec2) -> bool {
        point.x > self.min.x && point.x < self.max.x && point.y > self.min.y && point.y < self.max.y
    }
    /// Slabs, exclusivement Fixed : collision continue, coins compris.
    fn crosses(self, from: FixedVec2, to: FixedVec2) -> bool {
        let delta = to - from;
        let mut enter = Fixed::ZERO;
        let mut leave = Fixed::ONE;
        for (start, direction, min, max) in [
            (from.x, delta.x, self.min.x, self.max.x),
            (from.y, delta.y, self.min.y, self.max.y),
        ] {
            if direction == Fixed::ZERO {
                if start <= min || start >= max {
                    return false;
                }
            } else {
                let a = (min - start).saturating_div(direction);
                let b = (max - start).saturating_div(direction);
                enter = enter.max(a.min(b));
                leave = leave.min(a.max(b));
                if enter >= leave {
                    return false;
                }
            }
        }
        enter < leave
    }
}

pub fn cell(position: FixedVec2) -> GridPos {
    GridPos::new(
        (position.x / Fixed::from_num(CELL)).round().to_num(),
        (position.y / Fixed::from_num(CELL)).round().to_num(),
    )
}
pub fn point(cell: GridPos) -> FixedVec2 {
    FixedVec2::new(
        Fixed::from_num(cell.x * CELL),
        Fixed::from_num(cell.y * CELL),
    )
}

#[derive(Resource, Default)]
pub struct BotNavigation {
    geometry: Vec<(Rect, bool)>,
    body_key: Option<(Fixed, Fixed, Fixed, Fixed)>,
    obstacles: Vec<Rect>,
    walls: Vec<Rect>,
    blocked: BTreeSet<GridPos>,
    bounds: (i32, i32, i32, i32),
    geometry_bounds: (i32, i32, i32, i32),
    targets: Vec<(usize, GridPos)>,
    pub field: FlowField,
    approaches: BTreeMap<(GridPos, GridPos, Fixed), FlowField>,
}

impl BotNavigation {
    pub fn update(
        &mut self,
        geometry: &[(Rect, bool)],
        body: &AgentBody,
        enemies: &[(usize, FixedVec2)],
    ) {
        let key = (body.left, body.right, body.down, body.up);
        let changed = self.geometry != geometry || self.body_key != Some(key);
        if changed {
            self.approaches.clear();
            self.geometry = geometry.to_vec();
            self.body_key = Some(key);
            self.obstacles = geometry
                .iter()
                .map(|(rect, _)| rect.expanded(body))
                .collect();
            self.walls = geometry
                .iter()
                .filter(|(_, wall)| *wall)
                .map(|(rect, _)| *rect)
                .collect();
            self.blocked.clear();
            let mut bounds = (0, 0, 0, 0);
            for rect in &self.obstacles {
                let min = cell(rect.min);
                let max = cell(rect.max);
                bounds = (
                    bounds.0.min(min.x - 16),
                    bounds.1.max(max.x + 16),
                    bounds.2.min(min.y - 16),
                    bounds.3.max(max.y + 16),
                );
                for x in min.x - 1..=max.x + 1 {
                    for y in min.y - 1..=max.y + 1 {
                        let p = GridPos::new(x, y);
                        if rect.contains(point(p)) {
                            self.blocked.insert(p);
                        }
                    }
                }
            }
            self.geometry_bounds = bounds;
        }
        let targets: Vec<_> = enemies
            .iter()
            .map(|(id, position)| (*id, cell(*position)))
            .collect();
        if !changed && targets == self.targets {
            return;
        }
        self.approaches.clear();
        self.targets = targets;
        self.bounds = self.geometry_bounds;
        for (_, target) in &self.targets {
            self.bounds = (
                self.bounds.0.min(target.x - 32),
                self.bounds.1.max(target.x + 32),
                self.bounds.2.min(target.y - 32),
                self.bounds.3.max(target.y + 32),
            );
        }
        let mut sources = Vec::new();
        for (owner, (_, target)) in self.targets.iter().enumerate() {
            // Plusieurs côtés de la fenêtre : une source extérieure ne doit pas masquer
            // le poste de tir intérieur, qui appartient à une autre composante accessible.
            for radius in [8, 12, 20] {
                for (dx, dy) in [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ] {
                    let p = GridPos::new(target.x + radius * dx, target.y + radius * dy);
                    if self.free(p) && self.visible(point(p), point(*target)) {
                        sources.push((p, owner));
                    }
                }
            }
        }
        self.field = self.build_field(&sources);
    }
    fn free(&self, p: GridPos) -> bool {
        let (x0, x1, y0, y1) = self.bounds;
        p.x >= x0 && p.x <= x1 && p.y >= y0 && p.y <= y1 && !self.blocked.contains(&p)
    }
    pub fn visible(&self, from: FixedVec2, to: FixedVec2) -> bool {
        !self.walls.iter().any(|rect| rect.crosses(from, to))
    }
    pub fn clear(&self, from: FixedVec2, to: FixedVec2) -> bool {
        !self.obstacles.iter().any(|rect| {
            if rect.contains(from) {
                let clearance = |p: FixedVec2| {
                    (p.x - rect.min.x)
                        .min(rect.max.x - p.x)
                        .min(p.y - rect.min.y)
                        .min(rect.max.y - p.y)
                };
                rect.contains(to) && clearance(to) > clearance(from)
            } else {
                rect.crosses(from, to)
            }
        })
    }
    fn step_free(&self, from: GridPos, to: GridPos) -> bool {
        self.free(to)
            && (from.x == to.x
                || from.y == to.y
                || (self.free(GridPos::new(from.x, to.y)) && self.free(GridPos::new(to.x, from.y))))
    }
    fn build_field(&self, sources: &[(GridPos, usize)]) -> FlowField {
        let mut field = FlowField::default();
        let mut heap = BinaryHeap::new();
        for &(p, owner) in sources {
            if field.costs.contains_key(&p) {
                continue;
            }
            field.costs.insert(p, 0);
            field.directions.insert(p, p);
            field.owners.insert(p, owner);
            heap.push(Reverse((0u32, p)));
        }
        while let Some(Reverse((cost, p))) = heap.pop() {
            if field.costs.get(&p) != Some(&cost) {
                continue;
            }
            for next in p.neighbors_8() {
                if !self.step_free(p, next) {
                    continue;
                }
                let next_cost = cost
                    + if p.x == next.x || p.y == next.y {
                        10
                    } else {
                        14
                    };
                if field.costs.get(&next).is_some_and(|c| *c <= next_cost) {
                    continue;
                }
                field.costs.insert(next, next_cost);
                field.directions.insert(next, p);
                field.owners.insert(next, field.owners[&p]);
                heap.push(Reverse((next_cost, next)));
            }
        }
        field
    }
    pub fn chase(&self, from: FixedVec2) -> Option<FixedVec2> {
        let p = cell(from);
        if let Some(next) = self.field.directions.get(&p) {
            let target = point(*next);
            if self.clear(from, target) {
                return Some(target - from);
            }
        }
        // Une position physique valide peut s'arrondir dans une case bloquée.
        // Rejoindre la case couverte la plus proche, sans traverser un obstacle.
        (-2..=2)
            .flat_map(|dx| (-2..=2).map(move |dy| GridPos::new(p.x + dx, p.y + dy)))
            .filter(|p| self.field.costs.contains_key(p) && self.clear(from, point(*p)))
            .min_by_key(|p| (from.distance(&point(*p)), self.field.costs[p], *p))
            .map(|p| point(p) - from)
    }

    /// Champ dérivé partagé pour une interaction ou une zone d'aggro. Les zones
    /// d'ennemis sont quantifiées comme le champ de tir (la marge d'approche couvre
    /// l'erreur de quantification). Évite un A* exhaustif par bot et par frame.
    pub fn approach(
        &mut self,
        from: FixedVec2,
        target: Rect,
        reach: Fixed,
    ) -> Option<(FixedVec2, u32)> {
        if target.distance(from) <= reach {
            return Some((FixedVec2::ZERO, 0));
        }
        let key = (cell(target.min), cell(target.max), reach);
        if !self.approaches.contains_key(&key) {
            let zone = Rect {
                min: point(key.0),
                max: point(key.1),
            };
            let radius = (reach / Fixed::from_num(CELL)).ceil().to_num::<i32>() + 1;
            let mut sources = Vec::new();
            for x in key.0.x - radius..=key.1.x + radius {
                for y in key.0.y - radius..=key.1.y + radius {
                    let p = GridPos::new(x, y);
                    if self.free(p) && zone.distance(point(p)) <= reach {
                        sources.push((p, 0));
                    }
                }
            }
            let field = self.build_field(&sources);
            self.approaches.insert(key, field);
        }
        let field = &self.approaches[&key];
        let p = cell(from);
        if let Some(next) = field.directions.get(&p) {
            if self.clear(from, point(*next)) {
                return Some((point(*next) - from, field.costs[&p]));
            }
        }
        (-2..=2)
            .flat_map(|dx| (-2..=2).map(move |dy| GridPos::new(p.x + dx, p.y + dy)))
            .filter(|p| field.costs.contains_key(p) && self.clear(from, point(*p)))
            .min_by_key(|p| (field.costs[p], from.distance(&point(*p)), *p))
            .map(|p| (point(p) - from, field.costs[&p]))
    }
    /// Choisit parmi les huit directions du clavier, en vérifiant le segment réel du corps.
    pub fn safe_direction(&self, from: FixedVec2, desired: FixedVec2) -> FixedVec2 {
        if desired.length_squared() <= Fixed::from_num(4) {
            return FixedVec2::ZERO;
        }
        let mut best = None;
        for (dx, dy) in [
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ] {
            let direction =
                FixedVec2::new(Fixed::from_num(dx), Fixed::from_num(dy)).normalize_or_zero();
            let next = from + direction * Fixed::from_num(4);
            if !self.clear(from, next) {
                continue;
            }
            let score = (desired - direction * Fixed::from_num(4)).length_squared();
            if best.as_ref().is_none_or(|(value, _)| score < *value) {
                best = Some((score, direction));
            }
        }
        best.map_or(FixedVec2::ZERO, |(_, dir)| dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn vec(x: i32, y: i32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }
    fn body() -> AgentBody {
        AgentBody {
            left: Fixed::from_num(10),
            right: Fixed::from_num(10),
            down: Fixed::from_num(16),
            up: Fixed::from_num(4),
        }
    }
    fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
        Rect {
            min: vec(x0, y0),
            max: vec(x1, y1),
        }
    }
    #[test]
    fn wall_detour_and_no_corner_cutting() {
        let mut nav = BotNavigation::default();
        nav.update(
            &[(rect(-8, -64, 8, 64), true)],
            &body(),
            &[(1, vec(160, 0))],
        );
        assert!(!nav.visible(vec(-80, 0), vec(160, 0)));
        assert!(!nav.clear(vec(-80, 0), vec(80, 0)));
        let mut p = cell(vec(-80, 0));
        let mut went_around = false;
        for _ in 0..200 {
            let next = nav.field.directions[&p];
            if next == p {
                break;
            }
            assert!(nav.clear(point(p), point(next)));
            went_around |=
                point(next).y > Fixed::from_num(68) || point(next).y < Fixed::from_num(-80);
            p = next;
        }
        assert!(went_around);
        assert!(nav.visible(point(p), vec(160, 0)));
    }
    #[test]
    fn broken_window_still_blocks_body_but_allows_shooting() {
        let mut nav = BotNavigation::default();
        nav.update(
            &[(rect(-8, -48, 8, 48), false)],
            &body(),
            &[(1, vec(40, 0))],
        );
        assert!(nav.visible(vec(-40, 0), vec(40, 0)));
        assert!(!nav.clear(vec(-40, 0), vec(40, 0)));
        assert!(nav.chase(vec(-56, 0)).is_some());
    }
    #[test]
    fn door_opening_rebuilds_field_even_when_targets_do_not_move() {
        let mut nav = BotNavigation::default();
        let closed = [(rect(-8, -64, 8, 64), true)];
        let targets = [(1, vec(160, 0))];
        nav.update(&closed, &body(), &targets);
        let blocked_cost = nav.field.costs[&cell(vec(-80, 0))];
        nav.update(&[], &body(), &targets);
        assert!(nav.clear(vec(-80, 0), vec(160, 0)));
        assert!(nav.field.costs[&cell(vec(-80, 0))] < blocked_cost);
    }
    #[test]
    fn target_change_and_fresh_cache_give_identical_fields() {
        let geometry = [(rect(-8, -48, 8, 48), false)];
        let mut cached = BotNavigation::default();
        cached.update(&geometry, &body(), &[(1, vec(40, 0))]);
        let targets = [(1, vec(160, 80)), (2, vec(-160, 80))];
        cached.update(&geometry, &body(), &targets);
        let mut fresh = BotNavigation::default();
        fresh.update(&geometry, &body(), &targets);
        assert_eq!(cached.field.directions, fresh.field.directions);
        assert_eq!(cached.field.costs, fresh.field.costs);
        assert_eq!(cached.field.owners, fresh.field.owners);
    }
    #[test]
    fn collider_offset_and_two_tile_opening_are_respected() {
        let mut nav = BotNavigation::default();
        nav.update(
            &[(rect(-80, -8, -16, 8), true), (rect(16, -8, 80, 8), true)],
            &body(),
            &[(1, vec(0, 160))],
        );
        assert!(
            nav.clear(vec(0, -80), vec(0, 80)),
            "le milieu de l'ouverture de 32 px est praticable"
        );
        assert!(
            !nav.clear(vec(8, -80), vec(8, 80)),
            "à 8 px du milieu, les 20 px du corps heurtent le mur"
        );
        assert!(nav.chase(vec(0, -80)).is_some());
    }
    #[test]
    fn segment_crossing_rejects_corner_and_accepts_negative_coordinates() {
        let obstacle = rect(-16, -16, 0, 0);
        assert!(obstacle.crosses(vec(-32, 8), vec(8, -32)));
        assert!(!obstacle.crosses(vec(-32, 8), vec(8, 8)));
    }

    #[test]
    fn interaction_path_starts_with_a_physically_clear_step() {
        let mut nav = BotNavigation::default();
        nav.update(&[(rect(-8, -64, 8, 64), true)], &body(), &[]);
        // The real body is clear, but rounding places its grid cell inside the wall.
        let from = vec(-20, 0);
        let (direction, _) = nav
            .approach(from, rect(80, 0, 80, 0), Fixed::from_num(4))
            .unwrap();
        assert!(nav.clear(from, from + direction));
        assert!(direction != FixedVec2::ZERO);
    }
}
