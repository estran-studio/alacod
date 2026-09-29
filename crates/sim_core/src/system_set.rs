//! Ordre des systèmes de simulation dans `GgrsSchedule`.
//!
//! Déménagé de `crates/game/src/system_set.rs` en T0.2 (`docs/taches.md`) et complété,
//! pour que les futurs crates de vocabulaire (`combat`, `effects`, `behaviors`...)
//! puissent placer leurs systèmes sans dépendre de `game`. `crates/game/src/system_set.rs`
//! réexporte ce module ; `crates/game/src/core.rs` configure la chaîne d'ordre à partir de
//! [`RollbackSystemSet::ORDER`].

use bevy::prelude::SystemSet;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum RollbackSystemSet {
    /// Début de frame : nettoyage des `FrameEvents` de la frame précédente.
    FrameStart,
    Input,
    Interaction,
    Movement,
    Weapon,
    /// Déplacement et collisions des projectiles (après `Weapon`, qui les fait naître).
    Projectiles,
    CollisionDamage,
    /// Résolution des effets déclenchés (dégâts appliqués, soins, buffs/debuffs posés).
    Effects,
    /// Tick des statuts déjà posés (durée, empilement, expiration).
    Status,
    DeathManagement,
    AnimationUpdates,
    EnemySpawning,
    EnemyAI,
    /// État de run (vagues, étages, horloges) : après l'IA, avant le compteur de frame.
    Run,
    FrameCounter,
}

impl RollbackSystemSet {
    /// Ordre total de simulation. `crates/game/src/core.rs` configure `GgrsSchedule` en
    /// chaînant les paires consécutives de ce tableau (équivalent à `.chain()` sur un
    /// n-uplet, sans limite d'arité). Chaque variante apparaît exactement une fois ;
    /// voir les tests de ce module pour la vérification automatique (pas de doublon,
    /// couverture complète, séquence conforme à cette liste).
    pub const ORDER: [RollbackSystemSet; 15] = [
        RollbackSystemSet::FrameStart,
        RollbackSystemSet::Input,
        RollbackSystemSet::Interaction,
        RollbackSystemSet::Movement,
        RollbackSystemSet::Weapon,
        RollbackSystemSet::Projectiles,
        RollbackSystemSet::CollisionDamage,
        RollbackSystemSet::Effects,
        RollbackSystemSet::Status,
        RollbackSystemSet::DeathManagement,
        RollbackSystemSet::AnimationUpdates,
        RollbackSystemSet::EnemySpawning,
        RollbackSystemSet::EnemyAI,
        RollbackSystemSet::Run,
        RollbackSystemSet::FrameCounter,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Liste exhaustive indépendante de `ORDER`, dans l'ordre documenté par T0.2
    /// (`docs/taches.md`). Sert de référence aux trois tests ci-dessous.
    const ALL: [RollbackSystemSet; 15] = [
        RollbackSystemSet::FrameStart,
        RollbackSystemSet::Input,
        RollbackSystemSet::Interaction,
        RollbackSystemSet::Movement,
        RollbackSystemSet::Weapon,
        RollbackSystemSet::Projectiles,
        RollbackSystemSet::CollisionDamage,
        RollbackSystemSet::Effects,
        RollbackSystemSet::Status,
        RollbackSystemSet::DeathManagement,
        RollbackSystemSet::AnimationUpdates,
        RollbackSystemSet::EnemySpawning,
        RollbackSystemSet::EnemyAI,
        RollbackSystemSet::Run,
        RollbackSystemSet::FrameCounter,
    ];

    #[test]
    fn order_has_no_duplicate() {
        let set: BTreeSet<_> = RollbackSystemSet::ORDER.iter().copied().collect();
        assert_eq!(
            set.len(),
            RollbackSystemSet::ORDER.len(),
            "ORDER contient un doublon"
        );
    }

    #[test]
    fn order_contains_every_variant() {
        let order_set: BTreeSet<_> = RollbackSystemSet::ORDER.iter().copied().collect();
        let all_set: BTreeSet<_> = ALL.iter().copied().collect();
        assert_eq!(
            order_set, all_set,
            "ORDER doit contenir chaque variante de RollbackSystemSet exactement une fois"
        );
    }

    #[test]
    fn order_matches_documented_sequence() {
        assert_eq!(
            RollbackSystemSet::ORDER,
            ALL,
            "ORDER doit suivre la séquence documentée dans docs/taches.md (T0.2)"
        );
    }
}
