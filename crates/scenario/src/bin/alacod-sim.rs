//! `alacod-sim` (T2.11) : joue des centaines de parties bots headless, une par graine, et sort
//! des métriques JSON.
//!
//! ```text
//! alacod-sim --game zombies --bots 4 --profiles fonceur,fonceur,prudent,immobile \
//!     --seeds 1..50 --until-wave 10 --max-frames 20000 \
//!     [--save-scenario <dossier>] [--json <fichier>]
//! ```
//!
//! - `--seeds A..B` : graines `A` à `B` **inclusivement** (`1..50` = 50 graines, la carte est
//!   générée avec `map_seed = graine`).
//! - Un scénario est construit en mémoire par graine (`frames: max_frames`, un `PlayerScript` par
//!   bot, aucun `inputs`) et joué en synctest (comme `make test_scenarios`), avec arrêt anticipé
//!   dès que `--until-wave` est atteinte ou que tous les joueurs sont morts
//!   (`scenario::runner::StopEarly`).
//! - `--save-scenario <dossier>` : écrit `seed_<graine>.ron`, l'enregistrement exact des inputs
//!   envoyés à GGRS (rejouable en `Scripted`, voir `crates/scenario/tests/bots.rs`).
//! - `--json <fichier>` : le JSON (toujours un tableau, une entrée par graine) va dans ce
//!   fichier plutôt que stdout.
//! - Code de sortie 1 si au moins une graine a un desync (mismatch synctest) : un vrai bug de
//!   déterminisme, jamais à masquer (voir la doc de `bots::input`).

use std::path::PathBuf;
use std::time::Instant;

use game::character::player::input::InputSource;
use game::replay::{BotProfile, PlayerScript, Scenario};
use scenario::runner::{self, StopEarly};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SimResult {
    seed: i32,
    wave: u32,
    frames: u32,
    deaths: u32,
    kills: u32,
    wall_seconds: f64,
    desync: bool,
    sim_fps: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failures: Vec<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opt = |name: &str| -> Option<String> {
        args.iter().position(|a| a == name).map(|i| {
            args.get(i + 1)
                .unwrap_or_else(|| panic!("{name} demande une valeur"))
                .clone()
        })
    };
    let require = |name: &str| {
        opt(name).unwrap_or_else(|| {
            panic!(
                "usage : alacod-sim --game <jeu> --bots <n> --profiles <a,b,...> \
                 --seeds <de>..<à> --until-wave <n> --max-frames <n> \
                 [--save-scenario <dossier>] [--json <fichier>] ({name} manquant)"
            )
        })
    };

    let game = opt("--game").unwrap_or_else(|| "zombies".into());
    let bots: usize = require("--bots").parse().expect("--bots : entier");
    let profiles: Vec<BotProfile> = require("--profiles")
        .split(',')
        .map(|name| {
            BotProfile::parse_name(name.trim()).unwrap_or_else(|| {
                panic!(
                    "--profiles : profil inconnu « {name} » (attendu : immobile, fonceur, prudent)"
                )
            })
        })
        .collect();
    assert_eq!(
        profiles.len(),
        bots,
        "--profiles doit avoir {bots} entrées (--bots {bots}), en a {}",
        profiles.len()
    );

    let (seed_from, seed_to) = parse_seed_range(&require("--seeds"));
    let until_wave: u32 = require("--until-wave")
        .parse()
        .expect("--until-wave : entier");
    // Défaut généreux (20 000 frames ≈ 5.5 min de partie simulée) : `--until-wave` arrête
    // presque toujours la partie bien avant, `--max-frames` n'est qu'un filet de sécurité.
    let max_frames: u32 = opt("--max-frames")
        .map(|v| v.parse().expect("--max-frames : entier"))
        .unwrap_or(20_000);
    let save_scenario_dir = opt("--save-scenario").map(PathBuf::from);
    let json_path = opt("--json").map(PathBuf::from);

    if let Some(dir) = &save_scenario_dir {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("--save-scenario {dir:?} : {e}"));
    }

    // Carte de départ du jeu (manifeste, T1.5) : la même que ses parties normales, plutôt que le
    // défaut générique de `Scenario` (`exemples/test_map.ldtk`, qui n'est correct que par
    // coïncidence pour `zombies`).
    let game_root = runner::game_dir(&game);
    let (_registry, manifest, content_errors) = content::load_and_lint(&game_root)
        .unwrap_or_else(|e| panic!("alacod-sim : « {game} » : game.ron invalide : {e}"));
    assert!(
        content_errors.is_empty(),
        "alacod-sim : « {game} » : contenu invalide :\n{content_errors:#?}"
    );
    let map = manifest.entry.start_map.clone();

    eprintln!(
        "alacod-sim : {game}, {bots} bots ({}), graines {seed_from}..={seed_to}, jusqu'à la vague {until_wave} ou {max_frames} frames",
        profiles.iter().map(|p| p.name()).collect::<Vec<_>>().join(",")
    );

    let mut results = Vec::new();
    let mut any_desync = false;
    let run_started = Instant::now();

    for seed in seed_from..=seed_to {
        let players = profiles
            .iter()
            .map(|profile| PlayerScript {
                inputs: vec![],
                bot: Some(*profile),
                ..Default::default()
            })
            .collect();
        let run_scenario = Scenario {
            game: game.clone(),
            map: map.clone(),
            map_seed: seed,
            frames: max_frames,
            players,
            expect: vec![],
            weapon_overrides: vec![],
            wave_overrides: None,
            invariants: Default::default(),
            powerups: vec![],
            powerup_drop_chance_override: None,
        };

        let stop_early = StopEarly {
            until_wave: Some(until_wave),
            stop_when_all_players_dead: true,
        };

        let wall_start = Instant::now();
        // `InputSource::Bot` : aucun joueur scripté ici, contrairement aux scénarios RON qui
        // gardent `Scripted` comme mode de base (voir `runner::build_app`) — mais le résultat
        // serait identique avec `Scripted` : `BotAssignments` couvre tous les joueurs.
        let outcome = runner::run_with_options(
            &run_scenario,
            |app| {
                app.insert_resource(InputSource::Bot);
            },
            Some(stop_early),
        );
        let wall_seconds = wall_start.elapsed().as_secs_f64();

        let desync = outcome
            .failures
            .iter()
            .any(|f| f.contains("synctest mismatch"));
        any_desync |= desync;

        if let Some(dir) = &save_scenario_dir {
            let path = dir.join(format!("seed_{seed}.ron"));
            std::fs::write(&path, outcome.recorded.to_ron() + "\n")
                .unwrap_or_else(|e| panic!("--save-scenario {path:?} : {e}"));
        }

        let deaths = (bots as u32).saturating_sub(outcome.metrics.players_alive);
        let wave = outcome.metrics.final_wave;
        let frames_reached = outcome.metrics.frames;
        let kills = outcome.metrics.kills;
        let sim_fps = outcome.metrics.sim_fps;
        let desync_flag = if desync { "  DESYNC" } else { "" };
        eprintln!(
            "seed {seed:>6} : vague {wave:>2}  frames {frames_reached:>6}  morts {deaths}/{bots}  kills {kills:>3}  {sim_fps:>6.1} fps  {wall_seconds:>5.2}s{desync_flag}"
        );

        results.push(SimResult {
            seed,
            wave: outcome.metrics.final_wave,
            frames: outcome.metrics.frames,
            deaths,
            kills: outcome.metrics.kills,
            wall_seconds,
            desync,
            sim_fps: outcome.metrics.sim_fps,
            failures: outcome.failures,
        });
    }

    eprintln!(
        "alacod-sim : {} graine(s) en {:.1}s",
        results.len(),
        run_started.elapsed().as_secs_f64()
    );

    let json = serde_json::to_string_pretty(&results).expect("sérialisation JSON");
    match &json_path {
        Some(path) => {
            std::fs::write(path, json + "\n").unwrap_or_else(|e| panic!("--json {path:?} : {e}"))
        }
        None => println!("{json}"),
    }

    if any_desync {
        eprintln!(
            "alacod-sim : au moins un desync (mismatch synctest) : bug de déterminisme à corriger, \
             pas à masquer (voir le scénario sauvé avec --save-scenario et la frame en cause)"
        );
        std::process::exit(1);
    }
}

/// `--seeds A..B` : bornes inclusives (`1..50` = 50 graines). `map_seed` est un `i32`
/// (`game::replay::Scenario`), les graines le sont donc aussi.
fn parse_seed_range(spec: &str) -> (i32, i32) {
    let (from, to) = spec
        .split_once("..")
        .unwrap_or_else(|| panic!("--seeds : attendu <de>..<à>, reçu « {spec} »"));
    let from: i32 = from
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("--seeds : borne basse invalide : « {from} »"));
    let to: i32 = to
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("--seeds : borne haute invalide : « {to} »"));
    assert!(
        from <= to,
        "--seeds : {from}..{to} est vide (borne basse > borne haute)"
    );
    (from, to)
}
