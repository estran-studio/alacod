use animation::D2AnimationPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy::winit::WinitPlugin;
use bevy::{
    app::{PluginGroupBuilder, ScheduleRunnerPlugin},
    asset::AssetMetaCheck,
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    time::TimeUpdateStrategy,
    window::{ExitCondition, WindowResolution},
};
use bevy_fixed::{
    fixed_math::{self, sync_bevy_transforms_from_fixed},
    rng::RngStreams,
};
use bevy_ggrs::{GgrsPlugin, GgrsSchedule};
#[cfg(feature = "debug_ui")]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use serde::{Deserialize, Serialize};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
};

use crate::web::WebPlugin;

use crate::{
    audio::ZAudioPlugin,
    camera::CameraControlPlugin,
    character::{player::jjrs::PeerConfig, BaseCharacterGamePlugin},
    collider::{debug::DebugColliderGamePlugin, BaseColliderGamePlugin},
    content_hot_reload::ContentHotReloadPlugin,
    frame::{increase_frame_system, FrameDebugUIPlugin},
    global_asset::{add_global_asset, loading_asset_system},
    jjrs::{
        local::{setup_ggrs_local, system_after_map_loaded_local},
        log_ggrs_events,
        p2p::{start_matchbox_socket, system_after_map_loaded, wait_for_players},
        GggrsSessionConfigurationState, SessionLostEvent,
    },
    light::ZLightPlugin,
    system_set::RollbackSystemSet,
    ui::GameUiPlugin,
    waves::WaveSystemPlugin,
    weapons::BaseWeaponGamePlugin,
};

// Configuration that is static and bundle with the game
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CoreSetupConfig {
    pub app_name: String,
    /// Sans fenêtre ni GPU, une frame de simulation par update (voir [`is_headless`]).
    pub headless: bool,
    /// Dossier des assets ; `None` pour le dossier par défaut de bevy.
    pub asset_root: Option<String>,
}

impl CoreSetupConfig {
    /// Config d'un exécutable : le mode headless vient de `ALACOD_HEADLESS`.
    pub fn from_env(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            headless: is_headless(),
            asset_root: None,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading, // Initial loading step for all the required global asset to be resolved
    LobbyLocal,   // Create a local lobby for lan UDP or SyncTest Session
    LobbyOnline,  // Create an online lobby with matchbox
    GameLoading, // After the lobby as agree on the game parameters all required asset are loaded before the game can start
    GameStarting, // To launch the session after the game is loaded
    InGame,      // When the game is played with the active ggrs session from local or online
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OnlineState {
    #[default]
    Unset, // No ggrs system enable to start a game
    Online,  // For ggrs p2p system to be enable
    Offline, // For ggrs synctest/lan system to be enabe
}

// Ressource to share information about the identity of this game instance ( game name , version , .... )
// this is used between client to validate that their binary are compatible
#[derive(Debug, Clone, Resource)]
pub struct GameInfo {
    pub version: String,
}

impl Default for GameInfo {
    fn default() -> Self {
        Self {
            version: env!("APP_VERSION").into(),
        }
    }
}

// Core plugin for alacod
// Configure all infrastructure and game mechanics.
// Add other plugins for game logics and world creation to make a full game
#[derive(Default)]
pub struct CoreSetupPlugin(pub CoreSetupConfig);

impl Plugin for CoreSetupPlugin {
    fn build(&self, app: &mut App) {
        if self.0.headless {
            // Chaque update avance d'exactement une frame GGRS, aussi vite que le CPU le permet
            app.insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_nanos(1_000_000_000 / SIM_FPS),
            ));
        }

        app.add_plugins(WebPlugin);
        app.add_plugins(D2AnimationPlugin);
        app.add_plugins(GgrsPlugin::<PeerConfig>::default());
        // `GgrsPlugin` installe aussi `RollbackDespawnPlugin` : les entités de la simulation se
        // détruisent avec `despawn_rollback()` (voir CLAUDE.md, règle 9) et `GgrsSchedule` refuse
        // toute ambiguïté d'ordre entre systèmes (`ambiguity_detection: Error`).
        if !self.0.headless {
            app.add_plugins(PresentationPlugin);
        }

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
        // charge `economy.ron`/`perks.ron` et ajoute le système de points
        // (`RollbackSystemSet::Run`). Avant `InteractionPlugin` : les achats (portes, armes
        // murales, perks) vivent dans `interaction.rs` et lisent `Currency`/`Perks`.
        app.add_plugins(run::RunPlugin);
        // Terrain des cavernes (T1.6) : `CellGrid` et demandes de destruction, toutes neutres
        // au checksum (grille vide et files vides hors caverne : traces inchangées).
        app.add_plugins(world::WorldPlugin);
        // Effets v1 (T1.10) : composants neutres, exécution dans `DeathManagement`.
        app.add_plugins(crate::effects_runtime::EffectsRuntimePlugin);
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
        app.add_plugins(crate::loot::LootPlugin);
        app.add_plugins(crate::shop::ShopPlugin);
        // M2-T11 : fosses, tables renversables, barils explosifs (inertes sans eux)
        app.add_plugins(crate::props::PropsPlugin);
        app.add_plugins(crate::boss::BossPlugin);
        app.add_plugins(crate::profile::ProfilePlugin);
        // Feedback v1 (T1.17) : journal de présentation (`FeedbackLog`), rempli aussi en
        // headless pour prouver le feedback sans écran ; le rendu est dans `PresentationPlugin`.
        app.add_plugins(crate::feedback::FeedbackLogPlugin);
        app.add_plugins(GameUiPlugin);
        app.add_plugins(WaveSystemPlugin);
        // État de run (T2.4, chantier F1) : condition de victoire, résumé, relance sans
        // relancer le binaire (`RunRequest`). Voir `crate::run_state`.
        app.add_plugins(crate::run_state::RunStatePlugin);

        app.init_resource::<GameInfo>();
        app.init_resource::<GggrsSessionConfigurationState>();
        app.init_resource::<GgrsNetIdFactory>();
        app.init_resource::<FrameCount>();

        // Flux RNG nommés (T1.6) : remplacés au démarrage de session par ceux dérivés de
        // `RunSeed` (jjrs/local.rs, jjrs/p2p.rs) ; enregistrés en rollback plus bas.
        app.insert_resource(RngStreams::new(12345));

        app.add_message::<SessionLostEvent>();

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

        // First step is to load the global asset
        app.add_systems(Startup, add_global_asset);

        app.add_systems(
            Update,
            (loading_asset_system.run_if(in_state(AppState::Loading)),),
        );

        // Sync FixedTransform3D to Transform after GGRS schedule runs
        // This ensures visual representation matches the rollback simulation state
        app.add_systems(
            PostUpdate,
            sync_bevy_transforms_from_fixed.run_if(in_state(AppState::InGame)),
        );

        // Mode allumette (natif) : le flux HTTP (challenge → login → lobby → ICE)
        // remplit la ressource `AllumetteConfig` avant que le socket ne l'ouvre avec
        // l'URL ws(s)://hôte/JWT. Chaîné : l'insertion est visible du second système.
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            OnEnter(AppState::LobbyOnline),
            (
                crate::jjrs::allumette::start_allumette_flow,
                start_matchbox_socket,
            )
                .chain(),
        );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(OnEnter(AppState::LobbyOnline), start_matchbox_socket);

        app.add_systems(
            Update,
            (
                wait_for_players.run_if(in_state(AppState::LobbyOnline)),
                // D13 : après un retour au lobby local, attend que le joueur relance
                // (`ui::lobby` retire `LocalLobbyHold`) au lieu de relancer aussitôt.
                setup_ggrs_local
                    .run_if(in_state(AppState::LobbyLocal))
                    .run_if(not(resource_exists::<crate::run_state::LocalLobbyHold>)),
            ),
        );
        // D14 (§33) : restart en ligne — compteur de parties en ligne (remis à zéro à chaque
        // entrée normale dans `LobbyOnline`), socket de la partie fermé à sa sortie.
        app.init_resource::<crate::jjrs::restart::OnlineGames>()
            .add_systems(
                OnEnter(AppState::LobbyOnline),
                crate::jjrs::restart::reset_online_games,
            )
            .add_systems(
                OnEnter(AppState::InGame),
                crate::jjrs::restart::count_online_game,
            )
            .add_systems(
                OnExit(AppState::InGame),
                crate::jjrs::restart::drop_matchbox_socket,
            );
        // System for ggrs that register the session when the map is correctly loaded
        app.add_systems(
            OnEnter(AppState::GameStarting),
            (system_after_map_loaded, system_after_map_loaded_local),
        );
        // F5 (chantier m0-v11) : résout une fois, au lancement de la partie, les champs
        // numériques du contenu (vagues, prix, santé) en valeurs concrètes pour le nombre
        // de joueurs de la session — assets chargés, session configurée, avant tout spawn
        // et avant la première frame simulée. Plus jamais lu depuis les assets ensuite
        // (`crate::balance`), donc aucune expression dans l'état rollback.
        app.add_systems(
            OnEnter(AppState::GameLoading),
            (
                crate::balance::resolve_balance_system,
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
        // Rechargement à chaud du registre de contenu hors partie (T1.5) : no-op sans
        // `GameRoot` (tests de scénario) ou sans changement de fichier détecté (sans
        // feature `native`, aucun `AssetEvent` de modification n'est jamais émis).
        app.add_plugins(ContentHotReloadPlugin);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(remote) = crate::remote::RemoteControlPlugin::from_env() {
            app.add_plugins(remote);
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

/// Exécuteur monothread pour les schedules du rollback (`ReadInputs`, `AdvanceWorld`,
/// `GgrsSchedule`, `SaveWorld`, `LoadWorld`), dans le jeu comme dans les scénarios.
///
/// La simulation compte des centaines de petits systèmes par frame : l'exécuteur multithread
/// de Bevy y coûte plus (réveils de threads, ≈ 1 500 `futex` par frame) qu'il ne rapporte.
/// Mesuré le 2026-10-15 sur le smoke p2p de `zombies` à deux clients (97 entités), profil
/// `utils::snapshot_profile` : une frame de simulation à 6,8 ms en moyenne (23,6 ms au pire)
/// en multithread, contre 0,15 ms sur la même charge en monothread dans le runner de
/// scénarios. En ligne, un rollback de 12 frames coûte donc 80 ms au lieu de 2. L'ordre
/// d'exécution ne change aucun résultat : `GgrsSchedule` refuse toute ambiguïté d'ordre
/// (`ambiguity_detection: Error`) et les traces de référence le prouvent.
/// `ALACOD_EXECUTOR=multi` rétablit l'exécuteur par défaut (comparaison).
pub fn single_threaded_rollback_schedules(app: &mut App) {
    if std::env::var("ALACOD_EXECUTOR").is_ok_and(|v| v == "multi") {
        return;
    }
    let mut schedules = app
        .world_mut()
        .resource_mut::<bevy::ecs::schedule::Schedules>();
    use bevy::ecs::schedule::ScheduleLabel as _;
    let labels = [
        bevy_ggrs::ReadInputs.intern(),
        bevy_ggrs::AdvanceWorld.intern(),
        GgrsSchedule.intern(),
        bevy_ggrs::SaveWorld.intern(),
        bevy_ggrs::LoadWorld.intern(),
    ];
    for label in labels {
        if let Some(schedule) = schedules.get_mut(label) {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        }
    }
}

impl CoreSetupPlugin {
    pub fn get_default_plugin(&self) -> PluginGroupBuilder {
        let window_plugin = WindowPlugin {
            primary_window: Some(Window {
                title: self.0.app_name.to_string(),
                resolution: WindowResolution::new(800, 600),

                resizable: true,
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#bevy-canvas".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut asset_plugin = AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            #[cfg(target_arch = "wasm32")]
            file_path: format!("{}/assets", env!("APP_VERSION")),
            ..Default::default()
        };
        if let Some(asset_root) = &self.0.asset_root {
            asset_plugin.file_path = asset_root.clone();
        }

        // Une caverne = un asset (m1-integration-scenarios) : source `cave://`, enregistrée
        // avant `AssetPlugin` sur la même racine.
        let cave_source = crate::cave_assets::CaveAssetSourcePlugin {
            file_path: asset_plugin.file_path.clone(),
        };
        let floor_source = crate::cave_assets::FloorAssetSourcePlugin {
            file_path: asset_plugin.file_path.clone(),
        };
        let plugins = DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(asset_plugin)
            .add_before::<AssetPlugin>(cave_source)
            .add_before::<AssetPlugin>(floor_source)
            .disable::<LogPlugin>();

        if !self.0.headless {
            return plugins.set(window_plugin);
        }

        // Headless : ni fenêtre ni GPU, la boucle tourne sans attendre
        let plugins = plugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..Default::default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                    backends: None,
                    ..Default::default()
                })),
                ..Default::default()
            })
            .add(ScheduleRunnerPlugin::run_loop(std::time::Duration::ZERO));
        #[cfg(not(target_arch = "wasm32"))]
        let plugins = plugins.disable::<WinitPlugin>();
        plugins
    }
}

/// Tout ce qui ne sert qu'à afficher ou faire entendre la partie : caméra, lumière,
/// audio, UI de debug. Absent en headless ; la simulation ne doit jamais en dépendre.
pub struct PresentationPlugin;

/// Presentation diagnostics are opt-in, independently of the native build features.
pub fn debug_ui_enabled() -> bool {
    std::env::var("ALACOD_DEBUG_UI").is_ok_and(|value| value == "1")
}

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = std::env::var_os("ALACOD_PERF_CSV") {
            app.add_plugins(crate::performance::PerformanceCapturePlugin(path.into()));
        }
        app.add_plugins(ZLightPlugin);
        app.add_plugins(ZAudioPlugin);
        if debug_ui_enabled() {
            app.add_plugins(FrameDebugUIPlugin);
        }
        app.add_plugins(CameraControlPlugin);
        app.add_plugins(DebugColliderGamePlugin);
        app.add_plugins(crate::character::CharacterPresentationPlugin);
        app.add_plugins(crate::ui::weapon_visuals::WeaponPresentationPlugin);
        app.add_plugins(crate::feedback::FeedbackPlugin);
        app.add_plugins(crate::items::BlankPresentationPlugin);
        app.add_plugins(crate::powerup_visuals::PowerUpVisualPlugin);
        app.add_plugins(crate::room_fog::RoomFogPlugin);
        app.add_plugins(crate::ui::hud::HudPlugin);
        #[cfg(feature = "ui_hui")]
        app.add_plugins((
            crate::ui::screens::hui::HuiScreensPlugin,
            crate::ui::game_over::GameOverUiPlugin,
            crate::ui::mutation_hui::MutationHuiPlugin,
        ));
        app.add_plugins(crate::ui::floor_transition::FloorTransitionPlugin);
        #[cfg(feature = "debug_ui")]
        if debug_ui_enabled() {
            app.add_plugins(EguiPlugin::default());
            app.add_plugins(WorldInspectorPlugin::new());
        }
    }
}

/// Fréquence de simulation GGRS (valeur par défaut de `RollbackFrameRate`).
pub const SIM_FPS: u64 = 60;

/// Mode headless (`ALACOD_HEADLESS=1`) : pas de fenêtre ni de GPU, et le temps
/// avance d'une frame de simulation par update. Pour les tests et les replays.
pub fn is_headless() -> bool {
    std::env::var("ALACOD_HEADLESS").is_ok_and(|v| v == "1")
}
