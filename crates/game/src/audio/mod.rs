#[cfg(feature = "harmonium")]
pub mod music;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

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

        // M-7 : musique générative, seulement si le manifeste du jeu déclare `music`.
        #[cfg(feature = "harmonium")]
        music::add_music(app);
    }
}
