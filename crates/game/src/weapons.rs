//! Armes, côté jeu : tout `combat::weapons` (simulation) plus le registre que lit la simulation.
//!
//! `combat` est un crate de simulation (ECS seul, `make check_sim_boundary`) : il ne connaît ni
//! `Asset`, ni chargeur RON, ni `Visibility`. Les tables `weapons.ron` et `melee_weapons.ron` sont
//! désérialisées en [`WeaponAsset`] / [`melee::MeleeWeaponAsset`] (types de `combat`). La simulation
//! les lit dans [`WeaponRegistry`] (une ressource de donnée, `BTreeMap`) ; leurs enveloppes d'asset
//! (`WeaponsConfig`, `MeleeWeaponsConfig`), le rechargement à chaud, la synchro qui remplit le
//! registre et la visibilité des armes portées vivent côté application, dans
//! [`crate::weapon_assets`] (D69 étape 2, E3b). Les chemins `game::weapons::…` d'avant ne changent
//! pas : ce module réexporte `combat::weapons`.

use std::collections::BTreeMap;

use bevy::prelude::*;
pub use combat::weapons::*;

use melee::MeleeWeaponAsset;

pub mod melee {
    pub use combat::weapons::melee::*;
}

/// Armes du jeu en cours, lues par la simulation (créateurs de personnages, butin, effets, boutique,
/// progression, divisions de champions, bots, scénarios) : `weapons.ron` (armes à distance) et
/// `melee_weapons.ron` (mêlée), une table par jeu.
///
/// Ressource de donnée : ni rollback ni checksum (écrite hors `GgrsSchedule`, donnée statique d'une
/// partie comme l'étaient `Assets<WeaponsConfig>` et `Assets<MeleeWeaponsConfig>`), `BTreeMap`
/// comme tout registre de simulation. Remplie en `PreUpdate`, avant `RunGgrsSystems`, par
/// `weapon_assets::sync_weapon_registry` ; une surcharge de scénario (`apply_weapon_overrides`)
/// s'écrit directement dessus. Les tables sont lues par `.get` seulement.
#[derive(Resource, Default, Clone)]
pub struct WeaponRegistry {
    pub weapons: BTreeMap<String, WeaponAsset>,
    pub melee: BTreeMap<String, MeleeWeaponAsset>,
}
