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
    jjrs::restart::{
        restart_room, restart_seed, restart_timed_out, OnlineGames, OnlineRestart,
        RESTART_TIMEOUT_SECS,
    },
    jjrs::signal_watch::SharedSightings,
    jjrs::{
        GggrsSessionConfiguration, GggrsSessionConfigurationState, GgrsPlayer, GgrsSessionBuilding,
    },
    session::{AppState, OnlineState},
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

/// D109 : pairs distants de la session **en cours** (remplacés à chaque construction de session).
/// Un pair qui quitte la salle pendant que la session se synchronise ne reviendra pas (identifiant
/// neuf à chaque socket) : la session est sans issue, `abandon_unsynchronized_session` la quitte
/// sans attendre `SYNC_TIMEOUT_SECS`.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct CurrentSessionPeers(pub BTreeSet<PeerId>);

/// Parmi les changements d'état du socket, le départ d'un pair de la session en cours.
pub fn session_peer_left(changes: &[(PeerId, PeerState)], current: &BTreeSet<PeerId>) -> bool {
    changes
        .iter()
        .any(|(peer, state)| matches!(state, PeerState::Disconnected) && current.contains(peer))
}

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

/// D116 : politique d'admission du lobby. Le socket classe les joueurs par identifiant de pair (ordre
/// identique chez tous les pairs qui voient le même ensemble) : les `max_player` premiers sont admis,
/// les suivants sont **ignorés** (ils ne jouent pas, la session ne les compte pas). Un pair en trop
/// n'empêche donc plus les admis de démarrer ; l'ignoré attend une place ([`RoomFull`], ligne d'état du
/// lobby), puis, au bout de `LobbyConnectWatch::timeout_secs`, revient au lobby en tenant les admis pour
/// occupés (ils rejoignent `PreviousSessionPeers`, comme les pairs d'une session passée).
///
/// Limites assumées : la décision porte sur les pairs **connectés au moment où elle est prise**, après
/// au plus [`ADMISSION_SETTLE_SECS`] d'attente d'un pair vu par le signaling mais pas encore connecté
/// (sans quoi deux clients qui ne voient pas le même ensemble choisiraient des admis différents). Un pair qui
/// arrive après le démarrage des deux autres voit trois pairs et peut se classer parmi les admis
/// (identifiant bas) : sa session ne se synchronise alors jamais (les autres ont déjà la leur) et
/// `abandon_unsynchronized_session` le ramène au lobby au bout de [`SYNC_TIMEOUT_SECS`]. Pas de protocole
/// d'engagement entre pairs : l'arrivée tardive reste un cas de repli, pas un cas exact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admission {
    /// Les `max_player` premiers joueurs, dans l'ordre du socket (les handles GGRS en dépendent).
    pub admitted: Vec<PlayerType<PeerId>>,
    /// Le joueur local (`PlayerType::Local`) fait partie des admis.
    pub local_admitted: bool,
    /// Pairs en trop, ignorés (admis ou non, le même nombre chez tous).
    pub excess: usize,
}

/// Applique l'admission à `players` (déjà débarrassés des pairs d'une session passée, [`live_players`]).
pub fn admit_players(mut players: Vec<PlayerType<PeerId>>, max_player: usize) -> Admission {
    let excess = players.len().saturating_sub(max_player);
    let rest = players.split_off(players.len().min(max_player));
    let local_admitted = !rest.iter().any(|p| matches!(p, PlayerType::Local));
    Admission {
        admitted: players,
        local_admitted,
        excess,
    }
}

/// D116 : joueurs admis de la session en construction, fixés par `wait_for_players` et repris tels
/// quels par [`system_after_map_loaded`] (le chargement de la carte dure : un pair arrivé entre-temps
/// ne doit changer ni les handles ni la taille de la session). Vide hors ligne.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct AdmittedPlayers(pub Vec<PlayerType<PeerId>>);
// For matchbox socket connection

/// D87 (mesure, natif) : `ALACOD_P2P_CHANNEL=unreliable` ouvre le canal GGRS non fiable et non ordonné
/// (`ChannelConfig::unreliable()`) au lieu du canal fiable du jeu. Réglage d'essai (rapport
/// `m2-p2p-deux-machines` : il ne supprime pas les silences) ; les deux pairs doivent avoir le même.
pub const CHANNEL_ENV: &str = "ALACOD_P2P_CHANNEL";

/// D87 (mesure, natif) : `ALACOD_DISCONNECT_TIMEOUT_MS=<ms>` remplace [`DISCONNECT_TIMEOUT`] pour un essai.
pub const DISCONNECT_TIMEOUT_ENV: &str = "ALACOD_DISCONNECT_TIMEOUT_MS";

/// Silence d'un pair au bout duquel GGRS le déclare déconnecté (la notification `NetworkInterrupted`, elle,
/// reste à 500 ms). D87 : le défaut de GGRS (2 s) est dépassé sous perte de paquets : le pair se tait 1,0 à 1,5 s
/// (une retransmission SCTP après `RTO_MIN` = 1 s, [INFERENCE]) puis 3,0 à 3,4 s (deux de suite) ; à 3 % de perte par sens, 4 essais sur 8 perdaient
/// la session en cours de partie avec 2 s, 1 sur 7 avec 5 s (rapport `m2-p2p-deux-machines` §5). Cette
/// comparaison n'est **pas significative** (Fisher bilatéral, p ≈ 0,28) ; la décision repose sur le
/// contrefactuel : trois essais à 5 s ont repris après 3,07 à 3,28 s de silence, que 2 s aurait déconnectés
/// (4 essais sur 7 avec le seul échec à 5 s). Le coût : un pair réellement parti est constaté au bout de 5 s au
/// lieu de 2 s (la partie attend, c'est tout).
///
/// Doit rester **inférieur** à `state_trace::CONFIRMATION_TIMEOUT` (10 s) : la trace attend au plus ce temps
/// les dernières frames d'un pair parti, qui ne sont confirmées qu'une fois sa déconnexion constatée.
pub const DISCONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

const _: () = assert!(
    DISCONNECT_TIMEOUT.as_millis() < crate::state_trace::CONFIRMATION_TIMEOUT.as_millis(),
    "le délai de déconnexion doit rester sous l'attente de confirmation de la trace"
);

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

/// [`DISCONNECT_TIMEOUT`] ou `ALACOD_DISCONNECT_TIMEOUT_MS` : une valeur invalide, nulle ou qui atteint
/// l'attente de confirmation de la trace (10 s) est ignorée avec un `warn!`.
fn disconnect_timeout() -> std::time::Duration {
    let Ok(value) = std::env::var(DISCONNECT_TIMEOUT_ENV) else {
        return DISCONNECT_TIMEOUT;
    };
    match value.parse::<u64>() {
        Ok(ms)
            if ms > 0 && u128::from(ms) < crate::state_trace::CONFIRMATION_TIMEOUT.as_millis() =>
        {
            std::time::Duration::from_millis(ms)
        }
        _ => {
            warn!(
                "{DISCONNECT_TIMEOUT_ENV}={value:?} invalide (entier de ms, 0 < n < {} attendu), {} ms gardées",
                crate::state_trace::CONFIRMATION_TIMEOUT.as_millis(),
                DISCONNECT_TIMEOUT.as_millis()
            );
            DISCONNECT_TIMEOUT
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
    }
}

/// Valeur en secondes (> 0) de la variable d'environnement `name`, sinon `default` (avec un
/// `warn!` si elle est invalide).
fn secs_from_env(name: &str, default: f32) -> f32 {
    match std::env::var(name) {
        Err(_) => default,
        Ok(value) => value
            .parse::<f32>()
            .ok()
            .filter(|secs| *secs > 0.0)
            .unwrap_or_else(|| {
                warn!("{name}={value:?} invalide (secondes > 0 attendues), {default} s gardées");
                default
            }),
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
            return Some(format!(
                "Peer connection failed: new attempt ({attempt}/{of})"
            ))
        }
        Some((ConnectNotice::BackToLobby { attempts }, age)) if age < CONNECT_NOTICE_SECS => {
            return Some(format!(
                "Peer connection failed after {attempts} attempt(s): back to the lobby"
            ))
        }
        Some((ConnectNotice::RoomFull { peers, max }, age)) if age < CONNECT_NOTICE_SECS => {
            return Some(format!(
                "Room full ({peers} peers for {max} places): back to the lobby"
            ))
        }
        _ => {}
    }
    if let Some((peers, max)) = room_full {
        return Some(format!(
            "Room full ({peers} peers for {max} places): waiting for a free place"
        ));
    }
    pending_secs
        .filter(|secs| *secs >= CONNECT_SLOW_SECS)
        .map(|secs| format!("Connecting to peer... {secs:.0} s"))
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
    #[cfg(target_arch = "wasm32")]
    let (url, ice_server) = (
        browser_session
            .as_ref()
            .map(|s| s.signaling_url.clone())
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
    let pending = watch.sightings.lock().ok().and_then(|sightings| {
        sightings.pending(
            bevy::platform::time::Instant::now(),
            &connected,
            &previous.0,
        )
    });
    // Log the current state of player connections. D116 : un pair vu mais pas encore connecté peut
    // changer les admis (identifiant plus bas) : on ne s'engage pas avant de l'avoir connecté, sinon deux
    // clients qui ne voient pas le même ensemble ne choisiraient pas les mêmes joueurs. Cette attente
    // dure au plus `ADMISSION_SETTLE_SECS` : un pair à moitié connecté qui ne finit jamais (D109) ne doit
    // pas empêcher les pairs connectés de jouer ; le délai de D109 reste la borne des attentes à
    // moins de `max_player` joueurs.
    let settling = pending.is_some_and(|(_, waited)| waited.as_secs_f32() < ADMISSION_SETTLE_SECS);
    if players.len() < num_players || settling {
        info!(
            "Waiting for players: {}/{} connected",
            players.len(),
            num_players
        );
        watch.pending_secs = pending.map(|(_, waited)| waited.as_secs_f32());
        if pending.is_some() && !watch.announced {
            watch.announced = true;
            info!("pair(s) vu(s) par le signaling, connexion en cours (D109)");
        }
        if let Some((pending_count, waited)) =
            pending.filter(|(_, waited)| connect_timed_out(*waited, watch.timeout_secs))
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
    commands.insert_resource(AdmittedPlayers(players));

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
/// réseau seulement). Horloge **murale** (`Instant`), pas `Time<Real>` : en headless, bevy avance
/// `Time<Real>` de 1/60 s par update (`TimeUpdateStrategy::ManualDuration`), donc d'un délai bien
/// plus court que les secondes qu'on lit.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct SessionSyncWatch {
    /// Instant où la session a été vue en `Synchronizing` pour la première fois.
    pub since: Option<bevy::platform::time::Instant>,
    pub timeout_secs: f32,
}

impl Default for SessionSyncWatch {
    fn default() -> Self {
        Self {
            since: None,
            timeout_secs: SYNC_TIMEOUT_SECS,
        }
    }
}

/// Le délai de synchronisation est écoulé (`waited` : depuis le premier `Synchronizing`).
pub fn sync_timed_out(waited: Option<std::time::Duration>, timeout_secs: f32) -> bool {
    waited.is_some_and(|waited| waited.as_secs_f32() >= timeout_secs)
}

/// `OnEnter(InGame)` : une session neuve repart d'une surveillance vierge.
pub fn reset_sync_watch(mut watch: ResMut<SessionSyncWatch>) {
    watch.since = None;
    watch.timeout_secs = secs_from_env(SYNC_TIMEOUT_ENV, SYNC_TIMEOUT_SECS);
}

/// Session p2p restée en `Synchronizing` plus de [`SessionSyncWatch::timeout_secs`], ou dont un
/// pair a quitté la salle (D109 : sans issue, ce pair ne revient pas) : abandon et retour au lobby
/// (`RunRequest::ToLobby`, qui ferme le socket ; ses pairs deviennent des
/// [`PreviousSessionPeers`] du prochain lobby).
#[allow(clippy::too_many_arguments)]
pub fn abandon_unsynchronized_session(
    mut commands: Commands,
    online_state: Res<OnlineState>,
    session: Option<Res<bevy_ggrs::Session<PeerConfig>>>,
    frame: Res<utils::frame::FrameCount>,
    request: Option<Res<crate::run_state::RunRequest>>,
    mut watch: ResMut<SessionSyncWatch>,
    socket: Option<ResMut<MatchboxSocket>>,
    current: Res<CurrentSessionPeers>,
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
    if let Some(mut socket) = socket {
        if let Ok(changes) = socket.try_update_peers() {
            if session_peer_left(&changes, &current.0) {
                warn!(
                    "ggrs{{f={} session_peer_left waited_secs={:.1} players={}}} pair de la session parti avant la synchronisation, abandon et retour au lobby",
                    frame.frame,
                    watch.since.map_or(0.0, |since| since.elapsed().as_secs_f32()),
                    session.num_players()
                );
                watch.since = None;
                commands.insert_resource(crate::run_state::RunRequest::ToLobby);
                return;
            }
        }
    }
    let since = *watch
        .since
        .get_or_insert_with(bevy::platform::time::Instant::now);
    let waited = since.elapsed();
    if sync_timed_out(Some(waited), watch.timeout_secs) {
        // `ggrs{…}` sans identifiant de pair : comparable (grep) d'un client à l'autre.
        warn!(
            "ggrs{{f={} session_sync_timeout waited_secs={:.1} players={}}} session non synchronisée, abandon et retour au lobby",
            frame.frame,
            waited.as_secs_f32(),
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
        .with_fps(crate::session::SIM_FPS as usize)?
        .with_max_prediction_window(MAX_PREDICTION_WINDOW)
        .with_disconnect_timeout(disconnect_timeout())
        .with_desync_detection_mode(ggrs::DesyncDetection::On {
            interval: config.desync_interval,
        })
        .with_input_delay(config.input_delay))
}

#[cfg(test)]
#[path = "p2p_tests.rs"]
mod tests;
