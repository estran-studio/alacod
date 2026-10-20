//! Visée d'un personnage (M3 suivi Q17, `docs/conventions.md` §66) : libre (360°, défaut) ou à
//! quatre directions (`isaac` : les larmes partent vers le haut, le bas, la gauche ou la droite).
//!
//! # Composant statique, hors rollback, absent par défaut
//!
//! [`AimMode`] est une option de personnage (`aim:` du RON, `CharacterConfig::aim`), posée une fois à
//! la création du joueur **seulement si elle n'est pas `Free`** : un personnage sans `aim:` ne porte
//! aucun composant, ni dans le checksum GGRS (le composant n'est pas enregistré en rollback, comme
//! [`crate::team::Team`]) ni dans la trace. Les autres jeux ne voient donc aucune trace bouger.
//! Comme `Team`, il survit à un rollback tant que l'entité n'est détruite que par
//! `despawn_rollback()` (règle 9 de `CLAUDE.md`).

use bevy::prelude::Component;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use serde::{Deserialize, Serialize};

/// Mode de visée d'un personnage joueur.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AimMode {
    /// Visée libre à 360° (comportement historique).
    #[default]
    Free,
    /// Visée quantifiée à la direction la plus proche parmi haut, bas, gauche, droite
    /// (voir [`four_way`]).
    FourWay,
}

/// Direction de visée d'une entrée (`pan_x`, `pan_y` en `[-127, 127]`), unitaire, ou `ZERO` sans
/// visée. `Free` : le vecteur `pan` normalisé (le calcul historique du tir, au bit près) ;
/// `FourWay` : l'axe dominant de `pan` ([`four_way`]).
pub fn aim_direction(mode: AimMode, pan_x: i16, pan_y: i16) -> FixedVec2 {
    match mode {
        AimMode::Free => {
            let mut aim_dir = FixedVec2::new(Fixed::from_num(pan_x), Fixed::from_num(pan_y));
            aim_dir.x /= fixed_math::new(127.0);
            aim_dir.y /= fixed_math::new(127.0);
            aim_dir.normalize_or_zero()
        }
        AimMode::FourWay => {
            let (x, y) = four_way(pan_x, pan_y);
            FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
        }
    }
}

/// Quantifie `pan` à la direction la plus proche parmi `(±1, 0)` et `(0, ±1)` : l'axe de plus
/// grande valeur absolue l'emporte, avec le signe de sa composante. **Égalité exacte des deux axes
/// (diagonale parfaite) : l'axe horizontal l'emporte.** `pan` nul : `(0, 0)`, pas de tir. Comparaison
/// sur les entiers du `pan`, avant toute normalisation : aucune division, aucun arrondi.
pub fn four_way(pan_x: i16, pan_y: i16) -> (i8, i8) {
    let (ax, ay) = (i32::from(pan_x).abs(), i32::from(pan_y).abs());
    if ax == 0 && ay == 0 {
        (0, 0)
    } else if ax >= ay {
        (pan_x.signum() as i8, 0)
    } else {
        (0, pan_y.signum() as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_way_garde_l_axe_dominant_et_son_signe() {
        assert_eq!(four_way(100, 30), (1, 0));
        assert_eq!(four_way(-100, 30), (-1, 0));
        assert_eq!(four_way(30, 100), (0, 1));
        assert_eq!(four_way(30, -100), (0, -1));
        assert_eq!(four_way(127, 0), (1, 0));
        assert_eq!(four_way(0, -127), (0, -1));
    }

    /// Diagonale parfaite : l'horizontale gagne, quels que soient les signes ; sans visée, rien.
    #[test]
    fn four_way_egalite_va_a_l_horizontale() {
        assert_eq!(four_way(50, 50), (1, 0));
        assert_eq!(four_way(-50, 50), (-1, 0));
        assert_eq!(four_way(50, -50), (1, 0));
        assert_eq!(four_way(-50, -50), (-1, 0));
        assert_eq!(four_way(0, 0), (0, 0));
    }

    /// Les extrêmes de `i16` ne débordent pas (`abs` en 32 bits).
    #[test]
    fn four_way_supporte_les_extremes() {
        assert_eq!(four_way(i16::MIN, 0), (-1, 0));
        assert_eq!(four_way(0, i16::MIN), (0, -1));
        assert_eq!(four_way(i16::MIN, i16::MAX), (-1, 0));
    }

    /// `Free` est le calcul d'avant (normalisation du `pan`), `FourWay` rend un axe unitaire exact.
    #[test]
    fn modes_de_visee() {
        assert_eq!(aim_direction(AimMode::Free, 0, 0), FixedVec2::ZERO);
        assert_eq!(aim_direction(AimMode::FourWay, 0, 0), FixedVec2::ZERO);
        assert_eq!(
            aim_direction(AimMode::FourWay, 90, -40),
            FixedVec2::new(Fixed::from_num(1), Fixed::ZERO)
        );
        assert_eq!(
            aim_direction(AimMode::FourWay, -5, -40),
            FixedVec2::new(Fixed::ZERO, Fixed::from_num(-1))
        );
        let free = aim_direction(AimMode::Free, 100, 100);
        assert_eq!(free.x, free.y);
        let norm = free.x.to_num::<f64>().hypot(free.y.to_num::<f64>());
        assert!((norm - 1.0).abs() < 1e-3, "norme {norm}");
    }

    /// Le défaut est `Free` (un RON sans `aim:` ne change rien) et le RON se lit.
    #[test]
    fn defaut_libre_et_ron() {
        assert_eq!(AimMode::default(), AimMode::Free);
        assert_eq!(
            ron::from_str::<AimMode>("FourWay").unwrap(),
            AimMode::FourWay
        );
        assert_eq!(ron::from_str::<AimMode>("Free").unwrap(), AimMode::Free);
    }
}
