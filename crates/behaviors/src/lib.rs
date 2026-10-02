//! Contrat de vocabulaire d'IA (M1, chantier D1). Remplacera behavior.rs/pathing.rs
//! de game en T1.4 : aucun système de sélection, perception ou navigation ici encore.
//! Les futurs systèmes utiliseront RollbackSystemSet::EnemyAI, sans nouveau set.
//! Kinds : catégories snake_case, noms Rust exacts (PascalCase) ; références de contenu
//! à valider par T1.12. Les ids de profils, patterns et armes sont des chaînes de contenu,
//! pour ne dépendre ni de game ni du crate qui exécutera un pattern.
use bevy::prelude::{App, Component, Plugin};
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::{
    kinds::{KindDecl, KindRegistry},
    tag::Tag,
};
use utils::{net_id::GgrsNetId, rollback::RollbackTraceApp};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Behavior {
    Chase { profile: String },
    KeepDistance { min: Fixed, max: Fixed },
    Strafe,
    Charge { telegraph: u32 },
    Shoot(String),
    Melee(String),
    Flee,
    Wander,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Perception {
    Sight(Fixed),
    Hearing(Fixed),
}

/// Compose les sens ; needs_light concerne la vue, pas l'ouïe. Données seules en vague 0.
#[derive(Clone, Default, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct PerceptionConfig {
    pub senses: Vec<Perception>,
    #[serde(default)]
    pub needs_light: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum Targeting {
    Nearest { ignore: Vec<Tag> },
}

/// Indice de la règle sélectionnée (priorité = ordre RON), frame d'entrée et cible stable.
/// T1.4 définira la sélection ; aucun personnage n'en porte encore en vague 0.
#[derive(Component, Clone, Default, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct BehaviorState {
    pub selected_rule: Option<usize>,
    pub since_frame: u32,
    pub target: Option<GgrsNetId>,
}

pub struct BehaviorsPlugin;
impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        for (category, names) in [
            (
                "behavior",
                &[
                    "Chase",
                    "KeepDistance",
                    "Strafe",
                    "Charge",
                    "Shoot",
                    "Melee",
                    "Flee",
                    "Wander",
                ][..],
            ),
            ("perception", &["Sight", "Hearing"][..]),
            ("targeting", &["Nearest"][..]),
        ] {
            app.register_kinds(names.iter().map(|name| KindDecl::new(category, *name)));
        }
        app.rollback_and_trace::<BehaviorState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::kinds::Kinds;
    use utils::rollback::{StateTracers, TracedTypes};
    #[test]
    fn behavior_ron_round_trip() {
        let values = vec![
            Behavior::Chase {
                profile: "Ground".into(),
            },
            Behavior::KeepDistance {
                min: Fixed::from_num(100),
                max: Fixed::from_num(200),
            },
            Behavior::Strafe,
            Behavior::Charge { telegraph: 60 },
            Behavior::Shoot("salve".into()),
            Behavior::Melee("griffe".into()),
            Behavior::Flee,
            Behavior::Wander,
        ];
        assert_eq!(
            ron::from_str::<Vec<Behavior>>(&ron::to_string(&values).unwrap()).unwrap(),
            values
        );
    }
    #[test]
    fn perception_ron_round_trip() {
        let values = vec![
            Perception::Sight(Fixed::from_num(400)),
            Perception::Hearing(Fixed::from_num(300)),
        ];
        assert_eq!(
            ron::from_str::<Vec<Perception>>(&ron::to_string(&values).unwrap()).unwrap(),
            values
        );
    }
    #[test]
    fn targeting_ron_round_trip() {
        let value: Targeting = ron::from_str(r#"Nearest(ignore: ["disguised", "ghost"])"#).unwrap();
        assert_eq!(
            ron::from_str::<Targeting>(&ron::to_string(&value).unwrap()).unwrap(),
            value
        );
    }
    #[test]
    fn plugin_registers_kinds_and_rollback_state() {
        let mut app = App::new();
        app.add_plugins(BehaviorsPlugin);
        let kinds = app.world().resource::<Kinds>();
        for name in [
            "Chase",
            "KeepDistance",
            "Strafe",
            "Charge",
            "Shoot",
            "Melee",
            "Flee",
            "Wander",
        ] {
            assert!(kinds.has("behavior", name));
        }
        for name in ["Sight", "Hearing"] {
            assert!(kinds.has("perception", name));
        }
        assert!(kinds.has("targeting", "Nearest"));
        assert_eq!(kinds.names("behavior").count(), 8);
        assert_eq!(kinds.names("perception").count(), 2);
        assert_eq!(kinds.names("targeting").count(), 1);
        let name = std::any::type_name::<BehaviorState>();
        assert!(app.world().resource::<TracedTypes>().0.contains(&name));
        assert!(app
            .world()
            .resource::<StateTracers>()
            .components
            .iter()
            .any(|(n, _)| *n == name));
    }
}
