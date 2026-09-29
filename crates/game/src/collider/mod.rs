use crate::rollback::RollbackTraceApp;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;

#[cfg(feature = "lighting")]
use bevy_light_2d::light::PointLight2d;

use serde::{Deserialize, Serialize};
use utils::net_id::GgrsNetId;

pub mod debug;

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub enum ColliderShape {
    Circle {
        radius: fixed_math::Fixed,
    },
    Rectangle {
        width: fixed_math::Fixed,
        height: fixed_math::Fixed,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColliderConfig {
    pub shape: ColliderShape,
    pub offset: fixed_math::FixedVec3,
}

impl From<&ColliderConfig> for Collider {
    fn from(val: &ColliderConfig) -> Self {
        Collider {
            shape: val.shape.clone(),
            offset: val.offset,
        }
    }
}

#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: fixed_math::FixedVec3, // Offset from entity transform
}

#[derive(Component, Clone, Debug, Hash)]
pub struct Wall;

#[derive(Component, Clone, Debug, Hash)]
pub struct Window;

#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct CollisionLayer(pub usize);

#[derive(Resource)]
pub struct CollisionSettings {
    pub enemy_layer: usize,
    pub environment_layer: usize,
    pub player_layer: usize,
    pub wall_layer: usize,
    pub window_layer: usize,
    pub bullet_layer: usize,
    pub layer_matrix: [[bool; 8]; 8], // Collision matrix for which layers collide
}

impl Default for CollisionSettings {
    fn default() -> Self {
        // Initialize empty collision matrix
        let mut layer_matrix = [[false; 8]; 8];

        // Define collision layers
        let enemy_layer = 1;
        let environment_layer = 2;
        let player_layer = 3;
        let wall_layer = 4;
        let window_layer = 5;
        let bullet_layer = 6;

        // Set up collision relationships
        layer_matrix[enemy_layer][wall_layer] = true;
        layer_matrix[wall_layer][enemy_layer] = true;

        layer_matrix[wall_layer][player_layer] = true;
        layer_matrix[player_layer][wall_layer] = true;

        // Player-Enemy collision is "soft" - handled specially in move_characters
        // to allow pushing through but with resistance
        layer_matrix[enemy_layer][player_layer] = true;
        layer_matrix[player_layer][enemy_layer] = true;

        // Window layer only collides with player bodies (not bullets or enemies)
        layer_matrix[window_layer][player_layer] = true;
        layer_matrix[player_layer][window_layer] = true;

        // Bullets collide with walls and enemies, but not windows or players
        layer_matrix[bullet_layer][wall_layer] = true;
        layer_matrix[wall_layer][bullet_layer] = true;
        layer_matrix[bullet_layer][enemy_layer] = true;
        layer_matrix[enemy_layer][bullet_layer] = true;

        Self {
            enemy_layer,
            environment_layer,
            player_layer,
            wall_layer,
            window_layer,
            bullet_layer,
            layer_matrix,
        }
    }
}

pub fn is_colliding(
    pos_a: &fixed_math::FixedVec3,
    collider_a: &Collider,
    pos_b: &fixed_math::FixedVec3,
    collider_b: &Collider,
) -> bool {
    // Assuming collider_a.offset is FixedVec2. If it's FixedVec3, just add directly.
    let final_pos_a = *pos_a + collider_a.offset;
    let final_pos_b = *pos_b + collider_b.offset;

    match (&collider_a.shape, &collider_b.shape) {
        // Circle to Circle
        (
            ColliderShape::Circle { radius: radius_a },
            ColliderShape::Circle { radius: radius_b },
        ) => {
            // (final_pos_a - final_pos_b) is FixedVec3. Its length_squared() returns FixedWide.
            let distance_sq_fw: fixed_math::FixedWide =
                (final_pos_a - final_pos_b).length_squared();

            // Radii are Fixed. Sum them as Fixed, then convert to FixedWide for squaring.
            let combined_radius_fixed = *radius_a + *radius_b;
            let combined_radius_fw =
                fixed_math::FixedWide::from_num(combined_radius_fixed.to_num::<f32>());
            let combined_radius_sq_fw = combined_radius_fw.saturating_mul(combined_radius_fw);

            distance_sq_fw < combined_radius_sq_fw // Compare FixedWide < FixedWide
        }

        // Rectangle to Rectangle (AABB)
        (
            ColliderShape::Rectangle {
                width: width_a,
                height: height_a,
            },
            ColliderShape::Rectangle {
                width: width_b,
                height: height_b,
            },
        ) => {
            // This logic uses Fixed directly and should be fine as it's AABB.
            let two_fx = fixed_math::new(2.0); // Or fixed_math::Fixed::from_num(2)
            let half_width_a = width_a.saturating_div(two_fx);
            let half_height_a = height_a.saturating_div(two_fx);
            let half_width_b = width_b.saturating_div(two_fx);
            let half_height_b = height_b.saturating_div(two_fx);

            let min_a_x = final_pos_a.x - half_width_a;
            let max_a_x = final_pos_a.x + half_width_a;
            let min_a_y = final_pos_a.y - half_height_a; // Assuming 2D collision logic for AABB using Y
            let max_a_y = final_pos_a.y + half_height_a;

            let min_b_x = final_pos_b.x - half_width_b;
            let max_b_x = final_pos_b.x + half_width_b;
            let min_b_y = final_pos_b.y - half_height_b;
            let max_b_y = final_pos_b.y + half_height_b;

            // Check for overlap (standard AABB)
            min_a_x < max_b_x && // Use < for non-inclusive boundary, <= for inclusive
            max_a_x > min_b_x &&
            min_a_y < max_b_y &&
            max_a_y > min_b_y
        }

        // Circle to Rectangle
        (ColliderShape::Circle { radius }, ColliderShape::Rectangle { width, height }) => {
            // Pass FixedVec3 for positions
            circle_rect_collision_fixed(final_pos_a, *radius, final_pos_b, *width, *height)
        }

        // Rectangle to Circle (swap arguments)
        (ColliderShape::Rectangle { width, height }, ColliderShape::Circle { radius }) => {
            // Pass FixedVec3 for positions
            circle_rect_collision_fixed(final_pos_b, *radius, final_pos_a, *width, *height)
        }
    }
}

// Helper function for circle-to-rectangle collision
fn circle_rect_collision_fixed(
    circle_pos_v3: fixed_math::FixedVec3, // Now explicitly FixedVec3
    circle_radius_fixed: fixed_math::Fixed,
    rect_pos_v3: fixed_math::FixedVec3, // Now explicitly FixedVec3
    rect_width_fixed: fixed_math::Fixed,
    rect_height_fixed: fixed_math::Fixed,
) -> bool {
    let two_fx = fixed_math::new(2.0);
    let half_width_fixed = rect_width_fixed.saturating_div(two_fx);
    let half_height_fixed = rect_height_fixed.saturating_div(two_fx);

    // Find the closest point on the rectangle to the circle center (using X and Y components)
    let closest_x_fixed = circle_pos_v3
        .x
        .max(rect_pos_v3.x - half_width_fixed)
        .min(rect_pos_v3.x + half_width_fixed);
    let closest_y_fixed = circle_pos_v3
        .y
        .max(rect_pos_v3.y - half_height_fixed)
        .min(rect_pos_v3.y + half_height_fixed);

    // Calculate distance from circle center's XY projection to closest XY point on rect
    let diff_v2 = fixed_math::FixedVec2::new(
        circle_pos_v3.x - closest_x_fixed,
        circle_pos_v3.y - closest_y_fixed,
    );

    // diff_v2.length_squared() returns FixedWide
    let distance_sq_fw: fixed_math::FixedWide = diff_v2.length_squared();

    // Convert circle_radius (Fixed) to FixedWide for squaring and comparison
    let circle_radius_fw = fixed_math::FixedWide::from_num(circle_radius_fixed.to_num::<f32>());
    let radius_sq_fw = circle_radius_fw.saturating_mul(circle_radius_fw);

    // Compare FixedWide < FixedWide
    distance_sq_fw < radius_sq_fw
}

// test function for wall

pub fn spawn_test_wall(
    commands: &mut Commands,
    position: Vec3,
    size: Vec2,
    collision_settings: &Res<CollisionSettings>,
    color: Color,
    g_id: GgrsNetId,
) {
    let translation = fixed_math::FixedVec3::new(
        fixed_math::new(position.x),
        fixed_math::new(position.y),
        fixed_math::new(position.z),
    );
    let transform = fixed_math::FixedTransform3D::new(
        translation,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );

    let width = size.x;
    let height = size.y;

    #[cfg(feature = "lighting")]
    let diagonal = (width.powi(2) + height.powi(2)).sqrt();
    #[cfg(feature = "lighting")]
    let desired_light_radius = diagonal * 1.5;

    #[cfg(feature = "lighting")]
    {
        commands
            .spawn((
                Wall,
                transform.to_bevy_transform(),
                transform,
                Sprite {
                    color: color.clone(),
                    custom_size: Some(size),
                    ..Default::default()
                },
                Collider {
                    shape: ColliderShape::Rectangle {
                        width: fixed_math::Fixed::from_num(size.x),
                        height: fixed_math::Fixed::from_num(size.y),
                    },
                    offset: fixed_math::FixedVec3::ZERO,
                },
                PointLight2d {
                    radius: desired_light_radius,
                    color: color,
                    intensity: 5.0,
                    falloff: 1.0,
                    ..default()
                },
                CollisionLayer(collision_settings.wall_layer),
                g_id,
            ))
            .insert(Rollback);
    }

    #[cfg(not(feature = "lighting"))]
    {
        commands
            .spawn((
                Wall,
                transform.to_bevy_transform(),
                transform,
                Sprite {
                    color: color.clone(),
                    custom_size: Some(size),
                    ..Default::default()
                },
                Collider {
                    shape: ColliderShape::Rectangle {
                        width: fixed_math::Fixed::from_num(size.x),
                        height: fixed_math::Fixed::from_num(size.y),
                    },
                    offset: fixed_math::FixedVec3::ZERO,
                },
                CollisionLayer(collision_settings.wall_layer),
                g_id,
            ))
            .insert(Rollback);
    }
}

pub struct BaseColliderGamePlugin {}

impl Plugin for BaseColliderGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionSettings>();

        app.rollback_and_trace::<Collider>()
            .rollback_and_trace::<Wall>()
            .rollback_and_trace::<Window>()
            .rollback_and_trace::<CollisionLayer>();
    }
}
