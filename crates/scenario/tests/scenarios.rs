//! Joue chaque scénario de `tests/scenarios/*.ron`, vérifie ses attentes et compare sa
//! trace d'état à `tests/scenarios/<nom>.trace`.
//!
//! - `make test_scenarios` : compile sans rendu, avec le profil `headless`.
//! - `ALACOD_BLESS=1` : réécrit les traces de référence (après un changement voulu).
//! - `ALACOD_SCENARIO=<nom>` : ne joue que ce scénario.

use std::path::PathBuf;

use scenario::{run, Scenario};

fn scenarios_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/scenarios"))
}

#[test]
fn scenarios() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("scénarios ignorés : compilés avec le rendu des tilemaps (utiliser `make test_scenarios`)");
        return;
    }

    let bless = std::env::var("ALACOD_BLESS").is_ok_and(|v| v == "1");
    let only = std::env::var("ALACOD_SCENARIO").ok().filter(|name| !name.is_empty());

    let mut paths: Vec<PathBuf> = std::fs::read_dir(scenarios_dir())
        .expect("dossier tests/scenarios")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ron"))
        .collect();
    paths.sort();

    let mut failures = Vec::new();
    for path in paths {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if only.as_ref().is_some_and(|only| *only != name) {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        let scenario = match Scenario::from_ron(&source) {
            Ok(scenario) => scenario,
            Err(err) => {
                failures.push(format!("{name}: RON invalide : {err}"));
                continue;
            }
        };

        let started = std::time::Instant::now();
        let outcome = run(&scenario);
        eprintln!("{name}: {} ({:.1?})", outcome.summary, started.elapsed());
        if std::env::var("ALACOD_EVENTS").is_ok_and(|v| v == "1") {
            for event in &outcome.events {
                eprintln!("  f{:>5} {:<7} {}", event.frame, event.kind, event.label);
            }
        }

        failures.extend(outcome.failures.iter().map(|f| format!("{name}: {f}")));

        let golden_path = path.with_extension("trace");
        let trace = outcome.trace.join("\n") + "\n";
        if bless {
            std::fs::write(&golden_path, &trace).unwrap();
            eprintln!("{name}: trace de référence écrite");
        } else if let Ok(golden) = std::fs::read_to_string(&golden_path) {
            if let Some((line, (expected, actual))) = golden
                .lines()
                .zip(trace.lines())
                .enumerate()
                .find(|(_, (expected, actual))| expected != actual)
            {
                failures.push(format!(
                    "{name}: trace différente de la référence à la ligne {} (attendu `{expected}`, obtenu `{actual}`) ; ALACOD_BLESS=1 si le changement est voulu",
                    line + 1
                ));
            } else if golden.lines().count() != trace.lines().count() {
                failures.push(format!("{name}: la trace n'a pas la même longueur que la référence"));
            }
        } else {
            failures.push(format!("{name}: pas de trace de référence (lancer avec ALACOD_BLESS=1)"));
        }
    }

    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

/// Un scénario rejoué depuis son propre enregistrement donne la même trace : ce que
/// l'enregistrement capture suffit à reproduire la partie.
#[test]
fn recording_replays_identically() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let source = std::fs::read_to_string(scenarios_dir().join("shoot_around.ron")).unwrap();
    let original = run(&Scenario::from_ron(&source).unwrap());

    // Le RON écrit doit se relire
    let mut recorded = Scenario::from_ron(&original.recorded.to_ron()).unwrap();
    recorded.frames = original.trace.len() as u32;
    let replayed = run(&recorded);

    assert_eq!(original.trace, replayed.trace, "le replay de l'enregistrement diverge");
}
