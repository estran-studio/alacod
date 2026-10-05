use bevy::prelude::*;
use bevy_fixed::rng::{RngStreams, RunSeed};
use bevy_ggrs::ggrs::PlayerType;
use bevy_matchbox::{
    matchbox_socket::{ChannelConfig, RtcIceServerConfig, WebRtcSocketBuilder},
    prelude::PeerState,
    MatchboxSocket,
};
use content::manifest::GameManifest;
use content::registry::Registry;
use map::generation::config::MapGenerationConfig;
use run::Run;

#[cfg(not(target_arch = "wasm32"))]
use crate::jjrs::allumette::AllumetteConfig;
use crate::{
    character::player::jjrs::PeerConfig,
    core::{AppState, OnlineState},
    jjrs::restart::{
        restart_room, restart_seed, restart_timed_out, OnlineGames, OnlineRestart,
        RESTART_TIMEOUT_SECS,
    },
    jjrs::{
        GggrsSessionConfiguration, GggrsSessionConfigurationState, GgrsPlayer, GgrsSessionBuilding,
    },
    run_state::resolve_run_mode_with_floors,
};

// For matchbox socket connection

/// STUN Google par défaut : configuration historique du chemin `--matchbox`
/// (inchangée) et repli si l'API allumette ne renvoie aucun serveur ICE.
fn default_ice_server() -> RtcIceServerConfig {
    RtcIceServerConfig {
        urls: vec![
            "stun:stun.l.google.com:19302".to_string(),
            "stun:stun1.l.google.com:19302".to_string(),
        ],
        username: None,
        credential: None,
    }
}

pub fn start_matchbox_socket(
    mut commands: Commands,
    ggrs_config: Res<GggrsSessionConfiguration>,
    // D14 : restart en ligne → salle `{lobby}-r{n}` (voir `jjrs::restart`).
    restart: Option<ResMut<OnlineRestart>>,
    time: Res<Time<Real>>,
    // Mode allumette (natif uniquement) : ressource remplie par
    // `jjrs::allumette::start_allumette_flow`, chaîné avant ce système.
    #[cfg(not(target_arch = "wasm32"))] allumette: Option<Res<AllumetteConfig>>,
) {
    // Mode allumette : URL ws(s)://hôte/JWT et ICE reçus de l'API (`/ice-servers`),
    // au lieu de `{matchbox_url}/{lobby}` et du STUN en dur. Le builder matchbox
    // n'accepte qu'un seul `RtcIceServerConfig` : on prend la première entrée.
    // Sinon, chemin `--matchbox` historique, inchangé.
    let room = match restart {
        Some(mut restart) => {
            restart.since = Some(time.elapsed_secs());
            info!("restart en ligne : partie {}", restart.game + 1);
            restart_room(&ggrs_config.lobby, restart.game)
        }
        None => ggrs_config.lobby.clone(),
    };
    #[cfg(not(target_arch = "wasm32"))]
    let (url, ice_server) = match allumette.as_deref() {
        Some(config) => (
            config.ws_url.clone(),
            config
                .ice_servers
                .first()
                .map(|ice| RtcIceServerConfig {
                    urls: ice.urls.clone(),
                    username: ice.username.clone(),
                    credential: ice.credential.clone(),
                })
                .unwrap_or_else(default_ice_server),
        ),
        None => (
            format!("{}/{}", ggrs_config.matchbox_url, room),
            default_ice_server(),
        ),
    };
    #[cfg(target_arch = "wasm32")]
    let (url, ice_server) = (
        format!("{}/{}", ggrs_config.matchbox_url, room),
        default_ice_server(),
    );

    let socket = WebRtcSocketBuilder::new(url)
        .ice_server(ice_server)
        .add_channel(ChannelConfig::reliable())
        .build();

    commands.insert_resource(MatchboxSocket::from(socket));

    info!(
        "start p2p connection with CID={} (salle {room})",
        ggrs_config.cid
    );
}

/// Ouvre un socket matchbox sur `{matchbox_url}/{room}` (chemin `--matchbox`).
fn open_matchbox_socket(commands: &mut Commands, matchbox_url: &str, room: &str) {
    let socket = WebRtcSocketBuilder::new(format!("{matchbox_url}/{room}"))
        .ice_server(default_ice_server())
        .add_channel(ChannelConfig::reliable())
        .build();
    commands.insert_resource(MatchboxSocket::from(socket));
}

#[allow(clippy::too_many_arguments)]
pub fn wait_for_players(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    socket: Option<ResMut<MatchboxSocket>>,
    ggrs_config: Res<GggrsSessionConfiguration>,
    online_state: Res<OnlineState>,
    session_state: Res<GggrsSessionConfigurationState>,
    restart: Option<Res<OnlineRestart>>,
    time: Res<Time<Real>>,
    mut games: ResMut<OnlineGames>,
) {
    if !matches!(online_state.as_ref(), OnlineState::Online) {
        return;
    }
    let Some(mut socket) = socket else {
        return;
    };

    // D14 : personne dans la salle de restart après le délai → salle d'origine, compteur à
    // zéro (comme un « Lobby »).
    if let Some(restart) = restart.as_deref() {
        if socket.players().len() < ggrs_config.connection.max_player
            && restart_timed_out(restart.since, time.elapsed_secs())
        {
            warn!(
                "restart en ligne : pairs absents de la salle {} après {RESTART_TIMEOUT_SECS} s, retour au lobby",
                restart_room(&ggrs_config.lobby, restart.game)
            );
            commands.remove_resource::<OnlineRestart>();
            games.0 = 0;
            open_matchbox_socket(&mut commands, &ggrs_config.matchbox_url, &ggrs_config.lobby);
            return;
        }
    }

    // regularly call update_peers to update the list of connected peers
    let Ok(peer_changes) = socket.try_update_peers() else {
        warn!("socket dropped");
        return;
    };

    // Check for new connections
    for (peer, new_state) in peer_changes {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => info!("peer {peer} connected"),
            PeerState::Disconnected => info!("peer {peer} disconnected"),
        }
    }
    let players = socket.players();

    let num_players = ggrs_config.connection.max_player;

    // Log the current state of player connections
    if players.len() < num_players {
        info!(
            "Waiting for players: {}/{} connected",
            players.len(),
            num_players
        );
        return; // wait for more players
    }

    if !session_state.ready {
        info!(
            "All players connected ({}/{}), but waiting for session configuration to be ready",
            players.len(),
            num_players
        );
        return;
    }

    info!(
        "All {} players are connected and ready, transitioning to GameLoading",
        num_players
    );

    // Build GgrsSessionBuilding by matching socket players with config players
    // Socket players: Local player first, then Remote players
    // Config players: In order from frontend (may have is_local flag)
    let local_config = ggrs_config.players.iter().find(|p| p.is_local);
    let remote_configs: Vec<_> = ggrs_config.players.iter().filter(|p| !p.is_local).collect();

    let mut ggrs_players = Vec::new();
    let mut remote_idx = 0;

    for (i, player_type) in players.iter().enumerate() {
        let (name, pubkey, is_local) = match player_type {
            PlayerType::Local => {
                // Safely get local config, falling back to first player or default
                if let Some(config) = local_config.or_else(|| ggrs_config.players.first()) {
                    (config.name.clone(), config.pubkey.clone(), true)
                } else {
                    // Fallback if players array is empty
                    (format!("Player {}", i + 1), "local".to_string(), true)
                }
            }
            PlayerType::Remote(_) => {
                let config = remote_configs.get(remote_idx);
                remote_idx += 1;
                match config {
                    Some(c) => (c.name.clone(), c.pubkey.clone(), false),
                    None => (
                        format!("Player {}", i + 1),
                        format!("player_{}", i + 1),
                        false,
                    ),
                }
            }
            PlayerType::Spectator(_) => (
                format!("Spectator {}", i + 1),
                format!("spectator_{}", i + 1),
                false,
            ),
        };

        ggrs_players.push(GgrsPlayer {
            handle: i,
            is_local,
            name,
            pubkey,
        });
    }

    commands.insert_resource(GgrsSessionBuilding {
        players: ggrs_players,
    });

    app_state.set(AppState::GameLoading);
}

#[allow(clippy::too_many_arguments)]
pub fn system_after_map_loaded(
    mut commands: Commands,

    mut app_state: ResMut<NextState<AppState>>,
    mut socket: Option<ResMut<MatchboxSocket>>,
    ggrs_config: Res<GggrsSessionConfiguration>,
    online_state: Res<OnlineState>,
    map_config: Option<Res<MapGenerationConfig>>,
    session_building: Res<GgrsSessionBuilding>,
    manifest: Option<Res<GameManifest>>,
    registry: Option<Res<Registry>>,
    floors_override: Option<Res<crate::run_state::FloorsOverride>>,
    restart: Option<Res<OnlineRestart>>,
) {
    if !matches!(online_state.as_ref(), OnlineState::Online) {
        return;
    }

    let socket = socket.as_mut().unwrap();

    let channel = socket.take_channel(0).unwrap();
    let num_players = ggrs_config.connection.max_player;

    // start the GGRS session
    let mut session_builder = ggrs::SessionBuilder::<PeerConfig>::new()
        .with_num_players(num_players)
        .expect("invalid number of players")
        .with_max_prediction_window(12)
        .with_input_delay(ggrs_config.connection.input_delay);

    let players = socket.players();

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    // Dérive la graine de run à partir de la graine de carte
    let run_seed = match map_config {
        Some(config) => RunSeed(config.seed as u32),
        None => RunSeed(12345), // Graine par défaut si la map n'est pas configurée
    };
    // D14 : partie relancée en ligne → graine dérivée du numéro de partie, identique chez
    // tous les pairs (même compteur). Le restart est consommé.
    let run_seed = match restart.as_deref() {
        Some(restart) => {
            commands.remove_resource::<OnlineRestart>();
            RunSeed(restart_seed(run_seed.0, restart.game))
        }
        None => run_seed,
    };
    let rng_streams = RngStreams::new(run_seed.0);

    // État de run (T2.4, chantier F1) : voir la doc de `jjrs::local::system_after_map_loaded_local`
    // (même raisonnement). D14 : une relance en ligne recrée le socket et la session, et
    // repasse donc ici (graine dérivée ci-dessus).
    let run_players: Vec<usize> = session_building
        .players
        .iter()
        .map(|player| player.handle)
        .collect();
    // T1.8 : une séquence imposée (scénario, `alacod-sim --floors`) force le mode `Floors`.
    let mode = resolve_run_mode_with_floors(
        manifest.as_deref(),
        registry.as_deref(),
        floors_override.as_deref().map(|o| o.0.as_str()),
    );
    let run = Run::new(run_seed.0, mode, run_players, 0);

    commands.insert_resource(run_seed);
    commands.insert_resource(rng_streams);
    commands.insert_resource(run);
    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));

    app_state.set(AppState::InGame);
}
