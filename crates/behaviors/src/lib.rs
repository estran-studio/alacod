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
pub mod familiar;
pub use boss::{
    BossDef, BossDefeated, BossPhaseChanged, BossState, Phase, PhaseEnd, Timeline, TimelineEvent,
};
pub use familiar::FamiliarGun;

use bevy::prelude::{App, Component, Plugin};
use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use sim_core::{
    custom::CustomParams,
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
    /// M2-T8 (§49) : l'ennemi (membre d'une escouade, `ai.squad`) vise sa place sur une
    /// formation autour de la cible. Variante en fin d'enum : les discriminants existants ne
    /// bougent pas.
    Formation {
        shape: FormationShape,
        /// Tir alterné : `Shoot` ne part que dans la fenêtre de `period` frames du membre
        /// (`slot = (frame / period) mod effectif == rang`).
        period: u32,
        /// `true` : tir groupé, pas d'alternance.
        #[serde(default)]
        volley: bool,
    },
    /// M3-T0d (`docs/conventions.md` §63) : **familier** (personnage cité par un objet
    /// `familiar`). Suit son porteur à `gap` unités, en file derrière les autres familiers du
    /// même porteur (rang = ordre d'apparition, [`FamiliarOf::rank`]). Contrat seul : l'exécution
    /// vient de M3-T6 ; [`select`] l'ignore d'ici là. Variante en fin d'enum.
    Follow {
        gap: Fixed,
    },
    /// M3-T0d : familier **orbital**. Tourne autour du porteur à `radius` unités, un tour en
    /// `frames_per_turn` frames ; phase = rang × 2π / nombre d'orbitaux.
    Orbit {
        radius: Fixed,
        frames_per_turn: u32,
    },
    /// M3-T0d : tire quand le porteur tire, dans sa direction de visée, au plus une fois tous
    /// les `every` frames. Q13 (William, 2026-10-16) : le familier tire **les larmes du
    /// porteur** : son arme active composée avec ses objets et synergies (§66), ses stats de
    /// larmes, le crédit revient au porteur. Pas d'arme propre au familier.
    MirrorShot {
        every: u32,
    },
    /// M3-T0d : une balle hostile qui touche le familier est détruite (comme le blank) ; le
    /// familier est immunisé.
    BlockBullets,
    /// X-1 (`docs/conventions.md` §89) : behavior **propre à un jeu**, exécuté par son plugin
    /// (`id` déclaré dans `plugin.behaviors` de `game.ron`, `params` bruts décodés par le plugin).
    /// Sa règle est retenue quand le plugin le déclare applicable ([`select_with`]) ; son
    /// déplacement vient de lui (`game::extension::CustomBehaviorImpl`). Variante en fin d'enum.
    Custom {
        id: String,
        #[serde(default)]
        params: CustomParams,
    },
}

/// Familier lié à son porteur (M3-T0d, §63). Composant rollback **neutre** : sans porteur de ce
/// composant (aucun avant M3-T6), il ne contribue pas au checksum.
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FamiliarOf {
    /// `GgrsNetId` du porteur.
    pub owner: GgrsNetId,
    /// Rang d'apparition parmi les familiers du même porteur (0, 1, 2…).
    pub rank: u32,
}

impl Behavior {
    /// Le behavior n'a de sens que sur un familier (M3-T0d) : le lint exige un objet qui le cite.
    pub fn is_familiar(&self) -> bool {
        matches!(
            self,
            Behavior::Follow { .. }
                | Behavior::Orbit { .. }
                | Behavior::MirrorShot { .. }
                | Behavior::BlockBullets
        )
    }
}

/// Forme d'une formation (M2-T8).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum FormationShape {
    /// Places réparties sur un arc de cercle de rayon `radius` centré sur la cible, ouvert de
    /// `spread` radians (]0, 2π]) et orienté vers le centre de l'escouade.
    Arc { radius: Fixed, spread: Fixed },
}

impl Behavior {
    /// Nom de la variante (attente `EnemyState`, logs) ; l'`id` d'un `Custom`.
    pub fn name(&self) -> &str {
        match self {
            Behavior::Chase { .. } => "Chase",
            Behavior::KeepDistance { .. } => "KeepDistance",
            Behavior::Strafe => "Strafe",
            Behavior::Charge { .. } => "Charge",
            Behavior::Shoot { .. } => "Shoot",
            Behavior::Melee(_) => "Melee",
            Behavior::Flee => "Flee",
            Behavior::Wander => "Wander",
            Behavior::Formation { .. } => "Formation",
            Behavior::Follow { .. } => "Follow",
            Behavior::Orbit { .. } => "Orbit",
            Behavior::MirrorShot { .. } => "MirrorShot",
            Behavior::BlockBullets => "BlockBullets",
            Behavior::Custom { id, .. } => id,
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

/// Applicabilité implicite d'une règle (`index` dans la liste, pour l'hystérésis). Un `Custom` n'est
/// jamais applicable ici : voir [`applicable_with`].
pub fn applicable(index: usize, behavior: &Behavior, ctx: &SelectionContext) -> bool {
    applicable_with(index, behavior, ctx, &|_, _| false)
}

/// Décide si un behavior `Custom { id, params }` est applicable (fourni par le plugin du jeu).
pub type CustomApplicable<'a> = dyn Fn(&str, &CustomParams) -> bool + 'a;

/// Comme [`applicable`], un `Custom { id, params }` étant tranché par `custom` (X-1).
pub fn applicable_with(
    index: usize,
    behavior: &Behavior,
    ctx: &SelectionContext,
    custom: &CustomApplicable,
) -> bool {
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
        Behavior::Formation { .. } => distance.is_some_and(|d| d < ctx.sight),
        Behavior::Wander => true,
        // M3-T0d : comportements de familier, exécutés par M3-T6 ; jamais retenus par la sélection
        // des ennemis (un familier n'a pas d'IA de poursuite).
        Behavior::Follow { .. }
        | Behavior::Orbit { .. }
        | Behavior::MirrorShot { .. }
        | Behavior::BlockBullets => false,
        Behavior::Custom { id, params } => custom(id, params),
    }
}

/// Sélection par priorité : index de la première règle applicable, `None` si aucune.
pub fn select(rules: &[Behavior], ctx: &SelectionContext) -> Option<usize> {
    select_with(rules, ctx, &|_, _| false)
}

/// Comme [`select`], avec l'applicabilité des `Custom` fournie par le plugin du jeu.
pub fn select_with(
    rules: &[Behavior],
    ctx: &SelectionContext,
    custom: &CustomApplicable,
) -> Option<usize> {
    rules
        .iter()
        .enumerate()
        .find(|(index, behavior)| applicable_with(*index, behavior, ctx, custom))
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
                    "Formation",
                    "Follow",
                    "Orbit",
                    "MirrorShot",
                    "BlockBullets",
                ][..],
            ),
            ("perception", &["Sight", "Hearing"][..]),
            ("targeting", &["Nearest"][..]),
        ] {
            app.register_kinds(names.iter().map(|name| KindDecl::new(category, *name)));
        }
        app.rollback_and_trace::<BehaviorState>();
        // M3-T0d : neutre (aucun porteur avant M3-T6), les traces ne bougent pas.
        app.rollback_and_trace_neutral::<FamiliarOf>();
        // M3-T6 : idem (posé seulement sur un familier à `MirrorShot`).
        app.rollback_and_trace_neutral::<FamiliarGun>();
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
            Behavior::Formation {
                shape: FormationShape::Arc {
                    radius: Fixed::from_num(120),
                    spread: Fixed::from_num(2),
                },
                period: 30,
                volley: false,
            },
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

    /// M3-T0d (§63) : les behaviors de familier font l'aller-retour RON, portent leur nom, ne sont
    /// jamais retenus par la sélection (exécutés par M3-T6) et se reconnaissent comme tels.
    #[test]
    fn familiar_behaviors_are_contract_only() {
        let familiar = vec![
            Behavior::Follow {
                gap: Fixed::from_num(28),
            },
            Behavior::Orbit {
                radius: Fixed::from_num(40),
                frames_per_turn: 120,
            },
            Behavior::MirrorShot { every: 12 },
            Behavior::BlockBullets,
        ];
        let text = ron::to_string(&familiar).unwrap();
        assert_eq!(ron::from_str::<Vec<Behavior>>(&text).unwrap(), familiar);
        assert_eq!(
            familiar.iter().map(Behavior::name).collect::<Vec<_>>(),
            ["Follow", "Orbit", "MirrorShot", "BlockBullets"]
        );
        assert!(familiar.iter().all(Behavior::is_familiar));
        assert!(!chase().is_familiar());
        // Même avec une cible à portée et une règle de repli : la sélection saute le familier.
        let rules = [familiar, vec![chase()]].concat();
        assert_eq!(select(&rules, &ctx()), Some(4));
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
            "Formation",
            "Follow",
            "Orbit",
            "MirrorShot",
            "BlockBullets",
        ] {
            assert!(kinds.has("behavior", name));
        }
        for name in ["Sight", "Hearing"] {
            assert!(kinds.has("perception", name));
        }
        assert!(kinds.has("targeting", "Nearest"));
        assert_eq!(kinds.names("behavior").count(), 13);
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
