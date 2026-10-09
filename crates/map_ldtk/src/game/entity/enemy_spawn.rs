use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use map::game::entity::map::{enemy_spawn::EnemySpawnerComponent, map_rollback::MapRollbackMarker};

use crate::map_const;

pub fn enemy_spawner_component_from_field(
    entity_instance: &EntityInstance,
) -> EnemySpawnerComponent {
    // Optional defence-room binding for exterior wave sources (LDtk pixels, top-down).
    let mut config = EnemySpawnerComponent::default();
    if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
        entity_instance.get_int_field("active_x"),
        entity_instance.get_int_field("active_y"),
        entity_instance.get_int_field("active_width"),
        entity_instance.get_int_field("active_height"),
    ) {
        if *w > 0 && *h > 0 {
            use bevy_fixed::fixed_math::{Fixed, FixedVec2};
            config.activation_area = Some((
                FixedVec2::new(Fixed::from_num(*x), Fixed::from_num(-*y - *h)),
                FixedVec2::new(Fixed::from_num(*w), Fixed::from_num(*h)),
            ));
        }
    }
    config
}

pub fn room_fog_from_field(
    entity_instance: &EntityInstance,
) -> map::game::entity::map::room::RoomFog {
    use map::game::entity::map::room::{RoomBounds, RoomFog};
    RoomFog(
        enemy_spawner_component_from_field(entity_instance)
            .activation_area
            .map(|(position, size)| RoomBounds { position, size }),
    )
}

#[derive(Bundle, LdtkEntity)]
pub struct EnemySpawnBundle {
    #[with(enemy_spawner_component_from_field)]
    spawner: EnemySpawnerComponent,
    #[with(room_fog_from_field)]
    fog: map::game::entity::map::room::RoomFog,
    rollback_marker: MapRollbackMarker,
}

impl Default for EnemySpawnBundle {
    fn default() -> Self {
        Self {
            rollback_marker: MapRollbackMarker("enemy_spawn".into()),
            spawner: EnemySpawnerComponent::default(),
            fog: Default::default(),
        }
    }
}
