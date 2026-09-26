#[cfg(feature = "harmonium")]
pub mod adaptive;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
#[cfg(feature = "harmonium")]
use harmonium_bevy::HarmoniumPlugin;

#[derive(Component)]
pub struct AudioState {}

pub struct ZAudioPlugin;

impl Plugin for ZAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin);
        app.add_plugins(SpatialAudioPlugin);

        #[cfg(feature = "harmonium")]
        add_harmonium(app);
    }
}

#[cfg(feature = "harmonium")]
fn add_harmonium(app: &mut App) {
    app.add_plugins(HarmoniumPlugin);

    // Initialize adaptive music config
    app.init_resource::<adaptive::AdaptiveMusicConfig>();

    // Setup systems
    app.add_systems(Startup, setup_harmonium_music);

    // Adaptive music system runs in PostUpdate (outside GGRS rollback)
    // This ensures music doesn't affect game state determinism
    app.add_systems(PostUpdate, adaptive::update_adaptive_music);

    #[cfg(debug_assertions)]
    app.add_systems(PostUpdate, adaptive::debug_adaptive_music);
}

// Setup system to initialize generative music
#[cfg(feature = "harmonium")]
fn setup_harmonium_music(mut commands: Commands) {
    use harmonium_bevy::components::{HarmoniumSource, GenerativeConfig, OdinConfig};
    use harmonium_bevy::harmonium_core::sequencer::RhythmMode;

    // Create the music controller entity
    commands.spawn((
        Name::new("Generative Music"),
        HarmoniumSource {
            is_enabled: true,
            config: GenerativeConfig {
                rhythm_mode: RhythmMode::Euclidean,
                tempo: 120.0,
                density: 0.5,
                tension: 0.3,
                steps: 16,
            },
            // Use defaults which loads embedded presets
            synth: OdinConfig::default(),
            manual_visual_params: Default::default(),
        }
    ));
}