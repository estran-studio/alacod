//! Contrat de configuration des interactions, consommé par game.
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use serde::{Deserialize, Serialize};

/// Component that marks an entity as interactable
#[derive(Component, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Interactable {
    /// Range within which interaction is possible
    pub interaction_range: fixed_math::Fixed,
    /// Type of interaction
    pub interaction_type: InteractionType,
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            interaction_range: fixed_math::new(50.0),
            interaction_type: InteractionType::Door,
        }
    }
}

/// Types of interactions available
#[derive(Clone, Copy, Debug, Hash, Serialize, Deserialize, Reflect, PartialEq, Eq)]
pub enum InteractionType {
    Door,
    Window,
    /// Réanimer un joueur à terre (T1.3, chantier B6). Posé avec `Interactable` sur le
    /// joueur à terre lui-même (`character::health::rollback_apply_accumulated_damage`,
    /// `character::health::rollback_apply_bleedout`) ; consommé par
    /// `game::interaction::handle_revive_interaction`.
    Revive,
    /// Ramasser une arme au sol (T2.2, chantier B7). Posé avec `Interactable` sur une
    /// entité `weapons::WeaponPickup` (`weapons::spawn_weapon_pickup` : lâcher, ou une
    /// arme murale T2.3) ; consommé par `game::interaction::handle_weapon_pickup_interaction`.
    Weapon,
    /// Acheter un perk (T2.3, chantier C5 v1). Posé avec `Interactable` sur une entité
    /// `economy::PerkMachine` (`map_ldtk::game::local::spawn_soda_locations_when_map_loaded`,
    /// entité LDtk `SodaLocation`) ; consommé par `game::interaction::handle_perk_purchase_interaction`.
    Perk,
    // Future: Crate, etc.
}
