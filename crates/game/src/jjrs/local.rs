use std::net::SocketAddr;

use bevy::prelude::*;
use bevy_fixed::rng::{RngStreams, RunSeed};
use bevy_ggrs::{ggrs::PlayerType, prelude::*};
use content::manifest::GameManifest;
use content::registry::Registry;
use ggrs::UdpNonBlockingSocket;
use map::generation::config::MapGenerationConfig;
use run::Run;

use crate::{
    character::player::jjrs::PeerConfig,
    collider::spawn_test_wall,
    core::{AppState, OnlineState},
    jjrs::{
        GggrsSessionConfiguration, GggrsSessionConfigurationState, GgrsPlayer, GgrsSessionBuilding,
    },
};

pub fn setup_ggrs_local(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    session_config: Res<GggrsSessionConfiguration>,
    online_state: Res<OnlineState>,
    session_state: Res<GggrsSessionConfigurationState>,
) {
    if !matches!(online_state.as_ref(), OnlineState::Offline) {
        return;
    }

    if !session_state.ready {
        println!("NOT READY");
        return;
    }

    let mut ggrs_player = vec![];

    for (i, player_config) in session_config.players.iter().enumerate() {
        let local = player_config.pubkey == "local" || player_config.pubkey == "localhost";
        ggrs_player.push(GgrsPlayer {
            handle: i,
            is_local: local,
            name: player_config.name.clone(),
            pubkey: player_config.pubkey.clone(),
        });
    }
    commands.insert_resource(GgrsSessionBuilding {
        players: ggrs_player,
    });

    app_state.set(AppState::GameLoading);
}

// For local connection
#[allow(clippy::too_many_arguments)]
pub fn system_after_map_loaded_local(
    mut app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,

    session_config: Res<GggrsSessionConfiguration>,

    ggrs_config: Res<GggrsSessionConfiguration>,
    online_state: Res<OnlineState>,
    map_config: Option<Res<MapGenerationConfig>>,
    session_building: Res<GgrsSessionBuilding>,
    manifest: Option<Res<GameManifest>>,
    registry: Option<Res<Registry>>,
    floors_override: Option<Res<crate::run_state::FloorsOverride>>,
    profiles: Option<Res<meta::Profiles>>,
    hub: Option<Res<crate::hub::HubSession>>,
) {
    if !matches!(online_state.as_ref(), OnlineState::Offline) {
        return;
    }

    info!("start local connection with CID={}", ggrs_config.cid);

    let mut sess_build = SessionBuilder::<PeerConfig>::new()
        .with_num_players(session_config.connection.max_player)
        .expect("invalid number of players")
        .with_desync_detection_mode(ggrs::DesyncDetection::On {
            interval: session_config.connection.desync_interval,
        })
        .with_input_delay(session_config.connection.input_delay)
        .with_check_distance(session_config.connection.check_distance);

    for (i, player_config) in session_config.players.iter().enumerate() {
        let local = player_config.pubkey == "local" || player_config.pubkey == "localhost";
        if local {
            sess_build = sess_build
                .add_player(PlayerType::Local, i)
                .expect("Failed to add player");
        } else {
            let _remote_addr: SocketAddr = player_config.pubkey.parse().unwrap();
            //sess_build = sess_build.add_player(PlayerType::Remote(remote_addr), i).expect("Failed to add player");
        }
    }

    // Start a synctest session
    let sess = if !session_config.connection.socket {
        let sess = sess_build
            .start_synctest_session()
            .expect("Failed to start synctest session");

        Session::SyncTest(sess)
    } else {
        let _socket = UdpNonBlockingSocket::bind_to_port(session_config.connection.udp_port)
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to bind udp to {}",
                    session_config.connection.udp_port
                )
            });
        panic!("");
        //let sess = sess_build.start_p2p_session(socket).expect("failed to start p2p session");

        //Session::P2P(sess)
    };

    // Dérive la graine de run à partir de la graine de carte
    let run_seed = match map_config {
        Some(config) => RunSeed(config.seed as u32),
        None => RunSeed(12345), // Graine par défaut si la map n'est pas configurée
    };
    // `ALACOD_RUN_SEED_ONLINE_GAME=<n>` (natif, enregistrement des bots pour le p2p, D85) : la graine de
    // run est celle qu'une partie **en ligne** de rang `n` aurait (`restart_seed`, §33) alors que la
    // graine de carte reste celle du hub : la run lancée par la porte du hub (n = 1) se rejoue ainsi
    // à l'identique en ligne. Absente (toujours, hors `scripts/p2p-gungeon-inputs.sh`) : inchangé.
    #[cfg(not(target_arch = "wasm32"))]
    let run_seed = match std::env::var("ALACOD_RUN_SEED_ONLINE_GAME")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
    {
        Some(game) => RunSeed(crate::jjrs::restart::restart_seed(run_seed.0, game)),
        None => run_seed,
    };
    let rng_streams = RngStreams::new(run_seed.0);

    // État de run (T2.4, chantier F1) : créée au même instant que `RunSeed`/`RngStreams`
    // (voir la doc de `run::run::Run` et `docs/conventions.md` section « Run »), y compris
    // pour une relance (`RunRequest::Restart` repasse par ce même système, voir
    // `game::run_state`) — même graine, même mode, mêmes handles à chaque fois, tant que
    // `GggrsSessionConfiguration`/`GgrsSessionBuilding` ne changent pas entre-temps (jamais
    // le cas pour un `Restart`, qui ne les touche pas).
    let players: Vec<usize> = session_building
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
    // M2-T12 : déblocages de la partie = union des profils des joueurs locaux, figés ici.
    // Seuls les `Unlock` du jeu comptent, pas les jalons du profil : une relance rejoue à l'identique.
    let run_unlocks = crate::profile::local_run_unlocks(profiles.as_deref(), &players, |id| {
        registry
            .as_deref()
            .is_some_and(|registry| registry.unlocks.contains_key(id))
    });
    let run = Run::new(run_seed.0, mode, players, 0);
    commands.insert_resource(run_unlocks);

    // Insert the GGRS session resource
    commands.insert_resource(run_seed);
    commands.insert_resource(rng_streams);
    commands.insert_resource(run);
    commands.insert_resource(sess);

    app_state.set(AppState::InGame);
}
