//! Host-provided session admission, outside the rollback simulation.
use bevy::prelude::*;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
#[derive(Resource, Deserialize)]
pub struct BrowserSession {
    pub signaling_url: String,
}
#[derive(Deserialize)]
pub struct Participant {
    pub player_id: String,
    pub username: String,
    pub peer_id: Option<String>,
}
pub fn attribute(name: &str) -> Option<String> {
    web_sys::window()?
        .document()?
        .query_selector("#bevy-canvas")
        .ok()??
        .get_attribute(name)
}
pub fn session() -> Option<BrowserSession> {
    serde_json::from_str(&attribute("data-session")?).ok()
}
pub fn participants() -> Option<Vec<Participant>> {
    serde_json::from_str(&attribute("data-participants")?).ok()
}
#[wasm_bindgen]
extern "C" {
    pub fn alacod_return_to_lobby();
}

/// On the single-threaded browser runtime, stop GGRS sends as soon as the
/// Matchbox future closes. Its default socket adapter panics on a closed channel.
/// The host destroys this iframe after receiving the return-to-lobby message.
pub struct BrowserChannel {
    channel: bevy_matchbox::matchbox_socket::WebRtcChannel,
    returned: bool,
}

impl BrowserChannel {
    pub fn new(channel: bevy_matchbox::matchbox_socket::WebRtcChannel) -> Self {
        Self {
            channel,
            returned: false,
        }
    }

    fn closed(&mut self) -> bool {
        if !self.channel.is_closed() {
            return false;
        }
        if !self.returned {
            self.returned = true;
            alacod_return_to_lobby();
        }
        true
    }
}

impl ggrs::NonBlockingSocket<bevy_matchbox::prelude::PeerId> for BrowserChannel {
    fn send_to(&mut self, message: &ggrs::Message, peer: &bevy_matchbox::prelude::PeerId) {
        if !self.closed() {
            ggrs::NonBlockingSocket::send_to(&mut self.channel, message, peer);
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(bevy_matchbox::prelude::PeerId, ggrs::Message)> {
        if self.closed() {
            Vec::new()
        } else {
            ggrs::NonBlockingSocket::receive_all_messages(&mut self.channel)
        }
    }
}
