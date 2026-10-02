use bevy::prelude::*;

pub mod disconnected;
pub mod game_over;
pub mod hud;
pub mod lobby;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lobby::LobbyUiPlugin);
        app.add_plugins(disconnected::DisconnectedUiPlugin);
        app.add_plugins(game_over::GameOverUiPlugin);
    }
}

#[cfg(feature = "debug_ui")]
pub mod weapon_hud;
pub mod weapon_visuals;
