//! Profil de méta-progression (M2-T0c, `docs/conventions.md` §38) : colle entre `meta` et la
//! partie. **Hors simulation** : un système d'`Update` lit `Run` (jamais `GgrsSchedule`) et, à la
//! fin de la run, crédite le [`meta::Profile`] de chaque joueur local puis l'écrit. La
//! simulation ne lit jamais [`meta::Profiles`] : un déblocage entre dans la run par
//! [`items::RunUnlocks`], calculé **une fois** au lancement (voir [`local_run_unlocks`] et
//! [`online_run_unlocks`], `docs/conventions.md` §51).

use std::path::PathBuf;

use bevy::prelude::*;
use items::{RunUnlocks, UnlockDef};
use meta::{local_id, MetaRules, Profile, Profiles, PurchaseError, PurchaseTerms};
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

/// Variable d'environnement de la configuration de session en ligne : déblocages de la partie
/// (`a,b,c`), ceux de l'hôte. Elle joue pour eux le rôle de la graine : posée à l'identique chez
/// tous les pairs au lancement (la synchronisation automatique des profils viendra avec la
/// pubkey allumette). Absente : aucun déblocage.
pub const RUN_UNLOCKS_ENV: &str = "ALACOD_RUN_UNLOCKS";

/// Partie locale : l'union des déblocages des profils des joueurs de la partie (`handles`).
/// Un profil absent ne débloque rien. Seuls les identifiants que `known` reconnaît (les `Unlock`
/// du jeu) comptent : les jalons du profil (`first_run`, …) grossissent à chaque fin de run, et
/// les relire à la relance ferait différer l'état dès la frame 0.
pub fn local_run_unlocks(
    profiles: Option<&Profiles>,
    handles: &[usize],
    known: impl Fn(&str) -> bool,
) -> RunUnlocks {
    let Some(profiles) = profiles else {
        return RunUnlocks::default();
    };
    RunUnlocks(
        handles
            .iter()
            .filter_map(|handle| profiles.0.get(&local_id(*handle)))
            .flat_map(|profile| profile.unlocks.iter())
            .filter(|id| known(id))
            .cloned()
            .collect(),
    )
}

/// Partie en ligne : les déblocages de la configuration de session (`spec` = valeur de
/// [`RUN_UNLOCKS_ENV`], séparés par des virgules), jamais le profil du pair local : deux pairs
/// aux profils différents jouent le même ensemble.
pub fn online_run_unlocks(spec: Option<&str>) -> RunUnlocks {
    RunUnlocks(
        spec.unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

/// Achète `id` pour `profile` hors simulation (menu, `alacod-profile unlock`, scénario).
pub fn purchase_unlock(
    profile: &mut Profile,
    id: &str,
    def: &UnlockDef,
) -> Result<(), PurchaseError> {
    profile.purchase(
        id,
        &PurchaseTerms {
            cost: def.cost,
            requires: &def.requires,
            stat: def.stat.as_ref().map(|(stat, need)| (stat.as_str(), *need)),
        },
    )
}

/// Achat d'un joueur du hub (M2-T13) : débite et écrit son profil tout de suite (hors
/// simulation) ; le déblocage n'entre dans une run qu'au prochain lancement (`RunUnlocks`, §51).
pub fn purchase_for_handle(
    profiles: &mut Profiles,
    settings: &ProfileSettings,
    handle: usize,
    id: &str,
    def: &UnlockDef,
) -> Result<(), PurchaseError> {
    let profile_id = local_id(handle);
    let profile = profiles
        .0
        .entry(profile_id.clone())
        .or_insert_with(|| Profile::new(profile_id));
    purchase_unlock(profile, id, def)?;
    if let Some(dir) = &settings.dir {
        if let Err(e) = meta::save(dir, profile) {
            warn!("profil {} non écrit : {e}", profile.id);
        }
    }
    Ok(())
}

/// Crédite et écrit les profils quand la run se termine (une seule fois par run).
pub fn write_profiles_at_run_end(
    run: Option<Res<Run>>,
    settings: Res<ProfileSettings>,
    mut profiles: ResMut<Profiles>,
    rules: Res<MetaRules>,
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
        profile.apply_run_with(&summary, &rules.0);
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
            .init_resource::<MetaRules>()
            .add_systems(Update, write_profiles_at_run_end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profiles() -> Profiles {
        let mut a = Profile::new(local_id(0));
        a.unlocks.insert("x".into());
        let mut b = Profile::new(local_id(1));
        b.unlocks.extend(["y".to_string(), "x".to_string()]);
        Profiles([a, b].into_iter().map(|p| (p.id.clone(), p)).collect())
    }

    #[test]
    fn local_is_the_union_of_the_players_profiles() {
        let profiles = profiles();
        let ids = |u: RunUnlocks| u.0.into_iter().collect::<Vec<_>>();
        assert_eq!(
            ids(local_run_unlocks(Some(&profiles), &[0], |_| true)),
            ["x"]
        );
        assert_eq!(
            ids(local_run_unlocks(Some(&profiles), &[0, 1], |_| true)),
            ["x", "y"]
        );
        assert_eq!(
            ids(local_run_unlocks(Some(&profiles), &[2], |_| true)),
            Vec::<String>::new()
        );
        assert!(local_run_unlocks(None, &[0], |_| true).0.is_empty());
    }

    #[test]
    fn local_keeps_only_known_unlocks() {
        let profiles = profiles();
        let ids = local_run_unlocks(Some(&profiles), &[0, 1], |id| id == "y").0;
        assert_eq!(ids.into_iter().collect::<Vec<_>>(), ["y"]);
    }

    #[test]
    fn online_ignores_local_profiles() {
        assert!(online_run_unlocks(None).0.is_empty());
        assert!(online_run_unlocks(Some("")).0.is_empty());
        let set = online_run_unlocks(Some("b, a,,b"));
        assert_eq!(set.0.into_iter().collect::<Vec<_>>(), ["a", "b"]);
    }
}
