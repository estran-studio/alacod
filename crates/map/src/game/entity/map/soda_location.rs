use bevy::prelude::*;

/// Marqueur posé par l'entité LDtk `SodaLocation` (T2.3, chantier C5 v1) : au chargement de
/// la map, `map_ldtk::game::local::spawn_soda_locations_when_map_loaded` lit ce composant sur
/// l'entité d'origine LDtk et fait apparaître la machine à perk correspondante
/// (`game::economy::PerkMachine`).
///
/// `perk` reste une chaîne brute (`PerkId` du registre, `games/<jeu>/assets/economy/perks.ron`)
/// pour la même raison que `WeaponLocationComponent::weapon` (voir sa doc) : ce crate ne
/// dépend ni de `content` ni de `game`. Pas de champ `price` : le prix vit dans `perks.ron`,
/// une seule source de vérité (voir la doc de `game::economy::PerkMachine`).
#[derive(Component, Clone, Debug, Default)]
pub struct SodaLocationComponent {
    /// `PerkId` du registre (`economy/perks.ron`).
    pub perk: String,
}
