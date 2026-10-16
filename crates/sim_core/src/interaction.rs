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
    /// Ramasser un objet passif ou actif au sol (M2-T0b, `docs/conventions.md` §36). Posé avec
    /// `Interactable` sur une entité `items::ItemPickup` (`game::items::spawn_item_pickup`) ;
    /// consommé par `game::items::handle_item_interaction`.
    Item,
    /// Ouvrir un coffre (M2-T5, `docs/conventions.md` §48). Posé avec `Interactable` sur une entité
    /// `items::Chest` (`game::loot::spawn_chest`) ; consommé par `game::loot::chest_interaction_system`.
    Chest,
    /// Renverser une table (M2-T11, `docs/conventions.md` §46). Posé avec `Interactable` sur
    /// une entité `world::TableState` debout (`map_ldtk::game::plugin::spawn_map_items`) ;
    /// consommé par `game::props::table_flip_system`.
    Table,
    /// Acheter un objet sur un présentoir (M2-T6, `docs/conventions.md` §50). Posé avec
    /// `Interactable` sur une entité `items::Shelf` (`game::shop::spawn_shelf`) ; consommé par
    /// `game::shop::shelf_interaction_system`.
    Shop,
    /// Utiliser une station du hub (M2-T13, `docs/conventions.md` §57). Posé avec `Interactable`
    /// sur une entité `game::hub::HubStation` ; consommé par `game::hub::hub_station_system`.
    Hub,
    /// Ouvrir une porte à clé (M3-T5, `docs/conventions.md` §70). Posé avec `Interactable` (même
    /// portée que la porte ordinaire qu'il remplace) sur une porte scellée d'une salle
    /// `door_lock: Key(n)` (`map_ldtk::game::rooms::room_lock_system`) ; consommé par
    /// `map_ldtk::game::rooms::key_door_system`, jamais par `game::interaction::handle_door_interaction`.
    KeyDoor,
    // Future: Crate, etc.
}
