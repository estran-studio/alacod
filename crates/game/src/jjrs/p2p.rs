use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy_fixed::rng::{RngStreams, RunSeed};
use bevy_ggrs::ggrs::PlayerType;
use bevy_matchbox::{
    matchbox_socket::{ChannelConfig, RtcIceServerConfig, WebRtcSocketBuilder},
    prelude::{PeerId, PeerState},
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
};

/// D86 : identifiants matchbox des pairs des sessions déjà jouées par ce processus.
///
/// Un pair obtient un identifiant neuf à chaque socket. Au retour au lobby, la salle d'origine
/// peut encore contenir le **socket de la partie finissante** de l'autre pair (il ne le ferme
/// qu'à `OnExit(InGame)`) : se connecter à cet identifiant-là construit une session GGRS que
/// personne n'écoute. Ces pairs sont donc ignorés par [`live_players`] (lobby, interface du
/// lobby, construction de la session). Rempli à la construction de chaque session
/// ([`system_after_map_loaded`]) ; les identifiants ne sont jamais réutilisés, l'ensemble n'est
/// jamais purgé.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct PreviousSessionPeers(pub BTreeSet<PeerId>);

/// Joueurs du socket sans les pairs d'une session déjà jouée. L'ordre (identifiants triés,
/// identique chez tous les pairs) est conservé : les handles GGRS en dépendent.
pub fn live_players(
    players: Vec<PlayerType<PeerId>>,
    stale: &BTreeSet<PeerId>,
) -> Vec<PlayerType<PeerId>> {
    players
        .into_iter()
        .filter(|player| !matches!(player, PlayerType::Remote(id) if stale.contains(id)))
        .collect()
}
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
    #[cfg(target_arch = "wasm32")] browser_session: Option<
        Res<crate::jjrs::browser::BrowserSession>,
    >,
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
        browser_session
            .as_ref()
            .map(|s| s.signaling_url.clone())
            .unwrap_or_else(|| format!("{}/{}", ggrs_config.matchbox_url, room)),
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
    previous: Res<PreviousSessionPeers>,
    mut ignored_logged: Local<BTreeSet<PeerId>>,
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
        if live_players(socket.players(), &previous.0).len() < ggrs_config.connection.max_player
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
            PeerState::Connected if previous.0.contains(&peer) => {
                // D86 : l'ancien socket de l'autre pair, pas lui. Pas de `ggrs{…}` : l'identifiant
                // diffère d'un client à l'autre, la ligne ne se compare pas.
                if ignored_logged.insert(peer) {
                    info!("peer {peer} d'une session précédente ignoré (D86)");
                }
            }
            PeerState::Connected => info!("peer {peer} connected"),
            PeerState::Disconnected => info!("peer {peer} disconnected"),
        }
    }
    let players = live_players(socket.players(), &previous.0);

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

    #[cfg(target_arch = "wasm32")]
    let browser_mapping = if crate::jjrs::browser::session().is_some() {
        let Some(mapping) = crate::jjrs::browser::participants() else {
            return;
        };
        let Some(local_id) = socket.id() else {
            return;
        };
        if players.iter().any(|p| {
            let id = match p {
                PlayerType::Local => local_id.to_string(),
                PlayerType::Remote(id) => id.to_string(),
                _ => return true,
            };
            !mapping
                .iter()
                .any(|m| m.peer_id.as_deref() == Some(id.as_str()))
        }) {
            return;
        }
        Some((mapping, local_id))
    } else {
        None
    };

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

        #[cfg(target_arch = "wasm32")]
        let (name, pubkey, is_local) = if let Some((mapping, local_id)) = &browser_mapping {
            let id = match player_type {
                PlayerType::Local => local_id.to_string(),
                PlayerType::Remote(id) => id.to_string(),
                _ => unreachable!(),
            };
            let member = mapping
                .iter()
                .find(|m| m.peer_id.as_deref() == Some(id.as_str()))
                .unwrap();
            (
                member.username.clone(),
                member.player_id.clone(),
                matches!(player_type, PlayerType::Local),
            )
        } else {
            (name, pubkey, is_local)
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
    hub: Option<Res<crate::hub::HubSession>>,
    restart: Option<Res<OnlineRestart>>,
    mut previous: ResMut<PreviousSessionPeers>,
) {
    if !matches!(online_state.as_ref(), OnlineState::Online) {
        return;
    }

    let socket = socket.as_mut().unwrap();

    let channel = socket.take_channel(0).unwrap();
    let mut session_builder = session_builder::<PeerConfig>(&ggrs_config.connection)
        .expect("invalid p2p session configuration");

    let players = live_players(socket.players(), &previous.0);
    // D86 : les pairs de cette session ne seront plus jamais des pairs du prochain lobby.
    previous
        .0
        .extend(players.iter().filter_map(|player| match player {
            PlayerType::Remote(id) => Some(*id),
            _ => None,
        }));

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    #[cfg(target_arch = "wasm32")]
    let ggrs_session = if crate::jjrs::browser::session().is_some() {
        session_builder.start_p2p_session(crate::jjrs::browser::BrowserChannel::new(channel))
    } else {
        session_builder.start_p2p_session(channel)
    }
    .expect("failed to start session");

    #[cfg(not(target_arch = "wasm32"))]
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
    let mode = crate::hub::resolve_session_mode(
        manifest.as_deref(),
        registry.as_deref(),
        floors_override.as_deref().map(|o| o.0.as_str()),
        hub.as_deref(),
    );
    let run = Run::new(run_seed.0, mode, run_players, 0);
    // M2-T12 : en ligne, les déblocages sont ceux de la configuration de session (l'hôte), jamais
    // le profil local : deux pairs aux profils différents jouent le même ensemble.
    commands.insert_resource(crate::profile::online_run_unlocks(
        std::env::var(crate::profile::RUN_UNLOCKS_ENV)
            .ok()
            .as_deref(),
    ));

    commands.insert_resource(run_seed);
    commands.insert_resource(rng_streams);
    commands.insert_resource(run);
    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));

    app_state.set(AppState::InGame);
}

/// D86 : durée (temps réel) pendant laquelle une session p2p peut rester en `Synchronizing`
/// avant d'être abandonnée pour un retour au lobby. L'autre pair peut légitimement finir de
/// charger sa carte pendant ce temps ; au-delà, personne ne répond (session construite sur un
/// identifiant qui n'écoute pas, pair parti) et la partie resterait figée pour toujours.
pub const SYNC_TIMEOUT_SECS: f32 = 30.0;

/// Surcharge de [`SYNC_TIMEOUT_SECS`] (secondes, essais seulement : `p2p-lobby.sh`).
pub const SYNC_TIMEOUT_ENV: &str = "ALACOD_SYNC_TIMEOUT_SECS";

/// Surveillance de la synchronisation de la session en cours (hors rollback, présentation du
/// réseau seulement).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct SessionSyncWatch {
    /// Temps réel où la session a été vue en `Synchronizing` pour la première fois.
    pub since: Option<f32>,
    pub timeout: f32,
}

impl Default for SessionSyncWatch {
    fn default() -> Self {
        Self {
            since: None,
            timeout: SYNC_TIMEOUT_SECS,
        }
    }
}

/// Le délai de synchronisation est écoulé.
pub fn sync_timed_out(since: Option<f32>, now: f32, timeout: f32) -> bool {
    since.is_some_and(|since| now - since >= timeout)
}

/// `OnEnter(InGame)` : une session neuve repart d'une surveillance vierge.
pub fn reset_sync_watch(mut watch: ResMut<SessionSyncWatch>) {
    watch.since = None;
    watch.timeout = std::env::var(SYNC_TIMEOUT_ENV)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(SYNC_TIMEOUT_SECS);
}

/// Session p2p restée en `Synchronizing` plus de [`SessionSyncWatch::timeout`] : abandon et
/// retour au lobby (`RunRequest::ToLobby`, qui ferme le socket ; ses pairs deviennent des
/// [`PreviousSessionPeers`] du prochain lobby).
pub fn abandon_unsynchronized_session(
    mut commands: Commands,
    online_state: Res<OnlineState>,
    session: Option<Res<bevy_ggrs::Session<PeerConfig>>>,
    time: Res<Time<Real>>,
    frame: Res<utils::frame::FrameCount>,
    request: Option<Res<crate::run_state::RunRequest>>,
    mut watch: ResMut<SessionSyncWatch>,
) {
    if !matches!(*online_state, OnlineState::Online) || request.is_some() {
        return;
    }
    let Some(session) = session else {
        return;
    };
    let bevy_ggrs::Session::P2P(session) = session.as_ref() else {
        return;
    };
    if session.current_state() != bevy_ggrs::ggrs::SessionState::Synchronizing {
        watch.since = None;
        return;
    }
    let now = time.elapsed_secs();
    let since = *watch.since.get_or_insert(now);
    if sync_timed_out(Some(since), now, watch.timeout) {
        // `ggrs{…}` sans identifiant de pair : comparable (grep) d'un client à l'autre.
        warn!(
            "ggrs{{f={} session_sync_timeout waited_secs={:.0} players={}}} session non synchronisée, abandon et retour au lobby",
            frame.frame,
            now - since,
            session.num_players()
        );
        watch.since = None;
        commands.insert_resource(crate::run_state::RunRequest::ToLobby);
    }
}

/// Frames que GGRS prédit au plus avant d'attendre les inputs distants : le pire rollback
/// restaure une frame puis en resimule autant (métrique `rollback_ms_worst` des scénarios).
pub const MAX_PREDICTION_WINDOW: usize = 12;

/// Configuration commune aux chemins natif et navigateur. GGRS désactive la détection
/// des desyncs par défaut : enregistrer des checksums ne suffit pas à les échanger.
/// Générique pour exercer cette politique avec deux pairs en mémoire dans les tests.
fn session_builder<C: ggrs::Config>(
    config: &crate::jjrs::GggrsConnectionConfiguration,
) -> Result<ggrs::SessionBuilder<C>, ggrs::GgrsError> {
    // Un intervalle nul est invalide pour GGRS. On conserve cette erreur explicite plutôt
    // que de démarrer une session sans surveillance (ou de modifier silencieusement l'intervalle).
    if config.desync_interval == 0 {
        return Err(ggrs::GgrsError::InvalidRequest {
            info: "desync_interval must be greater than zero".into(),
        });
    }
    Ok(ggrs::SessionBuilder::<C>::new()
        .with_num_players(config.max_player)?
        .with_fps(crate::core::SIM_FPS as usize)?
        .with_max_prediction_window(MAX_PREDICTION_WINDOW)
        .with_desync_detection_mode(ggrs::DesyncDetection::On {
            interval: config.desync_interval,
        })
        .with_input_delay(config.input_delay))
}

#[cfg(test)]
#[path = "p2p_tests.rs"]
mod tests;
