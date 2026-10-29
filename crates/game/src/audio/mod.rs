#[cfg(feature = "harmonium")]
pub mod adaptive;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
#[cfg(feature = "harmonium")]
use harmonium_bevy::HarmoniumPlugin;

#[derive(Component)]
pub struct AudioState {}

/// Volume global des sons de feedback, en pourcentage (S1-7), réglé par le curseur de l'écran de
/// pause. Présentation seule : hors rollback, hors trace, appliqué en plus des `volumes` par clé ;
/// `ALACOD_MUTE=1` reste prioritaire. Valeur de la session (pas de fichier de réglages).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MasterVolume(pub u8);

impl Default for MasterVolume {
    fn default() -> Self {
        Self(100)
    }
}

impl MasterVolume {
    /// Décale le curseur de `steps` crans de [`content::feedback::MASTER_VOLUME_STEP`], borné à 0..=100.
    pub fn stepped(self, steps: i8) -> Self {
        let delta = i32::from(steps) * i32::from(content::feedback::MASTER_VOLUME_STEP);
        Self((i32::from(self.0) + delta).clamp(0, 100) as u8)
    }
}

#[cfg(test)]
mod master_volume_tests {
    use super::*;

    #[test]
    fn curseur_borne() {
        assert_eq!(MasterVolume(100).stepped(1), MasterVolume(100));
        assert_eq!(MasterVolume(100).stepped(-1), MasterVolume(90));
        assert_eq!(MasterVolume(5).stepped(-1), MasterVolume(0));
        assert_eq!(MasterVolume(0).stepped(-1), MasterVolume(0));
        assert_eq!(MasterVolume(95).stepped(1), MasterVolume(100));
    }
}

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
    use harmonium_bevy::components::{GenerativeConfig, HarmoniumSource, OdinConfig};
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
        },
    ));
}
