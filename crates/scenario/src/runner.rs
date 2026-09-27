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
use utils::frame::FrameCount;

use crate::format::{Expectation, Scenario};

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

    ScenarioOutcome {
        trace,
        failures,
        summary,
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
