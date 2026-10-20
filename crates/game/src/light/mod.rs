use bevy::prelude::*;

#[cfg(feature = "lighting")]
use bevy_light_2d::{light::PointLight2d, plugin::Light2dPlugin};

/// Lumière de chaque joueur (D69 étape 2, E4) : `create_player` (simulation) ne pose plus `PointLight2d`.
/// `Added<Player>` : posée dès que le joueur existe (aussi pour un joueur distant ou restauré par un rollback).
#[cfg(feature = "lighting")]
pub fn attach_player_light(
    mut commands: Commands,
    added: Query<Entity, Added<crate::character::player::Player>>,
) {
    for entity in &added {
        commands.entity(entity).try_insert(PointLight2d {
            radius: 200.,
            cast_shadows: false,
            falloff: 4.,
            ..default()
        });
    }
}

#[derive(Default)]
pub struct ZLightPlugin;

impl Plugin for ZLightPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "lighting")]
        {
            app.add_plugins(Light2dPlugin);
            app.add_systems(Update, attach_player_light);
        }

        #[cfg(not(feature = "lighting"))]
        {
            // Lighting is disabled - no plugin added
            info!("Lighting system is disabled (feature 'lighting' not enabled)");
        }
    }
}
