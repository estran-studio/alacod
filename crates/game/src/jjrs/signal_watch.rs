//! D109 : pairs **vus par le signaling** mais pas (encore) connectés.
//!
//! Le socket matchbox n'expose que les pairs dont le canal de données est ouvert
//! (`MatchboxSocket::players`). Un pair que le serveur de signaling a annoncé (événement
//! `NewPeer`, ou premier `Signal` reçu de lui) mais dont la négociation ICE/DTLS ne finit jamais
//! reste invisible : le lobby affiche « Waiting for players: 1/2 » sans fin (un démarrage sur 24 à
//! 3 % de perte de paquets, `docs/taches/rapports/dettes-d105-d109.md`). Un [`Signaller`] qui
//! enveloppe celui de matchbox note ces apparitions dans un [`PeerSightings`] partagé ; le lobby
//! (`jjrs::p2p::wait_for_players`) en tire la durée d'attente du pair le plus ancien.
//!
//! Natif seulement pour l'observation (`ObservedSignallerBuilder` réimplémente le signaller
//! WebSocket de matchbox, dont le constructeur est privé) ; sur le web, [`PeerSightings`] reste
//! vide et le lobby n'a pas de délai. Présentation du réseau seulement : rien ici n'entre dans la
//! simulation, les traces ou le checksum.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::platform::time::Instant;
use bevy_matchbox::prelude::PeerId;

/// Pairs vus par le signaling, avec l'instant (horloge **murale**) de la première apparition.
#[derive(Debug, Default)]
pub struct PeerSightings {
    seen: BTreeMap<PeerId, Instant>,
}

/// [`PeerSightings`] partagé entre la tâche du signaller (thread du pool d'E/S) et le lobby.
pub type SharedSightings = Arc<Mutex<PeerSightings>>;

impl PeerSightings {
    /// Le serveur a annoncé `peer` (la première apparition seule compte).
    pub fn saw(&mut self, peer: PeerId, now: Instant) {
        self.seen.entry(peer).or_insert(now);
    }

    /// Le serveur a signalé le départ de `peer`.
    pub fn left(&mut self, peer: &PeerId) {
        self.seen.remove(peer);
    }

    /// Depuis combien de temps `peer` a été vu (`None` s'il ne l'a pas été).
    pub fn waited(&self, peer: &PeerId, now: Instant) -> Option<Duration> {
        self.seen
            .get(peer)
            .map(|since| now.saturating_duration_since(*since))
    }

    /// Nombre de pairs en attente (vus, ni connectés ni ignorés) et durée d'attente du plus
    /// ancien. `ignored` : pairs d'une session déjà jouée (`PreviousSessionPeers`, D86), dont le
    /// socket est fermé et qui ne se connecteront jamais.
    pub fn pending(
        &self,
        now: Instant,
        connected: &BTreeSet<PeerId>,
        ignored: &BTreeSet<PeerId>,
    ) -> Option<(usize, Duration)> {
        let waits: Vec<Duration> = self
            .seen
            .iter()
            .filter(|(peer, _)| !connected.contains(peer) && !ignored.contains(peer))
            .map(|(_, since)| now.saturating_duration_since(*since))
            .collect();
        let longest = waits.iter().max().copied()?;
        Some((waits.len(), longest))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::ObservedSignallerBuilder;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::time::Duration;

    use async_tungstenite::{
        async_std::{connect_async, ConnectStream},
        tungstenite::Message,
        WebSocketStream,
    };
    use bevy::{log::warn, platform::time::Instant};
    use bevy_matchbox::matchbox_socket::{
        async_trait::async_trait, PeerEvent, PeerRequest, SignalingError, Signaller,
        SignallerBuilder,
    };
    use futures_util::{SinkExt, StreamExt};

    use super::SharedSightings;

    /// Signaller WebSocket de matchbox (même protocole, même nombre de reconnexions) qui note
    /// dans `sightings` les pairs annoncés par le serveur.
    #[derive(Debug)]
    pub struct ObservedSignallerBuilder {
        pub sightings: SharedSightings,
    }

    struct ObservedSignaller {
        websocket_stream: WebSocketStream<ConnectStream>,
        sightings: SharedSightings,
    }

    #[async_trait]
    impl SignallerBuilder for ObservedSignallerBuilder {
        async fn new_signaller(
            &self,
            mut attempts: Option<u16>,
            room_url: String,
        ) -> Result<Box<dyn Signaller>, SignalingError> {
            let websocket_stream = loop {
                match connect_async(&room_url).await.map_err(SignalingError::from) {
                    Ok((stream, _)) => break stream,
                    Err(e) => {
                        let Some(attempts) = attempts.as_mut() else {
                            continue;
                        };
                        if *attempts <= 1 {
                            return Err(SignalingError::NegotiationFailed(Box::new(e)));
                        }
                        *attempts -= 1;
                        warn!(
                            "connection to signaling server failed, {attempts} attempt(s) remain"
                        );
                        futures_timer::Delay::new(Duration::from_secs(3)).await;
                    }
                }
            };
            Ok(Box::new(ObservedSignaller {
                websocket_stream,
                sightings: self.sightings.clone(),
            }))
        }
    }

    #[async_trait]
    impl Signaller for ObservedSignaller {
        async fn send(&mut self, request: PeerRequest) -> Result<(), SignalingError> {
            let request = serde_json::to_string(&request).expect("serializing request");
            self.websocket_stream
                .send(Message::Text(request.into()))
                .await
                .map_err(SignalingError::from)
        }

        async fn next_message(&mut self) -> Result<PeerEvent, SignalingError> {
            let message = match self.websocket_stream.next().await {
                Some(Ok(Message::Text(message))) => message,
                Some(Ok(_)) => return Err(SignalingError::UnknownFormat),
                Some(Err(err)) => return Err(SignalingError::from(err)),
                None => return Err(SignalingError::StreamExhausted),
            };
            let event: PeerEvent = serde_json::from_str(&message).map_err(|e| {
                warn!("failed to deserialize signaling message: {e:?}");
                SignalingError::UnknownFormat
            })?;
            if let Ok(mut sightings) = self.sightings.lock() {
                match &event {
                    PeerEvent::NewPeer(peer) | PeerEvent::Signal { sender: peer, .. } => {
                        sightings.saw(*peer, Instant::now());
                    }
                    PeerEvent::PeerLeft(peer) => sightings.left(peer),
                    PeerEvent::IdAssigned(_) => {}
                }
            }
            Ok(event)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(n: u8) -> PeerId {
        serde_json::from_str(&format!("\"00000000-0000-0000-0000-0000000000{n:02}\"")).unwrap()
    }

    #[test]
    fn pending_is_the_oldest_peer_neither_connected_nor_ignored() {
        let t0 = Instant::now();
        let mut sightings = PeerSightings::default();
        sightings.saw(peer(1), t0);
        sightings.saw(peer(2), t0 + Duration::from_secs(4));
        sightings.saw(peer(3), t0 + Duration::from_secs(1));
        // Une seconde apparition ne remet pas l'attente à zéro.
        sightings.saw(peer(1), t0 + Duration::from_secs(9));
        let now = t0 + Duration::from_secs(10);
        let none = BTreeSet::new();

        assert_eq!(
            sightings.pending(now, &none, &none),
            Some((3, Duration::from_secs(10)))
        );
        // Connecté : plus en attente ; ignoré (session précédente) : jamais compté.
        let connected = BTreeSet::from([peer(1)]);
        let ignored = BTreeSet::from([peer(3)]);
        assert_eq!(
            sightings.pending(now, &connected, &ignored),
            Some((1, Duration::from_secs(6)))
        );
        // Parti : retiré, et rien en attente sans pair vu.
        sightings.left(&peer(2));
        assert_eq!(sightings.pending(now, &connected, &ignored), None);
        assert_eq!(PeerSightings::default().pending(now, &none, &none), None);
    }
}
