pub mod p2p;
pub mod local;


use std::{default, net::SocketAddr};

use animation::SpriteSheetConfig;
use bevy::{color::palettes::{css::TURQUOISE, tailwind::{ORANGE_300, PURPLE_300}}, prelude::*};
use bevy_fixed::{fixed_math, rng::RollbackRng};
use bevy_ggrs::{ggrs::PlayerType, prelude::*};
use bevy_matchbox::{prelude::PeerState, MatchboxSocket};
use ggrs::UdpNonBlockingSocket;
use map::game::entity::map::enemy_spawn::EnemySpawnerComponent;
use utils::net_id::GgrsNetIdFactory;

use crate::{
    character::{
        config::CharacterConfig,
        enemy::spawning::EnemySpawnerState,
        player::{create::create_player, jjrs::PeerConfig},
    },
    collider::{spawn_test_wall, CollisionSettings},
    core::AppState,
    global_asset::GlobalAsset,
    weapons::WeaponsConfig,
};

// Shared configuration between the client for the ggrs configuration
// to apply to their game
pub struct GggrsConnectionConfiguration {
    pub max_player: usize,
    pub input_delay: usize,
    pub desync_interval: u32,
    pub socket: bool,
    pub udp_port: u16,
    pub check_distance: usize,
}

/// Player configuration data from frontend
#[derive(Clone, Debug)]
pub struct PlayerConfig {
    pub name: String,
    pub pubkey: String,
    pub is_local: bool,
}

// Shared configuration between the client for the matchbox + ggrs configuration
#[derive(Resource)]
pub struct GggrsSessionConfiguration {
    pub cid: String,
    pub matchbox: bool,
    pub matchbox_url: String,
    pub lobby: String,
    pub connection: GggrsConnectionConfiguration,
    pub players: Vec<PlayerConfig>,
}

// This state is used to mark if extra external settings need to be configure
// the ggrs system will wait for this to be true before Loading the map
// if you need to have extra ui to configure your game
#[derive(Resource, Default)]
pub struct GggrsSessionConfigurationState {
    pub ready: bool
}

impl GggrsSessionConfigurationState {
    pub fn ready() -> Self {
        Self { ready: true }
    }
}


pub struct GgrsPlayer {
    pub handle: usize,
    pub is_local: bool,
    pub name: String,
    pub pubkey: String,
}

// Resource to keep the information that will be used to generate the P2PSession
// after all player have joined and the game configuration is aggreed on
#[derive(Resource)]
pub struct GgrsSessionBuilding {
    pub players: Vec<GgrsPlayer>
}

#[derive(Event, Message)]
pub struct GameDisconnectedEvent(pub String);

pub fn log_ggrs_events(
    mut session: ResMut<bevy_ggrs::Session<PeerConfig>>,
    telemetry_config: Res<telemetry::TelemetryConfig>,
    #[cfg(not(target_arch = "wasm32"))] telemetry_sender: Option<Res<telemetry::TelemetrySender>>,
    mut disconnect_writer: MessageWriter<GameDisconnectedEvent>,
    session_building: Option<Res<GgrsSessionBuilding>>,
) {
    if let Session::P2P(session) = session.as_mut() {
        for event in session.events() {
            info!("GGRS Event: {:?}", event);
            match event {
                GgrsEvent::Disconnected { addr } => {
                    // Try to find the remote player's name from the session building resource
                    let player_name = session_building
                        .as_ref()
                        .and_then(|sb| {
                            sb.players
                                .iter()
                                .find(|p| !p.is_local)
                                .map(|p| p.name.clone())
                        })
                        .unwrap_or_else(|| format!("{:?}", addr));

                    error!("Player '{}' disconnected", player_name);
                    disconnect_writer.write(GameDisconnectedEvent(format!("{} disconnected", player_name)));
                }
                GgrsEvent::DesyncDetected {
                    frame,
                    local_checksum,
                    remote_checksum,
                    addr,
                } => {
                    error!(
                        "Desync detected on frame {} local {} remote {}@{:?}",
                        frame, local_checksum, remote_checksum, addr
                    );

                    let event = telemetry::TelemetryEvent {
                        level: "DESYNC".to_string(),
                        message: "Desync detected between local and remote".to_string(),
                        frame: Some(frame.try_into().unwrap_or(0)),
                        checksum_local: Some(local_checksum),
                        checksum_remote: Some(remote_checksum),
                        extra: Some(format!("{:?}", addr)),
                        timestamp: chrono::Utc::now().timestamp_micros(),
                    };

                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(sender) = telemetry_sender.as_ref() {
                        telemetry::send_event(sender, event);
                    }

                    #[cfg(target_arch = "wasm32")]
                    telemetry::send_event(&telemetry_config, event);
                }
                _ => (),
            }
        }
    }
}
