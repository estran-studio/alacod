//! Vocabulaire d'IA (M1, chantier D1) et **sélection par priorité** (T1.4, voir
//! `docs/conventions.md` §22). Ce crate porte les données (`Behavior`, `Perception`,
//! `Targeting`) et la règle de sélection, **pure** ([`select`]) : la liste d'un personnage est
//! ordonnée par priorité (ordre RON) et la première règle applicable gagne. Les faits d'une
//! frame ([`SelectionContext`]) et l'exécution des règles vivent dans `game`
//! (`character::enemy::ai`), dans `RollbackSystemSet::EnemyAI`.
//! Kinds : catégories snake_case, noms Rust exacts (PascalCase). Les ids de profils, patterns
//! et armes sont des chaînes de contenu, pour ne dépendre ni de game ni du crate qui exécutera
//! un pattern.
pub mod boss;
pub use boss::{BossDef, BossPhaseChanged, BossState, Phase, PhaseEnd, Timeline, TimelineEvent};

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
    Chase {
        profile: String,
    },
    KeepDistance {
        min: Fixed,
        max: Fixed,
    },
    Strafe,
    Charge {
        telegraph: u32,
    },
    /// T1.4 : remplace `ai.ranged` de T1.2 (le contrat T1.0a disait `Shoot(String)`).
    Shoot {
        weapon: String,
        pattern: String,
        range: Fixed,
        cooldown_frames: u32,
    },
    /// Arme de corps à corps (id de `melee_weapons.ron`), équipée au spawn.
    Melee(String),
    Flee,
    Wander,
}

impl Behavior {
    /// Nom de la variante (attente `EnemyState`, logs).
    pub fn name(&self) -> &'static str {
        match self {
            Behavior::Chase { .. } => "Chase",
            Behavior::KeepDistance { .. } => "KeepDistance",
            Behavior::Strafe => "Strafe",
            Behavior::Charge { .. } => "Charge",
            Behavior::Shoot { .. } => "Shoot",
            Behavior::Melee(_) => "Melee",
            Behavior::Flee => "Flee",
            Behavior::Wander => "Wander",
        }
    }
}

/// Faits d'une frame sur lesquels la sélection décide, calculés par `game` pour un ennemi.
/// Les distances sont en unités monde ; `target_distance` : `None` = aucune cible connue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionContext {
    pub target_distance: Option<Fixed>,
    /// Rayon de vue (`Perception::Sight`, repli `aggro_range`).
    pub sight: Fixed,
    /// Santé courante / santé max.
    pub health_ratio: Fixed,
    pub flee_threshold: Option<Fixed>,
    /// Portée de mêlée (`EnemyAiConfig::attack_range`) ; une cible (joueur, ou obstacle
    /// cassable sur la route) est à portée.
    pub attack_range: Fixed,
    pub melee_in_reach: bool,
    /// Une séquence de tir est en cours (émetteur posé) : `Shoot` reste retenu.
    pub shooting: bool,
    /// Tir possible : cible vivante, debout, refroidissement écoulé (la portée est vérifiée
    /// par la règle).
    pub shoot_ready: bool,
    /// Une charge est en cours (télégraphe ou ruée) : `Charge` reste retenu.
    pub charging: bool,
    /// Refroidissement de charge écoulé.
    pub charge_ready: bool,
    /// Règle retenue à la frame précédente (hystérésis de `KeepDistance`).
    pub previous: Option<usize>,
}

/// Facteur de la portée haute de `Charge` : la cible doit être entre `attack_range` et
/// `CHARGE_REACH × attack_range`.
pub const CHARGE_REACH: i32 = 3;

/// Applicabilité implicite d'une règle (`index` dans la liste, pour l'hystérésis).
pub fn applicable(index: usize, behavior: &Behavior, ctx: &SelectionContext) -> bool {
    let distance = ctx.target_distance;
    match behavior {
        Behavior::Melee(_) => ctx.melee_in_reach,
        Behavior::Shoot { range, .. } => {
            ctx.shooting || (ctx.shoot_ready && distance.is_some_and(|d| d < *range))
        }
        Behavior::Charge { .. } => {
            ctx.charging
                || (ctx.charge_ready
                    && distance.is_some_and(|d| {
                        d >= ctx.attack_range
                            && d <= ctx
                                .attack_range
                                .saturating_mul(Fixed::from_num(CHARGE_REACH))
                    }))
        }
        Behavior::KeepDistance { min, max } => {
            distance.is_some_and(|d| d < *min || (ctx.previous == Some(index) && d < *max))
        }
        Behavior::Flee => ctx
            .flee_threshold
            .is_some_and(|threshold| ctx.health_ratio <= threshold),
        Behavior::Strafe => distance.is_some_and(|d| d < ctx.sight),
        Behavior::Chase { .. } => distance.is_some(),
        Behavior::Wander => true,
    }
}

/// Sélection par priorité : index de la première règle applicable, `None` si aucune.
pub fn select(rules: &[Behavior], ctx: &SelectionContext) -> Option<usize> {
    rules
        .iter()
        .enumerate()
        .find(|(index, behavior)| applicable(*index, behavior, ctx))
        .map(|(index, _)| index)
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

/// Contrat de T1.0a. **Porté par personne en v1** (T1.4) : son enregistrement sous checksum
/// ordinaire est conservé tel quel — il compte dans la parité des types vides (CLAUDE.md,
/// checklist), le retirer, le passer en neutre ou le poser déplacerait toutes les traces.
/// L'état des behaviors vit dans `game` (`BehaviorRuntime`, checksum neutre).
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
            Behavior::Shoot {
                weapon: "fireball_gun".into(),
                pattern: "salve".into(),
                range: Fixed::from_num(260),
                cooldown_frames: 90,
            },
            Behavior::Melee("griffe".into()),
            Behavior::Flee,
            Behavior::Wander,
        ];
        assert_eq!(
            ron::from_str::<Vec<Behavior>>(&ron::to_string(&values).unwrap()).unwrap(),
            values
        );
    }
    fn ctx() -> SelectionContext {
        SelectionContext {
            target_distance: Some(Fixed::from_num(100)),
            sight: Fixed::from_num(300),
            health_ratio: Fixed::from_num(1),
            flee_threshold: None,
            attack_range: Fixed::from_num(40),
            melee_in_reach: false,
            shooting: false,
            shoot_ready: false,
            charging: false,
            charge_ready: true,
            previous: None,
        }
    }

    fn shoot() -> Behavior {
        Behavior::Shoot {
            weapon: "gun".into(),
            pattern: "p".into(),
            range: Fixed::from_num(200),
            cooldown_frames: 60,
        }
    }

    fn keep() -> Behavior {
        Behavior::KeepDistance {
            min: Fixed::from_num(120),
            max: Fixed::from_num(200),
        }
    }

    fn chase() -> Behavior {
        Behavior::Chase {
            profile: "GroundBreaker".into(),
        }
    }

    /// Table de cas : (règles, faits, règle attendue).
    #[test]
    fn selection_par_priorite() {
        let zombie = vec![Behavior::Melee("zombie_claws".into()), chase()];
        let kiter = vec![shoot(), keep(), chase()];
        let coward = vec![Behavior::Flee, Behavior::Melee("c".into()), chase()];
        let drifter = vec![Behavior::Strafe, Behavior::Wander];
        let charger = vec![Behavior::Charge { telegraph: 30 }, chase()];
        let cases: Vec<(&str, &Vec<Behavior>, SelectionContext, Option<usize>)> = vec![
            ("zombie loin : Chase", &zombie, ctx(), Some(1)),
            (
                "zombie au contact : Melee",
                &zombie,
                SelectionContext {
                    melee_in_reach: true,
                    ..ctx()
                },
                Some(0),
            ),
            (
                "zombie sans cible : rien",
                &zombie,
                SelectionContext {
                    target_distance: None,
                    ..ctx()
                },
                None,
            ),
            (
                "kiter trop près, pas de tir : KeepDistance",
                &kiter,
                ctx(),
                Some(1),
            ),
            (
                "kiter prêt à tirer : Shoot",
                &kiter,
                SelectionContext {
                    shoot_ready: true,
                    ..ctx()
                },
                Some(0),
            ),
            (
                "kiter en séquence, cible hors portée : Shoot reste",
                &kiter,
                SelectionContext {
                    shooting: true,
                    target_distance: Some(Fixed::from_num(500)),
                    ..ctx()
                },
                Some(0),
            ),
            (
                "kiter dans la bande, sans hystérésis : Chase",
                &kiter,
                SelectionContext {
                    target_distance: Some(Fixed::from_num(150)),
                    ..ctx()
                },
                Some(2),
            ),
            (
                "kiter dans la bande, recule déjà : KeepDistance",
                &kiter,
                SelectionContext {
                    target_distance: Some(Fixed::from_num(150)),
                    previous: Some(1),
                    ..ctx()
                },
                Some(1),
            ),
            (
                "kiter au-delà de max : Chase",
                &kiter,
                SelectionContext {
                    target_distance: Some(Fixed::from_num(250)),
                    previous: Some(1),
                    ..ctx()
                },
                Some(2),
            ),
            (
                "lâche blessé : Flee",
                &coward,
                SelectionContext {
                    flee_threshold: Some(Fixed::from_num(0.5)),
                    health_ratio: Fixed::from_num(0.4),
                    melee_in_reach: true,
                    ..ctx()
                },
                Some(0),
            ),
            (
                "lâche en forme : Melee",
                &coward,
                SelectionContext {
                    flee_threshold: Some(Fixed::from_num(0.5)),
                    melee_in_reach: true,
                    ..ctx()
                },
                Some(1),
            ),
            ("drifter qui voit : Strafe", &drifter, ctx(), Some(0)),
            (
                "drifter sans cible : Wander",
                &drifter,
                SelectionContext {
                    target_distance: None,
                    ..ctx()
                },
                Some(1),
            ),
            ("charger à 100 (40..120) : Charge", &charger, ctx(), Some(0)),
            (
                "charger à 150 : Chase",
                &charger,
                SelectionContext {
                    target_distance: Some(Fixed::from_num(150)),
                    ..ctx()
                },
                Some(1),
            ),
            (
                "charger en refroidissement : Chase",
                &charger,
                SelectionContext {
                    charge_ready: false,
                    ..ctx()
                },
                Some(1),
            ),
            (
                "charger en ruée, cible loin : Charge reste",
                &charger,
                SelectionContext {
                    charging: true,
                    target_distance: Some(Fixed::from_num(400)),
                    ..ctx()
                },
                Some(0),
            ),
        ];
        for (name, rules, ctx, expected) in cases {
            assert_eq!(select(rules, &ctx), expected, "{name}");
        }
    }

    #[test]
    fn noms_de_variantes() {
        assert_eq!(shoot().name(), "Shoot");
        assert_eq!(keep().name(), "KeepDistance");
        assert_eq!(Behavior::Wander.name(), "Wander");
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
