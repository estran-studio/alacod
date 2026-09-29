//! Debug Visualization for Flow Field Navigation
//!
//! Provides visual debugging tools for the flow field system.
//! Toggle with F3 key (configurable).

use bevy::prelude::*;

use super::navigation::{FlowFieldCache, NavProfile, GRID_CELL_SIZE};
use super::state::{EnemyAiConfig, EnemyTarget, MonsterState};
use crate::character::enemy::Enemy;

/// Resource to control flow field debug visualization
#[derive(Resource, Default)]
pub struct FlowFieldDebug {
    /// Whether debug visualization is enabled
    pub enabled: bool,
    /// Show grid cells
    pub show_grid: bool,
    /// Show direction arrows
    pub show_arrows: bool,
    /// Show cost values
    pub show_costs: bool,
    /// Show IntGrid source cells (vs collider-derived)
    pub show_intgrid_source: bool,
    /// Which navigation profile to visualize
    pub profile: NavProfile,
    /// Maximum cells to render (for performance)
    pub max_cells: usize,
    /// Radius around player to render (in grid cells)
    pub render_radius: i32,
}

impl FlowFieldDebug {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_grid: true,
            show_arrows: true,
            show_costs: false,
            show_intgrid_source: false, // Toggle with F6 to show IntGrid vs collider-derived cells
            profile: NavProfile::GroundBreaker, // Default to GroundBreaker (zombie profile)
            max_cells: 10000,           // High limit to show all cells
            render_radius: 100, // Large radius to show full flow field (50 cell search radius * 2)
        }
    }
}

/// System to toggle debug visualization with F3 key
pub fn toggle_flow_field_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ff_debug: ResMut<FlowFieldDebug>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        ff_debug.enabled = !ff_debug.enabled;
        info!(
            "Flow field debug: {}",
            if ff_debug.enabled { "ON" } else { "OFF" }
        );
    }

    // Cycle through profiles with F4
    if keyboard.just_pressed(KeyCode::F4) && ff_debug.enabled {
        ff_debug.profile = match ff_debug.profile {
            NavProfile::Ground => NavProfile::GroundBreaker,
            NavProfile::GroundBreaker => NavProfile::Flying,
            NavProfile::Flying => NavProfile::Phasing,
            NavProfile::Phasing => NavProfile::Ground,
        };
        info!("Flow field profile: {:?}", ff_debug.profile);
    }

    // Toggle IntGrid source visualization with F6
    if keyboard.just_pressed(KeyCode::F6) && ff_debug.enabled {
        ff_debug.show_intgrid_source = !ff_debug.show_intgrid_source;
        info!(
            "IntGrid source visualization: {}",
            if ff_debug.show_intgrid_source {
                "ON"
            } else {
                "OFF"
            }
        );
    }
}

/// System to draw flow field debug visualization
pub fn draw_flow_field_debug(
    ff_debug: Res<FlowFieldDebug>,
    cache: Res<FlowFieldCache>,
    mut gizmos: Gizmos,
) {
    if !ff_debug.enabled {
        return;
    }

    let flow_field = match cache.get_flow_field(ff_debug.profile) {
        Some(ff) => ff,
        None => return,
    };

    let cell_size = GRID_CELL_SIZE as f32;
    let half_cell = cell_size / 2.0;

    // Only render cells within render_radius of the target (player)
    let target = &cache.target_pos;
    let render_radius = ff_debug.render_radius;

    // Get max cost for color gradient (only within visible range)
    let max_cost = flow_field
        .costs
        .iter()
        .filter(|(pos, _)| {
            (pos.x - target.x).abs() <= render_radius && (pos.y - target.y).abs() <= render_radius
        })
        .map(|(_, cost)| *cost)
        .max()
        .unwrap_or(1)
        .max(1) as f32;

    let mut cell_count = 0;

    for (pos, next_pos) in flow_field.directions.iter() {
        // Skip cells outside render radius
        if (pos.x - target.x).abs() > render_radius || (pos.y - target.y).abs() > render_radius {
            continue;
        }

        if cell_count >= ff_debug.max_cells {
            break;
        }
        cell_count += 1;

        let world_x = pos.x as f32 * cell_size + half_cell;
        let world_y = pos.y as f32 * cell_size + half_cell;
        let center = Vec2::new(world_x, world_y);

        // Get cost for color
        let cost = flow_field.costs.get(pos).copied().unwrap_or(0) as f32;
        let t = (cost / max_cost).clamp(0.0, 1.0);

        // Color gradient: green (close) -> yellow -> red (far)
        let color = if t < 0.5 {
            Color::srgb(t * 2.0, 1.0, 0.0)
        } else {
            Color::srgb(1.0, 2.0 - t * 2.0, 0.0)
        };

        // Draw grid cell
        if ff_debug.show_grid {
            gizmos.rect_2d(
                Isometry2d::from_translation(center),
                Vec2::splat(cell_size - 2.0),
                color.with_alpha(0.3),
            );
        }

        // Draw direction arrow
        if ff_debug.show_arrows && pos != next_pos {
            let next_x = next_pos.x as f32 * cell_size + half_cell;
            let next_y = next_pos.y as f32 * cell_size + half_cell;
            let next_center = Vec2::new(next_x, next_y);

            let direction = (next_center - center).normalize_or_zero();
            let arrow_length = cell_size * 0.4;
            let arrow_end = center + direction * arrow_length;

            gizmos.line_2d(center, arrow_end, color);

            // Arrow head
            let perp = Vec2::new(-direction.y, direction.x) * 3.0;
            let head_back = arrow_end - direction * 5.0;
            gizmos.line_2d(arrow_end, head_back + perp, color);
            gizmos.line_2d(arrow_end, head_back - perp, color);
        }
    }

    // Draw blocked cells (only within render radius)
    for pos in cache.wall_cells.iter() {
        // Skip cells outside render radius
        if (pos.x - target.x).abs() > render_radius || (pos.y - target.y).abs() > render_radius {
            continue;
        }

        if cell_count >= ff_debug.max_cells {
            break;
        }
        cell_count += 1;

        let world_x = pos.x as f32 * cell_size + half_cell;
        let world_y = pos.y as f32 * cell_size + half_cell;

        // When show_intgrid_source is enabled, show different colors:
        // - Magenta: IntGrid source cells (perfect 1:1 tile mapping)
        // - Red: Collider-derived cells (dynamic walls like doors)
        let color = if ff_debug.show_intgrid_source {
            if cache.intgrid_wall_cells.contains(pos) {
                Color::srgba(1.0, 0.0, 1.0, 0.5) // Magenta for IntGrid
            } else {
                Color::srgba(1.0, 0.0, 0.0, 0.5) // Red for dynamic walls
            }
        } else {
            Color::srgba(1.0, 0.0, 0.0, 0.5) // Red for all
        };

        gizmos.rect_2d(
            Isometry2d::from_translation(Vec2::new(world_x, world_y)),
            Vec2::splat(cell_size - 2.0),
            color,
        );
    }

    // Draw target
    let target_x = cache.target_pos.x as f32 * cell_size + half_cell;
    let target_y = cache.target_pos.y as f32 * cell_size + half_cell;
    gizmos.circle_2d(
        Isometry2d::from_translation(Vec2::new(target_x, target_y)),
        cell_size * 0.6,
        Color::srgb(0.0, 1.0, 1.0),
    );
}

/// Resource for enemy state debug visualization
#[derive(Resource, Default)]
pub struct EnemyStateDebug {
    pub enabled: bool,
    pub show_aggro_range: bool,
    pub show_attack_range: bool,
    pub show_state: bool,
    pub show_target_line: bool,
}

impl EnemyStateDebug {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_aggro_range: true,
            show_attack_range: true,
            show_state: true,
            show_target_line: true,
        }
    }
}

/// Toggle enemy state debug with F5
pub fn toggle_enemy_state_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state_debug: ResMut<EnemyStateDebug>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        state_debug.enabled = !state_debug.enabled;
        info!(
            "Enemy state debug: {}",
            if state_debug.enabled { "ON" } else { "OFF" }
        );
    }
}

/// Draw enemy state debug visualization
pub fn draw_enemy_state_debug(
    state_debug: Res<EnemyStateDebug>,
    enemy_query: Query<
        (
            &bevy_fixed::fixed_math::FixedTransform3D,
            &EnemyAiConfig,
            &EnemyTarget,
            &MonsterState,
        ),
        With<Enemy>,
    >,
    mut gizmos: Gizmos,
) {
    if !state_debug.enabled {
        return;
    }

    for (transform, ai_config, target, state) in enemy_query.iter() {
        let pos = Vec2::new(
            transform.translation.x.to_num::<f32>(),
            transform.translation.y.to_num::<f32>(),
        );

        // Draw aggro range
        if state_debug.show_aggro_range {
            gizmos.circle_2d(
                Isometry2d::from_translation(pos),
                ai_config.aggro_range.to_num::<f32>(),
                Color::srgba(1.0, 1.0, 0.0, 0.2),
            );
        }

        // Draw attack range
        if state_debug.show_attack_range {
            gizmos.circle_2d(
                Isometry2d::from_translation(pos),
                ai_config.attack_range.to_num::<f32>(),
                Color::srgba(1.0, 0.0, 0.0, 0.3),
            );
        }

        // Draw line to target
        if state_debug.show_target_line {
            if let Some(target_pos) = target.last_known_position {
                let target_vec =
                    Vec2::new(target_pos.x.to_num::<f32>(), target_pos.y.to_num::<f32>());

                let color = match target.target_type {
                    super::state::TargetType::Player => Color::srgb(1.0, 0.0, 0.0),
                    super::state::TargetType::Obstacle => Color::srgb(1.0, 0.5, 0.0),
                    super::state::TargetType::None => Color::srgb(0.5, 0.5, 0.5),
                };

                gizmos.line_2d(pos, target_vec, color);
            }
        }

        // State indicator above enemy
        if state_debug.show_state {
            let indicator_pos = pos + Vec2::new(0.0, 30.0);
            let (color, radius) = match state {
                MonsterState::Idle => (Color::srgb(0.5, 0.5, 0.5), 5.0),
                MonsterState::Chasing => (Color::srgb(1.0, 1.0, 0.0), 6.0),
                MonsterState::Attacking { .. } => (Color::srgb(1.0, 0.0, 0.0), 8.0),
                MonsterState::Stunned { .. } => (Color::srgb(0.0, 0.0, 1.0), 7.0),
                MonsterState::Breaching { .. } => (Color::srgb(1.0, 0.5, 0.0), 7.0),
                MonsterState::Fleeing => (Color::srgb(0.0, 1.0, 1.0), 6.0),
                MonsterState::Dead => (Color::srgb(0.0, 0.0, 0.0), 5.0),
            };
            gizmos.circle_2d(Isometry2d::from_translation(indicator_pos), radius, color);
        }
    }
}
