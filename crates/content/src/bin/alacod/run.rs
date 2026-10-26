//! `alacod run` : lance le jeu du dossier courant (`cargo run` dans le dossier du jeu), avec ou sans bac à
//! sable, avec ou sans fenêtre. Aucune dépendance au Makefile ni aux scripts de l'engine
//! (`docs/conventions.md` §89).

use std::path::Path;
use std::process::Command;

/// Réglages de `cargo` pour la compilation sans rendu : le jeu est son propre workspace, il ne porte pas le
/// profil `headless` de l'engine (`Cargo.toml` racine) ; on le redéclare, mêmes réglages.
pub const HEADLESS_PROFILE_CONFIG: [&str; 8] = [
    "--profile",
    "headless",
    "--config",
    "profile.headless.inherits=\"dev\"",
    "--config",
    "profile.headless.opt-level=1",
    "--config",
    "profile.headless.debug=\"line-tables-only\"",
];
const HEADLESS_DEPS_CONFIG: [&str; 2] = ["--config", "profile.headless.package.\"*\".opt-level=3"];

#[derive(Debug, Default)]
pub struct RunOptions {
    /// `--sandbox [profil]` : le profil (`arene` si non précisé, valeur déjà résolue par clap).
    pub sandbox: Option<String>,
    pub headless: bool,
    pub seed: Option<i32>,
    /// Joueurs locaux (une session synctest) ; absent : un seul.
    pub players: Option<u32>,
    /// Arguments passés tels quels au jeu (après `--`).
    pub passthrough: Vec<String>,
}

/// Le jeu déclare-t-il la feature `native` (rechargement à chaud, UI de debug) ? Les jeux créés par
/// `alacod new` oui ; la demander à un jeu qui ne la déclare pas ferait échouer `cargo`.
fn has_native_feature(game_dir: &Path) -> bool {
    std::fs::read_to_string(game_dir.join("Cargo.toml")).is_ok_and(|text| {
        let mut in_features = false;
        text.lines().any(|line| {
            let line = line.trim();
            if line.starts_with('[') {
                in_features = line == "[features]";
                false
            } else {
                in_features && line.starts_with("native") && line[6..].trim_start().starts_with('=')
            }
        })
    })
}

/// Arguments de `cargo` (de `run` jusqu'à `--` compris, puis ceux du jeu).
pub fn cargo_args(game_dir: &Path, options: &RunOptions) -> Vec<String> {
    let mut args: Vec<String> = vec!["run".into()];
    if options.headless {
        args.push("--no-default-features".into());
    }
    if has_native_feature(game_dir) {
        args.extend(["--features".into(), "native".into()]);
    }
    if options.headless {
        args.extend(HEADLESS_PROFILE_CONFIG.iter().map(|s| s.to_string()));
        args.extend(HEADLESS_DEPS_CONFIG.iter().map(|s| s.to_string()));
    }
    args.push("--".into());
    args.extend(game_args(options));
    args
}

/// Arguments du jeu (`args::cli::Opt`) : `--sandbox`, `--seed`, `--players localhost…`, puis `--` du jeu.
pub fn game_args(options: &RunOptions) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(profile) = &options.sandbox {
        args.extend(["--sandbox".to_string(), profile.clone()]);
    }
    if let Some(seed) = options.seed {
        args.push(format!("--seed={seed}"));
    }
    if let Some(players) = options.players {
        args.push("--players".into());
        args.extend(std::iter::repeat_n("localhost".to_string(), players as usize));
    }
    args.extend(options.passthrough.iter().cloned());
    args
}

/// Lance le jeu au premier plan ; rend son code de sortie.
pub fn run(game_dir: &Path, options: &RunOptions) -> Result<i32, String> {
    if options.players == Some(0) {
        return Err("--players : au moins 1 joueur".into());
    }
    let mut cmd = Command::new("cargo");
    cmd.current_dir(game_dir).args(cargo_args(game_dir, options));
    if options.headless {
        cmd.env("ALACOD_HEADLESS", "1");
    }
    let status = cmd.status().map_err(|e| format!("cargo : {e}"))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_arguments_follow_the_cli_of_the_game() {
        let options = RunOptions {
            sandbox: Some("lourd".into()),
            seed: Some(-3),
            players: Some(2),
            passthrough: vec!["--debug-ai".into()],
            ..Default::default()
        };
        assert_eq!(
            game_args(&options),
            ["--sandbox", "lourd", "--seed=-3", "--players", "localhost", "localhost", "--debug-ai"]
        );
        assert!(game_args(&RunOptions::default()).is_empty());
    }

    #[test]
    fn headless_builds_without_render_features_and_uses_the_headless_profile() {
        let dir = std::env::temp_dir().join(format!("alacod-run-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[features]\ndefault = []\nnative = [\"x\"]\n").unwrap();
        let args = cargo_args(&dir, &RunOptions { headless: true, ..Default::default() });
        assert_eq!(&args[..4], ["run", "--no-default-features", "--features", "native"]);
        assert!(args.contains(&"headless".to_string()));
        assert_eq!(args.last().map(String::as_str), Some("--"));
        std::fs::write(dir.join("Cargo.toml"), "[features]\ndefault = []\n").unwrap();
        assert_eq!(cargo_args(&dir, &RunOptions::default()), ["run", "--"]);
        let _ = std::fs::remove_dir_all(dir);
    }
}
