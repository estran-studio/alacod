//! Bombe du joueur (M4-T0c, `docs/conventions.md` §62) : contrat seul.
//!
//! - [`BombDef`] : définition en données dans le personnage (`bomb: Some((...))`) ; absente, le
//!   bouton `Bomb` (bit `INPUT_BOMB`) n'a aucun effet (aucun contenu existant n'en a).
//! - [`Bomb`] : composant rollback **neutre** de la bombe posée (aucune hors M4-T4).
//! - Pose, mèche et explosion (`effects::Action::Explode`) : M4-T4.

use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use utils::net_id::GgrsNetId;

/// Définition d'une bombe, lue dans le personnage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BombDef {
    /// Frames entre la pose et l'explosion (> 0).
    pub fuse_frames: u32,
    /// Rayon de l'explosion (> 0).
    pub radius: Fixed,
    /// Dégâts aux personnages du rayon (>= 0).
    pub damage: Fixed,
}

/// Une bombe posée. Rollback + trace en variante neutre : portée par aucune entité tant que
/// M4-T4 n'a pas livré la pose.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bomb {
    /// Le joueur qui l'a posée.
    pub owner: GgrsNetId,
    /// Frame (absolue) à laquelle elle explose.
    pub fuse_until: u32,
    pub radius: Fixed,
    pub damage: Fixed,
}

impl Bomb {
    /// Bombe posée à `frame` par `owner` selon `def`.
    pub fn placed(owner: GgrsNetId, frame: u32, def: &BombDef) -> Self {
        Self {
            owner,
            fuse_until: frame.saturating_add(def.fuse_frames),
            radius: def.radius,
            damage: def.damage,
        }
    }

    /// La mèche est consumée à `frame`.
    pub fn is_due(&self, frame: u32) -> bool {
        frame >= self.fuse_until
    }
}

/// Le bouton `Bomb` est-il une demande de pose ? Seulement si le personnage a une définition
/// de bombe : sans `bomb`, l'input est ignoré (contrat M4-T0c).
pub fn bomb_requested(buttons: u32, def: Option<&BombDef>, bit: u32) -> bool {
    def.is_some() && buttons & bit != 0
}

/// Problèmes d'une définition (lint) ; vide si elle est valide.
pub fn bomb_problems(def: &BombDef) -> Vec<String> {
    let mut out = Vec::new();
    if def.fuse_frames == 0 {
        out.push("bomb : fuse_frames = 0 : doit être > 0".to_string());
    }
    if def.radius <= Fixed::ZERO {
        out.push(format!("bomb : rayon = {} : doit être > 0", def.radius));
    }
    if def.damage < Fixed::ZERO {
        out.push(format!(
            "bomb : dégâts = {} : doivent être >= 0",
            def.damage
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def() -> BombDef {
        BombDef {
            fuse_frames: 90,
            radius: Fixed::from_num(40),
            damage: Fixed::from_num(10),
        }
    }

    #[test]
    fn meche_datee_de_la_pose() {
        let bomb = Bomb::placed(GgrsNetId(1, "player".into()), 100, &def());
        assert_eq!(bomb.fuse_until, 190);
        assert!(!bomb.is_due(189));
        assert!(bomb.is_due(190));
    }

    #[test]
    fn sans_definition_le_bouton_est_sans_effet() {
        let bit = 1 << 18;
        assert!(!bomb_requested(bit, None, bit));
        assert!(!bomb_requested(0, Some(&def()), bit));
        assert!(bomb_requested(bit | 1, Some(&def()), bit));
    }

    #[test]
    fn lint_de_la_definition() {
        assert!(bomb_problems(&def()).is_empty());
        let bad = BombDef {
            fuse_frames: 0,
            radius: Fixed::ZERO,
            damage: Fixed::from_num(-1),
        };
        assert_eq!(bomb_problems(&bad).len(), 3);
    }
}
