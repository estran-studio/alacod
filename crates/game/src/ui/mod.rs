use bevy::prelude::*;

pub mod lobby;
pub mod disconnected;
pub mod game_over;
pub mod hud;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lobby::LobbyUiPlugin);
        app.add_plugins(disconnected::DisconnectedUiPlugin);
        app.add_plugins(game_over::GameOverUiPlugin);
    }
}
