//! Écrans de pause et d'inventaire (M2-T18, `docs/conventions.md` §56).
//!
//! Le modèle pur ([`model`]) et l'état ([`ScreensPlugin`], aussi en headless) transforment les
//! touches (Échap / Start, I / Select, flèches / D-pad, Entrée / A, Retour / B) et le script de
//! scénario ([`ScreenScript`]) en [`UiAction`](model::UiAction). Les commandes de sortie sont
//! [`ScreenCommand`] (`RunRequest::Restart` / `ToLobby`).
//!
//! Le rendu `bevy_hui` (`hui`, feature `ui_hui`) lie les gabarits `ui/*_screen.html` aux sections
//! nommées de la vue. Présentation seule : la simulation continue pendant qu'un écran est ouvert,
//! les entrées de jeu continuent d'être envoyées, et aucune trace ne change.

#[cfg(feature = "ui_hui")]
pub mod hui;
pub mod model;

use bevy::prelude::*;
use combat::inventory::AmmoReserves;
use items::{Inventory, ItemKind, ItemTable};
use model::{
    actions_from_keys, inventory_focus_len, inventory_view, merchant_lines, merchant_view,
    pause_view, reduce, ActiveLine, InventoryInput, NavKey, ScreenCommand, ScreenId, ScreenView,
    ScreensState, UiAction, WeaponLine,
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
    /// Frame de la dernière visite au marchand déjà ouverte, par joueur (M2-T13).
    pub merchant_seen: std::collections::BTreeMap<usize, u32>,
    /// Résultat du dernier achat, affiché par le marchand.
    pub merchant_message: String,
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

/// Ce que le marchand du hub lit et écrit (M2-T13) : profils (hors simulation), déblocages du
/// jeu, visites décidées dans la simulation.
#[derive(bevy::ecs::system::SystemParam)]
struct MerchantSource<'w> {
    registry: Option<Res<'w, content::registry::Registry>>,
    profiles: Option<ResMut<'w, meta::Profiles>>,
    settings: Option<Res<'w, crate::profile::ProfileSettings>>,
    hub: Option<Res<'w, crate::hub::HubState>>,
    confirmed: Option<Res<'w, bevy_ggrs::ConfirmedFrameCount>>,
}

impl MerchantSource<'_> {
    /// Les déblocages du jeu, dans l'ordre des ids.
    fn unlocks(&self) -> Vec<(String, items::UnlockDef)> {
        self.registry
            .as_deref()
            .map(|registry| {
                registry
                    .unlocks
                    .values()
                    .map(|entry| (entry.id.clone(), entry.def.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn profile(&self, handle: usize) -> meta::Profile {
        let id = meta::local_id(handle);
        self.profiles
            .as_deref()
            .and_then(|profiles| profiles.0.get(&id))
            .cloned()
            .unwrap_or_else(|| meta::Profile::new(id))
    }

    /// Visite au marchand décidée dans la simulation et **confirmée**, pas encore ouverte.
    fn new_visit(&self, handle: usize, seen: Option<u32>) -> Option<u32> {
        let visit = self.hub.as_deref()?.visits.get(&handle)?;
        let confirmed = self
            .confirmed
            .as_deref()
            .map_or(i64::MAX, |c| i64::from(c.0));
        (i64::from(visit.frame) <= confirmed && seen != Some(visit.frame)).then_some(visit.frame)
    }

    fn buy(&mut self, handle: usize, id: &str, def: &items::UnlockDef) -> Result<(), String> {
        let (Some(profiles), Some(settings)) =
            (self.profiles.as_deref_mut(), self.settings.as_deref())
        else {
            return Err("pas de profil".into());
        };
        crate::profile::purchase_for_handle(profiles, settings, handle, id, def)
            .map_err(|error| error.to_string())
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
    mut merchant: MerchantSource,
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
    let handle = hud_player.handle();
    let mut state = screens.state;
    let mut message = screens.merchant_message.clone();
    let mut merchant_seen = screens.merchant_seen.clone();
    // M2-T13 : une visite au marchand décidée dans la simulation et confirmée ouvre son écran.
    if let (Some(handle), None) = (handle, state.open) {
        if let Some(frame) = merchant.new_visit(handle, merchant_seen.get(&handle).copied()) {
            merchant_seen.insert(handle, frame);
            state = ScreensState {
                open: Some(ScreenId::Merchant),
                ..ScreensState::default()
            };
            message.clear();
        }
    }
    let unlocks = merchant.unlocks();
    for action in actions {
        let lines = merchant_lines(&unlocks, &merchant.profile(handle.unwrap_or(0)));
        let list_len = if state.open == Some(ScreenId::Merchant) {
            lines.len()
        } else {
            inventory_focus_len(&inventory)
        };
        match reduce(&mut state, action, list_len) {
            Some(ScreenCommand::Restart) => {
                commands.insert_resource(RunRequest::Restart);
            }
            Some(ScreenCommand::ToLobby) => {
                commands.insert_resource(RunRequest::ToLobby);
            }
            Some(ScreenCommand::Buy(index)) => {
                if let (Some(handle), Some((id, def))) = (handle, unlocks.get(index)) {
                    message = match merchant.buy(handle, id, def) {
                        Ok(()) => format!("{} : débloqué", def.name),
                        Err(reason) => format!("{} : {reason}", def.name),
                    };
                }
            }
            None => {}
        }
    }
    let view = state.open.map(|screen| match screen {
        ScreenId::Pause => pause_view(&state),
        ScreenId::Inventory => inventory_view(&inventory, &state),
        ScreenId::Merchant => {
            let profile = merchant.profile(handle.unwrap_or(0));
            merchant_view(
                &merchant_lines(&unlocks, &profile),
                &state,
                profile.currency(meta::ESSENCE),
                &message,
            )
        }
    });
    if screens.state != state
        || screens.view != view
        || screens.merchant_message != message
        || screens.merchant_seen != merchant_seen
    {
        screens.state = state;
        screens.view = view;
        screens.merchant_message = message;
        screens.merchant_seen = merchant_seen;
    }
}
