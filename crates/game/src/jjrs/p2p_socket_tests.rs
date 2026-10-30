//! Tests de la moitié application de la session p2p (`p2p_socket`) : délais de connexion du lobby (D109),
//! ligne d'état, départ d'un pair de la session. Sans socket ni serveur.

fn peer(n: u8) -> bevy_matchbox::prelude::PeerId {
    serde_json::from_str(&format!("\"00000000-0000-0000-0000-0000000000{n:02}\"")).unwrap()
}
/// D109 : une nouvelle tentative, puis le retour au lobby ; sans URL rouvrable (allumette),
/// directement le retour au lobby.
#[test]
fn connect_timeout_retries_once_then_returns_to_lobby() {
    use super::{connect_timed_out, connect_timeout_action, ConnectTimeoutAction::*};
    use std::time::Duration;
    assert_eq!(connect_timeout_action(1, 2, true), Retry);
    assert_eq!(connect_timeout_action(2, 2, true), ToLobby);
    assert_eq!(connect_timeout_action(1, 1, true), ToLobby);
    assert_eq!(connect_timeout_action(1, 2, false), ToLobby);
    assert!(!connect_timed_out(Duration::from_millis(14_900), 15.0));
    assert!(connect_timed_out(Duration::from_secs(15), 15.0));
}

/// D109/D116 : la ligne d'état du lobby dit l'abandon d'abord, puis la salle pleine, puis l'attente
/// lente, sinon rien.
#[test]
fn lobby_status_line_reports_slow_peers_and_abandoned_attempts() {
    use super::{connect_status_text as text, ConnectNotice, CONNECT_NOTICE_SECS};
    assert_eq!(text(None, None, None), None);
    assert_eq!(
        text(Some(2.9), None, None),
        None,
        "attente normale : rien à dire"
    );
    assert_eq!(
        text(Some(7.4), None, None).as_deref(),
        Some("Connexion au pair… 7 s")
    );
    let retry = Some((ConnectNotice::Retrying { attempt: 2, of: 2 }, 1.0));
    assert_eq!(
        text(Some(7.4), None, retry).as_deref(),
        Some("Connexion au pair échouée : nouvelle tentative (2/2)")
    );
    let back = Some((ConnectNotice::BackToLobby { attempts: 2 }, 1.0));
    assert_eq!(
        text(None, None, back).as_deref(),
        Some("Connexion au pair échouée après 2 tentatives : retour au salon")
    );
    // L'avis expire : retour à l'attente.
    let stale = Some((
        ConnectNotice::BackToLobby { attempts: 2 },
        CONNECT_NOTICE_SECS,
    ));
    assert_eq!(text(None, None, stale), None);
    assert_eq!(
        text(Some(7.4), Some((3, 2)), None).as_deref(),
        Some("Salle pleine (3 pairs pour 2 places) : en attente d'une place")
    );
    let full = Some((ConnectNotice::RoomFull { peers: 3, max: 2 }, 1.0));
    assert_eq!(
        text(None, Some((3, 2)), full).as_deref(),
        Some("Salle pleine (3 pairs pour 2 places) : retour au salon")
    );
}

/// D109 : seul le départ d'un pair de la session **en cours** rend la synchronisation sans issue ;
/// l'arrivée d'un pair ou le départ d'un pair d'une session passée n'y change rien.
#[test]
fn only_a_departing_current_session_peer_abandons_the_session() {
    use super::session_peer_left;
    use bevy_matchbox::prelude::PeerState::{Connected, Disconnected};
    use std::collections::BTreeSet;
    let current = BTreeSet::from([peer(2)]);
    assert!(session_peer_left(&[(peer(2), Disconnected)], &current));
    assert!(session_peer_left(
        &[(peer(1), Connected), (peer(2), Disconnected)],
        &current
    ));
    assert!(!session_peer_left(&[(peer(2), Connected)], &current));
    assert!(!session_peer_left(&[(peer(1), Disconnected)], &current));
    assert!(!session_peer_left(&[], &current));
}

/// D116 : l'attente d'admission suit la dernière apparition (pair vu, excédent) pendant 3 s, mais jamais plus
/// de `ADMISSION_MAX_SECS` depuis que la salle a assez de joueurs : un pair qui clignote ne la retient pas.
#[test]
fn admission_wait_is_bounded_when_extra_peers_keep_appearing() {
    use super::{admission_waits as waits, ADMISSION_MAX_SECS, ADMISSION_SETTLE_SECS};
    use std::time::Duration;
    let ms = Duration::from_millis;
    // Rien de récent : on n'attend pas.
    assert!(!waits(None, None, ms(0)));
    assert!(!waits(Some(ms(3_000)), Some(ms(3_000)), ms(500)));
    // Un pair vu ou un excédent apparu depuis moins de 3 s : on attend.
    assert!(waits(Some(ms(2_900)), None, ms(2_900)));
    assert!(waits(None, Some(ms(100)), ms(100)));
    // Borne cumulée : même un pair apparu à l'instant ne retient plus l'admission.
    assert!(waits(Some(ms(0)), Some(ms(0)), ms(9_999)));
    assert!(!waits(Some(ms(0)), Some(ms(0)), ms(10_000)));
    assert!(ADMISSION_SETTLE_SECS < ADMISSION_MAX_SECS);
    assert!(ADMISSION_MAX_SECS < super::CONNECT_TIMEOUT_SECS);

    // Fantôme qui réapparaît toutes les 2 s (une nouvelle apparition au signaling, puis un excédent qui part et
    // revient), salle à deux joueurs connectés dès t = 0 ; une décision tous les 100 ms, 60 s.
    let admitted_at = |bounded: bool| {
        (0..=600u64).map(|tick| ms(tick * 100)).find(|now| {
            let since_ghost = ms(now.as_millis() as u64 % 2_000);
            let excess = (since_ghost < ms(1_000)).then_some(since_ghost);
            let enough_for = if bounded { *now } else { Duration::ZERO };
            !waits(Some(since_ghost), excess, enough_for)
        })
    };
    assert_eq!(
        admitted_at(false),
        None,
        "sans la borne cumulée, le fantôme retient l'admission sans fin"
    );
    assert_eq!(admitted_at(true), Some(ms(10_000)));
}
