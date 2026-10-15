//! Repères au sol dérivés de l'état courant, exclusivement dans la présentation.
use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;

use crate::{
    global_asset::GlobalAsset,
    powerups::{PowerUpPickup, PowerUpsConfig},
};

pub struct PowerUpVisualPlugin;

impl Plugin for PowerUpVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_powerup_visuals);
    }
}

#[derive(Component)]
struct PowerUpLabel(Entity);

fn sync_powerup_visuals(
    mut commands: Commands,
    pickups: Query<(Entity, &PowerUpPickup, &FixedTransform3D)>,
    mut labels: Query<(Entity, &PowerUpLabel, &mut Transform, &mut Visibility)>,
    global: Res<GlobalAsset>,
    configs: Res<Assets<PowerUpsConfig>>,
    assets: Res<AssetServer>,
    mut gizmos: Gizmos,
    fog: Res<crate::room_fog::FogView>,
    map: Option<Res<crate::room_fog::FogMap>>,
) {
    let config = global.powerups_config.as_ref().and_then(|h| configs.get(h));
    // Les entités visuelles sont indépendantes des entités rollback : un retrait,
    // une expiration ou une résurrection se reflète au prochain Update.
    for (entity, label, mut transform, mut visibility) in &mut labels {
        if let Ok((_, _, position)) = pickups.get(label.0) {
            let pos = position.to_bevy_transform().translation;
            transform.translation = Vec3::new(pos.x, pos.y + 16.0, 50.0);
            *visibility = if fog.sees(map.as_deref(), pos.truncate()) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        } else {
            commands.entity(entity).despawn();
        }
    }
    for (entity, pickup, position) in &pickups {
        let pos = position.to_bevy_transform().translation.truncate();
        if !fog.sees(map.as_deref(), pos) {
            continue;
        }
        let color = Color::srgb(0.3, 1.0, 0.65);
        gizmos.circle_2d(pos, 10.0, color);
        gizmos.line_2d(pos - Vec2::X * 5.0, pos + Vec2::X * 5.0, color);
        gizmos.line_2d(pos - Vec2::Y * 5.0, pos + Vec2::Y * 5.0, color);
        if labels.iter().any(|(_, label, _, _)| label.0 == entity) {
            continue;
        }
        let name = config
            .and_then(|c| c.powerups.get(&pickup.id))
            .map_or(pickup.id.as_str(), |def| def.name.as_str());
        commands.spawn((
            PowerUpLabel(entity),
            Text2d::new(name),
            TextFont {
                font: assets.load("fonts/FiraMono-Medium.ttf").into(),
                font_size: FontSize::Px(10.0),
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(pos.x, pos.y + 16.0, 50.0),
        ));
    }
}
