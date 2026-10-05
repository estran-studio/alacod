use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use map::{
    game::entity::map::{
        level_id::LevelId,
        room::{RoomBounds, RoomComponent},
    },
    generation::{entity::room::RoomConfig, LEVEL_PROPERTIES_SPAWN_NAME},
};

pub fn add_room_component_to_ldtk_level(
    mut level_events: MessageReader<LevelEvent>,
    levels: Query<(Entity, &LevelIid)>,
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
    slots: crate::game::floors::FloorSlots,
    mut commands: Commands,
) {
    for level_event in level_events.read() {
        if matches!(level_event, LevelEvent::Spawned(_)) {
            for (entity, level_iid) in levels.iter() {
                // println!("spawn level {} {}", entity, level_iid);

                // T1.8 : le projet du monde de ce niveau (un monde par niveau en `Floors`).
                let Some(world) = slots.world_of(entity) else {
                    continue;
                };
                let Ok(project) = projects.get(world) else {
                    continue;
                };
                let level_data = project_assets
                    .get(project)
                    .expect("project asset should be loaded if levels are spawned")
                    .get_raw_level_by_iid(&level_iid.to_string())
                    .expect("spawned level should exist in the loaded project");

                let is_spawn = level_data
                    .get_bool_field(LEVEL_PROPERTIES_SPAWN_NAME)
                    .expect("level should have non-nullable title string field");

                let room_config = RoomConfig { spawn: *is_spawn };
                // Prototype m1-proto-etage-salles : type de salle (champ de niveau `room_kind`,
                // recopié du gabarit par l'assembleur), absent des cartes existantes.
                if let Ok(Some(kind)) = level_data.get_maybe_string_field("room_kind") {
                    commands
                        .entity(entity)
                        .insert(crate::game::rooms_proto::RoomKindTag(kind.clone()));
                }

                commands.entity(entity).insert((
                    RoomComponent {
                        config: room_config,
                    },
                    RoomBounds {
                        position: FixedVec2::new(
                            fixed_math::Fixed::from_num(level_data.world_x as f32),
                            fixed_math::Fixed::from_num(
                                -level_data.world_y as f32 - level_data.px_hei as f32,
                            ),
                        ),
                        size: FixedVec2::new(
                            fixed_math::Fixed::from_num(level_data.px_wid as f32),
                            fixed_math::Fixed::from_num(level_data.px_hei as f32),
                        ),
                    },
                    LevelId(level_iid.to_string()),
                ));

                debug!(
                    "Added RoomBounds to level {:?} ({}): pos=({}, {}), size=({}, {})",
                    entity,
                    level_iid,
                    Fixed::from_num(level_data.world_x),
                    Fixed::from_num(-level_data.world_y - level_data.px_hei),
                    Fixed::from_num(level_data.px_wid),
                    Fixed::from_num(level_data.px_hei)
                );

                if *is_spawn {
                    println!("found a spawn level: {}", level_iid);
                }
            }
        }
    }
}
