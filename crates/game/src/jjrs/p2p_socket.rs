//! Session p2p, moitié **application** (D69 étape 2, E6) : sockets matchbox, lobby en ligne
//! (`wait_for_players`, délais de connexion et d'admission, D109/D116), observation du départ des pairs
//! pendant la synchronisation, construction de la session GGRS, fermeture du socket. Tout ce qui lit ou
//! écrit `MatchboxSocket`. La simulation (admission, appariement joueurs ↔ configuration, surveillance de
//! la synchronisation, réglages GGRS) est `jjrs::p2p`, sans socket : cette moitié lui passe des listes de pairs.
//!
//! Chemin final (E9) : `game_app/src/jjrs/p2p_socket.rs`.

use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy_ggrs::ggrs::PlayerType;
use bevy_matchbox::{
    matchbox_socket::{ChannelConfig, RtcIceServerConfig, WebRtcSocketBuilder},
    prelude::{PeerId, PeerState},
    MatchboxSocket,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::jjrs::allumette::AllumetteConfig;
use crate::{
    character::player::jjrs::PeerConfig,
    jjrs::p2p::{
        admit_players, live_players, pair_session_players, secs_from_env, session_builder,
        AdmittedPlayers, CurrentSessionPeers, PreviousSessionPeers, SessionPeerLeft,
    },
    jjrs::restart::{
        restart_room, restart_timed_out, OnlineGames, OnlineRestart, RESTART_TIMEOUT_SECS,
    },
    jjrs::signal_watch::SharedSightings,
    jjrs::{GggrsSessionConfiguration, GggrsSessionConfigurationState, GgrsSessionBuilding},
    session::{AppState, OnlineState},
};
/// Parmi les changements d'état du socket, le départ d'un pair de la session en cours.
pub fn session_peer_left(changes: &[(PeerId, PeerState)], current: &BTreeSet<PeerId>) -> bool {
    changes
        .iter()
        .any(|(peer, state)| matches!(state, PeerState::Disconnected) && current.contains(peer))
}
// For matchbox socket connection

/// D87 (mesure, natif) : `ALACOD_P2P_CHANNEL=unreliable` ouvre le canal GGRS non fiable et non ordonné
/// (`ChannelConfig::unreliable()`) au lieu du canal fiable du jeu. Réglage d'essai (rapport
/// `m2-p2p-deux-machines` : il ne supprime pas les silences) ; les deux pairs doivent avoir le même.
pub const CHANNEL_ENV: &str = "ALACOD_P2P_CHANNEL";

fn channel_config() -> ChannelConfig {
    match std::env::var(CHANNEL_ENV).as_deref() {
        Ok("unreliable") => ChannelConfig::unreliable(),
        Ok("reliable") | Err(_) => ChannelConfig::reliable(),
        Ok(other) => {
            warn!("{CHANNEL_ENV}={other:?} inconnu (reliable ou unreliable attendu), canal fiable gardé");
            ChannelConfig::reliable()
        }
    }
}

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

/// D109 : durée (horloge **murale**, hors simulation) pendant laquelle un pair vu par le signaling
/// (`jjrs::signal_watch`) peut rester sans se connecter au lobby avant une nouvelle tentative.
/// Une négociation WebRTC normale prend moins d'une seconde sur la boucle locale ; sous perte de
/// paquets (3 %, RTT 120 ms) le rapport `dettes-d105-d109` mesure la distribution des temps de
/// connexion et justifie cette valeur. Surcharge d'essai : [`CONNECT_TIMEOUT_ENV`].
pub const CONNECT_TIMEOUT_SECS: f32 = 15.0;

/// Surcharge de [`CONNECT_TIMEOUT_SECS`] (secondes, essais seulement : `p2p-connect-timeout.sh`).
pub const CONNECT_TIMEOUT_ENV: &str = "ALACOD_CONNECT_TIMEOUT_SECS";

/// Essais par entrée au lobby : le socket d'origine, puis une nouvelle tentative (même salle,
/// nouveau socket) ; au délai suivant, retour au lobby (`LobbyOnline` ré-entré : socket et flux
/// allumette repris de zéro).
pub const CONNECT_ATTEMPTS: u32 = 2;

/// Surcharge de [`CONNECT_ATTEMPTS`] (entier ≥ 1, essais seulement : `p2p-connect-timeout.sh` ;
/// `1` : le premier délai écoulé ramène directement au lobby).
pub const CONNECT_ATTEMPTS_ENV: &str = "ALACOD_CONNECT_ATTEMPTS";

/// Attente d'un pair vu mais pas connecté à partir de laquelle l'écran du lobby le dit.
pub const CONNECT_SLOW_SECS: f32 = 3.0;

/// D116 : durée (horloge murale) pendant laquelle un pair vu par le signaling, pas encore connecté, retarde
/// l'admission quand le nombre de pairs connectés suffit déjà (voir [`admit_players`]) : le temps d'une
/// négociation WebRTC normale (< 1 s sur la boucle locale), pas celui d'une connexion qui ne finit pas.
pub const ADMISSION_SETTLE_SECS: f32 = 3.0;

/// D116 : borne **cumulée** de l'attente d'admission, comptée depuis que la salle a assez de joueurs connectés.
/// [`ADMISSION_SETTLE_SECS`] repart à chaque apparition (pair vu, excédent qui revient) : un pair qui clignote
/// (fantôme qui se reconnecte, client relancé en boucle) plus vite que 3 s retiendrait l'admission sans fin.
/// Passé ce délai, les admis du moment démarrent ; un désaccord entre clients sur l'ensemble des pairs finit,
/// comme une arrivée tardive, par l'abandon de la session (`SYNC_TIMEOUT_SECS`). Sous le délai de D109 (15 s).
pub const ADMISSION_MAX_SECS: f32 = 10.0;

/// D116 : l'admission attend-elle encore ? Oui pendant [`ADMISSION_SETTLE_SECS`] après l'apparition du pair en
/// attente le plus récent (`youngest_pending`) ou de l'excédent courant (`excess_for`), **dans la limite** de
/// [`ADMISSION_MAX_SECS`] depuis que la salle a assez de joueurs (`enough_for`).
pub fn admission_waits(
    youngest_pending: Option<std::time::Duration>,
    excess_for: Option<std::time::Duration>,
    enough_for: std::time::Duration,
) -> bool {
    let recent =
        |d: Option<std::time::Duration>| d.is_some_and(|d| d.as_secs_f32() < ADMISSION_SETTLE_SECS);
    (recent(youngest_pending) || recent(excess_for))
        && enough_for.as_secs_f32() < ADMISSION_MAX_SECS
}

/// Durée (secondes, murale) pendant laquelle l'écran du lobby garde l'avis d'une tentative
/// abandonnée.
pub const CONNECT_NOTICE_SECS: f32 = 20.0;

/// Dernière tentative de connexion abandonnée, montrée à l'écran du lobby.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectNotice {
    /// Nouvelle tentative en cours (`attempt` sur `of`).
    Retrying { attempt: u32, of: u32 },
    /// Les `attempts` tentatives ont échoué : le lobby est ré-entré.
    BackToLobby { attempts: u32 },
    /// D116 : salle pleine (`peers` pairs pour `max` places, joueur local ignoré) : le lobby est ré-entré.
    RoomFull { peers: usize, max: usize },
}

/// D116 : le joueur local est ignoré par l'admission ([`admit_players`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomFull {
    pub peers: usize,
    pub max: usize,
    /// Début de l'exclusion (horloge **murale**).
    pub since: bevy::platform::time::Instant,
}

/// Ce que fait le lobby quand l'attente d'un pair vu par le signaling dépasse le délai.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectTimeoutAction {
    Retry,
    ToLobby,
}

/// Surveillance de la connexion des pairs au lobby (hors rollback, réseau seulement).
#[derive(Resource, Debug)]
pub struct LobbyConnectWatch {
    /// Pairs vus par le signaling pour le socket courant (neuf à chaque socket).
    pub sightings: SharedSightings,
    /// Rang de l'essai courant (1 = socket d'origine), 0 avant le premier socket.
    pub attempt: u32,
    /// Essais permis avant le retour au lobby ([`CONNECT_ATTEMPTS`]).
    pub max_attempts: u32,
    pub timeout_secs: f32,
    /// URL de salle à rouvrir pour une nouvelle tentative ; `None` en allumette (URL à jeton,
    /// flux HTTP à refaire : directement le retour au lobby) et sur le web.
    pub retry_url: Option<String>,
    /// Attente (s) du pair vu le plus ancien, `None` sans pair en attente.
    pub pending_secs: Option<f32>,
    pub notice: Option<(ConnectNotice, bevy::platform::time::Instant)>,
    /// L'apparition d'un pair vu par le signaling est déjà journalisée pour ce socket.
    announced: bool,
    /// D116 : depuis quand la salle compte plus de pairs connectés que de places (horloge murale), pour
    /// l'attente bornée de l'excédent avant l'admission.
    excess_since: Option<bevy::platform::time::Instant>,
    /// D116 : depuis quand la salle compte au moins `max_player` joueurs connectés (horloge murale), pour la
    /// borne cumulée [`ADMISSION_MAX_SECS`] ; remis à zéro quand il en manque un ou à chaque socket neuf.
    enough_since: Option<bevy::platform::time::Instant>,
    /// La borne [`ADMISSION_MAX_SECS`] a coupé l'attente : déjà journalisé pour ce socket.
    admission_capped_logged: bool,
    /// Le socket courant est fermé (signaling refusé ou fermé avant la partie) : déjà signalé.
    socket_lost: bool,
    /// D116 : le joueur local est **ignoré** (salle pleine, [`admit_players`]) depuis `since`.
    pub room_full: Option<RoomFull>,
}

impl Default for LobbyConnectWatch {
    fn default() -> Self {
        Self {
            sightings: SharedSightings::default(),
            attempt: 0,
            max_attempts: CONNECT_ATTEMPTS,
            timeout_secs: CONNECT_TIMEOUT_SECS,
            retry_url: None,
            pending_secs: None,
            notice: None,
            announced: false,
            excess_since: None,
            enough_since: None,
            admission_capped_logged: false,
            socket_lost: false,
            room_full: None,
        }
    }
}

impl LobbyConnectWatch {
    /// Un socket neuf s'ouvre (`attempt` : son rang) : sightings vierges, délai relu.
    fn begin(&mut self, attempt: u32, retry_url: Option<String>) {
        self.sightings = SharedSightings::default();
        self.attempt = attempt;
        self.timeout_secs = secs_from_env(CONNECT_TIMEOUT_ENV, CONNECT_TIMEOUT_SECS);
        self.max_attempts = match std::env::var(CONNECT_ATTEMPTS_ENV) {
            Err(_) => CONNECT_ATTEMPTS,
            Ok(value) => value.parse::<u32>().ok().filter(|n| *n >= 1).unwrap_or_else(|| {
                warn!("{CONNECT_ATTEMPTS_ENV}={value:?} invalide (entier >= 1 attendu), {CONNECT_ATTEMPTS} gardés");
                CONNECT_ATTEMPTS
            }),
        };
        self.retry_url = retry_url;
        self.pending_secs = None;
        self.announced = false;
        self.room_full = None;
        self.excess_since = None;
        self.enough_since = None;
        self.admission_capped_logged = false;
        self.socket_lost = false;
    }
}

/// L'attente d'un pair vu par le signaling a atteint le délai.
pub fn connect_timed_out(waited: std::time::Duration, timeout_secs: f32) -> bool {
    waited.as_secs_f32() >= timeout_secs
}

/// Nouvelle tentative tant qu'il en reste (`max_attempts`) et qu'une URL de salle est rouvrable,
/// sinon retour au lobby.
pub fn connect_timeout_action(
    attempt: u32,
    max_attempts: u32,
    can_retry: bool,
) -> ConnectTimeoutAction {
    if can_retry && attempt < max_attempts {
        ConnectTimeoutAction::Retry
    } else {
        ConnectTimeoutAction::ToLobby
    }
}

/// Ligne d'état de l'écran du lobby : avis d'une tentative abandonnée (`notice`, âge en secondes),
/// sinon salle pleine (`room_full` : pairs, places ; joueur local ignoré, D116), sinon attente lente
/// d'un pair vu par le signaling (`pending_secs`), sinon rien.
pub fn connect_status_text(
    pending_secs: Option<f32>,
    room_full: Option<(usize, usize)>,
    notice: Option<(ConnectNotice, f32)>,
) -> Option<String> {
    match notice {
        Some((ConnectNotice::Retrying { attempt, of }, age)) if age < CONNECT_NOTICE_SECS => {
            return Some(crate::ui::texts::peer_retry(attempt, of))
        }
        Some((ConnectNotice::BackToLobby { attempts }, age)) if age < CONNECT_NOTICE_SECS => {
            return Some(crate::ui::texts::peer_failed_back_to_lobby(attempts))
        }
        Some((ConnectNotice::RoomFull { peers, max }, age)) if age < CONNECT_NOTICE_SECS => {
            return Some(crate::ui::texts::room_full_back_to_lobby(peers, max))
        }
        _ => {}
    }
    if let Some((peers, max)) = room_full {
        return Some(crate::ui::texts::room_full_waiting(peers, max));
    }
    pending_secs
        .filter(|secs| *secs >= CONNECT_SLOW_SECS)
        .map(|secs| crate::ui::texts::peer_connecting(secs))
}

/// Socket matchbox sur `url`. En natif, son signaller note dans `watch.sightings` les pairs que
/// le serveur annonce (D109).
fn build_socket(
    url: String,
    ice_server: RtcIceServerConfig,
    watch: &LobbyConnectWatch,
) -> MatchboxSocket {
    let builder = WebRtcSocketBuilder::new(url)
        .ice_server(ice_server)
        .add_channel(channel_config());
    #[cfg(not(target_arch = "wasm32"))]
    let builder = builder.signaller_builder(std::sync::Arc::new(
        crate::jjrs::signal_watch::ObservedSignallerBuilder {
            sightings: watch.sightings.clone(),
        },
    ));
    #[cfg(target_arch = "wasm32")]
    let _ = watch;
    MatchboxSocket::from(builder.build())
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
    mut watch: ResMut<LobbyConnectWatch>,
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
    // S1-3 : le ticket de signaling vit 60 s ; le chargeur le rafraîchit dans `data-session` pendant le
    // chargement, relu ici à l'ouverture du socket (la ressource du démarrage en repli).
    #[cfg(target_arch = "wasm32")]
    let (url, ice_server) = (
        crate::jjrs::browser::session()
            .map(|s| s.signaling_url)
            .or_else(|| browser_session.as_ref().map(|s| s.signaling_url.clone()))
            .unwrap_or_else(|| format!("{}/{}", ggrs_config.matchbox_url, room)),
        default_ice_server(),
    );

    let retry_url = {
        #[cfg(not(target_arch = "wasm32"))]
        {
            allumette.is_none().then(|| url.clone())
        }
        #[cfg(target_arch = "wasm32")]
        {
            None::<String>
        }
    };
    watch.begin(1, retry_url);
    commands.insert_resource(build_socket(url, ice_server, &watch));

    info!(
        "start p2p connection with CID={} (salle {room})",
        ggrs_config.cid
    );
}

/// Ouvre un socket matchbox sur `{matchbox_url}/{room}` (chemin `--matchbox`).
fn open_matchbox_socket(
    commands: &mut Commands,
    watch: &mut LobbyConnectWatch,
    matchbox_url: &str,
    room: &str,
) {
    let url = format!("{matchbox_url}/{room}");
    watch.begin(1, Some(url.clone()));
    commands.insert_resource(build_socket(url, default_ice_server(), watch));
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
    mut previous: ResMut<PreviousSessionPeers>,
    mut ignored_logged: Local<BTreeSet<PeerId>>,
    mut admission_logged: Local<Option<(usize, bool)>>,
    mut watch: ResMut<LobbyConnectWatch>,
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
            open_matchbox_socket(
                &mut commands,
                &mut watch,
                &ggrs_config.matchbox_url,
                &ggrs_config.lobby,
            );
            return;
        }
    }

    // regularly call update_peers to update the list of connected peers
    let Ok(peer_changes) = socket.try_update_peers() else {
        // Signaling refusé ou fermé avant la partie (ticket de 60 s périmé, session Allumette expirée ou
        // effacée : S1-3). Sur le web, le salon du site reprend la main tout de suite au lieu de laisser
        // l'écran d'attente figé jusqu'à ce que le serveur efface la session (120 s) ; en natif, l'écran
        // reste (chemins d'essai), signalé une fois.
        if !watch.socket_lost {
            watch.socket_lost = true;
            warn!("ggrs{{f=0 lobby_socket_lost}} socket de signaling fermé avant la partie");
            #[cfg(target_arch = "wasm32")]
            if crate::jjrs::browser::session().is_some() {
                crate::jjrs::browser::alacod_return_to_lobby();
            }
        }
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
            PeerState::Connected => {
                let seen_for = watch
                    .sightings
                    .lock()
                    .ok()
                    .and_then(|s| s.waited(&peer, bevy::platform::time::Instant::now()));
                match seen_for {
                    Some(waited) => info!(
                        "peer {peer} connected (vu par le signaling depuis {:.2} s)",
                        waited.as_secs_f32()
                    ),
                    None => info!("peer {peer} connected"),
                }
            }
            PeerState::Disconnected => info!("peer {peer} disconnected"),
        }
    }
    let players = live_players(socket.players(), &previous.0);

    let num_players = ggrs_config.connection.max_player;

    // D109 : un pair vu par le signaling qui ne se connecte jamais.
    let connected: BTreeSet<PeerId> = socket.connected_peers().collect();
    let now = bevy::platform::time::Instant::now();
    let (pending, youngest) = {
        let sightings = watch.sightings.lock().ok();
        (
            sightings
                .as_ref()
                .and_then(|s| s.pending(now, &connected, &previous.0)),
            sightings
                .as_ref()
                .and_then(|s| s.youngest_pending(now, &connected, &previous.0)),
        )
    };
    // D116 : un pair connecté en trop peut être le socket d'une partie finissante qui n'a pas encore quitté la
    // salle (D86, §4.5 de `dettes-d105-d109.md`) et dont l'identifiant n'est pas encore dans
    // `PreviousSessionPeers` : l'admettre d'emblée (identifiant bas) construirait une session que personne
    // n'écoute. Tout excédent est donc attendu `ADMISSION_SETTLE_SECS` (depuis son apparition, remis à zéro
    // quand il disparaît ou à chaque socket neuf) : le temps que ce socket parte.
    if players.len() > num_players {
        watch.excess_since.get_or_insert(now);
    } else {
        watch.excess_since = None;
    }
    if players.len() >= num_players {
        watch.enough_since.get_or_insert(now);
    } else {
        watch.enough_since = None;
    }
    let enough_for = watch
        .enough_since
        .map_or(std::time::Duration::ZERO, |since| {
            now.saturating_duration_since(since)
        });
    let excess_for = watch
        .excess_since
        .map(|since| now.saturating_duration_since(since));
    // Log the current state of player connections. D116 : un pair vu mais pas encore connecté peut
    // changer les admis (identifiant plus bas) : on ne s'engage pas avant de l'avoir connecté, sinon deux
    // clients qui ne voient pas le même ensemble ne choisiraient pas les mêmes joueurs. Cette attente
    // dure au plus `ADMISSION_SETTLE_SECS` **après la dernière apparition** (le pair en attente le plus
    // récent, pas le plus ancien : un fantôme à moitié connecté, D109, ne doit pas annuler l'attente d'un
    // pair qui arrive après lui) ; le délai de D109 reste la borne des attentes à moins de `max_player`
    // joueurs, `ADMISSION_MAX_SECS` celle de l'attente d'admission (pairs qui clignotent).
    let settling = admission_waits(youngest, excess_for, enough_for);
    let capped = !settling && admission_waits(youngest, excess_for, std::time::Duration::ZERO);
    if players.len() >= num_players && capped && !watch.admission_capped_logged {
        watch.admission_capped_logged = true;
        warn!(
            "ggrs{{f=0 lobby_admission_capped peers={} max={num_players} waited_secs={:.1}}} des pairs apparaissent encore : admission sans attendre davantage",
            players.len(),
            enough_for.as_secs_f32()
        );
    }
    if players.len() < num_players || settling {
        info!(
            "Waiting for players: {}/{} connected",
            players.len(),
            num_players
        );
        watch.pending_secs = pending.map(|(_, waited)| waited.as_secs_f32());
        // Un client ignoré qui se retrouve seul (les admis sont partis) n'est plus en salle pleine : son
        // compte à rebours repart de zéro à la prochaine exclusion.
        if players.len() < num_players {
            watch.room_full = None;
            *admission_logged = None;
        }
        if pending.is_some() && !watch.announced {
            watch.announced = true;
            info!("pair(s) vu(s) par le signaling, connexion en cours (D109)");
        }
        // Le délai de D109 ne vise que l'attente d'un pair manquant : pendant l'attente d'admission de 3 s
        // (assez de pairs connectés), il ne retente ni ne ramène au lobby.
        if let Some((pending_count, waited)) = pending
            .filter(|_| players.len() < num_players)
            .filter(|(_, waited)| connect_timed_out(*waited, watch.timeout_secs))
        {
            let action = connect_timeout_action(
                watch.attempt,
                watch.max_attempts,
                watch.retry_url.is_some(),
            );
            // `ggrs{…}` sans identifiant de pair : comparable (grep) d'un client à l'autre.
            warn!(
                "ggrs{{f=0 lobby_connect_timeout attempt={}/{} action={} waited_secs={:.1} pending={pending_count} connected={}}} pair vu par le signaling, jamais connecté : {}",
                watch.attempt,
                watch.max_attempts,
                match action {
                    ConnectTimeoutAction::Retry => "retry",
                    ConnectTimeoutAction::ToLobby => "lobby",
                },
                waited.as_secs_f32(),
                players.len(),
                match action {
                    ConnectTimeoutAction::Retry => "nouvelle tentative",
                    ConnectTimeoutAction::ToLobby => "retour au lobby",
                }
            );
            let now = bevy::platform::time::Instant::now();
            match action {
                ConnectTimeoutAction::Retry => {
                    let attempt = watch.attempt + 1;
                    let url = watch.retry_url.clone().expect("retry sans URL de salle");
                    watch.begin(attempt, Some(url.clone()));
                    watch.notice = Some((
                        ConnectNotice::Retrying {
                            attempt,
                            of: watch.max_attempts,
                        },
                        now,
                    ));
                    commands.insert_resource(build_socket(url, default_ice_server(), &watch));
                }
                ConnectTimeoutAction::ToLobby => {
                    watch.notice = Some((
                        ConnectNotice::BackToLobby {
                            attempts: watch.attempt,
                        },
                        now,
                    ));
                    commands.remove_resource::<OnlineRestart>();
                    games.0 = 0;
                    app_state.set(AppState::LobbyOnline);
                }
            }
        }
        return; // wait for more players
    }
    // D116 : plus de pairs que de places (client fantôme, salon partagé, socket d'une partie finissante
    // d'un pair d'une session passée sans être déjà filtré) : les `max_player` premiers par identifiant
    // sont admis, les autres ignorés. Les admis démarrent sans attendre les pairs en trop.
    let total = players.len();
    let admission = admit_players(players, num_players);
    if admission.excess > 0 {
        if *admission_logged != Some((total, admission.local_admitted)) {
            *admission_logged = Some((total, admission.local_admitted));
            warn!(
                "ggrs{{f=0 lobby_admission peers={total} max={num_players} excess={} local={}}} {}",
                admission.excess,
                if admission.local_admitted {
                    "admitted"
                } else {
                    "ignored"
                },
                if admission.local_admitted {
                    "pairs en trop ignorés, la partie démarre avec les premiers par identifiant"
                } else {
                    "salle pleine : ce client est ignoré"
                }
            );
        }
    } else {
        *admission_logged = None;
    }
    watch.pending_secs = None;
    if !admission.local_admitted {
        let now = bevy::platform::time::Instant::now();
        let since = watch.room_full.map_or(now, |full| full.since);
        watch.room_full = Some(RoomFull {
            peers: total,
            max: num_players,
            since,
        });
        let waited = now.saturating_duration_since(since);
        if connect_timed_out(waited, watch.timeout_secs) {
            warn!(
                "ggrs{{f=0 lobby_room_full peers={total} max={num_players} action=lobby waited_secs={:.1}}} salle pleine, ce client est resté ignoré : retour au lobby",
                waited.as_secs_f32()
            );
            watch.notice = Some((
                ConnectNotice::RoomFull {
                    peers: total,
                    max: num_players,
                },
                now,
            ));
            watch.room_full = None;
            // Les admis jouent à présent : le prochain socket de ce client ne doit pas les revoir comme des
            // pairs libres (il serait admis, à son tour, à une session que personne n'écoute). Comme pour
            // une session passée (D86), leurs identifiants sont ignorés ; un pair neuf, lui, compte.
            previous
                .0
                .extend(admission.admitted.iter().filter_map(|p| match p {
                    PlayerType::Remote(id) => Some(*id),
                    _ => None,
                }));
            commands.remove_resource::<OnlineRestart>();
            games.0 = 0;
            app_state.set(AppState::LobbyOnline);
        }
        return;
    }
    watch.room_full = None;
    let players = admission.admitted;

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

    // L'appariement joueurs du socket ↔ configuration de session est de la simulation (`pair_session_players`,
    // sans socket) ; sur le web, l'hôte impose les noms (`data-participants`).
    #[cfg(target_arch = "wasm32")]
    let identity = |player_type: &PlayerType<PeerId>| -> Option<(String, String, bool)> {
        let (mapping, local_id) = browser_mapping.as_ref()?;
        let id = match player_type {
            PlayerType::Local => local_id.to_string(),
            PlayerType::Remote(id) => id.to_string(),
            _ => unreachable!(),
        };
        let member = mapping
            .iter()
            .find(|m| m.peer_id.as_deref() == Some(id.as_str()))
            .unwrap();
        Some((
            member.username.clone(),
            member.player_id.clone(),
            matches!(player_type, PlayerType::Local),
        ))
    };
    #[cfg(not(target_arch = "wasm32"))]
    let identity = |_: &PlayerType<PeerId>| None;
    let ggrs_players = pair_session_players(&players, &ggrs_config, identity);

    commands.insert_resource(GgrsSessionBuilding {
        players: ggrs_players,
    });
    commands.insert_resource(AdmittedPlayers(players));

    app_state.set(AppState::GameLoading);
}

/// `OnEnter(GameStarting)`, moitié réseau de l'ancien `system_after_map_loaded` : le canal du socket, les
/// joueurs admis (`AdmittedPlayers`, fixés par `wait_for_players`) et la session GGRS (`Session::P2P`). La moitié
/// simulation (graine, `Run`, déblocages, `AppState::InGame`) est `start_online_run`.
pub fn build_p2p_session(
    mut commands: Commands,
    mut socket: Option<ResMut<MatchboxSocket>>,
    ggrs_config: Res<GggrsSessionConfiguration>,
    online_state: Res<OnlineState>,
    mut previous: ResMut<PreviousSessionPeers>,
    mut current: ResMut<CurrentSessionPeers>,
    admitted: Res<AdmittedPlayers>,
) {
    if !matches!(online_state.as_ref(), OnlineState::Online) {
        return;
    }

    let socket = socket.as_mut().unwrap();

    let channel = socket.take_channel(0).unwrap();
    let mut session_builder = session_builder::<PeerConfig>(&ggrs_config.connection)
        .expect("invalid p2p session configuration");

    let players = admitted.0.clone();
    // D86 : les pairs de cette session ne seront plus jamais des pairs du prochain lobby.
    let remote_peers: BTreeSet<PeerId> = players
        .iter()
        .filter_map(|player| match player {
            PlayerType::Remote(id) => Some(*id),
            _ => None,
        })
        .collect();
    previous.0.extend(remote_peers.iter().copied());
    current.0 = remote_peers;

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

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
}

/// Moitié réseau de l'abandon d'une session qui ne se synchronise pas (D109) : tant que la session est en
/// `Synchronizing`, lit les changements d'état du socket et note dans [`SessionPeerLeft`] le départ d'un
/// pair de la session en cours (identifiant neuf à chaque socket : il ne reviendra pas). Mêmes gardes et même
/// frame que l'ancien `abandon_unsynchronized_session`, qui lit cette ressource (simulation, sans socket).
pub fn observe_session_peers(
    online_state: Res<OnlineState>,
    session: Option<Res<bevy_ggrs::Session<PeerConfig>>>,
    request: Option<Res<crate::run_state::RunRequest>>,
    socket: Option<ResMut<MatchboxSocket>>,
    current: Res<CurrentSessionPeers>,
    mut left: ResMut<SessionPeerLeft>,
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
        return;
    }
    if let Some(mut socket) = socket {
        if let Ok(changes) = socket.try_update_peers() {
            if session_peer_left(&changes, &current.0) {
                left.0 = true;
            }
        }
    }
}

/// `OnExit(InGame)` : ferme le socket matchbox de la partie (le pair resté n'y voit plus
/// l'autre ; le prochain `LobbyOnline` en rouvre un).
pub fn drop_matchbox_socket(mut commands: Commands, socket: Option<Res<MatchboxSocket>>) {
    if socket.is_some() {
        commands.remove_resource::<MatchboxSocket>();
    }
}

#[cfg(test)]
#[path = "p2p_socket_tests.rs"]
mod tests;
