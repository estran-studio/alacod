//! Teste la politique de session de production avec deux pairs GGRS : inputs variables,
//! messages retardés, rollback réel et checksum volontairement divergent. Sans port ni serveur.
use super::session_builder;
use crate::jjrs::GggrsConnectionConfiguration;
use ggrs::{
    Config, DesyncDetection, GgrsError, GgrsEvent, GgrsRequest, Message, NonBlockingSocket,
    PlayerType, PredictRepeatLast, SessionState,
};
use std::sync::{Arc, Mutex};

struct TestConfig;
impl Config for TestConfig {
    type Input = u8;
    type InputPredictor = PredictRepeatLast;
    type State = (i32, u64);
    type Address = usize;
}

type Mailboxes = Arc<Mutex<[Vec<(usize, Message)>; 2]>>;
struct MemorySocket {
    peer: usize,
    mailboxes: Mailboxes,
    polls: usize,
}
impl NonBlockingSocket<usize> for MemorySocket {
    fn send_to(&mut self, msg: &Message, addr: &usize) {
        self.mailboxes.lock().unwrap()[*addr].push((self.peer, msg.clone()));
    }

    fn receive_all_messages(&mut self) -> Vec<(usize, Message)> {
        self.polls += 1;
        // Le premier poll participe à la synchronisation ; ensuite un retard borné,
        // distinct pour chaque pair, force des prédictions erronées et leur resimulation.
        if self.polls > 20 && !self.polls.is_multiple_of(self.peer + 2) {
            return Vec::new();
        }
        std::mem::take(&mut self.mailboxes.lock().unwrap()[self.peer])
    }
}

fn config() -> GggrsConnectionConfiguration {
    GggrsConnectionConfiguration {
        max_player: 2,
        input_delay: 0,
        desync_interval: 10,
        socket: false,
        udp_port: 0,
        check_distance: 2,
    }
}

#[test]
fn invalid_desync_interval_is_rejected() {
    let mut config = config();
    config.desync_interval = 0;
    assert!(session_builder::<TestConfig>(&config).is_err());
}

fn exercise_peers(corrupt_checksum: bool) -> (usize, usize) {
    let mailboxes: Mailboxes = Arc::default();
    let mut sessions: Vec<_> = (0..2)
        .map(|peer| {
            session_builder::<TestConfig>(&config())
                .unwrap()
                .add_player(PlayerType::Local, peer)
                .unwrap()
                .add_player(PlayerType::Remote(1 - peer), 1 - peer)
                .unwrap()
                .start_p2p_session(MemorySocket {
                    peer,
                    mailboxes: mailboxes.clone(),
                    polls: 0,
                })
                .unwrap()
        })
        .collect();
    for session in &sessions {
        assert_eq!(
            session.desync_detection(),
            DesyncDetection::On { interval: 10 }
        );
    }
    let mut states = [(0_i32, 0_u64); 2];
    let mut rollbacks = 0;
    let mut desyncs = 0;
    for _ in 0..600 {
        for (peer, session) in sessions.iter_mut().enumerate() {
            session.poll_remote_clients();
            for event in session.events() {
                match event {
                    GgrsEvent::DesyncDetected { .. } => desyncs += 1,
                    GgrsEvent::Disconnected { .. } => panic!("unexpected disconnect"),
                    _ => {}
                }
            }
            if session.current_state() != SessionState::Running {
                continue;
            }
            let frame = session.current_frame();
            let input = ((frame as usize + peer) % 7) as u8;
            session.add_local_input(peer, input).unwrap();
            let requests = match session.advance_frame() {
                Ok(requests) => requests,
                Err(GgrsError::PredictionThreshold) => continue,
                Err(error) => panic!("advance_frame: {error}"),
            };
            for request in requests {
                match request {
                    GgrsRequest::SaveGameState { cell, frame } => {
                        assert_eq!(states[peer].0, frame);
                        let checksum = states[peer].1 as u128
                            + u128::from(corrupt_checksum && peer == 1 && frame >= 30);
                        cell.save(frame, Some(states[peer]), Some(checksum));
                    }
                    GgrsRequest::LoadGameState { cell, .. } => {
                        rollbacks += 1;
                        states[peer] = cell.load().unwrap();
                    }
                    GgrsRequest::AdvanceFrame { inputs } => {
                        states[peer].0 += 1;
                        for (handle, (input, _)) in inputs.into_iter().enumerate() {
                            states[peer].1 += (handle as u64 + 1) * u64::from(input);
                        }
                    }
                }
            }
        }
    }
    for session in &sessions {
        assert!(session.current_frame() >= 200, "session did not progress");
    }
    assert!(rollbacks > 0, "test must exercise rollback");
    (rollbacks, desyncs)
}

#[test]
fn delayed_inputs_resimulate_without_desync() {
    let (_, desyncs) = exercise_peers(false);
    assert_eq!(desyncs, 0);
}

#[test]
fn p2p_detects_a_divergent_checksum() {
    let (_, desyncs) = exercise_peers(true);
    assert!(
        desyncs > 0,
        "a divergent checksum must reach DesyncDetected"
    );
}

fn peer(n: u8) -> bevy_matchbox::prelude::PeerId {
    serde_json::from_str(&format!("\"00000000-0000-0000-0000-0000000000{n:02}\"")).unwrap()
}

/// D86 : les pairs d'une session déjà jouée disparaissent de la liste, l'ordre des autres
/// (donc les handles GGRS) est celui du socket.
#[test]
fn live_players_ignores_previous_session_peers_and_keeps_order() {
    use super::live_players;
    use bevy_ggrs::ggrs::PlayerType as P;
    let players = vec![
        P::Remote(peer(1)),
        P::Local,
        P::Remote(peer(2)),
        P::Remote(peer(3)),
    ];
    let stale = [peer(1), peer(3)].into_iter().collect();
    assert_eq!(
        live_players(players.clone(), &stale),
        vec![P::Local, P::Remote(peer(2))]
    );
    assert_eq!(live_players(players.clone(), &Default::default()), players);
}

#[test]
fn sync_timeout_is_reached_only_after_the_delay() {
    use super::sync_timed_out;
    use std::time::Duration;
    assert!(!sync_timed_out(None, 30.0));
    assert!(!sync_timed_out(Some(Duration::from_millis(29_900)), 30.0));
    assert!(sync_timed_out(Some(Duration::from_secs(30)), 30.0));
}
