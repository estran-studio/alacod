//! Profil de méta-progression (M2-T0c, `docs/conventions.md` §38) : colle entre `meta` et la
//! partie. **Hors simulation** : un système d'`Update` lit `Run` (jamais `GgrsSchedule`) et, à la
//! fin de la run, crédite le [`meta::Profile`] de chaque joueur local puis l'écrit. La
//! simulation ne lit jamais [`meta::Profiles`] : un déblocage entre dans la run par
//! [`items::RunUnlocks`], calculé **une fois** au lancement (voir [`local_run_unlocks`] et
//! [`online_run_unlocks`], `docs/conventions.md` §51).
//!
//! Accomplissements (M3-T10, §75) : la simulation n'écrit que [`RunHistory`] (boss vaincus, objets
//! tenus), ressource rollback **sans checksum** que seul ce module lit, à la fin de la run
//! ([`write_achievements_at_run_end`], `Update`) ; les `unlocks` des accomplissements atteints
//! entrent dans le profil et la run suivante les voit par [`RunUnlocks`], sans autre chemin.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bevy::prelude::*;
use items::{Inventory, RunUnlocks, UnlockDef};
use meta::{
    achieved, local_id, Achievements, MetaRules, Profile, Profiles, PurchaseError, PurchaseTerms,
    RunFacts,
};
use run::Run;

use crate::character::player::Player;
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;

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

/// Statistiques du profil de l'hôte (`nom:valeur` séparés par des virgules), avec
/// [`RUN_UNLOCKS_ENV`] : ce que le hub en ligne propose (M2-T13, décision de William : l'hôte
/// décide). Absente : aucune statistique.
pub const HOST_STATS_ENV: &str = "ALACOD_HOST_STATS";

/// Profil de l'hôte reconstitué de la configuration de session : ses déblocages (`unlocks`) et ses
/// statistiques (`stats`). Ni monnaie ni identité : le hub n'en lit que les conditions de service.
pub fn online_host_profile(unlocks: Option<&str>, stats: Option<&str>) -> Profile {
    let mut profile = Profile::new("host");
    profile.unlocks = online_run_unlocks(unlocks).0;
    for entry in stats.unwrap_or_default().split(',') {
        if let Some((name, value)) = entry.split_once(':') {
            if let Ok(value) = value.trim().parse::<u64>() {
                profile.stats.insert(name.trim().to_string(), value);
            }
        }
    }
    profile
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

/// Ce que la run a fait, pour les accomplissements (M3-T10, §75). Écrit par la simulation (le
/// boss meurt dans `boss_defeated_system`, l'inventaire est relu à chaque frame par
/// [`record_held_items`]), lu **seulement** par [`write_achievements_at_run_end`] : la simulation
/// ne le lit jamais.
///
/// Ressource rollback **sans checksum** (`rollback_and_trace_resource_no_checksum`) : un historique
/// écrit pendant une frame prédite est défait par le rollback (un boss « vaincu » par une
/// prédiction ratée ne reste pas), et aucune trace ni aucun checksum ne bouge puisque rien ne le
/// lit en simulation. Remise à zéro à chaque relance (`run_state::cleanup_rollback_world_system`).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct RunHistory {
    /// Personnages-boss vaincus pendant la run (nom du personnage, celui de `GgrsNetId`).
    pub bosses_defeated: BTreeSet<String>,
    /// Objets (passifs et actif) tenus par les joueurs à la dernière frame où l'un d'eux en tenait :
    /// la mort d'un joueur détruit son inventaire, la dernière valeur reste.
    pub items_held: BTreeSet<String>,
}

/// Mémorise les objets tenus par les joueurs (`GgrsSchedule`, dernier set : lit l'état de la frame).
/// N'écrit que si l'ensemble change, et jamais quand aucun joueur n'a d'inventaire (joueurs morts,
/// ou rien ramassé : une valeur vide n'efface pas la dernière connue). Un joueur mort n'efface rien.
pub fn record_held_items(
    run: Option<Res<Run>>,
    mut history: ResMut<RunHistory>,
    inventories: Query<&Inventory, With<Player>>,
) {
    if !run.is_some_and(|run| run.is_playing()) {
        return;
    }
    let mut held: BTreeSet<String> = BTreeSet::new();
    let mut any = false;
    for inventory in &inventories {
        any = true;
        held.extend(inventory.passives.iter().cloned());
        held.extend(inventory.active.iter().map(|slot| slot.item.clone()));
    }
    if any && held != history.items_held {
        history.items_held = held;
    }
}

/// Accomplissements obtenus à la fin de la run en cours, par handle de joueur local (présentation :
/// écran de fin, source `achievements` du HUD). Hors rollback, vidé à la relance.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct AchievementsEarned(pub BTreeMap<usize, Vec<String>>);

impl AchievementsEarned {
    /// Noms affichés des accomplissements du joueur `handle` (le nom du contenu, sinon l'id).
    pub fn names(&self, handle: usize, table: Option<&Achievements>) -> Vec<String> {
        self.0
            .get(&handle)
            .into_iter()
            .flatten()
            .map(|id| {
                table
                    .and_then(|table| table.0.get(id))
                    .map_or(id.as_str(), |def| def.display_name(id))
                    .to_string()
            })
            .collect()
    }

    /// Noms de tous les accomplissements obtenus (écran de fin), un par id, sans doublon entre
    /// joueurs.
    pub fn all_names(&self, table: Option<&Achievements>) -> Vec<String> {
        let ids: BTreeSet<&String> = self.0.values().flatten().collect();
        ids.into_iter()
            .map(|id| {
                table
                    .and_then(|table| table.0.get(id))
                    .map_or(id.as_str(), |def| def.display_name(id))
                    .to_string()
            })
            .collect()
    }
}

/// Joueurs dont le profil reçoit la fin de run : les joueurs locaux ; sans `LocalPlayer` (partie
/// locale ou entités déjà détruites), tous les joueurs de la run.
fn ended_handles(locals: &Query<&Player, With<LocalPlayer>>, run: &Run) -> Vec<usize> {
    let mut handles: Vec<usize> = locals.iter().map(|p| p.handle).collect();
    if handles.is_empty() {
        handles = run.players.clone();
    }
    handles.sort_unstable();
    handles.dedup();
    handles
}

/// Les accomplissements de `table` atteints par `facts` et nouveaux pour `profile` : leurs
/// `unlocks` entrent dans le profil. Renvoie les ids nouvellement obtenus (ordre des ids).
pub fn grant_achievements(
    profile: &mut Profile,
    table: &Achievements,
    facts: &RunFacts,
) -> Vec<String> {
    table
        .0
        .iter()
        .filter(|(_, def)| achieved(facts, def))
        .filter_map(|(id, def)| profile.grant_achievement(id, def).then(|| id.clone()))
        .collect()
}

/// Un résumé est **définitif** quand sa frame de fin est confirmée par tous les pairs : une fin de
/// run vue pendant une prédiction peut être défaite par un rollback, or un accomplissement écrit
/// sur disque ne se reprend pas. Sans session GGRS (`confirmed` absent), toujours définitif.
pub fn summary_is_final(end_frame: u32, confirmed: Option<i32>) -> bool {
    confirmed.is_none_or(|confirmed| i64::from(end_frame) <= i64::from(confirmed))
}

/// Évalue les accomplissements quand la run se termine (`Update`, une seule fois par run, une fois
/// la fin **confirmée** : [`summary_is_final`]) : chaque joueur local reçoit dans son profil les
/// accomplissements atteints et leurs déblocages, le fichier est écrit. Les faits sont ceux de la
/// partie commune ([`RunHistory`], [`run::RunSummary`]) : identiques chez tous les pairs ; en ligne
/// chaque pair n'écrit que le profil de ses joueurs locaux, et la run suivante ne lit que le profil
/// de l'hôte (`RunUnlocks`, §51).
#[allow(clippy::too_many_arguments)]
pub fn write_achievements_at_run_end(
    run: Option<Res<Run>>,
    history: Res<RunHistory>,
    table: Option<Res<Achievements>>,
    settings: Res<ProfileSettings>,
    mut profiles: ResMut<Profiles>,
    mut earned: ResMut<AchievementsEarned>,
    locals: Query<&Player, With<LocalPlayer>>,
    confirmed: Option<Res<bevy_ggrs::ConfirmedFrameCount>>,
    mut done: Local<bool>,
) {
    let Some(run) = run else {
        return;
    };
    let Some(summary) = run.summary.filter(|_| !run.is_playing()) else {
        // Nouvelle run (relance) : le prochain `Ended` évalue de nouveau, l'écran de fin se vide.
        *done = false;
        if !earned.0.is_empty() {
            earned.0.clear();
        }
        return;
    };
    if *done || !summary_is_final(summary.frames, confirmed.as_deref().map(|c| c.0)) {
        return;
    }
    *done = true;
    let Some(table) = table.filter(|table| !table.0.is_empty()) else {
        return;
    };
    let facts = RunFacts {
        summary,
        bosses_defeated: history.bosses_defeated.clone(),
        items_held: history.items_held.clone(),
    };
    for handle in ended_handles(&locals, &run) {
        let id = local_id(handle);
        let profile = profiles
            .0
            .entry(id.clone())
            .or_insert_with(|| Profile::new(id));
        let obtained = grant_achievements(profile, &table, &facts);
        if obtained.is_empty() {
            continue;
        }
        for achievement in &obtained {
            info!("profil {} : accomplissement {achievement}", profile.id);
        }
        if let Some(dir) = &settings.dir {
            if let Err(e) = meta::save(dir, profile) {
                warn!("profil {} non écrit : {e}", profile.id);
            }
        }
        earned.0.insert(handle, obtained);
    }
}

/// Crédite et écrit les profils quand la run se termine (une seule fois par run).
pub fn write_profiles_at_run_end(
    run: Option<Res<Run>>,
    settings: Res<ProfileSettings>,
    mut profiles: ResMut<Profiles>,
    rules: Res<MetaRules>,
    locals: Query<&Player, With<LocalPlayer>>,
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
    // En ligne, seuls les joueurs locaux ont un profil ici (voir `ended_handles`).
    for handle in ended_handles(&locals, &run) {
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
            .init_resource::<Achievements>()
            .init_resource::<AchievementsEarned>()
            .init_resource::<RunHistory>()
            .rollback_and_trace_resource_no_checksum::<RunHistory>()
            .add_systems(
                Update,
                (write_profiles_at_run_end, write_achievements_at_run_end),
            )
            .add_systems(
                bevy_ggrs::GgrsSchedule,
                record_held_items.in_set(RollbackSystemSet::FrameCounter),
            );
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

    fn facts(outcome: run::RunEnd, floor: u32, bosses: &[&str], items: &[&str]) -> RunFacts {
        RunFacts {
            summary: run::RunSummary {
                wave_reached: 0,
                kills: 0,
                points_total: 0,
                frames: 900,
                outcome,
                floor_reached: floor,
            },
            bosses_defeated: bosses.iter().map(|b| b.to_string()).collect(),
            items_held: items.iter().map(|i| i.to_string()).collect(),
        }
    }

    fn table() -> Achievements {
        use meta::{AchievementCondition as When, AchievementDef};
        let def = |when: Vec<When>, unlock: &str| AchievementDef {
            name: String::new(),
            when,
            unlocks: vec![unlock.to_string()],
        };
        Achievements(
            [
                (
                    "boss",
                    def(vec![When::BossDefeated("gardien".into())], "u_boss"),
                ),
                ("fini", def(vec![When::Victory], "u_fini")),
                ("profond", def(vec![When::FloorAtLeast(1)], "u_profond")),
            ]
            .into_iter()
            .map(|(id, def)| (id.to_string(), def))
            .collect(),
        )
    }

    #[test]
    fn grants_only_what_the_facts_reach_and_only_once() {
        let table = table();
        let mut profile = Profile::new(local_id(0));
        // Boss vaincu puis défaite à l'étage 0 : « boss » seulement, avec son déblocage.
        let got = grant_achievements(
            &mut profile,
            &table,
            &facts(run::RunEnd::Defeat, 0, &["gardien"], &[]),
        );
        assert_eq!(got, ["boss"]);
        assert!(profile.has_unlock("u_boss") && !profile.has_unlock("u_fini"));
        // Même run rejouée : rien de nouveau. Victoire à l'étage 1 : les deux autres.
        let again = grant_achievements(
            &mut profile,
            &table,
            &facts(run::RunEnd::Victory, 1, &["gardien"], &[]),
        );
        assert_eq!(again, ["fini", "profond"]);
        assert_eq!(
            profile
                .achievements
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["boss", "fini", "profond"]
        );
        // Un boss non vaincu ne donne rien, et une table vide non plus.
        let mut other = Profile::new(local_id(1));
        assert!(grant_achievements(
            &mut other,
            &table,
            &facts(run::RunEnd::Defeat, 0, &["autre"], &[])
        )
        .is_empty());
        assert!(grant_achievements(
            &mut other,
            &Achievements::default(),
            &facts(run::RunEnd::Victory, 3, &["gardien"], &[])
        )
        .is_empty());
        assert!(other.unlocks.is_empty() && other.achievements.is_empty());
    }

    #[test]
    fn the_end_is_final_only_once_confirmed() {
        assert!(summary_is_final(900, None));
        assert!(summary_is_final(900, Some(900)));
        assert!(summary_is_final(900, Some(1200)));
        assert!(!summary_is_final(900, Some(899)));
        // Avant toute frame confirmée GGRS compte -1.
        assert!(!summary_is_final(0, Some(-1)));
    }

    #[test]
    fn earned_names_use_the_content_name_then_the_id() {
        let mut table = table();
        table.0.get_mut("fini").unwrap().name = "Run finie".into();
        let earned = AchievementsEarned(
            [
                (0, vec!["fini".to_string(), "boss".to_string()]),
                (1, vec!["boss".to_string()]),
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(earned.names(0, Some(&table)), ["Run finie", "boss"]);
        assert_eq!(earned.names(2, Some(&table)), Vec::<String>::new());
        assert_eq!(earned.names(1, None), ["boss"]);
        assert_eq!(earned.all_names(Some(&table)), ["boss", "Run finie"]);
    }

    fn history_app() -> App {
        let mut app = App::new();
        app.init_resource::<RunHistory>()
            .insert_resource(Run::new(1, run::RunMode::Sandbox, vec![0], 0))
            .add_systems(Update, record_held_items);
        app
    }

    #[test]
    fn held_items_follow_the_players_and_survive_their_death() {
        let mut app = history_app();
        let held = |app: &App| {
            app.world()
                .resource::<RunHistory>()
                .items_held
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        };
        // Un joueur sans inventaire (rien ramassé) n'écrit rien.
        let player = app
            .world_mut()
            .spawn(Player {
                handle: 0,
                ..Default::default()
            })
            .id();
        app.update();
        assert!(held(&app).is_empty());
        app.world_mut().entity_mut(player).insert(Inventory {
            passives: vec!["lame".into(), "bottes".into()],
            ..Default::default()
        });
        app.update();
        assert_eq!(held(&app), ["bottes", "lame"]);
        // Il lâche un objet : l'ensemble suit (objets *tenus*, pas ramassés).
        app.world_mut()
            .entity_mut(player)
            .get_mut::<Inventory>()
            .unwrap()
            .passives
            .retain(|item| item != "lame");
        app.update();
        assert_eq!(held(&app), ["bottes"]);
        // Mort du joueur : son inventaire disparaît, la dernière valeur reste.
        app.world_mut().entity_mut(player).despawn();
        app.update();
        assert_eq!(held(&app), ["bottes"]);
    }

    #[test]
    fn held_items_are_not_recorded_once_the_run_is_over() {
        let mut app = history_app();
        app.world_mut().spawn((
            Player {
                handle: 0,
                ..Default::default()
            },
            Inventory {
                passives: vec!["lame".into()],
                ..Default::default()
            },
        ));
        app.world_mut().resource_mut::<Run>().step = run::RunStep::Ended {
            at_frame: 10,
            outcome: run::RunEnd::Victory,
        };
        app.update();
        assert!(app.world().resource::<RunHistory>().items_held.is_empty());
    }

    #[test]
    fn local_keeps_only_known_unlocks() {
        let profiles = profiles();
        let ids = local_run_unlocks(Some(&profiles), &[0, 1], |id| id == "y").0;
        assert_eq!(ids.into_iter().collect::<Vec<_>>(), ["y"]);
    }

    #[test]
    fn host_profile_reads_unlocks_and_stats() {
        let profile = online_host_profile(Some("a, b"), Some("runs:3, kills:x, wins : 2,bad"));
        assert!(profile.has_unlock("a") && profile.has_unlock("b"));
        assert_eq!(profile.stat("runs"), 3);
        assert_eq!(profile.stat("wins"), 2);
        assert_eq!(profile.stat("kills"), 0);
    }

    #[test]
    fn online_ignores_local_profiles() {
        assert!(online_run_unlocks(None).0.is_empty());
        assert!(online_run_unlocks(Some("")).0.is_empty());
        let set = online_run_unlocks(Some("b, a,,b"));
        assert_eq!(set.0.into_iter().collect::<Vec<_>>(), ["a", "b"]);
    }
}
