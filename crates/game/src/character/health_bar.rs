use bevy::prelude::*;
use bevy_fixed::fixed_math;

use crate::character::health::Health;

#[derive(Component)]
pub struct HealthBar;

/*
pub fn setup_health_bars(
    mut commands: Commands,
    query: Query<(Entity, Option<&Enemy>), Added<Health>>,
) {
    for (entity, opt_enemy) in query.iter() {
        // Create a health bar entity as a child of the entity
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                HealthBar,
                Sprite {
                    color: opt_enemy.map_or(LinearRgba::GREEN, |_| LinearRgba::RED ).into(),
                    custom_size: Some(Vec2::new(30.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 10.0, 0.1)),
            ));
        });
    }
}
*/

pub fn update_health_bars(
    health_query: Query<(&Health, &Children)>,
    mut health_bar_query: Query<(&mut Transform, &mut Sprite), With<HealthBar>>,
) {
    for (health, children) in health_query.iter() {
        for child in children.iter() {
            if let Ok((_transform, mut sprite)) = health_bar_query.get_mut(child.clone()) {
                // Update the health bar size based on current/max health
                let health_ratio = health.current / health.max;
                sprite.custom_size = Some(Vec2::new(30.0 * fixed_math::to_f32(health_ratio), 3.0));
            }
        }
    }
}
