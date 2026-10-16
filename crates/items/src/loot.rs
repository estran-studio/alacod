//! Butin, coffres et clés (M2-T5, `docs/conventions.md` §48) : contrats sans dépendance au rendu.
//!
//! - [`LootTableDef`] : un pool de `loot/<id>.ron` (kind `LootTable`) : des entrées pondérées
//!   (un objet précis ou une rareté), et, pour la récompense de salle, une chance
//!   ([`RoomRewardDef`]).
//! - [`draw_loot`] : tirage pur et déterministe. Il lit un [`RollbackRng`] **local**, dont la
//!   graine ne dépend que de la partie, de l'étage et de la clé de contenu du coffre ou de la
//!   salle ([`loot_seed`]) : le moment de l'ouverture ne change pas l'objet, et aucun flux du
//!   gameplay (`RngStreams`) n'est touché.
//! - [`Chest`] : coffre au sol (rollback, **neutre**) ; [`ChestOpened`] : `FrameEvents` neutre.

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use bevy_fixed::rng::{fnv1a, RollbackRng};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::{Inventory, ItemKind, ItemTable, Rarity, RunUnlocks, UnlockTable};

/// Ce qu'une entrée de table désigne.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LootPick {
    /// Un objet précis (fichier de `items/`).
    Item(String),
    /// Un objet de cette rareté, tiré uniformément parmi ceux de la table d'objets.
    Rarity(Rarity),
}

/// Une entrée pondérée. Un poids ≤ 0 est refusé par le lint.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootEntry {
    pub pick: LootPick,
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub weight: i32,
}

/// Récompense d'une salle de combat nettoyée (`Cleared`) tirée dans cette table.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomRewardDef {
    /// Chance (0 à 1) qu'une salle nettoyée laisse une récompense.
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    pub chance: Fixed,
    /// Si une récompense tombe : chance (0 à 1) que ce soit un coffre plutôt qu'un
    /// consommable de la table.
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    pub chest_chance: Fixed,
    /// Rareté du coffre posé.
    #[serde(default)]
    pub chest_rarity: Rarity,
}

/// Coffre posé par un type de salle (`rooms/<id>.ron`, champ `chest`) : une fois par salle de ce
/// type, au centre, dès que l'étage la présente.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomChestDef {
    /// Pool du coffre ; `None` : celui de l'étage, sinon [`DEFAULT_CHEST_TABLE`].
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub rarity: Rarity,
}

/// Boutique posée par un type de salle (`rooms/<id>.ron`, champ `shop`, M2-T6,
/// `docs/conventions.md` §50) : `shelves` présentoirs tirés dans le pool `table`, au centre de
/// chaque salle de ce type que l'étage présente.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomShopDef {
    /// Pool de butin des présentoirs (`loot/<table>.ron`).
    pub table: String,
    /// Nombre de présentoirs (> 0).
    pub shelves: u32,
}

/// Un pool de butin, `loot/<id>.ron`.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootTableDef {
    pub entries: Vec<LootEntry>,
    /// Exclut les objets déjà tenus (passifs et actif) par le joueur qui ouvre. Faux par
    /// défaut : l'objet ne dépend alors que de la graine (même objet quel que soit le moment).
    #[serde(default)]
    pub exclude_held: bool,
    #[serde(default)]
    pub reward: Option<RoomRewardDef>,
}

/// Les pools du jeu par identifiant (hors rollback).
#[derive(Resource, Clone, Debug, Default)]
pub struct LootTables(pub BTreeMap<String, LootTableDef>);

/// Pool utilisé par un coffre sans table propre, hors liste de l'étage.
pub const DEFAULT_CHEST_TABLE: &str = "commun";
/// Pool de la récompense de salle, hors liste de l'étage.
pub const DEFAULT_REWARD_TABLE: &str = "salle";

/// Graine du tirage d'une clé de contenu (`key`) à l'étage `floor` d'une partie de graine
/// `run_seed`. Indépendante du temps, des `Entity` et des `GgrsNetId`.
pub fn loot_seed(run_seed: u32, floor: u32, key: &str) -> u32 {
    let hash = fnv1a(format!("loot:{run_seed}:{floor}:{key}").as_bytes());
    (hash ^ (hash >> 32)) as u32
}

/// Rareté d'un objet pour le filtre d'un coffre (un consommable compte pour `Common`).
fn effective_rarity(table: &ItemTable, id: &str) -> Rarity {
    table.0.get(id).map(|def| def.rarity).unwrap_or_default()
}

/// Les candidats d'une entrée, triés par identifiant, après filtres.
fn candidates<'a>(
    entry: &'a LootEntry,
    items: &'a ItemTable,
    held: Option<&Inventory>,
    min_rarity: Rarity,
    consumables_only: bool,
    hidden: &BTreeSet<&str>,
) -> Vec<&'a str> {
    let ok = |id: &str, def: &crate::ItemDef| {
        !hidden.contains(id)
            && (!consumables_only || def.kind == ItemKind::Consumable)
            && effective_rarity(items, id) >= min_rarity
            && !held.is_some_and(|inventory| inventory.has(id))
    };
    match &entry.pick {
        LootPick::Item(id) => items
            .0
            .get_key_value(id)
            .filter(|(id, def)| ok(id, def))
            .map(|(id, _)| vec![id.as_str()])
            .unwrap_or_default(),
        LootPick::Rarity(rarity) => items
            .0
            .iter()
            .filter(|(id, def)| def.rarity == *rarity && ok(id, def))
            .map(|(id, _)| id.as_str())
            .collect(),
    }
}

/// Tire un objet du pool : une entrée par poids, puis un candidat uniforme (toujours deux
/// tirages consommés, le flux avance donc de la même façon quels que soient les filtres).
/// `min_rarity` : rareté minimale (celle du coffre) ; `held` : inventaire à exclure si le pool a
/// `exclude_held` ; `consumables_only` : récompense de salle sans coffre. Si le filtre ne laisse
/// rien, retombe sur le pool sans filtre de rareté ni d'exclusion (jamais vide si le pool ne
/// l'est pas).
pub fn draw_loot(
    table: &LootTableDef,
    items: &ItemTable,
    held: Option<&Inventory>,
    min_rarity: Rarity,
    consumables_only: bool,
    rng: &mut RollbackRng,
) -> Option<String> {
    draw_loot_with(
        table,
        items,
        held,
        min_rarity,
        consumables_only,
        &BTreeSet::new(),
        rng,
    )
}

/// Le pool `id` tel que les déblocages actifs le laissent : les entrées ajoutées s'ajoutent
/// après les siennes. Sans entrée ajoutée, le pool lui-même (aucune copie).
pub fn unlocked_table<'a>(
    id: &str,
    table: &'a LootTableDef,
    unlocks: &UnlockTable,
    active: &RunUnlocks,
) -> Cow<'a, LootTableDef> {
    let extra = unlocks.extra_entries(id, active);
    if extra.is_empty() {
        return Cow::Borrowed(table);
    }
    let mut table = table.clone();
    table.entries.extend(extra);
    Cow::Owned(table)
}

/// [`draw_loot`] avec des objets exclus (`hidden` : réservés à un déblocage manquant, voir
/// [`UnlockTable::hidden_items`]). Le flux RNG avance pareil : `hidden` ne change que les
/// candidats.
pub fn draw_loot_with(
    table: &LootTableDef,
    items: &ItemTable,
    held: Option<&Inventory>,
    min_rarity: Rarity,
    consumables_only: bool,
    hidden: &BTreeSet<&str>,
    rng: &mut RollbackRng,
) -> Option<String> {
    let held = held.filter(|_| table.exclude_held);
    let pick = rng.next_u32();
    let pick_candidate = rng.next_u32();
    let mut pools: Vec<(i32, Vec<&str>)> = Vec::new();
    for (min, held) in [(min_rarity, held), (Rarity::Common, None)] {
        pools = table
            .entries
            .iter()
            .filter(|entry| entry.weight > 0)
            .map(|entry| {
                (
                    entry.weight,
                    candidates(entry, items, held, min, consumables_only, hidden),
                )
            })
            .filter(|(_, ids)| !ids.is_empty())
            .collect();
        if !pools.is_empty() || consumables_only {
            break;
        }
    }
    let total: u64 = pools.iter().map(|(weight, _)| *weight as u64).sum();
    if total == 0 {
        return None;
    }
    let mut roll = u64::from(pick) % total;
    for (weight, ids) in &pools {
        if roll < *weight as u64 {
            let index = pick_candidate as usize % ids.len();
            return Some(ids[index].to_string());
        }
        roll -= *weight as u64;
    }
    None
}

/// Un coffre (rollback, checksum et trace en variante **neutre** : absent de toute partie sans
/// coffre). `key` est sa clé de contenu (graine du tirage).
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chest {
    pub key: String,
    /// Pool du coffre ; `None` : celui de l'étage, sinon [`DEFAULT_CHEST_TABLE`].
    pub table: Option<String>,
    /// Rareté minimale de l'objet tiré.
    pub rarity: Rarity,
    /// Consomme une clé (`key`) à l'ouverture.
    pub locked: bool,
    pub opened: bool,
}

/// Un coffre ouvert (`FrameEvents`, neutre).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChestOpened {
    pub frame: u32,
    pub player_handle: usize,
    pub chest_key: String,
    pub item_id: String,
}

/// Présentoir d'une boutique (rollback, checksum et trace en variante **neutre** : absent de
/// toute partie sans boutique). `key` est sa clé de contenu (`shelf:<salle>:<i>`), `price` en
/// `shells` (`Currency`), `sold` une fois acheté (l'`Interactable` est alors retiré).
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shelf {
    pub key: String,
    pub item_id: String,
    pub price: u32,
    pub sold: bool,
    /// Anti-rebond d'un achat refusé : pas de nouvel essai avant cette frame.
    pub retry_after: u32,
}

/// Douilles au sol (`shells`), ramassées au contact et créditées à `Currency` (rollback,
/// neutre).
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShellPickup {
    pub amount: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ItemDef;

    fn item(kind: ItemKind, rarity: Rarity) -> ItemDef {
        ItemDef {
            name: String::new(),
            kind,
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
        }
    }

    fn items() -> ItemTable {
        ItemTable(
            [
                ("a", item(ItemKind::Passive, Rarity::Common)),
                ("b", item(ItemKind::Passive, Rarity::Rare)),
                ("c", item(ItemKind::Passive, Rarity::Rare)),
                ("key", item(ItemKind::Consumable, Rarity::Common)),
            ]
            .into_iter()
            .map(|(id, def)| (id.to_string(), def))
            .collect(),
        )
    }

    fn table(entries: Vec<LootEntry>, exclude_held: bool) -> LootTableDef {
        LootTableDef {
            entries,
            exclude_held,
            reward: None,
        }
    }

    fn entry(pick: LootPick, weight: i32) -> LootEntry {
        LootEntry { pick, weight }
    }

    #[test]
    fn seed_depends_on_inputs_only() {
        assert_eq!(loot_seed(1, 0, "x"), loot_seed(1, 0, "x"));
        assert_ne!(loot_seed(1, 0, "x"), loot_seed(2, 0, "x"));
        assert_ne!(loot_seed(1, 0, "x"), loot_seed(1, 1, "x"));
        assert_ne!(loot_seed(1, 0, "x"), loot_seed(1, 0, "y"));
    }

    #[test]
    fn draw_is_a_function_of_the_seed() {
        let table = table(vec![entry(LootPick::Rarity(Rarity::Rare), 1)], false);
        let first = draw_loot(
            &table,
            &items(),
            None,
            Rarity::Common,
            false,
            &mut RollbackRng::new(7),
        );
        let again = draw_loot(
            &table,
            &items(),
            None,
            Rarity::Common,
            false,
            &mut RollbackRng::new(7),
        );
        assert_eq!(first, again);
        assert!(matches!(first.as_deref(), Some("b") | Some("c")));
    }

    #[test]
    fn rarity_group_is_weighted_once() {
        // Deux objets rares, un commun : le poids est celui de l'entrée, pas par objet.
        let table = table(
            vec![
                entry(LootPick::Item("a".into()), 1),
                entry(LootPick::Rarity(Rarity::Rare), 1),
            ],
            false,
        );
        let mut a = 0;
        for seed in 0..2000 {
            if draw_loot(
                &table,
                &items(),
                None,
                Rarity::Common,
                false,
                &mut RollbackRng::new(seed),
            ) == Some("a".into())
            {
                a += 1;
            }
        }
        assert!((800..1200).contains(&a), "a = {a}");
    }

    #[test]
    fn held_excluded_only_when_asked() {
        let mut inventory = Inventory::default();
        inventory.passives.push("a".into());
        let entries = vec![
            entry(LootPick::Item("a".into()), 1),
            entry(LootPick::Item("b".into()), 1),
        ];
        for seed in 0..50 {
            let drawn = draw_loot(
                &table(entries.clone(), true),
                &items(),
                Some(&inventory),
                Rarity::Common,
                false,
                &mut RollbackRng::new(seed),
            );
            assert_eq!(drawn.as_deref(), Some("b"));
        }
        let any_a = (0..50).any(|seed| {
            draw_loot(
                &table(entries.clone(), false),
                &items(),
                Some(&inventory),
                Rarity::Common,
                false,
                &mut RollbackRng::new(seed),
            ) == Some("a".into())
        });
        assert!(any_a);
    }

    #[test]
    fn min_rarity_filters_then_falls_back() {
        let only_common = table(vec![entry(LootPick::Item("a".into()), 1)], false);
        let drawn = draw_loot(
            &only_common,
            &items(),
            None,
            Rarity::Rare,
            false,
            &mut RollbackRng::new(1),
        );
        assert_eq!(drawn.as_deref(), Some("a"));
        let mixed = table(
            vec![
                entry(LootPick::Item("a".into()), 1),
                entry(LootPick::Item("b".into()), 1),
            ],
            false,
        );
        for seed in 0..50 {
            let drawn = draw_loot(
                &mixed,
                &items(),
                None,
                Rarity::Rare,
                false,
                &mut RollbackRng::new(seed),
            );
            assert_eq!(drawn.as_deref(), Some("b"));
        }
    }

    #[test]
    fn consumables_only_never_falls_back() {
        let passives = table(vec![entry(LootPick::Item("a".into()), 1)], false);
        assert_eq!(
            draw_loot(
                &passives,
                &items(),
                None,
                Rarity::Common,
                true,
                &mut RollbackRng::new(1)
            ),
            None
        );
        let with_key = table(
            vec![
                entry(LootPick::Item("a".into()), 1),
                entry(LootPick::Item("key".into()), 1),
            ],
            false,
        );
        assert_eq!(
            draw_loot(
                &with_key,
                &items(),
                None,
                Rarity::Common,
                true,
                &mut RollbackRng::new(1)
            )
            .as_deref(),
            Some("key")
        );
    }
}
