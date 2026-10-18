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
use combat::actors::{INPUT_BLANK, INPUT_USE_ACTIVE};
use combat::weapons::Bullet;
use effects::Action;
use items::{
    item_source, synergy_source, ActiveSlot, BlankUsed, Inventory, ItemKind, ItemPicked,
    ItemPickup, ItemTable, SynergyTable,
};
use sim_core::damage::DamageEvent;
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::interaction::{Interactable, InteractionType};
use sim_core::modifier::{Modifier, ModifierSource, Modifiers};
use sim_core::team::Team;
use std::collections::{BTreeMap, BTreeSet};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};
use world::{RoomChanged, RoomState};

use crate::character::health::Health;
use crate::character::player::jjrs::PeerConfig;
use crate::character::player::Player;
use crate::effects_runtime::{apply_action_sourced, EffectAssets, EffectState};
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
pub(crate) type FreshInventories = BTreeMap<usize, (Entity, Inventory)>;

/// Applique `f` à l'inventaire du joueur, existant ou à créer (inséré par
/// [`insert_fresh_inventories`] en fin de système).
pub(crate) fn with_inventory<R>(
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

pub(crate) fn insert_fresh_inventories(commands: &mut Commands, fresh: FreshInventories) {
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
        Option<&'static mut Health>,
    ),
    Without<ItemPickup>,
>;

/// Ramassage au contact des consommables : le joueur de plus petit `GgrsNetId` à portée.
#[allow(clippy::type_complexity)]
pub fn item_consumable_pickup_system(
    frame: Res<FrameCount>,
    table: Res<ItemTable>,
    mut commands: Commands,
    assets: EffectAssets,
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
        // M3-T3 (§68) : le premier joueur (ordre `GgrsNetId`) à portée **qui peut le prendre** :
        // compteur au plafond (`max_stack`) ou soin refusé à pleine vie (`refuse_when_full`)
        // laissent l'objet au sol, et un autre joueur à portée le prend. Sans ces deux champs,
        // `accepts` est toujours vrai : le premier joueur à portée, comme avant.
        let Some((player_net_id, _)) = order.iter().find(|(id, position)| {
            pickup_pos.distance(position) <= def.pickup_range
                && players
                    .iter()
                    .find(|(other, ..)| other.0 == *id)
                    .is_some_and(|(_, _, _, _, inventory, _, health)| {
                        let stock = match inventory {
                            Some(inventory) => inventory.consumable(&pickup.item_id),
                            None => fresh
                                .get(id)
                                .map_or(0, |(_, fresh)| fresh.consumable(&pickup.item_id)),
                        };
                        let full = health.is_some_and(|health| health.current >= health.max);
                        def.accepts(stock, full)
                    })
        }) else {
            continue;
        };
        let Some((owner_id, entity, player, _, inventory, mut modifiers, health)) =
            players.iter_mut().find(|(id, ..)| id.0 == *player_net_id)
        else {
            continue;
        };
        if def.effects.is_empty() {
            with_inventory(inventory, &mut fresh, *player_net_id, entity, |inventory| {
                *inventory
                    .consumables
                    .entry(pickup.item_id.clone())
                    .or_default() += 1;
            });
        } else {
            // M2-T4 : un consommable à effets `OnUse` s'applique au ramassage (aucun bouton
            // d'usage n'existe pour lui) et n'entre pas dans l'inventaire.
            apply_instant_effects(
                def,
                &pickup.item_id,
                frame.frame,
                owner_id,
                health,
                &mut modifiers,
                &mut commands,
                entity,
                &assets,
            );
        }
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
        let Some((_, entity, player, player_transform, inventory, mut modifiers, _)) = players
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
                    inventory.active = Some(ActiveSlot::new(pickup.item_id.clone()));
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

/// Applique les effets `OnUse` d'un objet (actif utilisé, consommable ramassé), source `item:<id>` :
/// chaque action passe par `effects_runtime::apply_action_sourced`, la même exécution que les effets
/// de personnage et d'objet tenu (D80 : une seule liste d'actions). `Explode` pose donc une bombe à
/// mèche d'une frame au point du porteur lu à l'application, créditée à `net_id`. Le lint
/// (`content::lint`) ne laisse passer à `OnUse` que les actions qui ont un sens pour un objet
/// (`Heal`, `GainShield`, `GainSoul`, `Modifier`, `TimedModifier`, `Explode`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_instant_effects(
    def: &items::ItemDef,
    item_id: &str,
    frame: u32,
    net_id: &GgrsNetId,
    mut health: Option<Mut<Health>>,
    modifiers: &mut Modifiers,
    commands: &mut Commands,
    entity: Entity,
    assets: &EffectAssets,
) {
    let source: ModifierSource = item_source(item_id);
    // Aucune action `OnUse` n'écrit l'état d'effet (`GaugeAdd` est refusée par le lint, sans jauge
    // ici) : il n'existe que pour la signature commune.
    let mut state = EffectState::default();
    for effect in def.effects.iter().filter(|e| e.on == effects::On::OnUse) {
        for action in &effect.r#do {
            apply_action_sourced(
                action,
                net_id,
                source.clone(),
                frame,
                health.as_deref_mut(),
                modifiers,
                &mut state,
                None,
                (entity, false),
                commands,
                assets,
            );
        }
    }
}

/// Synergies (M2-T4, `docs/conventions.md` §44) : les modificateurs `synergy:<id>` d'un joueur
/// suivent l'`Inventory` à chaque frame (état dérivé, aucun composant) : posés quand tous les
/// objets requis sont tenus (et, pour une transformation, `count` objets distincts du tag :
/// `items::SynergyDef::is_active`, suivi Q10), retirés dès qu'un manque (actif remplacé). Synergies par id,
/// joueurs par `GgrsNetId`. Les effets des synergies sont lus par
/// `game::effects_runtime` (clé `synergy:<id>:<index>`).
pub fn synergy_modifiers_system(
    synergies: Res<SynergyTable>,
    items: Res<ItemTable>,
    mut players: Query<(&GgrsNetId, &Inventory, &mut Modifiers)>,
) {
    if synergies.0.is_empty() {
        return;
    }
    for (_, inventory, mut modifiers) in utils::order_mut_iter!(players) {
        for (id, def) in &synergies.0 {
            if def.modifiers.is_empty() {
                continue;
            }
            let source = synergy_source(id);
            let wanted = def.is_active(inventory, &items);
            let present = modifiers.iter().any(|m| m.source == source);
            if wanted && !present {
                for modifier in &def.modifiers {
                    modifiers.push(Modifier {
                        stat: modifier.stat.clone(),
                        op: modifier.op,
                        value: modifier.value,
                        source: source.clone(),
                        until: None,
                    });
                }
            } else if !wanted && present {
                modifiers.remove_by_source(&source);
            }
        }
    }
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
/// M2-T4 : refusé (la charge reste) si l'étage a déjà servi `uses_per_floor` fois, ou si le
/// stock du consommable `cost` manque ; sinon le coût est prélevé et l'usage compté.
#[allow(clippy::type_complexity)]
pub fn item_use_active_system(
    frame: Res<FrameCount>,
    table: Res<ItemTable>,
    floor: Option<Res<run::FloorState>>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    mut commands: Commands,
    assets: EffectAssets,
    mut players: Query<(
        &GgrsNetId,
        Entity,
        &Player,
        &mut Inventory,
        &mut Modifiers,
        Option<&mut Health>,
    )>,
) {
    let floor_index = floor.map_or(0, |f| f.index);
    for (net_id, entity, player, mut inventory, mut modifiers, health) in
        utils::order_mut_iter!(players)
    {
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
        if def
            .uses_per_floor
            .is_some_and(|limit| slot.uses_on(floor_index) >= limit)
        {
            continue;
        }
        if let Some((cost_item, count)) = &def.cost {
            if inventory.consumable(cost_item) < *count {
                continue;
            }
        }
        if let Some((cost_item, count)) = &def.cost {
            if let Some(stock) = inventory.consumables.get_mut(cost_item) {
                *stock -= *count;
            }
        }
        apply_instant_effects(
            def,
            &item_id,
            frame.frame,
            net_id,
            health,
            &mut modifiers,
            &mut commands,
            entity,
            &assets,
        );
        let counts_uses = def.uses_per_floor.is_some();
        if let Some(slot) = inventory.active.as_mut() {
            slot.charge = 0;
            if counts_uses {
                slot.uses = Some((floor_index, slot.uses_on(floor_index) + 1));
            }
        }
        info!(
            "ggrs{{f={} item_use net_id={} item={}}}",
            frame.frame, net_id.0, item_id
        );
    }
}

/// Consommables remontés à `n` au début de chaque étage du mode `Floors` (M2-T1) : donnée du
/// contenu (`FloorsEntry::consumables_per_floor`), hors rollback, posée par `map_ldtk` au
/// chargement ; vide sinon.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct FloorStartConsumables(pub BTreeMap<String, u32>);

/// Mémoire du blank d'un joueur (M2-T1), rollback **neutre**, posée par [`blank_system`] sur un
/// joueur qui a un `Inventory` et un motif d'en avoir (appui, blank, recharge d'étage) : `held` = Blank était tenu à la frame précédente (front montant :
/// un Blank tenu ne vide pas l'inventaire), `floor` = étage dont les consommables ont été servis.
/// Réécrit à chaque frame, avant tout retour anticipé (piège « donnée dérivée non rollbackée »).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlankState {
    pub held: bool,
    pub floor: u32,
}

/// Blank (M2-T1, §41) : au front montant de `Blank`, avec un consommable porteur de
/// [`items::BlankSpec`] en inventaire : en consomme un, efface (`despawn_rollback`) les balles
/// ennemies dans le rayon (jamais celles des joueurs ni des alliés), rend le joueur invulnérable
/// `iframes` frames, émet `BlankUsed`. Remonte aussi les consommables de début d'étage.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn blank_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    table: Res<ItemTable>,
    refill: Res<FloorStartConsumables>,
    floor: Option<Res<run::FloorState>>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    mut events: ResMut<FrameEvents<BlankUsed>>,
    mut players: Query<
        (
            &GgrsNetId,
            Entity,
            &Player,
            &Team,
            &fixed_math::FixedTransform3D,
            &mut Inventory,
            Option<&mut BlankState>,
            Option<&mut crate::character::health::Health>,
        ),
        Without<Bullet>,
    >,
    bullets: Query<(&GgrsNetId, Entity, &fixed_math::FixedTransform3D, &Bullet), With<Rollback>>,
) {
    let floor_index = floor.map_or(0, |f| f.index);
    for (net_id, entity, player, team, transform, mut inventory, state, health) in
        utils::order_mut_iter!(players)
    {
        let pressed = inputs[player.handle].0.buttons & INPUT_BLANK != 0;
        // Pas de mémoire tant que rien ne la justifie (appui, blank en poche, recharge d'étage) :
        // un joueur avec `Inventory` mais sans blank ne change pas la trace (parité des types).
        let has_blank = table
            .0
            .iter()
            .any(|(id, def)| def.blank.is_some() && inventory.consumable(id) > 0);
        let (held, served_floor) = match &state {
            Some(state) => (state.held, state.floor),
            None if pressed || has_blank || !refill.0.is_empty() => (false, floor_index),
            None => continue,
        };
        let rising = pressed && !held;
        if served_floor != floor_index {
            for (id, n) in &refill.0 {
                let count = inventory.consumables.entry(id.clone()).or_default();
                *count = (*count).max(*n);
            }
        }
        match state {
            Some(mut state) => {
                state.held = pressed;
                state.floor = floor_index;
            }
            None => {
                commands.entity(entity).insert(BlankState {
                    held: pressed,
                    floor: floor_index,
                });
            }
        }
        if !rising {
            continue;
        }
        let Some((blank_id, spec)) = table.0.iter().find_map(|(id, def)| {
            let spec = def.blank?;
            (inventory.consumable(id) > 0).then_some((id.clone(), spec))
        }) else {
            continue;
        };
        if let Some(count) = inventory.consumables.get_mut(&blank_id) {
            *count -= 1;
        }
        let center = transform.translation.truncate();
        let mut cleared = 0u32;
        for (_, bullet_entity, bullet_transform, bullet) in order_iter!(bullets) {
            let hostile = bullet.source_team != *team
                && !matches!(bullet.source_team, Team::Allies | Team::Neutral);
            if hostile && bullet_transform.translation.truncate().distance(&center) <= spec.radius {
                commands.entity(bullet_entity).despawn_rollback();
                cleared += 1;
            }
        }
        if let (Some(mut health), true) = (health, spec.iframes > 0) {
            let until = frame.frame + spec.iframes - 1;
            health.invulnerable_until_frame = Some(
                health
                    .invulnerable_until_frame
                    .map_or(until, |current| current.max(until)),
            );
        }
        events.send(BlankUsed {
            frame: frame.frame,
            player_handle: player.handle,
            x: center.x,
            y: center.y,
            radius: spec.radius,
            cleared,
        });
        info!(
            "ggrs{{f={} blank net_id={} item={} cleared={}}}",
            frame.frame, net_id.0, blank_id, cleared
        );
    }
}

/// Dernier blank utilisé, pour l'onde de présentation (M2-T1) : dérivé de `FrameEvents<BlankUsed>`
/// dans `Update`, jamais lu par la simulation. Un rollback qui réémet l'événement réécrit les
/// mêmes valeurs.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct LastBlank(pub Option<BlankUsedAt>);

#[derive(Clone, Copy, Debug)]
pub struct BlankUsedAt {
    pub frame: u32,
    pub center: Vec2,
    pub radius: f32,
}

/// Durée de l'onde, en frames de simulation.
pub const BLANK_WAVE_FRAMES: u32 = 24;

fn track_blank_system(events: Res<FrameEvents<BlankUsed>>, mut last: ResMut<LastBlank>) {
    for blank in events.iter() {
        last.0 = Some(BlankUsedAt {
            frame: blank.frame,
            center: Vec2::new(blank.x.to_num::<f32>(), blank.y.to_num::<f32>()),
            radius: blank.radius.to_num::<f32>(),
        });
    }
}

/// Onde du blank : un cercle qui s'étend jusqu'au rayon et s'estompe (`Update`, présentation).
fn draw_blank_wave_system(
    last: Res<LastBlank>,
    frame: Res<FrameCount>,
    config: Option<Res<crate::feedback::FeedbackConfigLoaded>>,
    mut gizmos: Gizmos,
) {
    // M2-T19 (§55) : l'anneau de `ui/feedback.ron` (`blank`) remplace cette onde quand il est actif.
    if config.is_some_and(|config| config.0.blank.enabled) {
        return;
    }
    let Some(blank) = last.0 else {
        return;
    };
    let age = frame.frame.saturating_sub(blank.frame);
    if age >= BLANK_WAVE_FRAMES {
        return;
    }
    let progress = age as f32 / BLANK_WAVE_FRAMES as f32;
    let color = Color::srgba(0.7, 0.9, 1.0, 1.0 - progress);
    gizmos.circle_2d(
        Isometry2d::from_translation(blank.center),
        blank.radius * progress,
        color,
    );
}

/// Présentation du blank (absente en headless, avec `PresentationPlugin`).
pub struct BlankPresentationPlugin;

impl Plugin for BlankPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastBlank>().add_systems(
            Update,
            (
                track_blank_system,
                draw_blank_wave_system.after(track_blank_system),
            ),
        );
    }
}

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemTable>()
            .init_resource::<SynergyTable>()
            .init_resource::<FloorStartConsumables>()
            .add_frame_events_neutral::<ItemPicked>()
            .add_frame_events_neutral::<BlankUsed>()
            .rollback_and_trace_neutral::<BlankState>()
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
                blank_system.after(item_use_active_system),
                synergy_modifiers_system.after(blank_system),
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
