//! Identifiants de stats et composant `Stats` (valeurs de base, avant modificateurs).
//! La résolution avec les modificateurs vit dans [`crate::modifier`].

use bevy::prelude::Component;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Identifiant de stat. Enum ouvert (`Custom`) pour que le contenu (RON) puisse définir
/// des stats propres à un jeu sans toucher à l'engine.
///
/// Variantes ajoutées par T1.2 (chantier B2, « Stats branchées ») : `HealthRegen`,
/// `Acceleration`, `SprintMultiplier` (mouvement du joueur) et les cinq stats d'ennemi
/// (`SeparationDistance`, `SeparationForce`, `SlowDownDistance`, `OptimalAttackDistance`,
/// `EnemyMoveSpeed`), qui remplacent les constantes jusque-là figées dans
/// `character::enemy::ai::pathing::PathfindingConfig`. Voir
/// `character::create::create_character` pour la valeur de base de chacune.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StatId {
    MaxHealth,
    /// Vie régénérée par seconde (`character::health::HealthRegen::regen_rate`).
    HealthRegen,
    MoveSpeed,
    /// Accélération du joueur (`MovementConfig::acceleration`).
    Acceleration,
    /// Multiplicateur de vitesse en sprint, à pleine charge (`MovementConfig::sprint_multiplier`).
    SprintMultiplier,
    Damage,
    FireRate,
    ReloadSpeed,
    Range,
    Armor,
    Luck,
    /// Distance sous laquelle deux ennemis se repoussent (ex-`PathfindingConfig::enemy_separation_distance`).
    SeparationDistance,
    /// Force de répulsion entre ennemis trop proches (ex-`PathfindingConfig::enemy_separation_force`).
    SeparationForce,
    /// Distance à laquelle un ennemi commence à ralentir en approchant sa cible
    /// (ex-`PathfindingConfig::slow_down_distance`).
    SlowDownDistance,
    /// Distance d'attaque optimale d'un ennemi : il s'arrête à cette distance de sa cible
    /// (ex-`PathfindingConfig::optimal_attack_distance`).
    OptimalAttackDistance,
    /// Vitesse de déplacement d'un ennemi, lue par `move_enemies`. Distincte de `MoveSpeed`
    /// (posée sur tous les personnages depuis `movement.max_speed`) pour ne pas coupler le
    /// réglage du déplacement joueur et celui des ennemis : un modificateur qui vise l'un
    /// ne touche jamais l'autre.
    EnemyMoveSpeed,
    Custom(String),
}

/// Valeurs de base d'une entité, avant modificateurs (voir [`crate::modifier::resolve`]).
///
/// **Pas encore posé sur d'entité en T0.2** (contrainte « traces identiques » de la
/// tâche) : ce sera le travail de T1.2 (chantier B2) de l'attacher à des entités et de
/// l'enregistrer en rollback (`app.rollback_and_trace::<Stats>()`), avec le `BLESS=1`
/// que ça implique pour `tests/scenarios/*.trace`.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Stats(BTreeMap<StatId, Fixed>);

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &StatId) -> Option<Fixed> {
        self.0.get(id).copied()
    }

    pub fn set(&mut self, id: StatId, value: Fixed) {
        self.0.insert(id, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    #[test]
    fn get_set_roundtrip() {
        let mut stats = Stats::new();
        assert_eq!(stats.get(&StatId::MoveSpeed), None);

        stats.set(StatId::MoveSpeed, fx(150.0));
        assert_eq!(stats.get(&StatId::MoveSpeed), Some(fx(150.0)));

        stats.set(StatId::MoveSpeed, fx(200.0));
        assert_eq!(
            stats.get(&StatId::MoveSpeed),
            Some(fx(200.0)),
            "set écrase la valeur précédente"
        );
    }

    #[test]
    fn custom_stat_ids_are_distinct_by_name() {
        let mut stats = Stats::new();
        stats.set(StatId::Custom("sacre".into()), fx(1.0));
        stats.set(StatId::Custom("faim".into()), fx(2.0));
        assert_eq!(stats.get(&StatId::Custom("sacre".into())), Some(fx(1.0)));
        assert_eq!(stats.get(&StatId::Custom("faim".into())), Some(fx(2.0)));
    }
}
