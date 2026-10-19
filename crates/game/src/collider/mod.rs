use crate::collision_grid::{
    maybe_rebuild_wall_grid, rebuild_character_grid_post_movement,
    rebuild_character_grid_pre_movement, CollisionGrids,
};
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;
use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;

pub use combat::collider::{
    is_colliding, overlap_area, slide_axes, step_blocked_by, Collider, ColliderConfig,
    ColliderShape, CollisionLayer, CollisionSettings, Wall, Window,
};
pub mod debug;
pub mod test_wall;

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
