//! CLI `alacod` : `alacod lint <dossier-du-jeu>` valide le contenu d'un jeu (manifeste,
//! registre, références, plages, kinds) sans lancer le moteur (`docs/plan-engine.md` §4.2,
//! carte T1.5). Code de sortie 1 et messages sur stderr en cas d'erreur, 0 sinon.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use content::load_and_lint;

#[derive(Parser)]
#[command(name = "alacod", about = "Outils de contenu pour l'engine alacod")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Valide le contenu d'un jeu : `alacod lint games/zombies`.
    Lint {
        /// Dossier du jeu (contient `assets/game.ron`), ex. `games/zombies`.
        game_dir: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Lint { game_dir } => lint(&game_dir),
    }
}

fn lint(game_dir: &PathBuf) -> ExitCode {
    let (registry, _manifest, errors) = match load_and_lint(game_dir) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    if errors.is_empty() {
        println!(
            "{} : aucune erreur ({} personnages, {} armes, {} armes de corps à corps, {} vagues, {} cartes)",
            game_dir.display(),
            registry.characters.len(),
            registry.weapons.len(),
            registry.melee_weapons.len(),
            registry.waves.len(),
            registry.maps.len(),
        );
        return ExitCode::SUCCESS;
    }

    for error in &errors {
        eprintln!("{error}");
    }
    eprintln!(
        "{} : {} erreur(s) de contenu",
        game_dir.display(),
        errors.len()
    );
    ExitCode::FAILURE
}
