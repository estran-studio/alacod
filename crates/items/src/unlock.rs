//! Déblocages de méta-progression (M2-T12, `docs/conventions.md` §51) : contrats sans dépendance
//! au rendu ni au profil.
//!
//! - [`UnlockDef`] : un déblocage de `unlocks/<id>.ron` (kind `Unlock`) : son prix en essence ou
//!   ses conditions, et ce qu'il accorde ([`UnlockGrant`]).
//! - [`RunUnlocks`] : l'ensemble des déblocages **de la partie**, figé au lancement et identique
//!   chez tous les pairs (ressource rollback neutre, statique pendant la run). C'est la seule
//!   chose du profil que la simulation lit.
//! - [`UnlockTable::hidden_items`] / [`UnlockTable::extra_entries`] : ce que l'ensemble change au
//!   tirage du butin (`draw_loot_with`). Un ensemble vide ne retire que les objets réservés à un
//!   déblocage et n'ajoute rien.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{LootEntry, LootPick};

/// Ce qu'un déblocage accorde.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum UnlockGrant {
    /// L'objet (fichier de `items/`) n'est tiré dans aucun pool tant que ce déblocage manque.
    Item(String),
    /// Une entrée de plus dans le pool `table` (`loot/<table>.ron`).
    Loot {
        table: String,
        pick: LootPick,
        weight: i32,
    },
}

/// Un déblocage, `unlocks/<id>.ron` (kind `Unlock`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UnlockDef {
    #[serde(default)]
    pub name: String,
    /// Prix en essence ; `None` : gratuit (conditions seules). Un prix ≤ 0 est refusé par le lint.
    #[serde(default)]
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub cost: Option<i64>,
    /// Déblocages à posséder d'abord (cycle refusé par le lint).
    #[serde(default)]
    pub requires: Vec<String>,
    /// Statistique de profil minimale `(nom, valeur)`.
    #[serde(default)]
    pub stat: Option<(String, u64)>,
    #[serde(default)]
    pub grants: Vec<UnlockGrant>,
}

/// Les déblocages du jeu par identifiant (hors rollback).
#[derive(Resource, Clone, Debug, Default)]
pub struct UnlockTable(pub BTreeMap<String, UnlockDef>);

/// Déblocages actifs pendant la partie. Défaut = vide = comportement d'avant M2-T12.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct RunUnlocks(pub BTreeSet<String>);

impl RunUnlocks {
    pub fn has(&self, id: &str) -> bool {
        self.0.contains(id)
    }
}

impl UnlockTable {
    /// Objets réservés à un déblocage que `active` ne contient pas.
    pub fn hidden_items(&self, active: &RunUnlocks) -> BTreeSet<&str> {
        self.0
            .iter()
            .filter(|(id, _)| !active.has(id))
            .flat_map(|(_, def)| def.grants.iter())
            .filter_map(|grant| match grant {
                UnlockGrant::Item(item) => Some(item.as_str()),
                UnlockGrant::Loot { .. } => None,
            })
            .collect()
    }

    /// Entrées ajoutées au pool `table` par les déblocages actifs (ordre des identifiants).
    pub fn extra_entries(&self, table: &str, active: &RunUnlocks) -> Vec<LootEntry> {
        self.0
            .iter()
            .filter(|(id, _)| active.has(id))
            .flat_map(|(_, def)| def.grants.iter())
            .filter_map(|grant| match grant {
                UnlockGrant::Loot {
                    table: t,
                    pick,
                    weight,
                } if t == table => Some(LootEntry {
                    pick: pick.clone(),
                    weight: *weight,
                }),
                _ => None,
            })
            .collect()
    }

    /// Cycle de prérequis : l'identifiant d'un déblocage qui se requiert lui-même, directement
    /// ou par un détour.
    pub fn requires_cycle(&self) -> Option<String> {
        for start in self.0.keys() {
            let mut seen = BTreeSet::new();
            let mut stack: Vec<&str> = self.0[start].requires.iter().map(String::as_str).collect();
            while let Some(id) = stack.pop() {
                if id == start {
                    return Some(start.clone());
                }
                if !seen.insert(id) {
                    continue;
                }
                if let Some(def) = self.0.get(id) {
                    stack.extend(def.requires.iter().map(String::as_str));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(requires: &[&str], grants: Vec<UnlockGrant>) -> UnlockDef {
        UnlockDef {
            name: String::new(),
            cost: Some(5),
            requires: requires.iter().map(|s| s.to_string()).collect(),
            stat: None,
            grants,
        }
    }

    fn table() -> UnlockTable {
        UnlockTable(
            [
                (
                    "a".to_string(),
                    def(&[], vec![UnlockGrant::Item("sceptre".into())]),
                ),
                (
                    "b".to_string(),
                    def(
                        &["a"],
                        vec![UnlockGrant::Loot {
                            table: "commun".into(),
                            pick: LootPick::Item("sceptre".into()),
                            weight: 2,
                        }],
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }

    #[test]
    fn empty_set_hides_gated_items_and_adds_nothing() {
        let table = table();
        let none = RunUnlocks::default();
        assert!(table.hidden_items(&none).contains("sceptre"));
        assert!(table.extra_entries("commun", &none).is_empty());
        let both = RunUnlocks(["a".to_string(), "b".to_string()].into());
        assert!(table.hidden_items(&both).is_empty());
        assert_eq!(table.extra_entries("commun", &both).len(), 1);
        assert!(table.extra_entries("autre", &both).is_empty());
    }

    #[test]
    fn detects_prerequisite_cycles() {
        let mut table = table();
        assert_eq!(table.requires_cycle(), None);
        table.0.get_mut("a").unwrap().requires = vec!["b".into()];
        assert!(table.requires_cycle().is_some());
        table.0.get_mut("a").unwrap().requires = vec!["a".into()];
        assert!(table.requires_cycle().is_some());
    }
}
