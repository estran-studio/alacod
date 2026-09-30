//! Réserves de munitions par type, partagées entre toutes les armes d'un même joueur qui
//! déclarent le même `AmmoType` (T2.2, chantier B7 « Munitions typées et inventaire
//! d'armes »).
//!
//! Composant rollback (posé une fois à la création du joueur,
//! `game::character::player::create::create_player`) : initialisé à la somme, par type de
//! munition, de `mag_limit × mag_size` (mode par défaut) de chaque arme de départ — deux
//! armes qui partagent un type s'additionnent. Un rechargement
//! (`game::weapons::weapon_rollback_system`) puise `mag_size` unités dans la réserve du type
//! de l'arme active, à la place de l'ancien compteur `mag_quantity` par arme (retiré de
//! `WeaponModeState` ; voir le rapport de la tâche T2.2 pour la migration et son
//! équivalence).

use std::collections::BTreeMap;

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};
use sim_core::ammo::AmmoType;

/// Réserve de munitions d'un joueur, par type. `BTreeMap` (pas `HashMap`, CLAUDE.md règle 5,
/// déterminisme) : ordre stable si un système parcourt un jour tous les types à la fois.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AmmoReserves(pub BTreeMap<AmmoType, u32>);

impl AmmoReserves {
    pub fn new() -> Self {
        Self::default()
    }

    /// Quantité disponible pour `ammo_type` (0 si le type n'a jamais été crédité : une arme
    /// ramassée d'un type absent des armes de départ ne peut jamais se recharger tant que
    /// rien ne crédite ce type — un futur achat de munitions, T2.3, s'en chargera).
    pub fn get(&self, ammo_type: &AmmoType) -> u32 {
        self.0.get(ammo_type).copied().unwrap_or(0)
    }

    /// Ajoute `amount` à la réserve de `ammo_type` (crédit à la création du joueur, futur
    /// achat T2.3).
    pub fn add(&mut self, ammo_type: AmmoType, amount: u32) {
        *self.0.entry(ammo_type).or_insert(0) += amount;
    }

    /// Retire jusqu'à `amount` de la réserve de `ammo_type`, borné à ce qui est disponible
    /// (jamais négatif). Renvoie la quantité réellement retirée.
    pub fn take(&mut self, ammo_type: &AmmoType, amount: u32) -> u32 {
        let Some(current) = self.0.get_mut(ammo_type) else {
            return 0;
        };
        let taken = amount.min(*current);
        *current -= taken;
        taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_defaults_to_zero() {
        let reserves = AmmoReserves::new();
        assert_eq!(reserves.get(&AmmoType::Plomb), 0);
    }

    #[test]
    fn add_accumulates_shared_types() {
        let mut reserves = AmmoReserves::new();
        reserves.add(AmmoType::Balle, 48);
        reserves.add(AmmoType::Balle, 6);
        assert_eq!(reserves.get(&AmmoType::Balle), 54);
        assert_eq!(reserves.get(&AmmoType::Plomb), 0);
    }

    #[test]
    fn take_is_bounded_by_available_amount() {
        let mut reserves = AmmoReserves::new();
        reserves.add(AmmoType::Cartouche, 5);
        assert_eq!(reserves.take(&AmmoType::Cartouche, 3), 3);
        assert_eq!(reserves.get(&AmmoType::Cartouche), 2);
        assert_eq!(reserves.take(&AmmoType::Cartouche, 10), 2);
        assert_eq!(reserves.get(&AmmoType::Cartouche), 0);
        assert_eq!(reserves.take(&AmmoType::Cartouche, 1), 0);
    }
}
