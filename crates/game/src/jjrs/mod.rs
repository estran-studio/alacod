#[cfg(target_arch = "wasm32")]
pub mod browser;
pub mod local;
pub mod p2p;
pub mod restart;
pub mod signal_watch;
// Client HTTP allumette (D12, m0-v9) : natif uniquement (le flux challenge/login
// n'existe pas sur le web en v1, et `ureq` n'est pas compilé pour wasm).
#[cfg(not(target_arch = "wasm32"))]
pub mod allumette;

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::prelude::*;

use crate::{character::player::jjrs::PeerConfig, collider::spawn_test_wall};

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
    /// URL de base HTTP du serveur allumette (`--allumette`) ; vide = mode matchbox.
    /// Consommée par `jjrs::allumette::start_allumette_flow` (natif uniquement).
    pub allumette_url: String,
    pub lobby: String,
    pub connection: GggrsConnectionConfiguration,
    pub players: Vec<PlayerConfig>,
}

// This state is used to mark if extra external settings need to be configure
// the ggrs system will wait for this to be true before Loading the map
// if you need to have extra ui to configure your game
#[derive(Resource, Default)]
pub struct GggrsSessionConfigurationState {
    pub ready: bool,
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
    pub players: Vec<GgrsPlayer>,
}

/// La session en ligne n'est plus jouable : un pair a quitté, ou les états ont divergé
/// (desync). Affiché par `ui::disconnected` ; en headless, la partie s'arrête avec
/// [`SESSION_LOST_EXIT_CODE`] pour qu'un test p2p ne puisse pas passer pour un succès.
#[derive(Event, Message, Debug, Clone, PartialEq, Eq)]
pub struct SessionLostEvent {
    pub title: &'static str,
    pub message: String,
}

/// Code de sortie d'une partie headless dont la session est perdue (déconnexion, desync).
pub const SESSION_LOST_EXIT_CODE: u8 = 3;

/// `ALACOD_DEBUG_DESYNC_AT=<frame>` (exercice de desync, natif) : à cette frame, **ce client
/// seulement** retire un point de vie à son joueur local, hors de tout input partagé. Les
/// checksums des pairs divergent dès lors : `DesyncDetected` doit suivre sur chaque client
/// (écran « DESYNC », code de sortie 3 en headless). Sert à prouver la chaîne de détection sur
/// le vrai transport ; jamais défini en jeu normal.
static DEBUG_DESYNC_AT: std::sync::LazyLock<Option<u32>> = std::sync::LazyLock::new(|| {
    std::env::var("ALACOD_DEBUG_DESYNC_AT").ok().map(|v| {
        v.parse()
            .expect("ALACOD_DEBUG_DESYNC_AT doit être un entier")
    })
});

pub fn debug_desync_system(
    frame: Res<utils::frame::FrameCount>,
    mut local: Query<&mut combat::actors::Health, With<crate::character::player::LocalPlayer>>,
) {
    if *DEBUG_DESYNC_AT != Some(frame.frame) {
        return;
    }
    for mut health in &mut local {
        health.current -= fixed_math::FIXED_ONE;
        warn!(
            "ALACOD_DEBUG_DESYNC_AT : état local altéré à la frame {} (desync volontaire)",
            frame.frame
        );
    }
}

/// Nom des joueurs derrière une adresse de pair (`handles_by_address`), sinon l'adresse.
fn players_at<'a>(
    handles: impl IntoIterator<Item = usize>,
    players: Option<&'a GgrsSessionBuilding>,
    addr: &impl std::fmt::Debug,
) -> String {
    let names: Vec<&str> = handles
        .into_iter()
        .filter_map(|handle| {
            players?
                .players
                .iter()
                .find(|p| p.handle == handle)
                .map(|p| p.name.as_str())
        })
        .collect();
    if names.is_empty() {
        format!("{addr:?}")
    } else {
        names.join(", ")
    }
}

pub fn log_ggrs_events(
    mut session: ResMut<bevy_ggrs::Session<PeerConfig>>,
    telemetry_config: Res<telemetry::TelemetryConfig>,
    #[cfg(not(target_arch = "wasm32"))] telemetry_sender: Option<Res<telemetry::TelemetrySender>>,
    mut lost_writer: MessageWriter<SessionLostEvent>,
    mut exit: MessageWriter<AppExit>,
    session_building: Option<Res<GgrsSessionBuilding>>,
) {
    let Session::P2P(session) = session.as_mut() else {
        return;
    };
    let mut lost = Vec::new();
    let mut desync = false;
    // `events()` draine la file en gardant la session empruntée : on la vide d'abord pour
    // pouvoir interroger `handles_by_address` ensuite.
    let events: Vec<_> = session.events().collect();
    for event in events {
        info!("GGRS Event: {:?}", event);
        match event {
            GgrsEvent::Disconnected { addr } => {
                let who = players_at(
                    session.handles_by_address(addr),
                    session_building.as_deref(),
                    &addr,
                );
                error!("Player '{}' disconnected", who);
                lost.push(SessionLostEvent {
                    title: "GAME DISCONNECTED",
                    message: format!("{who} disconnected"),
                });
            }
            GgrsEvent::DesyncDetected {
                frame,
                local_checksum,
                remote_checksum,
                addr,
            } => {
                let who = players_at(
                    session.handles_by_address(addr),
                    session_building.as_deref(),
                    &addr,
                );
                error!(
                    "Desync detected on frame {} local {:032x} remote {:032x}@{}",
                    frame, local_checksum, remote_checksum, who
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

                // Une simulation qui a divergé n'est plus une partie commune : on l'arrête
                // (les deux pairs continueraient sinon chacun leur jeu, en silence).
                desync = true;
                lost.push(SessionLostEvent {
                    title: "DESYNC",
                    message: format!("Frame {frame}: état différent de {who}"),
                });
            }
            _ => (),
        }
    }
    if lost.is_empty() {
        return;
    }
    // Headless : seul le desync arrête le processus. Une déconnexion y est normale (le pair a
    // fini sa trace et quitté) et la comparaison des traces reste le juge.
    if desync && crate::core::is_headless() {
        exit.write(AppExit::from_code(SESSION_LOST_EXIT_CODE));
    }
    lost_writer.write_batch(lost);
}
