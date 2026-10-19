//! Moitié **simulation** de `CoreSetupPlugin` (D69 étape 2, E7) : tous les plugins de simulation, les
//! ressources et états de session, les enregistrements rollback, l'ordre de `GgrsSchedule` et les
//! systèmes de session qui ne touchent ni socket, ni asset, ni rendu. Chemin final (E9) :
//! `game_sim/src/core.rs`.
//!
//! **Ordre.** `build` reprend, dans l'ordre exact de l'ancien `CoreSetupPlugin::build`, les éléments de
//! simulation (`docs/taches/rapports/dette-d69-plan-etape2/annexe-session.md` §1.1). Les cinq
//! enregistrements de clôture (`GgrsNetIdFactory`, `FrameCount`, `FixedTransform3D`, `GgrsNetId`,
//! `RngStreams`) restent **les derniers** enregistrements rollback : après eux, ce plugin n'en ajoute
//! plus, et `CoreAppPlugin` (`core.rs`), ajouté ensuite, n'en enregistre aucun. `BaseCharacterGamePlugin`
//! et `ProfilePlugin` lisent des ressources de l'appelant (`DebugAiConfig`, `ProfileSettings`) au moment
//! du `build` : elles restent dans ce bloc, après les `insert_resource` de l'appelant.
//!
//! Aucune dépendance de présentation (`camera`, `audio`, `light`, `ui` hors modèles purs, `Window`,
//! `AssetServer`, `MatchboxSocket`) : la moitié application est `core::CoreAppPlugin`.

use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_fixed::{
    fixed_math::{self, sync_bevy_transforms_from_fixed},
    rng::RngStreams,
};
use bevy_ggrs::{GgrsPlugin, GgrsSchedule};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
};

use crate::{
    character::{player::jjrs::PeerConfig, BaseCharacterGamePlugin},
    collider::BaseColliderGamePlugin,
    frame_counter::increase_frame_system,
    jjrs::{
        local::{setup_ggrs_local, system_after_map_loaded_local},
        log_ggrs_events,
        p2p::start_online_run,
        DesyncObserved, GggrsSessionConfigurationState, SessionLostEvent,
    },
    session::{single_threaded_rollback_schedules, AppState, CoreSetupConfig, SIM_FPS},
    system_set::RollbackSystemSet,
    waves::WaveSystemPlugin,
    weapons::BaseWeaponGamePlugin,
};

/// Simulation de `CoreSetupPlugin` : voir la documentation du module. Réutilisable sans `CoreAppPlugin`
/// (`bots`, simulation seule) : il ne lit aucune ressource de présentation.
#[derive(Default)]
pub struct CoreSimPlugin(pub CoreSetupConfig);

impl Plugin for CoreSimPlugin {
    fn build(&self, app: &mut App) {
        if self.0.headless {
            // Chaque update avance d'exactement une frame GGRS, aussi vite que le CPU le permet
            app.insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_nanos(1_000_000_000 / SIM_FPS),
            ));
        }

        // L'état logique d'animation (rollback) est enregistré par `sim_core`, avant la présentation.
        app.add_plugins(sim_core::animation_state::AnimationStatePlugin);
        app.add_plugins(GgrsPlugin::<PeerConfig>::default());
        // `GgrsPlugin` installe aussi `RollbackDespawnPlugin` : les entités de la simulation se
        // détruisent avec `despawn_rollback()` (voir CLAUDE.md, règle 9) et `GgrsSchedule` refuse
        // toute ambiguïté d'ordre entre systèmes (`ambiguity_detection: Error`).

        app.add_plugins(BaseWeaponGamePlugin {});
        // Contrats M1 : enregistrés ensemble, aucun état posé ni système exécuté.
        app.add_plugins((
            combat::CombatPlugin,
            behaviors::BehaviorsPlugin,
            effects::EffectsPlugin,
        ));
        app.add_plugins(BaseColliderGamePlugin {});
        app.add_plugins(BaseCharacterGamePlugin {});
        // Monnaie et perks (T2.3, chantier C5 v1) : `run::RunPlugin` enregistre
        // `Currency`/`Perks`/`FrameEvents<CurrencyEvent>` en rollback ; `economy::EconomyPlugin`
        // ajoute le système de points (`RollbackSystemSet::Run`). Avant `InteractionPlugin` : les
        // achats (portes, armes murales, perks) vivent dans `interaction.rs` et lisent
        // `Currency`/`Perks`.
        app.add_plugins(run::RunPlugin);
        // Terrain des cavernes (T1.6) : `CellGrid` et demandes de destruction, toutes neutres
        // au checksum (grille vide et files vides hors caverne : traces inchangées).
        app.add_plugins(world::WorldPlugin);
        // Effets v1 (T1.10) : composants neutres, exécution dans `DeathManagement`.
        app.add_plugins(crate::effects_runtime::EffectsRuntimePlugin);
        app.add_plugins(crate::effects_v3::EffectsV3Plugin);
        app.add_plugins(crate::progression::ProgressionPlugin);
        app.add_plugins(crate::economy::EconomyPlugin);
        app.add_plugins(crate::interaction::InteractionPlugin);
        // Power-ups (T2.5, chantier C1 v0) : après `EconomyPlugin` (l'action
        // `CurrencyMultiplier` lit la stat qu'elle pose via `economy::award_points_system`,
        // ordre de *systèmes* explicite entre `RollbackSystemSet::Effects` et `::Run`, pas
        // besoin d'ordre entre plugins ici — seulement une dépendance de lecture, listée
        // après par lisibilité).
        app.add_plugins(crate::powerups::PowerUpsPlugin);
        app.add_plugins(crate::items::ItemsPlugin);
        // M3-T6 : familiers et orbitaux (§71), après les tirs des joueurs.
        app.add_plugins(crate::familiars::FamiliarsPlugin);
        // M3-T7 : `on_death`, butin des champions (§72).
        app.add_plugins(crate::champions::ChampionsPlugin);
        app.add_plugins(crate::loot::LootPlugin);
        // M3-T9 : tirage des curses à l'entrée de chaque étage et leurs effets (§74).
        app.add_plugins(crate::curses::CursesPlugin);
        app.add_plugins(crate::shop::ShopPlugin);
        app.add_plugins(crate::hub::HubPlugin);
        // M2-T11 : fosses, tables renversables, barils explosifs (inertes sans eux)
        app.add_plugins(crate::props::PropsPlugin);
        app.add_plugins(crate::bombs::BombsPlugin);
        app.add_plugins(crate::boss::BossPlugin);
        app.add_plugins(crate::profile::ProfilePlugin);
        // Feedback v1 (T1.17) : journal de présentation (`FeedbackLog`), rempli aussi en
        // headless pour prouver le feedback sans écran ; le rendu est dans `PresentationPlugin`.
        app.add_plugins(crate::feedback_log::FeedbackLogPlugin);
        // Modèles de vue de l'interface, tenus à jour aussi en headless (lus par les attentes de
        // scénario `HudText` et `ScreenText`) : sans entrée ni rendu ; leurs entrées sont dans
        // `CoreAppPlugin` (`GameUiInputPlugin`).
        // T1.16 : modèle de vue de l'écran de mutation, aussi en headless.
        app.add_plugins(crate::ui::mutation_screen::MutationScreenModelPlugin);
        // T1.18 : valeurs du HUD du joueur affiché (`HudSnapshot`), aussi en headless.
        app.add_plugins(crate::ui::hud_model::HudModelPlugin);
        // M2-T18 : état et vue des écrans pause et inventaire, aussi en headless.
        app.add_plugins(crate::ui::screens::ScreensPlugin);
        app.add_plugins(WaveSystemPlugin);
        // État de run (T2.4, chantier F1) : condition de victoire, résumé, relance sans
        // relancer le binaire (`RunRequest`). Voir `crate::run_state`.
        app.add_plugins(crate::run_state::RunStatePlugin);

        app.init_resource::<GggrsSessionConfigurationState>();
        app.init_resource::<GgrsNetIdFactory>();
        app.init_resource::<FrameCount>();

        // Flux RNG nommés (T1.6) : remplacés au démarrage de session par ceux dérivés de
        // `RunSeed` (jjrs/local.rs, jjrs/p2p.rs) ; enregistrés en rollback plus bas.
        app.insert_resource(RngStreams::new(12345));

        app.add_message::<SessionLostEvent>();
        app.add_message::<DesyncObserved>();

        app.init_state::<AppState>();
        // app.set_rollback_schedule_fps(60);

        use crate::rollback::RollbackTraceApp;

        app.rollback_and_trace_copy_resource::<GgrsNetIdFactory>()
            .rollback_and_trace_copy_resource::<FrameCount>()
            .rollback_and_trace::<fixed_math::FixedTransform3D>()
            .rollback_and_trace::<GgrsNetId>()
            .rollback_and_trace_resource::<RngStreams>();

        // Ordre total : `RollbackSystemSet::ORDER` (sim_core, T0.2), chaîné pair à pair
        // (mêmes arêtes qu'un `.chain()` sur un n-uplet, sans limite d'arité de tuple).
        for pair in RollbackSystemSet::ORDER.windows(2) {
            app.configure_sets(GgrsSchedule, pair[1].after(pair[0]));
        }
        // `ALACOD_SYSTEM_PROFILE=1` : temps par set (`utils::system_profile`), une frontière
        // entre deux sets voisins, une avant le premier, une après le dernier ; rien sinon.
        // Le champ de flux (`update_flow_field_system`, entre `EnemySpawning` et `EnemyAI`) a sa
        // propre tranche : mesuré le 2026-10-15 (p2p_smoke, 4 joueurs qui bougent), il pèse
        // 9,1 ms des 9,3 ms d'une frame de simulation (reconstruction à chaque changement de
        // case d'un joueur, `FlowFieldCache::update_interval` vaut 0 par `Default`).
        if utils::system_profile::requested() {
            use crate::character::enemy::ai::navigation::update_flow_field_system;
            use utils::system_profile::boundary;
            app.init_resource::<utils::system_profile::SetTimings>();
            let order = RollbackSystemSet::ORDER;
            app.add_systems(
                GgrsSchedule,
                boundary(Some(order[0].name())).before(order[0]),
            );
            for pair in order.windows(2) {
                let (before, after) = (pair[0], pair[1]);
                if after == RollbackSystemSet::EnemyAI {
                    app.add_systems(
                        GgrsSchedule,
                        (
                            boundary(Some("FlowField"))
                                .after(before)
                                .before(update_flow_field_system),
                            boundary(Some(after.name()))
                                .after(update_flow_field_system)
                                .before(after),
                        ),
                    );
                } else {
                    app.add_systems(
                        GgrsSchedule,
                        boundary(Some(after.name())).after(before).before(after),
                    );
                }
            }
            app.add_systems(GgrsSchedule, boundary(None).after(order[order.len() - 1]));
        }

        // Sync FixedTransform3D to Transform after GGRS schedule runs
        // This ensures visual representation matches the rollback simulation state
        app.add_systems(
            PostUpdate,
            sync_bevy_transforms_from_fixed.run_if(in_state(AppState::InGame)),
        );

        app.add_systems(
            Update,
            // D13 : après un retour au lobby local, attend que le joueur relance
            // (`ui::lobby` retire `LocalLobbyHold`) au lieu de relancer aussitôt.
            setup_ggrs_local
                .run_if(in_state(AppState::LobbyLocal))
                .run_if(not(resource_exists::<crate::run_state::LocalLobbyHold>)),
        );
        // D14 (§33) : restart en ligne — compteur de parties en ligne (remis à zéro à chaque
        // entrée normale dans `LobbyOnline`).
        // D86 : pairs des sessions déjà jouées (ignorés au lobby suivant) et abandon d'une
        // session qui ne se synchronise pas (le socket est lu par `CoreAppPlugin`,
        // `observe_session_peers`, placé avant `abandon_unsynchronized_session`).
        app.init_resource::<crate::jjrs::restart::OnlineGames>()
            .init_resource::<crate::jjrs::p2p::PreviousSessionPeers>()
            .init_resource::<crate::jjrs::p2p::CurrentSessionPeers>()
            .init_resource::<crate::jjrs::p2p::AdmittedPlayers>()
            .init_resource::<crate::jjrs::p2p::SessionSyncWatch>()
            .init_resource::<crate::jjrs::p2p::SessionPeerLeft>()
            .add_systems(
                OnEnter(AppState::InGame),
                crate::jjrs::p2p::reset_sync_watch,
            )
            .add_systems(
                Update,
                crate::jjrs::p2p::abandon_unsynchronized_session.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                OnEnter(AppState::LobbyOnline),
                crate::jjrs::restart::reset_online_games,
            )
            .add_systems(
                OnEnter(AppState::InGame),
                crate::jjrs::restart::count_online_game,
            );
        // Moitié simulation de l'ancien `system_after_map_loaded` : graine de run, flux de RNG, `Run`,
        // `AppState::InGame`. La session GGRS en ligne est construite par `CoreAppPlugin`
        // (`build_p2p_session`, même `OnEnter`).
        app.add_systems(
            OnEnter(AppState::GameStarting),
            (start_online_run, system_after_map_loaded_local),
        );
        // F5 (chantier m0-v11) : résout une fois, au lancement de la partie, les champs
        // numériques du contenu (vagues, prix, santé) en valeurs concrètes pour le nombre
        // de joueurs de la session — assets chargés, session configurée, avant tout spawn
        // et avant la première frame simulée. Plus jamais lu depuis les assets ensuite
        // (`crate::balance`), donc aucune expression dans l'état rollback. Les résolutions qui
        // copient un asset (`balance_assets`, `powerups_assets`) sont dans `CoreAppPlugin`, avant
        // celles-ci quand elles lisent leur résultat (`SessionPlayers`).
        app.add_systems(
            OnEnter(AppState::GameLoading),
            (
                // T1.2 : patterns nommés (kind `Pattern`), hors rollback, voir
                // `crate::patterns`.
                crate::patterns::resolve_pattern_library_system,
                // T1.10 : progression et mutations, hors rollback, voir `crate::progression`.
                crate::progression::resolve_progression_system,
                // T1.3 : statuts (kind `Status`), hors rollback, voir `crate::statuses`.
                crate::statuses::resolve_status_library_system,
                // T1.9 : horloges et difficulté activées, voir `crate::clock`.
                crate::clock::resolve_clocks_system,
            ),
        );

        app.add_systems(Update, log_ggrs_events.run_if(in_state(AppState::InGame)));

        app.add_plugins(crate::state_trace::StateTracePlugin);
        app.add_plugins(crate::recording::RecordingPlugin);
        // `ALACOD_SNAPSHOT_PROFILE=1` : coût du rollback (sauvegarde, restauration, simulation,
        // inventaire des snapshots) écrit dans le journal à la sortie par `state_trace`. Le
        // runner de scénarios installe le plugin lui-même (métriques) : jamais deux fois.
        if std::env::var("ALACOD_SNAPSHOT_PROFILE").is_ok_and(|v| v == "1")
            && !app.is_plugin_added::<utils::snapshot_profile::SnapshotProfilePlugin>()
        {
            app.add_plugins(utils::snapshot_profile::SnapshotProfilePlugin);
        }

        app.add_systems(
            GgrsSchedule,
            (
                // T1.9 : l'horloge avance avant l'incrément (après le passage d'étage, set
                // `Run`) ; sans horloge ni difficulté activée, elle ne fait rien.
                crate::clock::clock_system.before(increase_frame_system),
                increase_frame_system,
                // Exercice de desync (`ALACOD_DEBUG_DESYNC_AT`) : no-op sans la variable.
                crate::jjrs::debug_desync_system.after(increase_frame_system),
            )
                .in_set(RollbackSystemSet::FrameCounter),
        );
    }

    /// Après tous les `build` : exécuteur des schedules du rollback (voir
    /// [`single_threaded_rollback_schedules`]).
    fn finish(&self, app: &mut App) {
        single_threaded_rollback_schedules(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::schedule::ScheduleLabel as _;
    use bevy::state::app::StatesPlugin;
    use bevy_ggrs::{AdvanceWorld, LoadWorld, ReadInputs, SaveWorld};
    use utils::rollback::TracedTypes;

    /// `CoreSimPlugin` se construit sans `CoreAppPlugin` (ni assets, ni fenêtre, ni socket) sur
    /// `MinimalPlugins` : c'est ce qui rend la simulation réutilisable seule (`bots`, E11). Les schedules
    /// du rollback s'initialisent (`GgrsSchedule` refuse toute ambiguïté) et les cinq types de clôture sont
    /// les derniers enregistrés.
    #[test]
    fn core_sim_seul_sur_minimal_plugins() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(crate::profile::ProfileSettings { dir: None });
        app.add_plugins(CoreSimPlugin(CoreSetupConfig {
            app_name: "core_sim".into(),
            headless: true,
            asset_root: None,
        }));
        app.finish();
        app.cleanup();

        let labels = [
            ("ReadInputs", ReadInputs.intern()),
            ("GgrsSchedule", GgrsSchedule.intern()),
            ("AdvanceWorld", AdvanceWorld.intern()),
            ("SaveWorld", SaveWorld.intern()),
            ("LoadWorld", LoadWorld.intern()),
        ];
        for (name, label) in labels {
            app.world_mut()
                .try_schedule_scope(label, |world, schedule| {
                    schedule
                        .initialize(world)
                        .unwrap_or_else(|e| panic!("{name} : {e}"));
                })
                .unwrap_or_else(|_| panic!("schedule {name} absent"));
        }

        let traced = &app.world().resource::<TracedTypes>().0;
        let tail: Vec<&str> = traced
            .iter()
            .rev()
            .take(5)
            .map(|name| name.rsplit("::").next().unwrap())
            .collect();
        assert_eq!(
            tail,
            [
                "RngStreams",
                "GgrsNetId",
                "FixedTransform3D",
                "FrameCount",
                "GgrsNetIdFactory"
            ],
            "les cinq enregistrements de clôture ferment la liste rollback"
        );
    }
}
