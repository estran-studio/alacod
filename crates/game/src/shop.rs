//! Monnaie au sol et boutique (M2-T6, chantier C5, `docs/conventions.md` §50).
//!
//! - **`shells`** : à la mort d'un ennemi, `EconomyConfig::shells_drop` `(min, max)` donne un tas
//!   de douilles ([`items::ShellPickup`]) à sa position. Tirage par un `RollbackRng` **local** de
//!   graine [`items::loot_seed`]`(partie, étage, "shells:<GgrsNetId du mort>")` : aucun flux de
//!   `RngStreams` n'est touché. [`shell_pickup_system`] les ramasse au contact (le plus petit
//!   `GgrsNetId` à portée) et crédite `Currency` (`CurrencyEvent`, raison `pickup`).
//! - **Présentoir** ([`items::Shelf`], neutre) : objet + prix, `Interactable { Shop }`.
//!   [`shelf_interaction_system`] débite `Currency` puis donne l'objet comme un ramassage
//!   (passif, actif, consommable). Solde insuffisant : refus (`CurrencyEvent { delta: 0 }`),
//!   anti-rebond `Shelf::retry_after`. Un présentoir vendu perd son `Interactable`.
//! - Les présentoirs d'une salle `boutique` sont posés par `map_ldtk::game::rooms::room_shop_system`
//!   ([`draw_shelves`] fait le tirage).

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use bevy_fixed::rng::{RollbackRng, RunSeed};
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackDespawnCommandExtension};
use items::{
    draw_loot, loot_seed, Inventory, ItemKind, ItemPicked, ItemTable, LootTables, Rarity, Shelf,
    ShellPickup,
};
use sim_core::frame_events::FrameEvents;
use sim_core::interaction::{Interactable, InteractionType};
use sim_core::modifier::Modifiers;
use sim_core::team::Team;
use std::collections::BTreeSet;
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter,
};

use crate::character::health::{Death, Health};
use crate::character::player::Player;
use crate::economy::EconomyConfig;
use crate::global_asset::GlobalAsset;
use crate::interaction::InteractionEvent;
use crate::items::{apply_instant_effects, spawn_item_pickup, with_inventory, FreshInventories};
use crate::loot::seed_context;
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;
use run::currency::{Currency, CurrencyEvent};

/// Prix d'un présentoir dont la rareté n'a pas de prix dans `economy.ron`.
pub const DEFAULT_SHELF_PRICE: u32 = 10;
/// Portée de ramassage des douilles (unités monde).
const SHELL_PICKUP_RANGE: i32 = 20;
/// Portée d'interaction d'un présentoir.
const SHELF_RANGE: i32 = 16;
/// Écart entre deux présentoirs d'une salle.
pub const SHELF_SPACING: i32 = 32;
/// Frames d'attente après un achat refusé.
pub const SHELF_RETRY_FRAMES: u32 = 30;

/// Réglage de scénario (`Scenario::shells_drop_override`), hors rollback.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ShopOverrides {
    pub shells_drop: Option<(u32, u32)>,
}

/// Prix d'un objet de rareté `rarity` selon l'économie du jeu.
pub fn shelf_price(economy: Option<&EconomyConfig>, rarity: Rarity) -> u32 {
    economy
        .and_then(|economy| economy.shop_prices.get(&rarity).copied())
        .unwrap_or(DEFAULT_SHELF_PRICE)
}

/// Fait apparaître un présentoir.
pub fn spawn_shelf(
    commands: &mut Commands,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    shelf: Shelf,
    position: fixed_math::FixedVec3,
) -> Entity {
    let transform = fixed_math::FixedTransform3D::new(
        position,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );
    let g_id = id_factory.next(format!("shelf_{}", shelf.key));
    commands
        .spawn((
            transform.to_bevy_transform(),
            transform,
            shelf,
            Interactable {
                interaction_range: Fixed::from_num(SHELF_RANGE),
                interaction_type: InteractionType::Shop,
            },
            g_id,
            Rollback,
        ))
        .id()
}

/// Tire les `count` objets d'une boutique dans `table` : un `RollbackRng` local de graine
/// `loot_seed(partie, étage, "shop:<salle>")`, indépendant du moment. Un objet déjà tiré est
/// retiré (au plus 8 essais par présentoir, le flux avance de la même façon dans tous les cas).
pub fn draw_shelves(
    table: &items::LootTableDef,
    items: &ItemTable,
    run_seed: u32,
    floor: u32,
    room: &str,
    count: u32,
) -> Vec<String> {
    let mut rng = RollbackRng::new(loot_seed(run_seed, floor, &format!("shop:{room}")));
    let mut out: Vec<String> = Vec::new();
    for _ in 0..count {
        for _ in 0..8 {
            let Some(item) = draw_loot(table, items, None, Rarity::Common, false, &mut rng) else {
                break;
            };
            if !out.contains(&item) {
                out.push(item);
                break;
            }
        }
    }
    out
}

/// Lâche des `shells` à la mort d'un ennemi (équipe `Enemies`, hors barils).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shell_drop_on_death_system(
    frame: Res<FrameCount>,
    overrides: Res<ShopOverrides>,
    global_assets: Res<GlobalAsset>,
    economies: Res<Assets<EconomyConfig>>,
    run_seed: Option<Res<RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut commands: Commands,
    dying: Query<
        (&GgrsNetId, &Team, &fixed_math::FixedTransform3D),
        (With<Rollback>, Added<Death>, Without<world::BarrelState>),
    >,
) {
    let drop = overrides.shells_drop.or_else(|| {
        global_assets
            .economy_config
            .as_ref()
            .and_then(|handle| economies.get(handle))
            .and_then(|economy| economy.shells_drop)
    });
    let Some((min, max)) = drop else {
        return;
    };
    let (run_seed, floor) = seed_context(&run_seed, &floor);
    for (net_id, team, transform) in order_iter!(dying) {
        if *team != Team::Enemies {
            continue;
        }
        let mut rng = RollbackRng::new(loot_seed(run_seed, floor, &format!("shells:{}", net_id.0)));
        let amount = rng.next_u32_range_inclusive(min, max.max(min));
        if amount == 0 {
            continue;
        }
        info!(
            "ggrs{{f={} shells_drop enemy_net_id={} amount={}}}",
            frame.frame, net_id.0, amount
        );
        spawn_shell_pickup(
            &mut commands,
            &mut id_factory,
            amount,
            transform.translation,
            net_id.0,
        );
    }
}

/// Fait apparaître un tas de douilles ; `origin` (id du mort ou clé de placement) donne la clé
/// de contenu du `GgrsNetId`.
pub fn spawn_shell_pickup(
    commands: &mut Commands,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    amount: u32,
    position: fixed_math::FixedVec3,
    origin: usize,
) -> Entity {
    let transform = fixed_math::FixedTransform3D::new(
        position,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );
    let g_id = id_factory.next(format!("shells_{origin}"));
    commands
        .spawn((
            transform.to_bevy_transform(),
            transform,
            ShellPickup { amount },
            g_id,
            Rollback,
        ))
        .id()
}

/// Ramassage au contact : le joueur de plus petit `GgrsNetId` à portée encaisse.
#[allow(clippy::type_complexity)]
pub fn shell_pickup_system(
    frame: Res<FrameCount>,
    mut commands: Commands,
    mut currency_events: ResMut<FrameEvents<CurrencyEvent>>,
    pickups: Query<
        (
            &GgrsNetId,
            Entity,
            &ShellPickup,
            &fixed_math::FixedTransform3D,
        ),
        With<Rollback>,
    >,
    mut players: Query<
        (
            &GgrsNetId,
            &Player,
            &fixed_math::FixedTransform3D,
            &mut Currency,
        ),
        Without<ShellPickup>,
    >,
) {
    if pickups.is_empty() {
        return;
    }
    let range = Fixed::from_num(SHELL_PICKUP_RANGE);
    let mut order: Vec<(usize, FixedVec2)> = players
        .iter()
        .map(|(id, _, t, _)| (id.0, t.translation.truncate()))
        .collect();
    order.sort_by_key(|(id, _)| *id);
    for (net_id, entity, pickup, transform) in order_iter!(pickups) {
        let position = transform.translation.truncate();
        let Some((player_id, _)) = order.iter().find(|(_, at)| position.distance(at) <= range)
        else {
            continue;
        };
        let Some((_, player, _, mut currency)) =
            players.iter_mut().find(|(id, ..)| id.0 == *player_id)
        else {
            continue;
        };
        currency.earn(pickup.amount);
        currency_events.send(CurrencyEvent {
            handle: player.handle,
            delta: pickup.amount as i64,
            reason: "pickup".to_string(),
        });
        info!(
            "ggrs{{f={} shells_pickup net_id={} amount={} by={}}}",
            frame.frame, net_id.0, pickup.amount, player_id
        );
        commands.entity(entity).despawn_rollback();
    }
}

/// Achat sur un présentoir : le premier joueur (ordre `GgrsNetId`) l'emporte.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shelf_interaction_system(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    table: Res<ItemTable>,
    mut commands: Commands,
    mut picked: ResMut<FrameEvents<ItemPicked>>,
    mut currency_events: ResMut<FrameEvents<CurrencyEvent>>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut shelves: Query<&mut Shelf, With<Rollback>>,
    mut players: Query<
        (
            &GgrsNetId,
            Entity,
            &Player,
            &fixed_math::FixedTransform3D,
            Option<&mut Inventory>,
            &mut Modifiers,
            Option<&mut Health>,
            &mut Currency,
        ),
        Without<Shelf>,
    >,
) {
    let mut fresh = FreshInventories::new();
    let mut handled: BTreeSet<usize> = BTreeSet::new();
    for event in events.iter() {
        if event.interaction_type != InteractionType::Shop
            || handled.contains(&event.interactable_net_id.0)
        {
            continue;
        }
        let Ok(mut shelf) = shelves.get_mut(event.interactable) else {
            continue;
        };
        if shelf.sold || frame.frame < shelf.retry_after {
            continue;
        }
        let Some(def) = table.0.get(&shelf.item_id) else {
            continue;
        };
        let Some((
            _,
            entity,
            player,
            player_transform,
            inventory,
            mut modifiers,
            health,
            mut wallet,
        )) = players
            .iter_mut()
            .find(|(id, ..)| id.0 == event.interactor_net_id.0)
        else {
            continue;
        };
        handled.insert(event.interactable_net_id.0);
        if !wallet.spend(shelf.price) {
            shelf.retry_after = frame.frame + SHELF_RETRY_FRAMES;
            info!(
                "ggrs{{f={} shop_refused net_id={} by={} price={} balance={}}}",
                frame.frame,
                event.interactable_net_id.0,
                event.interactor_net_id.0,
                shelf.price,
                wallet.0
            );
            currency_events.send(CurrencyEvent {
                handle: player.handle,
                delta: 0,
                reason: "shop_refused".to_string(),
            });
            continue;
        }
        currency_events.send(CurrencyEvent {
            handle: player.handle,
            delta: -(shelf.price as i64),
            reason: "shop".to_string(),
        });
        shelf.sold = true;
        let item_id = shelf.item_id.clone();
        commands.entity(event.interactable).remove::<Interactable>();

        // Même résultat qu'un ramassage (`items::handle_item_interaction` et
        // `item_consumable_pickup_system`).
        let mut dropped: Option<String> = None;
        with_inventory(
            inventory,
            &mut fresh,
            event.interactor_net_id.0,
            entity,
            |inventory| match def.kind {
                ItemKind::Passive => {
                    inventory.passives.push(item_id.clone());
                    for modifier in &def.modifiers {
                        modifiers.push(modifier.to_modifier(&item_id));
                    }
                }
                ItemKind::Active(_) => {
                    dropped = inventory.active.take().map(|slot| slot.item);
                    inventory.active = Some(items::ActiveSlot::new(item_id.clone()));
                }
                ItemKind::Consumable => {
                    if def.effects.is_empty() {
                        *inventory.consumables.entry(item_id.clone()).or_default() += 1;
                    }
                }
            },
        );
        if def.kind == ItemKind::Consumable && !def.effects.is_empty() {
            apply_instant_effects(
                def,
                &item_id,
                frame.frame,
                health,
                &mut modifiers,
                &mut commands,
                entity,
            );
        }
        if let Some(old) = dropped {
            spawn_item_pickup(
                &mut commands,
                &table,
                old,
                player_transform.translation,
                &mut id_factory,
            );
        }
        info!(
            "ggrs{{f={} shop_buy net_id={} item={} price={} by={}}}",
            frame.frame,
            event.interactable_net_id.0,
            item_id,
            shelf.price,
            event.interactor_net_id.0
        );
        picked.send(ItemPicked {
            frame: frame.frame,
            player_handle: player.handle,
            item_id,
        });
    }
    crate::items::insert_fresh_inventories(&mut commands, fresh);
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopOverrides>()
            .rollback_and_trace_neutral::<Shelf>()
            .rollback_and_trace_neutral::<ShellPickup>();
        app.add_systems(
            GgrsSchedule,
            shelf_interaction_system
                .after(crate::loot::chest_interaction_system)
                .before(RollbackSystemSet::Movement)
                .in_set(RollbackSystemSet::Interaction),
        );
        app.add_systems(
            GgrsSchedule,
            shell_pickup_system
                .after(crate::items::item_consumable_pickup_system)
                .in_set(RollbackSystemSet::Effects),
        );
        app.add_systems(
            GgrsSchedule,
            shell_drop_on_death_system
                // Après les autres drops : ils consomment tous `GgrsNetIdFactory`, les ids
                // existants ne changent pas.
                .after(crate::powerups::loot_drop_on_death_system)
                .after(crate::progression::weapon_drop_on_death_system)
                .after(crate::character::health::rollback_apply_accumulated_damage)
                .before(crate::character::health::rollback_apply_death)
                .in_set(RollbackSystemSet::DeathManagement),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use items::{ItemDef, LootEntry, LootPick, LootTableDef};

    fn item(rarity: Rarity) -> ItemDef {
        ItemDef {
            name: String::new(),
            kind: ItemKind::Passive,
            modifiers: vec![],
            effects: vec![],
            tags: vec![],
            rarity,
            pickup_range: Fixed::from_num(24),
            blank: None,
            uses_per_floor: None,
            cost: None,
            projectile: None,
        }
    }

    fn fixtures() -> (LootTableDef, ItemTable) {
        let table = LootTableDef {
            entries: vec![LootEntry {
                pick: LootPick::Rarity(Rarity::Common),
                weight: 1,
            }],
            exclude_held: false,
            reward: None,
        };
        let items = ItemTable(
            ["a", "b", "c", "d", "e"]
                .into_iter()
                .map(|id| (id.to_string(), item(Rarity::Common)))
                .collect(),
        );
        (table, items)
    }

    #[test]
    fn same_seed_same_shelves_without_duplicates() {
        let (table, items) = fixtures();
        let first = draw_shelves(&table, &items, 7, 0, "room_a", 3);
        assert_eq!(first, draw_shelves(&table, &items, 7, 0, "room_a", 3));
        assert_eq!(first.len(), 3);
        let unique: BTreeSet<_> = first.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn other_room_or_seed_changes_the_draw_stream() {
        let (table, items) = fixtures();
        let base = draw_shelves(&table, &items, 7, 0, "room_a", 5);
        let others = [
            draw_shelves(&table, &items, 8, 0, "room_a", 5),
            draw_shelves(&table, &items, 7, 0, "room_b", 5),
            draw_shelves(&table, &items, 7, 1, "room_a", 5),
        ];
        assert!(others.iter().any(|other| *other != base));
    }

    #[test]
    fn price_falls_back_to_default() {
        assert_eq!(shelf_price(None, Rarity::Rare), DEFAULT_SHELF_PRICE);
        let mut economy = EconomyConfig::default();
        economy.shop_prices.insert(Rarity::Rare, 50);
        assert_eq!(shelf_price(Some(&economy), Rarity::Rare), 50);
        assert_eq!(
            shelf_price(Some(&economy), Rarity::Common),
            DEFAULT_SHELF_PRICE
        );
    }
}
