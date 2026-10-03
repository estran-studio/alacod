use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
pub struct Opt {
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
