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
    /// T1.10 : montée de niveau du porteur (C4, jauge de progression franchie).
    OnLevelUp,
    /// M2-T3 : début d'un étage du porteur, le premier compris (index d'étage qui change).
    OnFloorStart,
    /// M2-T3 : le porteur utilise une Blank (M2-T1, `FrameEvents<BlankUsed>`) : contrat et lint
    /// seulement tant que M2-T1 n'est pas mergée.
    OnBlank,
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

/// Quand les charges d'un effet reviennent (M2-T3, `Effect::charges`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Recharge {
    /// À chaque salle nettoyée.
    PerRoom,
    /// À chaque début d'étage.
    PerFloor,
}

/// Conditions et actions conservées dans l'ordre de déclaration RON.
///
/// M2-T3 : `cooldown` (frames minimales entre deux exécutions), `charges` (nombre d'exécutions
/// avant épuisement) et `recharge` (retour des charges) sont optionnels ; sans eux, l'effet se
/// comporte exactement comme en v1. Le `Hash` est écrit à la main : un effet sans ces champs
/// garde le hash d'avant (traces existantes inchangées).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Effect {
    pub on: On,
    #[serde(default, rename = "if")]
    pub r#if: Vec<Condition>,
    #[serde(rename = "do")]
    pub r#do: Vec<Action>,
    #[serde(default)]
    pub cooldown: Option<u32>,
    #[serde(default)]
    pub charges: Option<u32>,
    #[serde(default)]
    pub recharge: Option<Recharge>,
}

impl std::hash::Hash for Effect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.on.hash(state);
        self.r#if.hash(state);
        self.r#do.hash(state);
        // Champs v2 : rien d'écrit tant qu'ils sont absents (hash v1 conservé).
        if self.cooldown.is_some() || self.charges.is_some() || self.recharge.is_some() {
            self.cooldown.hash(state);
            self.charges.hash(state);
            self.recharge.hash(state);
        }
    }
}

impl Effect {
    /// Effet v1 : sans cooldown ni charges.
    pub fn new(on: On, r#if: Vec<Condition>, r#do: Vec<Action>) -> Self {
        Self {
            on,
            r#if,
            r#do,
            cooldown: None,
            charges: None,
            recharge: None,
        }
    }
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
            On::OnLevelUp,
            On::OnFloorStart,
            On::OnBlank,
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

#[cfg(test)]
mod v2_tests {
    use super::*;
    use std::hash::{Hash, Hasher};

    fn hash_of(effect: &Effect) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        effect.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn effet_v1_garde_son_hash() {
        // Hash d'un effet v1 = hash de ses trois champs d'avant M2-T3.
        let effect = Effect::new(On::OnKill, vec![Condition::HasTag(Tag::new("a"))], vec![]);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        effect.on.hash(&mut hasher);
        effect.r#if.hash(&mut hasher);
        effect.r#do.hash(&mut hasher);
        assert_eq!(hash_of(&effect), hasher.finish());
        let limited = Effect {
            cooldown: Some(60),
            ..effect.clone()
        };
        assert_ne!(hash_of(&limited), hash_of(&effect));
    }

    #[test]
    fn champs_v2_ron_optionnels() {
        let v1: Effect = ron::from_str("(on: OnRoomClear, do: [])").unwrap();
        assert_eq!((v1.cooldown, v1.charges, v1.recharge), (None, None, None));
        let v2: Effect = ron::from_str(
            "(on: OnReload, do: [], cooldown: Some(120), charges: Some(2), recharge: Some(PerFloor))",
        )
        .unwrap();
        assert_eq!(v2.cooldown, Some(120));
        assert_eq!(v2.charges, Some(2));
        assert_eq!(v2.recharge, Some(Recharge::PerFloor));
    }
}
