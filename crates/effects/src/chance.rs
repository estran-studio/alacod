//! M3-T0a (`docs/conventions.md` §60) : chance d'effet liée à la stat `Luck`.
//!
//! M3-T1 (§66) l'appelle pour `Action::Chance` (`on_hit` d'un projectile) ; M3-T2 pour les effets
//! d'objet. Le lecteur de `Luck` déclaré par `sim_core::stats::stat_reader`.

use bevy_fixed::fixed_math::{Fixed, FIXED_ONE};
use bevy_fixed::rng::{fnv1a, RollbackRng};

/// Chance finale d'un effet : `clamp(base + luck × per_point, 0, 1)`, tout en `Fixed` (jamais de
/// `f32`), opérations saturantes. `base` : chance du contenu (ex. `"0.1"`) ; `luck` : stat `Luck`
/// résolue du porteur (0 sans modificateur) ; `per_point` : gain par point de chance (négatif
/// possible : un effet que la chance rend plus rare).
pub fn chance_with_luck(base: Fixed, luck: Fixed, per_point: Fixed) -> Fixed {
    base.saturating_add(luck.saturating_mul(per_point))
        .clamp(Fixed::ZERO, FIXED_ONE)
}

/// Graine du tirage de chance d'une action : `(graine de run, net id du projectile, rang de
/// l'action)`, mélangés (splitmix64 : deux projectiles aux net ids voisins ne donnent pas des
/// tirages voisins, ce que fait un `LCG` amorcé par un simple XOR). Aucun état partagé : le
/// tirage ne dépend ni de l'ordre des systèmes ni des autres consommateurs de `RngStreams`.
pub fn chance_seed(run_seed: u32, net_id: usize, rank: u32) -> u32 {
    let mut z = fnv1a(b"chance")
        ^ (u64::from(run_seed) << 32)
        ^ (net_id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(rank).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as u32
}

/// Tire la chance `chance` (dans `[0, 1]`) avec le RNG local de [`chance_seed`] : `true` avec une
/// probabilité `chance` (granularité 2⁻¹⁶, `RollbackRng::next_fixed` ∈ `[0, 1)`). 0 n'aboutit
/// jamais, 1 aboutit toujours, sans consommer de tirage.
pub fn roll_chance(run_seed: u32, net_id: usize, rank: u32, chance: Fixed) -> bool {
    if chance <= Fixed::ZERO {
        return false;
    }
    if chance >= FIXED_ONE {
        return true;
    }
    RollbackRng::new(chance_seed(run_seed, net_id, rank)).next_fixed() < chance
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

    #[test]
    fn tirage_local_deterministe_et_independant_du_rang() {
        let a = roll_chance(7, 42, 0, fx(0.5));
        assert_eq!(
            a,
            roll_chance(7, 42, 0, fx(0.5)),
            "même graine, même tirage"
        );
        assert_ne!(chance_seed(7, 42, 0), chance_seed(7, 42, 1));
        assert_ne!(chance_seed(7, 42, 0), chance_seed(7, 43, 0));
        assert_ne!(chance_seed(7, 42, 0), chance_seed(8, 42, 0));
    }

    #[test]
    fn tirage_aux_bornes_et_frequence() {
        assert!(!roll_chance(1, 1, 0, Fixed::ZERO));
        assert!(roll_chance(1, 1, 0, FIXED_ONE));
        // 4 000 projectiles consécutifs : une chance de 0,5 tombe près de la moitié (le mélange
        // casse la corrélation des net ids voisins).
        let hits = (0..4000)
            .filter(|net_id| roll_chance(123_456, *net_id, 0, fx(0.5)))
            .count();
        assert!((1800..2200).contains(&hits), "{hits} succès sur 4000");
        let quarter = (0..4000)
            .filter(|net_id| roll_chance(123_456, *net_id, 3, fx(0.25)))
            .count();
        assert!((800..1200).contains(&quarter), "{quarter} succès sur 4000");
    }
}
