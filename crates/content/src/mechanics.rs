//! M3-T12 (`docs/conventions.md` §77) : atteignabilité statique des mécaniques de M3 (larmes, effets,
//! familiers, bombes, salle secrète, clés, cœurs, pools à épuisement, marché du diable, champions,
//! curses, accomplissements). Lecture du contenu seule, aucune partie jouée ; tout est avertissement
//! (`LintWarningKind::MechanicUnmet`).
//!
//! Deux familles de règles :
//! - **toujours actives** (le contenu est là, rien ne le porte) : pool qu'aucune salle ne tire,
//!   familier dont aucun objet n'a de source, salle spéciale vide, curse jamais tirée, accomplissement
//!   inaccomplissable, déblocage jamais accordé, synergie à objet sans source, mur secret sans bombe ;
//! - **déclarées** par `mechanics` du manifeste : la ligne du tableau « Atteignabilité » de
//!   `docs/taches.md` §8.1, vérifiée sur la **partie ordinaire** (la séquence `entry.floors`, sans
//!   `FloorsOverride`). Sans déclaration, aucune vérification : le contenu qui n'existe pas encore n'est
//!   pas signalé.
//!
//! Les règles toujours actives comptent **toute** séquence déclarée comme racine (comme §53) ; les
//! règles déclarées ne comptent que la partie ordinaire.

use std::collections::{BTreeMap, BTreeSet};

use bevy_fixed::fixed_math::Fixed;
use sim_core::stats::StatId;

use crate::expr::NumOrExpr;
use crate::manifest::{EntryMode, GameManifest, Mechanic};
use crate::reach::{
    item_sources, playable_levels, pool_items, reach_from, reachable_characters,
    unlock_grant_sources, LintWarning, LintWarningKind, Reach,
};
use crate::registry::{self, Registry};

const MANIFEST_FILE: &str = "game.ron";
/// Consommable que lâche une bombe (`game::loot::BOMB_ITEM`, que `content` ne peut pas importer).
const BOMB: &str = "bomb";
/// Consommable qui ouvre une porte `Key` (`game::loot::KEY_ITEM`).
const KEY: &str = "key";

/// Nom français d'une mécanique, pour les messages.
pub(crate) fn label(mechanic: Mechanic) -> &'static str {
    match mechanic {
        Mechanic::Tears => "larmes",
        Mechanic::Effects => "effets et transformations",
        Mechanic::Familiars => "familiers",
        Mechanic::Bombs => "bombes, rochers et piques",
        Mechanic::SecretRoom => "salle secrète",
        Mechanic::Keys => "clés et trésor fermé",
        Mechanic::Hearts => "cœurs, sous et boutique",
        Mechanic::ExhaustPools => "pools à épuisement",
        Mechanic::DevilMarket => "marché du diable",
        Mechanic::Champions => "champions",
        Mechanic::Curses => "curses",
        Mechanic::Achievements => "accomplissements",
    }
}

fn is_tear_item(item: &registry::ItemEntry) -> bool {
    item.def.projectile.is_some()
        || item
            .def
            .modifiers
            .iter()
            .any(|m| matches!(m.stat, StatId::ShotSpeed | StatId::ShotSize))
}

/// L'action sous ses enveloppes (`Scaled`, `Chance`).
fn innermost(action: &effects::Action) -> &effects::Action {
    match action {
        effects::Action::Scaled { action, .. } => innermost(action),
        effects::Action::Chance { then, .. } => innermost(then),
        other => other,
    }
}

fn heals(item: &registry::ItemEntry) -> bool {
    item.def
        .effects
        .iter()
        .flat_map(|effect| &effect.r#do)
        .any(|action| {
            matches!(
                innermost(action),
                effects::Action::Heal(_) | effects::Action::GainSoul(_)
            )
        })
}

/// Mécaniques (noms français) qu'un objet porte, pour les messages « aucune source ».
pub(crate) fn item_mechanics(item: &registry::ItemEntry) -> Vec<&'static str> {
    let mut out = Vec::new();
    if is_tear_item(item) {
        out.push("larmes");
    }
    if item.def.familiar.is_some() {
        out.push("familiers");
    }
    if item.def.kind != items::ItemKind::Consumable && !item.def.effects.is_empty() {
        out.push("effets");
    }
    if item.id == BOMB {
        out.push("bombes");
    }
    if item.id == KEY {
        out.push("clés");
    }
    if heals(item) {
        out.push("cœurs");
    }
    out
}

/// Boutique au sens des règles déclarées : une salle à `shop` dont le prix n'est pas la vie maximale (le
/// marché du diable, `cost: MaxHealth`, a sa règle `DevilMarket`).
fn is_shop(def: &world::RoomKindDef) -> bool {
    def.shop
        .as_ref()
        .is_some_and(|shop| !matches!(shop.cost, Some(items::Price::MaxHealth(_))))
}

/// Mécanique qu'un type de salle porte, pour les messages « aucune grammaire ne peut le tirer ».
pub(crate) fn room_mechanic(def: &world::RoomKindDef) -> Option<&'static str> {
    match def.door_lock {
        Some(world::DoorLock::Secret) => Some("salle secrète (murée, ouverte à la bombe)"),
        Some(world::DoorLock::Key(_)) => Some("trésor fermé à clé"),
        None if def.shop.as_ref().is_some_and(|shop| shop.cost.is_some()) => {
            Some("marché payant (présentoirs à `cost`)")
        }
        None if def.shop.is_some() => Some("boutique"),
        None if def.boss_drop.is_some() => Some("butin du boss"),
        None if def.arena.is_some() => Some("arène"),
        None if def.chest.is_some() => Some("coffre"),
        None => None,
    }
}

/// Une partie : racines, séquences et ce qu'elles atteignent.
struct Play<'a> {
    registry: &'a Registry,
    designations: Vec<&'a str>,
    sequences: Vec<&'a registry::FloorsEntry>,
    reach: Reach<'a>,
}

/// Un site qui tire un pool de butin.
struct Draw {
    pool: String,
    site: String,
    /// Tirages possibles par partie (au moins).
    times: u32,
}

impl<'a> Play<'a> {
    fn new(
        registry: &'a Registry,
        designations: Vec<&'a str>,
        sequences: Vec<&'a registry::FloorsEntry>,
    ) -> Self {
        let reach = reach_from(registry, designations.clone());
        Self {
            registry,
            designations,
            sequences,
            reach,
        }
    }

    /// Toute séquence déclarée est une racine (§53).
    fn everything(registry: &'a Registry, manifest: &'a GameManifest) -> Self {
        let mut designations = vec![manifest.entry.start_map.as_str()];
        for floors in registry.floors.values() {
            designations.extend(floors.levels.iter().map(String::as_str));
        }
        Self::new(registry, designations, registry.floors.values().collect())
    }

    /// La partie ordinaire : en mode `Floors`, la séquence de `entry.floors` (à défaut la première par
    /// ordre d'id, comme le jeu) ; sinon la carte de départ.
    fn ordinary(registry: &'a Registry, manifest: &'a GameManifest) -> Self {
        if manifest.entry.mode == Some(EntryMode::Floors) {
            let sequence = match &manifest.entry.floors {
                Some(id) => registry.floors.values().find(|f| f.id.as_str() == id),
                None => registry.floors.values().next(),
            };
            if let Some(sequence) = sequence {
                return Self::new(
                    registry,
                    sequence.levels.iter().map(String::as_str).collect(),
                    vec![sequence],
                );
            }
        }
        Self::new(registry, vec![manifest.entry.start_map.as_str()], vec![])
    }

    /// Grammaires atteintes et nombre d'étages de la partie qui les désignent.
    fn grammars(&self) -> Vec<(&'a registry::FloorGrammarEntry, u32)> {
        self.reach
            .grammars
            .iter()
            .map(|id| {
                let floors = self
                    .designations
                    .iter()
                    .filter(|d| registry::floor_designation(d).as_ref() == Some(*id))
                    .count() as u32;
                (&self.registry.floor_grammars[*id], floors)
            })
            .collect()
    }

    /// Salles de ce type qu'une partie peut poser (au moins ; le remplissage compte pour trois).
    fn draws_of(&self, kind: &str) -> u32 {
        let mut n = 0;
        for (entry, floors) in self.grammars() {
            let g = &entry.grammar;
            let post = g
                .post_boss()
                .is_some_and(|post| post.kind == kind && post.chance > Fixed::ZERO);
            let per_floor = if kind == g.start || kind == g.boss || post {
                1
            } else if let Some(required) = g.required.get(kind) {
                *required as u32
            } else if kind == g.filler {
                3
            } else {
                0
            };
            n += per_floor * floors;
        }
        for map_id in &self.reach.maps {
            if let Some(map) = self.registry.maps.get(map_id) {
                n += map.room_kinds.iter().filter(|(_, k)| k == kind).count() as u32;
            }
        }
        n
    }

    /// Pools de butin des étages de la partie (`loot` des séquences), sans doublon.
    fn floor_tables(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for sequence in &self.sequences {
            for table in &sequence.loot {
                if !out.contains(table) {
                    out.push(table.clone());
                }
            }
        }
        out
    }

    /// Pool d'un coffre sans `table` : celui de l'étage de chaque séquence, `commun` pour une séquence
    /// sans `loot` (`game::loot::chest_table_id`).
    fn chest_tables(&self, chest: &items::RoomChestDef) -> Vec<String> {
        if let Some(table) = &chest.table {
            return vec![table.clone()];
        }
        let default = || items::DEFAULT_CHEST_TABLE.to_string();
        let mut out: Vec<String> = Vec::new();
        if self.sequences.is_empty() {
            out.push(default());
        }
        for sequence in &self.sequences {
            let tables = if sequence.loot.is_empty() {
                vec![default()]
            } else {
                sequence.loot.clone()
            };
            for table in tables {
                if !out.contains(&table) {
                    out.push(table);
                }
            }
        }
        out
    }

    /// Pools de la boutique : `floor_tables` quand il y en a (le jeu prend `floor_tables[min(étage, dernier)]`,
    /// `RoomShopDef::table_for_floor`), sinon `table`.
    fn shop_tables(shop: &items::RoomShopDef) -> Vec<String> {
        if shop.floor_tables.is_empty() {
            vec![shop.table.clone()]
        } else {
            shop.floor_tables.clone()
        }
    }

    /// Pools de la récompense des salles `combat` : le pool de l'étage s'il définit `reward`, sinon
    /// `salle` (`game::loot::room_reward_system`), pour chaque séquence de la partie ; `salle` seul sans
    /// séquence.
    fn reward_tables(&self) -> Vec<String> {
        let default = || items::DEFAULT_REWARD_TABLE.to_string();
        let rewards = |pool: &str| {
            self.registry
                .loot_tables
                .get(pool)
                .is_some_and(|t| t.def.reward.is_some())
        };
        let mut out: Vec<String> = Vec::new();
        let mut push = |pool: String| {
            if !out.contains(&pool) {
                out.push(pool);
            }
        };
        if self.sequences.is_empty() {
            push(default());
        }
        for sequence in &self.sequences {
            if sequence.loot.is_empty() {
                push(default());
            }
            for pool in &sequence.loot {
                push(if rewards(pool) {
                    pool.clone()
                } else {
                    default()
                });
            }
        }
        out
    }

    /// Pools que tirent les salles de ces types (coffre, boutique) : pool → types.
    fn kinds_pools(&self, kinds: &BTreeSet<String>) -> BTreeMap<String, Vec<String>> {
        let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for kind in kinds {
            let Some(room) = self.registry.rooms.get(kind) else {
                continue;
            };
            let mut pools = Vec::new();
            if let Some(chest) = &room.def.chest {
                pools.extend(self.chest_tables(chest));
            }
            if let Some(drop) = &room.def.boss_drop {
                pools.push(drop.table.clone());
            }
            if let Some(shop) = &room.def.shop {
                pools.extend(Self::shop_tables(shop));
            }
            for pool in pools {
                if self.registry.loot_tables.contains_key(&pool) {
                    out.entry(pool).or_default().push(kind.clone());
                }
            }
        }
        out
    }

    /// Tout ce qui tire un pool : coffres et boutiques des salles atteintes, récompense des salles
    /// `combat` (et son coffre), rochers à butin des niveaux joués, butin des champions des ennemis atteints.
    fn draws(&self) -> Vec<Draw> {
        let registry = self.registry;
        let known = |pool: &str| registry.loot_tables.contains_key(pool);
        let mut out = Vec::new();
        for room in registry.rooms.values() {
            if !self.reach.room_kinds.contains(&room.id) {
                continue;
            }
            let times = self.draws_of(&room.id).max(1);
            if let Some(chest) = &room.def.chest {
                for pool in self.chest_tables(chest).into_iter().filter(|p| known(p)) {
                    out.push(Draw {
                        pool,
                        site: format!("coffre de la salle « {} »", room.id),
                        times,
                    });
                }
            }
            if let Some(drop) = &room.def.boss_drop {
                if known(&drop.table) {
                    out.push(Draw {
                        pool: drop.table.clone(),
                        site: format!("butin du boss de la salle « {} »", room.id),
                        times,
                    });
                }
            }
            if let Some(shop) = &room.def.shop {
                for pool in Self::shop_tables(shop).into_iter().filter(|p| known(p)) {
                    out.push(Draw {
                        pool,
                        site: format!("boutique de la salle « {} »", room.id),
                        times,
                    });
                }
            }
        }
        let floor_tables = self.floor_tables();
        if self.reach.room_kinds.contains("combat") {
            for pool in self.reward_tables().into_iter().filter(|p| known(p)) {
                // Un coffre de récompense (`chest_chance > 0`) tire le pool de l'étage, sinon `commun`.
                let chest = registry.loot_tables[&pool]
                    .def
                    .reward
                    .is_some_and(|reward| reward.chest_chance > Fixed::ZERO);
                out.push(Draw {
                    pool,
                    site: "récompense des salles « combat »".to_string(),
                    times: 3,
                });
                if chest {
                    let anywhere = items::RoomChestDef {
                        table: None,
                        locked: false,
                        rarity: items::Rarity::Common,
                    };
                    for pool in self
                        .chest_tables(&anywhere)
                        .into_iter()
                        .filter(|p| known(p))
                    {
                        out.push(Draw {
                            pool,
                            site: "coffre de la récompense des salles « combat »".to_string(),
                            times: 3,
                        });
                    }
                }
            }
        }
        let rocks: u32 = playable_levels(registry, &self.reach)
            .iter()
            .map(|level| level.loot_rocks)
            .sum();
        if rocks > 0 {
            let known_floor: Vec<&String> = floor_tables.iter().filter(|p| known(p)).collect();
            let pools: Vec<String> = if known_floor.is_empty() {
                vec![items::DEFAULT_REWARD_TABLE.to_string()]
            } else {
                known_floor.into_iter().cloned().collect()
            };
            for pool in pools.into_iter().filter(|p| known(p)) {
                out.push(Draw {
                    pool,
                    site: "rochers à butin des niveaux joués".to_string(),
                    times: rocks,
                });
            }
        }
        for id in reachable_characters(registry, &self.reach) {
            let Some(character) = registry
                .characters
                .get(&registry::CharacterId::from(id.clone()))
            else {
                continue;
            };
            let Some(variants) = character
                .variants
                .as_ref()
                .filter(|v| variant_chance_possible(&v.chance))
            else {
                continue;
            };
            for (name, variant) in &variants.table {
                if let Some(pool) = variant.loot.as_ref().filter(|p| known(p)) {
                    out.push(Draw {
                        pool: pool.clone(),
                        site: format!("champion « {name} » de « {id} »"),
                        times: 2,
                    });
                }
            }
        }
        out
    }
}

/// Objets que donnent ces pools.
fn items_in(
    registry: &Registry,
    pools: impl IntoIterator<Item = impl AsRef<str>>,
) -> BTreeSet<String> {
    pools
        .into_iter()
        .filter_map(|pool| registry.loot_tables.get(pool.as_ref()))
        .flat_map(|table| pool_items(registry, &table.def))
        .collect()
}

fn pool_gives(registry: &Registry, pool: &str, pick: impl Fn(&items::LootPick) -> bool) -> bool {
    registry
        .loot_tables
        .get(pool)
        .is_some_and(|table| table.def.entries.iter().any(|entry| pick(&entry.pick)))
}

/// Une variante peut-elle sortir (chance non nulle pour au moins un nombre de joueurs et un étage) ?
fn variant_chance_possible(chance: &NumOrExpr) -> bool {
    [(1u32, 0u32), (4, 0), (1, 5), (4, 5)]
        .iter()
        .any(|(players, floor)| {
            let ctx = BTreeMap::from([
                ("players".to_string(), Fixed::from_num(*players)),
                ("floor".to_string(), Fixed::from_num(*floor)),
            ]);
            matches!(chance.resolve_with(&ctx), Ok(value) if value > Fixed::ZERO)
        })
}

fn players(registry: &Registry) -> impl Iterator<Item = &registry::CharacterEntry> {
    registry
        .characters
        .values()
        .filter(|c| !c.has_ai && !c.test_only)
}

/// Une explosion qui casse les murs est atteignable : bombe jouable (stock **et** personnage jouable à
/// `bomb`) ou objet source à `Explode { destroys: true }`, même enveloppé (`Scaled`, `Chance`).
fn explosion_source(registry: &Registry, play: &Play<'_>, sources: &BTreeSet<String>) -> bool {
    (players(registry).any(|c| c.bomb.is_some()) && bomb_obtainable(registry, play))
        || registry
            .items
            .values()
            .filter(|item| sources.contains(&item.id))
            .flat_map(|item| &item.def.effects)
            .flat_map(|effect| &effect.r#do)
            .any(|action| {
                matches!(
                    innermost(action),
                    effects::Action::Explode { destroys: true, .. }
                )
            })
}

pub(crate) fn warnings(registry: &Registry, manifest: &GameManifest) -> Vec<LintWarning> {
    let mut out = Vec::new();
    let all = Play::everything(registry, manifest);
    let ordinary = Play::ordinary(registry, manifest);
    let sources = item_sources(registry);
    always_on(registry, &all, &sources, &mut out);
    declared(registry, manifest, &ordinary, &mut out);
    out
}

fn unmet(out: &mut Vec<LintWarning>, mechanic: &str, file: impl Into<String>, message: String) {
    out.push(LintWarning {
        kind: LintWarningKind::MechanicUnmet,
        file: file.into(),
        message: format!("mécanique « {mechanic} » : {message}"),
    });
}

/// Règles qui ne demandent aucune déclaration : un contenu de mécanique M3 que rien ne porte.
fn always_on(
    registry: &Registry,
    all: &Play<'_>,
    sources: &BTreeSet<String>,
    out: &mut Vec<LintWarning>,
) {
    let draws = all.draws();
    let drawn: BTreeSet<&str> = draws.iter().map(|d| d.pool.as_str()).collect();

    // Pool qu'aucune salle, séquence, rocher ni champion ne tire.
    for pool in registry.loot_tables.values() {
        if !drawn.contains(pool.id.as_str()) {
            let given: Vec<String> = pool_items(registry, &pool.def).into_iter().collect();
            unmet(
                out,
                "pools",
                pool.file.display().to_string(),
                format!(
                    "pool « {} » : aucune salle (coffre, boutique, récompense de combat), séquence, rocher à butin ni champion ne le tire{} ; une salle trésor, diable ou secrète (`chest`/`shop` de rooms/<type>.ron), un rocher à butin ou une variante (`loot`) doit le porter",
                    pool.id,
                    if given.is_empty() {
                        String::new()
                    } else {
                        format!(" (objets : {})", given.join(", "))
                    }
                ),
            );
        }
    }

    // Objets d'un pool tiré, de départ ou d'étage (hors pool jamais tiré) : une source atteignable.
    let mut reachable = items_in(registry, drawn.iter().copied());
    for c in players(registry) {
        reachable.extend(c.starting_consumables.keys().cloned());
    }
    for sequence in &all.sequences {
        reachable.extend(sequence.consumables_per_floor.keys().cloned());
    }
    reachable.extend(crate::reach::unlock_grant_sources(registry).0);

    // Familier qu'aucun objet atteignable ne cite.
    let mut by_character: BTreeMap<&str, Vec<&registry::ItemEntry>> = BTreeMap::new();
    for item in registry.items.values() {
        if let Some(familiar) = &item.def.familiar {
            by_character
                .entry(familiar.character.as_str())
                .or_default()
                .push(item);
        }
    }
    for (character, citing) in &by_character {
        // D63 : un personnage de test ne porte rien à une partie ordinaire.
        if registry
            .characters
            .get(&registry::CharacterId::from(character.to_string()))
            .is_some_and(|c| c.test_only)
        {
            continue;
        }
        if citing.iter().all(|item| !reachable.contains(&item.id)) {
            let file = registry
                .characters
                .get(&registry::CharacterId::from(character.to_string()))
                .map(|c| c.file.display().to_string())
                .unwrap_or_default();
            let names: Vec<&str> = citing.iter().map(|item| item.id.as_str()).collect();
            unmet(
                out,
                "familiers",
                file,
                format!(
                    "familier « {character} » : aucun objet atteignable ne le porte (objet(s) : {}, sans source) ; un pool de trésor ou de boutique doit en donner un",
                    names.join(", ")
                ),
            );
        }
    }

    // Synergie (transformation) dont un objet requis n'a aucune source.
    for synergy in registry.synergies.values() {
        let missing: Vec<&str> = synergy
            .def
            .requires
            .iter()
            .filter(|id| registry.items.contains_key(*id) && !sources.contains(*id))
            .map(String::as_str)
            .collect();
        if !missing.is_empty() {
            unmet(
                out,
                "effets et transformations",
                synergy.file.display().to_string(),
                format!(
                    "synergie « {} » : objet(s) requis sans source ({}) : jamais active",
                    synergy.id,
                    missing.join(", ")
                ),
            );
        }
    }

    // Salles spéciales d'une grammaire en grille qui ne portent rien.
    let mut seen = BTreeSet::new();
    for (entry, _) in all.grammars() {
        let g = &entry.grammar;
        let mut specials: Vec<(String, &str)> = Vec::new();
        if let world::FloorLayout::Grid {
            secret, post_boss, ..
        } = g.layout()
        {
            if g.required.contains_key(&secret) {
                specials.push((secret, "salle secrète"));
            }
            if let Some(post) = post_boss.filter(|p| p.chance > Fixed::ZERO) {
                specials.push((post.kind, "marché du diable (salle post-boss)"));
            }
            for kind in g.required.keys() {
                if *kind != g.start && *kind != g.boss && !specials.iter().any(|(k, _)| k == kind) {
                    specials.push((kind.clone(), "salle exigée"));
                }
            }
        }
        for (kind, role) in specials {
            let Some(room) = registry.rooms.get(&kind) else {
                continue;
            };
            let def = &room.def;
            if def.chest.is_none()
                && def.shop.is_none()
                && def.arena.is_none()
                && def.boss_drop.is_none()
                && !def.locks
            {
                if seen.insert(kind.clone()) {
                    unmet(
                        out,
                        role,
                        room.file.display().to_string(),
                        format!(
                            "type de salle « {kind} » posé par la grammaire « {} » mais sans coffre, boutique ni arène : aucun objet de pool n'y est tiré ; ajouter `chest` ou `shop` à rooms/{kind}.ron",
                            entry.id
                        ),
                    );
                }
            }
        }
    }

    // Mur secret sans explosion atteignable.
    let secret_kinds: Vec<&registry::RoomEntry> = registry
        .rooms
        .values()
        .filter(|room| {
            room.def.door_lock == Some(world::DoorLock::Secret)
                && all.reach.room_kinds.contains(&room.id)
        })
        .collect();
    if !secret_kinds.is_empty() && !explosion_source(registry, all, sources) {
        for room in secret_kinds {
            unmet(
                out,
                "salle secrète",
                room.file.display().to_string(),
                format!(
                    "type de salle « {} » : mur secret atteint mais aucune bombe (consommable « {BOMB} » et personnage à `bomb`) ni objet à `Explode {{ destroys: true }}` n'a de source : jamais ouvert",
                    room.id
                ),
            );
        }
    }

    // Curse qu'aucune séquence ne tire.
    for curse in registry.curses.values() {
        let drawn = all.sequences.iter().any(|s| {
            s.curses
                .as_ref()
                .is_some_and(|c| c.chance > Fixed::ZERO && c.pool.contains(&curse.id))
        });
        if !drawn {
            unmet(
                out,
                "curses",
                curse.file.display().to_string(),
                format!(
                    "curse « {} » : aucune séquence `Floors` ne la met dans `curses.pool` (ou la chance est 0) : jamais tirée",
                    curse.id
                ),
            );
        }
    }

    // Variante à chance toujours nulle.
    for character in registry.characters.values().filter(|c| !c.test_only) {
        if let Some(variants) = &character.variants {
            if !variants.table.is_empty() && !variant_chance_possible(&variants.chance) {
                unmet(
                    out,
                    "champions",
                    character.file.display().to_string(),
                    format!(
                        "personnage « {} » : `variants.chance` nulle pour 1 et 4 joueurs aux étages 0 et 5 : aucun champion ne sort",
                        character.id
                    ),
                );
            }
        }
    }

    achievements(registry, all, sources, out);
}

/// Ce que les conditions d'un accomplissement lisent, pour une partie.
struct Facts<'a> {
    characters: BTreeSet<String>,
    /// Nombre d'étages de la plus longue séquence quand toutes finissent (`victory_at_end`).
    finite_floors: Option<usize>,
    sources: &'a BTreeSet<String>,
}

impl<'a> Facts<'a> {
    fn new(registry: &Registry, play: &Play<'_>, sources: &'a BTreeSet<String>) -> Self {
        let finite_floors = (!play.sequences.is_empty()
            && play.sequences.iter().all(|s| s.victory_at_end))
        .then(|| {
            play.sequences
                .iter()
                .map(|s| s.levels.len())
                .max()
                .unwrap_or(0)
        });
        Self {
            characters: reachable_characters(registry, &play.reach),
            finite_floors,
            sources,
        }
    }

    /// Pourquoi la condition ne peut pas être remplie, `None` si elle le peut.
    fn problem(&self, condition: &meta::AchievementCondition) -> Option<String> {
        match condition {
            meta::AchievementCondition::Victory => None,
            meta::AchievementCondition::FloorAtLeast(n) => self
                .finite_floors
                .filter(|len| *n as usize >= *len)
                .map(|len| {
                    format!(
                        "FloorAtLeast({n}) : les séquences se terminent à l'étage {} (victory_at_end)",
                        len.saturating_sub(1)
                    )
                }),
            meta::AchievementCondition::BossDefeated(id) => (!self.characters.contains(id))
                .then(|| format!("BossDefeated(« {id} ») : ce boss n'apparaît dans aucune partie")),
            meta::AchievementCondition::ItemHeld(id) => (!self.sources.contains(id))
                .then(|| format!("ItemHeld(« {id} ») : cet objet n'a aucune source")),
        }
    }
}

/// Accomplissements inaccomplissables et déblocages jamais accordés.
fn achievements(
    registry: &Registry,
    all: &Play<'_>,
    sources: &BTreeSet<String>,
    out: &mut Vec<LintWarning>,
) {
    let facts = Facts::new(registry, all, sources);
    for achievement in registry.achievements.values() {
        for condition in &achievement.def.when {
            if let Some(problem) = facts.problem(condition) {
                unmet(
                    out,
                    "accomplissements",
                    achievement.file.display().to_string(),
                    format!(
                        "accomplissement « {} » : condition inaccomplissable, {problem}",
                        achievement.id
                    ),
                );
            }
        }
    }
    let merchant = registry
        .hub_services
        .values()
        .any(|s| s.def.action == meta::HubAction::Merchant);
    let granted: BTreeSet<&str> = registry
        .achievements
        .values()
        .flat_map(|a| a.def.unlocks.iter().map(String::as_str))
        .collect();
    for unlock in registry.unlocks.values() {
        if !merchant && !granted.contains(unlock.id.as_str()) {
            unmet(
                out,
                "accomplissements",
                unlock.file.display().to_string(),
                format!(
                    "déblocage « {} » : aucun accomplissement ni service `Merchant` du hub ne l'accorde",
                    unlock.id
                ),
            );
        }
    }
}

/// Règles de chaque mécanique déclarée, sur la partie ordinaire.
fn declared(
    registry: &Registry,
    manifest: &GameManifest,
    ordinary: &Play<'_>,
    out: &mut Vec<LintWarning>,
) {
    if manifest.mechanics.is_empty() {
        return;
    }
    let draws = ordinary.draws();
    let drawn: BTreeSet<String> = draws.iter().map(|d| d.pool.clone()).collect();
    let grammars = ordinary.grammars();

    // Types de salle par rôle, dans les grammaires de la partie ordinaire.
    let mut treasure: BTreeSet<String> = BTreeSet::new();
    let mut bosses: BTreeSet<String> = BTreeSet::new();
    let mut required_shops: BTreeSet<String> = BTreeSet::new();
    let mut required_locks: BTreeSet<String> = BTreeSet::new();
    for (entry, _) in &grammars {
        let g = &entry.grammar;
        let secret = match g.layout() {
            world::FloorLayout::Grid { secret, .. } => Some(secret),
            world::FloorLayout::Graph => None,
        };
        bosses.insert(g.boss.clone());
        for kind in g.required.keys() {
            let Some(room) = registry.rooms.get(kind) else {
                continue;
            };
            if is_shop(&room.def) {
                required_shops.insert(kind.clone());
            }
            if matches!(room.def.door_lock, Some(world::DoorLock::Key(_))) {
                required_locks.insert(kind.clone());
            }
            if *kind != g.start
                && *kind != g.boss
                && Some(kind) != secret.as_ref()
                && room.def.chest.is_some()
            {
                treasure.insert(kind.clone());
            }
        }
    }
    let shops: BTreeSet<String> = registry
        .rooms
        .values()
        .filter(|room| is_shop(&room.def) && ordinary.reach.room_kinds.contains(&room.id))
        .map(|room| room.id.clone())
        .collect();
    let treasure_pools = ordinary.kinds_pools(&treasure);
    let shop_pools = ordinary.kinds_pools(&shops);
    let boss_pools = ordinary.kinds_pools(&bosses);
    let treasure_items = items_in(registry, treasure_pools.keys());
    let shop_items = items_in(registry, shop_pools.keys());
    let boss_items = items_in(registry, boss_pools.keys());
    let kinds_list = |kinds: &BTreeSet<String>| {
        if kinds.is_empty() {
            "aucune".to_string()
        } else {
            kinds.iter().cloned().collect::<Vec<_>>().join(", ")
        }
    };
    let pools_list = |pools: &BTreeMap<String, Vec<String>>| {
        if pools.is_empty() {
            "aucun".to_string()
        } else {
            pools.keys().cloned().collect::<Vec<_>>().join(", ")
        }
    };
    let treasure_where = format!(
        "salle(s) exigée(s) par la grammaire à coffre : {} ; pool(s) : {}",
        kinds_list(&treasure),
        pools_list(&treasure_pools)
    );
    let shop_where = format!(
        "boutique(s) atteinte(s) : {} ; pool(s) : {}",
        kinds_list(&shops),
        pools_list(&shop_pools)
    );

    // Les pools du jeu (sans les armes) par groupe, pour la récompense de salle.
    let reward_pools: BTreeSet<&str> = draws
        .iter()
        .filter(|d| d.site.starts_with("récompense"))
        .map(|d| d.pool.as_str())
        .collect();
    let reward_items = items_in(registry, reward_pools.iter().copied());
    let drawn_items = items_in(registry, &drawn);
    let mut obtainable = drawn_items.clone();
    for c in players(registry) {
        obtainable.extend(c.starting_consumables.keys().cloned());
    }
    for sequence in &ordinary.sequences {
        obtainable.extend(sequence.consumables_per_floor.keys().cloned());
    }

    for mechanic in manifest.mechanics.iter().copied() {
        let name = label(mechanic);
        let mut warn = |file: &str, message: String| unmet(out, name, file, message);
        match mechanic {
            Mechanic::Tears => {
                let tears: Vec<&registry::ItemEntry> = registry
                    .items
                    .values()
                    .filter(|item| is_tear_item(item))
                    .collect();
                if !tears.iter().any(|item| treasure_items.contains(&item.id)) {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucun objet de larmes (`projectile`, `ShotSpeed`, `ShotSize`) dans le pool trésor de la partie ordinaire ({treasure_where})"
                        ),
                    );
                }
                if !tears.iter().any(|item| shop_items.contains(&item.id)) {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucun objet de larmes dans le pool d'une boutique de la partie ordinaire ({shop_where})"
                        ),
                    );
                }
                // Deux exemplaires : un pool qui ne les exclut pas (`exclude_held`) et ne s'épuise pas
                // (`exhaust`), ou deux pools tirés qui ne les excluent pas, doivent donner l'objet.
                for item in tears
                    .iter()
                    .filter(|i| treasure_items.contains(&i.id) || shop_items.contains(&i.id))
                {
                    let giving: Vec<&registry::LootTableEntry> = drawn
                        .iter()
                        .filter_map(|pool| registry.loot_tables.get(pool))
                        .filter(|pool| pool_items(registry, &pool.def).contains(&item.id))
                        .collect();
                    let open: Vec<&&registry::LootTableEntry> = giving
                        .iter()
                        .filter(|pool| !pool.def.exclude_held)
                        .collect();
                    if !(open.iter().any(|pool| !pool.def.exhaust) || open.len() >= 2) {
                        warn(
                            &item.file.display().to_string(),
                            format!(
                                "objet de larmes « {} » : deux exemplaires impossibles (pool(s) qui le donnent : {} ; chacun a `exclude_held` ou `exhaust`, ou il n'y en a qu'un) : l'empilement n'est jamais rencontré",
                                item.id,
                                giving.iter().map(|p| p.id.as_str()).collect::<Vec<_>>().join(", ")
                            ),
                        );
                    }
                }
            }
            Mechanic::Effects => {
                let effectful: Vec<&registry::ItemEntry> = registry
                    .items
                    .values()
                    .filter(|item| {
                        item.def.kind != items::ItemKind::Consumable && !item.def.effects.is_empty()
                    })
                    .collect();
                for (group, pool_items_of, where_) in [
                    ("trésor", &treasure_items, &treasure_where),
                    ("boutique", &shop_items, &shop_where),
                    (
                        "boss",
                        &boss_items,
                        &format!(
                            "salle(s) boss : {} ; pool(s) : {}",
                            kinds_list(&bosses),
                            pools_list(&boss_pools)
                        ),
                    ),
                ] {
                    if !effectful
                        .iter()
                        .any(|item| pool_items_of.contains(&item.id))
                    {
                        warn(
                            MANIFEST_FILE,
                            format!(
                                "aucun objet à effets dans le pool {group} de la partie ordinaire ({where_})"
                            ),
                        );
                    }
                }
                let transformable = registry.synergies.values().any(|synergy| {
                    synergy
                        .def
                        .requires
                        .iter()
                        .all(|id| obtainable.contains(id))
                });
                if !transformable {
                    warn(
                        MANIFEST_FILE,
                        "aucune synergie (transformation) dont tous les objets sont atteignables par pool, départ ou étage".to_string(),
                    );
                }
            }
            Mechanic::Familiars => {
                let count = |set: &BTreeSet<String>| {
                    registry
                        .items
                        .values()
                        .filter(|item| item.def.familiar.is_some() && set.contains(&item.id))
                        .count()
                };
                let in_treasure = count(&treasure_items);
                if in_treasure < 2 {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "{in_treasure} objet(s) à `familiar` dans le pool trésor de la partie ordinaire, 2 attendus ({treasure_where})"
                        ),
                    );
                }
                if count(&shop_items) == 0 {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucun objet à `familiar` dans le pool d'une boutique de la partie ordinaire ({shop_where})"
                        ),
                    );
                }
            }
            Mechanic::Bombs => {
                if !bomb_obtainable(registry, ordinary) {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucune bombe de départ (`starting_consumables` « {BOMB} » d'un personnage jouable), d'étage (`consumables_per_floor`) ni de récompense (pool de combat : {}) dans la partie ordinaire",
                            reward_pools.iter().copied().collect::<Vec<_>>().join(", ")
                        ),
                    );
                }
                if !players(registry).any(|c| c.bomb.is_some()) {
                    warn(
                        MANIFEST_FILE,
                        "aucun personnage jouable n'a `bomb` : le bouton Bomb ne ferait rien"
                            .to_string(),
                    );
                }
                let fillers: BTreeSet<&str> = grammars
                    .iter()
                    .map(|(entry, _)| entry.grammar.filler.as_str())
                    .collect();
                let levels: Vec<&registry::MapLevelInfo> =
                    playable_levels(registry, &ordinary.reach)
                        .into_iter()
                        .filter(|level| {
                            level
                                .room_kind
                                .as_deref()
                                .is_some_and(|kind| fillers.contains(kind))
                        })
                        .collect();
                let filler_list = fillers.iter().copied().collect::<Vec<_>>().join(", ");
                if !levels
                    .iter()
                    .any(|level| level.rocks + level.loot_rocks > 0)
                {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucun rocher (couche `Rocks` de la carte de gabarits) dans un gabarit de type « {filler_list} » de la partie ordinaire"
                        ),
                    );
                }
                let hazard_values: BTreeSet<u8> = registry
                    .surfaces
                    .values()
                    .filter(|s| s.hazard.is_some())
                    .map(|s| s.intgrid_value)
                    .collect();
                if !levels.iter().any(|level| {
                    level
                        .surface_values
                        .iter()
                        .any(|v| hazard_values.contains(v))
                }) {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucune pique (surface à `hazard`, couche `Surfaces`) dans un gabarit de type « {filler_list} » de la partie ordinaire"
                        ),
                    );
                }
            }
            Mechanic::SecretRoom => {
                let grids: Vec<_> = grammars
                    .iter()
                    .filter(|(entry, _)| {
                        matches!(entry.grammar.layout(), world::FloorLayout::Grid { .. })
                    })
                    .collect();
                if grids.is_empty() {
                    warn(
                        MANIFEST_FILE,
                        "aucune grammaire en grille (`layout: Grid`) dans la partie ordinaire : pas de salle secrète".to_string(),
                    );
                }
                for (entry, _) in grids {
                    let world::FloorLayout::Grid { secret, .. } = entry.grammar.layout() else {
                        continue;
                    };
                    let file = entry.file.display().to_string();
                    if !entry.grammar.required.contains_key(&secret) {
                        warn(
                            &file,
                            format!(
                                "grammaire « {} » : `required` ne demande pas la salle secrète « {secret} » (une par étage)",
                                entry.id
                            ),
                        );
                    }
                    if registry
                        .rooms
                        .get(&secret)
                        .is_some_and(|r| r.def.door_lock != Some(world::DoorLock::Secret))
                    {
                        warn(
                            &file,
                            format!(
                                "grammaire « {} » : le type « {secret} » n'a pas `door_lock: Secret` : la salle n'est pas murée",
                                entry.id
                            ),
                        );
                    }
                }
                if !explosion_source(registry, ordinary, &obtainable) {
                    warn(
                        MANIFEST_FILE,
                        "aucune bombe atteignable dans la partie ordinaire : la salle secrète ne s'ouvre jamais".to_string(),
                    );
                }
            }
            Mechanic::Keys => {
                if !reward_items.contains(KEY) {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucune clé (« {KEY} ») dans le pool de récompense de combat (pool(s) : {})",
                            reward_pools.iter().copied().collect::<Vec<_>>().join(", ")
                        ),
                    );
                }
                if !shop_items.contains(KEY) {
                    warn(
                        MANIFEST_FILE,
                        format!("aucune clé dans le pool d'une boutique de la partie ordinaire ({shop_where})"),
                    );
                }
                if required_locks.is_empty() {
                    warn(
                        MANIFEST_FILE,
                        "aucune salle exigée par la grammaire n'a `door_lock: Key` (trésor fermé à clé)".to_string(),
                    );
                }
            }
            Mechanic::Hearts => {
                if !reward_pools.iter().any(|pool| {
                    pool_gives(registry, pool, |p| {
                        matches!(p, items::LootPick::Currency(_))
                    })
                }) {
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucun tas de sous (`Currency`) dans le pool de récompense de combat (pool(s) : {})",
                            reward_pools.iter().copied().collect::<Vec<_>>().join(", ")
                        ),
                    );
                }
                if !registry
                    .items
                    .values()
                    .any(|item| heals(item) && reward_items.contains(&item.id))
                {
                    warn(
                        MANIFEST_FILE,
                        "aucun cœur (objet à `Heal` ou `GainSoul`) dans le pool de récompense de combat".to_string(),
                    );
                }
                if required_shops.is_empty() {
                    warn(
                        MANIFEST_FILE,
                        "aucune boutique exigée par la grammaire (`required` d'une salle à `shop`)"
                            .to_string(),
                    );
                }
            }
            Mechanic::ExhaustPools => {
                let exhausting: Vec<&registry::LootTableEntry> = registry
                    .loot_tables
                    .values()
                    .filter(|pool| pool.def.exhaust)
                    .collect();
                let times = |pool: &str| -> u32 {
                    draws
                        .iter()
                        .filter(|d| d.pool == pool)
                        .map(|d| d.times)
                        .sum()
                };
                if !exhausting.iter().any(|pool| times(&pool.id) >= 2) {
                    let detail = if exhausting.is_empty() {
                        "aucun pool n'a `exhaust: true`".to_string()
                    } else {
                        exhausting
                            .iter()
                            .map(|pool| format!("« {} » tiré {} fois", pool.id, times(&pool.id)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    warn(
                        MANIFEST_FILE,
                        format!(
                            "aucun pool à épuisement tiré au moins deux fois par partie ordinaire ({detail}) : l'épuisement n'agit jamais"
                        ),
                    );
                }
            }
            Mechanic::DevilMarket => {
                let posed = grammars.iter().any(|(entry, _)| {
                    entry
                        .grammar
                        .post_boss()
                        .is_some_and(|p| p.chance > Fixed::ZERO)
                });
                if !posed {
                    warn(
                        MANIFEST_FILE,
                        "aucune grammaire de la partie ordinaire n'a de salle post-boss à chance non nulle (`layout: Grid(post_boss: Some(…))`)".to_string(),
                    );
                }
                // Le marché : la salle post-boss propose des présentoirs payés (`shop` à `cost`, T8).
                for (entry, _) in &grammars {
                    let Some(post) = entry.grammar.post_boss().filter(|p| p.chance > Fixed::ZERO)
                    else {
                        continue;
                    };
                    if let Some(room) = registry.rooms.get(&post.kind) {
                        if !room
                            .def
                            .shop
                            .as_ref()
                            .is_some_and(|shop| shop.cost.is_some())
                        {
                            warn(
                                &room.file.display().to_string(),
                                format!(
                                    "type de salle « {} » (post-boss de la grammaire « {} ») sans `shop` à `cost` : aucun présentoir payé, ce n'est pas un marché",
                                    post.kind, entry.id
                                ),
                            );
                        }
                    }
                }
            }
            Mechanic::Champions => {
                let reachable = reachable_characters(registry, &ordinary.reach);
                let any = registry.characters.values().any(|c| {
                    !c.test_only
                        && reachable.contains(c.id.as_str())
                        && c.variants.as_ref().is_some_and(|v| {
                            !v.table.is_empty() && variant_chance_possible(&v.chance)
                        })
                });
                if !any {
                    warn(
                        MANIFEST_FILE,
                        "aucun ennemi des salles de la partie ordinaire ne porte `variants` à chance non nulle (gabarits de combat de la grammaire)".to_string(),
                    );
                }
            }
            Mechanic::Curses => {
                let any = ordinary.sequences.iter().any(|sequence| {
                    sequence.curses.as_ref().is_some_and(|c| {
                        c.chance > Fixed::ZERO
                            && c.pool
                                .iter()
                                .any(|id| registry.curses.get(id).is_some_and(|d| d.def.weight > 0))
                    })
                });
                if !any {
                    let file = ordinary.sequences.first().map_or_else(
                        || MANIFEST_FILE.to_string(),
                        |s| s.file.display().to_string(),
                    );
                    warn(
                        &file,
                        "la séquence ordinaire n'a pas de `curses` (chance non nulle et pool de curses de poids > 0)".to_string(),
                    );
                }
            }
            Mechanic::Achievements => {
                let sources = item_sources(registry);
                let facts = Facts::new(registry, ordinary, &sources);
                let visible = registry.achievements.values().any(|achievement| {
                    achievement
                        .def
                        .when
                        .iter()
                        .all(|condition| facts.problem(condition).is_none())
                        && achievement.def.unlocks.iter().any(|unlock| {
                            registry.unlocks.get(unlock).is_some_and(|unlock| {
                                unlock.def.grants.iter().any(|grant| match grant {
                                    items::UnlockGrant::Item(item) => drawn_items.contains(item),
                                    items::UnlockGrant::Loot { table, .. } => drawn.contains(table),
                                })
                            })
                        })
                });
                if !visible {
                    warn(
                        MANIFEST_FILE,
                        "aucun accomplissement dont le déblocage donne un objet ou une entrée à un pool tiré par la partie ordinaire".to_string(),
                    );
                }
            }
        }
    }
}

/// Bombe de départ d'un personnage jouable, d'étage, donnée par un pool tiré de la partie ou par un
/// déblocage.
fn bomb_obtainable(registry: &Registry, play: &Play<'_>) -> bool {
    players(registry).any(|c| c.starting_consumables.get(BOMB).is_some_and(|n| *n > 0))
        || play
            .sequences
            .iter()
            .any(|s| s.consumables_per_floor.contains_key(BOMB))
        || items_in(registry, play.draws().iter().map(|d| d.pool.as_str())).contains(BOMB)
        || unlock_grant_sources(registry).0.contains(BOMB)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Mechanic` a une étiquette pour chaque variante (le `match` de `label` est exhaustif) et les
    /// étiquettes sont distinctes : un message nomme sans ambiguïté la mécanique.
    #[test]
    fn labels_are_distinct() {
        let all = [
            Mechanic::Tears,
            Mechanic::Effects,
            Mechanic::Familiars,
            Mechanic::Bombs,
            Mechanic::SecretRoom,
            Mechanic::Keys,
            Mechanic::Hearts,
            Mechanic::ExhaustPools,
            Mechanic::DevilMarket,
            Mechanic::Champions,
            Mechanic::Curses,
            Mechanic::Achievements,
        ];
        let labels: BTreeSet<&str> = all.iter().map(|m| label(*m)).collect();
        assert_eq!(labels.len(), all.len());
    }
}
