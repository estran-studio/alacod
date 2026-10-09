//! Shared room fog, derived from current state without modifying rollback simulation.
use crate::{character::player::Player, collider::Collider};
use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use map::game::entity::map::{door::DoorComponent, player_spawn::PlayerSpawnConfig, room::RoomFog};

pub struct RoomFogPlugin;
impl Plugin for RoomFogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_room_fog);
    }
}

#[derive(Component)]
struct FogMask(Entity);

fn contains(rect: (Vec2, Vec2), point: Vec2, margin: f32) -> bool {
    point.cmpge(rect.0 - Vec2::splat(margin)).all()
        && point.cmple(rect.0 + rect.1 + Vec2::splat(margin)).all()
}

fn revealed_rooms(rooms: &[(Vec2, Vec2)], starts: &[Vec2], doors: &[Vec2]) -> Vec<bool> {
    let mut visible: Vec<_> = rooms
        .iter()
        .map(|&r| starts.iter().any(|&p| contains(r, p, 0.0)))
        .collect();
    loop {
        let before = visible.clone();
        for &door in doors {
            // Bounds exclude a 16 px perimeter; adjacent rooms share a wall.
            let neighbours: Vec<_> = rooms
                .iter()
                .enumerate()
                .filter(|(_, &r)| contains(r, door, 24.0))
                .map(|(i, _)| i)
                .collect();
            if neighbours.iter().any(|&i| visible[i]) {
                for i in neighbours {
                    visible[i] = true;
                }
            }
        }
        if visible == before {
            return visible;
        }
    }
}

fn sync_room_fog(
    mut commands: Commands,
    rooms: Query<(Entity, &RoomFog)>,
    players: Query<&FixedTransform3D, With<Player>>,
    spawns: Query<&GlobalTransform, With<PlayerSpawnConfig>>,
    doors: Query<(&FixedTransform3D, Option<&Collider>), With<DoorComponent>>,
    mut masks: Query<(Entity, &FogMask, &mut Visibility)>,
) {
    let bounds: Vec<_> = rooms
        .iter()
        .filter_map(|(e, fog)| {
            fog.0.map(|b| {
                (
                    e,
                    (
                        Vec2::new(b.position.x.to_num(), b.position.y.to_num()),
                        Vec2::new(b.size.x.to_num(), b.size.y.to_num()),
                    ),
                )
            })
        })
        .collect();
    let mut starts: Vec<_> = players
        .iter()
        .map(|p| p.to_bevy_transform().translation.truncate())
        .collect();
    starts.extend(spawns.iter().map(|p| p.translation().truncate()));
    let open: Vec<_> = doors
        .iter()
        .filter(|(_, collider)| collider.is_none())
        .map(|(p, _)| p.to_bevy_transform().translation.truncate())
        .collect();
    let visible = revealed_rooms(
        &bounds.iter().map(|(_, b)| *b).collect::<Vec<_>>(),
        &starts,
        &open,
    );
    for (entity, mask, mut visibility) in &mut masks {
        if let Some(i) = bounds.iter().position(|(e, _)| *e == mask.0) {
            *visibility = if visible[i] {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        } else {
            commands.entity(entity).despawn();
        }
    }
    for (i, &(entity, (min, size))) in bounds.iter().enumerate() {
        if masks.iter().any(|(_, m, _)| m.0 == entity) {
            continue;
        }
        // Opaque overlay above terrain, actors, purchases and world-space labels.
        // Leave shared wall/door faces visible so exits can still be identified.
        let center = min + size * 0.5;
        commands.spawn((
            FogMask(entity),
            Sprite::from_color(Color::BLACK, size + Vec2::splat(16.0)),
            Transform::from_xyz(center.x, center.y, 200.0),
            if visible[i] {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_rooms_hidden_open_connections_reveal_and_rollback_hides() {
        let rooms = [
            (Vec2::ZERO, Vec2::new(352.0, 288.0)),
            (Vec2::new(368.0, 0.0), Vec2::new(352.0, 288.0)),
            (Vec2::new(736.0, 0.0), Vec2::new(352.0, 288.0)),
        ];
        let start = [Vec2::new(100.0, 100.0)];
        assert_eq!(revealed_rooms(&rooms, &start, &[]), [true, false, false]);
        let door = Vec2::new(360.0, 140.0);
        assert_eq!(revealed_rooms(&rooms, &start, &[door]), [true, true, false]);
        assert_eq!(
            revealed_rooms(&rooms, &start, &[door, Vec2::new(728.0, 140.0)]),
            [true, true, true]
        );
        assert_eq!(revealed_rooms(&rooms, &start, &[]), [true, false, false]);
        assert_eq!(
            revealed_rooms(&rooms, &[start[0], Vec2::new(800.0, 100.0)], &[]),
            [true, false, true]
        );
    }
}
