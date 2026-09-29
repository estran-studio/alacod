use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::door::{ldtk_entity_size_from_instance, LdtkEntitySize};
use map::{
    game::entity::map::{map_rollback::MapRollbackMarker, window::WindowComponent},
    generation::entity::window::WindowConfig,
};

pub fn window_from_field(_: &EntityInstance) -> WindowComponent {
    WindowComponent {
        config: WindowConfig {},
    }
}

#[derive(Bundle, LdtkEntity)]
pub struct WindowBundle {
    #[with(window_from_field)]
    window: WindowComponent,
    rollback_marker: MapRollbackMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[with(ldtk_entity_size_from_instance)]
    ldtk_size: LdtkEntitySize,
}

impl Default for WindowBundle {
    fn default() -> Self {
        Self {
            rollback_marker: MapRollbackMarker("window".into()),
            window: WindowComponent::default(),
            sprite_sheet: Sprite::default(),
            ldtk_size: LdtkEntitySize {
                width: 16.0,
                height: 16.0,
            },
        }
    }
}
