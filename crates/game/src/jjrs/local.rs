

use std::{net::SocketAddr};

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
    }, collider::{spawn_test_wall, CollisionSettings}, core::{AppState, OnlineState}, global_asset::GlobalAsset, jjrs::{GggrsSessionConfiguration, GggrsSessionConfigurationState, GgrsPlayer, GgrsSessionBuilding}, weapons::WeaponsConfig
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
pub fn system_after_map_loaded_local(
    mut app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,

    session_config: Res<GggrsSessionConfiguration>,

    ggrs_config: Res<GggrsSessionConfiguration>,
    online_state: Res<OnlineState>,
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
        .with_input_delay(session_config.connection.input_delay);


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

    // Insert the GGRS session resource
    commands.insert_resource(RollbackRng::new(12345));
    commands.insert_resource(sess);

    app_state.set(AppState::InGame);
}