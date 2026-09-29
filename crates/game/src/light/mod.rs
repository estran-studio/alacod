use bevy::prelude::*;

#[cfg(feature = "lighting")]
use bevy_light_2d::plugin::Light2dPlugin;

#[derive(Default)]
pub struct ZLightPlugin;

impl Plugin for ZLightPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "lighting")]
        {
            app.add_plugins(Light2dPlugin);
        }

        #[cfg(not(feature = "lighting"))]
        {
            // Lighting is disabled - no plugin added
            info!("Lighting system is disabled (feature 'lighting' not enabled)");
        }
    }
}
