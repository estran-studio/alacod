use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::game::{collider::create_wall_colliders_from_ldtk, plugin::LdtkMapLoadingPlugin};

use super::{
    game::{
        entity::{
            character_spawn::CharacterSpawnBundle,
            door::DoorBundle,
            enemy_spawn::EnemySpawnBundle,
            hub_station::HubStationBundle,
            player_spawn::PlayerSpawnBundle,
            prop::{BarrelBundle, TableBundle},
            soda_location::SodaLocationBundle,
            weapon_location::WeaponLocationBundle,
            window::WindowBundle,
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
        .register_ldtk_entity::<SodaLocationBundle>(map_const::ENTITY_SODA_LOCATION)
        .register_ldtk_entity::<HubStationBundle>(map_const::ENTITY_HUB_STATION)
        .register_ldtk_entity::<TableBundle>(map_const::ENTITY_TABLE)
        .register_ldtk_entity::<BarrelBundle>(map_const::ENTITY_BARREL);
    }
}

/// M3-T17 (§82), présentation : la case de carte d'un rocher cassé n'est plus dessinée. La couche IntGrid
/// `Rocks` est une couche de tuiles LDtk (statique) : sans ceci le rocher disparaît de la simulation
/// (collider, `Rock`) mais reste à l'écran. Lit les indices `RockBroken` du journal de feedback (dérivés
/// des `FrameEvents` transitoires), cherche la tuile `IntGridCell` dont le centre est à moins de 4 unités
/// et la rend invisible ; sans rendu (headless) il n'y a aucune tuile et rien ne se passe.
pub fn hide_broken_rock_tiles(
    log: Res<game::feedback::FeedbackLog>,
    mut tiles: Query<(
        &IntGridCell,
        &GlobalTransform,
        &mut bevy_ecs_tilemap::tiles::TileVisible,
    )>,
) {
    for cue in &log.frame_cues {
        if cue.kind != game::feedback::FeedbackKind::RockBroken {
            continue;
        }
        for (_, transform, mut visible) in &mut tiles {
            if transform.translation().truncate().distance(cue.position) < 4.0 {
                visible.0 = false;
            }
        }
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
            .add_plugins(EntityPlugin)
            .add_systems(
                PostUpdate,
                hide_broken_rock_tiles
                    .run_if(resource_exists::<game::feedback::FeedbackLog>)
                    .after(game::feedback_v3::record_v3_cues),
            );
    }
}
