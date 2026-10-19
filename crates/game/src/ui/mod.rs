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

/// Modèles de vue de l'interface, tenus à jour aussi en headless (lus par les attentes de
/// scénario `HudText` et `ScreenText`) : simulation de présentation, sans entrée ni rendu.
pub struct GameUiModelPlugin;

impl Plugin for GameUiModelPlugin {
    fn build(&self, app: &mut App) {
        // T1.16 : modèle de vue de l'écran de mutation, aussi en headless.
        app.add_plugins(mutation_screen::MutationScreenModelPlugin);
        // T1.18 : valeurs du HUD du joueur affiché (`HudSnapshot`), aussi en headless.
        app.add_plugins(hud_model::HudModelPlugin);
        // M2-T18 : état et vue des écrans pause et inventaire, aussi en headless.
        app.add_plugins(screens::ScreensPlugin);
    }
}

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
