//! M2-T15 (`docs/conventions.md` §53) : atteignabilité **statique** du contenu et stats sans
//! lecteur. Calcul sur le contenu déjà lu par le lint, sans jouer de partie (l'atteignabilité
//! dynamique est le bot explorateur, §47). Tout est avertissement ([`LintWarning`]) : un
//! avertissement n'échoue pas `alacod lint`, il est listé.
//!
//! **Racines** : `entry.start_map` et chaque niveau de chaque séquence du dossier `Floors`
//! (carte directe, caverne ou grammaire d'étage `floor:<id>`). Le dossier `Floors` n'a pas de
//! séquence « par défaut » implicite (D59 : `entry.floors` la désigne explicitement, à défaut la
//! première par ordre d'id) : toute séquence déclarée reste une racine, car un scénario ou le
//! lobby peut en imposer une autre.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::manifest::GameManifest;
use crate::registry::{self, Registry};
use sim_core::stats::StatId;

/// Catégorie d'avertissement, pour les tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintWarningKind {
    /// Contenu valide qu'aucune partie ne peut rencontrer (calcul statique).
    Unreachable,
    /// Modificateur sur une stat qu'aucun système ne lit (aujourd'hui une erreur de lint : variante
    /// gardée pour un futur avertissement).
    UnreadStat,
}

/// Un avertissement de lint : même forme qu'une [`crate::LintError`], mais n'échoue pas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintWarning {
    pub kind: LintWarningKind,
    pub file: String,
    pub message: String,
}

impl fmt::Display for LintWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.message)
    }
}

/// Objets donnés par un pool de butin (M2-T5, `docs/conventions.md` §48) : les objets nommés
/// par une entrée `Item`, plus tous ceux d'une rareté visée par une entrée `Rarity`. Un pool
/// compte même si aucune salle ne le pose : le lint dit « un pool donne cet objet », pas
/// « une partie le rencontre » (l'atteignabilité dynamique est le bot explorateur, §47).
fn loot_pool_items(registry: &Registry) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for pool in registry.loot_tables.values() {
        for entry in &pool.def.entries {
            match &entry.pick {
                items::LootPick::Item(id) => {
                    found.insert(id.clone());
                }
                items::LootPick::Rarity(rarity) => {
                    found.extend(
                        registry
                            .items
                            .values()
                            .filter(|item| item.def.rarity == *rarity)
                            .map(|item| item.id.clone()),
                    );
                }
            }
        }
    }
    found
}

/// Armes données par un pool de butin : aucune entrée de pool ne désigne d'arme (M2-T5 ne
/// tire que des objets, les armes viennent de la boutique, T6).
fn loot_pool_weapons(_registry: &Registry) -> BTreeSet<String> {
    BTreeSet::new()
}

/// Ce qu'une partie ordinaire peut atteindre, déduit des racines.
#[derive(Default)]
struct Reach<'a> {
    /// Grammaires d'étage désignées par une racine.
    grammars: BTreeSet<&'a registry::FloorGrammarId>,
    /// Cavernes désignées par une racine.
    caves: BTreeSet<&'a registry::CaveId>,
    /// Cartes posées directement (tous leurs niveaux sont joués).
    maps: BTreeSet<registry::MapId>,
    /// Types de salle pouvant être tirés.
    room_kinds: BTreeSet<String>,
    /// Gabarits `(carte, niveau)` pouvant être tirés par une grammaire.
    drawn_templates: BTreeSet<(registry::MapId, String)>,
}

fn designations<'a>(registry: &'a Registry, manifest: &'a GameManifest) -> Vec<&'a str> {
    let mut out = vec![manifest.entry.start_map.as_str()];
    for floors in registry.floors.values() {
        out.extend(floors.levels.iter().map(String::as_str));
    }
    out
}

fn reach<'a>(registry: &'a Registry, manifest: &'a GameManifest) -> Reach<'a> {
    let mut reach = Reach::default();
    for designation in designations(registry, manifest) {
        if let Some(cave) = registry::cave_designation(designation) {
            if let Some((id, _)) = registry.caves.get_key_value(&cave) {
                reach.caves.insert(id);
            }
        } else if let Some(grammar) = registry::floor_designation(designation) {
            if let Some((id, _)) = registry.floor_grammars.get_key_value(&grammar) {
                reach.grammars.insert(id);
            }
        } else {
            reach.maps.insert(registry::map_id_from_path(designation));
        }
    }
    for id in &reach.grammars {
        let entry = &registry.floor_grammars[*id];
        let map_id = registry::map_id_from_path(&entry.templates);
        let Some(map) = registry.maps.get(&map_id) else {
            continue;
        };
        let kinds: BTreeSet<&str> = entry.grammar.kinds_needed().into_iter().collect();
        for (level, kind) in &map.room_kinds {
            if kinds.contains(kind.as_str()) {
                reach
                    .drawn_templates
                    .insert((map_id.clone(), level.clone()));
            }
        }
        reach.room_kinds.extend(kinds.iter().map(|k| k.to_string()));
    }
    for map_id in &reach.maps {
        if let Some(map) = registry.maps.get(map_id) {
            reach
                .room_kinds
                .extend(map.room_kinds.iter().map(|(_, kind)| kind.clone()));
        }
    }
    reach
}

/// Niveaux de carte que la partie peut jouer : tous ceux des cartes posées, et les gabarits
/// qu'une grammaire peut tirer.
fn playable_levels<'a>(
    registry: &'a Registry,
    reach: &Reach<'_>,
) -> Vec<&'a registry::MapLevelInfo> {
    let mut out = Vec::new();
    for map in registry.maps.values() {
        for level in &map.levels {
            if reach.maps.contains(&map.id)
                || reach
                    .drawn_templates
                    .contains(&(map.id.clone(), level.id.clone()))
            {
                out.push(level);
            }
        }
    }
    out
}

/// Personnages que la partie peut faire apparaître : cartes, arènes, vagues, cavernes,
/// renforts de boss (point fixe : un renfort n'est atteint que si son boss l'est).
fn reachable_characters(registry: &Registry, reach: &Reach<'_>) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for level in playable_levels(registry, reach) {
        out.extend(level.characters.iter().cloned());
    }
    for room in registry.rooms.values() {
        if !reach.room_kinds.contains(&room.id) {
            continue;
        }
        for wave in room.def.arena.iter().flat_map(|arena| &arena.waves) {
            out.extend(wave.enemies.iter().map(|(id, _)| id.clone()));
        }
    }
    for wave in registry.waves.values() {
        out.extend(wave.enemy_refs.iter().map(|id| id.to_string()));
    }
    for id in &reach.caves {
        out.extend(registry.caves[*id].config.characters.iter().cloned());
    }
    loop {
        let mut added = BTreeSet::new();
        for id in &out {
            let Some(boss) = registry
                .characters
                .get(&registry::CharacterId::from(id.clone()))
                .and_then(|c| c.boss.as_ref())
            else {
                continue;
            };
            for phase in &boss.phases {
                let timeline = phase.timeline.iter().flat_map(|t| &t.events);
                let actions = phase
                    .on_enter
                    .iter()
                    .chain(timeline.flat_map(|event| &event.r#do));
                for action in actions {
                    if let effects::Action::SpawnEnemies { id, .. } = action {
                        if !out.contains(id) {
                            added.insert(id.clone());
                        }
                    }
                }
            }
        }
        if added.is_empty() {
            break;
        }
        out.extend(added);
    }
    out
}

/// Chaque usage d'une stat par le contenu : `(fichier, endroit, stat)`.
pub(crate) fn stat_uses(registry: &Registry) -> Vec<(String, String, StatId)> {
    let mut out = Vec::new();
    let mut actions = |file: &std::path::Path, at: String, list: &[effects::Action]| {
        for action in list {
            match action {
                effects::Action::Modifier { stat, .. }
                | effects::Action::TimedModifier { stat, .. } => {
                    out.push((file.display().to_string(), at.clone(), stat.clone()))
                }
                _ => {}
            }
        }
    };
    for c in registry.characters.values() {
        for effect in &c.effects {
            actions(
                &c.file,
                format!("personnage « {} » : effects", c.id),
                &effect.r#do,
            );
        }
    }
    for item in registry.items.values() {
        for effect in &item.def.effects {
            actions(
                &item.file,
                format!("objet « {} » : effects", item.id),
                &effect.r#do,
            );
        }
    }
    for synergy in registry.synergies.values() {
        for effect in &synergy.def.effects {
            actions(
                &synergy.file,
                format!("synergie « {} » : effects", synergy.id),
                &effect.r#do,
            );
        }
    }
    for mutation in registry.mutations.values() {
        for effect in &mutation.effects {
            actions(
                &mutation.file,
                format!("mutation « {} » : effects", mutation.id),
                &effect.r#do,
            );
        }
    }
    for powerup in registry.powerups.values() {
        actions(
            &powerup.file,
            format!("power-up « {} » : actions", powerup.id),
            &powerup.actions,
        );
    }
    for c in registry.characters.values() {
        let file = c.file.display().to_string();
        for stat in c.stats.keys() {
            out.push((
                file.clone(),
                format!("personnage « {} » : stats", c.id),
                stat.clone(),
            ));
        }
        for (name, variant) in c.variants.iter().flat_map(|v| &v.table) {
            for modifier in &variant.modifiers {
                out.push((
                    file.clone(),
                    format!("personnage « {} » : variante « {name} »", c.id),
                    modifier.stat.clone(),
                ));
            }
        }
    }
    for perk in registry.perks.values() {
        for modifier in &perk.modifiers {
            out.push((
                perk.file.display().to_string(),
                format!("perk « {} » : modifiers", perk.id),
                modifier.stat.clone(),
            ));
        }
    }
    for item in registry.items.values() {
        for modifier in &item.def.modifiers {
            out.push((
                item.file.display().to_string(),
                format!("objet « {} » : modifiers", item.id),
                modifier.stat.clone(),
            ));
        }
    }
    for synergy in registry.synergies.values() {
        for modifier in &synergy.def.modifiers {
            out.push((
                synergy.file.display().to_string(),
                format!("synergie « {} » : modifiers", synergy.id),
                modifier.stat.clone(),
            ));
        }
    }
    out
}

/// Avertissements : contenu inatteignable (les stats sans lecteur sont des erreurs, voir
/// `lint::lint_stat_readers`).
pub fn warnings(registry: &Registry, manifest: &GameManifest) -> Vec<LintWarning> {
    let mut out = Vec::new();
    let mut warn = |kind: LintWarningKind, file: String, message: String| {
        out.push(LintWarning {
            kind,
            file,
            message,
        })
    };

    let reach = reach(registry, manifest);

    for room in registry.rooms.values() {
        if !reach.room_kinds.contains(&room.id) {
            warn(
                LintWarningKind::Unreachable,
                room.file.display().to_string(),
                format!(
                    "type de salle « {} » : aucune grammaire ni carte d'une séquence ne peut le tirer",
                    room.id
                ),
            );
        }
    }

    let template_maps: BTreeSet<registry::MapId> = registry
        .floor_grammars
        .values()
        .map(|g| registry::map_id_from_path(&g.templates))
        .collect();
    for map in registry.maps.values() {
        if !template_maps.contains(&map.id) || reach.maps.contains(&map.id) {
            continue;
        }
        for (level, kind) in &map.room_kinds {
            if !reach
                .drawn_templates
                .contains(&(map.id.clone(), level.clone()))
            {
                warn(
                    LintWarningKind::Unreachable,
                    map.file.display().to_string(),
                    format!(
                        "gabarit « {level} » (room_kind « {kind} ») : aucune grammaire d'une séquence ne tire ce type"
                    ),
                );
            }
        }
    }

    let characters = reachable_characters(registry, &reach);
    for c in registry.characters.values() {
        if c.has_ai && !characters.contains(c.id.as_str()) {
            warn(
                LintWarningKind::Unreachable,
                c.file.display().to_string(),
                format!(
                    "ennemi « {} » : n'apparaît dans aucune salle, arène, vague, caverne ni renfort de boss d'une partie",
                    c.id
                ),
            );
        }
    }

    let mut item_sources: BTreeMap<&str, &str> = BTreeMap::new();
    for c in registry.characters.values() {
        for id in c.starting_consumables.keys() {
            item_sources.insert(id, "départ");
        }
    }
    for floors in registry.floors.values() {
        for id in floors.consumables_per_floor.keys() {
            item_sources.insert(id, "étage");
        }
    }
    let pool_items = loot_pool_items(registry);
    for item in registry.items.values() {
        if !item_sources.contains_key(item.id.as_str()) && !pool_items.contains(&item.id) {
            warn(
                LintWarningKind::Unreachable,
                item.file.display().to_string(),
                format!(
                    "objet « {} » : aucune source (départ, étage, salle, coffre ni pool)",
                    item.id
                ),
            );
        }
    }

    let mut weapon_sources: BTreeSet<String> = loot_pool_weapons(registry);
    for c in registry.characters.values() {
        weapon_sources.extend(c.starting_weapons.iter().map(|w| w.to_string()));
    }
    for level in playable_levels(registry, &reach) {
        weapon_sources.extend(level.weapons.iter().cloned());
    }
    for progression in registry.progression.values() {
        for pool in &progression.weapon_pool {
            weapon_sources.extend(pool.weapons.iter().cloned());
        }
    }
    for weapon in registry.weapons.values() {
        if !weapon_sources.contains(weapon.id.as_str()) {
            warn(
                LintWarningKind::Unreachable,
                weapon.file.display().to_string(),
                format!(
                    "arme « {} » : aucun personnage, mur ni pool ne la donne",
                    weapon.id
                ),
            );
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La stat `Custom` lue par la simulation reste dans la liste fermée de `sim_core`.
    #[test]
    fn currency_multiplier_stat_has_a_reader() {
        let stat = StatId::Custom(effects::CURRENCY_MULTIPLIER_STAT.to_string());
        assert!(sim_core::stats::stat_reader(&stat).is_some());
    }

    #[test]
    fn armor_has_a_reader_luck_has_none() {
        assert!(sim_core::stats::stat_reader(&StatId::Armor).is_some());
        assert!(sim_core::stats::stat_reader(&StatId::Luck).is_none());
        assert!(sim_core::stats::stat_reader(&StatId::MoveSpeed).is_some());
    }
}
