//! Écrans de pause et d'inventaire (M2-T18, `docs/conventions.md` §56).
//!
//! Le modèle pur ([`model`]) et l'état ([`ScreensPlugin`], aussi en headless) transforment les
//! touches (Échap / Start, I / Select, flèches / D-pad, Entrée / A, Retour / B) et le script de
//! scénario ([`ScreenScript`](model::ScreenScript)) en [`UiAction`](model::UiAction). Les commandes de sortie sont
//! [`ScreenCommand`] (`RunRequest::Restart` / `ToLobby`).
//!
//! Le rendu `bevy_hui` (`hui`, feature `ui_hui`) lie les gabarits `ui/*_screen.html` aux sections
//! nommées de la vue. Présentation seule : la simulation continue pendant qu'un écran est ouvert,
//! les entrées de jeu continuent d'être envoyées, et aucune trace ne change.

pub mod help;
#[cfg(feature = "ui_hui")]
pub mod hui;
pub mod input;
pub mod model;

use bevy::prelude::*;
use combat::inventory::AmmoReserves;
use items::{Inventory, ItemKind, ItemTable};
use model::{
    inventory_focus_len, inventory_view, merchant_lines, merchant_view, pause_view_with_help,
    reduce, ActiveLine, InventoryInput, ScreenCommand, ScreenId, ScreenScript, ScreenView,
    ScreensState, UiAction, WeaponLine,
};
use utils::frame::FrameCount;

use crate::character::config::{CharacterConfigKey, CharacterConfigs};
use crate::character::control::get_input_map;
use crate::progression::ProgressionTable;
use crate::run_state::RunRequest;
use crate::session::AppState;
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

/// Actions en attente (clavier, manette, lues par `input::read_nav_keys`) : vidées à chaque
/// update.
#[derive(Resource, Default)]
pub struct UiActionQueue(pub Vec<UiAction>);

/// Modèle des écrans, aussi en headless (attente de scénario `ScreenText`). Sans clavier ni
/// manette : `input::ScreensInputPlugin`.
pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Screens>()
            .init_resource::<UiActionQueue>()
            .add_systems(Update, update_screens.run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), |mut screens: ResMut<Screens>| {
                *screens = Screens::default();
            });
    }
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
    /// Les déblocages du jeu à acheter, dans l'ordre des ids : ceux qu'un accomplissement accorde
    /// (M3-T10, §75) n'y sont pas, on les obtient en jouant.
    fn unlocks(&self) -> Vec<(String, items::UnlockDef)> {
        self.registry
            .as_deref()
            .map(|registry| {
                let earned = registry.achievement_unlocks();
                registry
                    .unlocks
                    .values()
                    .filter(|entry| !earned.contains(entry.id.as_str()))
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

/// Ce que l'aide des touches lit pour filtrer ses lignes par clone (`help::HelpContext`).
#[derive(bevy::ecs::system::SystemParam)]
struct HelpSource<'w, 's> {
    configs: Option<Res<'w, CharacterConfigs>>,
    keys: Query<'w, 's, &'static CharacterConfigKey>,
    items: Option<Res<'w, ItemTable>>,
    progression: Option<Res<'w, ProgressionTable>>,
}

impl HelpSource<'_, '_> {
    fn context(&self, player: Option<Entity>) -> help::HelpContext {
        let config = player
            .and_then(|player| self.keys.get(player).ok())
            .and_then(|key| self.configs.as_deref()?.get(&key.0));
        let items = self.items.as_deref().map(|table| &table.0);
        help::HelpContext {
            dash: config.is_some_and(|config| {
                config.dash.is_some()
                    || config.movement.dash_distance > bevy_fixed::fixed_math::Fixed::ZERO
            }),
            active: items.is_some_and(|items| {
                items
                    .values()
                    .any(|def| matches!(def.kind, ItemKind::Active(_)))
            }),
            blank: items.is_some_and(|items| items.values().any(|def| def.blank.is_some())),
            bomb: config.is_some_and(|config| config.bomb.is_some()),
            mutations: self
                .progression
                .as_deref()
                .is_some_and(|table| table.active.is_some()),
        }
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
    help: HelpSource,
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
        ScreenId::Pause => {
            let help_text = if state.help_shown {
                help::help_text(&get_input_map(), &help.context(hud_player.entity()))
            } else {
                String::new()
            };
            pause_view_with_help(&state, &help_text)
        }
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
