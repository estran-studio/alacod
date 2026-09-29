// Alternative simple version for crates/game/src/collider/debug.rs
// Use this if you don't want egui or mesh-based rendering

use bevy::{color::palettes::css::*, prelude::*};
use bevy_fixed::fixed_math;

use super::{Collider, ColliderShape, CollisionLayer, Wall};
use crate::character::{enemy::Enemy, player::Player};

/// Resource to control collider debug visualization
#[derive(Resource)]
pub struct ColliderDebugSettings {
    pub enabled: bool,
}

impl Default for ColliderDebugSettings {
    fn default() -> Self {
        Self { enabled: false } // Start disabled
    }
}

/// System to toggle collider debug visualization with F3 key
pub fn toggle_collider_debug(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<ColliderDebugSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        debug_settings.enabled = !debug_settings.enabled;
        info!(
            "Collider debug: {}",
            if debug_settings.enabled { "ON" } else { "OFF" }
        );
    }
}

/// System to draw debug visualization for all colliders
pub fn draw_collider_debug(
    mut gizmos: Gizmos,
    debug_settings: Res<ColliderDebugSettings>,

    // Query for all entities with colliders
    collider_query: Query<(
        &Collider,
        &fixed_math::FixedTransform3D,
        Option<&Wall>,
        Option<&Player>,
        Option<&Enemy>,
        Option<&crate::weapons::Bullet>,
    )>,
) {
    if !debug_settings.enabled {
        return;
    }

    for (collider, fixed_transform, wall, player, enemy, bullet) in collider_query.iter() {
        // Determine color based on entity type
        let color = match (wall, player, enemy, bullet) {
            (Some(_), _, _, _) => ORANGE, // Walls
            (_, Some(_), _, _) => GREEN,  // Players
            (_, _, Some(_), _) => RED,    // Enemies
            (_, _, _, Some(_)) => YELLOW, // Bullets
            _ => PURPLE,                  // Other
        };

        // Calculate world position including collider offset
        let world_pos = fixed_transform.translation + collider.offset;
        let world_pos_bevy = Vec3::new(
            fixed_math::to_f32(world_pos.x),
            fixed_math::to_f32(world_pos.y),
            fixed_math::to_f32(world_pos.z) + 0.1, // Slightly above to avoid z-fighting
        );

        // Draw based on collider shape
        match &collider.shape {
            ColliderShape::Circle { radius } => {
                let radius_f32 = fixed_math::to_f32(*radius);

                // Draw circle outline
                gizmos.circle(
                    Isometry3d::from_translation(world_pos_bevy),
                    radius_f32,
                    color,
                );

                // Draw center point
                gizmos.sphere(Isometry3d::from_translation(world_pos_bevy), 2.0, color);

                // Draw cross for center reference
                let cross_size = radius_f32 * 0.3;
                gizmos.line(
                    world_pos_bevy + Vec3::new(-cross_size, 0.0, 0.0),
                    world_pos_bevy + Vec3::new(cross_size, 0.0, 0.0),
                    color,
                );
                gizmos.line(
                    world_pos_bevy + Vec3::new(0.0, -cross_size, 0.0),
                    world_pos_bevy + Vec3::new(0.0, cross_size, 0.0),
                    color,
                );
            }

            ColliderShape::Rectangle { width, height } => {
                let width_f32 = fixed_math::to_f32(*width);
                let height_f32 = fixed_math::to_f32(*height);

                // Draw rectangle using gizmos
                gizmos.rect(
                    Isometry3d::from_translation(world_pos_bevy),
                    Vec2::new(width_f32, height_f32),
                    color,
                );

                // Draw center point
                gizmos.sphere(Isometry3d::from_translation(world_pos_bevy), 2.0, color);
            }
        }
    }
}

/// Optional: System to display collider count in console
pub fn debug_collider_stats(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    collider_query: Query<(
        &Collider,
        Option<&Wall>,
        Option<&Player>,
        Option<&Enemy>,
        Option<&crate::weapons::Bullet>,
    )>,
) {
    if keyboard_input.just_pressed(KeyCode::F4) {
        let mut wall_count = 0;
        let mut player_count = 0;
        let mut enemy_count = 0;
        let mut bullet_count = 0;
        let mut other_count = 0;

        for (_, wall, player, enemy, bullet) in collider_query.iter() {
            match (wall, player, enemy, bullet) {
                (Some(_), _, _, _) => wall_count += 1,
                (_, Some(_), _, _) => player_count += 1,
                (_, _, Some(_), _) => enemy_count += 1,
                (_, _, _, Some(_)) => bullet_count += 1,
                _ => other_count += 1,
            }
        }

        let total = wall_count + player_count + enemy_count + bullet_count + other_count;

        info!("=== COLLIDER STATS ===");
        info!("Total: {}", total);
        info!("Walls: {}", wall_count);
        info!("Players: {}", player_count);
        info!("Enemies: {}", enemy_count);
        info!("Bullets: {}", bullet_count);
        info!("Other: {}", other_count);
        info!("======================");
    }
}

pub struct DebugColliderGamePlugin;

impl Plugin for DebugColliderGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColliderDebugSettings>();

        app.add_systems(
            Update,
            (
                toggle_collider_debug,
                draw_collider_debug,
                debug_collider_stats, // Optional stats system
            ),
        );
    }
}
