//! Contrats C1 : déclencheurs et conditions, sans exécution en vague 0.
use crate::Action;
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::tag::Tag;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum GaugeThreshold {
    Above(Fixed),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum On {
    OnHit,
    OnKill,
    OnDamageTaken,
    OnDodge,
    OnReload,
    OnRoomClear,
    OnPickup,
    OnUse,
    OnGauge(String, GaugeThreshold),
    Tick(u32),
    OnEvent(String),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Condition {
    TargetTag(Tag),
    Carrying(String),
    HpBelow(Fixed),
    HasTag(Tag),
    SquadSize(u32),
    TargetInRange(Fixed),
    NotHitFor(u32),
}

/// Conditions et actions conservées dans l'ordre de déclaration RON.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Effect {
    pub on: On,
    #[serde(default, rename = "if")]
    pub r#if: Vec<Condition>,
    #[serde(rename = "do")]
    pub r#do: Vec<Action>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gauge_threshold_ron_round_trip() {
        let value = GaugeThreshold::Above(Fixed::from_num(0.75));
        assert_eq!(
            ron::from_str::<GaugeThreshold>(&ron::to_string(&value).unwrap()).unwrap(),
            value
        );
    }
    #[test]
    fn on_ron_round_trip() {
        let values = vec![
            On::OnHit,
            On::OnKill,
            On::OnDamageTaken,
            On::OnDodge,
            On::OnReload,
            On::OnRoomClear,
            On::OnPickup,
            On::OnUse,
            On::OnGauge("sacre".into(), GaugeThreshold::Above(Fixed::from_num(0.75))),
            On::Tick(60),
            On::OnEvent("porte".into()),
        ];
        assert_eq!(
            ron::from_str::<Vec<On>>(&ron::to_string(&values).unwrap()).unwrap(),
            values
        );
    }
    #[test]
    fn condition_ron_round_trip() {
        let values = vec![
            Condition::TargetTag(Tag::new("damne")),
            Condition::Carrying("objet".into()),
            Condition::HpBelow(Fixed::from_num(0.2)),
            Condition::HasTag(Tag::new("refractaire")),
            Condition::SquadSize(3),
            Condition::TargetInRange(Fixed::from_num(250)),
            Condition::NotHitFor(180),
        ];
        assert_eq!(
            ron::from_str::<Vec<Condition>>(&ron::to_string(&values).unwrap()).unwrap(),
            values
        );
    }
    #[test]
    fn effect_uses_on_if_do_ron_fields() {
        let value: Effect = ron::from_str(r#"(on: OnGauge("sacre", Above("0.75")), if: [TargetTag("damne"), HpBelow("0.2")], do: [RefillAmmo, CurrencyMultiplier(factor: "2", frames: 60)])"#).unwrap();
        let encoded = ron::to_string(&value).unwrap();
        assert!(encoded.contains("if:"));
        assert!(encoded.contains("do:"));
        assert_eq!(ron::from_str::<Effect>(&encoded).unwrap(), value);
    }
}
