//! Affiche un scénario : `cargo run -p scenario --features render --bin play_scenario -- <fichier.ron>`
//! (ou `make play_scenario SCENARIO=<nom>`).
//!
//! `--capture <dossier> [--every N]` : capture une image toutes les N frames de simulation
//! (utilisé par `scripts/scenario-video`).

use scenario::runner::{capture, play, CaptureConfig};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args
        .first()
        .expect("usage : play_scenario <fichier.ron> [--capture <dossier>] [--every N]");
    let option = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .map(|i| args.get(i + 1).unwrap_or_else(|| panic!("{name} demande une valeur")).clone())
    };

    let source = std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{path} : {err}"));
    let scenario = scenario::Scenario::from_ron(&source).unwrap_or_else(|err| panic!("{path} : {err}"));

    match option("--capture") {
        Some(dir) => {
            let every = option("--every").map_or(1, |n| n.parse().expect("--every : entier"));
            capture(&scenario, CaptureConfig { dir: dir.into(), every });
        }
        None => {
            play(&scenario);
        }
    }
}
