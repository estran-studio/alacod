//! Butin, coffres et clés (M2-T5, chantier C4, `docs/conventions.md` §48).
//!
//! - **Tirage** (`items::draw_loot`) : un `RollbackRng` **local**, de graine
//!   [`items::loot_seed`]`(graine de la partie, étage, clé de contenu)`. Il ne touche aucun flux
//!   de `RngStreams` (le flux `loot` des power-ups, consommé à chaque mort, ferait dépendre
//!   l'objet du moment de l'ouverture) : un coffre ouvert tôt ou tard, par l'un ou l'autre joueur,
//!   donne le même objet. La clé d'un coffre est sa clé de contenu (`room:<salle>`,
//!   `reward:<salle>`, ou celle d'un placement de scénario), jamais son `Entity` ni son id réseau.
//! - **Coffre** ([`items::Chest`], rollback neutre) : interactable `Chest`. Verrouillé, il consomme
//!   une `key` ; sans clé l'ouverture est refusée (prompt HUD « Fermé — 1 clé »). Ouvert, il pose un
//!   `ItemPickup` à côté, perd son `Interactable` et émet `FrameEvents<ChestOpened>`.
//! - **Récompense de salle** ([`room_reward_system`]) : une salle `combat` qui passe `Cleared`
//!   laisse, selon la chance du pool (`LootTableDef::reward`), un consommable ou un coffre au
//!   centre de la salle.
//! - **Pool d'un étage** ([`FloorLootTables`]) : liste de la séquence `Floors` (`loot`).

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed};
use bevy_fixed::rng::{RollbackRng, RunSeed};
use bevy_ggrs::{GgrsSchedule, Rollback};
use items::{
    draw_loot, loot_seed, Chest, ChestOpened, Inventory, ItemTable, LootTables,
    DEFAULT_CHEST_TABLE, DEFAULT_REWARD_TABLE,
};
use map::game::entity::map::{level_id::LevelId, room::RoomBounds};
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::interaction::{Interactable, InteractionType};
use std::collections::BTreeSet;
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
};
use world::{RoomChanged, RoomState};

use crate::character::player::Player;
use crate::interaction::InteractionEvent;
use crate::items::spawn_item_pickup;
use crate::rollback::RollbackTraceApp;
use crate::system_set::RollbackSystemSet;

/// Consommable qui ouvre un coffre verrouillé.
pub const KEY_ITEM: &str = "key";
/// Portée d'interaction d'un coffre (unités monde).
const CHEST_RANGE: i32 = 32;
/// Écart (unités monde) entre un coffre et l'objet qu'il laisse.
const DROP_OFFSET: i32 = 16;

/// Pool de butin de chaque étage (index = étage, le dernier vaut pour les suivants), posé par
/// `map_ldtk` depuis la séquence `Floors` ; vide sinon. Hors rollback (contenu).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct FloorLootTables(pub Vec<String>);

impl FloorLootTables {
    pub fn for_floor(&self, floor: u32) -> Option<&str> {
        let last = self.0.len().checked_sub(1)?;
        Some(self.0[(floor as usize).min(last)].as_str())
    }
}

/// Réglages de scénario (`Scenario::room_reward_chance_override`), hors rollback.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct LootOverrides {
    /// Force `RoomRewardDef::chance` de tous les pools.
    pub reward_chance: Option<Fixed>,
}

/// Contexte commun des tirages : graine de la partie et étage courant.
fn seed_context(
    run_seed: &Option<Res<RunSeed>>,
    floor: &Option<Res<run::FloorState>>,
) -> (u32, u32) {
    (
        run_seed.as_ref().map_or(0, |seed| seed.0),
        floor.as_ref().map_or(0, |floor| floor.index),
    )
}

/// Fait apparaître un coffre.
pub fn spawn_chest(
    commands: &mut Commands,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
    chest: Chest,
    position: fixed_math::FixedVec3,
) -> Entity {
    let transform = fixed_math::FixedTransform3D::new(
        position,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::ONE,
    );
    let g_id = id_factory.next(format!("chest_{}", chest.key));
    commands
        .spawn((
            transform.to_bevy_transform(),
            transform,
            chest,
            Interactable {
                interaction_range: Fixed::from_num(CHEST_RANGE),
                interaction_type: InteractionType::Chest,
            },
            g_id,
            Rollback,
        ))
        .id()
}

/// Le pool du coffre : le sien, sinon celui de l'étage, sinon `commun`.
fn chest_table_id<'a>(chest: &'a Chest, floors: &'a FloorLootTables, floor: u32) -> &'a str {
    chest
        .table
        .as_deref()
        .or_else(|| floors.for_floor(floor))
        .unwrap_or(DEFAULT_CHEST_TABLE)
}

/// Ouverture d'un coffre par Interaction : le premier joueur (ordre `GgrsNetId`) l'emporte.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn chest_interaction_system(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    items: Res<ItemTable>,
    tables: Res<LootTables>,
    floors: Res<FloorLootTables>,
    run_seed: Option<Res<RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut opened: ResMut<FrameEvents<ChestOpened>>,
    mut chests: Query<(&mut Chest, &fixed_math::FixedTransform3D), With<Rollback>>,
    mut players: Query<(&GgrsNetId, &Player, Option<&mut Inventory>), Without<Chest>>,
) {
    let (run_seed, floor) = seed_context(&run_seed, &floor);
    let mut taken: BTreeSet<usize> = BTreeSet::new();
    for event in events.iter() {
        if event.interaction_type != InteractionType::Chest
            || taken.contains(&event.interactable_net_id.0)
        {
            continue;
        }
        let Ok((mut chest, transform)) = chests.get_mut(event.interactable) else {
            continue;
        };
        if chest.opened {
            continue;
        }
        let Some((_, player, mut inventory)) = players
            .iter_mut()
            .find(|(id, ..)| id.0 == event.interactor_net_id.0)
        else {
            continue;
        };
        if chest.locked
            && !inventory
                .as_deref()
                .is_some_and(|inventory| inventory.consumable(KEY_ITEM) > 0)
        {
            info!(
                "ggrs{{f={} chest_refused net_id={} by={}}}",
                frame.frame, event.interactable_net_id.0, event.interactor_net_id.0
            );
            continue;
        }
        let table_id = chest_table_id(&chest, &floors, floor).to_string();
        let Some(table) = tables.0.get(&table_id) else {
            warn!(
                "coffre « {} » : pool de butin « {table_id} » inconnu",
                chest.key
            );
            continue;
        };
        let mut rng = RollbackRng::new(loot_seed(run_seed, floor, &chest.key));
        let held = inventory.as_deref();
        let Some(item_id) = draw_loot(table, &items, held, chest.rarity, false, &mut rng) else {
            warn!(
                "coffre « {} » : le pool « {table_id} » ne donne rien",
                chest.key
            );
            continue;
        };
        taken.insert(event.interactable_net_id.0);
        if chest.locked {
            if let Some(stock) = inventory
                .as_deref_mut()
                .and_then(|inventory| inventory.consumables.get_mut(KEY_ITEM))
            {
                *stock -= 1;
            }
        }
        chest.opened = true;
        commands.entity(event.interactable).remove::<Interactable>();
        let mut position = transform.translation;
        position.x += Fixed::from_num(DROP_OFFSET);
        spawn_item_pickup(
            &mut commands,
            &items,
            item_id.clone(),
            position,
            &mut id_factory,
        );
        info!(
            "ggrs{{f={} chest_open net_id={} key={} item={} by={}}}",
            frame.frame, event.interactable_net_id.0, chest.key, item_id, event.interactor_net_id.0
        );
        opened.send(ChestOpened {
            frame: frame.frame,
            player_handle: player.handle,
            chest_key: chest.key.clone(),
            item_id,
        });
    }
}

/// Récompense d'une salle `combat` nettoyée.
#[allow(clippy::too_many_arguments)]
pub fn room_reward_system(
    changed: Res<FrameEvents<RoomChanged>>,
    items: Res<ItemTable>,
    tables: Res<LootTables>,
    floors: Res<FloorLootTables>,
    overrides: Res<LootOverrides>,
    run_seed: Option<Res<RunSeed>>,
    floor: Option<Res<run::FloorState>>,
    rooms: Query<(&LevelId, &RoomBounds)>,
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    frame: Res<FrameCount>,
) {
    let (run_seed, floor) = seed_context(&run_seed, &floor);
    for event in changed.iter() {
        if event.to != RoomState::Cleared || event.kind != "combat" {
            continue;
        }
        // Le pool de l'étage s'il définit une récompense, sinon `salle`.
        let floor_table = floors
            .for_floor(floor)
            .and_then(|id| tables.0.get(id))
            .filter(|table| table.reward.is_some());
        let Some(table) = floor_table.or_else(|| tables.0.get(DEFAULT_REWARD_TABLE)) else {
            continue;
        };
        let Some(reward) = table.reward else {
            continue;
        };
        let mut candidates: Vec<_> = rooms.iter().filter(|(id, _)| id.0 == event.room).collect();
        candidates.sort_by_key(|(_, bounds)| (bounds.position.x, bounds.position.y));
        let Some((_, bounds)) = candidates.first() else {
            continue;
        };
        let two = Fixed::from_num(2);
        let center = fixed_math::FixedVec3::new(
            bounds.position.x + bounds.size.x / two,
            bounds.position.y + bounds.size.y / two,
            fixed_math::FIXED_ZERO,
        );
        let key = format!("reward:{}", event.room);
        let mut rng = RollbackRng::new(loot_seed(run_seed, floor, &key));
        let chance = overrides.reward_chance.unwrap_or(reward.chance);
        if rng.next_fixed() >= chance {
            continue;
        }
        let want_chest = rng.next_fixed() < reward.chest_chance;
        let consumable = if want_chest {
            None
        } else {
            draw_loot(table, &items, None, items::Rarity::Common, true, &mut rng)
        };
        match consumable {
            Some(item_id) => {
                info!(
                    "ggrs{{f={} room_reward room={} item={}}}",
                    frame.frame, event.room, item_id
                );
                spawn_item_pickup(&mut commands, &items, item_id, center, &mut id_factory);
            }
            None => {
                info!(
                    "ggrs{{f={} room_reward room={} chest}}",
                    frame.frame, event.room
                );
                spawn_chest(
                    &mut commands,
                    &mut id_factory,
                    Chest {
                        key,
                        table: None,
                        rarity: reward.chest_rarity,
                        locked: false,
                        opened: false,
                    },
                    center,
                );
            }
        }
    }
}

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LootTables>()
            .init_resource::<FloorLootTables>()
            .init_resource::<LootOverrides>()
            .add_frame_events_neutral::<ChestOpened>()
            .rollback_and_trace_neutral::<Chest>();
        app.add_systems(
            GgrsSchedule,
            chest_interaction_system
                .after(crate::items::handle_item_interaction)
                .before(RollbackSystemSet::Movement)
                .in_set(RollbackSystemSet::Interaction),
        );
        app.add_systems(
            GgrsSchedule,
            room_reward_system
                .after(RollbackSystemSet::Run)
                .before(RollbackSystemSet::FrameCounter),
        );
    }
}
