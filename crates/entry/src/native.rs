//! Modes d'outillage du binaire d'un jeu (natif) : `lint` et `scenarios`, choisis par le premier
//! argument. Une partie ne prend jamais d'argument positionnel (`--players`, `--sandbox`… sont des
//! options) : aucun conflit avec la ligne de commande de la partie.

use bevy::prelude::AppExit;

use crate::{GameSpec, Source};

const USAGE: &str = "usage : <jeu> lint [--json] | <jeu> scenarios gen [--play] [--bless] [--only id] [--list] [--no-trace] [--out dossier] [--json] | <jeu> scenarios play [chemin…] [--bless] [--json]";

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
