use bevy::prelude::*;

pub mod disconnected;
pub mod floor_transition;
pub mod game_over;
pub mod hud;
pub mod lobby;
pub mod mutation_screen;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lobby::LobbyUiPlugin);
        app.add_plugins(disconnected::DisconnectedUiPlugin);
        app.add_plugins(game_over::GameOverUiPlugin);
        // T1.16 : modèle de vue de l'écran de mutation, aussi en headless.
        app.add_plugins(mutation_screen::MutationScreenModelPlugin);
    }
}

#[cfg(feature = "debug_ui")]
pub mod weapon_hud;
pub mod weapon_visuals;
