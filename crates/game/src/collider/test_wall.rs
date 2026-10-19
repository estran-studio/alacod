//! Mur de test (`Sprite`, lumière sous `lighting`), moitié **application** (D69 étape 2, E4) : sorti de
//! `collider/mod.rs` (simulation) tel quel. Appelants : `map_ldtk::game::collider`, `examples/character_tester.rs`.
//!
//! Chemin final (E9) : `game_app/src/collider/test_wall.rs`.

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;

#[cfg(feature = "lighting")]
use bevy_light_2d::light::PointLight2d;

use super::{Collider, ColliderShape, CollisionLayer, CollisionSettings, Wall};
use utils::net_id::GgrsNetId;

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
