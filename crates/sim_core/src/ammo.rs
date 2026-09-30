//! Type de munition d'une arme à distance (T2.2, chantier B7 « Munitions typées et
//! inventaire d'armes »). Sert de clé au réserve partagée par joueur
//! (`combat::inventory::AmmoReserves`, `BTreeMap<AmmoType, u32>`) : deux armes qui déclarent
//! le même `AmmoType` puisent dans le même stock.
//!
//! Enum ouvert (`Custom`), comme [`crate::stats::StatId`] : cinq types couvrent le contenu
//! `zombies` d'aujourd'hui, `Custom(String)` laisse un jeu définir les siens sans toucher à
//! l'engine.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AmmoType {
    /// Munition de pistolet (`pistol` dans le contenu `zombies`).
    Plomb,
    /// Munition d'arme automatique (`machine_gun`).
    Balle,
    /// Cartouche de fusil à pompe (`shotgun`).
    Cartouche,
    /// Munition d'arme à énergie (aucune arme du contenu `zombies` aujourd'hui).
    Energie,
    /// Munition hors catégorie courante.
    Special,
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_ron() {
        for ammo in [
            AmmoType::Plomb,
            AmmoType::Balle,
            AmmoType::Cartouche,
            AmmoType::Energie,
            AmmoType::Special,
            AmmoType::Custom("plasma".into()),
        ] {
            let ron = ron::to_string(&ammo).expect("sérialisation RON");
            let back: AmmoType = ron::from_str(&ron).expect("désérialisation RON");
            assert_eq!(ammo, back);
        }
    }

    #[test]
    fn distinct_custom_names_are_distinct_keys() {
        use std::collections::BTreeMap;
        let mut reserves: BTreeMap<AmmoType, u32> = BTreeMap::new();
        reserves.insert(AmmoType::Custom("plasma".into()), 10);
        reserves.insert(AmmoType::Custom("laser".into()), 5);
        assert_eq!(reserves.get(&AmmoType::Custom("plasma".into())), Some(&10));
        assert_eq!(reserves.get(&AmmoType::Custom("laser".into())), Some(&5));
    }
}
