use bevy::prelude::*;

/// Marqueur posé par l'entité LDtk `WeaponLocation` (T2.3, chantier C5 v1) : au chargement
/// de la map, `map_ldtk::game::local::spawn_weapon_locations_when_map_loaded` lit ce
/// composant sur l'entité d'origine LDtk et fait apparaître l'arme murale correspondante
/// (`weapons::spawn_weapon_pickup`, `price: Some(self.price)`).
///
/// `weapon` reste une chaîne brute (`WeaponId` du registre, `games/<jeu>/assets/weapons.ron`)
/// plutôt qu'un type du crate `content` : ce crate (`map`) ne dépend ni de `content` ni de
/// `game`, comme `CharacterSpawnComponent` (voir sa doc).
#[derive(Component, Clone, Debug, Default)]
pub struct WeaponLocationComponent {
    /// `WeaponId` du registre (`weapons.ron`).
    pub weapon: String,
    /// Prix d'achat, en points (`run::currency::Currency`).
    pub price: u32,
}
