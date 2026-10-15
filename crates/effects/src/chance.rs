//! M4-T0a (`docs/conventions.md` §60) : chance d'effet liée à la stat `Luck`.
//!
//! Contrat seulement : rien ne l'appelle encore dans la simulation (M4-T1 pour les statuts
//! posés par une larme, M4-T2 pour les effets d'objet). Le lecteur de `Luck` déclaré par
//! `sim_core::stats::stat_reader`.

use bevy_fixed::fixed_math::{Fixed, FIXED_ONE};

/// Chance finale d'un effet : `clamp(base + luck × per_point, 0, 1)`, tout en `Fixed` (jamais de
/// `f32`), opérations saturantes. `base` : chance du contenu (ex. `"0.1"`) ; `luck` : stat `Luck`
/// résolue du porteur (0 sans modificateur) ; `per_point` : gain par point de chance (négatif
/// possible : un effet que la chance rend plus rare).
pub fn chance_with_luck(base: Fixed, luck: Fixed, per_point: Fixed) -> Fixed {
    base.saturating_add(luck.saturating_mul(per_point))
        .clamp(Fixed::ZERO, FIXED_ONE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    #[test]
    fn sans_chance_la_base_est_rendue_exactement() {
        assert_eq!(chance_with_luck(fx(0.25), Fixed::ZERO, fx(0.1)), fx(0.25));
    }

    #[test]
    fn chaque_point_ajoute_per_point() {
        assert_eq!(chance_with_luck(fx(0.25), fx(2.0), fx(0.125)), fx(0.5));
        assert_eq!(chance_with_luck(fx(0.5), fx(-1.0), fx(0.25)), fx(0.25));
    }

    #[test]
    fn bornee_a_zero_et_un() {
        assert_eq!(chance_with_luck(fx(0.9), fx(10.0), fx(0.1)), FIXED_ONE);
        assert_eq!(chance_with_luck(fx(0.1), fx(-10.0), fx(0.1)), Fixed::ZERO);
        assert_eq!(
            chance_with_luck(fx(2.0), Fixed::ZERO, Fixed::ZERO),
            FIXED_ONE
        );
        // Saturation : pas de panique sur des valeurs extrêmes.
        assert_eq!(
            chance_with_luck(Fixed::MAX, Fixed::MAX, Fixed::MAX),
            FIXED_ONE
        );
    }
}
