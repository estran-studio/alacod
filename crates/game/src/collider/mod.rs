use crate::collision_grid::{
    maybe_rebuild_wall_grid, rebuild_character_grid_post_movement,
    rebuild_character_grid_pre_movement, CollisionGrids,
};
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::{GgrsSchedule, Rollback};

#[cfg(feature = "lighting")]
use bevy_light_2d::light::PointLight2d;

pub use combat::collider::{
    is_colliding, slide_axes, Collider, ColliderConfig, ColliderShape, CollisionLayer,
    CollisionSettings, Wall, Window,
};
use utils::net_id::GgrsNetId;

pub mod debug;

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

        // Grilles spatiales dérivées (T2.1, chantier B4b) : ressource non rollback, non
        // checksum (voir `crate::collision_grid`), reconstruite à deux points du planning.
        app.init_resource::<CollisionGrids>();
        app.add_systems(
            GgrsSchedule,
            (
                (maybe_rebuild_wall_grid, rebuild_character_grid_pre_movement)
                    .chain()
                    .after(RollbackSystemSet::Interaction)
                    .before(RollbackSystemSet::Movement),
                rebuild_character_grid_post_movement
                    .after(RollbackSystemSet::Movement)
                    .before(RollbackSystemSet::Weapon),
            ),
        );
    }
}
