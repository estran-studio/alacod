//! Profil de méta-progression (M2-T0c, `docs/conventions.md` §38) : colle entre `meta` et la
//! partie. **Hors simulation** : un système d'`Update` lit `Run` (jamais `GgrsSchedule`) et, à la
//! fin de la run, crédite le [`meta::Profile`] de chaque joueur local puis l'écrit. La
//! simulation ne lit jamais [`meta::Profiles`] (M2-T12 décidera comment un déblocage entre dans
//! une run de façon déterministe).

use std::path::PathBuf;

use bevy::prelude::*;
use meta::{local_id, Profile, Profiles};
use run::Run;

use crate::character::player::LocalPlayer;

/// Où écrire les profils ; `None` : ne rien écrire (profils gardés en mémoire seulement).
#[derive(Resource, Clone, Debug, Default)]
pub struct ProfileSettings {
    pub dir: Option<PathBuf>,
}

impl ProfileSettings {
    /// `ALACOD_PROFILE_DIR` si défini ; sinon le dossier de données de l'utilisateur, sauf en
    /// headless (`ALACOD_HEADLESS`) où rien n'est écrit (tests, scénarios, bots).
    pub fn from_env() -> Self {
        let explicit = std::env::var_os("ALACOD_PROFILE_DIR").is_some_and(|d| !d.is_empty());
        let headless = std::env::var_os("ALACOD_HEADLESS").is_some();
        Self {
            dir: (explicit || !headless).then(meta::profile_dir).flatten(),
        }
    }
}

/// Crédite et écrit les profils quand la run se termine (une seule fois par run).
pub fn write_profiles_at_run_end(
    run: Option<Res<Run>>,
    settings: Res<ProfileSettings>,
    mut profiles: ResMut<Profiles>,
    locals: Query<&crate::character::player::Player, With<LocalPlayer>>,
    mut written: Local<bool>,
) {
    let Some(run) = run else {
        return;
    };
    let Some(summary) = run.summary.filter(|_| !run.is_playing()) else {
        // Nouvelle run (relance) : le prochain `Ended` doit écrire de nouveau.
        *written = false;
        return;
    };
    if *written {
        return;
    }
    *written = true;
    // En ligne, seuls les joueurs locaux ont un profil ici ; sans `LocalPlayer` (partie locale
    // ou entités déjà détruites), tous les joueurs de la run.
    let mut handles: Vec<usize> = locals.iter().map(|p| p.handle).collect();
    if handles.is_empty() {
        handles = run.players.clone();
    }
    handles.sort_unstable();
    handles.dedup();
    for handle in handles {
        let id = local_id(handle);
        let profile = profiles
            .0
            .entry(id.clone())
            .or_insert_with(|| Profile::new(id));
        profile.apply_run(&summary);
        info!(
            "profil {} : essence {}",
            profile.id,
            profile.currency(meta::ESSENCE)
        );
        if let Some(dir) = &settings.dir {
            if let Err(e) = meta::save(dir, profile) {
                warn!("profil {} non écrit : {e}", profile.id);
            }
        }
    }
}

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        // Une `ProfileSettings` déjà posée (runner de scénarios) l'emporte sur l'environnement.
        let settings = app
            .world()
            .get_resource::<ProfileSettings>()
            .cloned()
            .unwrap_or_else(ProfileSettings::from_env);
        // Lecture au lancement (hors rollback) ; un fichier illisible est signalé, jamais écrasé.
        let profiles = settings
            .dir
            .as_deref()
            .map(|dir| {
                let (profiles, errors) = Profiles::load_dir(dir);
                for (path, error) in errors {
                    warn!("{} : {error}", path.display());
                }
                profiles
            })
            .unwrap_or_default();
        app.insert_resource(settings)
            .insert_resource(profiles)
            .add_systems(Update, write_profiles_at_run_end);
    }
}
