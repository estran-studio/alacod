use bevy::prelude::SystemSet;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum RollbackSystemSet {
    /// Début de frame : nettoyage des `FrameEvents` de la frame précédente
    FrameStart,
    Input,
    Interaction,
    Movement,
    Weapon,
    CollisionDamage,
    DeathManagement,
    AnimationUpdates,
    EnemySpawning,
    EnemyAI,
    FrameCounter,
}
