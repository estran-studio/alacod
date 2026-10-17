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
use combat::actors::ExtraHealth;
use items::{
    draw_loot, loot_seed, Inventory, ItemKind, ItemPicked, ItemTable, LootTables, Rarity, Shelf,
    ShellPickup,
};
use sim_core::frame_events::FrameEvents;
use sim_core::interaction::{Interactable, InteractionType};
use sim_core::modifier::{Modifier, ModifierOp, ModifierSource, Modifiers};
use sim_core::stats::StatId;
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
use crate::loot::{assign_slot, seed_context, SlotContext};
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;
use crate::weapons::WeaponsConfig;
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
    weapon_shelves: u32,
) -> Vec<String> {
    let mut rng = RollbackRng::new(loot_seed(run_seed, floor, &format!("shop:{room}")));
    let mut out: Vec<String> = Vec::new();
    // D62 : les premiers présentoirs ne tirent que des armes (au moins une par boutique).
    let weapons_only = table.weapons_only();
    for shelf in 0..count {
        let table = if shelf < weapon_shelves && !weapons_only.entries.is_empty() {
            &weapons_only
        } else {
            table
        };
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

/// Les `count` objets d'une boutique dont le pool s'épuise, assignés à la **génération** de
/// l'étage (M3-T8, §73) : un emplacement par présentoir (`shelf:<salle>:<i>`), tiré sur sa propre
/// graine avec l'index d'épuisement du pool, donc sans doublon dans la boutique ni avec une autre
/// salle tant que le pool n'est pas vide. `None` si le pool n'a pas `exhaust` : l'appelant garde
/// [`draw_shelves`] (comportement d'avant, un seul flux par boutique).
pub fn assign_shelves(
    ctx: &SlotContext,
    pools: &mut items::PoolState,
    table_id: &str,
    room: &str,
    count: u32,
    weapon_shelves: u32,
) -> Option<Vec<String>> {
    if !ctx.tables.0.get(table_id)?.exhaust {
        return None;
    }
    Some(
        (0..count)
            .filter_map(|shelf| {
                assign_slot(
                    ctx,
                    pools,
                    table_id,
                    Rarity::Common,
                    shelf < weapon_shelves,
                    &format!("shelf:{room}:{shelf}"),
                )
            })
            .collect(),
    )
}

/// Ce qu'un paiement par [`Shelf::cost`] a retiré au payeur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payment {
    /// Douilles débitées (0 hors prix en douilles).
    pub coins: i64,
    /// Points de cœur d'âme retirés.
    pub soul_spent: Fixed,
    /// Vie maximale (et courante) perdue, à rendre **permanente** par un modificateur de `MaxHealth`.
    pub max_health_lost: Fixed,
}

/// Paie `cost` : `Price::pay` sur les valeurs du payeur, écrites **seulement** si le paiement
/// réussit (refus : rien ne bouge). Sans `Inventory` un prix en consommable est refusé ; sans
/// `Health` un prix de vie l'est aussi (sauf l'âme qui le couvre entièrement).
pub fn pay_cost(
    cost: &items::Price,
    wallet: &mut Currency,
    inventory: Option<&mut Inventory>,
    health: Option<&mut Health>,
    soul: Option<&mut ExtraHealth>,
) -> Option<Payment> {
    let mut scratch = Inventory::default();
    let inventory = inventory.unwrap_or(&mut scratch);
    let (mut max_health, mut current) = health
        .as_ref()
        .map_or((Fixed::ZERO, Fixed::ZERO), |h| (h.max, h.current));
    let mut soul_points = soul.as_ref().map_or(Fixed::ZERO, |s| s.soul);
    let (max_before, soul_before, coins_before) = (max_health, soul_points, wallet.0);
    let mut coins = wallet.0;
    if !cost.pay(
        &mut coins,
        inventory,
        &mut max_health,
        &mut current,
        &mut soul_points,
    ) {
        return None;
    }
    wallet.0 = coins;
    if let Some(health) = health {
        health.max = max_health;
        health.current = current;
    }
    if let Some(soul) = soul {
        soul.soul = soul_points;
    }
    Some(Payment {
        coins: i64::from(coins_before - coins),
        soul_spent: soul_before - soul_points,
        max_health_lost: max_before - max_health,
    })
}

/// Lâche des `shells` à la mort d'un ennemi (équipe `Enemies`, hors barils).
///
/// `With<Death>`, pas `Added<Death>` : `rollback_apply_death` détruit toute entité morte dans la
/// même frame, donc chaque mort est vue une fois ; `Added` ne survit pas à un rollback quand la mort
/// est posée après ce système (fin d'arène, salle du boss nettoyée) : la frame rejouée depuis l'état
/// restauré ne la voit plus « ajoutée » (desync de la campagne `etages_deux`, 2026-10-15).
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
        (With<Rollback>, With<Death>, Without<world::BarrelState>),
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
    global_assets: Res<GlobalAsset>,
    weapons: Res<Assets<WeaponsConfig>>,
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
            Option<&mut ExtraHealth>,
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
        let weapon = items::loot_weapon(&shelf.item_id).map(str::to_string);
        let def = table.0.get(&shelf.item_id);
        if weapon.is_none() && def.is_none() {
            continue;
        }
        let Some((
            _,
            entity,
            player,
            player_transform,
            mut inventory,
            mut modifiers,
            mut health,
            mut wallet,
            mut soul,
        )) = players
            .iter_mut()
            .find(|(id, ..)| id.0 == event.interactor_net_id.0)
        else {
            continue;
        };
        handled.insert(event.interactable_net_id.0);
        // M3-T3 (§68) : un consommable que le joueur ne peut pas prendre (compteur au plafond,
        // soin à pleine vie) n'est pas vendu : rien n'est dépensé, le présentoir reste.
        if let Some(def) = def.filter(|def| def.kind == ItemKind::Consumable) {
            let stock = match inventory.as_deref() {
                Some(inventory) => inventory.consumable(&shelf.item_id),
                None => fresh
                    .get(&event.interactor_net_id.0)
                    .map_or(0, |(_, fresh)| fresh.consumable(&shelf.item_id)),
            };
            let full = health
                .as_deref()
                .is_some_and(|health| health.current >= health.max);
            if !def.accepts(stock, full) {
                shelf.retry_after = frame.frame + SHELF_RETRY_FRAMES;
                info!(
                    "ggrs{{f={} shop_full net_id={} by={} item={}}}",
                    frame.frame,
                    event.interactable_net_id.0,
                    event.interactor_net_id.0,
                    shelf.item_id
                );
                currency_events.send(CurrencyEvent {
                    handle: player.handle,
                    delta: 0,
                    reason: "shop_full".to_string(),
                });
                continue;
            }
        }
        // M3-T8 (§73, Q6) : un présentoir à `cost` (vie maximale, consommable) se paie par
        // `Price::pay`, les cœurs d'âme d'abord ; sans `cost`, le prix en douilles d'avant.
        let mut max_health_lost = Fixed::ZERO;
        let mut paid_note = String::new();
        if let Some(cost) = shelf.cost.clone() {
            let Some(payment) = pay_cost(
                &cost,
                &mut wallet,
                inventory.as_deref_mut(),
                health.as_deref_mut(),
                soul.as_deref_mut(),
            ) else {
                shelf.retry_after = frame.frame + SHELF_RETRY_FRAMES;
                info!(
                    "ggrs{{f={} shop_refused net_id={} by={} cost={:?} balance={}}}",
                    frame.frame,
                    event.interactable_net_id.0,
                    event.interactor_net_id.0,
                    cost,
                    wallet.0
                );
                currency_events.send(CurrencyEvent {
                    handle: player.handle,
                    delta: 0,
                    reason: "shop_refused".to_string(),
                });
                continue;
            };
            if payment.coins > 0 {
                currency_events.send(CurrencyEvent {
                    handle: player.handle,
                    delta: -payment.coins,
                    reason: "shop".to_string(),
                });
            }
            max_health_lost = payment.max_health_lost;
            paid_note = format!(
                " soul={} max_health_lost={}",
                payment.soul_spent, payment.max_health_lost
            );
        } else {
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
        }
        // La vie maximale perdue l'est **définitivement** : `sync_health_from_stats` réécrit
        // `Health.max` depuis la stat chaque frame, donc le prix est un modificateur permanent.
        if max_health_lost > Fixed::ZERO {
            modifiers.push(Modifier {
                stat: StatId::MaxHealth,
                op: ModifierOp::Add,
                value: -max_health_lost,
                source: ModifierSource::Named(format!("price:{}", shelf.key)),
                until: None,
            });
        }
        shelf.sold = true;
        let item_id = shelf.item_id.clone();
        commands.entity(event.interactable).remove::<Interactable>();

        // D62 : une arme achetée tombe au pied du joueur, ramassée ensuite par l'interaction
        // habituelle (échange d'emplacement compris).
        if let Some(weapon_id) = weapon {
            crate::loot::spawn_loot_weapon(
                &mut commands,
                &global_assets,
                &weapons,
                &weapon_id,
                player_transform.translation,
                &mut id_factory,
            );
            info!(
                "ggrs{{f={} shop_buy net_id={} item={} price={} by={}{}}}",
                frame.frame,
                event.interactable_net_id.0,
                item_id,
                shelf.price,
                event.interactor_net_id.0,
                paid_note
            );
            continue;
        }
        let Some(def) = def else {
            continue;
        };

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
            "ggrs{{f={} shop_buy net_id={} item={} price={} by={}{}}}",
            frame.frame,
            event.interactable_net_id.0,
            item_id,
            shelf.price,
            event.interactor_net_id.0,
            paid_note
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
            familiar: None,
            max_stack: None,
            refuse_when_full: false,
            stack: Default::default(),
        }
    }

    fn fixtures() -> (LootTableDef, ItemTable) {
        let table = LootTableDef {
            entries: vec![LootEntry {
                pick: LootPick::Rarity(Rarity::Common),
                weight: 1,
            }],
            exclude_held: false,
            exhaust: false,
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
        let first = draw_shelves(&table, &items, 7, 0, "room_a", 3, 0);
        assert_eq!(first, draw_shelves(&table, &items, 7, 0, "room_a", 3, 0));
        assert_eq!(first.len(), 3);
        let unique: BTreeSet<_> = first.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn other_room_or_seed_changes_the_draw_stream() {
        let (table, items) = fixtures();
        let base = draw_shelves(&table, &items, 7, 0, "room_a", 5, 0);
        let others = [
            draw_shelves(&table, &items, 8, 0, "room_a", 5, 0),
            draw_shelves(&table, &items, 7, 0, "room_b", 5, 0),
            draw_shelves(&table, &items, 7, 1, "room_a", 5, 0),
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

    fn health(current: i32, max: i32) -> Health {
        Health {
            current: Fixed::from_num(current),
            max: Fixed::from_num(max),
            invulnerable_until_frame: None,
        }
    }

    fn heart() -> items::Price {
        items::Price::MaxHealth(Fixed::from_num(2))
    }

    /// Q6 : un cœur en vie maximale baisse la vie courante **et** maximale, rien d'autre.
    #[test]
    fn a_heart_price_lowers_current_and_max_health() {
        let (mut wallet, mut hp) = (Currency::new(7), health(6, 6));
        let payment = pay_cost(&heart(), &mut wallet, None, Some(&mut hp), None).unwrap();
        assert_eq!(
            payment,
            Payment {
                coins: 0,
                soul_spent: Fixed::ZERO,
                max_health_lost: Fixed::from_num(2)
            }
        );
        assert_eq!((hp.current, hp.max), (Fixed::from_num(4), Fixed::from_num(4)));
        assert_eq!(wallet.0, 7, "les douilles ne bougent pas");
    }

    /// Q6 : à 1 PV (ou 2 : le paiement tuerait) le marché refuse et rien ne change.
    #[test]
    fn a_lethal_heart_price_is_refused_and_nothing_moves() {
        for current in [1, 2] {
            let (mut wallet, mut hp) = (Currency::new(7), health(current, 6));
            assert_eq!(
                pay_cost(&heart(), &mut wallet, None, Some(&mut hp), None),
                None
            );
            assert_eq!(
                (hp.current, hp.max),
                (Fixed::from_num(current), Fixed::from_num(6))
            );
        }
        // Sans santé du tout (joueur sans `Health`) : refus aussi.
        let mut wallet = Currency::new(7);
        assert_eq!(pay_cost(&heart(), &mut wallet, None, None, None), None);
    }

    /// Q6 : les cœurs d'âme paient à la place, et sauvent un payeur à 1 PV.
    #[test]
    fn soul_hearts_pay_instead_of_max_health() {
        let (mut wallet, mut hp, mut soul) = (
            Currency::new(0),
            health(1, 6),
            ExtraHealth {
                soul: Fixed::from_num(4),
            },
        );
        let payment =
            pay_cost(&heart(), &mut wallet, None, Some(&mut hp), Some(&mut soul)).unwrap();
        assert_eq!(payment.soul_spent, Fixed::from_num(2));
        assert_eq!(payment.max_health_lost, Fixed::ZERO);
        assert_eq!(soul.soul, Fixed::from_num(2));
        assert_eq!((hp.current, hp.max), (Fixed::ONE, Fixed::from_num(6)));
        // Âme insuffisante : elle part toute, le reste en vie maximale.
        let big = items::Price::MaxHealth(Fixed::from_num(5));
        let mut hp = health(6, 6);
        let payment = pay_cost(&big, &mut wallet, None, Some(&mut hp), Some(&mut soul)).unwrap();
        assert_eq!(payment.soul_spent, Fixed::from_num(2));
        assert_eq!(payment.max_health_lost, Fixed::from_num(3));
        assert_eq!(soul.soul, Fixed::ZERO);
        assert_eq!((hp.current, hp.max), (Fixed::from_num(3), Fixed::from_num(3)));
    }

    #[test]
    fn a_consumable_price_needs_the_stock_and_spends_it() {
        let cost = items::Price::Consumable("key".into(), 2);
        let mut wallet = Currency::new(0);
        assert_eq!(pay_cost(&cost, &mut wallet, None, None, None), None);
        let mut inventory = Inventory::default();
        inventory.consumables.insert("key".into(), 1);
        assert_eq!(
            pay_cost(&cost, &mut wallet, Some(&mut inventory), None, None),
            None
        );
        assert_eq!(inventory.consumable("key"), 1, "refus : le stock est intact");
        inventory.consumables.insert("key".into(), 3);
        assert!(pay_cost(&cost, &mut wallet, Some(&mut inventory), None, None).is_some());
        assert_eq!(inventory.consumable("key"), 1);
    }

    /// Un prix en douilles par `cost` passe par le même chemin que `price` et rend les douilles.
    #[test]
    fn a_currency_cost_reports_the_coins_spent() {
        let mut wallet = Currency::new(50);
        let payment =
            pay_cost(&items::Price::Currency(20), &mut wallet, None, None, None).unwrap();
        assert_eq!(payment.coins, 20);
        assert_eq!(wallet.0, 30);
    }

    fn slot_context<'a>(
        tables: &'a LootTables,
        items: &'a ItemTable,
        unlocks: &'a items::UnlockTable,
        active: &'a items::RunUnlocks,
    ) -> SlotContext<'a> {
        SlotContext {
            frame: 0,
            tables,
            items,
            unlocks,
            active,
            run_seed: 9,
            floor: 0,
        }
    }

    /// M3-T8 : les présentoirs d'un pool qui s'épuise sont assignés à la génération, sans doublon
    /// dans la boutique ni avec la salle suivante ; un pool sans `exhaust` garde `draw_shelves`.
    #[test]
    fn exhausting_shelves_never_repeat_across_rooms() {
        let (mut table, item_table) = fixtures();
        table.exhaust = true;
        let tables = LootTables([("p".to_string(), table.clone())].into());
        let (unlocks, active) = Default::default();
        let ctx = slot_context(&tables, &item_table, &unlocks, &active);
        let mut pools = items::PoolState::default();
        let first = assign_shelves(&ctx, &mut pools, "p", "room_a", 2, 0).unwrap();
        let second = assign_shelves(&ctx, &mut pools, "p", "room_b", 3, 0).unwrap();
        let all: BTreeSet<_> = first.iter().chain(&second).collect();
        assert_eq!(all.len(), 5, "cinq présentoirs, cinq objets : {first:?} {second:?}");
        assert_eq!(pools.count("p"), 5);
        // Rejouer la génération de la même salle rend les mêmes objets sans rien consommer.
        assert_eq!(
            assign_shelves(&ctx, &mut pools, "p", "room_a", 2, 0).unwrap(),
            first
        );
        assert_eq!(pools.count("p"), 5);
        // Sans `exhaust` : `None`, l'appelant tire comme avant et rien n'est écrit.
        let plain = LootTables([("p".to_string(), fixtures().0)].into());
        let ctx = slot_context(&plain, &item_table, &unlocks, &active);
        let mut pools = items::PoolState::default();
        assert_eq!(assign_shelves(&ctx, &mut pools, "p", "room_a", 2, 0), None);
        assert_eq!(pools, items::PoolState::default());
    }
}
