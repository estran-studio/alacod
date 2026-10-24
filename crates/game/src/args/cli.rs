use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
pub struct Opt {
    /// Reproducible map seed (local games).
    #[clap(long, allow_hyphen_values = true)]
    pub seed: Option<i32>,
    #[clap(short, long)]
    pub matchbox: Option<String>,
    /// URL de base HTTP du serveur allumette (authentification par challenge Ed25519,
    /// lobby HTTP, WebSocket de signalisation avec le JWT dans le chemin).
    /// Exclusif avec `--matchbox`.
    #[clap(short, long, conflicts_with = "matchbox")]
    pub allumette: Option<String>,
    #[clap(long)]
    pub lobby: Option<String>,
    #[clap(short, long)]
    pub number_player: Option<usize>,
    #[clap(short, long)]
    pub local_port: Option<u16>,
    #[clap(short, long, num_args = 1..)]
    pub players: Option<Vec<String>>,
    #[clap(short, long, num_args = 1..)]
    pub spectators: Option<Vec<SocketAddr>>,
    #[clap(long)]
    pub cid: Option<String>,
    /// Display name for the local player
    #[clap(long)]
    pub name: Option<String>,
    /// Bac à sable (`docs/taches/plan-bac-a-sable.md`) : ouvre l'arène du jeu avec un profil de départ
    /// (`sandbox/<profil>.ron`, `arene` sans argument), sans vagues ni étages, en mode éditeur
    /// (remote `alacod/*`, `check_distance` 0).
    #[clap(long, value_name = "PROFIL", num_args = 0..=1, default_missing_value = content::sandbox::DEFAULT_PROFILE)]
    pub sandbox: Option<String>,
    #[clap(long)]
    pub debug_ai: bool,
    #[clap(long)]
    pub telemetry: bool,
    #[clap(
        long,
        default_value = "http://localhost:5080/api/default/default/_json"
    )]
    pub telemetry_url: String,
    #[clap(long, default_value = "")]
    pub telemetry_auth: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox(args: &[&str]) -> Option<String> {
        let mut argv = vec!["jeu"];
        argv.extend_from_slice(args);
        Opt::try_parse_from(argv)
            .expect("ligne de commande")
            .sandbox
    }

    /// `--sandbox` sans profil charge `arene` (`content::sandbox::DEFAULT_PROFILE`), avec profil celui-là, et
    /// ne mange jamais l'option suivante.
    #[test]
    fn sandbox_a_un_profil_facultatif() {
        assert_eq!(sandbox(&[]), None);
        assert_eq!(sandbox(&["--sandbox"]).as_deref(), Some("arene"));
        assert_eq!(sandbox(&["--sandbox", "lourd"]).as_deref(), Some("lourd"));
        assert_eq!(
            sandbox(&["--sandbox", "--players", "localhost"]).as_deref(),
            Some("arene")
        );
        assert_eq!(
            sandbox(&["--seed", "7", "--sandbox", "lourd", "--local-port", "7000"]).as_deref(),
            Some("lourd")
        );
    }
}
