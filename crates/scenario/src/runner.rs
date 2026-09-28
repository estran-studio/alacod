//! Exécution headless d'un scénario, dans le processus courant.

use bevy::prelude::*;
use game::{
    args::{GameArgs, GameArgsPlugin},
    character::player::{
        input::{InputSource, ScriptedInputs},
        Player,
    },
    core::{CoreSetupConfig, CoreSetupPlugin},
    jjrs::PlayerConfig,
    state_trace::{StateTraceRecorder, StateTraceRecorderPlugin},
    waves::{WaveDebugEnabled, WaveModeEnabled, WaveState},
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};
use crate::events::{GameEvent, GameEvents, GameEventsPlugin};
use game::recording::InputRecorder;
use map::generation::config::MapGenerationConfig;
use utils::frame::FrameCount;

use game::replay::{Expectation, Scenario};

/// Updates maximum pour charger la map avant la première frame de simulation.
const MAX_LOADING_UPDATES: u32 = 10_000;

/// Résultat d'un scénario.
pub struct ScenarioOutcome {
    /// Trace d'état, une ligne par frame (voir `game::state_trace`).
    pub trace: Vec<String>,
    /// Attentes non satisfaites, avec leur frame.
    pub failures: Vec<String>,
    /// État en fin de partie, pour écrire ou déboguer un scénario.
    pub summary: String,
    /// Inputs réellement envoyés à GGRS, réenregistrés en scénario.
    pub recorded: Scenario,
    /// Moments clés de la partie.
    pub events: Vec<GameEvent>,
}

/// Dossier des assets du dépôt.
pub fn assets_dir() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").to_string()
}

/// App de la partie décrite par le scénario (même partie que `map_explorer`).
/// Avec `headless: false`, la partie est affichée (voir le binaire `play_scenario`).
pub fn build_app(scenario: &Scenario, headless: bool) -> App {
    let core_plugin = CoreSetupPlugin(CoreSetupConfig {
        app_name: "scenario".into(),
        headless,
        asset_root: Some(assets_dir()),
    });

    let mut app = App::new();
    app.add_plugins(core_plugin.get_default_plugin())
        .add_plugins(GameArgsPlugin(game_args(scenario.players.len())))
        .add_plugins(core_plugin)
        .add_plugins(LdtkRoguePlugin)
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: scenario.map.clone(),
            seed: scenario.map_seed,
        }))
        .insert_resource(WaveModeEnabled(true))
        .insert_resource(WaveDebugEnabled(true))
        .add_plugins(StateTraceRecorderPlugin { full: false })
        .add_plugins(GameEventsPlugin)
        .insert_resource(InputSource::Scripted)
        .insert_resource::<ScriptedInputs>(scenario.scripted_inputs());
    app
}

/// Joue le scénario jusqu'à `scenario.frames` et vérifie ses attentes.
pub fn run(scenario: &Scenario) -> ScenarioOutcome {
    let mut app = build_app(scenario, true);
    app.finish();
    app.cleanup();

    let mut pending: Vec<&Expectation> = scenario.expect.iter().collect();
    pending.sort_by_key(|e| e.at_frame());
    let mut failures = Vec::new();

    let max_updates = MAX_LOADING_UPDATES + scenario.frames;
    let mut frame = 0;
    for _ in 0..max_updates {
        app.update();
        frame = app.world().resource::<FrameCount>().frame;

        while pending.first().is_some_and(|e| e.at_frame() <= frame) {
            let expectation = pending.remove(0);
            if let Err(reason) = check(app.world_mut(), expectation) {
                failures.push(format!("frame {frame}: {expectation:?} : {reason}"));
            }
        }

        if frame >= scenario.frames {
            break;
        }
    }

    if frame < scenario.frames {
        failures.push(format!(
            "la simulation n'a atteint que la frame {frame} sur {} (map pas chargée ?)",
            scenario.frames
        ));
    }

    let trace = app
        .world()
        .resource::<StateTraceRecorder>()
        .lines_until(scenario.frames)
        .map(str::to_string)
        .collect();

    let summary = summarize(app.world_mut(), frame);
    let recorded = app
        .world()
        .resource::<InputRecorder>()
        .to_scenario(app.world().get_resource::<MapGenerationConfig>());

    let events = app.world().resource::<GameEvents>().events.clone();

    ScenarioOutcome {
        trace,
        failures,
        summary,
        recorded,
        events,
    }
}

fn game_args(player_count: usize) -> GameArgs {
    GameArgs {
        local_port: 0,
        number_player: player_count,
        players: (0..player_count)
            .map(|i| PlayerConfig {
                name: format!("Player {}", i + 1),
                pubkey: "local".into(),
                is_local: true,
            })
            .collect(),
        spectators: vec![],
        matchbox: String::new(),
        lobby: String::new(),
        cid: "scenario".into(),
        debug_ai: false,
        telemetry: false,
        telemetry_url: String::new(),
        telemetry_auth: String::new(),
    }
}

fn check(world: &mut World, expectation: &Expectation) -> Result<(), String> {
    match expectation {
        Expectation::PlayerAlive { handle, .. } => {
            if player_alive(world, *handle) {
                Ok(())
            } else {
                Err("joueur mort".into())
            }
        }
        Expectation::PlayerDead { handle, .. } => {
            if player_alive(world, *handle) {
                Err("joueur vivant".into())
            } else {
                Ok(())
            }
        }
        Expectation::WaveAtLeast { wave, .. } => {
            let current = world.resource::<WaveState>().current_wave;
            if current >= *wave {
                Ok(())
            } else {
                Err(format!("vague {current}"))
            }
        }
        Expectation::KillsAtLeast { kills, .. } => {
            let killed = world.resource::<WaveState>().total_enemies_killed;
            if killed >= *kills {
                Ok(())
            } else {
                Err(format!("{killed} ennemis tués"))
            }
        }
    }
}

fn summarize(world: &mut World, frame: u32) -> String {
    let mut alive: Vec<usize> = world
        .query::<&Player>()
        .iter(world)
        .map(|player| player.handle)
        .collect();
    alive.sort();
    let waves = world.resource::<WaveState>();
    format!(
        "frame {frame} : vague {}, {} ennemis tués, joueurs vivants {alive:?}",
        waves.current_wave, waves.total_enemies_killed
    )
}

fn player_alive(world: &mut World, handle: usize) -> bool {
    world
        .query::<&Player>()
        .iter(world)
        .any(|player| player.handle == handle)
}

/// Joue le scénario avec rendu, à vitesse réelle, et quitte à la fin de ses frames.
/// Les attentes ne sont pas vérifiées : c'est un outil de visualisation.
pub fn play(scenario: &Scenario) -> AppExit {
    let frames = scenario.frames;
    let mut app = build_app(scenario, false);
    app.add_systems(
        Update,
        move |frame: Res<FrameCount>, mut exit: MessageWriter<AppExit>| {
            if frame.frame >= frames {
                exit.write(AppExit::Success);
            }
        },
    );
    app.run()
}

/// Capture d'un scénario en images, pour les vidéos.
pub struct CaptureConfig {
    /// Dossier des images (`frame_00000.png`, ...), numérotées par frame de simulation.
    pub dir: std::path::PathBuf,
    /// Une image toutes les `every` frames (2 → vidéo à 30 images/s).
    pub every: u32,
}

#[derive(Resource)]
struct CaptureState {
    dir: std::path::PathBuf,
    every: u32,
    frames: u32,
    last_captured: Option<u32>,
    updates_after_end: u32,
    /// Image où la caméra rend pendant la capture (taille fixe, indépendante de la fenêtre).
    target: Option<Handle<Image>>,
}

/// Taille des images capturées.
pub const CAPTURE_SIZE: (u32, u32) = (960, 540);

/// Joue le scénario avec rendu et capture une image toutes les `every` frames de
/// simulation. Le temps avance d'exactement une frame par update : l'image `n` montre
/// toujours la frame `n`, quelle que soit la vitesse de la machine.
pub fn capture(scenario: &Scenario, config: CaptureConfig) -> AppExit {
    std::fs::create_dir_all(&config.dir).expect("dossier de capture");

    let mut app = build_app(scenario, false);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_nanos(1_000_000_000 / game::core::SIM_FPS),
    ))
    .insert_resource(CaptureState {
        dir: config.dir,
        every: config.every.max(1),
        frames: scenario.frames,
        last_captured: None,
        updates_after_end: 0,
        target: None,
    })
    .add_systems(Startup, configure_capture_window)
    .add_systems(Update, (render_cameras_to_image, capture_frames).chain());
    app.run()
}

/// La fenêtre ne sert pas : cachée, et sans vsync pour capturer aussi vite que possible.
fn configure_capture_window(mut windows: Query<&mut bevy::window::Window>) {
    for mut window in &mut windows {
        window.present_mode = bevy::window::PresentMode::AutoNoVsync;
        window.visible = false;
    }
}

/// Fait rendre les caméras dans une image de taille fixe ([`CAPTURE_SIZE`]) : le cadrage ne
/// dépend pas de la taille de la fenêtre (donnée par le gestionnaire de fenêtres).
fn render_cameras_to_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<CaptureState>,
    cameras: Query<(Entity, &bevy::camera::RenderTarget), With<Camera>>,
) {
    let target = state
        .target
        .get_or_insert_with(|| {
            images.add(Image::new_target_texture(
                CAPTURE_SIZE.0,
                CAPTURE_SIZE.1,
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                None,
            ))
        })
        .clone();
    for (camera, current) in &cameras {
        if !matches!(current, bevy::camera::RenderTarget::Image(_)) {
            commands
                .entity(camera)
                .insert(bevy::camera::RenderTarget::Image(target.clone().into()));
        }
    }
}

fn capture_frames(
    mut commands: Commands,
    frame: Res<FrameCount>,
    session: Option<Res<bevy_ggrs::Session<game::character::player::jjrs::PeerConfig>>>,
    mut state: ResMut<CaptureState>,
    events: Res<GameEvents>,
    mut exit: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};

    // Rien à capturer avant le début de la partie
    if session.is_none() {
        return;
    }
    let Some(target) = state.target.clone() else {
        return;
    };

    let frame = frame.frame;
    if frame >= state.frames {
        // Laisser le temps aux dernières captures d'être écrites
        state.updates_after_end += 1;
        if state.updates_after_end > 10 {
            // Moments clés à côté des images, pour la page de revue
            let frames = state.frames;
            let events: Vec<&GameEvent> = events.events.iter().filter(|e| e.frame < frames).collect();
            let json = serde_json::to_string_pretty(&events).expect("sérialisation des moments clés");
            std::fs::write(state.dir.join("events.json"), json).expect("écriture de events.json");
            exit.write(AppExit::Success);
        }
        return;
    }

    if frame % state.every == 0 && state.last_captured != Some(frame) {
        let path = state.dir.join(format!("frame_{frame:05}.png"));
        commands.spawn(Screenshot::image(target)).observe(save_to_disk(path));
        state.last_captured = Some(frame);
    }
}
