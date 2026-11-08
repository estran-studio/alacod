use bevy::prelude::*;

/// Serrure d'une porte de carte (X-6a1, `docs/conventions.md` §117) : valeur d'**auteur** (champs LDtk
/// `lock_key` / `lock_consume`), portée telle quelle à travers la génération (contrairement au coût,
/// recalculé de la profondeur) ; une porte appariée partage la serrure de sa jumelle.
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct DoorLockConfig {
    /// Identifiant de la clé d'équipe (`items::TeamKeys`) qui ouvre la porte ; jamais vide.
    pub key: String,
    /// La clé est retirée de l'état d'équipe à l'ouverture (`true`) ou gardée (`false`).
    pub consume: bool,
}

#[derive(Debug, Clone, Reflect)]
pub struct DoorConfig {
    // cost to open the door
    pub cost: i32,
    // if the door need electricity to be open
    pub electrify: bool,
    // Whether this door is interactable (false for dead-ends)
    pub interactable: bool,
    // If this door is paired with another (at a room connection), store the paired door's level_iid and position
    pub paired_door: Option<(String, (i32, i32))>, // (level_iid, position)
    /// Serrure à clé (X-6a1) : `None` pour toute porte existante.
    pub lock: Option<DoorLockConfig>,
}

impl Default for DoorConfig {
    fn default() -> Self {
        Self {
            cost: 0,
            electrify: false,
            interactable: true, // Doors are interactable by default
            paired_door: None,
            lock: None,
        }
    }
}
