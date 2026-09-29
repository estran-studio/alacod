use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use map::game::entity::map::{
    character_spawn::CharacterSpawnComponent, map_rollback::MapRollbackMarker,
};

use crate::map_const;

pub fn character_spawn_component_from_field(
    entity_instance: &EntityInstance,
) -> CharacterSpawnComponent {
    let character = entity_instance
        .get_string_field(map_const::FIELD_CHARACTER_NAME)
        .ok()
        .cloned()
        .unwrap_or_default();
    let team = entity_instance
        .get_string_field(map_const::FIELD_TEAM_NAME)
        .ok()
        .cloned()
        .filter(|s| !s.is_empty());
    CharacterSpawnComponent { character, team }
}

#[derive(Bundle, LdtkEntity)]
pub struct CharacterSpawnBundle {
    #[with(character_spawn_component_from_field)]
    spawn: CharacterSpawnComponent,
    rollback_marker: MapRollbackMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl Default for CharacterSpawnBundle {
    fn default() -> Self {
        Self {
            rollback_marker: MapRollbackMarker("character_spawn".into()),
            spawn: CharacterSpawnComponent::default(),
            sprite_sheet: Sprite::default(),
        }
    }
}
