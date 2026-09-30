//! Perks achetés pendant la partie (T2.3, chantier C5 v1) : un set de ce que ce joueur a déjà
//! acheté, pour n'autoriser qu'un seul achat par perk et par joueur
//! (`game::interaction::handle_perk_purchase_interaction`).
//!
//! `BTreeSet<String>` (pas `HashSet`, CLAUDE.md règle 5) : ordre déterministe s'il faut un
//! jour itérer (HUD, résumé de run T2.4). La clé est l'id du perk (`games/<jeu>/assets/economy/perks.ron`,
//! ex. `"juggernog"`), la même chaîne que `sim_core::modifier::ModifierSource::Named("perk:<id>")`
//! utilisée pour poser ses modificateurs de stats (voir `game::economy`).

use std::collections::BTreeSet;

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

/// Perks déjà achetés par ce joueur. Composant rollback (voir [`super::currency::RunPlugin`]),
/// posé vide à la création du joueur comme [`super::currency::Currency`].
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Perks(pub BTreeSet<String>);

impl Perks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Vrai si `id` a déjà été acheté.
    pub fn has(&self, id: &str) -> bool {
        self.0.contains(id)
    }

    /// Enregistre `id` comme acheté. Renvoie `true` si c'était un nouvel achat (comme
    /// `BTreeSet::insert`) ; `false` si déjà possédé (achat à ne pas facturer deux fois — voir
    /// l'appelant, `handle_perk_purchase_interaction`, qui vérifie [`Self::has`] avant de
    /// débiter plutôt que de se fier seulement à cette valeur de retour).
    pub fn insert(&mut self, id: String) -> bool {
        self.0.insert(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_has_is_true() {
        let mut perks = Perks::new();
        assert!(!perks.has("juggernog"));
        assert!(perks.insert("juggernog".to_string()));
        assert!(perks.has("juggernog"));
    }

    #[test]
    fn insert_twice_returns_false_second_time() {
        let mut perks = Perks::new();
        assert!(perks.insert("juggernog".to_string()));
        assert!(!perks.insert("juggernog".to_string()));
        assert_eq!(perks.0.len(), 1);
    }
}
