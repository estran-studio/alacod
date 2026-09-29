use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use map::game::entity::map::{enemy_spawn::EnemySpawnerComponent, map_rollback::MapRollbackMarker};

use crate::map_const;

pub fn enemy_spawner_component_from_field(
    _entity_instance: &EntityInstance,
) -> EnemySpawnerComponent {
    // You can customize spawner properties from LDTK fields if needed
    // For now, using default values
    EnemySpawnerComponent::default()
}

#[derive(Bundle, LdtkEntity)]
pub struct EnemySpawnBundle {
    #[with(enemy_spawner_component_from_field)]
    spawner: EnemySpawnerComponent,
    rollback_marker: MapRollbackMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl Default for EnemySpawnBundle {
    fn default() -> Self {
        Self {
            rollback_marker: MapRollbackMarker("enemy_spawn".into()),
            spawner: EnemySpawnerComponent::default(),
            sprite_sheet: Sprite::default(),
        }
    }
}
