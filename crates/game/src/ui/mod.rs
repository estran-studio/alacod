use bevy::prelude::*;

pub mod disconnected;
pub mod floor_transition;
pub mod game_over;
pub mod hud;
pub mod hud_model;
pub mod lobby;
pub mod mutation_hui;
pub mod mutation_input;
pub mod mutation_screen;
pub mod screens;
pub mod texts;

/// Lobby et écran de déconnexion de l'application. Hors `PresentationPlugin` : ils tournent aussi
/// en headless p2p (`LobbyOnline`).
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lobby::LobbyUiPlugin);
        app.add_plugins(disconnected::DisconnectedUiPlugin);
    }
}

/// Clavier et manette des écrans (pause, inventaire, mutation), après leurs modèles : en headless
/// il n'y a rien à lire, comme avant la coupe (D69 E5a).
pub struct GameUiInputPlugin;

impl Plugin for GameUiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(mutation_input::MutationInputPlugin);
        app.add_plugins(screens::input::ScreensInputPlugin);
    }
}

#[cfg(feature = "debug_ui")]
pub mod weapon_hud;
pub mod weapon_visuals;
