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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    // M3-T0a (`docs/conventions.md` §60) : stats de larmes, **après** `Custom` pour garder
    // l'ordre (`Ord` des clés de `Stats`) et le hash (discriminant) des stats existantes. Base 1
    // pour tout personnage sans être posées dans `Stats` (lues par `StatReader::get` avec 1 par
    // défaut, voir [`default_base`]) : `Stats` de tous les personnages reste inchangé.
    /// Multiplie la vitesse de la balle au tir (`combat::weapons`).
    ShotSpeed,
    /// Multiplie le rayon de collision et le sprite de la balle au tir, comme le modificateur
    /// de projectile `Size` (`combat::weapons`).
    ShotSize,
}

/// M3-T0a : base d'une stat qu'aucun personnage ne porte dans [`Stats`] mais que l'engine lit
/// avec une valeur par défaut (`ShotSpeed`, `ShotSize` : 1 ; `Luck` : 0, M3-T1). `None` : la base
/// vient de `Stats`.
/// Sert aux lecteurs hors simulation (attente `Stat`, générateur) pour résoudre ces stats comme
/// le tir les résout.
pub fn default_base(stat: &StatId) -> Option<Fixed> {
    match stat {
        StatId::ShotSpeed | StatId::ShotSize => Some(bevy_fixed::fixed_math::FIXED_ONE),
        StatId::Luck => Some(bevy_fixed::fixed_math::FIXED_ZERO),
        _ => None,
    }
}

/// M2-T15 (`docs/conventions.md` §53) : où la simulation **lit** une stat, ou `None` si aucun
/// système ne la lit (un modificateur dessus ne ferait rien : D61, `Armor`). Liste fermée, tenue
/// à jour à la main : ajouter une ligne en même temps que le système qui lit la stat. Le lint
/// (`content::lint`) refuse un modificateur sur une stat sans lecteur.
pub fn stat_reader(stat: &StatId) -> Option<&'static str> {
    match stat {
        StatId::MaxHealth => Some("character::health (santé max), interaction (soin)"),
        StatId::HealthRegen => Some("character::health (régénération)"),
        StatId::MoveSpeed => {
            Some("character::player::input, combat::projectile, character::surface")
        }
        StatId::Acceleration => Some("character::player::input (accélération)"),
        StatId::SprintMultiplier => Some("character::player::input (sprint)"),
        StatId::Damage => Some("combat::weapons (dégâts), character::enemy::ai::behavior"),
        StatId::FireRate => Some("combat::weapons (cadence)"),
        StatId::ReloadSpeed => Some("combat::weapons (rechargement)"),
        StatId::Range => Some("combat::weapons (portée)"),
        StatId::SeparationDistance
        | StatId::SeparationForce
        | StatId::SlowDownDistance
        | StatId::OptimalAttackDistance
        | StatId::EnemyMoveSpeed => Some("character::enemy::ai::pathing (move_enemies)"),
        // D61 : réduction des dégâts reçus, `dégât × 100 / (100 + armure)`.
        StatId::Armor => Some("character::health (rollback_resolve_damage_events : armure)"),
        // M3-T0a : lecteur de contrat, la chance d'effet (`effects::chance_with_luck`) ; M3-T1
        // et M3-T2 l'appellent pour les chances d'effet des objets.
        StatId::Luck => Some("effects::chance_with_luck (chance d'effet)"),
        StatId::ShotSpeed => Some("combat::weapons (vitesse de la balle)"),
        StatId::ShotSize => Some("combat::weapons (taille de la balle)"),
        // Seule stat `Custom` lue : le multiplicateur de points des power-ups
        // (`effects::CURRENCY_MULTIPLIER_STAT`, lu par `game::economy`).
        StatId::Custom(name) if name == "powerup_currency_multiplier" => {
            Some("game::economy (points gagnés)")
        }
        StatId::Custom(_) => None,
    }
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

    /// Réécrit les bases présentes dans `other` et garde les autres (rechargement à chaud :
    /// les défauts propres à la création, ex. ceux d'un ennemi, restent).
    pub fn overlay(&mut self, other: &Stats) {
        for (id, value) in &other.0 {
            self.0.insert(id.clone(), *value);
        }
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
