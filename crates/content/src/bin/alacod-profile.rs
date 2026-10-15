//! CLI `alacod-profile` (M2-T12, `docs/conventions.md` §51) : achète un déblocage hors
//! simulation. `alacod-profile list games/gungeon` montre les déblocages du jeu et l'état du
//! profil ; `alacod-profile unlock games/gungeon <id> [--handle 0]` débite l'essence et écrit le
//! profil (écriture atomique, `meta::save`). Dossier : `ALACOD_PROFILE_DIR` ou celui de
//! l'utilisateur (`meta::profile_dir`).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use content::load_and_lint;
use meta::{local_id, Profile, PurchaseTerms};

#[derive(Parser)]
#[command(
    name = "alacod-profile",
    about = "Profil de méta-progression d'un jeu alacod"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Liste les déblocages du jeu avec leur prix et l'état du profil.
    List {
        game_dir: PathBuf,
        #[arg(long, default_value_t = 0)]
        handle: usize,
    },
    /// Achète le déblocage `id` pour le profil du joueur `handle`.
    Unlock {
        game_dir: PathBuf,
        id: String,
        #[arg(long, default_value_t = 0)]
        handle: usize,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let (game_dir, handle) = match &cli.command {
        Command::List { game_dir, handle }
        | Command::Unlock {
            game_dir, handle, ..
        } => (game_dir, *handle),
    };
    let (registry, _, errors) = load_and_lint(game_dir).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!(
            "{} : contenu invalide ({} erreur(s))",
            game_dir.display(),
            errors.len()
        ));
    }
    let dir = meta::profile_dir().ok_or("pas de dossier de profil (ALACOD_PROFILE_DIR ?)")?;
    let id = local_id(handle);
    let mut profile = meta::load(&dir, &id)
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| Profile::new(id.clone()));
    match cli.command {
        Command::List { .. } => {
            println!("{id} : essence {}", profile.currency(meta::ESSENCE));
            for unlock in registry.unlocks.values() {
                let state = if profile.has_unlock(&unlock.id) {
                    "possédé"
                } else {
                    "à acheter"
                };
                println!(
                    "  {} — {} ({state}, prix {})",
                    unlock.id,
                    unlock.def.name,
                    unlock
                        .def
                        .cost
                        .map_or("gratuit".to_string(), |c| c.to_string())
                );
            }
            Ok(())
        }
        Command::Unlock { id: unlock, .. } => {
            let def = &registry
                .unlocks
                .get(&unlock)
                .ok_or_else(|| format!("déblocage « {unlock} » inconnu"))?
                .def;
            profile
                .purchase(
                    &unlock,
                    &PurchaseTerms {
                        cost: def.cost,
                        requires: &def.requires,
                        stat: def.stat.as_ref().map(|(s, n)| (s.as_str(), *n)),
                    },
                )
                .map_err(|e| format!("achat refusé : {e}"))?;
            meta::save(&dir, &profile).map_err(|e| e.to_string())?;
            println!(
                "{id} : « {unlock} » débloqué, essence {}",
                profile.currency(meta::ESSENCE)
            );
            Ok(())
        }
    }
}
