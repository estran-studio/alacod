use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use map::game::entity::map::{map_rollback::MapRollbackMarker, player_spawn::PlayerSpawnConfig};

use crate::map_const;

pub fn player_spawn_component_from_field(entity_instance: &EntityInstance) -> PlayerSpawnConfig {
    println!("player spawn {:?}", entity_instance.field_instances);
    PlayerSpawnConfig {
        index: *entity_instance
            .get_int_field(map_const::FIELD_PLAYER_SPAWN_INDEX_NAME)
            .unwrap() as usize,
    }
}

#[derive(Bundle, LdtkEntity)]
pub struct PlayerSpawnBundle {
    #[with(player_spawn_component_from_field)]
    player_spawn: PlayerSpawnConfig,
    rollback_marker: MapRollbackMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl Default for PlayerSpawnBundle {
    fn default() -> Self {
        Self {
            rollback_marker: MapRollbackMarker("p_spawn".into()),
            player_spawn: PlayerSpawnConfig::default(),
            sprite_sheet: Sprite::default(),
        }
    }
}
