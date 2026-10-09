//! Objets (M2-T0b, chantier C2, `docs/conventions.md` §36) : ramassage, charge et usage des
//! objets définis par la crate `items` (`ItemDef`, `Inventory`, `ItemPickup`).
//!
//! - **Contact** : un consommable au sol (`key`, `blank`…) est ramassé quand un joueur entre dans
//!   sa `pickup_range` ; plusieurs joueurs sur le même frame : le plus petit `GgrsNetId`.
//! - **Interaction** : un passif ou un actif porte un `Interactable { Item }` ; l'appui sur
//!   Interaction le ramasse (`handle_item_interaction`). Un actif remplace l'actif tenu, qui
//!   tombe au sol (sa charge est perdue). Les modificateurs d'un passif passent par `Modifiers`
//!   (source `item:<id>`, comme `powerup:<id>`).
//! - **Charge** (`item_charge_system`) : `Rooms(n)` compte les salles nettoyées
//!   (`world::RoomChanged` vers `Cleared`), `Damage(n)` les dégâts infligés par le porteur
//!   (`DamageEvent`), `Frames(n)` les frames.
//! - **Usage** (`item_use_active_system`) : bit `INPUT_USE_ACTIVE` avec l'actif chargé : les
//!   effets `OnUse` s'appliquent (`TimedModifier` : modificateur temporaire, `Modifier` :
//!   permanent) et la charge retombe à 0.
//!
//! `Inventory` est posé au **premier ramassage** : un joueur qui n'a rien ramassé n'en porte pas
//! (composant neutre au checksum, traces existantes inchangées).

use bevy::ecs::change_detection::Mut;
use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedVec2};
use bevy_ggrs::{GgrsSchedule, PlayerInputs, Rollback, RollbackDespawnCommandExtension};
use combat::actors::INPUT_USE_ACTIVE;
use effects::Action;
use items::{item_source, ActiveSlot, Inventory, ItemKind, ItemPicked, ItemPickup, ItemTable};
use sim_core::damage::DamageEvent;
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::interaction::{Interactable, InteractionType};
use sim_core::modifier::{ModifierSource, Modifiers};
use std::collections::{BTreeMap, BTreeSet};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};
use world::{RoomChanged, RoomState};

use crate::character::player::jjrs::PeerConfig;
use crate::character::player::Player;
use crate::interaction::InteractionEvent;
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;

/// Fait apparaître un objet au sol. Un passif ou un actif est interactable ; un consommable se
/// ramasse au contact. `None` si l'objet est inconnu de la table.
pub fn spawn_item_pickup(
    commands: &mut Commands,
    table: &ItemTable,
    item_id: String,
    position: fixed_math::FixedVec3,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Option<Entity> {
    let def = table.0.get(&item_id)?;
    let transform = fixed_math::FixedTransform3D::new(
        position,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );
    let g_id = id_factory.next(format!("item_{item_id}"));
    let mut entity = commands.spawn((
        transform.to_bevy_transform(),
        transform,
        ItemPickup { item_id },
        g_id,
    ));
    if def.kind != ItemKind::Consumable {
        entity.insert(Interactable {
            interaction_range: def.pickup_range,
            interaction_type: InteractionType::Item,
        });
    }
    Some(entity.insert(Rollback).id())
}

/// Inventaires à créer pendant un système, par `GgrsNetId` du joueur : un joueur sans
/// `Inventory` peut recevoir plusieurs objets dans la même frame.
type FreshInventories = BTreeMap<usize, (Entity, Inventory)>;

/// Applique `f` à l'inventaire du joueur, existant ou à créer (inséré par
/// [`insert_fresh_inventories`] en fin de système).
fn with_inventory<R>(
    existing: Option<Mut<Inventory>>,
    fresh: &mut FreshInventories,
    net_id: usize,
    entity: Entity,
    f: impl FnOnce(&mut Inventory) -> R,
) -> R {
    match existing {
        Some(mut inventory) => f(&mut inventory),
        None => f(&mut fresh
            .entry(net_id)
            .or_insert_with(|| (entity, Inventory::default()))
            .1),
    }
}

fn insert_fresh_inventories(commands: &mut Commands, fresh: FreshInventories) {
    for (_, (entity, inventory)) in fresh {
        commands.entity(entity).insert(inventory);
    }
}

type PlayerItems<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        Entity,
        &'static Player,
        &'static fixed_math::FixedTransform3D,
        Option<&'static mut Inventory>,
        &'static mut Modifiers,
    ),
    Without<ItemPickup>,
>;

/// Ramassage au contact des consommables : le joueur de plus petit `GgrsNetId` à portée.
#[allow(clippy::type_complexity)]
pub fn item_consumable_pickup_system(
    frame: Res<FrameCount>,
    table: Res<ItemTable>,
    mut commands: Commands,
    mut picked: ResMut<FrameEvents<ItemPicked>>,
    pickups: Query<
        (
            &GgrsNetId,
            Entity,
            &ItemPickup,
            &fixed_math::FixedTransform3D,
        ),
        With<Rollback>,
    >,
    mut players: PlayerItems,
) {
    if pickups.is_empty() {
        return;
    }
    let mut order: Vec<(usize, FixedVec2)> = players
        .iter()
        .map(|(id, _, _, t, ..)| (id.0, t.translation.truncate()))
        .collect();
    order.sort_by_key(|(id, _)| *id);

    let mut fresh = FreshInventories::new();
    for (pickup_net_id, pickup_entity, pickup, pickup_transform) in order_iter!(pickups) {
        let Some(def) = table.0.get(&pickup.item_id) else {
            continue;
        };
        if def.kind != ItemKind::Consumable {
            continue;
        }
        let pickup_pos = pickup_transform.translation.truncate();
        let Some((player_net_id, _)) = order
            .iter()
            .find(|(_, position)| pickup_pos.distance(position) <= def.pickup_range)
        else {
            continue;
        };
        let Some((_, entity, player, _, inventory, _)) =
            players.iter_mut().find(|(id, ..)| id.0 == *player_net_id)
        else {
            continue;
        };
        with_inventory(inventory, &mut fresh, *player_net_id, entity, |inventory| {
            *inventory
                .consumables
                .entry(pickup.item_id.clone())
                .or_default() += 1;
        });
        info!(
            "ggrs{{f={} item_pickup net_id={} item={} by={}}}",
            frame.frame, pickup_net_id.0, pickup.item_id, player_net_id
        );
        picked.send(ItemPicked {
            frame: frame.frame,
            player_handle: player.handle,
            item_id: pickup.item_id.clone(),
        });
        commands.entity(pickup_entity).despawn_rollback();
    }
    insert_fresh_inventories(&mut commands, fresh);
}

/// Ramassage par Interaction d'un passif ou d'un actif.
#[allow(clippy::too_many_arguments)]
pub fn handle_item_interaction(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    table: Res<ItemTable>,
    mut commands: Commands,
    mut picked: ResMut<FrameEvents<ItemPicked>>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    pickups: Query<(&ItemPickup, &fixed_math::FixedTransform3D), With<Rollback>>,
    mut players: PlayerItems,
) {
    let mut fresh = FreshInventories::new();
    // Un objet ne se ramasse qu'une fois : le premier événement (ordre `GgrsNetId` de
    // l'interactor) l'emporte.
    let mut taken: BTreeSet<usize> = BTreeSet::new();
    for event in events.iter() {
        if event.interaction_type != InteractionType::Item {
            continue;
        }
        if taken.contains(&event.interactable_net_id.0) {
            continue;
        }
        let Ok((pickup, pickup_transform)) = pickups.get(event.interactable) else {
            continue;
        };
        let Some(def) = table.0.get(&pickup.item_id) else {
            continue;
        };
        let Some((_, entity, player, player_transform, inventory, mut modifiers)) = players
            .iter_mut()
            .find(|(id, ..)| id.0 == event.interactor_net_id.0)
        else {
            continue;
        };
        taken.insert(event.interactable_net_id.0);

        let mut dropped: Option<String> = None;
        with_inventory(
            inventory,
            &mut fresh,
            event.interactor_net_id.0,
            entity,
            |inventory| match def.kind {
                ItemKind::Passive => {
                    inventory.passives.push(pickup.item_id.clone());
                    for modifier in &def.modifiers {
                        modifiers.push(modifier.to_modifier(&pickup.item_id));
                    }
                }
                ItemKind::Active(_) => {
                    dropped = inventory.active.take().map(|slot| slot.item);
                    inventory.active = Some(ActiveSlot {
                        item: pickup.item_id.clone(),
                        charge: 0,
                    });
                }
                ItemKind::Consumable => {}
            },
        );
        if def.kind == ItemKind::Consumable {
            continue;
        }
        info!(
            "ggrs{{f={} item_pickup net_id={} item={} by={}}}",
            frame.frame, event.interactable_net_id.0, pickup.item_id, event.interactor_net_id.0
        );
        picked.send(ItemPicked {
            frame: frame.frame,
            player_handle: player.handle,
            item_id: pickup.item_id.clone(),
        });
        commands.entity(event.interactable).despawn_rollback();
        // L'actif remplacé tombe aux pieds du joueur (sa charge est perdue).
        if let Some(old) = dropped {
            spawn_item_pickup(
                &mut commands,
                &table,
                old,
                player_transform.translation,
                &mut id_factory,
            );
        }
        let _ = pickup_transform;
    }
    insert_fresh_inventories(&mut commands, fresh);
}

/// Charge des actifs tenus : frames, salles nettoyées, dégâts infligés.
pub fn item_charge_system(
    table: Res<ItemTable>,
    rooms: Option<Res<FrameEvents<RoomChanged>>>,
    damage: Option<Res<FrameEvents<DamageEvent>>>,
    mut players: Query<(&GgrsNetId, &mut Inventory)>,
) {
    let cleared = rooms.map_or(0, |events| {
        events.iter().filter(|e| e.to == RoomState::Cleared).count() as u32
    });
    for (net_id, mut inventory) in players.iter_mut() {
        let Some(slot) = inventory.active.as_ref() else {
            continue;
        };
        let Some(ItemKind::Active(charge)) = table.0.get(&slot.item).map(|def| def.kind) else {
            continue;
        };
        let delta = match charge {
            items::ActiveCharge::Frames(_) => 1,
            items::ActiveCharge::Rooms(_) => cleared,
            items::ActiveCharge::Damage(_) => damage.as_ref().map_or(0, |events| {
                events
                    .iter()
                    .filter(|e| e.source.0 == net_id.0)
                    .map(|e| e.amount.to_num::<u32>())
                    .sum()
            }),
        };
        if delta == 0 {
            continue;
        }
        if let Some(slot) = inventory.active.as_mut() {
            slot.charge = slot.charge.saturating_add(delta).min(charge.required());
        }
    }
}

/// `UseActive` avec l'actif chargé : applique ses effets `OnUse`, la charge retombe à 0.
pub fn item_use_active_system(
    frame: Res<FrameCount>,
    table: Res<ItemTable>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    mut players: Query<(&GgrsNetId, &Player, &mut Inventory, &mut Modifiers)>,
) {
    for (net_id, player, mut inventory, mut modifiers) in utils::order_mut_iter!(players) {
        if inputs[player.handle].0.buttons & INPUT_USE_ACTIVE == 0 {
            continue;
        }
        let Some(slot) = inventory.active.as_ref() else {
            continue;
        };
        let item_id = slot.item.clone();
        let Some(def) = table.0.get(&item_id) else {
            continue;
        };
        let ItemKind::Active(charge) = def.kind else {
            continue;
        };
        if slot.charge < charge.required() {
            continue;
        }
        let source: ModifierSource = item_source(&item_id);
        for effect in def.effects.iter().filter(|e| e.on == effects::On::OnUse) {
            for action in &effect.r#do {
                match action {
                    Action::TimedModifier { .. } => {
                        if let Some(modifier) = action.as_modifier(frame.frame, source.clone()) {
                            modifiers.push(modifier);
                        }
                    }
                    Action::Modifier { stat, op, value } => {
                        modifiers.push_from(source.clone(), stat.clone(), *op, *value, None)
                    }
                    // Refusées par le lint (effets v2).
                    _ => {}
                }
            }
        }
        if let Some(slot) = inventory.active.as_mut() {
            slot.charge = 0;
        }
        info!(
            "ggrs{{f={} item_use net_id={} item={}}}",
            frame.frame, net_id.0, item_id
        );
    }
}

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemTable>()
            .add_frame_events_neutral::<ItemPicked>()
            .rollback_and_trace_neutral::<Inventory>()
            .rollback_and_trace_neutral::<ItemPickup>();
        // Ramassage par Interaction : après les autres handlers d'`InteractionEvent`.
        app.add_systems(
            GgrsSchedule,
            handle_item_interaction
                .after(crate::interaction::handle_perk_purchase_interaction)
                .before(RollbackSystemSet::Movement)
                .in_set(RollbackSystemSet::Interaction),
        );
        app.add_systems(
            GgrsSchedule,
            (
                item_consumable_pickup_system.after(crate::powerups::apply_powerup_actions_system),
                item_use_active_system.after(item_consumable_pickup_system),
            )
                .in_set(RollbackSystemSet::Effects),
        );
        // La charge lit `RoomChanged` (émis dans `Run`, vidé au `FrameStart` suivant) et les
        // dégâts de la frame : après `Run`, avant le compteur de frames. Un actif chargé par la
        // salle nettoyée est utilisable à la frame suivante.
        app.add_systems(
            GgrsSchedule,
            item_charge_system
                .after(RollbackSystemSet::Run)
                .before(RollbackSystemSet::FrameCounter),
        );
    }
}
