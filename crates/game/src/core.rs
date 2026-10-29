//! Moitié **application** du socle (D69 étape 2, E7) : `CoreSetupPlugin` (le point d'entrée de tous les
//! jeux, qui ajoute `core_sim::CoreSimPlugin` puis `CoreAppPlugin`), `CoreAppPlugin` (chargement des
//! assets, réseau, fenêtre, présentation), `get_default_plugin`, `PresentationPlugin`, `GameInfo`.
//! Chemin final (E9) : `game_app/src/core.rs`.

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
    window::{ExitCondition, WindowResolution},
};
#[cfg(feature = "debug_ui")]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::core_sim::CoreSimPlugin;
use crate::session::{AppState, CoreSetupConfig};
use crate::web::WebPlugin;

use crate::{
    audio::ZAudioPlugin,
    camera::CameraControlPlugin,
    collider::debug::DebugColliderGamePlugin,
    content_hot_reload::ContentHotReloadPlugin,
    frame::FrameDebugUIPlugin,
    global_asset::{add_global_asset, loading_asset_system},
    jjrs::{
        log_ggrs_events,
        p2p_socket::{build_p2p_session, start_matchbox_socket, wait_for_players},
    },
    light::ZLightPlugin,
    ui::{GameUiInputPlugin, GameUiPlugin},
};

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
//
// Deux moitiés ajoutées dans cet ordre : la simulation (`CoreSimPlugin`, tous les enregistrements
// rollback et l'ordre de `GgrsSchedule`), puis l'application (`CoreAppPlugin`). Les plugins de la
// seconde n'enregistrent aucun type rollback : leur position après la simulation ne déplace aucune
// trace. `finish` (exécuteur monothread des schedules du rollback) est celui de `CoreSimPlugin`,
// appelé comme celui de tout plugin imbriqué.
#[derive(Default)]
pub struct CoreSetupPlugin(pub CoreSetupConfig);

impl Plugin for CoreSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CoreSimPlugin(self.0.clone()));
        app.add_plugins(CoreAppPlugin(self.0.clone()));
    }
}

/// Moitié application de `CoreSetupPlugin` : tout ce qui charge des assets, ouvre un socket, lit la
/// fenêtre ou affiche. Ajouté **après** `CoreSimPlugin` ; ses éléments gardent entre eux l'ordre qu'ils
/// avaient dans l'ancien `CoreSetupPlugin::build`. Les systèmes qui s'ordonnaient par rapport à un
/// système de simulation (`.chain()`, `.after`) le font maintenant par une arête explicite posée ici,
/// de l'application vers la simulation (jamais l'inverse).
pub struct CoreAppPlugin(pub CoreSetupConfig);

impl Plugin for CoreAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WebPlugin);
        app.add_plugins(D2AnimationPlugin);
        if !self.0.headless {
            app.add_plugins(PresentationPlugin);
        }

        app.add_plugins(crate::weapon_assets::WeaponAssetsPlugin);
        // Moitié application du personnage (chargeur des configurations, périphériques, debug de l'IA) :
        // sans leafwing en headless.
        app.add_plugins(crate::character::app_plugin::CharacterAppPlugin {
            headless: self.0.headless,
        });
        app.add_plugins(crate::economy_assets::EconomyAssetsPlugin);
        app.add_plugins(crate::powerups_assets::PowerUpsAssetsPlugin);
        app.add_plugins(crate::feedback::FeedbackAssetsPlugin);
        // Lobby et déconnexion (hors `PresentationPlugin` : aussi en headless p2p), puis les entrées
        // des écrans (pause, inventaire, mutation) : leurs modèles sont dans `CoreSimPlugin`.
        app.add_plugins(GameUiPlugin);
        app.add_plugins(GameUiInputPlugin);
        app.add_plugins(crate::waves::assets::WaveAssetsPlugin);

        app.init_resource::<GameInfo>();

        // First step is to load the global asset
        app.add_systems(Startup, add_global_asset);

        app.add_systems(
            Update,
            (loading_asset_system.run_if(in_state(AppState::Loading)),),
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
            wait_for_players.run_if(in_state(AppState::LobbyOnline)),
        );
        // Web : le retour au lobby d'une session hébergée est un crochet JS (application), demandé par la
        // simulation (`run_state::ReturnToHostLobby`) ; voir `run_state::apply_run_request_system`.
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            crate::jjrs::browser::return_to_host_lobby
                .after(crate::run_state::apply_run_request_system)
                .run_if(in_state(AppState::InGame)),
        );
        // D14 (§33) : le socket de la partie est fermé à la sortie de `InGame`.
        // D86 : le watcher du lobby, et la lecture du socket (`observe_session_peers`) que
        // `abandon_unsynchronized_session` (simulation, sans socket) suit dans le même update (ancien
        // `.chain()` du tuple, avec la même condition).
        app.init_resource::<crate::jjrs::p2p_socket::LobbyConnectWatch>()
            .add_systems(
                Update,
                crate::jjrs::p2p_socket::observe_session_peers
                    .before(crate::jjrs::p2p::abandon_unsynchronized_session)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                OnExit(AppState::InGame),
                crate::jjrs::p2p_socket::drop_matchbox_socket,
            );
        // Moitié réseau de l'ancien `system_after_map_loaded` : le canal du socket et la session GGRS
        // (`Session::P2P`) ; la graine, `Run` et `AppState::InGame` sont dans `CoreSimPlugin`.
        app.add_systems(OnEnter(AppState::GameStarting), build_p2p_session);
        // E3c : le nombre de joueurs de la session, lu par la simulation (`SessionPlayers`), publié avant
        // les deux résolutions qui s'en servent ; la table des power-ups est copiée de l'asset **après**
        // l'équilibrage. `resolve_clocks_system` (simulation) passe après `publish_session_players`.
        app.add_systems(
            OnEnter(AppState::GameLoading),
            (
                crate::balance_assets::publish_session_players
                    .before(crate::clock::resolve_clocks_system),
                crate::balance_assets::resolve_balance_system
                    .after(crate::balance_assets::publish_session_players),
                crate::powerups_assets::resolve_powerups_table_system
                    .after(crate::balance_assets::resolve_balance_system),
            ),
        );

        // Télémétrie des desyncs (application) : lit le message écrit par `log_ggrs_events`.
        app.add_systems(
            Update,
            crate::jjrs::telemetry::forward_desync_telemetry
                .after(log_ggrs_events)
                .run_if(in_state(AppState::InGame)),
        );

        // `write_recording_on_exit` passait après `bevy::window::ExitSystems` ; la simulation ne nomme plus ce
        // set (D69 E6) : l'application le place avant `ExitRequests`, que l'enregistrement suit.
        app.configure_sets(
            Last,
            bevy::window::ExitSystems.before(crate::state_trace::ExitRequests),
        );
        // Rechargement à chaud du registre de contenu hors partie (T1.5) : no-op sans
        // `GameRoot` (tests de scénario) ou sans changement de fichier détecté (sans
        // feature `native`, aucun `AssetEvent` de modification n'est jamais émis).
        app.add_plugins(ContentHotReloadPlugin);
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Mode éditeur (K5-1) : voulu par l'exécutable (`CoreSetupConfig::editor`, K5-3) ou par
            // `ALACOD_EDITOR=1`. Posé avant le remote, qui lit sa présence ; il implique le remote.
            // `--sandbox` (`BaseArgsPlugin`, avant ce plugin) implique le mode éditeur.
            let editor = self.0.editor
                || crate::editor_mode::EditorMode::enabled_by_env()
                || app
                    .world()
                    .contains_resource::<crate::sandbox::SandboxRequest>();
            if editor {
                app.insert_resource(crate::editor_mode::EditorMode);
            }
            if let Some(remote) = crate::remote::RemoteControlPlugin::new(editor) {
                app.add_plugins(remote);
            }
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
        // S1-5 : `?perf=1` (navigateur) ou `ALACOD_FPS=1` ; rien n'est installé sans demande.
        if let Some(request) = crate::fps_counter::FpsRequest::requested() {
            app.add_plugins(crate::fps_counter::FpsCounterPlugin(request));
        }
        app.add_plugins(ZLightPlugin);
        app.add_plugins(ZAudioPlugin);
        if debug_ui_enabled() {
            app.add_plugins(FrameDebugUIPlugin);
        }
        app.add_plugins(CameraControlPlugin);
        app.add_plugins(crate::display_interpolation::DisplayInterpolationPlugin);
        app.add_plugins(DebugColliderGamePlugin);
        app.add_plugins(crate::character::app_plugin::CharacterPresentationPlugin);
        app.add_plugins(crate::ui::weapon_visuals::WeaponPresentationPlugin);
        app.add_plugins(crate::feedback::FeedbackPlugin);
        app.add_plugins(crate::items_view::BlankPresentationPlugin);
        app.add_plugins(crate::interaction_view::InteractionPresentationPlugin);
        app.add_plugins(crate::waves::debug::WaveDebugPlugin);
        app.add_plugins(crate::powerup_visuals::PowerUpVisualPlugin);
        app.add_plugins(crate::world_sprites::WorldSpritePlugin);
        app.add_plugins(crate::legibility::LegibilityPlugin);
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
