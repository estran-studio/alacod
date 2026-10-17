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

/// Une tuile de la couche `Rocks` est visible ssi un rocher vivant est à moins de 4 unités de son centre.
pub fn rock_tile_visible(tile: Vec2, rocks: impl IntoIterator<Item = Vec2>) -> bool {
    rocks
        .into_iter()
        .any(|rock| rock.distance(tile) < ROCK_TILE_RADIUS)
}

const ROCK_TILE_RADIUS: f32 = 4.0;

/// M3-T17 (§82), présentation : la case de carte d'un rocher n'est dessinée que tant que le `Rock` existe.
/// La couche IntGrid `Rocks` est une couche de tuiles LDtk statique : sans ceci un rocher détruit disparaît
/// de la simulation (collider, `Rock`) mais reste à l'écran. Visibilité **dérivée de l'état à chaque tick**
/// (jamais d'un événement) : juste après plusieurs frames simulées par tick et après un rollback qui
/// ressuscite le rocher (les `Rock` désactivés par `despawn_rollback` sont exclus de la requête). Seules les
/// tuiles non nulles de la couche `Rocks` sont touchées ; sans rendu (headless), aucune tuile.
pub fn sync_rock_tiles(
    rocks: Query<&bevy_fixed::fixed_math::FixedTransform3D, With<world::Rock>>,
    layers: Query<&LayerMetadata>,
    mut tiles: Query<(
        &IntGridCell,
        &GlobalTransform,
        &ChildOf,
        &mut bevy_ecs_tilemap::tiles::TileVisible,
    )>,
) {
    let centers: Vec<Vec2> = rocks
        .iter()
        .map(|t| {
            Vec2::new(
                t.translation.x.to_num::<f32>(),
                t.translation.y.to_num::<f32>(),
            )
        })
        .collect();
    for (cell, transform, parent, mut visible) in &mut tiles {
        if cell.value == 0
            || !layers
                .get(parent.parent())
                .is_ok_and(|layer| layer.identifier == world::LAYER_ROCKS)
        {
            continue;
        }
        let wanted = rock_tile_visible(transform.translation().truncate(), centers.iter().copied());
        if visible.0 != wanted {
            visible.0 = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::{Fixed, FixedMat3, FixedTransform3D, FixedVec3};

    fn rock_at(app: &mut App, x: f32, y: f32) -> Entity {
        let t = FixedTransform3D::new(
            FixedVec3::new(Fixed::from_num(x), Fixed::from_num(y), Fixed::ZERO),
            FixedMat3::IDENTITY,
            FixedVec3::ONE,
        );
        app.world_mut()
            .spawn((
                world::Rock {
                    cell: (0, 0),
                    loot: false,
                },
                t,
            ))
            .id()
    }

    fn tile(app: &mut App, layer: &str, x: f32, y: f32) -> Entity {
        let layer = app
            .world_mut()
            .spawn(LayerMetadata {
                identifier: layer.into(),
                ..default()
            })
            .id();
        app.world_mut()
            .spawn((
                IntGridCell { value: 1 },
                GlobalTransform::from_xyz(x, y, 0.0),
                bevy_ecs_tilemap::tiles::TileVisible(true),
                ChildOf(layer),
            ))
            .id()
    }

    fn visible(app: &App, tile: Entity) -> bool {
        app.world()
            .get::<bevy_ecs_tilemap::tiles::TileVisible>(tile)
            .unwrap()
            .0
    }

    /// Rocher retiré (casse) : tuile cachée ; rocher qui réapparaît (rollback) : tuile revue. Une tuile d'une
    /// autre couche n'est jamais touchée.
    #[test]
    fn la_tuile_suit_l_etat_du_rocher() {
        let mut app = App::new();
        app.add_systems(Update, sync_rock_tiles);
        let rock = rock_at(&mut app, 24.0, 40.0);
        let rock_tile = tile(&mut app, "Rocks", 24.0, 40.0);
        let wall_tile = tile(&mut app, "Walls", 24.0, 40.0);
        app.update();
        assert!(visible(&app, rock_tile));
        app.world_mut().despawn(rock);
        app.update();
        assert!(!visible(&app, rock_tile), "rocher cassé : tuile cachée");
        assert!(visible(&app, wall_tile), "autre couche intacte");
        rock_at(&mut app, 24.0, 40.0);
        app.update();
        assert!(visible(&app, rock_tile), "rocher ressuscité : tuile revue");
    }

    #[test]
    fn rayon_de_la_tuile() {
        let tile = Vec2::new(8.0, 8.0);
        assert!(rock_tile_visible(tile, [Vec2::new(10.0, 8.0)]));
        assert!(!rock_tile_visible(tile, [Vec2::new(16.0, 8.0)]));
        assert!(!rock_tile_visible(tile, []));
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
            .add_systems(PostUpdate, sync_rock_tiles);
    }
}
