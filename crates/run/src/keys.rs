//! Clés d'équipe (X-6a1, `docs/conventions.md` §117) : l'état partagé de **tous** les joueurs d'une
//! partie coop, par opposition aux consommables `key` d'un joueur (`items::Inventory`, salles d'`isaac`).
//!
//! [`TeamKeys`] compte les clés tenues par identifiant (`BTreeMap` : itération déterministe). Une clé
//! tombée d'un ennemi (`items::KeyPickup`) y entre quand l'un ou l'autre joueur la ramasse ; une porte de
//! carte à serrure (`map::generation::entity::door::DoorLockConfig`) la lit et, selon sa donnée, la retire.
//! [`KeyEvent`] annonce chaque variation (et chaque refus) aux lecteurs hors état (moments clés de
//! scénario) : le HUD, lui, se dérive de [`TeamKeys`].
//!
//! Les enregistrements rollback (ressource et file en variante neutre : la valeur par défaut et la file
//! vide ne contribuent rien au checksum) sont posés par `game::core_sim`, avant les cinq de clôture.

use std::collections::BTreeMap;

use bevy::prelude::*;

/// Clés tenues par l'équipe, par identifiant. Invariant : aucune entrée à `0` (une clé épuisée est
/// retirée), pour que « aucune clé » soit exactement la valeur par défaut (contribution neutre au
/// checksum).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TeamKeys(pub BTreeMap<String, u32>);

impl TeamKeys {
    /// Nombre de clés `key` tenues.
    pub fn count(&self, key: &str) -> u32 {
        self.0.get(key).copied().unwrap_or(0)
    }

    /// Ajoute une clé `key`.
    pub fn add(&mut self, key: &str) {
        let held = self.0.entry(key.to_string()).or_insert(0);
        *held = held.saturating_add(1);
    }

    /// Retire une clé `key` ; `false` (rien ne change) si l'équipe n'en tient aucune.
    pub fn take(&mut self, key: &str) -> bool {
        let Some(held) = self.0.get_mut(key) else {
            return false;
        };
        *held -= 1;
        if *held == 0 {
            self.0.remove(key);
        }
        true
    }
}

/// Variation de [`TeamKeys`] ou refus d'une porte à serrure. `handle` est le handle GGRS du joueur
/// (jamais une `Entity`, CLAUDE.md règle 3). `reason` : `pickup` (`delta` +1), `door` (clé consommée,
/// `delta` −1), `door_open` (porte ouverte, clé gardée, `delta` 0), `door_refused` (aucune clé, `delta` 0).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub handle: usize,
    pub key: String,
    pub delta: i32,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_cle_epuisee_est_retiree_et_redonne_la_valeur_par_defaut() {
        let mut keys = TeamKeys::default();
        keys.add("a");
        keys.add("a");
        keys.add("b");
        assert_eq!(
            (keys.count("a"), keys.count("b"), keys.count("c")),
            (2, 1, 0)
        );
        assert!(keys.take("a") && keys.take("a") && keys.take("b"));
        assert_eq!(keys, TeamKeys::default());
    }

    #[test]
    fn retirer_une_cle_absente_ne_change_rien() {
        let mut keys = TeamKeys::default();
        keys.add("a");
        assert!(!keys.take("b"));
        assert_eq!(keys.count("a"), 1);
    }
}
