use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::{collider::create_wall_colliders_from_ldtk, plugin::LdtkMapLoadingPlugin};

use super::{
    game::{
        entity::{
            character_spawn::CharacterSpawnBundle, door::DoorBundle, enemy_spawn::EnemySpawnBundle,
            player_spawn::PlayerSpawnBundle, soda_location::SodaLocationBundle,
            weapon_location::WeaponLocationBundle, window::WindowBundle,
        },
        system::add_level_components::add_room_component_to_ldtk_level,
    },
    map_const,
};

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (add_room_component_to_ldtk_level.run_if(on_message::<LevelEvent>),),
        )
        .register_ldtk_entity::<PlayerSpawnBundle>(map_const::ENTITY_PLAYER_SPAWN_LOCATION)
        .register_ldtk_entity::<EnemySpawnBundle>(map_const::ENTITY_ZOMBIE_SPAWN_LOCATION)
        .register_ldtk_entity::<WindowBundle>(map_const::ENTITY_WINDOW_HORIZONTAL_LOCATION)
        .register_ldtk_entity::<WindowBundle>(map_const::ENTITY_WINDOW_VERTICAL_LOCATION)
        .register_ldtk_entity::<DoorBundle>(map_const::ENTITY_DOOR_HORIZONTAL_LOCATION)
        .register_ldtk_entity::<DoorBundle>(map_const::ENTITY_DOOR_VERTICAL_LOCATION)
        .register_ldtk_entity::<CharacterSpawnBundle>(map_const::ENTITY_CHARACTER_SPAWN_LOCATION)
        .register_ldtk_entity::<WeaponLocationBundle>(map_const::ENTITY_WEAPON_LOCATION)
        .register_ldtk_entity::<SodaLocationBundle>(map_const::ENTITY_SODA_LOCATION);
    }
}

pub struct LdtkRoguePlugin;

impl Plugin for LdtkRoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .add_plugins(LdtkMapLoadingPlugin)
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: false,
                },
                ..default()
            })
            .add_plugins(EntityPlugin);
    }
}
