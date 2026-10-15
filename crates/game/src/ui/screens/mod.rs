//! Écrans du test comparatif d'UI (M2-T18 partie 1, `docs/conventions.md` §56) : le menu de
//! pause et l'inventaire gungeon, faits deux fois.
//!
//! - **Modèle** ([`model`], pur) et **état** ([`ScreensPlugin`], aussi en headless, ressource
//!   [`Screens`]) : communs aux deux options. Les touches (Échap / Start, I / Select, flèches /
//!   D-pad, Entrée / A, Retour / B) et le script de scénario ([`ScreenScript`]) deviennent des
//!   [`UiAction`](model::UiAction) ; le seul chemin hors de l'écran est [`ScreenCommand`]
//!   (`RunRequest::Restart` / `ToLobby`, comme les boutons de l'écran de fin).
//! - **Option A, couche maison** ([`custom`]) : `ui/pause_screen.ron`, `ui/inventory_screen.ron`
//!   (arbre de boîtes et de sections liées par nom) construits en `bevy_ui`.
//! - **Option B, `bevy_hui`** (`hui`, feature `ui_hui`) : `ui/*_screen.html`, gabarits
//!   pseudo-HTML liés aux mêmes sections.
//!
//! Le choix du rendu est un réglage, `ALACOD_SCREENS=custom|hui` (défaut `custom`). La simulation
//! ne s'arrête jamais (GGRS) : les entrées de jeu continuent d'être envoyées pendant qu'un écran
//! est ouvert (comme l'écran de mutation) ; les bloquer est une décision de la partie 2.
//! Présentation seule : aucune trace ne change.

pub mod custom;
#[cfg(feature = "ui_hui")]
pub mod hui;
pub mod model;

use bevy::prelude::*;
use combat::inventory::AmmoReserves;
use items::{Inventory, ItemKind, ItemTable};
use model::{
    actions_from_keys, inventory_focus_len, inventory_view, pause_view, reduce, ActiveLine,
    InventoryInput, NavKey, ScreenCommand, ScreenId, ScreenView, ScreensState, UiAction,
    WeaponLine,
};
use serde::{Deserialize, Serialize};
use utils::frame::FrameCount;

use crate::core::AppState;
use crate::run_state::RunRequest;
use crate::ui::hud_model::HudPlayer;
use crate::weapons::{WeaponInventory, WeaponModesState, WeaponState};

/// État et vue des écrans (présentation, hors rollback).
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct Screens {
    pub state: ScreensState,
    /// `None` : aucun écran ouvert.
    pub view: Option<ScreenView>,
}

/// Un pas du script d'un scénario : l'action est jouée à la frame `at_frame` (format RON :
/// `(at_frame: 20, action: Toggle(Pause))`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStep {
    pub at_frame: u32,
    pub action: UiAction,
}

/// Script d'actions d'UI d'un scénario (ressource posée par le runner ; absente en jeu).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct ScreenScript {
    pub steps: Vec<UiStep>,
    /// Dernière frame dont les pas ont été joués : un pas ne l'est qu'une fois.
    pub played_until: Option<u32>,
}

/// Actions en attente (clavier, manette) : vidées à chaque update.
#[derive(Resource, Default)]
struct UiActionQueue(Vec<UiAction>);

/// Rendu choisi.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenBackend {
    Custom,
    Hui,
}

impl ScreenBackend {
    /// `ALACOD_SCREENS=custom|hui` ; défaut `custom`.
    pub fn from_env() -> Self {
        match std::env::var("ALACOD_SCREENS").as_deref() {
            Ok("hui") => Self::Hui,
            _ => Self::Custom,
        }
    }
}

/// Modèle des écrans, aussi en headless (attente de scénario `ScreenText`).
pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Screens>()
            .init_resource::<UiActionQueue>()
            .add_systems(
                Update,
                (read_nav_keys, update_screens)
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), |mut screens: ResMut<Screens>| {
                *screens = Screens::default();
            });
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

/// Ce que l'inventaire lit du joueur affiché.
#[derive(bevy::ecs::system::SystemParam)]
struct InventorySource<'w, 's> {
    weapons: Query<'w, 's, (&'static WeaponInventory, &'static AmmoReserves)>,
    states: Query<'w, 's, (&'static WeaponState, &'static WeaponModesState)>,
    inventories: Query<'w, 's, &'static Inventory>,
    table: Option<Res<'w, ItemTable>>,
}

impl InventorySource<'_, '_> {
    fn read(&self, player: Option<Entity>) -> InventoryInput {
        let Some(player) = player else {
            return InventoryInput::default();
        };
        let mut input = InventoryInput::default();
        if let Ok((inventory, reserves)) = self.weapons.get(player) {
            for (index, (entity, weapon)) in inventory.weapons.iter().enumerate() {
                let mag = self.states.get(*entity).ok().and_then(|(state, modes)| {
                    modes.modes.get(&state.active_mode).map(|m| m.mag_ammo)
                });
                input.weapons.push(WeaponLine {
                    name: weapon.config.name.clone(),
                    mag,
                    reserve: reserves.get(&weapon.config.ammo_type),
                    active: index == inventory.active_weapon_index,
                });
            }
        }
        if let Ok(items) = self.inventories.get(player) {
            let name = |id: &str| {
                self.table
                    .as_deref()
                    .and_then(|table| table.0.get(id))
                    .map_or_else(|| id.to_string(), |def| def.name.clone())
            };
            input.active = items.active.as_ref().map(|slot| ActiveLine {
                name: name(&slot.item),
                charge: slot.charge,
                required: self
                    .table
                    .as_deref()
                    .and_then(|table| table.0.get(&slot.item))
                    .and_then(|def| match def.kind {
                        ItemKind::Active(charge) => Some(charge.required()),
                        _ => None,
                    })
                    .unwrap_or(0),
            });
            input.passives = items.passives.iter().map(|id| name(id)).collect();
            input.consumables = items
                .consumables
                .iter()
                .filter(|(_, count)| **count > 0)
                .map(|(id, count)| (name(id), *count))
                .collect();
        }
        input
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_screens(
    mut commands: Commands,
    mut screens: ResMut<Screens>,
    mut queue: ResMut<UiActionQueue>,
    script: Option<ResMut<ScreenScript>>,
    frame: Res<FrameCount>,
    hud_player: HudPlayer,
    source: InventorySource,
) {
    let mut actions: Vec<UiAction> = queue.0.drain(..).collect();
    if let Some(mut script) = script {
        let from = script.played_until.map_or(0, |played| played + 1);
        actions.extend(
            script
                .steps
                .iter()
                .filter(|step| step.at_frame >= from && step.at_frame <= frame.frame)
                .map(|step| step.action),
        );
        script.played_until = Some(frame.frame);
    }
    let inventory = source.read(hud_player.entity());
    let mut state = screens.state;
    for action in actions {
        if let Some(command) = reduce(&mut state, action, inventory_focus_len(&inventory)) {
            commands.insert_resource(match command {
                ScreenCommand::Restart => RunRequest::Restart,
                ScreenCommand::ToLobby => RunRequest::ToLobby,
            });
        }
    }
    let view = state.open.map(|screen| match screen {
        ScreenId::Pause => pause_view(&state),
        ScreenId::Inventory => inventory_view(&inventory, &state),
    });
    if screens.state != state || screens.view != view {
        screens.state = state;
        screens.view = view;
    }
}
