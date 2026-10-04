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
use run::{
    Currency, FloorState, Run, RunContext, RunEnd, RunMode, RunModeRules, RunStep, RunSummary,
};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};

use crate::{
    character::{
        enemy::{ai::navigation::FlowFieldCache, Enemy},
        player::{jjrs::PeerConfig, Player},
    },
    core::{AppState, OnlineState},
    economy::{award_points_system, RepairPointsTracking},
    system_set::RollbackSystemSet,
    waves::WaveState,
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
    // T1.8 : une séquence imposée (`FloorsOverride`, scénario ou `alacod-sim --floors`)
    // passe par `resolve_run_mode_with_floors`, pas par ici.
    resolve_run_mode_with_floors(manifest, registry, None)
}

/// Comme [`resolve_run_mode`], avec une séquence de niveaux imposée (T1.8 : champ `floors`
/// d'un scénario, option `--floors` d'`alacod-sim`, voir [`FloorsOverride`]) : `Some(id)`
/// force `RunMode::Floors { config: id }` quel que soit `entry.mode`. Sinon, `entry.mode:
/// Floors` prend la première séquence (ordre des ids) du dossier `Floors`.
pub fn resolve_run_mode_with_floors(
    manifest: Option<&GameManifest>,
    registry: Option<&Registry>,
    floors_override: Option<&str>,
) -> RunMode {
    if let Some(config) = floors_override {
        return RunMode::Floors {
            config: config.to_string(),
        };
    }
    let wave_config_id = registry
        .and_then(|r| r.waves.keys().next())
        .map(|id| id.to_string());
    match manifest.and_then(|m| m.entry.mode) {
        Some(EntryMode::Floors) => RunMode::Floors {
            config: registry
                .and_then(|r| r.floors.keys().next())
                .map(|id| id.to_string())
                .unwrap_or_default(),
        },
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

/// Séquence de niveaux imposée à la partie (T1.8), hors rollback : id d'une entrée du
/// dossier `Floors` (`content::registry::FloorsConfigId`). Posée par le runner de scénarios
/// (champ `floors`) ; lue par `jjrs::{local, p2p}` (mode de la partie) et par
/// `map_ldtk::game::local` (cartes à charger). Absente : le mode vient du manifeste.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct FloorsOverride(pub String);

/// Cartes des niveaux du mode `Floors` (T1.8) : la séquence `config` du registre. `None`
/// si le mode n'est pas `Floors` ou que la séquence est inconnue (le lint l'aurait refusée).
pub fn floor_levels(mode: &RunMode, registry: Option<&Registry>) -> Option<Vec<String>> {
    let RunMode::Floors { config } = mode else {
        return None;
    };
    registry?
        .floors
        .get(&content::registry::FloorsConfigId::from(config.clone()))
        .map(|entry| entry.levels.clone())
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

/// D13 : le lobby local attend une action du joueur avant de relancer une partie. Posée
/// par [`apply_run_request_system`] quand une partie locale retourne au lobby ; tant
/// qu'elle existe, `jjrs::local::setup_ggrs_local` ne démarre pas de session (condition
/// dans `core::CoreSetupPlugin`). `ui::lobby` l'affiche (résumé de la partie quittée) et la
/// retire sur Entrée ou un clic sur « Nouvelle partie ». Absente au premier lancement : le
/// lobby local démarre aussitôt, comme avant D13 (scénarios, `make zombies`). Hors
/// rollback, comme [`RunRequest`].
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct LocalLobbyHold {
    /// Résumé de la partie quittée (calculé à l'abandon, D13), `None` si elle n'en avait pas.
    pub summary: Option<RunSummary>,
}

/// Ce qu'une [`RunRequest`] fait (D13) : décision pure de [`apply_run_request_system`].
#[derive(Debug, Clone, PartialEq, Eq)]
struct RunRequestPlan {
    /// État suivant.
    next: AppState,
    /// La partie était en cours : elle se termine en `RunEnd::Abandon`, avec son résumé.
    abandon: bool,
    /// Retour au lobby **local** : il attend une action du joueur ([`LocalLobbyHold`]).
    hold_local_lobby: bool,
    /// `Restart` demandé en p2p, traité comme `ToLobby` (voir [`RunRequest::Restart`]).
    restart_redirected: bool,
}

fn plan_run_request(request: RunRequest, online: bool, playing: bool) -> RunRequestPlan {
    let restart_redirected = request == RunRequest::Restart && online;
    if request == RunRequest::ToLobby || restart_redirected {
        RunRequestPlan {
            next: if online {
                AppState::LobbyOnline
            } else {
                AppState::LobbyLocal
            },
            abandon: playing,
            hold_local_lobby: !online,
            restart_redirected,
        }
    } else {
        // Restart local : `MapGenerationConfig`/`GggrsSessionConfiguration`/
        // `GgrsSessionBuilding` restent en place, inchangées (voir la doc du module) —
        // retour direct en `GameLoading`, sans repasser par un lobby.
        RunRequestPlan {
            next: AppState::GameLoading,
            abandon: false,
            hold_local_lobby: false,
            restart_redirected: false,
        }
    }
}

/// Résumé d'une partie terminée (T2.4) ; D13 : partagé par la fin normale
/// ([`finalize_run_summary_system`]) et l'abandon ([`apply_run_request_system`]), mêmes
/// champs. `points_total` = somme des soldes des joueurs encore en jeu, dans l'ordre
/// `GgrsNetId` (voir `docs/conventions.md` section « Run »).
///
/// T1.8, mode `Floors` : `kills` = ennemis placés par les niveaux chargés
/// (`FloorState::enemies_placed`) moins ceux encore en vie (`enemies_alive`), les ennemis du
/// mode ne venant pas des vagues ; `floor_reached` = `FloorState::index`.
fn run_summary(
    mode: &RunMode,
    frame: u32,
    wave_state: &WaveState,
    floor_state: &FloorState,
    enemies_alive: u32,
    points_total: u32,
    outcome: RunEnd,
) -> RunSummary {
    let floors = matches!(mode, RunMode::Floors { .. });
    let ctx = RunContext {
        frame,
        current_wave: wave_state.current_wave,
        max_wave: None,
        kills: if floors {
            floor_state.enemies_placed.saturating_sub(enemies_alive)
        } else {
            wave_state.total_enemies_killed
        },
        points_total,
        floor: floor_state.index,
    };
    mode.summarize(&ctx, outcome)
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
    balance: Res<crate::balance::ResolvedBalance>,
    wave_state: Res<WaveState>,
) {
    if !run.is_playing() {
        return;
    }

    // F5 (chantier m0-v11) : config résolue une fois au lancement (voir `crate::balance`).
    let max_wave = balance.waves.max_wave;

    let ctx = RunContext {
        frame: frame.frame,
        current_wave: wave_state.current_wave,
        max_wave,
        kills: wave_state.total_enemies_killed,
        points_total: 0,
        floor: 0,
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
    floor_state: Res<FloorState>,
    enemies: Query<(), With<Enemy>>,
    currencies: Query<(&GgrsNetId, &Currency), With<Player>>,
    mut run: ResMut<Run>,
) {
    let RunStep::Ended { outcome, .. } = run.step else {
        return;
    };
    if run.summary.is_some() {
        return;
    }

    let mut points_total = 0u32;
    for (_, currency) in order_iter!(currencies) {
        points_total = points_total.saturating_add(currency.0);
    }

    let summary = run_summary(
        &run.mode,
        frame.frame,
        &wave_state,
        &floor_state,
        enemies.iter().count() as u32,
        points_total,
        outcome,
    );
    info!(
        "f{} run summary: wave={} kills={} points={} outcome={:?}",
        frame.frame, summary.wave_reached, summary.kills, summary.points_total, summary.outcome
    );
    run.summary = Some(summary);
}

/// Consomme [`RunRequest`] (posée par `ui::game_over`) à la prochaine frame hors
/// `GgrsSchedule` (décision : [`plan_run_request`]) : si la partie était encore en cours et
/// qu'on quitte vers le lobby, fige `Run.step` en `Ended { outcome: Abandon }` **et calcule
/// son résumé** (D13 : mêmes champs qu'une défaite, issue `Abandon` ; avant D13, aucun
/// résumé — `finalize_run_summary_system` ne tourne pas avant la destruction de `Run`). Un
/// retour au lobby local pose [`LocalLobbyHold`] : le lobby attend le joueur au lieu de
/// relancer aussitôt une partie (D13). La destruction proprement dite (entités, session,
/// ressources) est déclenchée par la transition d'état elle-même
/// (`OnExit(AppState::InGame)`, voir [`cleanup_rollback_world_system`] et la doc du
/// module), pas ici.
fn apply_run_request_system(
    mut commands: Commands,
    request: Option<Res<RunRequest>>,
    mut app_state: ResMut<NextState<AppState>>,
    online_state: Res<OnlineState>,
    frame: Res<FrameCount>,
    wave_state: Res<WaveState>,
    floor_state: Res<FloorState>,
    enemies: Query<(), With<Enemy>>,
    currencies: Query<(&GgrsNetId, &Currency), With<Player>>,
    mut run: ResMut<Run>,
) {
    let Some(request) = request else {
        return;
    };

    let online = matches!(*online_state, OnlineState::Online);
    let plan = plan_run_request(*request, online, run.is_playing());
    if plan.restart_redirected {
        warn!("RunRequest::Restart demandé en p2p : non supporté, retour au lobby à la place (voir la doc de RunRequest::Restart)");
    }
    if plan.abandon {
        run.step = RunStep::Ended {
            at_frame: frame.frame,
            outcome: RunEnd::Abandon,
        };
        let mut points_total = 0u32;
        for (_, currency) in order_iter!(currencies) {
            points_total = points_total.saturating_add(currency.0);
        }
        let summary = run_summary(
            &run.mode,
            frame.frame,
            &wave_state,
            &floor_state,
            enemies.iter().count() as u32,
            points_total,
            RunEnd::Abandon,
        );
        info!(
            "f{} run summary: wave={} kills={} points={} outcome={:?}",
            frame.frame, summary.wave_reached, summary.kills, summary.points_total, summary.outcome
        );
        run.summary = Some(summary);
    }
    if plan.hold_local_lobby {
        commands.insert_resource(LocalLobbyHold {
            summary: run.summary,
        });
    }
    app_state.set(plan.next);

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
    // T1.8 : l'index de niveau repart de zéro (la nouvelle partie repose son portail au
    // chargement de la carte, `map_ldtk::game::floors`).
    commands.insert_resource(FloorState::default());
    // T1.9 : horloges et difficulté de la partie suivante repartent de zéro.
    commands.insert_resource(run::Clock::default());
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

    #[test]
    fn to_lobby_en_local_abandonne_et_fait_attendre_le_lobby() {
        // D13 : partie en cours quittée vers le lobby local.
        let plan = plan_run_request(RunRequest::ToLobby, false, true);
        assert_eq!(plan.next, AppState::LobbyLocal);
        assert!(plan.abandon);
        assert!(plan.hold_local_lobby);
        // Depuis l'écran de fin (partie déjà terminée) : pas d'abandon, mais le lobby
        // local attend quand même le joueur.
        let plan = plan_run_request(RunRequest::ToLobby, false, false);
        assert!(!plan.abandon && plan.hold_local_lobby);
    }

    #[test]
    fn to_lobby_et_restart_en_ligne_vont_au_lobby_en_ligne_sans_attente_locale() {
        for request in [RunRequest::ToLobby, RunRequest::Restart] {
            let plan = plan_run_request(request, true, true);
            assert_eq!(plan.next, AppState::LobbyOnline);
            assert!(plan.abandon);
            assert!(!plan.hold_local_lobby);
            assert_eq!(plan.restart_redirected, request == RunRequest::Restart);
        }
    }

    #[test]
    fn restart_local_relance_sans_lobby_ni_abandon() {
        let plan = plan_run_request(RunRequest::Restart, false, true);
        assert_eq!(
            plan,
            RunRequestPlan {
                next: AppState::GameLoading,
                abandon: false,
                hold_local_lobby: false,
                restart_redirected: false,
            }
        );
    }

    #[test]
    fn resume_a_l_abandon_memes_champs_qu_une_defaite() {
        let wave_state = WaveState {
            current_wave: 3,
            total_enemies_killed: 17,
            ..Default::default()
        };
        let mode = RunMode::Waves {
            config: "wave_config".to_string(),
        };
        let floors = FloorState::default();
        let abandon = run_summary(&mode, 1234, &wave_state, &floors, 0, 2500, RunEnd::Abandon);
        let defeat = run_summary(&mode, 1234, &wave_state, &floors, 0, 2500, RunEnd::Defeat);
        assert_eq!(
            abandon,
            RunSummary {
                wave_reached: 3,
                kills: 17,
                points_total: 2500,
                frames: 1234,
                outcome: RunEnd::Abandon,
                floor_reached: 0,
            }
        );
        assert_eq!(
            RunSummary {
                outcome: RunEnd::Defeat,
                ..abandon
            },
            defeat
        );
    }

    fn manifest_with_mode(mode: Option<EntryMode>) -> GameManifest {
        GameManifest {
            name: "demo".to_string(),
            content_folders: vec![],
            entry: EntryPoint {
                start_map: "exemples/test_map.ldtk".to_string(),
                default_seed: 1,
                mode,
                progression: None,
                clocks: None,
                difficulty: None,
            },
            generate_template: None,
        }
    }

    #[test]
    fn resume_floors_niveau_et_ennemis_elimines() {
        let mode = RunMode::Floors {
            config: "deux_salles".to_string(),
        };
        let floors = FloorState {
            index: 2,
            enemies_placed: 7,
            ..Default::default()
        };
        // Les vagues ne comptent pas en `Floors` : 7 placés, 2 encore en vie → 5 kills.
        let wave_state = WaveState {
            total_enemies_killed: 40,
            ..Default::default()
        };
        let summary = run_summary(&mode, 900, &wave_state, &floors, 2, 10, RunEnd::Defeat);
        assert_eq!(summary.kills, 5);
        assert_eq!(summary.floor_reached, 2);
        assert_eq!(summary.wave_reached, 0);
    }

    #[test]
    fn floors_impose_ou_declare_par_le_manifeste() {
        assert_eq!(
            resolve_run_mode_with_floors(None, None, Some("deux_salles")),
            RunMode::Floors {
                config: "deux_salles".to_string()
            }
        );
        let manifest = manifest_with_mode(Some(EntryMode::Floors));
        assert_eq!(
            resolve_run_mode(Some(&manifest), None),
            RunMode::Floors {
                config: String::new()
            }
        );
        // L'override gagne même sur un manifeste `Sandbox`.
        let manifest = manifest_with_mode(Some(EntryMode::Sandbox));
        assert!(matches!(
            resolve_run_mode_with_floors(Some(&manifest), None, Some("x")),
            RunMode::Floors { .. }
        ));
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
