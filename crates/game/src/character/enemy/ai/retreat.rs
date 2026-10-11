//! Retreat uses the same passable edges and collider margins as chase navigation.
use super::navigation::{AgentBody, AgentSize, FlowFieldCache, GridPos, NavKey};
use bevy_fixed::fixed_math::{FixedVec2, FixedWide};

impl FlowFieldCache {
    /// Move toward a reachable cell farther from the targets. At a local maximum,
    /// settle inside the current cell rather than pushing directly into the wall.
    pub fn retreat_direction(
        &self,
        key: NavKey,
        pos: FixedVec2,
        body: &AgentBody,
    ) -> Option<FixedVec2> {
        let field = self.get_flow_field(key)?;
        let here = GridPos::from_fixed(pos);
        let here_cost = *field.costs.get(&here)?;
        let blocked = |x, y| self.is_blocked(&GridPos::new(x, y), key.profile);
        let mut best: Option<(u32, GridPos)> = None;
        for neighbor in here.neighbors_8() {
            let Some(&cost) = field.costs.get(&neighbor) else {
                continue;
            };
            if cost <= here_cost
                || self.is_blocked_for(&neighbor, key)
                || self.is_too_narrow_for(&neighbor, key)
            {
                continue;
            }
            let (dx, dy) = (neighbor.x - here.x, neighbor.y - here.y);
            if dx != 0 && dy != 0 {
                if world::nav::diagonal_cuts_corner(
                    blocked,
                    here.x,
                    here.y,
                    dx,
                    dy,
                    key.size == AgentSize::Large,
                ) {
                    continue;
                }
            } else if key.size == AgentSize::Small
                && world::nav::chicane_step(blocked, here.x, here.y, dx, dy)
            {
                continue;
            }
            if best.is_none_or(|(best_cost, cell)| {
                cost > best_cost || (cost == best_cost && neighbor < cell)
            }) {
                best = Some((cost, neighbor));
            }
        }
        let cell = best.map_or(here, |(_, cell)| cell);
        let direction = self.steering_point(cell, key.profile, body) - pos;
        let distance = direction.length_squared();
        // One unit of arrival tolerance avoids oscillating around a fixed-point center.
        let arrived = best.is_none() && distance <= FixedWide::from_num(1);
        (!arrived && distance > FixedWide::ZERO).then(|| direction.normalize_or_zero())
    }
}

#[cfg(test)]
mod tests {
    use super::super::navigation::{FlowField, MOVEMENT_FLOW_KEY};
    use super::*;
    use bevy_fixed::fixed_math::{Fixed, FixedVec3};
    use combat::collider::{Collider, ColliderShape};

    pub fn body() -> AgentBody {
        AgentBody::from_collider(&Collider {
            shape: ColliderShape::Rectangle {
                width: Fixed::from_num(20),
                height: Fixed::from_num(20),
            },
            offset: FixedVec3::new(Fixed::ZERO, Fixed::from_num(-6), Fixed::ZERO),
        })
    }
    fn cache(costs: &[(i32, i32, u32)], walls: &[(i32, i32)]) -> FlowFieldCache {
        let mut cache = FlowFieldCache::new();
        let mut field = FlowField::default();
        for &(x, y, cost) in costs {
            field.costs.insert(GridPos::new(x, y), cost);
        }
        cache.layers.insert(MOVEMENT_FLOW_KEY, field);
        cache
            .wall_cells
            .extend(walls.iter().map(|&(x, y)| GridPos::new(x, y)));
        cache
    }
    #[test]
    fn retreat_never_cuts_a_corner_even_if_diagonal_cell_has_highest_cost() {
        let cache = cache(&[(1, 1, 10), (2, 2, 50), (1, 2, 30)], &[(2, 1)]);
        let direction = cache
            .retreat_direction(MOVEMENT_FLOW_KEY, GridPos::new(1, 1).to_fixed(), &body())
            .unwrap();
        assert!(
            direction.x <= Fixed::ZERO && direction.y > Fixed::ZERO,
            "{direction:?}"
        );
    }
    #[test]
    fn retreat_does_not_cross_a_chicane() {
        let cache = cache(
            &[(10, 37, 10), (11, 37, 50), (10, 36, 30)],
            &[(10, 38), (11, 36), (12, 36)],
        );
        let direction = cache
            .retreat_direction(MOVEMENT_FLOW_KEY, GridPos::new(10, 37).to_fixed(), &body())
            .unwrap();
        assert!(
            direction.x <= Fixed::ZERO && direction.y < Fixed::ZERO,
            "{direction:?}"
        );
    }
    #[test]
    fn local_maximum_moves_the_pillard_body_clear_of_the_corner() {
        // Native diagnostic corner: wall x592..608, floor rock y0..48.
        let mut walls = Vec::new();
        for y in 0..7 {
            walls.push((37, y));
        }
        for x in 30..37 {
            for y in 0..3 {
                walls.push((x, y));
            }
        }
        let cache = cache(&[(36, 4, 50)], &walls);
        let pos = FixedVec2::new(Fixed::from_num(581), Fixed::from_num(64));
        let direction = cache
            .retreat_direction(MOVEMENT_FLOW_KEY, pos, &body())
            .unwrap();
        // Recenter away from the right wall; no direct vector into either wall.
        assert!(
            direction.x < Fixed::ZERO && direction.y > Fixed::ZERO,
            "{direction:?}"
        );
    }
    #[test]
    fn highest_valid_cost_wins_and_centered_local_maximum_stops() {
        let cache = cache(&[(0, 0, 0), (1, 0, 10), (2, 0, 20)], &[]);
        let direction = cache
            .retreat_direction(MOVEMENT_FLOW_KEY, GridPos::new(1, 0).to_fixed(), &body())
            .unwrap();
        assert!(direction.x > Fixed::ZERO);
        assert_eq!(
            cache.retreat_direction(MOVEMENT_FLOW_KEY, GridPos::new(2, 0).to_fixed(), &body()),
            None
        );
    }
}
