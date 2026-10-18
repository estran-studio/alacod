//! Session p2p, moitié **simulation** (D69 étape 2, E6) : tout ce qui se décide sans socket, à partir de la
//! liste des pairs que l'application lit (`jjrs::p2p_socket`, moitié réseau : sockets matchbox, lobby,
//! construction de la session GGRS). Politique d'admission du lobby (D116), appariement joueurs ↔
//! configuration, surveillance de la synchronisation, réglages de session GGRS.
//!
//! Aucun type de présentation ni de réseau ici : `PeerId` vient de `matchbox_protocol` (crate de données).

use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy_fixed::rng::{RngStreams, RunSeed};
use bevy_ggrs::ggrs::PlayerType;
use content::manifest::GameManifest;
use content::registry::Registry;
use map::generation::config::MapGenerationConfig;
use matchbox_protocol::PeerId;
use run::Run;

use crate::{
    character::player::jjrs::PeerConfig,
    jjrs::restart::{restart_seed, OnlineRestart},
    jjrs::{GggrsSessionConfiguration, GgrsPlayer, GgrsSessionBuilding},
    session::{AppState, OnlineState},
};

/// D86 : identifiants matchbox des pairs des sessions déjà jouées par ce processus.
///
/// Un pair obtient un identifiant neuf à chaque socket. Au retour au lobby, la salle d'origine
/// peut encore contenir le **socket de la partie finissante** de l'autre pair (il ne le ferme
/// qu'à `OnExit(InGame)`) : se connecter à cet identifiant-là construit une session GGRS que
/// personne n'écoute. Ces pairs sont donc ignorés par [`live_players`] (lobby, interface du
/// lobby, construction de la session). Rempli à la construction de chaque session
/// (`p2p_socket::build_p2p_session`) ; les identifiants ne sont jamais réutilisés, l'ensemble n'est
/// jamais purgé.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct PreviousSessionPeers(pub BTreeSet<PeerId>);

/// D109 : pairs distants de la session **en cours** (remplacés à chaque construction de session).
/// Un pair qui quitte la salle pendant que la session se synchronise ne reviendra pas (identifiant
/// neuf à chaque socket) : la session est sans issue, `abandon_unsynchronized_session` la quitte
/// sans attendre `SYNC_TIMEOUT_SECS`.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct CurrentSessionPeers(pub BTreeSet<PeerId>);

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
/// n'empêche donc plus les admis de démarrer ; l'ignoré attend une place (ligne d'état du
/// lobby), puis, au bout de `LobbyConnectWatch::timeout_secs`, revient au lobby en tenant les admis pour
/// occupés (ils rejoignent `PreviousSessionPeers`, comme les pairs d'une session passée).
///
/// Limites assumées : la décision porte sur les pairs **connectés au moment où elle est prise**, après
/// au plus `ADMISSION_SETTLE_SECS` d'attente d'un pair vu par le signaling mais pas encore connecté
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
/// quels par `build_p2p_session` (le chargement de la carte dure : un pair arrivé entre-temps
/// ne doit changer ni les handles ni la taille de la session). Vide hors ligne.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct AdmittedPlayers(pub Vec<PlayerType<PeerId>>);

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

/// Valeur en secondes (> 0) de la variable d'environnement `name`, sinon `default` (avec un
/// `warn!` si elle est invalide).
pub fn secs_from_env(name: &str, default: f32) -> f32 {
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

/// Appariement des joueurs du socket et de la configuration de session (`GggrsSessionConfiguration::players`),
/// **sans socket** : l'application passe la liste des joueurs admis (`socket.players()` filtré par
/// [`live_players`] puis [`admit_players`]). Le joueur local prend la configuration `is_local` (à défaut la première,
/// à défaut un nom de repli), les pairs distants les configurations non locales dans l'ordre ; `identity` peut imposer le
/// triplet `(nom, clé, local)` d'un joueur (le web : l'hôte fournit les participants). Le handle GGRS est le rang dans
/// `players` (l'ordre du socket).
pub fn pair_session_players(
    players: &[PlayerType<PeerId>],
    config: &GggrsSessionConfiguration,
    identity: impl Fn(&PlayerType<PeerId>) -> Option<(String, String, bool)>,
) -> Vec<GgrsPlayer> {
    let local_config = config.players.iter().find(|p| p.is_local);
    let remote_configs: Vec<_> = config.players.iter().filter(|p| !p.is_local).collect();

    let mut ggrs_players = Vec::new();
    let mut remote_idx = 0;

    for (i, player_type) in players.iter().enumerate() {
        let (name, pubkey, is_local) = match player_type {
            PlayerType::Local => {
                // Safely get local config, falling back to first player or default
                if let Some(config) = local_config.or_else(|| config.players.first()) {
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
        let (name, pubkey, is_local) = identity(player_type).unwrap_or((name, pubkey, is_local));
        ggrs_players.push(GgrsPlayer {
            handle: i,
            is_local,
            name,
            pubkey,
        });
    }
    ggrs_players
}

/// `OnEnter(GameStarting)`, moitié simulation de l'ancien `system_after_map_loaded` : graine de run, flux
/// de RNG, `Run`, déblocages en ligne, puis `AppState::InGame`. La session GGRS (`Session::P2P`) est construite par
/// `p2p_socket::build_p2p_session`, dans le même tuple `OnEnter` (les commandes des deux sont appliquées au
/// même point de synchronisation, avant l'entrée en `InGame`).
#[allow(clippy::too_many_arguments)]
pub fn start_online_run(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    online_state: Res<OnlineState>,
    map_config: Option<Res<MapGenerationConfig>>,
    session_building: Res<GgrsSessionBuilding>,
    manifest: Option<Res<GameManifest>>,
    registry: Option<Res<Registry>>,
    floors_override: Option<Res<crate::run_state::FloorsOverride>>,
    hub: Option<Res<crate::hub::HubSession>>,
    restart: Option<Res<OnlineRestart>>,
) {
    if !matches!(online_state.as_ref(), OnlineState::Online) {
        return;
    }

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

/// D109 : un pair de la session **en cours** a quitté la salle pendant qu'elle se synchronise (écrit par
/// l'application, `p2p_socket::observe_session_peers`, qui lit le socket ; lu par
/// [`abandon_unsynchronized_session`]). Remis à faux à chaque session neuve ([`reset_sync_watch`]).
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SessionPeerLeft(pub bool);

/// `OnEnter(InGame)` : une session neuve repart d'une surveillance vierge.
pub fn reset_sync_watch(mut watch: ResMut<SessionSyncWatch>, mut left: ResMut<SessionPeerLeft>) {
    watch.since = None;
    watch.timeout_secs = secs_from_env(SYNC_TIMEOUT_ENV, SYNC_TIMEOUT_SECS);
    left.0 = false;
}

/// Session p2p restée en `Synchronizing` plus de [`SessionSyncWatch::timeout_secs`], ou dont un
/// pair a quitté la salle ([`SessionPeerLeft`], D109 : sans issue, ce pair ne revient pas) : abandon et retour au
/// lobby (`RunRequest::ToLobby`, qui ferme le socket ; ses pairs deviennent des
/// [`PreviousSessionPeers`] du prochain lobby).
pub fn abandon_unsynchronized_session(
    mut commands: Commands,
    online_state: Res<OnlineState>,
    session: Option<Res<bevy_ggrs::Session<PeerConfig>>>,
    frame: Res<utils::frame::FrameCount>,
    request: Option<Res<crate::run_state::RunRequest>>,
    mut watch: ResMut<SessionSyncWatch>,
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
        watch.since = None;
        return;
    }
    if left.0 {
        left.0 = false;
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
pub fn session_builder<C: ggrs::Config>(
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
