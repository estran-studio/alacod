use std::net::SocketAddr;

use bevy::prelude::*;
use utils::cid::generate_random_correlation_id;

use crate::{
    core::OnlineState,
    jjrs::{GggrsConnectionConfiguration, GggrsSessionConfiguration, PlayerConfig},
};

/// Resource to control debug AI visualization from startup
#[derive(Resource, Default)]
pub struct DebugAiConfig {
    /// Whether to enable AI debug visualization (flow field + enemy state) from startup
    pub enabled: bool,
}

#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(target_arch = "wasm32")]
pub mod web;

/// Parsed arguments for game configuration
#[derive(Clone, Debug)]
pub struct GameArgs {
    pub local_port: u16,
    pub number_player: usize,
    pub players: Vec<PlayerConfig>,
    pub spectators: Vec<SocketAddr>,
    pub matchbox: String,
    pub lobby: String,
    pub cid: String,
    pub debug_ai: bool,
    pub telemetry: bool,
    pub telemetry_url: String,
    pub telemetry_auth: String,
}

pub fn get_args() -> GameArgs {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use clap::Parser;
        let args = cli::Opt::parse();

        // Get the local player's display name (from --name or fallback to CID or default)
        let local_player_name = args
            .name
            .clone()
            .or_else(|| args.cid.clone())
            .unwrap_or_else(|| "Player".to_string());

        // Convert CLI player strings to PlayerConfig
        // For local players (localhost), use the --name argument
        // For remote players, use a default name with index
        let mut remote_index = 0;
        let players: Vec<PlayerConfig> = args
            .players
            .unwrap_or(vec![])
            .into_iter()
            .map(|player_str| {
                let is_local = player_str == "localhost";
                if is_local {
                    PlayerConfig {
                        name: local_player_name.clone(),
                        pubkey: "local".to_string(),
                        is_local: true,
                    }
                } else {
                    remote_index += 1;
                    PlayerConfig {
                        name: format!("Player {}", remote_index + 1),
                        pubkey: player_str, // Use the address/identifier as pubkey
                        is_local: false,
                    }
                }
            })
            .collect();

        GameArgs {
            local_port: args.local_port.unwrap_or(0),
            number_player: args.number_player.unwrap_or(0),
            players,
            spectators: args.spectators.unwrap_or(vec![]),
            matchbox: args.matchbox.unwrap_or(String::new()),
            lobby: args.lobby.unwrap_or(String::new()),
            cid: args.cid.unwrap_or(generate_random_correlation_id()),
            debug_ai: args.debug_ai,
            telemetry: args.telemetry,
            telemetry_url: args.telemetry_url,
            telemetry_auth: args.telemetry_auth,
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use web::read_canvas_data_system;
        let canvas_config = read_canvas_data_system();

        // Convert web player data to PlayerConfig
        // Use and_then to treat empty array as None (fallback to default)
        let players: Vec<PlayerConfig> = canvas_config
            .players
            .and_then(|player_data| {
                if player_data.is_empty() {
                    None
                } else {
                    Some(player_data)
                }
            })
            .map(|player_data| {
                player_data
                    .into_iter()
                    .map(|p| PlayerConfig {
                        name: p.name,
                        pubkey: p.pubkey,
                        is_local: p.is_local,
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                // Default: single local player
                vec![PlayerConfig {
                    name: "Player 1".to_string(),
                    pubkey: "local".to_string(),
                    is_local: true,
                }]
            });

        GameArgs {
            local_port: 0,
            number_player: canvas_config.number_player.unwrap_or(1),
            players,
            spectators: vec![],
            matchbox: canvas_config.matchbox.unwrap_or(String::new()),
            lobby: canvas_config.lobby.unwrap_or(String::new()),
            cid: generate_random_correlation_id(),
            debug_ai: false, // debug_ai not supported on WASM
            telemetry: canvas_config.telemetry,
            telemetry_url: canvas_config.telemetry_url,
            telemetry_auth: canvas_config.telemetry_auth,
        }
    }
}

/// Configure la partie depuis la ligne de commande (natif) ou le canvas (web).
pub struct BaseArgsPlugin;

impl Plugin for BaseArgsPlugin {
    fn build(&self, app: &mut App) {
        let args = get_args();

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(utils::logs::NativeLogPlugin(args.cid.clone()));

        app.add_plugins(telemetry::TelemetryPlugin);

        app.add_plugins(GameArgsPlugin(args));
    }
}

/// Configure la session de jeu à partir d'arguments déjà construits, sans lire la ligne
/// de commande ni installer de logs ou de télémétrie globaux (utilisé par les scénarios).
pub struct GameArgsPlugin(pub GameArgs);

impl Plugin for GameArgsPlugin {
    fn build(&self, app: &mut App) {
        let args = self.0.clone();
        app.insert_resource(DebugAiConfig { enabled: args.debug_ai });

        let mut nbr_player = args.number_player;
        if nbr_player == 0 {
            nbr_player = args.players.len();
        }

        app.insert_resource(telemetry::TelemetryConfig {
            enabled: args.telemetry,
            url: args.telemetry_url,
            auth_token: args.telemetry_auth,
        });

        app.insert_resource(if !args.matchbox.is_empty() {
            OnlineState::Online
        } else {
            OnlineState::Offline
        })
        .insert_resource(GggrsSessionConfiguration {
            cid: args.cid,
            matchbox: !args.matchbox.is_empty(),
            lobby: args.lobby,
            matchbox_url: args.matchbox,
            connection: GggrsConnectionConfiguration {
                input_delay: 5,
                max_player: nbr_player,
                desync_interval: 10,
                // UDP seulement s'il y a des joueurs distants : plusieurs joueurs locaux
                // partagent une session synctest
                socket: args.players.iter().any(|p| !p.is_local),
                udp_port: args.local_port,
            },
            players: args.players,
        });
    }
}
