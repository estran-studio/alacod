//! Monster State Machine
//!
//! Generic state machine for enemy AI, replacing the hardcoded ZombieState.
//! Supports composable behaviors through configuration.

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use serde::{Deserialize, Serialize};
use sim_core::damage::FriendlyFire;
use std::collections::HashSet;
use utils::net_id::GgrsNetId;

use super::navigation::NavProfile;
use super::obstacle::ObstacleType;
use behaviors::{Behavior, Perception, PerceptionConfig, Targeting};

/// Generic monster state - replaces ZombieState
#[derive(
    Component, Clone, Debug, Hash, PartialEq, Eq, Reflect, Serialize, Deserialize, Default,
)]
pub enum MonsterState {
    /// Waiting or wandering randomly
    #[default]
    Idle,
    /// Following the flow field toward target
    Chasing,
    /// Attacking a target (player or obstacle)
    Attacking {
        target: AttackTarget,
        last_attack_frame: u32,
    },
    // T1.4 : `Stunned`, `Breaching`, `Fleeing` retirés — jamais posés (code mort). Aucun
    // effet sur les traces : le `derive(Hash)` hache l'index de variante, et ni `Dead` (seule
    // variante dont l'index change) ni les variantes retirées ne sont jamais posés. La fuite
    // est le behavior `Flee`, l'étourdissement un statut (T1.3).
    /// Dead but not yet despawned
    Dead,
}

/// Target for attacks
#[derive(Clone, Debug, Hash, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum AttackTarget {
    /// Attacking a player
    Player { net_id: GgrsNetId },
    /// Attacking an obstacle (window, barricade, etc.)
    Obstacle { net_id: GgrsNetId },
    /// Attacking at a position (for area attacks)
    Position { x: i32, y: i32 },
}

/// Movement type for enemies
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Reflect, Serialize, Deserialize)]
pub enum MovementType {
    /// Standard ground movement
    #[default]
    Ground,
    /// Flying - ignores water, pits
    Flying,
    /// Phasing - passes through most obstacles
    Phasing,
}

impl From<MovementType> for NavProfile {
    fn from(movement: MovementType) -> Self {
        match movement {
            MovementType::Ground => NavProfile::Ground,
            MovementType::Flying => NavProfile::Flying,
            MovementType::Phasing => NavProfile::Phasing,
        }
    }
}

/// AI configuration for an enemy - loaded from RON
///
/// `Hash` manuel (voir l'impl plus bas) : exclut `stationary` (T2.9). Motif identique à
/// `CharacterAppearance` (`character/visuals.rs`).
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct EnemyAiConfig {
    /// Movement type determines pathfinding behavior
    pub movement_type: MovementType,
    /// Range at which enemy becomes aggressive
    pub aggro_range: fixed_math::Fixed,
    /// Range at which enemy can attack
    pub attack_range: fixed_math::Fixed,
    /// Frames between attacks
    pub attack_cooldown_frames: u32,
    /// Obstacle types this enemy can break
    pub can_break: Vec<ObstacleType>,
    /// Obstacle types this enemy can attack through
    pub attack_through: Vec<ObstacleType>,
    /// Obstacle types this enemy ignores for pathfinding
    pub ignores: Vec<ObstacleType>,
    /// Whether to use GroundBreaker profile (path through breakables)
    pub path_through_breakables: bool,
    /// Optional flee threshold (0.0-1.0 health percentage)
    pub flee_threshold: Option<fixed_math::Fixed>,
    /// Damage dealt per attack
    pub attack_damage: fixed_math::Fixed,
    /// Politique de tir ami de l'attaque (T1.1, chantier B1). `Never` : un ennemi ne
    /// touche jamais un autre ennemi (même bord, voir `combat::team::team_allows_hit`).
    pub friendly_fire: FriendlyFire,
    /// Si vrai, ne bouge jamais (T2.9, testbed : `dummy`/`target`/`ally`/`civilian`) :
    /// `move_enemies` (`character::enemy::ai::pathing`) laisse cet ennemi immobile quelle
    /// que soit la flow field. `false` par défaut : aucun personnage zombie existant n'est
    /// stationnaire.
    pub stationary: bool,
    /// Tir à distance (T1.2, `docs/conventions.md` §20) : `None` (tout le contenu d'avant
    /// T1.2) = corps à corps seulement, comportement inchangé.
    #[serde(default)]
    pub ranged: Option<RangedAttack>,
}

/// Tir à distance d'un ennemi (T1.2) : à moins de `range` de sa cible `Player` et
/// refroidissement écoulé, l'ennemi pose un `combat::emitter::Emitter` jouant `pattern`
/// (pattern nommé, kind `Pattern`) avec les projectiles de `weapon` (équipée par
/// `spawn_enemy`), ne bouge plus pendant le télégraphe et le tir, puis attend
/// `cooldown_frames` après la fin (ou l'interruption) de la séquence.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangedAttack {
    pub weapon: String,
    pub pattern: String,
    pub range: fixed_math::Fixed,
    pub cooldown_frames: u32,
}

/// État rollback du tir à distance d'un ennemi (T1.2), posé par `spawn_enemy` sur les seuls
/// personnages à `ranged` (enregistré à checksum neutre : aucun ennemi existant n'en
/// porte). `target` : cible de la séquence en cours (`Some` tant qu'un émetteur tire pour
/// cet ennemi) ; `ready_at` : première frame où une nouvelle séquence peut partir.
#[derive(Component, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct RangedAttackState {
    pub target: Option<GgrsNetId>,
    pub ready_at: u32,
}

/// Hash manuel : hache exactement les champs présents avant T2.9, dans le même ordre que
/// l'ancien `#[derive(Hash)]` — `stationary` en est exclu. Ajouter un champ à une struct
/// hachée par un `derive` change toujours le hash produit, même à valeur « neutre »
/// (`false`) : sur du contenu zombie existant (aucun ne pose `stationary`), ça déplacerait
/// le checksum GGRS agrégé de toute entité `Enemy` (`EnemyAiConfig` y est déjà enregistré
/// via `rollback_and_trace`, avec checksum) et casserait les traces de référence
/// (`tests/scenarios/*.trace`) sans aucun changement de gameplay. Vérifié empiriquement :
/// sans cet impl manuel, `idle.ron` diverge dès l'apparition du premier zombie (f181).
/// Même motif que `CharacterAppearance` (`character/visuals.rs`), qui exclut
/// `health_bar_color` pour une raison différente (pas de `Hash`).
impl std::hash::Hash for EnemyAiConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.movement_type.hash(state);
        self.aggro_range.hash(state);
        self.attack_range.hash(state);
        self.attack_cooldown_frames.hash(state);
        self.can_break.hash(state);
        self.attack_through.hash(state);
        self.ignores.hash(state);
        self.path_through_breakables.hash(state);
        self.flee_threshold.hash(state);
        self.attack_damage.hash(state);
        self.friendly_fire.hash(state);
        // T1.2 : haché seulement s'il est présent — hacher un `None` ajouterait des octets
        // et déplacerait le checksum de tous les ennemis existants (aucun n'a `ranged`).
        if let Some(ranged) = &self.ranged {
            ranged.hash(state);
        }
    }
}

impl Default for EnemyAiConfig {
    fn default() -> Self {
        Self {
            movement_type: MovementType::Ground,
            aggro_range: fixed_math::new(300.0),
            attack_range: fixed_math::new(35.0),
            attack_cooldown_frames: 60,
            can_break: vec![ObstacleType::Window],
            attack_through: vec![ObstacleType::Window],
            ignores: vec![],
            path_through_breakables: false,
            flee_threshold: None,
            attack_damage: fixed_math::new(10.0),
            friendly_fire: FriendlyFire::Never,
            stationary: false,
            ranged: None,
        }
    }
}

impl EnemyAiConfig {
    /// Get the navigation profile for this enemy
    pub fn nav_profile(&self) -> NavProfile {
        if self.path_through_breakables {
            NavProfile::GroundBreaker
        } else {
            self.movement_type.into()
        }
    }

    /// Check if this enemy can break a given obstacle type
    pub fn can_break_obstacle(&self, obstacle_type: ObstacleType) -> bool {
        self.can_break.contains(&obstacle_type)
    }

    /// Check if this enemy can attack through a given obstacle type
    pub fn can_attack_through_obstacle(&self, obstacle_type: ObstacleType) -> bool {
        self.attack_through.contains(&obstacle_type)
    }

    /// Check if this enemy ignores a given obstacle type
    pub fn ignores_obstacle(&self, obstacle_type: ObstacleType) -> bool {
        self.ignores.contains(&obstacle_type)
    }

    /// Create a standard zombie configuration
    pub fn zombie() -> Self {
        Self {
            movement_type: MovementType::Ground,
            aggro_range: fixed_math::new(500.0),
            attack_range: fixed_math::new(40.0),
            attack_cooldown_frames: 60,
            can_break: vec![ObstacleType::Window, ObstacleType::Barricade],
            attack_through: vec![ObstacleType::Window],
            ignores: vec![],
            path_through_breakables: true, // Zombies path through breakables (windows)
            flee_threshold: None,
            attack_damage: fixed_math::new(10.0),
            friendly_fire: FriendlyFire::Never,
            stationary: false,
            ranged: None,
        }
    }

    /// Create a flying enemy configuration
    pub fn flying() -> Self {
        Self {
            movement_type: MovementType::Flying,
            aggro_range: fixed_math::new(400.0),
            attack_range: fixed_math::new(50.0),
            attack_cooldown_frames: 45,
            can_break: vec![],
            attack_through: vec![],
            ignores: vec![ObstacleType::Water, ObstacleType::Pit],
            path_through_breakables: false,
            flee_threshold: None,
            attack_damage: fixed_math::new(8.0),
            friendly_fire: FriendlyFire::Never,
            stationary: false,
            ranged: None,
        }
    }

    /// Create a ghost configuration
    pub fn ghost() -> Self {
        Self {
            movement_type: MovementType::Phasing,
            aggro_range: fixed_math::new(350.0),
            attack_range: fixed_math::new(30.0),
            attack_cooldown_frames: 90,
            can_break: vec![],
            attack_through: vec![
                ObstacleType::Window,
                ObstacleType::Barricade,
                ObstacleType::LowCover,
            ],
            ignores: vec![
                ObstacleType::Window,
                ObstacleType::Barricade,
                ObstacleType::Water,
                ObstacleType::Pit,
                ObstacleType::LowCover,
            ],
            path_through_breakables: false,
            flee_threshold: None,
            attack_damage: fixed_math::new(15.0),
            friendly_fire: FriendlyFire::Never,
            stationary: false,
            ranged: None,
        }
    }

    /// Create a tank zombie that smashes through obstacles
    pub fn tank() -> Self {
        Self {
            movement_type: MovementType::Ground,
            aggro_range: fixed_math::new(400.0),
            attack_range: fixed_math::new(50.0),
            attack_cooldown_frames: 90,
            can_break: vec![
                ObstacleType::Window,
                ObstacleType::Barricade,
                ObstacleType::LowCover,
            ],
            attack_through: vec![],
            ignores: vec![],
            path_through_breakables: true, // Uses GroundBreaker profile
            flee_threshold: None,
            attack_damage: fixed_math::new(25.0),
            friendly_fire: FriendlyFire::Never,
            stationary: false,
            ranged: None,
        }
    }
}

/// RON configuration for enemy AI (for data-driven setup)
#[derive(Clone, Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct EnemyAiConfigRon {
    pub movement_type: Option<MovementType>,
    pub aggro_range: Option<String>, // Fixed as string "300.0"
    pub attack_range: Option<String>,
    pub attack_cooldown_frames: Option<u32>,
    pub can_break: Option<Vec<ObstacleType>>,
    pub attack_through: Option<Vec<ObstacleType>>,
    pub ignores: Option<Vec<ObstacleType>>,
    pub path_through_breakables: Option<bool>,
    pub flee_threshold: Option<String>,
    pub attack_damage: Option<String>,
    /// Voir `EnemyAiConfig::stationary`. `None` = inchangé (`false`, comportement zombie
    /// existant).
    #[serde(default)]
    pub stationary: Option<bool>,
    /// T1.4 : règles de comportement, par priorité (ordre RON, `docs/conventions.md` §22).
    /// Absent : liste par défaut dérivée de la config ([`default_behaviors`]). Se compile
    /// vers l'état existant : `Shoot` remplit [`EnemyAiConfig::ranged`].
    #[serde(default)]
    pub behaviors: Option<Vec<Behavior>>,
    /// T1.4 : `Sight(r)` remplace `aggro_range` ; `Hearing(r)`, voir [`EnemyBehaviors`].
    #[serde(default)]
    pub perception: Option<PerceptionConfig>,
    /// T1.4 : repli `Nearest { ignore: [] }` (l'algorithme actuel).
    #[serde(default)]
    pub targeting: Option<Targeting>,
}

impl From<&EnemyAiConfigRon> for EnemyAiConfig {
    fn from(ron: &EnemyAiConfigRon) -> Self {
        let mut config = EnemyAiConfig::default();

        if let Some(movement) = ron.movement_type {
            config.movement_type = movement;
        }
        if let Some(ref range) = ron.aggro_range {
            if let Ok(val) = range.parse::<f32>() {
                config.aggro_range = fixed_math::new(val);
            } else {
                warn!("Failed to parse aggro_range '{}' from RON config.", range);
            }
        }
        if let Some(ref range) = ron.attack_range {
            if let Ok(val) = range.parse::<f32>() {
                config.attack_range = fixed_math::new(val);
            } else {
                warn!("Failed to parse attack_range '{}' from RON config.", range);
            }
        }
        if let Some(cooldown) = ron.attack_cooldown_frames {
            config.attack_cooldown_frames = cooldown;
        }
        if let Some(ref can_break) = ron.can_break {
            config.can_break = can_break.clone();
        }
        if let Some(ref attack_through) = ron.attack_through {
            config.attack_through = attack_through.clone();
        }
        if let Some(ref ignores) = ron.ignores {
            config.ignores = ignores.clone();
        }
        if let Some(path_through) = ron.path_through_breakables {
            config.path_through_breakables = path_through;
        }
        if let Some(ref threshold) = ron.flee_threshold {
            if let Ok(val) = threshold.parse::<f32>() {
                config.flee_threshold = Some(fixed_math::new(val));
            } else {
                warn!(
                    "Failed to parse flee_threshold '{}' from RON config.",
                    threshold
                );
            }
        }
        if let Some(ref damage) = ron.attack_damage {
            if let Ok(val) = damage.parse::<f32>() {
                config.attack_damage = fixed_math::new(val);
            } else {
                warn!(
                    "Failed to parse attack_damage '{}' from RON config.",
                    damage
                );
            }
        }
        if let Some(stationary) = ron.stationary {
            config.stationary = stationary;
        }
        // T1.4 : `Shoot` se compile vers `ranged` (la première règle `Shoot`, une seule en v1),
        // `Sight` vers `aggro_range` : l'état et son hash restent ceux de T1.2.
        if let Some(behaviors) = &ron.behaviors {
            config.ranged = behaviors.iter().find_map(|behavior| match behavior {
                Behavior::Shoot {
                    weapon,
                    pattern,
                    range,
                    cooldown_frames,
                } => Some(RangedAttack {
                    weapon: weapon.clone(),
                    pattern: pattern.clone(),
                    range: *range,
                    cooldown_frames: *cooldown_frames,
                }),
                _ => None,
            });
        }
        if let Some(perception) = &ron.perception {
            for sense in &perception.senses {
                if let Perception::Sight(radius) = sense {
                    config.aggro_range = *radius;
                }
            }
        }

        config
    }
}

/// T1.4 : liste par défaut d'un personnage sans `behaviors:` (`docs/conventions.md` §22),
/// dérivée de sa config : `[Flee]` si `flee_threshold`, puis `Shoot` si `ranged`, puis
/// `Melee("zombie_claws")` si `attack_range > 0`, puis `Chase` (profil = `movement_type`)
/// sauf si `stationary`. Reproduit exactement le comportement d'avant T1.4.
pub fn default_behaviors(config: &EnemyAiConfig) -> Vec<Behavior> {
    let mut rules = Vec::new();
    if config.flee_threshold.is_some() {
        rules.push(Behavior::Flee);
    }
    if let Some(ranged) = &config.ranged {
        rules.push(Behavior::Shoot {
            weapon: ranged.weapon.clone(),
            pattern: ranged.pattern.clone(),
            range: ranged.range,
            cooldown_frames: ranged.cooldown_frames,
        });
    }
    if config.attack_range > fixed_math::FIXED_ZERO {
        rules.push(Behavior::Melee(DEFAULT_MELEE_WEAPON.to_string()));
    }
    if !config.stationary {
        rules.push(Behavior::Chase {
            profile: movement_profile_name(config.movement_type).to_string(),
        });
    }
    rules
}

/// Arme de mêlée de la liste par défaut (avant T1.4 : codée en dur dans `spawn_enemy`).
pub const DEFAULT_MELEE_WEAPON: &str = "zombie_claws";

/// Nom du profil de navigation d'un `movement_type` (profil de `Chase`).
pub fn movement_profile_name(movement: MovementType) -> &'static str {
    match movement {
        MovementType::Ground => "Ground",
        MovementType::Flying => "Flying",
        MovementType::Phasing => "Phasing",
    }
}

/// Profils de `Chase` connus (lint, `NavProfile`).
pub const CHASE_PROFILES: &[&str] = &["Ground", "Flying", "Phasing", "GroundBreaker"];

/// T1.4 : règles d'un ennemi, résolues au spawn (liste du RON, sinon [`default_behaviors`]).
/// Composant **statique, hors rollback** (comme `Team`) : jamais modifié après le spawn,
/// absent du checksum — les ennemis existants ne portent aucun nouvel état rollback.
#[derive(Component, Clone, Debug, Default)]
pub struct EnemyBehaviors {
    pub rules: Vec<Behavior>,
    /// `Perception::Hearing(r)` : un tir de joueur né à moins de `r` rend le tireur connu.
    pub hearing: Option<fixed_math::Fixed>,
    /// `Targeting::Nearest { ignore }` : joueurs portant un de ces tags ignorés.
    pub ignore: Vec<sim_core::tag::Tag>,
}

impl EnemyBehaviors {
    pub fn from_config(ron: Option<&EnemyAiConfigRon>, config: &EnemyAiConfig) -> Self {
        let rules = ron
            .and_then(|ron| ron.behaviors.clone())
            .unwrap_or_else(|| default_behaviors(config));
        let hearing = ron
            .and_then(|ron| ron.perception.as_ref())
            .and_then(|perception| {
                perception.senses.iter().find_map(|sense| match sense {
                    Perception::Hearing(radius) => Some(*radius),
                    _ => None,
                })
            });
        let ignore = ron
            .and_then(|ron| ron.targeting.as_ref())
            .map(|Targeting::Nearest { ignore }| ignore.clone())
            .unwrap_or_default();
        Self {
            rules,
            hearing,
            ignore,
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.rules.iter().any(|rule| rule.name() == name)
    }

    /// Arme de la première règle `Melee`.
    pub fn melee_weapon(&self) -> Option<&str> {
        self.rules.iter().find_map(|rule| match rule {
            Behavior::Melee(weapon) => Some(weapon.as_str()),
            _ => None,
        })
    }

    /// Un behavior nouveau (T1.4) à état est listé : l'ennemi porte [`BehaviorRuntime`].
    pub fn needs_runtime(&self) -> bool {
        self.rules.iter().any(|rule| {
            matches!(
                rule,
                Behavior::KeepDistance { .. }
                    | Behavior::Strafe
                    | Behavior::Charge { .. }
                    | Behavior::Flee
                    | Behavior::Wander
            )
        })
    }
}

/// Phase d'une charge (`Behavior::Charge`).
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub enum ChargePhase {
    #[default]
    Idle,
    /// Immobile jusqu'à `until` (exclu), puis ruée vers `target` (position figée).
    Telegraph {
        until: u32,
        target: (fixed_math::Fixed, fixed_math::Fixed),
    },
    /// Ruée vers `target` jusqu'au contact ou `until`.
    Rush {
        until: u32,
        target: (fixed_math::Fixed, fixed_math::Fixed),
    },
}

/// T1.4 : état rollback des behaviors nouveaux (`KeepDistance`, `Strafe`, `Charge`, `Flee`,
/// `Wander`), posé **seulement** sur les ennemis qui en listent un
/// ([`EnemyBehaviors::needs_runtime`]) ; checksum neutre (aucun ennemi existant n'en porte).
#[derive(Component, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct BehaviorRuntime {
    /// Règle retenue (index dans [`EnemyBehaviors::rules`]).
    pub selected: Option<u32>,
    /// Frame d'entrée dans la règle retenue.
    pub since_frame: u32,
    pub charge: ChargePhase,
    /// Première frame où une nouvelle charge peut partir.
    pub charge_ready_at: u32,
    /// Frame du contact d'une ruée (dégât émis la frame suivante, `CollisionDamage`).
    pub charge_hit: Option<(u32, GgrsNetId)>,
    /// Direction d'errance (unitaire) et prochaine frame de tirage (flux `behaviors`).
    pub wander_dir: (fixed_math::Fixed, fixed_math::Fixed),
    pub wander_next: u32,
    /// Tireur entendu (`Perception::Hearing`) et frame d'oubli.
    pub heard: Option<(GgrsNetId, u32)>,
}

/// Current target information for an enemy
#[derive(Component, Clone, Debug, Hash, Default, Serialize, Deserialize)]
pub struct EnemyTarget {
    /// The current target entity (if any)
    pub target: Option<GgrsNetId>,
    /// Type of the current target
    pub target_type: TargetType,
    /// Last known position of target
    pub last_known_position: Option<fixed_math::FixedVec2>,
}

/// Type of target being pursued
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Reflect, Serialize, Deserialize)]
pub enum TargetType {
    #[default]
    None,
    Player,
    Obstacle,
}
