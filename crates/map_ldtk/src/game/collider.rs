use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use game::character::enemy::ai::navigation::FlowFieldCache;
use game::collider::{spawn_test_wall, Collider, ColliderShape, CollisionSettings, Wall};
use utils::net_id::GgrsNetIdFactory;

/// System that creates optimized wall colliders from LDTK IntGrid tiles
pub fn create_wall_colliders_from_ldtk(
    mut commands: Commands,
    levels: Query<(Entity, &LevelIid, &Transform)>,
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut flow_field_cache: ResMut<FlowFieldCache>,
) {
    // Collect and sort levels by IID for deterministic order
    let mut sorted_levels: Vec<_> = levels.iter().collect();
    sorted_levels.sort_by(|a, b| a.1.to_string().cmp(&b.1.to_string()));

    let mut total_walls = 0;

    for (_level_entity, level_iid, level_transform) in sorted_levels {
        let project = project_assets
            .get(projects.single().unwrap())
            .expect("project asset should be loaded if levels are spawned");

        let level_data = project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("spawned level should exist in the loaded project");

        // Find the collision layer (assuming it's named "Collision" or similar)
        if let Some(collision_layer) = level_data.layer_instances.as_ref().and_then(|layers| {
            layers.iter().find(|layer| {
                // Adjust this condition to match your collision layer name
                layer.identifier == "Collision" || layer.identifier == "Walls"
            })
        }) {
            let tile_size = collision_layer.grid_size;
            let level_width = (collision_layer.c_wid) as usize;
            let level_height = (collision_layer.c_hei) as usize;

            // Convert IntGrid values to a 2D grid (1 = wall, 0 = empty)
            let grid =
                create_collision_grid(&collision_layer.int_grid_csv, level_width, level_height);

            // Load IntGrid data directly into FlowFieldCache for perfect 1:1 pathfinding
            // This must happen BEFORE generating merged rectangles (which lose tile info)
            flow_field_cache.load_intgrid_walls(
                &grid,
                level_transform.translation.truncate(),
                level_height,
                level_width,
            );

            // Generate optimized rectangles for physics colliders only
            let rectangles = generate_collision_rectangles(&grid);

            // Spawn wall entities for each rectangle
            for rect in rectangles {
                spawn_invisible_wall_collider(
                    &mut commands,
                    &collision_settings,
                    &mut id_factory,
                    rect,
                    tile_size,
                    level_transform.translation.truncate(),
                    level_height,
                );
                total_walls += 1;
            }
        }
    }

    // Initialize flow field cache with wall count so it knows walls are ready
    if total_walls > 0 {
        flow_field_cache.last_wall_entity_count = total_walls;
        info!(
            "LDTK walls created: {} colliders, {} IntGrid cells for flow field",
            total_walls,
            flow_field_cache.intgrid_wall_cells.len()
        );
    }
}

/// Represents a collision rectangle in tile coordinates
#[derive(Debug, Clone, Copy)]
struct CollisionRect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

/// Convert LDTK IntGrid CSV data to a 2D boolean grid
fn create_collision_grid(int_grid_csv: &[i32], width: usize, height: usize) -> Vec<Vec<bool>> {
    let mut grid = vec![vec![false; width]; height];

    for (i, &value) in int_grid_csv.iter().enumerate() {
        let x = i % width;
        let y = i / width;
        if y < height && x < width {
            // Assuming value 1 represents walls, adjust as needed for your setup
            grid[y][x] = value == 1;
        }
    }

    grid
}

/// Generate optimized collision rectangles from a 2D grid using a greedy algorithm
fn generate_collision_rectangles(grid: &[Vec<bool>]) -> Vec<CollisionRect> {
    let height = grid.len();
    if height == 0 {
        return vec![];
    }
    let width = grid[0].len();

    let mut processed = vec![vec![false; width]; height];
    let mut rectangles = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if grid[y][x] && !processed[y][x] {
                // Found an unprocessed wall tile, try to expand it into a rectangle
                let rect = expand_rectangle(grid, &mut processed, x, y, width, height);
                rectangles.push(rect);
            }
        }
    }

    rectangles
}

/// Expand a single tile into the largest possible rectangle
fn expand_rectangle(
    grid: &[Vec<bool>],
    processed: &mut [Vec<bool>],
    start_x: usize,
    start_y: usize,
    grid_width: usize,
    grid_height: usize,
) -> CollisionRect {
    // First, expand horizontally as much as possible
    let mut width = 1;
    while start_x + width < grid_width
        && grid[start_y][start_x + width]
        && !processed[start_y][start_x + width]
    {
        width += 1;
    }

    // Then, expand vertically while maintaining the width
    let mut height = 1;
    'outer: while start_y + height < grid_height {
        // Check if the entire row can be added
        for x in start_x..start_x + width {
            if !grid[start_y + height][x] || processed[start_y + height][x] {
                break 'outer;
            }
        }
        height += 1;
    }

    // Mark all tiles in this rectangle as processed
    for y in start_y..start_y + height {
        for x in start_x..start_x + width {
            processed[y][x] = true;
        }
    }

    CollisionRect {
        x: start_x,
        y: start_y,
        width,
        height,
    }
}

/// Spawn an invisible wall entity with collider for the given rectangle
fn spawn_invisible_wall_collider(
    commands: &mut Commands,
    collision_settings: &CollisionSettings,
    id_factory: &mut GgrsNetIdFactory,
    rect: CollisionRect,
    tile_size: i32,
    level_offset: Vec2,
    level_height: usize,
) {
    let rect_center_x = rect.x as f32 + (rect.width as f32 / 2.0);
    let rect_center_y = rect.y as f32 + (rect.height as f32 / 2.0);

    let world_x = level_offset.x + (rect_center_x * tile_size as f32);
    let flipped_y = (level_height as f32) - rect_center_y;
    let world_y = level_offset.y + (flipped_y * tile_size as f32);
    let size = Vec2::new(
        rect.width as f32 * tile_size as f32,
        rect.height as f32 * tile_size as f32,
    );

    let translation = fixed_math::FixedVec3::new(
        fixed_math::new(world_x),
        fixed_math::new(world_y),
        fixed_math::new(0.0),
    );

    let transform = fixed_math::FixedTransform3D::new(
        translation,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );

    let g_id = id_factory.next(format!("ldtk_wall_{}x{}", rect.width, rect.height));

    commands
        .spawn((
            Wall,
            transform.to_bevy_transform(),
            transform,
            Collider {
                shape: ColliderShape::Rectangle {
                    width: fixed_math::Fixed::from_num(size.x),
                    height: fixed_math::Fixed::from_num(size.y),
                },
                offset: fixed_math::FixedVec3::ZERO,
            },
            game::collider::CollisionLayer(collision_settings.wall_layer),
            g_id,
            //Visibility::Hidden,
        ))
        .insert(Rollback);
}
