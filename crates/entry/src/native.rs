//! Modes d'outillage du binaire d'un jeu (natif) : `lint` et `scenarios`, choisis par le premier
//! argument. Une partie ne prend jamais d'argument positionnel (`--players`, `--sandbox`… sont des
//! options) : aucun conflit avec la ligne de commande de la partie.

use bevy::prelude::AppExit;

use crate::{GameSpec, Source};

const USAGE: &str = "usage : <jeu> lint [--json] | <jeu> scenarios gen [--play] [--bless] [--only id] [--list] [--no-trace] [--out dossier] [--json] | <jeu> scenarios play [chemin…] [--bless] [--json] [--only id] [--list] | <jeu> scenarios capture <scenario.ron> --dir <dossier> [--every N] [--follow <handle>] [--size <LxH>] [--at <f,…>] [--legend]";

fn exit(code: i32) -> AppExit {
    match u8::try_from(code) {
        Ok(0) => AppExit::Success,
        Ok(code) => AppExit::from_code(code),
        Err(_) => AppExit::from_code(1),
    }
}

/// `Some(code)` si la ligne de commande demande un mode d'outillage (le plugin est déjà installé).
pub(crate) fn tooling_mode(spec: &GameSpec) -> Option<AppExit> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first()?.as_str();
    if !matches!(mode, "lint" | "scenarios") {
        return None;
    }
    let Source::Dir(game_dir) = &spec.source;
    let rest = &args[1..];
    Some(exit(match mode {
        "lint" => lint(spec, game_dir, rest),
        _ => match rest.split_first() {
            Some((sub, options)) if sub == "gen" => {
                scenario::cli::run_gen(Some(game_dir.clone()), options)
            }
            Some((sub, options)) if sub == "play" => scenario::cli::run_play(game_dir, options),
            Some((sub, options)) if sub == "capture" => capture(spec, game_dir, options),
            _ => {
                eprintln!("{USAGE}");
                2
            }
        },
    }))
}

/// `<jeu> lint [--json]` : `alacod lint` avec le plugin installé (ses règles Rust s'exécutent).
fn lint(spec: &GameSpec, game_dir: &std::path::Path, args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let (registry, manifest, errors) = match content::load_and_lint(game_dir) {
        Ok(loaded) => loaded,
        Err(e) => {
            eprintln!("{} : {e}", spec.name);
            return 1;
        }
    };
    let warnings = content::warnings(&registry, &manifest);
    if json {
        let report = serde_json::json!({
            "errors": errors.iter().map(|e| serde_json::json!({
                "kind": format!("{:?}", e.kind), "file": e.file, "message": e.message,
            })).collect::<Vec<_>>(),
            "warnings": warnings.iter().map(|w| serde_json::json!({
                "kind": format!("{:?}", w.kind), "file": w.file, "message": w.message,
            })).collect::<Vec<_>>(),
        });
        println!("{report}");
    } else {
        for warning in &warnings {
            eprintln!("avertissement : {warning}");
        }
        for error in &errors {
            eprintln!("{error}");
        }
        if errors.is_empty() {
            println!(
                "{} : contenu valide ({} avertissement(s))",
                spec.name,
                warnings.len()
            );
        } else {
            eprintln!("{} : {} erreur(s) de contenu", spec.name, errors.len());
        }
    }
    i32::from(!errors.is_empty())
}

/// `<jeu> scenarios capture <scenario.ron> --dir <dossier> [--every N] [--follow <handle>]
/// [--size <LxH>] [--at <f,…>] [--legend]` (X-2b) : rend le scénario en images
/// (`scenario::runner::capture`, mêmes images que `play_scenario --capture`), avec le plugin du jeu
/// déjà installé. C'est par là que `alacod video` passe : le rendu ne dépend d'aucun script de l'engine.
fn capture(spec: &GameSpec, game_dir: &std::path::Path, args: &[String]) -> i32 {
    fn value(args: &[String], name: &str) -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    }
    let scenario_arg = args.iter().find(|a| !a.starts_with("--")).cloned();
    let Some(scenario_arg) = scenario_arg else {
        eprintln!("{USAGE}");
        return 2;
    };
    let Some(dir) = value(args, "--dir") else {
        eprintln!("scenarios capture : --dir <dossier> manque ({USAGE})");
        return 2;
    };
    let scenario_path = game_dir.join(&scenario_arg);
    let scenario_path = if scenario_path.is_file() {
        scenario_path
    } else {
        std::path::PathBuf::from(&scenario_arg)
    };
    let text = match std::fs::read_to_string(&scenario_path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("scenarios capture : {} : {e}", scenario_path.display());
            return 2;
        }
    };
    let scenario = match scenario::Scenario::from_ron(&text) {
        Ok(scenario) => scenario,
        Err(e) => {
            eprintln!("scenarios capture : {} : {e}", scenario_path.display());
            return 1;
        }
    };
    if scenario.game != spec.name {
        eprintln!(
            "scenarios capture : {} joue « {} », ce binaire est celui de « {} »",
            scenario_path.display(),
            scenario.game,
            spec.name
        );
        return 2;
    }
    let every = value(args, "--every")
        .map(|v| {
            v.parse::<u32>()
                .map_err(|_| format!("scenarios capture : --every : entier attendu, pas « {v} »"))
        })
        .transpose();
    let every = match every {
        Ok(every) => every.unwrap_or(2),
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let follow_handle = match value(args, "--follow")
        .map(|v| {
            v.parse::<usize>()
                .map_err(|_| format!("scenarios capture : --follow : entier attendu, pas « {v} »"))
        })
        .transpose()
    {
        Ok(follow) => follow,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let size = match value(args, "--size")
        .map(|v| {
            v.split_once('x')
                .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
                .filter(|(w, h)| *w > 0 && *h > 0)
                .ok_or_else(|| {
                    format!("scenarios capture : --size : <largeur>x<hauteur> attendu, pas « {v} »")
                })
        })
        .transpose()
    {
        Ok(size) => size.unwrap_or(scenario::runner::CAPTURE_SIZE),
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let at = value(args, "--at").map_or_else(Vec::new, |list| {
        list.split(',')
            .filter_map(|f| f.trim().parse::<u32>().ok())
            .collect()
    });
    let legend = args.iter().any(|a| a == "--legend");
    match scenario::runner::capture(
        &scenario,
        scenario::runner::CaptureConfig {
            dir: dir.into(),
            every,
            follow_handle,
            size,
            at,
            legend,
        },
    ) {
        AppExit::Success => 0,
        _ => 1,
    }
}
