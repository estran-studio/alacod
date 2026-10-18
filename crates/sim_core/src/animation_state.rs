//! État logique d'animation d'un personnage : orientation, animation courante, calques actifs.
//!
//! C'est de la **simulation** (rollback, checksum) : l'orientation commande la position de la
//! bouche de l'arme et la direction des coups de mêlée, l'état d'animation est lu par les
//! comportements. Ce qui dessine (feuilles de sprites, timers, `Handle`) reste dans le crate de
//! présentation `animation`, qui lit ces composants. Déménagé de `animation` (dette D69) :
//! `combat` ne dépend plus d'aucun crate de rendu. Les types, leur `Hash` et leur `Debug` sont
//! inchangés : aucune trace ne bouge.
//!
//! [`AnimationStatePlugin`] enregistre les trois composants en rollback. Il remplace
//! l'enregistrement qui vivait dans `animation::D2AnimationPlugin` et s'ajoute au même endroit
//! (avant `GgrsPlugin`, voir `game::core`), donc dans le même ordre d'enregistrement.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use utils::rollback::RollbackTraceApp;

#[derive(Component, Clone, Debug, Hash)]
pub struct ActiveLayers {
    // BTreeMap (pas HashMap) : ce composant est rollback, l'ordre d'itération doit être
    // stable entre clients.
    pub layers: BTreeMap<String, String>,
}

#[derive(Component, Reflect, Default, Clone, Debug, Hash, PartialEq, Eq)]
#[reflect(Component, PartialEq)] // Reflect needed for GGRS state hashing
pub struct AnimationState(pub String);

#[derive(
    Component, Reflect, Debug, Clone, Copy, Hash, PartialEq, Eq, Default, Serialize, Deserialize,
)]
#[reflect(Component, PartialEq)]
pub enum FacingDirection {
    #[default]
    Right, // 0 degrees
    UpRight,   // 45 degrees
    Up,        // 90 degrees
    UpLeft,    // 135 degrees
    Left,      // 180 degrees
    DownLeft,  // 225 degrees
    Down,      // 270 degrees
    DownRight, // 315 degrees
}

/// Vrai si `atan2_fixed(y, x)` déborderait. Rejoue les produits du polynôme choisi par octant
/// (`|y| < |x|` : `x·y / (x² + 0,28125·y²)`, sinon `x·y / (y² + 0,28125·x²)`) avec les mêmes arrondis.
fn atan2_would_overflow(
    y: bevy_fixed::fixed_math::Fixed,
    x: bevy_fixed::fixed_math::Fixed,
) -> bool {
    use bevy_fixed::fixed_math::{Fixed, FIXED_ZERO};
    // Sur un axe, `atan2` répond sans aucun produit.
    if x == FIXED_ZERO || y == FIXED_ZERO {
        return false;
    }
    let (major, minor) = if y.saturating_abs() < x.saturating_abs() {
        (x, y)
    } else {
        (y, x)
    };
    let coef = Fixed::from_num(0.28125);
    let fits = || -> Option<()> {
        x.checked_mul(y)?;
        let major_sq = major.checked_mul(major)?;
        let minor_sq = minor.checked_mul(minor)?;
        major_sq.checked_add(coef.checked_mul(minor_sq)?)?;
        Some(())
    };
    fits().is_none()
}

impl FacingDirection {
    /// Returns -1 for left-ish directions, 1 for right-ish directions
    pub fn to_int(&self) -> i32 {
        match self {
            FacingDirection::Right | FacingDirection::UpRight | FacingDirection::DownRight => 1,
            FacingDirection::Left | FacingDirection::UpLeft | FacingDirection::DownLeft => -1,
            FacingDirection::Up | FacingDirection::Down => 0,
        }
    }

    /// Returns the angle in radians for this direction
    pub fn to_radians(&self) -> f32 {
        match self {
            FacingDirection::Right => 0.0,
            FacingDirection::UpRight => std::f32::consts::PI / 4.0,
            FacingDirection::Up => std::f32::consts::PI / 2.0,
            FacingDirection::UpLeft => 3.0 * std::f32::consts::PI / 4.0,
            FacingDirection::Left => std::f32::consts::PI,
            FacingDirection::DownLeft => 5.0 * std::f32::consts::PI / 4.0,
            FacingDirection::Down => 3.0 * std::f32::consts::PI / 2.0,
            FacingDirection::DownRight => 7.0 * std::f32::consts::PI / 4.0,
        }
    }

    /// Returns the unit vector for this direction
    pub fn to_vector(&self) -> bevy::math::Vec2 {
        let angle = self.to_radians();
        bevy::math::Vec2::new(angle.cos(), angle.sin())
    }

    /// Determines the facing direction from a 2D vector (using f32 for non-rollback systems)
    pub fn from_vector(vec: bevy::math::Vec2) -> Self {
        if vec.length_squared() < 0.001 {
            return FacingDirection::default();
        }

        let angle = vec.y.atan2(vec.x);
        let normalized_angle = if angle < 0.0 {
            angle + 2.0 * std::f32::consts::PI
        } else {
            angle
        };

        // Divide circle into 8 equal segments (45 degrees each)
        let segment = ((normalized_angle + std::f32::consts::PI / 8.0)
            / (std::f32::consts::PI / 4.0)) as u8
            % 8;

        match segment {
            0 => FacingDirection::Right,
            1 => FacingDirection::UpRight,
            2 => FacingDirection::Up,
            3 => FacingDirection::UpLeft,
            4 => FacingDirection::Left,
            5 => FacingDirection::DownLeft,
            6 => FacingDirection::Down,
            7 => FacingDirection::DownRight,
            _ => FacingDirection::Right,
        }
    }

    /// Determines the facing direction from a fixed-point 2D vector (for deterministic rollback systems)
    pub fn from_fixed_vector(vec: bevy_fixed::fixed_math::FixedVec2) -> Self {
        use bevy_fixed::fixed_math;

        // `atan2_fixed` (polynômes de Lyons, `fixed_trigonometry`) déborde (panique en debug) quand `x·y`, `x²` ou
        // le dénominateur dépassent la plage de `Fixed` : vitesse de plus de ~181 (un champion rapide, graine 54 de
        // `etages_deux`, m2-salles-murs-gungeon). Seule la direction compte : on ne réduit (÷4) que si le calcul
        // déborderait vraiment, en rejouant les mêmes opérations en `checked_*`. Toute entrée qui ne paniquait pas
        // passe inchangée : aucune trace ne bouge.
        let mut vec = vec;
        while atan2_would_overflow(vec.y, vec.x) {
            let four = fixed_math::new(4.0);
            vec = bevy_fixed::fixed_math::FixedVec2::new(vec.x / four, vec.y / four);
        }
        if vec.length_squared() < fixed_math::new(0.001) {
            return FacingDirection::default();
        }

        let angle = fixed_math::atan2_fixed(vec.y, vec.x);
        let two_pi = fixed_math::new(2.0) * fixed_math::FIXED_PI;
        let normalized_angle = if angle < fixed_math::FIXED_ZERO {
            angle + two_pi
        } else {
            angle
        };

        // Divide circle into 8 equal segments (45 degrees each)
        let pi_over_8 = fixed_math::FIXED_PI / fixed_math::new(8.0);
        let pi_over_4 = fixed_math::FIXED_PI / fixed_math::new(4.0);
        let segment = ((normalized_angle + pi_over_8) / pi_over_4).to_num::<u8>() % 8;

        match segment {
            0 => FacingDirection::Right,
            1 => FacingDirection::UpRight,
            2 => FacingDirection::Up,
            3 => FacingDirection::UpLeft,
            4 => FacingDirection::Left,
            5 => FacingDirection::DownLeft,
            6 => FacingDirection::Down,
            7 => FacingDirection::DownRight,
            _ => FacingDirection::Right,
        }
    }

    /// Check if this direction is primarily horizontal
    pub fn is_horizontal(&self) -> bool {
        matches!(self, FacingDirection::Left | FacingDirection::Right)
    }

    /// Check if sprite should be flipped horizontally
    pub fn should_flip_x(&self) -> bool {
        matches!(
            self,
            FacingDirection::Left | FacingDirection::UpLeft | FacingDirection::DownLeft
        )
    }
}

/// État d'animation logique : fait partie de la simulation (rollback).
#[derive(Bundle)]
pub struct AnimationStateBundle {
    state: AnimationState,
    active_layers: ActiveLayers,
    facing_direction: FacingDirection,
}

impl AnimationStateBundle {
    pub fn new(starting_layers: BTreeMap<String, String>) -> Self {
        Self {
            state: AnimationState("Idle".into()),
            active_layers: ActiveLayers {
                layers: starting_layers,
            },
            facing_direction: FacingDirection::default(),
        }
    }
}

/// Enregistre `AnimationState`, `FacingDirection` et `ActiveLayers` en rollback (checksum et
/// trace), dans cet ordre.
pub struct AnimationStatePlugin;

impl Plugin for AnimationStatePlugin {
    fn build(&self, app: &mut App) {
        app.rollback_and_trace::<AnimationState>()
            .rollback_and_trace::<FacingDirection>()
            .rollback_and_trace::<ActiveLayers>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};

    /// `atan2_would_overflow` est exact : il prédit la panique de `atan2_fixed` (build de test = debug,
    /// débordements vérifiés) pour chaque entrée, y compris autour de la borne (~181 de long, axes et diagonales).
    #[test]
    fn atan2_overflow_predicate_matches_panics() {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let mut mismatches = Vec::new();
        let mut overflowing = 0;
        let mut ok = 0;
        for xi in -40..=40 {
            for yi in -40..=40 {
                for &(scale, jitter) in &[
                    (1.0f32, 0.0f32),
                    (4.5, 0.0),
                    (4.5, 0.03),
                    (4.52, 0.0),
                    (4.53, 0.0),
                    (5.0, 0.0),
                ] {
                    let x = Fixed::from_num(xi as f32 * scale + jitter);
                    let y = Fixed::from_num(yi as f32 * scale);
                    let panicked =
                        std::panic::catch_unwind(|| fixed_math::atan2_fixed(y, x)).is_err();
                    if panicked != atan2_would_overflow(y, x) {
                        mismatches.push((x, y));
                    }
                    if panicked {
                        overflowing += 1;
                    } else {
                        ok += 1;
                    }
                }
            }
        }
        std::panic::set_hook(previous_hook);
        assert!(mismatches.is_empty(), "prédiction fausse : {mismatches:?}");
        assert!(overflowing > 1000 && ok > 1000, "grille sans les deux cas");
    }

    /// Aux bornes : juste sous la limite, la direction est celle de `atan2` (inchangée) ; au-delà, elle est
    /// celle du vecteur réduit ; jamais de panique.
    #[test]
    fn from_fixed_vector_bounds() {
        let v = |x: f32, y: f32| FixedVec2::new(fixed_math::new(x), fixed_math::new(y));
        // Sur un axe, `atan2` ne multiplie rien : aucune réduction, même à 400.
        assert_eq!(
            FacingDirection::from_fixed_vector(v(181.0, 0.0)),
            FacingDirection::Right
        );
        assert_eq!(
            FacingDirection::from_fixed_vector(v(-182.0, 0.0)),
            FacingDirection::Left
        );
        assert_eq!(
            FacingDirection::from_fixed_vector(v(0.0, 400.0)),
            FacingDirection::Up
        );
        assert_eq!(
            FacingDirection::from_fixed_vector(v(0.0, -400.0)),
            FacingDirection::Down
        );
        // Diagonales : x·y déborde dès 181 de côté, la direction reste la bonne.
        assert_eq!(
            FacingDirection::from_fixed_vector(v(300.0, 300.0)),
            FacingDirection::UpRight
        );
        assert_eq!(
            FacingDirection::from_fixed_vector(v(-300.0, -300.0)),
            FacingDirection::DownLeft
        );
        // Cas de la graine 54 (champion rapide).
        assert_eq!(
            FacingDirection::from_fixed_vector(v(-181.6, -11.7)),
            FacingDirection::Left
        );
        // Petit vecteur : direction par défaut.
        assert_eq!(
            FacingDirection::from_fixed_vector(v(0.0, 0.0)),
            FacingDirection::default()
        );
    }
}
