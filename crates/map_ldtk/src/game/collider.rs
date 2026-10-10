use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use game::character::enemy::ai::navigation::FlowFieldCache;
use game::collider::{spawn_test_wall, Collider, ColliderShape, CollisionSettings, Wall};
use utils::net_id::GgrsNetIdFactory;

use super::floors::FloorSlots;

/// System that creates optimized wall colliders from LDTK IntGrid tiles
///
/// T1.8 : seulement les niveaux du premier monde (emplacement 0, la carte unique hors mode
/// `Floors`) ; les murs des niveaux suivants sont créés au passage du portail
/// (`super::floors`, même fonction [`spawn_level_walls`]).
#[allow(clippy::too_many_arguments)]
pub fn create_wall_colliders_from_ldtk(
    mut commands: Commands,
    levels: Query<(Entity, &LevelIid, &Transform)>,
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut flow_field_cache: ResMut<FlowFieldCache>,
    slots: FloorSlots,
    mut surfaces: ResMut<world::SurfaceGrid>,
) {
    let level_data: Vec<_> = levels
        .iter()
        .filter(|(entity, _, _)| slots.slot_of(*entity) == 0)
        .filter_map(|(entity, iid, transform)| {
            let project = project_assets.get(projects.get(slots.world_of(entity)?).ok()?)?;
            Some((iid, transform, project))
        })
        .collect();
    // T1.7 : surfaces de la carte (vide sans couche `Surfaces`, remise à zéro à chaque partie)
    *surfaces = surface_grid_of_levels(&level_data);
    spawn_level_walls(
        &mut commands,
        level_data,
        &collision_settings,
        &mut id_factory,
        &mut flow_field_cache,
    );
}

/// Murs (colliders rollback) et cases murées du flow field des niveaux donnés (niveau,
/// transform, projet LDtk du monde du niveau), triés par iid de niveau (ordre déterministe).
pub(crate) fn spawn_level_walls(
    commands: &mut Commands,
    mut sorted_levels: Vec<(&LevelIid, &Transform, &LdtkProject)>,
    collision_settings: &CollisionSettings,
    id_factory: &mut GgrsNetIdFactory,
    flow_field_cache: &mut FlowFieldCache,
) {
    // Collect and sort levels by IID for deterministic order
    sorted_levels.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

    let mut total_walls = 0;

    for (level_iid, level_transform, project) in sorted_levels {
        let level_data = project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("spawned level should exist in the loaded project");

        // Immutable presentation data; never registered with rollback or used by combat.
        if let Some(field) = level_data
            .field_instances
            .iter()
            .find(|f| f.identifier == "fog_layout")
        {
            if let bevy_ecs_ldtk::ldtk::FieldValue::String(Some(text)) = &field.value {
                match serde_json::from_str::<game::room_fog::FogMap>(text) {
                    Ok(mut fog) => {
                        fog.origin = [
                            level_transform.translation.x as i32,
                            level_transform.translation.y as i32,
                        ];
                        if let Err(error) = fog.validate() {
                            error!("fog_layout: {error}");
                        } else {
                            commands.insert_resource(fog);
                        }
                    }
                    Err(error) => error!("fog_layout: {error}"),
                }
            }
        }
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
                let name = format!("ldtk_wall_{}x{}", rect.width, rect.height);
                spawn_invisible_wall_collider(
                    commands,
                    collision_settings,
                    id_factory,
                    name,
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

/// Surfaces des niveaux donnés (T1.7, `docs/conventions.md` §26) : couche IntGrid optionnelle
/// `Surfaces`, valeur non nulle = `SurfaceId`, en cases de grille monde (le niveau est aligné
/// sur la grille de 16 depuis m0-v7 : `translation / 16` exact, y retourné comme les murs).
/// Un niveau sans la couche n'ajoute rien.
pub(crate) fn surface_grid_of_levels(
    levels: &[(&LevelIid, &Transform, &LdtkProject)],
) -> world::SurfaceGrid {
    let mut grid = world::SurfaceGrid::default();
    for (level_iid, level_transform, project) in levels {
        let Some(layer) = project
            .get_raw_level_by_iid(&level_iid.to_string())
            .and_then(|level| level.layer_instances.as_ref())
            .and_then(|layers| {
                layers
                    .iter()
                    .find(|l| l.identifier == world::surface::LAYER_SURFACES)
            })
        else {
            continue;
        };
        let size = layer.grid_size;
        let (w, h) = (layer.c_wid, layer.c_hei);
        let origin_x = (level_transform.translation.x as i32).div_euclid(size);
        let origin_y = (level_transform.translation.y as i32).div_euclid(size);
        for (i, &value) in layer.int_grid_csv.iter().enumerate() {
            if value <= 0 || value > u8::MAX as i32 {
                continue;
            }
            let (x, row) = (i as i32 % w, i as i32 / w);
            grid.cells
                .insert((origin_x + x, origin_y + h - 1 - row), value as u8);
        }
    }
    grid
}

/// Murs d'une caverne recréés depuis `world::CellGrid` après une destruction (T1.6) : même
/// fusion gloutonne que le chemin LDtk, niveau à l'origine (`CellGrid`), net ids
/// `cave_wall_<frame>_<i>` (la factory est rollback : numérotation identique au rejeu). Rend le
/// nombre de murs créés.
pub(crate) fn spawn_cave_walls(
    commands: &mut Commands,
    grid: &world::CellGrid,
    collision_settings: &CollisionSettings,
    id_factory: &mut GgrsNetIdFactory,
    frame: u32,
) -> usize {
    let (w, h) = (grid.width as usize, grid.height as usize);
    // Rangées LDtk (haut en bas) : la rangée r est la rangée de grille h - 1 - r
    let rows: Vec<Vec<bool>> = (0..h)
        .map(|r| {
            (0..w)
                .map(|x| {
                    grid.get(x as i32, (h - 1 - r) as i32)
                        .is_some_and(world::CellKind::is_solid)
                })
                .collect()
        })
        .collect();
    let rectangles = generate_collision_rectangles(&rows);
    let count = rectangles.len();
    for (i, rect) in rectangles.into_iter().enumerate() {
        spawn_invisible_wall_collider(
            commands,
            collision_settings,
            id_factory,
            format!("cave_wall_{frame}_{i}"),
            rect,
            world::CELL_SIZE,
            Vec2::ZERO,
            h,
        );
    }
    count
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
    name: String,
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

    let g_id = id_factory.next(name);

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
