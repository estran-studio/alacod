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
use bevy_fixed::rng::{
    fmix64, fnv1a, fnv1a_extend, fnv1a_extend_decimal, seed_from_hash, RollbackRng, FNV1A_OFFSET,
};
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
    /// Une arme de `weapons/weapons.ron` (D62). Le tirage la désigne par `weapon:<id>`
    /// ([`loot_weapon`]) ; la rareté sert au filtre d'un coffre et au prix d'un présentoir.
    Weapon(String, Rarity),
    /// M3-T3 (§68) : un tas de `n` sous au sol (un `ShellPickup`, la monnaie `shells`), ramassé au
    /// contact. Le tirage le désigne par `currency:<n>` ([`loot_currency`]) ; il compte comme un
    /// consommable de rareté `Common` (récompense de salle, rocher, coffre de rareté minimale
    /// `Common`). Refusé dans le pool d'une boutique (lint). Ajoutée en fin d'enum.
    Currency(u32),
}

/// Préfixe de l'identifiant tiré pour une arme (voir [`LootPick::Weapon`]).
pub const WEAPON_LOOT_PREFIX: &str = "weapon:";

/// L'arme désignée par un identifiant tiré (`weapon:<id>`), `None` pour un objet.
pub fn loot_weapon(drawn: &str) -> Option<&str> {
    drawn.strip_prefix(WEAPON_LOOT_PREFIX)
}

/// Préfixe de l'identifiant tiré pour un tas de sous (voir [`LootPick::Currency`]).
pub const CURRENCY_LOOT_PREFIX: &str = "currency:";

/// Le montant désigné par un identifiant tiré (`currency:<n>`), `None` pour un objet ou une arme.
pub fn loot_currency(drawn: &str) -> Option<u32> {
    drawn.strip_prefix(CURRENCY_LOOT_PREFIX)?.parse().ok()
}

/// Une entrée pondérée. Un poids ≤ 0 ou > [`MAX_LOOT_WEIGHT`] est refusé par le lint.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootEntry {
    pub pick: LootPick,
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0, ≤ 1000")))]
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
    /// D62 : nombre de présentoirs (les premiers) qui ne tirent que des armes du pool.
    #[serde(default)]
    pub weapon_shelves: u32,
    /// D62 : pool de chaque étage (index = étage, le dernier vaut pour les suivants) ; vide :
    /// `table` à tous les étages.
    #[serde(default)]
    pub floor_tables: Vec<String>,
    /// M3-T8 (§73) : prix **de chaque présentoir**, quelle que soit la rareté. Absent : le prix en
    /// douilles de `economy.ron` (`shop_prices`), comme avant. `Currency(n)` fixe un prix en
    /// douilles ; `MaxHealth(n)` / `Consumable(id, n)` sont exécutés par `game::shop` (marché du
    /// diable : vie maximale, cœurs d'âme d'abord).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<crate::Price>,
}

/// Butin du boss posé par un type de salle (`rooms/<id>.ron`, champ `boss_drop`, M3-T8, §73) : à la
/// mort du boss, un objet du pool `table` tombe au centre de sa salle. Tiré à la **génération de
/// l'étage** quand le pool a `exhaust` (option A de §64), sinon sur la graine de l'emplacement
/// (`boss:<salle>`) à la mort : indépendant du moment dans les deux cas.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossDropDef {
    /// Pool de butin (`loot/<table>.ron`).
    pub table: String,
    /// Rareté minimale de l'objet tiré.
    #[serde(default)]
    pub rarity: Rarity,
}

impl RoomShopDef {
    /// Le pool de la boutique à l'étage `floor`.
    pub fn table_for_floor(&self, floor: u32) -> &str {
        match self.floor_tables.len().checked_sub(1) {
            Some(last) => self.floor_tables[(floor as usize).min(last)].as_str(),
            None => self.table.as_str(),
        }
    }
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
    /// M3-T0e (§64) : un objet sorti de ce pool n'y revient plus pendant la run ([`PoolState`],
    /// [`draw_exhausting`]). Faux par défaut : comportement d'avant, hors du hash et des traces.
    #[serde(default)]
    pub exhaust: bool,
    #[serde(default)]
    pub reward: Option<RoomRewardDef>,
}

impl LootTableDef {
    /// Ce pool réduit à ses armes (la première étagère d'une boutique en tire une, D62).
    pub fn weapons_only(&self) -> LootTableDef {
        LootTableDef {
            entries: self
                .entries
                .iter()
                .filter(|entry| matches!(entry.pick, LootPick::Weapon(..)))
                .cloned()
                .collect(),
            exclude_held: false,
            exhaust: self.exhaust,
            reward: None,
        }
    }

    /// Rareté sous laquelle ce pool donne l'arme `id` (prix d'un présentoir).
    pub fn weapon_rarity(&self, id: &str) -> Option<Rarity> {
        self.entries.iter().find_map(|entry| match &entry.pick {
            LootPick::Weapon(weapon, rarity) if weapon == id => Some(*rarity),
            _ => None,
        })
    }
}

/// Les pools du jeu par identifiant (hors rollback).
#[derive(Resource, Clone, Debug, Default)]
pub struct LootTables(pub BTreeMap<String, LootTableDef>);

/// Pool utilisé par un coffre sans table propre, hors liste de l'étage.
pub const DEFAULT_CHEST_TABLE: &str = "commun";
/// Pool de la récompense de salle, hors liste de l'étage.
pub const DEFAULT_REWARD_TABLE: &str = "salle";

/// Hash FNV-1a de `loot:<graine>:<étage>:<clé>`, sans allocation (les mêmes octets que le `format!`).
fn loot_hash(run_seed: u32, floor: u32, key: &str) -> u64 {
    fnv1a_extend(loot_prefix_hash(run_seed, floor), key.as_bytes())
}

/// Début du hash d'une graine de butin : `loot:<graine>:<étage>:`.
fn loot_prefix_hash(run_seed: u32, floor: u32) -> u64 {
    let hash = fnv1a_extend(FNV1A_OFFSET, b"loot:");
    let hash = fnv1a_extend_decimal(hash, u64::from(run_seed));
    let hash = fnv1a_extend(hash, b":");
    let hash = fnv1a_extend_decimal(hash, u64::from(floor));
    fnv1a_extend(hash, b":")
}

/// Graine du tirage d'une clé de contenu (`key`) à l'étage `floor` d'une partie de graine
/// `run_seed` : `fnv1a("loot:<graine>:<étage>:<clé>")` replié sur 32 bits puis passé au finaliseur
/// `fmix32` ([`seed_from_hash`], D95 : sans lui, deux clés voisines, `champion:41` et `champion:42`,
/// restaient corrélées, r jusqu'à 0,063). Indépendante du temps, des `Entity` et des `GgrsNetId`.
pub fn loot_seed(run_seed: u32, floor: u32, key: &str) -> u32 {
    seed_from_hash(loot_hash(run_seed, floor, key))
}

/// [`loot_seed`] de la clé `<prefix><nombre>` (`shells:`, `champion:`…), sans construire la chaîne.
pub fn loot_seed_numbered(run_seed: u32, floor: u32, prefix: &str, number: u64) -> u32 {
    let hash = fnv1a_extend(loot_prefix_hash(run_seed, floor), prefix.as_bytes());
    seed_from_hash(fnv1a_extend_decimal(hash, number))
}

/// [`loot_seed`] de la clé formée en mettant `parts` bout à bout (`&["shop:", salle]`), sans la construire.
pub fn loot_seed_parts(run_seed: u32, floor: u32, parts: &[&str]) -> u32 {
    let hash = parts
        .iter()
        .fold(loot_prefix_hash(run_seed, floor), |hash, part| {
            fnv1a_extend(hash, part.as_bytes())
        });
    seed_from_hash(hash)
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
) -> Vec<Cow<'a, str>> {
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
            .map(|(id, _)| vec![Cow::Borrowed(id.as_str())])
            .unwrap_or_default(),
        LootPick::Rarity(rarity) => items
            .0
            .iter()
            .filter(|(id, def)| def.rarity == *rarity && ok(id, def))
            .map(|(id, _)| Cow::Borrowed(id.as_str()))
            .collect(),
        LootPick::Weapon(id, rarity) => {
            if consumables_only || *rarity < min_rarity {
                Vec::new()
            } else {
                vec![Cow::Owned(format!("{WEAPON_LOOT_PREFIX}{id}"))]
            }
        }
        // Un tas de sous compte comme un consommable `Common` : ni rareté ni exclusion d'objet tenu.
        LootPick::Currency(amount) => {
            if Rarity::Common < min_rarity {
                Vec::new()
            } else {
                vec![Cow::Owned(format!("{CURRENCY_LOOT_PREFIX}{amount}"))]
            }
        }
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
    draw_exhausting(
        table,
        items,
        held,
        min_rarity,
        consumables_only,
        hidden,
        &BTreeSet::new(),
        rng,
    )
}

/// Poids maximal d'une entrée de pool pris en compte par le tirage ([`draw_exhausting`] joue un billet
/// par unité de poids) ; le lint refuse au-delà (`OutOfRange`) et le tirage plafonne par sécurité.
pub const MAX_LOOT_WEIGHT: i32 = 1000;

/// Identité d'une entrée pour la course : le **contenu** de son `pick` (jamais son rang dans la table :
/// les entrées d'un déblocage s'ajoutent après celles du pool, dans l'ordre des identifiants de
/// déblocage, et un rang changerait quand un autre déblocage s'ajoute). Hash FNV-1a, sans allocation.
fn pick_identity(pick: &LootPick) -> u64 {
    let hash = FNV1A_OFFSET;
    match pick {
        LootPick::Item(id) => fnv1a_extend(fnv1a_extend(hash, b"item:"), id.as_bytes()),
        LootPick::Rarity(rarity) => fnv1a_extend(fnv1a_extend(hash, b"rarity:"), &[*rarity as u8]),
        LootPick::Weapon(id, rarity) => fnv1a_extend(
            fnv1a_extend(fnv1a_extend(hash, b"weapon:"), id.as_bytes()),
            &[b':', *rarity as u8],
        ),
        LootPick::Currency(amount) => {
            fnv1a_extend_decimal(fnv1a_extend(hash, b"currency:"), u64::from(*amount))
        }
    }
}

/// Clé d'un billet de la course : `base` (propre au tirage), `identity` (qui concourt), `salt` (quel
/// billet). `salt` passe par une multiplication impaire puis `fmix64` : deux billets d'une même
/// identité ont toujours deux clés distinctes ; les autres collisions valent 2⁻⁶⁴ (départagées par
/// l'identité).
fn race_key(base: u64, identity: u64, salt: u64) -> u64 {
    fmix64(fmix64(base ^ identity) ^ salt.wrapping_mul(0x9e37_79b9_7f4a_7c15))
}

/// Séparation des clés du second niveau (le candidat) de celles du premier (l'entrée).
const CANDIDATE_LEVEL: u64 = 0xd6e8_feb8_6659_fd93;

/// Une entrée du pool qui concourt : son identité, son poids (les entrées de même `pick` s'additionnent,
/// ce qui ne change pas la loi du tirage) et ses candidats.
struct Contender<'a> {
    identity: u64,
    weight: u64,
    ids: Vec<Cow<'a, str>>,
}

/// [`draw_loot_with`] pour un pool à épuisement (M3-T0e, `docs/conventions.md` §64) : `seen` est
/// l'ensemble des identifiants déjà sortis de ce pool pendant la run ([`PoolState::seen`]). Si
/// `table.exhaust`, ils ne sont plus candidats (une arme s'y désigne par `weapon:<id>`) ; sinon
/// `seen` est ignoré.
///
/// Le flux avance toujours de deux tirages, quel que soit `seen`. Quand l'épuisement vide les
/// candidats, le tirage retombe d'abord sur les raretés et exclusions assouplies de
/// [`draw_loot`], puis, **pool entièrement vu**, sur le pool sans l'exclusion de `seen` (un
/// objet déjà sorti peut alors ressortir : le pool « repart complet », il n'est jamais vide si le
/// pool ne l'est pas). L'appelant n'enregistre dans [`PoolState`] que ce qui est nouveau
/// ([`PoolState::record`]).
///
/// **Course pondérée (D96, §64).** Les deux tirages du flux forment une clé de base de 64 bits ; le
/// choix n'est plus `tirage % somme des poids` (la somme changeait quand un déblocage ajoutait une
/// entrée, et **tous** les emplacements se redistribuaient) mais une course à deux niveaux. 1) Chaque
/// entrée joue autant de billets que son poids, de clé `hash(base, contenu de l'entrée, n° du billet)` ;
/// la plus petite clé gagne : l'entrée gagne avec la probabilité `poids / somme` **exacte** (le
/// minimum de `n` clés uniformes ; une clé mise à l'échelle par `1/poids` donnerait 3/4 et non 2/3
/// pour les poids 1 et 2). 2) Parmi les candidats de l'entrée gagnante, la plus petite clé
/// `hash(base, contenu de l'entrée, identifiant)` gagne : tirage uniforme. Conséquence, **à `seen`,
/// raretés et exclusions égaux** : ajouter une entrée, ajouter un candidat à une entrée, ou augmenter
/// un poids ne change l'objet tiré que s'il devient un objet de l'entrée ajoutée, du candidat ajouté ou de
/// l'entrée alourdie : l'objet d'une entrée `Item`, l'arme ou le tas de sous d'une entrée `Weapon` ou
/// `Currency`, et, pour une entrée `Rarity` (ajoutée, alourdie, ou fusionnée avec une entrée de même
/// contenu), **n'importe lequel de ses candidats**, y compris un qui existait déjà. Retirer ce qui n'a pas
/// gagné ne change rien. Un déblocage ne déplace donc que les emplacements qui prennent un objet de ce
/// qu'il apporte.
/// Limite : si l'ajout rend non vide une passe plus stricte (rareté du coffre), c'est cette passe qui
/// tire, et l'objet peut changer. Coût : un `fmix64` par unité de poids du pool.
#[allow(clippy::too_many_arguments)]
pub fn draw_exhausting(
    table: &LootTableDef,
    items: &ItemTable,
    held: Option<&Inventory>,
    min_rarity: Rarity,
    consumables_only: bool,
    hidden: &BTreeSet<&str>,
    seen: &BTreeSet<String>,
    rng: &mut RollbackRng,
) -> Option<String> {
    let held = held.filter(|_| table.exclude_held);
    let blank = BTreeSet::new();
    let seen = if table.exhaust { seen } else { &blank };
    let pick = rng.next_u32();
    let pick_candidate = rng.next_u32();
    // Les passes d'assouplissement, de la plus stricte à la plus large ; une récompense de salle
    // (`consumables_only`) ne relâche jamais la rareté ni l'exclusion, seulement `seen`.
    let mut passes = vec![(min_rarity, held, seen)];
    if !consumables_only {
        passes.push((Rarity::Common, None, seen));
    }
    if !seen.is_empty() {
        let (min, held) = if consumables_only {
            (min_rarity, held)
        } else {
            (Rarity::Common, None)
        };
        passes.push((min, held, &blank));
    }
    let mut contenders: Vec<Contender> = Vec::new();
    for (min, held, skip) in passes {
        contenders.clear();
        for entry in table.entries.iter().filter(|entry| entry.weight > 0) {
            let mut ids = candidates(entry, items, held, min, consumables_only, hidden);
            ids.retain(|id| !skip.contains(id.as_ref()));
            if ids.is_empty() {
                continue;
            }
            let identity = pick_identity(&entry.pick);
            let weight = entry.weight.min(MAX_LOOT_WEIGHT) as u64;
            match contenders.iter_mut().find(|c| c.identity == identity) {
                Some(contender) => {
                    contender.weight = (contender.weight + weight).min(MAX_LOOT_WEIGHT as u64)
                }
                None => contenders.push(Contender {
                    identity,
                    weight,
                    ids,
                }),
            }
        }
        if !contenders.is_empty() {
            break;
        }
    }
    let base = u64::from(pick) << 32 | u64::from(pick_candidate);
    let winner = contenders.iter().min_by_key(|contender| {
        let best = (0..contender.weight)
            .map(|ticket| race_key(base, contender.identity, ticket))
            .min();
        (best, contender.identity)
    })?;
    winner
        .ids
        .iter()
        .map(|id| {
            let id: &str = id;
            let salt = fnv1a(id.as_bytes());
            (race_key(base ^ CANDIDATE_LEVEL, winner.identity, salt), id)
        })
        .min()
        .map(|(_, id)| id.to_string())
}

/// Ce qui est déjà sorti de chaque pool à épuisement pendant la run (M3-T0e, §64) et les objets
/// **assignés** aux emplacements à la génération des étages (M3-T8, §73). Épuisement **par run et
/// par pool** : un pool partagé par plusieurs salles (trésor, boutique, boss, diable) est le même
/// pour toutes. Ressource rollback **neutre** (vide tant qu'aucun pool n'a `exhaust`, donc aucune
/// trace existante ne bouge), remise à zéro à chaque relance.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PoolState {
    /// Pool (`loot/<id>.ron`) → identifiants tirés.
    pub seen: BTreeMap<String, BTreeSet<String>>,
    /// `"<étage>:<clé de contenu>"` → objet tiré à la génération de l'étage pour cet emplacement
    /// (coffre de salle, présentoir, butin du boss, placement de scénario). Option A de §64.
    pub assigned: BTreeMap<String, String>,
}

impl PoolState {
    /// Les identifiants déjà sortis du pool `pool` (vide si aucun).
    pub fn seen(&self, pool: &str) -> &BTreeSet<String> {
        static NONE: BTreeSet<String> = BTreeSet::new();
        self.seen.get(pool).unwrap_or(&NONE)
    }

    /// `item` est-il déjà sorti de `pool` ?
    pub fn has_seen(&self, pool: &str, item: &str) -> bool {
        self.seen(pool).contains(item)
    }

    /// Nombre d'objets sortis de `pool` : l'**index d'épuisement** du prochain tirage.
    pub fn count(&self, pool: &str) -> u32 {
        self.seen(pool).len() as u32
    }

    /// Note que `item` est sorti de `pool`. Renvoie `true` si c'est nouveau (`false` : il était
    /// déjà sorti, cas d'un pool entièrement vu qui repart complet).
    ///
    /// M3-T3 (§68) : un tas de sous (`currency:<n>`) n'est **jamais** noté : il n'épuise pas le
    /// pool (renvoie `false`, aucun état).
    pub fn record(&mut self, pool: &str, item: &str) -> bool {
        if loot_currency(item).is_some() {
            return false;
        }
        self.seen
            .entry(pool.to_string())
            .or_default()
            .insert(item.to_string())
    }

    /// Graine du prochain tirage du pool `pool` pour la clé de contenu `key` à l'étage `floor` :
    /// [`exhaust_seed`] avec [`Self::count`] comme index.
    pub fn next_seed(&self, pool: &str, run_seed: u32, floor: u32, key: &str) -> u32 {
        exhaust_seed(run_seed, floor, key, self.count(pool))
    }

    fn slot_key(floor: u32, key: &str) -> String {
        format!("{floor}:{key}")
    }

    /// L'objet assigné à l'emplacement `key` de l'étage `floor`, s'il l'a été à la génération.
    pub fn assignment(&self, floor: u32, key: &str) -> Option<&str> {
        self.assigned
            .get(&Self::slot_key(floor, key))
            .map(String::as_str)
    }

    /// Assigne un objet à l'emplacement `key` de l'étage `floor` dans le pool `pool` (M3-T8, option A
    /// de §64) : tirage `draw_exhausting` sur la graine d'emplacement [`Self::next_seed`], sans
    /// l'objet déjà sorti du pool, puis l'objet sort du pool ([`Self::record`]) et l'assignation est
    /// conservée. **Idempotent** : un emplacement déjà assigné rend son objet sans rien tirer (deux
    /// coffres de même clé donnent le même objet, comme avant). `None` sans épuisement
    /// (`table.exhaust` faux : l'appelant tire à l'usage, comme avant) ou si le pool ne donne rien.
    ///
    /// Le tirage ignore l'inventaire de l'ouvreur (`held` vaut `None`) : à la génération il n'y a
    /// pas d'ouvreur, donc l'objet ne dépend pas du moment (le lint refuse `exhaust` avec
    /// `exclude_held`). `hidden` : objets réservés à un déblocage manquant.
    #[allow(clippy::too_many_arguments)]
    pub fn assign(
        &mut self,
        pool: &str,
        table: &LootTableDef,
        items: &ItemTable,
        min_rarity: Rarity,
        hidden: &BTreeSet<&str>,
        run_seed: u32,
        floor: u32,
        key: &str,
    ) -> Option<String> {
        if !table.exhaust {
            return None;
        }
        if let Some(item) = self.assignment(floor, key) {
            return Some(item.to_string());
        }
        let mut rng = RollbackRng::new(self.next_seed(pool, run_seed, floor, key));
        let item = draw_exhausting(
            table,
            items,
            None,
            min_rarity,
            false,
            hidden,
            self.seen(pool),
            &mut rng,
        )?;
        self.record(pool, &item);
        self.assigned
            .insert(Self::slot_key(floor, key), item.clone());
        Some(item)
    }
}

/// Graine d'un tirage à épuisement : [`loot_seed`] pour l'index d'épuisement `0` (le premier
/// tirage d'un pool donne donc l'objet qu'il donnait avant `exhaust`), puis la même graine
/// mêlée à `index` (nombre d'objets déjà sortis du pool). Indépendante du temps et des
/// `Entity` : tirée dans l'ordre du plan de l'étage (graine de l'emplacement), l'index ne
/// dépend pas du moment où le joueur ouvre le coffre.
pub fn exhaust_seed(run_seed: u32, floor: u32, key: &str, index: u32) -> u32 {
    if index == 0 {
        loot_seed(run_seed, floor, key)
    } else {
        // Les mêmes octets que `loot_seed(…, "<clé>#<index>")`, sans allocation.
        let hash = fnv1a_extend(loot_hash(run_seed, floor, key), b"#");
        seed_from_hash(fnv1a_extend_decimal(hash, u64::from(index)))
    }
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
/// toute partie sans boutique). `price` conserve le hash historique ; `cost` est optionnel et
/// n'entre dans le hash que lorsqu'il est présent (M3-T0b).
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct Shelf {
    pub key: String,
    pub item_id: String,
    pub price: u32,
    pub sold: bool,
    pub retry_after: u32,
    pub cost: Option<crate::Price>,
}

impl std::hash::Hash for Shelf {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.item_id.hash(state);
        self.price.hash(state);
        self.sold.hash(state);
        self.retry_after.hash(state);
        if let Some(cost) = &self.cost {
            cost.hash(state);
        }
    }
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
            max_stack: None,
            refuse_when_full: false,
            stack: Default::default(),
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
            exhaust: false,
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
    /// D62 : une entrée `Weapon` donne `weapon:<id>`, suit le filtre de rareté d'un coffre, est
    /// ignorée d'une récompense de consommables, et `weapons_only` / `weapon_rarity` la retrouvent.
    #[test]
    fn weapon_entries() {
        let pool = table(
            vec![
                entry(LootPick::Weapon("revolver".into(), Rarity::Common), 1),
                entry(LootPick::Weapon("laser".into(), Rarity::Rare), 1),
                entry(LootPick::Item("a".into()), 1),
            ],
            false,
        );
        let mut seen = BTreeSet::new();
        for seed in 0..200 {
            let drawn = draw_loot(
                &pool,
                &items(),
                None,
                Rarity::Common,
                false,
                &mut RollbackRng::new(seed),
            )
            .unwrap();
            seen.insert(drawn);
        }
        assert!(seen.contains("weapon:revolver") && seen.contains("weapon:laser"));
        assert!(seen.contains("a"));
        assert_eq!(loot_weapon("weapon:laser"), Some("laser"));
        assert_eq!(loot_weapon("a"), None);
        // Coffre Rare : seules les raretés suffisantes (l'arme rare ; pas de repli si elle existe).
        for seed in 0..50 {
            let drawn = draw_loot(
                &pool,
                &items(),
                None,
                Rarity::Rare,
                false,
                &mut RollbackRng::new(seed),
            );
            assert_eq!(drawn.as_deref(), Some("weapon:laser"));
        }
        // Récompense de salle : consommables seulement, jamais d'arme.
        for seed in 0..50 {
            let drawn = draw_loot(
                &pool,
                &items(),
                None,
                Rarity::Common,
                true,
                &mut RollbackRng::new(seed),
            );
            assert!(drawn.is_none(), "{drawn:?}");
        }
        assert_eq!(pool.weapons_only().entries.len(), 2);
        assert_eq!(pool.weapon_rarity("laser"), Some(Rarity::Rare));
        assert_eq!(pool.weapon_rarity("absent"), None);
    }

    fn exhausting(entries: Vec<LootEntry>) -> LootTableDef {
        LootTableDef {
            exhaust: true,
            ..table(entries, false)
        }
    }

    fn pool_abc() -> LootTableDef {
        exhausting(vec![
            entry(LootPick::Item("a".into()), 1),
            entry(LootPick::Item("b".into()), 1),
            entry(LootPick::Item("c".into()), 1),
        ])
    }

    /// Tire `slots` emplacements dans l'ordre du plan, comme la génération d'un étage (T8) :
    /// graine d'emplacement + index d'épuisement, chaque nouveauté sort du pool.
    fn draw_slots(pool: &LootTableDef, run_seed: u32, slots: &[&str]) -> Vec<String> {
        let mut state = PoolState::default();
        slots
            .iter()
            .map(|key| {
                let mut rng = RollbackRng::new(state.next_seed("p", run_seed, 0, key));
                let drawn = draw_exhausting(
                    pool,
                    &items(),
                    None,
                    Rarity::Common,
                    false,
                    &BTreeSet::new(),
                    state.seen("p"),
                    &mut rng,
                )
                .unwrap();
                state.record("p", &drawn);
                drawn
            })
            .collect()
    }

    #[test]
    fn exhaust_false_ignores_seen_and_matches_draw_loot() {
        let plain = table(
            vec![
                entry(LootPick::Item("a".into()), 1),
                entry(LootPick::Item("b".into()), 1),
            ],
            false,
        );
        let seen: BTreeSet<String> = ["a".to_string()].into();
        for seed in 0..100 {
            let expected = draw_loot(
                &plain,
                &items(),
                None,
                Rarity::Common,
                false,
                &mut RollbackRng::new(seed),
            );
            let got = draw_exhausting(
                &plain,
                &items(),
                None,
                Rarity::Common,
                false,
                &BTreeSet::new(),
                &seen,
                &mut RollbackRng::new(seed),
            );
            assert_eq!(got, expected, "graine {seed}");
        }
    }

    #[test]
    fn seen_items_leave_the_pool_until_it_is_exhausted() {
        let pool = pool_abc();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut drawn = Vec::new();
        for seed in 0..3 {
            let item = draw_exhausting(
                &pool,
                &items(),
                None,
                Rarity::Common,
                false,
                &BTreeSet::new(),
                &seen,
                &mut RollbackRng::new(seed),
            )
            .unwrap();
            assert!(seen.insert(item.clone()), "{item} est déjà sorti");
            drawn.push(item);
        }
        drawn.sort();
        assert_eq!(drawn, ["a", "b", "c"]);
        // Pool entièrement vu : il repart complet, jamais vide.
        let again = draw_exhausting(
            &pool,
            &items(),
            None,
            Rarity::Common,
            false,
            &BTreeSet::new(),
            &seen,
            &mut RollbackRng::new(9),
        );
        assert!(matches!(again.as_deref(), Some("a" | "b" | "c")));
    }

    #[test]
    fn weapons_exhaust_by_their_prefixed_id() {
        let pool = exhausting(vec![
            entry(LootPick::Weapon("revolver".into(), Rarity::Common), 1),
            entry(LootPick::Weapon("laser".into(), Rarity::Common), 1),
        ]);
        let seen: BTreeSet<String> = ["weapon:revolver".to_string()].into();
        for seed in 0..50 {
            let drawn = draw_exhausting(
                &pool,
                &items(),
                None,
                Rarity::Common,
                false,
                &BTreeSet::new(),
                &seen,
                &mut RollbackRng::new(seed),
            );
            assert_eq!(drawn.as_deref(), Some("weapon:laser"));
        }
    }

    #[test]
    fn exhaustion_relaxes_rarity_before_repeating_an_item() {
        // Coffre Rare : b et c sont les seuls candidats. Une fois b vu, c sort ; b et c vus,
        // l'objet neuf plus commun (a) passe avant de répéter un objet vu.
        let pool = exhausting(vec![
            entry(LootPick::Item("b".into()), 1),
            entry(LootPick::Item("c".into()), 1),
            entry(LootPick::Item("a".into()), 1),
        ]);
        let draw = |seen: &[&str], seed| {
            draw_exhausting(
                &pool,
                &items(),
                None,
                Rarity::Rare,
                false,
                &BTreeSet::new(),
                &seen.iter().map(|id| id.to_string()).collect(),
                &mut RollbackRng::new(seed),
            )
        };
        for seed in 0..50 {
            assert_eq!(draw(&["b"], seed).as_deref(), Some("c"));
            assert_eq!(draw(&["b", "c"], seed).as_deref(), Some("a"));
        }
    }

    #[test]
    fn consumables_only_exhausts_without_relaxing_rarity() {
        let pool = exhausting(vec![
            entry(LootPick::Item("key".into()), 1),
            entry(LootPick::Item("a".into()), 1),
        ]);
        let seen: BTreeSet<String> = ["key".to_string()].into();
        // Seul consommable vu : le pool repart complet (jamais une passive).
        for seed in 0..20 {
            let drawn = draw_exhausting(
                &pool,
                &items(),
                None,
                Rarity::Common,
                true,
                &BTreeSet::new(),
                &seen,
                &mut RollbackRng::new(seed),
            );
            assert_eq!(drawn.as_deref(), Some("key"));
        }
    }

    #[test]
    fn the_stream_advances_the_same_way_whatever_seen_is() {
        let pool = pool_abc();
        let after = |seen: BTreeSet<String>| {
            let mut rng = RollbackRng::new(11);
            draw_exhausting(
                &pool,
                &items(),
                None,
                Rarity::Common,
                false,
                &BTreeSet::new(),
                &seen,
                &mut rng,
            );
            rng.next_u32()
        };
        let none = after(BTreeSet::new());
        let some = after(["a".to_string(), "b".to_string()].into());
        let all = after(["a".to_string(), "b".to_string(), "c".to_string()].into());
        assert_eq!(none, some);
        assert_eq!(none, all);
    }

    #[test]
    fn first_draw_keeps_the_loot_seed_then_the_index_changes_it() {
        assert_eq!(exhaust_seed(3, 1, "salle", 0), loot_seed(3, 1, "salle"));
        let seeds: BTreeSet<u32> = (0..8).map(|i| exhaust_seed(3, 1, "salle", i)).collect();
        assert_eq!(
            seeds.len(),
            8,
            "un index d'épuisement donne sa propre graine"
        );
        let mut state = PoolState::default();
        assert_eq!(
            state.next_seed("p", 3, 1, "salle"),
            loot_seed(3, 1, "salle")
        );
        state.record("p", "a");
        assert_eq!(
            state.next_seed("p", 3, 1, "salle"),
            exhaust_seed(3, 1, "salle", 1)
        );
        // Un autre pool a son propre index.
        assert_eq!(
            state.next_seed("q", 3, 1, "salle"),
            loot_seed(3, 1, "salle")
        );
    }

    #[test]
    fn plan_order_draws_never_repeat_and_do_not_depend_on_opening_time() {
        let pool = pool_abc();
        let plan = ["tresor", "boutique", "boss"];
        let first = draw_slots(&pool, 21, &plan);
        let mut sorted = first.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            3,
            "trois emplacements, trois objets : {first:?}"
        );
        // Le tirage est fait à la génération : l'ordre d'ouverture n'y entre pas, et la
        // même graine redonne la même liste.
        assert_eq!(draw_slots(&pool, 21, &plan), first);
        let other: Vec<_> = (22..40)
            .map(|seed| draw_slots(&pool, seed, &plan))
            .collect();
        assert!(other.iter().any(|draw| draw != &first), "la graine compte");
    }

    #[test]
    fn pool_state_records_per_pool_and_flags_repeats() {
        let mut state = PoolState::default();
        assert_eq!(state.count("p"), 0);
        assert!(!state.has_seen("p", "a"));
        assert!(state.record("p", "a"));
        assert!(!state.record("p", "a"), "déjà sorti : pas une nouveauté");
        assert!(state.record("q", "a"), "un autre pool ne partage rien");
        assert_eq!((state.count("p"), state.count("q")), (1, 1));
        assert!(state.has_seen("p", "a") && !state.has_seen("p", "b"));
    }

    /// M3-T8 : `assign` n'agit que sur un pool à épuisement ; sinon rien n'est écrit (le tirage se
    /// fait à l'usage, comme avant).
    #[test]
    fn assign_ignores_a_pool_without_exhaust() {
        let plain = table(vec![entry(LootPick::Item("a".into()), 1)], false);
        let mut state = PoolState::default();
        let got = state.assign(
            "p",
            &plain,
            &items(),
            Rarity::Common,
            &BTreeSet::new(),
            1,
            0,
            "k",
        );
        assert_eq!(got, None);
        assert_eq!(state, PoolState::default(), "aucun état écrit");
    }

    fn assign_all(
        state: &mut PoolState,
        pool: &LootTableDef,
        floor: u32,
        keys: &[&str],
    ) -> Vec<String> {
        keys.iter()
            .map(|key| {
                state
                    .assign(
                        "p",
                        pool,
                        &items(),
                        Rarity::Common,
                        &BTreeSet::new(),
                        21,
                        floor,
                        key,
                    )
                    .unwrap()
            })
            .collect()
    }

    /// Trois emplacements d'un pool de trois objets : trois objets distincts, conservés, et
    /// idempotents (un emplacement déjà assigné rend son objet sans consommer le pool).
    #[test]
    fn assign_gives_distinct_items_and_is_idempotent() {
        let pool = pool_abc();
        let mut state = PoolState::default();
        let first = assign_all(&mut state, &pool, 0, &["tresor", "boutique", "boss"]);
        let unique: BTreeSet<_> = first.iter().collect();
        assert_eq!(unique.len(), 3, "{first:?}");
        assert_eq!(state.count("p"), 3);
        for (key, item) in ["tresor", "boutique", "boss"].iter().zip(&first) {
            assert_eq!(state.assignment(0, key), Some(item.as_str()));
        }
        let before = state.clone();
        assert_eq!(
            assign_all(&mut state, &pool, 0, &["boss", "tresor"]),
            vec![first[2].clone(), first[0].clone()]
        );
        assert_eq!(state, before, "réassigner ne change rien");
        // Le premier emplacement d'un pool qui s'épuise donne l'objet qu'il donnait avant `exhaust`.
        let mut fresh = PoolState::default();
        let plain = LootTableDef {
            exhaust: false,
            ..pool_abc()
        };
        let mut rng = RollbackRng::new(loot_seed(21, 0, "tresor"));
        let unexhausted =
            draw_loot(&plain, &items(), None, Rarity::Common, false, &mut rng).unwrap();
        assert_eq!(
            assign_all(&mut fresh, &pool, 0, &["tresor"]),
            vec![unexhausted]
        );
    }

    /// L'épuisement est **par run** : le même emplacement (même clé) à l'étage suivant est un autre
    /// emplacement, et l'objet déjà sorti ne revient pas.
    #[test]
    fn assign_is_per_run_across_floors() {
        let pool = pool_abc();
        let mut state = PoolState::default();
        let floor0 = assign_all(&mut state, &pool, 0, &["tresor"]);
        let floor1 = assign_all(&mut state, &pool, 1, &["tresor"]);
        assert_ne!(floor0, floor1);
        assert_eq!(state.assignment(0, "tresor"), Some(floor0[0].as_str()));
        assert_eq!(state.assignment(1, "tresor"), Some(floor1[0].as_str()));
        assert_eq!(state.count("p"), 2);
    }

    /// Pool entièrement sorti : il repart complet (un quatrième emplacement donne un objet connu,
    /// jamais `None`), sans rien ajouter à ce qui est sorti.
    #[test]
    fn assign_restarts_a_fully_seen_pool() {
        let pool = pool_abc();
        let mut state = PoolState::default();
        let drawn = assign_all(&mut state, &pool, 0, &["a", "b", "c", "d"]);
        assert_eq!(state.count("p"), 3);
        assert!(state.seen("p").contains(&drawn[3]));
        assert_eq!(state.assignment(0, "d"), Some(drawn[3].as_str()));
    }

    /// Un objet réservé à un déblocage manquant (`hidden`) n'est jamais assigné.
    #[test]
    fn assign_skips_hidden_items() {
        let pool = pool_abc();
        let hidden: BTreeSet<&str> = BTreeSet::from(["a", "b"]);
        let mut state = PoolState::default();
        for key in ["x", "y", "z"] {
            let got = state
                .assign("p", &pool, &items(), Rarity::Common, &hidden, 5, 0, key)
                .unwrap();
            assert_eq!(got, "c");
        }
    }

    #[test]
    fn exhaust_defaults_to_false_in_a_pool_file() {
        let plain: LootTableDef =
            ron::from_str(r#"(entries: [(pick: Item("a"), weight: 1)])"#).unwrap();
        assert!(!plain.exhaust);
        let exhausting: LootTableDef =
            ron::from_str(r#"(entries: [(pick: Item("a"), weight: 1)], exhaust: true)"#).unwrap();
        assert!(exhausting.exhaust);
    }

    #[test]
    fn currency_pick_is_a_common_consumable_that_reads_back_as_an_amount() {
        assert_eq!(loot_currency("currency:7"), Some(7));
        assert_eq!(loot_currency("currency:x"), None);
        assert_eq!(loot_currency("weapon:laser"), None);
        assert_eq!(loot_currency("key"), None);
        let pool = table(
            vec![
                entry(LootPick::Currency(5), 1),
                entry(LootPick::Item("b".into()), 1),
            ],
            false,
        );
        let draw = |min: Rarity, consumables_only: bool, seed: u32| {
            draw_loot(
                &pool,
                &items(),
                None,
                min,
                consumables_only,
                &mut RollbackRng::new(seed),
            )
        };
        // Récompense de salle (consommables seulement) : l'objet rare est exclu, restent les sous.
        for seed in 0..50 {
            assert_eq!(
                draw(Rarity::Common, true, seed).as_deref(),
                Some("currency:5")
            );
        }
        // Coffre de rareté minimale Rare : les sous (Common) ne sont pas candidats.
        for seed in 0..50 {
            assert_eq!(draw(Rarity::Rare, false, seed).as_deref(), Some("b"));
        }
        // Coffre ordinaire : les deux sortent.
        let seen: BTreeSet<_> = (0..200)
            .filter_map(|seed| draw(Rarity::Common, false, seed))
            .collect();
        assert_eq!(
            seen,
            BTreeSet::from(["b".to_string(), "currency:5".to_string()])
        );
    }

    #[test]
    fn a_currency_pile_never_exhausts_the_pool() {
        let pool = exhausting(vec![
            entry(LootPick::Currency(5), 1),
            entry(LootPick::Item("a".into()), 1),
        ]);
        let mut state = PoolState::default();
        assert!(!state.record("p", "currency:5"));
        assert_eq!(state.count("p"), 0);
        // Sans rien de vu, les sous restent tirables à chaque emplacement, même à répétition.
        let mut got_coins = 0;
        for seed in 0..60 {
            let drawn = draw_exhausting(
                &pool,
                &items(),
                None,
                Rarity::Common,
                true,
                &BTreeSet::new(),
                state.seen("p"),
                &mut RollbackRng::new(seed),
            )
            .unwrap();
            if drawn == "currency:5" {
                got_coins += 1;
            }
            state.record("p", &drawn);
        }
        assert!(got_coins > 1, "{got_coins}");
    }

    // ---- D96 : la course pondérée ----------------------------------------------------------------

    /// Tirage d'un emplacement `n` (graine de `loot_seed`, comme le jeu), pool sans épuisement.
    fn drawn(table: &LootTableDef, items: &ItemTable, hidden: &BTreeSet<&str>, n: u32) -> String {
        let seed = loot_seed(7, 0, &format!("slot{n}"));
        draw_loot_with(
            table,
            items,
            None,
            Rarity::Common,
            false,
            hidden,
            &mut RollbackRng::new(seed),
        )
        .expect("le pool donne un objet")
    }

    /// Une table d'objets `x0`…`x{n-1}` de rareté `Common` plus `r0`…`r{m-1}` de rareté `Rare`.
    fn many_items(common: usize, rare: usize) -> ItemTable {
        ItemTable(
            (0..common)
                .map(|i| (format!("x{i}"), item(ItemKind::Passive, Rarity::Common)))
                .chain((0..rare).map(|i| (format!("r{i}"), item(ItemKind::Passive, Rarity::Rare))))
                .collect(),
        )
    }

    /// Proportions de la course : une entrée gagne avec `poids / somme` et ses candidats sont
    /// équiprobables. 100 000 tirages : un écart-type vaut au plus 0,0016.
    #[test]
    fn the_race_keeps_the_weighted_proportions() {
        // a : poids 1 ; key : poids 2 ; Rarity(Rare) = {b, c} : poids 3 -> 1/6, 2/6, 1/4, 1/4.
        let pool = table(
            vec![
                entry(LootPick::Item("a".into()), 1),
                entry(LootPick::Item("key".into()), 2),
                entry(LootPick::Rarity(Rarity::Rare), 3),
            ],
            false,
        );
        let items = items();
        let mut counts: BTreeMap<String, u32> = BTreeMap::new();
        const DRAWS: u32 = 100_000;
        for n in 0..DRAWS {
            *counts
                .entry(drawn(&pool, &items, &BTreeSet::new(), n))
                .or_default() += 1;
        }
        let expected = [
            ("a", 1.0 / 6.0),
            ("key", 2.0 / 6.0),
            ("b", 0.25),
            ("c", 0.25),
        ];
        for (id, share) in expected {
            let measured = f64::from(counts[id]) / f64::from(DRAWS);
            eprintln!("course : {id} {measured:.4} (attendu {share:.4})");
            assert!(
                (measured - share).abs() < 0.008,
                "{id} : {measured:.4} au lieu de {share:.4} ({counts:?})"
            );
        }
        assert_eq!(counts.len(), 4, "{counts:?}");

        // Quatre poids inégaux (1, 2, 3, 4) sur des objets distincts.
        let items = many_items(4, 0);
        let pool = table(
            (0..4)
                .map(|i| entry(LootPick::Item(format!("x{i}")), i + 1))
                .collect(),
            false,
        );
        let mut counts = [0u32; 4];
        for n in 0..DRAWS {
            let id = drawn(&pool, &items, &BTreeSet::new(), n);
            counts[id[1..].parse::<usize>().unwrap()] += 1;
        }
        for (i, count) in counts.iter().enumerate() {
            let (measured, share) = (
                f64::from(*count) / f64::from(DRAWS),
                (i as f64 + 1.0) / 10.0,
            );
            eprintln!("course : x{i} {measured:.4} (attendu {share:.4})");
            assert!(
                (measured - share).abs() < 0.008,
                "x{i} : {measured:.4} au lieu de {share:.4}"
            );
        }
    }

    /// Le contrat de D96 : à graine égale, ajouter une entrée, débloquer un objet caché, ajouter un
    /// candidat à une entrée de rareté ou augmenter un poids ne déplace un emplacement que vers l'objet
    /// ajouté ; l'ordre des entrées (et donc leur rang) n'y change rien.
    #[test]
    fn adding_to_a_pool_only_moves_slots_to_the_added_item() {
        const SLOTS: u32 = 5_000;
        let none = BTreeSet::new();
        let base_items = many_items(6, 3);
        let base = table(
            vec![
                entry(LootPick::Item("x0".into()), 2),
                entry(LootPick::Item("x1".into()), 1),
                entry(LootPick::Item("x2".into()), 3),
                entry(LootPick::Rarity(Rarity::Rare), 2),
                entry(LootPick::Item("x3".into()), 2),
            ],
            false,
        );
        let before: Vec<String> = (0..SLOTS)
            .map(|n| drawn(&base, &base_items, &none, n))
            .collect();
        let check = |label: &str, after: Vec<String>, added: &str| {
            let moved: Vec<(usize, &String)> = after
                .iter()
                .enumerate()
                .filter(|(n, item)| **item != before[*n])
                .collect();
            assert!(!moved.is_empty(), "{label} : aucun emplacement ne bouge");
            for (n, item) in &moved {
                assert_eq!(
                    item.as_str(),
                    added,
                    "{label} : l'emplacement {n} passe de {} à {item}",
                    before[*n]
                );
            }
            eprintln!(
                "{label} : {} emplacements sur {SLOTS} bougent, tous vers {added}",
                moved.len()
            );
        };
        let draws = |table: &LootTableDef, items: &ItemTable, hidden: &BTreeSet<&str>| {
            (0..SLOTS)
                .map(|n| drawn(table, items, hidden, n))
                .collect::<Vec<_>>()
        };

        // 1. Une entrée ajoutée à la fin, puis au début (le rang des autres change) : même résultat.
        let mut at_end = base.clone();
        at_end.entries.push(entry(LootPick::Item("x4".into()), 2));
        let after_end = draws(&at_end, &base_items, &none);
        check("entrée ajoutée à la fin", after_end.clone(), "x4");
        let mut at_start = base.clone();
        at_start
            .entries
            .insert(0, entry(LootPick::Item("x4".into()), 2));
        assert_eq!(draws(&at_start, &base_items, &none), after_end);

        // 2. Un objet caché qui se débloque (l'entrée existe, `hidden` la masquait).
        let mut hidden_pool = base.clone();
        hidden_pool
            .entries
            .push(entry(LootPick::Item("x5".into()), 2));
        let masked: BTreeSet<&str> = ["x5"].into_iter().collect();
        assert_eq!(draws(&hidden_pool, &base_items, &masked), before);
        check(
            "objet débloqué",
            draws(&hidden_pool, &base_items, &none),
            "x5",
        );

        // 3. Un candidat de plus dans l'entrée `Rarity(Rare)` ({r0, r1, r2} -> + r3).
        let richer = many_items(6, 4);
        let after = draws(&base, &richer, &none);
        check("candidat de rareté ajouté", after, "r3");

        // 4. Un poids augmenté : seul l'objet de l'entrée gagne des emplacements.
        let mut heavier = base.clone();
        heavier.entries[1].weight = 4;
        check("poids augmenté", draws(&heavier, &base_items, &none), "x1");

        // 5. L'ordre des entrées et leur doublon n'ont aucun effet sur la loi : mêmes tirages.
        let mut reversed = base.clone();
        reversed.entries.reverse();
        assert_eq!(draws(&reversed, &base_items, &none), before);
        let mut split = base.clone();
        split.entries[0].weight = 1;
        split.entries.push(entry(LootPick::Item("x0".into()), 1));
        assert_eq!(
            draws(&split, &base_items, &none),
            before,
            "deux entrées du même objet valent une entrée de poids somme"
        );
    }

    /// Le contrat pour les entrées à plusieurs candidats ou sans objet : ajouter ou alourdir une entrée `Rarity` ne
    /// déplace un emplacement que vers un candidat **de cette entrée** (pas vers l'objet d'une autre) ; une arme ou un
    /// tas de sous ajouté ne déplace que vers eux.
    #[test]
    fn rarity_weapon_and_currency_additions_stay_inside_what_they_bring() {
        const SLOTS: u32 = 5_000;
        let none = BTreeSet::new();
        let items = many_items(4, 3);
        let base = table(
            vec![
                entry(LootPick::Item("x0".into()), 2),
                entry(LootPick::Item("x1".into()), 1),
                entry(LootPick::Item("r0".into()), 2),
                entry(LootPick::Rarity(Rarity::Rare), 1),
            ],
            false,
        );
        let draws = |table: &LootTableDef| {
            (0..SLOTS)
                .map(|n| drawn(table, &items, &none, n))
                .collect::<Vec<_>>()
        };
        let before = draws(&base);
        let moved_to = |after: &[String]| -> BTreeSet<String> {
            after
                .iter()
                .zip(&before)
                .filter(|(now, was)| now != was)
                .map(|(now, _)| now.clone())
                .collect()
        };
        let rares: BTreeSet<String> = ["r0", "r1", "r2"].into_iter().map(String::from).collect();

        // Une entrée `Rarity` de même contenu, ajoutée : fusion, poids 1 + 3.
        let mut heavier = base.clone();
        heavier
            .entries
            .push(entry(LootPick::Rarity(Rarity::Rare), 3));
        let targets = moved_to(&draws(&heavier));
        assert!(
            !targets.is_empty() && targets.is_subset(&rares),
            "{targets:?}"
        );

        // Une entrée `Rarity` apportée à un pool qui n'en avait pas.
        let mut without = base.clone();
        without.entries.pop();
        let base_without = draws(&without);
        let mut with = without.clone();
        with.entries.push(entry(LootPick::Rarity(Rarity::Rare), 3));
        let moved: BTreeSet<String> = draws(&with)
            .into_iter()
            .zip(&base_without)
            .filter(|(now, was)| now != *was)
            .map(|(now, _)| now)
            .collect();
        assert!(!moved.is_empty() && moved.is_subset(&rares), "{moved:?}");

        // Une arme et un tas de sous : seulement eux.
        for (pick, expected) in [
            (
                LootPick::Weapon("lame".into(), Rarity::Common),
                "weapon:lame",
            ),
            (LootPick::Currency(5), "currency:5"),
        ] {
            let mut more = base.clone();
            more.entries.push(entry(pick, 2));
            let targets = moved_to(&draws(&more));
            assert_eq!(
                targets,
                BTreeSet::from([expected.to_string()]),
                "{expected}"
            );
        }
    }

    /// Poids plafonné : un poids énorme ne fait pas boucler le tirage (le lint le refuse).
    #[test]
    fn an_absurd_weight_is_capped() {
        let items = items();
        let pool = table(
            vec![
                entry(LootPick::Item("a".into()), i32::MAX),
                entry(LootPick::Item("key".into()), 1),
            ],
            false,
        );
        // Plafonné à MAX_LOOT_WEIGHT : `key` pèse 1/1001 ; le tirage se termine.
        let none = BTreeSet::new();
        let a = (0..2_000)
            .filter(|n| drawn(&pool, &items, &none, *n) == "a")
            .count();
        assert!(a > 1_990, "{a} tirages de `a` sur 2 000");
    }

    // ---- D95 : graines décorrélées entre clés voisines -------------------------------------------

    /// Anciennes graines de butin (`loot_seed` avant D95) : le repli FNV seul, sans finaliseur.
    fn legacy_seed(run_seed: u32, floor: u32, key: &str) -> u32 {
        let hash = fnv1a(format!("loot:{run_seed}:{floor}:{key}").as_bytes());
        (hash ^ (hash >> 32)) as u32
    }

    /// Fréquences de `hit(graine, i)` pour `keys` clés voisines (`i = 0…keys-1`) sur les graines
    /// `1…=seeds` : (fréquence min, fréquence max, |corrélation de Pearson| max entre clés voisines,
    /// part des graines où aucune clé ne touche).
    fn neighbour_stats(
        seeds: u32,
        keys: usize,
        hit: impl Fn(u32, usize) -> bool,
    ) -> (f64, f64, f64, f64) {
        let mut hits = vec![0u32; keys];
        let mut pairs = vec![0u32; keys - 1];
        let mut none = 0u32;
        for seed in 1..=seeds {
            let draws: Vec<bool> = (0..keys).map(|i| hit(seed, i)).collect();
            none += u32::from(draws.iter().all(|hit| !hit));
            for i in 0..keys {
                hits[i] += u32::from(draws[i]);
                if i + 1 < keys {
                    pairs[i] += u32::from(draws[i] && draws[i + 1]);
                }
            }
        }
        let n = f64::from(seeds);
        let freq: Vec<f64> = hits.iter().map(|h| f64::from(*h) / n).collect();
        let max_r = (0..keys - 1)
            .map(|i| {
                let covariance = f64::from(pairs[i]) / n - freq[i] * freq[i + 1];
                let variance = freq[i] * (1.0 - freq[i]) * freq[i + 1] * (1.0 - freq[i + 1]);
                (covariance / variance.sqrt()).abs()
            })
            .fold(0.0, f64::max);
        (
            freq.iter().copied().fold(1.0, f64::min),
            freq.iter().copied().fold(0.0, f64::max),
            max_r,
            f64::from(none) / n,
        )
    }

    /// Un pool de dix armes de poids 1 (aucun objet nécessaire) : `weapon:w0` sort avec p = 0,1.
    fn ten_weapons() -> LootTableDef {
        table(
            (0..10)
                .map(|i| entry(LootPick::Weapon(format!("w{i}"), Rarity::Common), 1))
                .collect(),
            false,
        )
    }

    /// Les familles de clés des sites de butin, avec le nombre qui varie en dernier ou non.
    fn key_families() -> Vec<(&'static str, Box<dyn Fn(usize) -> String>)> {
        vec![
            (
                "champion:100+i",
                Box::new(|i| format!("champion:{}", 100 + i)),
            ),
            (
                "champion:1000+i",
                Box::new(|i| format!("champion:{}", 1000 + i)),
            ),
            ("shells:100+i", Box::new(|i| format!("shells:{}", 100 + i))),
            ("shells:37+i", Box::new(|i| format!("shells:{}", 37 + i))),
            (
                "rock:3:10:5+i",
                Box::new(|i| format!("rock:3:10:{}", 5 + i)),
            ),
            ("rock:3:2+i:5", Box::new(|i| format!("rock:3:{}:5", 2 + i))),
            ("slot:i", Box::new(|i| format!("room:salle_{i}"))),
        ]
    }

    /// D95, mesure refaite en Rust sur le modèle de `variant::tests` : 16 clés voisines, deux indicateurs de
    /// p = 0,1 par clé : la première valeur du flux (`next_fixed < 0,1`, comme la chance d'une curse ou d'un
    /// champion ; 100 000 graines, bruit d'échantillon de r ≈ 0,003) et le tirage de butin entier (la course ;
    /// 20 000 graines, bruit ≈ 0,007). La fréquence de **chaque** clé est vérifiée, la dernière comprise. L'ancien repli
    /// FNV seul doit, lui, échouer.
    #[test]
    fn loot_seeds_of_neighbouring_keys_are_independent() {
        const FIRST_SEEDS: u32 = 100_000;
        const SEEDS: u32 = 20_000;
        const KEYS: usize = 16;
        let chance = Fixed::from_num(0.1);
        let pool = ten_weapons();
        let items = ItemTable::default();
        let mut worst_new = 0.0f64;
        let mut worst_old = 0.0f64;
        for (name, key) in key_families() {
            for (label, seed_of) in [
                (
                    "avant",
                    &(|s: u32, i: usize| legacy_seed(s, 0, &key(i))) as &dyn Fn(u32, usize) -> u32,
                ),
                ("après", &|s: u32, i: usize| loot_seed(s, 0, &key(i))),
            ] {
                let first = neighbour_stats(FIRST_SEEDS, KEYS, |s, i| {
                    RollbackRng::new(seed_of(s, i)).next_fixed() < chance
                });
                let full = neighbour_stats(SEEDS, KEYS, |s, i| {
                    draw_loot(
                        &pool,
                        &items,
                        None,
                        Rarity::Common,
                        false,
                        &mut RollbackRng::new(seed_of(s, i)),
                    )
                    .as_deref()
                        == Some("weapon:w0")
                });
                eprintln!(
                    "D95 {label} {name:>16} : 1er tirage f∈[{:.4},{:.4}] r={:.4} sans={:.3} | course f∈[{:.4},{:.4}] r={:.4} sans={:.3}",
                    first.0, first.1, first.2, first.3, full.0, full.1, full.2, full.3
                );
                let r = first.2.max(full.2);
                if label == "avant" {
                    worst_old = worst_old.max(r);
                    continue;
                }
                worst_new = worst_new.max(r);
                for (kind, stats) in [("1er tirage", first), ("course", full)] {
                    // Chaque clé, la dernière comprise : la fréquence mesurée reste proche de 0,1.
                    assert!(
                        stats.0 > 0.08 && stats.1 < 0.12,
                        "{name} ({kind}) : fréquences [{:.4}, {:.4}]",
                        stats.0,
                        stats.1
                    );
                    assert!(
                        stats.2 < 0.03,
                        "{name} ({kind}) : clés voisines corrélées, r = {:.4}",
                        stats.2
                    );
                    // 0,9^16 = 18,5 % des graines sans aucune occurrence.
                    assert!(
                        (0.15..=0.25).contains(&stats.3),
                        "{name} ({kind}) : {:.3} des graines sans occurrence",
                        stats.3
                    );
                }
            }
        }
        eprintln!("D95 : |r| max avant {worst_old:.4}, après {worst_new:.4}");
        assert!(
            worst_old > 0.03,
            "le détecteur ne voit plus l'ancien défaut (r = {worst_old:.4})"
        );
    }

    /// Les graines sans allocation sont celles des clés textuelles (mêmes octets, mêmes graines).
    #[test]
    fn allocation_free_seeds_equal_the_textual_keys() {
        for (run_seed, floor, number) in [
            (0u32, 0u32, 0u64),
            (1, 9, 100),
            (123_456, 3, 1_000),
            (u32::MAX, u32::MAX, u64::MAX),
        ] {
            let expect = |key: String| loot_seed(run_seed, floor, &key);
            assert_eq!(
                loot_seed_numbered(run_seed, floor, "shells:", number),
                expect(format!("shells:{number}"))
            );
            assert_eq!(
                loot_seed_parts(run_seed, floor, &["shop:", "salle_a"]),
                expect("shop:salle_a".to_string())
            );
            for index in [1u32, 2, 17, u32::MAX] {
                assert_eq!(
                    exhaust_seed(run_seed, floor, "room:x", index),
                    expect(format!("room:x#{index}")),
                    "{run_seed}:{floor} index {index}"
                );
            }
            assert_eq!(
                exhaust_seed(run_seed, floor, "room:x", 0),
                expect("room:x".to_string())
            );
        }
    }
}
