use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use map::game::entity::map::map_rollback::MapRollbackMarker;

use crate::map_const;
use map::game::entity::map::door::{DoorComponent, DoorGridPosition};
use map::generation::entity::door::DoorConfig;

#[derive(Component, Clone, Copy)]
pub struct LdtkEntitySize {
    pub width: f32,
    pub height: f32,
}

pub fn ldtk_entity_size_from_instance(entity_instance: &EntityInstance) -> LdtkEntitySize {
    LdtkEntitySize {
        width: entity_instance.width as f32,
        height: entity_instance.height as f32,
    }
}

pub fn door_grid_position_from_instance(entity_instance: &EntityInstance) -> DoorGridPosition {
    DoorGridPosition {
        level_iid: String::new(), // Will be populated by a system later
        grid_x: entity_instance.grid.x,
        grid_y: entity_instance.grid.y,
    }
}

pub fn door_component_from_field(entity_instance: &EntityInstance) -> DoorComponent {
    let interactable = entity_instance
        .get_bool_field(map_const::FIELD_INTERACTABLE_NAME)
        .ok()
        .copied()
        .unwrap_or(true); // Default to true if field doesn't exist

    // Try to read paired door information
    let paired_door = match (
        entity_instance
            .get_int_field(map_const::FIELD_PAIRED_DOOR_X_NAME)
            .ok()
            .copied(),
        entity_instance
            .get_int_field(map_const::FIELD_PAIRED_DOOR_Y_NAME)
            .ok()
            .copied(),
        entity_instance
            .get_string_field(map_const::FIELD_PAIRED_DOOR_LEVEL_NAME)
            .ok(),
    ) {
        (Some(paired_x), Some(paired_y), Some(paired_level)) => {
            Some((paired_level.clone(), (paired_x, paired_y)))
        }
        _ => None,
    };

    DoorComponent {
        config: DoorConfig {
            electrify: *entity_instance
                .get_bool_field(map_const::FIELD_ELECTRIFY_NAME)
                .unwrap(),
            cost: *entity_instance
                .get_int_field(map_const::FIELD_PRICE_NAME)
                .unwrap(),
            interactable,
            paired_door,
        },
    }
}

#[derive(Bundle, LdtkEntity)]
pub struct DoorBundle {
    #[with(door_component_from_field)]
    door: DoorComponent,
    #[with(door_grid_position_from_instance)]
    grid_position: DoorGridPosition,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    rollback_marker: MapRollbackMarker,
    visibility: Visibility,
    #[with(ldtk_entity_size_from_instance)]
    ldtk_size: LdtkEntitySize,
}

impl Default for DoorBundle {
    fn default() -> Self {
        Self {
            rollback_marker: MapRollbackMarker("door".into()),
            door: DoorComponent::default(),
            grid_position: DoorGridPosition {
                level_iid: String::new(),
                grid_x: 0,
                grid_y: 0,
            },
            sprite_sheet: Sprite::default(),
            visibility: Visibility::default(),
            ldtk_size: LdtkEntitySize {
                width: 64.0,
                height: 32.0,
            },
        }
    }
}
