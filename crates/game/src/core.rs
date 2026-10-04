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
    web::WebPlugin,
};

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
        GameDisconnectedEvent, GggrsSessionConfigurationState,
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

        app.add_message::<GameDisconnectedEvent>();

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
            )
                .in_set(RollbackSystemSet::FrameCounter),
        );
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

        let plugins = DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(asset_plugin)
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

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        app.add_plugins(ZLightPlugin);
        app.add_plugins(ZAudioPlugin);
        app.add_plugins(FrameDebugUIPlugin);
        app.add_plugins(CameraControlPlugin);
        app.add_plugins(DebugColliderGamePlugin);
        app.add_plugins(crate::character::CharacterPresentationPlugin);
        app.add_plugins(crate::ui::weapon_visuals::WeaponPresentationPlugin);
        app.add_plugins(crate::feedback::FeedbackPlugin);
        app.add_plugins(crate::ui::hud::HudPlugin);
        app.add_plugins(crate::ui::mutation_screen::MutationScreenPlugin);
        app.add_plugins(crate::ui::floor_transition::FloorTransitionPlugin);
        #[cfg(feature = "debug_ui")]
        app.add_plugins(EguiPlugin::default());

        #[cfg(feature = "debug_ui")]
        app.add_plugins(WorldInspectorPlugin::new());
    }
}

/// Fréquence de simulation GGRS (valeur par défaut de `RollbackFrameRate`).
pub const SIM_FPS: u64 = 60;

/// Mode headless (`ALACOD_HEADLESS=1`) : pas de fenêtre ni de GPU, et le temps
/// avance d'une frame de simulation par update. Pour les tests et les replays.
pub fn is_headless() -> bool {
    std::env::var("ALACOD_HEADLESS").is_ok_and(|v| v == "1")
}
