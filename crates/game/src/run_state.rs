//! Branche `run::run::Run`/`run::modes::RunModeRules` (T2.4, chantier F1) sur la
//! simulation : résolution du mode depuis le manifeste, condition de victoire du mode
//! `Waves`, résumé de fin de partie, et relance sans relancer le binaire
//! (`RunRequest`/[`RunStatePlugin`]).
//!
//! Nommé `run_state` (pas `run`) pour ne pas entrer en collision avec le nom du crate
//! externe `run` dont ce module dépend (`mod run;` à la racine de `game` masquerait
//! `extern crate run`, cassant tous les `use run::...` existants du crate).
//!
//! # Relance (`RunRequest`)
//!
//! Le bouton de fin de partie (`ui::game_over`) ou la touche `R` posent une ressource
//! `RunRequest` (hors rollback : ni checksum, ni trace — une simple commande, comme
//! `NextState`). [`apply_run_request_system`] la consomme à la prochaine frame **hors**
//! `GgrsSchedule` (`Update`, après que `PreUpdate` a fait tourner `GgrsSchedule` pour cette
//! frame Bevy) et déclenche la transition d'état ; [`cleanup_rollback_world_system`], sur
//! `OnExit(AppState::InGame)` (déclenché par cette transition, quelle que soit sa cause :
//! relance ou retour au lobby), détruit tout ce qui doit l'être. `map_ldtk` nettoie l'arbre
//! LDtk séparément sur le même hook (`map_ldtk::game::local::despawn_ldtk_world_on_exit_ingame`) —
//! ce crate ne dépend pas de `bevy_ecs_ldtk`.
//!
//! **Ce qui est détruit/remis à zéro** (voir [`cleanup_rollback_world_system`]) : toutes
//! les entités `Rollback` (joueurs, ennemis, balles, armes, murs, portes, machines à
//! perk...), la session GGRS (`Session<PeerConfig>`, ses ressources internes se
//! réinitialisent d'elles-mêmes dès qu'aucune session n'est présente — voir
//! `bevy_ggrs::schedule_systems::run_ggrs_schedules`), `Run` elle-même, et les ressources
//! rollback globales qui ne sont pas réinitialisées ailleurs par le chargement normal d'une
//! partie (`FrameCount`, `GgrsNetIdFactory`, `WaveState`, `FlowFieldCache`,
//! `RepairPointsTracking`). **`bevy_ggrs::RollbackOrdered`** (interne à bevy_ggrs, pas
//! passée par `RollbackTraceApp`) aussi : elle compte *tous* les `Rollback` jamais créés
//! depuis le lancement du processus (pas juste la partie en cours, voir sa doc), et
//! contribue au checksum via `EntityChecksumPlugin` — sans ce reset, la relance produit un
//! checksum différent dès la frame 0 malgré un état de jeu par ailleurs identique (trouvé en
//! comparant les traces complètes d'un boot frais et d'une relance après une longue partie :
//! seule `RollbackOrdered.len()` différait, aucune valeur de composant/ressource tracée).
//! **Ce qui n'est pas détruit, volontairement** : `RunSeed`,
//! `RngStreams`, `MapGenerationConfig` (via `LdtkGameMap`, jamais retirée de l'app),
//! `GggrsSessionConfiguration`/`GgrsSessionBuilding` — c'est justement ce qui permet à
//! `Restart` de rejouer « la même configuration » (même carte, même graine, mêmes joueurs)
//! sans reconstruire un lobby : ces ressources sont simplement écrasées par les mêmes
//! valeurs quand `game::jjrs::{local, p2p}` recrée `Run`/`RunSeed`/`RngStreams`/`Session` au
//! prochain `OnEnter(AppState::GameStarting)`.
//!
//! **Restart en p2p** : non supporté par ce chantier (voir la doc de
//! [`RunRequest::Restart`]) — redirigé vers `ToLobby`.

use bevy::prelude::*;
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackOrdered, Session};
use content::manifest::{EntryMode, GameManifest};
use content::registry::Registry;
use run::{Currency, Run, RunContext, RunEnd, RunMode, RunModeRules, RunStep};
use utils::{frame::FrameCount, net_id::GgrsNetIdFactory};

use crate::{
    character::{
        enemy::ai::navigation::FlowFieldCache,
        player::{jjrs::PeerConfig, Player},
    },
    core::{AppState, OnlineState},
    economy::{award_points_system, RepairPointsTracking},
    global_asset::GlobalAsset,
    system_set::RollbackSystemSet,
    waves::{WaveConfig, WaveState},
};

/// Résout le [`RunMode`] d'une session à partir du manifeste (`entry.mode`, T2.4) et du
/// registre de contenu (`registry.waves`). Appelée par `jjrs::local`/`jjrs::p2p` au
/// démarrage de session, avec les mêmes ressources que `GlobalAsset::create` (déjà
/// présentes à ce moment).
///
/// Règle (voir `docs/conventions.md` section « Run ») : `entry.mode` explicite (`Waves` ou
/// `Sandbox`) gagne toujours ; absent, `Waves` si le jeu déclare un dossier de contenu
/// `Wave` (le lint refuse `entry.mode: Waves` sans dossier `Wave`, mais pas l'absence
/// d'`entry.mode` avec un dossier `Wave` présent : ce chemin-ci), sinon `Sandbox`.
pub fn resolve_run_mode(manifest: Option<&GameManifest>, registry: Option<&Registry>) -> RunMode {
    let wave_config_id = registry
        .and_then(|r| r.waves.keys().next())
        .map(|id| id.to_string());
    match manifest.and_then(|m| m.entry.mode) {
        Some(EntryMode::Sandbox) => RunMode::Sandbox,
        Some(EntryMode::Waves) => RunMode::Waves {
            config: wave_config_id.unwrap_or_default(),
        },
        None => match wave_config_id {
            Some(config) => RunMode::Waves { config },
            None => RunMode::Sandbox,
        },
    }
}

/// Commande de relance (T2.4), posée hors rollback (ni checksum ni trace — comme
/// `NextState`) par le bouton de fin de partie ou la touche `R`
/// (`ui::game_over::button_system`). Consommée par [`apply_run_request_system`].
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunRequest {
    /// Rejoue la même configuration (même carte, même graine, mêmes joueurs) sans
    /// relancer le binaire. **Local seulement** : en p2p (`OnlineState::Online`), ce
    /// chantier ne resynchronise pas une nouvelle session entre pairs à partir d'un
    /// signal local — `apply_run_request_system` traite alors `Restart` comme
    /// `ToLobby` (le bouton « renvoie au lobby », comme demandé par la tâche).
    Restart,
    /// Retour au lobby (`LobbyLocal`/`LobbyOnline` selon `OnlineState`).
    ToLobby,
}

/// Branche [`Run`]/[`RunModeRules`] sur la simulation (victoire, résumé) et la relance
/// (`RunRequest`). Voir la doc du module.
pub struct RunStatePlugin;

impl Plugin for RunStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GgrsSchedule,
            (
                // Ordonnés explicitement : les trois touchent `Currency`/`Run` dans le
                // même `RollbackSystemSet::Run` (détection d'ambiguïté stricte sur
                // `GgrsSchedule`, voir `core::CoreSetupPlugin`) — `award_points_system`
                // (lecture/écriture `Currency`) avant `check_run_victory_system` (lit
                // `WaveState`, écrit `Run.step`) avant `finalize_run_summary_system`
                // (lit `Currency`/`WaveState`, écrit `Run.summary` une fois `step`
                // `Ended`, qu'il vienne de la victoire ci-dessus ou de la défaite T1.3
                // posée plus tôt dans la frame, `RollbackSystemSet::DeathManagement`).
                check_run_victory_system.after(award_points_system),
                finalize_run_summary_system.after(check_run_victory_system),
            )
                .in_set(RollbackSystemSet::Run),
        );

        app.add_systems(
            Update,
            apply_run_request_system.run_if(in_state(AppState::InGame)),
        );

        app.add_systems(OnExit(AppState::InGame), cleanup_rollback_world_system);
    }
}

/// Victoire (T2.4) : condition du mode actif (`RunModeRules::check_victory`), qui ne
/// dépend que de [`RunContext`] — aujourd'hui seul `RunMode::Waves` avec
/// `WaveConfig::max_wave` peut la déclencher (voir sa doc). `points_total` n'entre pas
/// dans la condition de victoire (seulement dans le résumé), laissé à `0` ici plutôt que
/// recalculé pour rien.
pub fn check_run_victory_system(
    frame: Res<FrameCount>,
    mut run: ResMut<Run>,
    wave_config_assets: Res<Assets<WaveConfig>>,
    global_assets: Res<GlobalAsset>,
    wave_state: Res<WaveState>,
) {
    if !run.is_playing() {
        return;
    }

    let max_wave = global_assets
        .wave_config
        .as_ref()
        .and_then(|handle| wave_config_assets.get(handle))
        .and_then(|config| config.max_wave);

    let ctx = RunContext {
        frame: frame.frame,
        current_wave: wave_state.current_wave,
        max_wave,
        kills: wave_state.total_enemies_killed,
        points_total: 0,
    };

    if let Some(outcome) = run.mode.check_victory(&ctx) {
        info!("f{} run ended: victory", frame.frame);
        run.step = RunStep::Ended {
            at_frame: frame.frame,
            outcome,
        };
    }
}

/// Résumé (T2.4) : calculé une seule fois, dès que `Run.step` devient `Ended` (défaite
/// T1.3, victoire ci-dessus, ou abandon — voir `apply_run_request_system`) et que
/// `Run.summary` est encore `None`. `points_total` = somme des soldes (`Currency`) de tous
/// les joueurs encore en jeu à cet instant (voir `docs/conventions.md` section « Run » pour
/// la décision : pas un total cumulé des gains, qui n'existe nulle part ailleurs dans le
/// jeu).
pub fn finalize_run_summary_system(
    frame: Res<FrameCount>,
    wave_state: Res<WaveState>,
    currencies: Query<&Currency, With<Player>>,
    mut run: ResMut<Run>,
) {
    let RunStep::Ended { outcome, .. } = run.step else {
        return;
    };
    if run.summary.is_some() {
        return;
    }

    let points_total = currencies
        .iter()
        .fold(0u32, |total, currency| total.saturating_add(currency.0));

    let ctx = RunContext {
        frame: frame.frame,
        current_wave: wave_state.current_wave,
        max_wave: None,
        kills: wave_state.total_enemies_killed,
        points_total,
    };
    let summary = run.mode.summarize(&ctx, outcome);
    info!(
        "f{} run summary: wave={} kills={} points={} outcome={:?}",
        frame.frame, summary.wave_reached, summary.kills, summary.points_total, summary.outcome
    );
    run.summary = Some(summary);
}

/// Consomme [`RunRequest`] (posée par `ui::game_over`) à la prochaine frame hors
/// `GgrsSchedule` : fige `Run.step` en `Ended { outcome: Abandon }` si la partie était
/// encore en cours et qu'on quitte vers le lobby (pas de résumé calculé pour ce cas — voir
/// le rapport de la tâche, limitation connue), puis change d'état. La destruction
/// proprement dite (entités, session, ressources) est déclenchée par la transition d'état
/// elle-même (`OnExit(AppState::InGame)`, voir [`cleanup_rollback_world_system`] et la doc
/// du module), pas ici.
fn apply_run_request_system(
    mut commands: Commands,
    request: Option<Res<RunRequest>>,
    mut app_state: ResMut<NextState<AppState>>,
    online_state: Res<OnlineState>,
    frame: Res<FrameCount>,
    mut run: ResMut<Run>,
) {
    let Some(request) = request else {
        return;
    };

    let online = matches!(*online_state, OnlineState::Online);
    let to_lobby = matches!(*request, RunRequest::ToLobby)
        || (matches!(*request, RunRequest::Restart) && online);

    if to_lobby {
        if online && matches!(*request, RunRequest::Restart) {
            warn!("RunRequest::Restart demandé en p2p : non supporté, retour au lobby à la place (voir la doc de RunRequest::Restart)");
        }
        if run.is_playing() {
            run.step = RunStep::Ended {
                at_frame: frame.frame,
                outcome: RunEnd::Abandon,
            };
        }
        app_state.set(if online {
            AppState::LobbyOnline
        } else {
            AppState::LobbyLocal
        });
    } else {
        // Restart local : `MapGenerationConfig`/`GggrsSessionConfiguration`/
        // `GgrsSessionBuilding` restent en place, inchangées (voir la doc du module) —
        // retour direct en `GameLoading`, sans repasser par un lobby.
        app_state.set(AppState::GameLoading);
    }

    commands.remove_resource::<RunRequest>();
}

/// Déclenché par `OnExit(AppState::InGame)`, quelle que soit la cause (voir la doc du
/// module pour la liste précise de ce qui est détruit/remis à zéro et pourquoi).
/// `try_despawn` (pas `despawn`) : le despawn d'une entité `Rollback` racine (ex. un
/// joueur) est déjà récursif (voir `bevy_ecs::system::commands::EntityCommands::despawn`,
/// « This will also despawn the entities in any RelationshipTarget... ») et détruit donc
/// aussi ses enfants non-`Rollback` (visuels) *et* ses enfants `Rollback` (ex. les armes,
/// `ChildOf` du joueur qui les porte — voir `weapons::weapon_rollback_system`) avant que la
/// query ci-dessous n'atteigne leur propre tour ; `try_despawn` ignore silencieusement une
/// entité déjà partie plutôt que de logger un avertissement pour chacune (pas un bug,
/// l'ordre d'itération de la query n'a aucune raison de suivre la hiérarchie).
fn cleanup_rollback_world_system(
    mut commands: Commands,
    rollback_entities: Query<Entity, With<Rollback>>,
) {
    let mut despawned = 0u32;
    for entity in &rollback_entities {
        commands.entity(entity).try_despawn();
        despawned += 1;
    }

    commands.remove_resource::<Session<PeerConfig>>();
    commands.remove_resource::<Run>();
    commands.insert_resource(FrameCount::default());
    commands.insert_resource(GgrsNetIdFactory::default());
    commands.insert_resource(WaveState::default());
    commands.insert_resource(FlowFieldCache::default());
    commands.insert_resource(RepairPointsTracking::default());
    // Compteur cumulatif interne à bevy_ggrs (voir la doc du module) : sans ce reset, le
    // checksum de la frame 0 d'une partie relancée diffère de celui d'un boot frais, même à
    // état de jeu par ailleurs strictement identique.
    commands.insert_resource(RollbackOrdered::default());

    info!("sortie d'InGame : {despawned} entité(s) rollback détruite(s), session et état de run remis à zéro");
}

#[cfg(test)]
mod tests {
    use super::*;
    use content::manifest::EntryPoint;

    fn manifest_with_mode(mode: Option<EntryMode>) -> GameManifest {
        GameManifest {
            name: "demo".to_string(),
            content_folders: vec![],
            entry: EntryPoint {
                start_map: "exemples/test_map.ldtk".to_string(),
                default_seed: 1,
                mode,
            },
        }
    }

    #[test]
    fn no_manifest_no_registry_is_sandbox() {
        assert_eq!(resolve_run_mode(None, None), RunMode::Sandbox);
    }

    #[test]
    fn explicit_sandbox_wins_even_with_waves_registry() {
        let manifest = manifest_with_mode(Some(EntryMode::Sandbox));
        // Un registre qui déclarerait des vagues n'est pas construit ici (coûteux,
        // nécessite des fichiers sur disque) : `None` suffit à prouver que le mode
        // explicite l'emporte sans même consulter le registre pour `Sandbox`/`Waves`
        // explicites (voir `resolve_run_mode`, les deux premiers bras du `match`).
        assert_eq!(resolve_run_mode(Some(&manifest), None), RunMode::Sandbox);
    }

    #[test]
    fn explicit_waves_without_registry_falls_back_to_empty_config_id() {
        let manifest = manifest_with_mode(Some(EntryMode::Waves));
        assert_eq!(
            resolve_run_mode(Some(&manifest), None),
            RunMode::Waves {
                config: String::new()
            }
        );
    }

    #[test]
    fn no_entry_mode_without_registry_is_sandbox() {
        let manifest = manifest_with_mode(None);
        assert_eq!(resolve_run_mode(Some(&manifest), None), RunMode::Sandbox);
    }
}
