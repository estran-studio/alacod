//! Contrats B5 : données de projectiles composables, sans exécution en vague 0.
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum ProjectileModifier {
    Bounce(u32),
    Pierce(u32),
    Size(Fixed),
    Lifetime(u32),
    Homing(Fixed),
    Gravity(Fixed),
}

/// Les projectiles sont référencés par leur id de contenu. Les angles sont en radians,
/// les vitesses en unités/seconde, les durées en frames. Sequence conserve l'ordre RON.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Pattern {
    Aimed {
        count: u32,
        spread: Fixed,
        projectile: String,
    },
    Spread {
        count: u32,
        spread: Fixed,
        projectile: String,
    },
    Ring {
        count: u32,
        speed: Fixed,
        projectile: String,
        every: u32,
    },
    Sequence(Vec<Pattern>),
    Telegraph(u32),
    Wait(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projectile_modifier_ron_round_trip() {
        let values = vec![
            ProjectileModifier::Bounce(2),
            ProjectileModifier::Pierce(3),
            ProjectileModifier::Size(Fixed::from_num(2)),
            ProjectileModifier::Lifetime(120),
            ProjectileModifier::Homing(Fixed::from_num(0.5)),
            ProjectileModifier::Gravity(Fixed::from_num(-1)),
        ];
        let encoded = ron::to_string(&values).unwrap();
        assert!(encoded.contains("Size(\"2\")"));
        assert_eq!(
            ron::from_str::<Vec<ProjectileModifier>>(&encoded).unwrap(),
            values
        );
    }
    #[test]
    fn pattern_ron_round_trip() {
        let sequence: Pattern = ron::from_str(r#"Sequence([Telegraph(60), Aimed(count: 4, spread: "0.1", projectile: "plomb"), Spread(count: 3, spread: "0.5", projectile: "plomb"), Ring(count: 12, speed: "150", projectile: "braise", every: 90), Wait(180)])"#).unwrap();
        assert_eq!(
            ron::from_str::<Pattern>(&ron::to_string(&sequence).unwrap()).unwrap(),
            sequence
        );
    }
}
