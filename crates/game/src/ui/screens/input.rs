//! Lecture du clavier et de la manette pour les écrans de pause et d'inventaire (D69 E5a).
//!
//! Adaptateur d'entrée (application) : le modèle des écrans (`ui::screens`, simulation de
//! présentation, aussi en headless) ne voit que la file [`UiActionQueue`].

use bevy::prelude::*;

use super::model::{actions_from_keys, NavKey};
use super::{update_screens, Screens, UiActionQueue};
use crate::session::AppState;

/// Clavier et manette des écrans ; ajouté après le modèle (`ui::GameUiInputPlugin`).
pub struct ScreensInputPlugin;

impl Plugin for ScreensInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            read_nav_keys
                .before(update_screens)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Touches pressées cette frame, clavier et manette (rien en headless ni avec
/// `ALACOD_INPUT=neutral`).
fn read_nav_keys(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    mut queue: ResMut<UiActionQueue>,
    screens: Res<Screens>,
) {
    if std::env::var("ALACOD_INPUT").is_ok_and(|v| v == "neutral") {
        return;
    }
    let mut pressed = Vec::new();
    if let Some(keyboard) = keyboard {
        for (key, nav) in [
            (KeyCode::Escape, NavKey::Menu),
            (KeyCode::KeyI, NavKey::Inventory),
            (KeyCode::ArrowUp, NavKey::Up),
            (KeyCode::ArrowDown, NavKey::Down),
            (KeyCode::Enter, NavKey::Confirm),
            (KeyCode::Space, NavKey::Confirm),
            (KeyCode::Backspace, NavKey::Back),
        ] {
            if keyboard.just_pressed(key) {
                pressed.push(nav);
            }
        }
    }
    for gamepad in &gamepads {
        for (button, nav) in [
            (GamepadButton::Start, NavKey::Menu),
            (GamepadButton::Select, NavKey::Inventory),
            (GamepadButton::DPadUp, NavKey::Up),
            (GamepadButton::DPadDown, NavKey::Down),
            (GamepadButton::South, NavKey::Confirm),
            (GamepadButton::East, NavKey::Back),
        ] {
            if gamepad.just_pressed(button) {
                pressed.push(nav);
            }
        }
    }
    // Hors d'un écran ouvert, les flèches, Entrée et A sont des touches de jeu.
    if screens.state.open.is_none() {
        pressed.retain(|key| matches!(key, NavKey::Menu | NavKey::Inventory));
    }
    queue
        .0
        .extend(actions_from_keys(screens.state.open, &pressed));
}
