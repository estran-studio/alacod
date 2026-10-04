//! Projectiles composables (B5 v1, T1.1) : les contrats de T1.0a (`ProjectileModifier`,
//! `Pattern`) et leur exécution sur le cycle de vie d'une balle.
//!
//! # Données (RON)
//!
//! Un mode de tir (`weapons::FiringModeConfig::projectile`) porte un [`ProjectileSpec`] :
//! modificateurs, `on_hit: [Action]` et `on_expire: [Spawn(pattern)]`. Les projectiles
//! nommés dans un pattern (`projectile: "eclat"`) sont décrits dans la table `projectiles`
//! de **la même arme** (`weapons::WeaponConfig::projectiles`, [`ProjectileDef`]) : une
//! table par arme en v1, résolue au tir et portée par la balle (voir [`Projectile::table`]).
//! Une explosion est un projectile de vitesse nulle, `Lifetime(0)`, grand `Size` et `Pierce`
//! élevé : il vit une seule frame de collisions et touche tout ce qui est dans son rayon.
//!
//! # Cycle de vie (ordre dans une frame)
//!
//! 1. `RollbackSystemSet::Weapon` : tir (`weapons::weapon_rollback_system`, la balle reçoit
//!    [`Projectile`] si le mode de tir déclare un `projectile` non vide), déplacement
//!    (`weapons::bullet_rollback_system` : la portée atteinte termine le projectile au lieu
//!    de le détruire), collisions des balles **sans** [`Projectile`] (inchangées).
//! 2. `RollbackSystemSet::Projectiles` : [`projectile_collision_system`] (personnages puis
//!    murs : `Pierce`, `Bounce`, `on_hit`), [`apply_projectile_on_hit_system`] (modificateurs
//!    de `on_hit` posés sur la cible touchée), [`projectile_expire_system`] (`Lifetime`, puis
//!    `on_expire` et destruction de tout projectile terminé), [`projectile_steering_system`]
//!    (`Gravity` et `Homing` modifient la vitesse appliquée à la frame suivante).
//!
//! `on_expire` se déclenche à **toute** fin du projectile : durée de vie, portée, mur sans
//! rebond restant, dernier personnage traversé. Une grenade qui touche un ennemi explose.
//!
//! Les balles sans `projectile:` (tout le contenu d'avant T1.1) ne portent pas
//! [`Projectile`] et suivent exactement le chemin d'avant.
use std::collections::BTreeMap;
use std::sync::Arc;

use bevy::{
    log::{tracing::span, Level},
    prelude::*,
};
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2, FixedVec3};
use bevy_ggrs::{Rollback, RollbackDespawnCommandExtension};
use effects::Action;
use serde::{Deserialize, Serialize};
use sim_core::{
    damage::{DamageEvent, DamageKind},
    frame_events::FrameEvents,
    modifier::{ModifierSource, Modifiers},
    team::Team,
};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter, order_mut_iter,
};

use crate::{
    actors::Health,
    collider::{is_colliding, Collider, CollisionLayer, CollisionSettings, Wall},
    collision_grid::{collider_aabb, union_aabb, CollisionGrids},
    team::team_allows_hit,
    weapons::{Bullet, BulletType},
};

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
    // T1.2 : variantes ajoutées **en fin d'enum** — le `derive(Hash)` hache l'index de la
    // variante, les variantes existantes gardent le leur (hash des armes et projectiles
    // existants inchangé).
    /// `count` tirs à des angles tirés au hasard dans `±spread/2` autour de la visée (flux
    /// RNG `"patterns"`, émetteurs seulement : refusé en `on_expire` par le lint).
    Scatter {
        count: u32,
        spread: Fixed,
        projectile: String,
    },
    /// Pattern nommé du contenu (`patterns/<nom>.ron`, kind `Pattern`), résolu par
    /// [`PatternLibrary`] (nom inconnu : erreur de lint, jamais de panique en jeu).
    Named(String),
}

impl Pattern {
    /// Vrai si le pattern (ou un de ses enfants) contient une étape temporelle ou aléatoire
    /// (`Telegraph`, `Wait`, `Scatter`, `Ring.every > 0` n'en est pas une : ignoré en
    /// `on_expire`). `Named` n'est pas résolu ici.
    pub fn has_emitter_only_step(&self) -> bool {
        match self {
            Pattern::Telegraph(_) | Pattern::Wait(_) | Pattern::Scatter { .. } => true,
            Pattern::Sequence(children) => children.iter().any(Pattern::has_emitter_only_step),
            _ => false,
        }
    }
}

/// Patterns nommés du jeu (kind de contenu `Pattern`, T1.2) : nom -> pattern. Ressource
/// **hors rollback** (comme `Assets`) : remplie par `game` depuis le registre de contenu,
/// identique sur tous les clients. Résout `Pattern::Named` au départ d'un émetteur et à
/// l'expiration d'un projectile (`on_expire: [Spawn(Named("..."))]`).
#[derive(Resource, Clone, Debug, Default)]
pub struct PatternLibrary {
    pub patterns: BTreeMap<String, Arc<Pattern>>,
}

/// Résout `pattern` par la bibliothèque (s'il y en a une) : un `Named` sans bibliothèque est
/// une erreur (le lint garantit que tout nom référencé existe). Utilisé à l'expiration d'un
/// projectile et au départ d'un émetteur (T1.2).
pub fn resolve_pattern(
    library: Option<&PatternLibrary>,
    pattern: &Pattern,
) -> Result<Pattern, String> {
    match library {
        Some(library) => library.resolve(pattern),
        None if matches!(pattern, Pattern::Named(_)) => Err(format!("{pattern:?}")),
        None => Ok(pattern.clone()),
    }
}

/// Profondeur maximale de `Named` imbriqués (le lint refuse les cycles).
pub const MAX_NAMED_DEPTH: u8 = 8;

impl PatternLibrary {
    /// Remplace récursivement chaque `Named` par sa définition. `Err(nom)` : nom inconnu
    /// ou chaîne de `Named` trop profonde (cycle) — refusés par le lint.
    pub fn resolve(&self, pattern: &Pattern) -> Result<Pattern, String> {
        self.resolve_depth(pattern, 0)
    }

    fn resolve_depth(&self, pattern: &Pattern, depth: u8) -> Result<Pattern, String> {
        match pattern {
            Pattern::Named(name) => {
                if depth >= MAX_NAMED_DEPTH {
                    return Err(name.clone());
                }
                let target = self.patterns.get(name).ok_or_else(|| name.clone())?;
                self.resolve_depth(target, depth + 1)
            }
            Pattern::Sequence(children) => children
                .iter()
                .map(|child| self.resolve_depth(child, depth))
                .collect::<Result<Vec<_>, _>>()
                .map(Pattern::Sequence),
            other => Ok(other.clone()),
        }
    }
}

/// Action déclenchée à la fin d'un projectile (`on_expire`). Une seule en v1.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum ExpireAction {
    /// Fait naître le pattern au point de fin du projectile. Seuls les patterns instantanés
    /// sont joués (`Aimed`, `Spread`, `Ring` une seule fois, `Sequence` de ceux-ci) ;
    /// `Telegraph`/`Wait` appartiennent aux émetteurs (T1.2) et sont refusés par le lint.
    Spawn(Pattern),
    /// Creuse le terrain d'une caverne au point de fin (T1.6, `effects::Action::DestroyTerrain`
    /// : `Rock` à moins de `radius` → `Floor`). Ajoutée en dernier : le hash des contenus
    /// existants ne bouge pas.
    DestroyTerrain { radius: Fixed },
}

/// Comportement composable d'un projectile, tel que déclaré dans un mode de tir. Vide (tous
/// les champs absents) : balle ordinaire, sans [`Projectile`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectileSpec {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ProjectileModifier>,
    /// Actions posées sur chaque personnage touché (`effects::Action` à modificateur :
    /// `TimedModifier`, `CurrencyMultiplier` ; les autres sont refusées par le lint).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_hit: Vec<Action>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_expire: Vec<ExpireAction>,
}

impl ProjectileSpec {
    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty() && self.on_hit.is_empty() && self.on_expire.is_empty()
    }
}

/// Projectile nommé de la table `projectiles` d'une arme, que `on_expire` fait naître.
/// `speed` en unités/seconde (remplacée par celle d'un `Ring`), `range` en unités (> 0).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectileDef {
    pub damage: Fixed,
    pub speed: Fixed,
    pub range: Fixed,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ProjectileModifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_hit: Vec<Action>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_expire: Vec<ExpireAction>,
}

impl ProjectileDef {
    pub fn spec(&self) -> ProjectileSpec {
        ProjectileSpec {
            modifiers: self.modifiers.clone(),
            on_hit: self.on_hit.clone(),
            on_expire: self.on_expire.clone(),
        }
    }
}

/// Table `projectiles` d'une arme (id -> définition), ordonnée.
pub type ProjectileTable = BTreeMap<String, ProjectileDef>;

/// Rayon de collision d'une balle avant `Size` (celui de `BulletType::Standard`).
pub const BASE_RADIUS: f32 = 5.0;
/// Côté du sprite d'une balle avant `Size` (celui de `weapons::spawn_bullet_rollback`).
pub const BASE_SPRITE: f32 = 3.5;
/// Garde-fou contre une chaîne `on_expire` sans fin (le lint refuse déjà les cycles).
pub const MAX_GENERATION: u8 = 8;

/// Valeurs résolues d'une liste de modificateurs. Un même modificateur répété : le dernier
/// l'emporte (le lint refuse les doublons).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedModifiers {
    pub bounces: u32,
    pub pierces: u32,
    pub size: Fixed,
    pub lifetime: Option<u32>,
    pub homing: Option<Fixed>,
    pub gravity: Option<Fixed>,
}

pub fn resolve_modifiers(modifiers: &[ProjectileModifier]) -> ResolvedModifiers {
    let mut resolved = ResolvedModifiers {
        bounces: 0,
        pierces: 0,
        size: fixed_math::FIXED_ONE,
        lifetime: None,
        homing: None,
        gravity: None,
    };
    for modifier in modifiers {
        match modifier {
            ProjectileModifier::Bounce(n) => resolved.bounces = *n,
            ProjectileModifier::Pierce(n) => resolved.pierces = *n,
            ProjectileModifier::Size(factor) => resolved.size = *factor,
            ProjectileModifier::Lifetime(frames) => resolved.lifetime = Some(*frames),
            ProjectileModifier::Homing(force) => resolved.homing = Some(*force),
            ProjectileModifier::Gravity(accel) => resolved.gravity = Some(*accel),
        }
    }
    resolved
}

/// État rollback d'un projectile composable, posé à côté de `weapons::Bullet`.
#[derive(Component, Clone, Hash)]
pub struct Projectile {
    /// Id de contenu : l'arme pour un projectile tiré, l'entrée de `projectiles` pour un
    /// projectile né d'un `on_expire` (filtre de l'attente `BulletCount`).
    pub id: String,
    pub bounces_left: u32,
    pub pierces_left: u32,
    pub size: Fixed,
    /// Frames restantes ; `Some(0)` : se termine à la fin de cette frame.
    pub lifetime_left: Option<u32>,
    pub homing: Option<Fixed>,
    /// Accélération verticale en unités/seconde² (positive vers le haut du monde).
    pub gravity: Option<Fixed>,
    /// Personnages déjà touchés (triés par `GgrsNetId`) : un projectile perforant ne touche
    /// chaque cible qu'une fois.
    pub hits: Vec<GgrsNetId>,
    /// Terminé cette frame (portée, mur, perforation épuisée, durée de vie) : détruit par
    /// [`projectile_expire_system`] après `on_expire`.
    pub ended: bool,
    /// 0 pour un projectile tiré, +1 par `on_expire` (voir [`MAX_GENERATION`]).
    pub generation: u8,
    /// Multiplicateur de dégâts du tireur au tir (stat `Damage`), transmis aux projectiles
    /// nés de celui-ci.
    pub damage_mult: Fixed,
    pub on_hit: Vec<Action>,
    pub on_expire: Vec<ExpireAction>,
    /// Table `projectiles` de l'arme d'origine, partagée par toute la descendance.
    pub table: Arc<ProjectileTable>,
}

/// Debug compact : la table n'est donnée que par ses clés (elle est identique pour toute la
/// descendance d'un tir, la répéter à chaque ligne de la trace détaillée n'apprend rien).
impl std::fmt::Debug for Projectile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Projectile")
            .field("id", &self.id)
            .field("bounces_left", &self.bounces_left)
            .field("pierces_left", &self.pierces_left)
            .field("size", &self.size)
            .field("lifetime_left", &self.lifetime_left)
            .field("homing", &self.homing)
            .field("gravity", &self.gravity)
            .field("hits", &self.hits)
            .field("ended", &self.ended)
            .field("generation", &self.generation)
            .field("damage_mult", &self.damage_mult)
            .field("on_hit", &self.on_hit)
            .field("on_expire", &self.on_expire)
            .field("table", &self.table.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Projectile {
    pub fn new(
        id: String,
        spec: &ProjectileSpec,
        table: Arc<ProjectileTable>,
        damage_mult: Fixed,
        generation: u8,
    ) -> Self {
        let resolved = resolve_modifiers(&spec.modifiers);
        Self {
            id,
            bounces_left: resolved.bounces,
            pierces_left: resolved.pierces,
            size: resolved.size,
            lifetime_left: resolved.lifetime,
            homing: resolved.homing,
            gravity: resolved.gravity,
            hits: Vec::new(),
            ended: false,
            generation,
            damage_mult,
            on_hit: spec.on_hit.clone(),
            on_expire: spec.on_expire.clone(),
            table,
        }
    }

    /// Rayon de collision : `base` × `Size`.
    pub fn radius(&self, base: Fixed) -> Fixed {
        base.saturating_mul(self.size)
    }

    /// Ce personnage peut-il encore être touché (pas déjà traversé) ?
    pub fn can_hit(&self, target: &GgrsNetId) -> bool {
        !self.hits.iter().any(|hit| hit.0 == target.0)
    }

    /// Enregistre un coup sur `target` (`Pierce`) : le projectile continue tant qu'il lui
    /// reste des perforations, sinon il est terminé. Retourne `true` s'il est terminé.
    pub fn register_hit(&mut self, target: GgrsNetId) -> bool {
        let at = self.hits.partition_point(|hit| hit.0 < target.0);
        self.hits.insert(at, target);
        if self.pierces_left == 0 {
            self.ended = true;
        } else {
            self.pierces_left -= 1;
        }
        self.ended
    }

    /// Contact avec un mur : rebondit s'il reste des rebonds (`true`), sinon se termine.
    pub fn register_wall(&mut self) -> bool {
        if self.bounces_left == 0 {
            self.ended = true;
            false
        } else {
            self.bounces_left -= 1;
            true
        }
    }

    /// Fin de frame (`Lifetime`) : `Some(0)` termine le projectile, sinon décompte.
    pub fn tick_lifetime(&mut self) {
        match self.lifetime_left {
            Some(0) => self.ended = true,
            Some(left) => self.lifetime_left = Some(left - 1),
            None => {}
        }
    }
}

/// Coup porté par un projectile composable, émis pour [`apply_projectile_on_hit_system`].
#[derive(Clone, Debug, Hash)]
pub struct ProjectileHit {
    pub projectile: String,
    pub source: GgrsNetId,
    pub target: GgrsNetId,
    pub actions: Vec<Action>,
    pub frame: u32,
}

/// Mur touché par un projectile composable (T1.6) : émis dans la branche `register_wall()` de
/// [`projectile_collision_system`], seulement pour un projectile qui porte des actions
/// `on_hit` (les autres n'ont rien à y appliquer ; file neutre : vide, elle laisse les traces
/// existantes intactes). Position = position du projectile au contact (dans le mur).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProjectileWallHit {
    pub projectile: String,
    pub source: GgrsNetId,
    pub x: Fixed,
    pub y: Fixed,
    pub actions: Vec<Action>,
    pub frame: u32,
}

/// Actions de terrain des projectiles (T1.6) : chaque `DestroyTerrain` d'un mur touché
/// (`on_hit`) devient une `world::DestroyTerrainRequest` au point d'impact, appliquée ensuite
/// dans `RollbackSystemSet::World`. Les `DestroyTerrain` d'`on_expire` sont émises par
/// [`projectile_expire_system`].
pub fn projectile_wall_terrain_system(
    wall_hits: Res<FrameEvents<ProjectileWallHit>>,
    requests: Option<ResMut<FrameEvents<world::DestroyTerrainRequest>>>,
) {
    let Some(mut requests) = requests else {
        return;
    };
    for hit in wall_hits.iter() {
        for action in &hit.actions {
            if let Action::DestroyTerrain { radius } = action {
                requests.send(world::DestroyTerrainRequest::new(
                    FixedVec2::new(hit.x, hit.y),
                    *radius,
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Mathématiques pures (testées unitairement)
// ---------------------------------------------------------------------------------------

/// `Gravity(g)` sur une frame (60 par seconde) : `v.y += g / 3600`.
pub fn apply_gravity(velocity: FixedVec2, gravity: Fixed) -> FixedVec2 {
    let per_frame = gravity / Fixed::from_num(3600);
    FixedVec2::new(velocity.x, velocity.y.saturating_add(per_frame))
}

/// `Homing(force)` sur une frame : la direction tourne vers `to_target`
/// (`v̂ + force·t̂`, renormalisée), la vitesse scalaire est conservée.
pub fn steer_towards(velocity: FixedVec2, to_target: FixedVec2, force: Fixed) -> FixedVec2 {
    let speed = velocity.length();
    let target_dir = to_target.normalize_or_zero();
    if speed == Fixed::ZERO || target_dir == FixedVec2::ZERO {
        return velocity;
    }
    let dir = velocity.normalize_or_zero();
    let blended = FixedVec2::new(
        dir.x.saturating_add(target_dir.x.saturating_mul(force)),
        dir.y.saturating_add(target_dir.y.saturating_mul(force)),
    )
    .normalize_or_zero();
    if blended == FixedVec2::ZERO {
        return velocity;
    }
    FixedVec2::new(
        blended.x.saturating_mul(speed),
        blended.y.saturating_mul(speed),
    )
}

/// Rebond sur un mur : `flip_x`/`flip_y` disent quel axe du déplacement entrait dans le mur
/// (voir [`bounce_axes`]).
pub fn reflect(velocity: FixedVec2, flip_x: bool, flip_y: bool) -> FixedVec2 {
    FixedVec2::new(
        if flip_x { -velocity.x } else { velocity.x },
        if flip_y { -velocity.y } else { velocity.y },
    )
}

/// Axes à inverser pour un rebond : on rejoue le déplacement de la frame axe par axe depuis
/// la position d'avant (`old`). L'axe dont le seul déplacement entre en collision s'inverse ;
/// un coin (aucun des deux seuls) inverse les deux.
pub fn bounce_axes(
    old: FixedVec3,
    velocity: FixedVec2,
    collides_at: impl Fn(&FixedVec3) -> bool,
) -> (bool, bool) {
    let x_only = FixedVec3::new(old.x + velocity.x, old.y, old.z);
    let y_only = FixedVec3::new(old.x, old.y + velocity.y, old.z);
    let flip_x = collides_at(&x_only);
    let flip_y = collides_at(&y_only);
    if !flip_x && !flip_y {
        (true, true)
    } else {
        (flip_x, flip_y)
    }
}

/// Un projectile à faire naître par un pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shot {
    pub projectile: String,
    /// Direction unitaire.
    pub direction: FixedVec2,
    /// Vitesse imposée par le pattern (`Ring`), sinon celle de la définition.
    pub speed: Option<Fixed>,
}

/// Angles d'un éventail de `count` projectiles centré sur `center`, sur `spread` radians
/// au total (bornes comprises), un seul : `center`.
pub(crate) fn fan_angles(center: Fixed, count: u32, spread: Fixed) -> Vec<Fixed> {
    if count <= 1 {
        return vec![center; count as usize];
    }
    let half = spread / Fixed::from_num(2);
    let step = spread / Fixed::from_num(count - 1);
    (0..count)
        .map(|i| center - half + step.saturating_mul(Fixed::from_num(i)))
        .collect()
}

/// Direction unitaire d'un angle. Un angle au-delà de `±2π` est d'abord ramené dans
/// `[-2π, 2π]` (T1.2) : le CORDIC de `fixed_math` perd sa précision puis **déborde** (panique
/// `fixed`) au-delà (ex. 7,07 rad : le 8e rayon d'une couronne dont le premier part à π/2,
/// cas d'une tourelle qui vise vers le haut). Les angles déjà dans `[-2π, 2π]` — tout le
/// contenu d'avant T1.2 : couronnes partant de l'axe +x, au plus 7τ/8 — passent tels quels,
/// valeurs inchangées au bit près.
pub(crate) fn direction_of(angle: Fixed) -> FixedVec2 {
    let mut angle = angle;
    while angle > fixed_math::FIXED_TAU {
        angle -= fixed_math::FIXED_TAU;
    }
    while angle < -fixed_math::FIXED_TAU {
        angle += fixed_math::FIXED_TAU;
    }
    FixedVec2::new(fixed_math::cos_fixed(angle), fixed_math::sin_fixed(angle))
}

pub(crate) fn angle_of(direction: FixedVec2) -> Fixed {
    // Normalisée d'abord : `atan2_fixed` déborde sur de grandes composantes (une distance
    // à la cible en unités monde).
    let direction = direction.normalize_or_zero();
    if direction == FixedVec2::ZERO {
        Fixed::ZERO
    } else {
        fixed_math::atan2_fixed(direction.y, direction.x)
    }
}

/// Projectiles d'un pattern instantané joué au point de fin d'un projectile. `forward` :
/// direction du projectile qui se termine (ou zéro : axe +x) ; `aim` : direction de la cible
/// la plus proche pour `Aimed` (sinon `forward`). Pure et sans aléa : un même pattern donne
/// toujours les mêmes tirs.
pub fn pattern_shots(pattern: &Pattern, forward: FixedVec2, aim: Option<FixedVec2>) -> Vec<Shot> {
    let mut shots = Vec::new();
    collect_shots(pattern, forward, aim, &mut shots);
    shots
}

fn collect_shots(
    pattern: &Pattern,
    forward: FixedVec2,
    aim: Option<FixedVec2>,
    shots: &mut Vec<Shot>,
) {
    match pattern {
        Pattern::Aimed {
            count,
            spread,
            projectile,
        } => {
            let center = angle_of(aim.unwrap_or(forward));
            for angle in fan_angles(center, *count, *spread) {
                shots.push(Shot {
                    projectile: projectile.clone(),
                    direction: direction_of(angle),
                    speed: None,
                });
            }
        }
        Pattern::Spread {
            count,
            spread,
            projectile,
        } => {
            for angle in fan_angles(angle_of(forward), *count, *spread) {
                shots.push(Shot {
                    projectile: projectile.clone(),
                    direction: direction_of(angle),
                    speed: None,
                });
            }
        }
        Pattern::Ring {
            count,
            speed,
            projectile,
            ..
        } => {
            let start = angle_of(forward);
            let step = if *count == 0 {
                Fixed::ZERO
            } else {
                fixed_math::FIXED_TAU / Fixed::from_num(*count)
            };
            for i in 0..*count {
                shots.push(Shot {
                    projectile: projectile.clone(),
                    direction: direction_of(start + step.saturating_mul(Fixed::from_num(i))),
                    speed: Some(*speed),
                });
            }
        }
        Pattern::Sequence(patterns) => {
            for child in patterns {
                collect_shots(child, forward, aim, shots);
            }
        }
        // Temporels et aléatoires : réservés aux émetteurs (T1.2, `crate::emitter`),
        // refusés en `on_expire` par le lint. `Named` est résolu avant (`PatternLibrary`).
        Pattern::Telegraph(_) | Pattern::Wait(_) | Pattern::Scatter { .. } | Pattern::Named(_) => {}
    }
}

/// Cible de `Homing` et de `Aimed` : le personnage le plus proche que l'équipe et la
/// politique de tir ami du projectile permettent de toucher, hors `Team::Neutral` et hors
/// cibles déjà traversées ; égalité de distance départagée par `GgrsNetId`.
fn nearest_target<'a>(
    position: &FixedVec3,
    bullet: &Bullet,
    projectile: &Projectile,
    targets: impl Iterator<Item = (&'a GgrsNetId, &'a fixed_math::FixedTransform3D, &'a Team)>,
) -> Option<FixedVec2> {
    let mut best: Option<(fixed_math::FixedWide, usize, FixedVec2)> = None;
    for (net_id, transform, team) in targets {
        if *team == Team::Neutral
            || !team_allows_hit(
                bullet.source_team,
                *team,
                bullet.friendly_fire,
                &bullet.tags,
            )
            || !projectile.can_hit(net_id)
        {
            continue;
        }
        let delta = FixedVec2::new(
            transform.translation.x - position.x,
            transform.translation.y - position.y,
        );
        let distance = delta.length_squared();
        let better = match &best {
            None => true,
            Some((best_distance, best_id, _)) => {
                distance < *best_distance || (distance == *best_distance && net_id.0 < *best_id)
            }
        };
        if better {
            best = Some((distance, net_id.0, delta));
        }
    }
    best.map(|(_, _, delta)| delta)
}

// ---------------------------------------------------------------------------------------
// Naissance
// ---------------------------------------------------------------------------------------

/// Paramètres communs d'un projectile né d'un autre (`on_expire`).
pub struct Parent<'a> {
    pub bullet: &'a Bullet,
    pub projectile: &'a Projectile,
    pub position: FixedVec3,
}

/// Fait naître un projectile de la table du parent (id `shot.projectile`) au point de fin
/// du parent : même tireur, équipe, tags et politique de tir ami. `None` si l'id est absent
/// de la table (refusé par le lint) ou si la chaîne dépasse [`MAX_GENERATION`].
pub fn spawn_child_projectile(
    commands: &mut Commands,
    parent: &Parent,
    shot: &Shot,
    frame: u32,
    collision_settings: &CollisionSettings,
    id_factory: &mut GgrsNetIdFactory,
) -> Option<Entity> {
    let generation = parent.projectile.generation + 1;
    if generation > MAX_GENERATION {
        warn!(
            "projectile {} : on_expire au-delà de {} générations, ignoré",
            shot.projectile, MAX_GENERATION
        );
        return None;
    }
    let Some(def) = parent.projectile.table.get(&shot.projectile) else {
        warn!(
            "projectile {} : absent de la table projectiles de l'arme, ignoré",
            shot.projectile
        );
        return None;
    };
    let projectile = Projectile::new(
        shot.projectile.clone(),
        &def.spec(),
        parent.projectile.table.clone(),
        parent.projectile.damage_mult,
        generation,
    );
    let speed = shot.speed.unwrap_or(def.speed);
    let velocity = FixedVec2::new(
        shot.direction.x.saturating_mul(speed) / Fixed::from_num(60),
        shot.direction.y.saturating_mul(speed) / Fixed::from_num(60),
    );
    // Les tags du parent contiennent déjà `bullet` (insertion idempotente, `Tags` est un
    // ensemble) : la balle née porte exactement les mêmes.
    Some(crate::weapons::spawn_bullet(
        commands,
        collision_settings,
        id_factory,
        crate::weapons::BulletSpawn {
            position: parent.position,
            rotation: fixed_math::FixedMat3::IDENTITY,
            velocity,
            bullet_type: BulletType::Standard {
                damage: def.damage,
                speed,
            },
            damage: def.damage.saturating_mul(parent.projectile.damage_mult),
            range: def.range,
            player_handle: parent.bullet.player_handle,
            created_at: frame,
            source: &parent.bullet.source,
            source_team: parent.bullet.source_team,
            source_tags: &parent.bullet.tags,
            friendly_fire: parent.bullet.friendly_fire,
            projectile: Some(projectile),
            id_label: shot.projectile.clone(),
        },
    ))
}

// ---------------------------------------------------------------------------------------
// Systèmes (RollbackSystemSet::Projectiles)
// ---------------------------------------------------------------------------------------

type TargetQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GgrsNetId,
        &'static fixed_math::FixedTransform3D,
        &'static Collider,
        &'static Team,
    ),
    (With<Health>, Without<Bullet>, With<Rollback>),
>;

/// Collisions des projectiles composables. Personnages d'abord (tous ceux en contact cette
/// frame, par `GgrsNetId`, tant que `Pierce` le permet ; jamais deux fois le même), puis
/// murs (`Bounce` : vitesse réfléchie et retour à la position d'avant le déplacement ; sinon
/// fin). Mêmes filtres que les balles ordinaires : murs par `layer_matrix`, personnages par
/// `team_allows_hit` (`Team::Neutral` arrête le projectile sans être blessé).
#[allow(clippy::type_complexity)]
pub fn projectile_collision_system(
    frame: Res<FrameCount>,
    settings: Res<CollisionSettings>,
    grids: Res<CollisionGrids>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    mut hit_events: ResMut<FrameEvents<ProjectileHit>>,
    mut wall_hit_events: ResMut<FrameEvents<ProjectileWallHit>>,
    mut projectile_query: Query<
        (
            &GgrsNetId,
            &mut fixed_math::FixedTransform3D,
            &mut Bullet,
            &mut Projectile,
            &Collider,
            &CollisionLayer,
        ),
        With<Rollback>,
    >,
    wall_query: Query<
        (&fixed_math::FixedTransform3D, &Collider, &CollisionLayer),
        (With<Wall>, With<Rollback>, Without<Bullet>),
    >,
    target_query: TargetQuery,
) {
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "projectile_collisions"
    );
    let _enter = system_span.enter();

    for (net_id, mut transform, mut bullet, mut projectile, collider, layer) in
        order_mut_iter!(projectile_query)
    {
        if projectile.ended {
            continue;
        }
        let old_pos = FixedVec3::new(
            transform.translation.x - bullet.velocity.x,
            transform.translation.y - bullet.velocity.y,
            transform.translation.z,
        );
        let swept = union_aabb(
            collider_aabb(&old_pos, collider),
            collider_aabb(&transform.translation, collider),
        );

        // Personnages : tous les candidats en contact, par GgrsNetId.
        let mut characters: Vec<GgrsNetId> = Vec::new();
        for entry in grids.characters.query_aabb(&swept) {
            let Ok((target_id, target_transform, target_collider, target_team)) =
                target_query.get(entry.entity)
            else {
                continue;
            };
            if !team_allows_hit(
                bullet.source_team,
                *target_team,
                bullet.friendly_fire,
                &bullet.tags,
            ) || !projectile.can_hit(target_id)
            {
                continue;
            }
            if is_colliding(
                &transform.translation,
                collider,
                &target_transform.translation,
                target_collider,
            ) {
                characters.push(target_id.clone());
            }
        }
        characters.sort_unstable_by_key(|id| id.0);
        characters.dedup_by_key(|id| id.0);

        for target in characters {
            info!("projectile {} hits {}", net_id, target);
            damage_events.send(DamageEvent {
                source: bullet.source.clone(),
                target: target.clone(),
                kind: DamageKind::Physical,
                amount: bullet.damage,
                frame: frame.frame,
                tags: bullet.tags.clone(),
                source_team: bullet.source_team,
                friendly_fire: bullet.friendly_fire,
            });
            if !projectile.on_hit.is_empty() {
                hit_events.send(ProjectileHit {
                    projectile: projectile.id.clone(),
                    source: bullet.source.clone(),
                    target: target.clone(),
                    actions: projectile.on_hit.clone(),
                    frame: frame.frame,
                });
            }
            if projectile.register_hit(target) {
                break;
            }
        }
        if projectile.ended {
            continue;
        }

        // Murs.
        let walls: Vec<_> = grids
            .walls
            .query_aabb(&swept)
            .into_iter()
            .filter_map(|entry| wall_query.get(entry.entity).ok())
            .filter(|(_, _, wall_layer)| settings.layer_matrix[layer.0][wall_layer.0])
            .collect();
        let collides_at = |pos: &FixedVec3| {
            walls.iter().any(|(wall_transform, wall_collider, _)| {
                is_colliding(pos, collider, &wall_transform.translation, wall_collider)
            })
        };
        if !collides_at(&transform.translation) {
            continue;
        }
        if !projectile.on_hit.is_empty() {
            wall_hit_events.send(ProjectileWallHit {
                projectile: projectile.id.clone(),
                source: bullet.source.clone(),
                x: transform.translation.x,
                y: transform.translation.y,
                actions: projectile.on_hit.clone(),
                frame: frame.frame,
            });
        }
        if projectile.register_wall() {
            let (flip_x, flip_y) = bounce_axes(old_pos, bullet.velocity, collides_at);
            bullet.velocity = reflect(bullet.velocity, flip_x, flip_y);
            transform.translation = old_pos;
            info!(
                "projectile {} bounces ({} left)",
                net_id, projectile.bounces_left
            );
        } else {
            info!("projectile {} stopped by a wall", net_id);
        }
    }
}

/// Fin des projectiles : décompte `Lifetime`, puis pour chaque projectile terminé (cette
/// frame, par n'importe quelle cause) joue `on_expire` et le détruit (`despawn_rollback`).
/// Ordre `GgrsNetId` : les `GgrsNetId` des projectiles nés sont alloués dans le même ordre
/// sur tous les clients.
#[allow(clippy::too_many_arguments)]
pub fn projectile_expire_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut projectile_query: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &Bullet,
            &mut Projectile,
        ),
        With<Rollback>,
    >,
    target_query: TargetQuery,
    // T1.2 : résolution des `Named` de `on_expire` (absente : aucun pattern nommé).
    library: Option<Res<PatternLibrary>>,
    // T1.6 : `on_expire: [DestroyTerrain]` (absente hors jeu complet : ignorée).
    mut terrain_requests: Option<ResMut<FrameEvents<world::DestroyTerrainRequest>>>,
) {
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "projectile_expire"
    );
    let _enter = system_span.enter();

    for (net_id, entity, transform, bullet, mut projectile) in order_mut_iter!(projectile_query) {
        if !projectile.ended {
            projectile.tick_lifetime();
        }
        if !projectile.ended {
            continue;
        }
        info!("projectile {} expires", net_id);
        let forward = bullet.velocity.normalize_or_zero();
        let aim = nearest_target(
            &transform.translation,
            bullet,
            &projectile,
            order_iter!(target_query)
                .into_iter()
                .map(|(id, t, _, team)| (id, t, team)),
        );
        let parent = Parent {
            bullet,
            projectile: &projectile,
            position: transform.translation,
        };
        for action in &projectile.on_expire {
            match action {
                ExpireAction::DestroyTerrain { radius } => {
                    if let Some(requests) = terrain_requests.as_mut() {
                        requests.send(world::DestroyTerrainRequest::new(
                            transform.translation.truncate(),
                            *radius,
                        ));
                    }
                }
                ExpireAction::Spawn(pattern) => {
                    let pattern = match resolve_pattern(library.as_deref(), pattern) {
                        Ok(pattern) => pattern,
                        Err(name) => {
                            warn!(
                                "projectile {} : on_expire : pattern nommé inconnu « {} » (voir `alacod lint`), ignoré",
                                net_id, name
                            );
                            continue;
                        }
                    };
                    for shot in pattern_shots(&pattern, forward, aim) {
                        spawn_child_projectile(
                            &mut commands,
                            &parent,
                            &shot,
                            frame.frame,
                            &settings,
                            &mut id_factory,
                        );
                    }
                }
            }
        }
        commands.entity(entity).despawn_rollback();
    }
}

/// `Gravity` puis `Homing` sur la vitesse, appliquée au déplacement de la frame suivante.
pub fn projectile_steering_system(
    frame: Res<FrameCount>,
    mut projectile_query: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &mut Bullet,
            &Projectile,
        ),
        With<Rollback>,
    >,
    target_query: TargetQuery,
) {
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "projectile_steering"
    );
    let _enter = system_span.enter();

    for (_net_id, transform, mut bullet, projectile) in order_mut_iter!(projectile_query) {
        if projectile.ended {
            continue;
        }
        if let Some(gravity) = projectile.gravity {
            bullet.velocity = apply_gravity(bullet.velocity, gravity);
        }
        if let Some(force) = projectile.homing {
            if let Some(to_target) = nearest_target(
                &transform.translation,
                &bullet,
                projectile,
                order_iter!(target_query)
                    .into_iter()
                    .map(|(id, t, _, team)| (id, t, team)),
            ) {
                bullet.velocity = steer_towards(bullet.velocity, to_target, force);
            }
        }
    }
}

/// `on_hit` : chaque action à modificateur (`Action::as_modifier`) est posée sur la cible
/// touchée (si elle a des `Modifiers`), sous la source `projectile:<id>:<rang>`. Un nouveau
/// coup du même projectile rafraîchit le modificateur au lieu de l'empiler.
pub fn apply_projectile_on_hit_system(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<ProjectileHit>>,
    mut target_query: Query<(&GgrsNetId, &mut Modifiers), With<Rollback>>,
) {
    if events.is_empty() {
        return;
    }
    let system_span = span!(
        Level::INFO,
        "ggrs",
        f = frame.frame,
        s = "projectile_on_hit"
    );
    let _enter = system_span.enter();

    let mut by_net_id: BTreeMap<usize, Mut<Modifiers>> = target_query
        .iter_mut()
        .map(|(net_id, modifiers)| (net_id.0, modifiers))
        .collect();
    for event in events.iter() {
        let Some(modifiers) = by_net_id.get_mut(&event.target.0) else {
            continue;
        };
        for (rank, action) in event.actions.iter().enumerate() {
            let source = ModifierSource::Named(format!("projectile:{}:{rank}", event.projectile));
            if let Some(modifier) = action.as_modifier(event.frame, source.clone()) {
                modifiers.remove_by_source(&source);
                modifiers.push(modifier);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

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

    fn net(id: usize) -> GgrsNetId {
        GgrsNetId(id as _, "t".into())
    }

    fn projectile(modifiers: Vec<ProjectileModifier>) -> Projectile {
        Projectile::new(
            "test".into(),
            &ProjectileSpec {
                modifiers,
                ..Default::default()
            },
            Arc::new(ProjectileTable::new()),
            fixed_math::FIXED_ONE,
            0,
        )
    }

    #[test]
    fn spec_ron_et_vide() {
        let spec: ProjectileSpec = ron::from_str(
            r#"(modifiers: [Bounce(2)], on_expire: [Spawn(Ring(count: 8, speed: "300", projectile: "eclat", every: 0))])"#,
        )
        .unwrap();
        assert_eq!(spec.modifiers, vec![ProjectileModifier::Bounce(2)]);
        assert!(!spec.is_empty());
        assert!(ron::from_str::<ProjectileSpec>("()").unwrap().is_empty());
    }

    #[test]
    fn resolve_le_dernier_l_emporte_et_size_par_defaut() {
        let resolved =
            resolve_modifiers(&[ProjectileModifier::Bounce(1), ProjectileModifier::Bounce(3)]);
        assert_eq!(resolved.bounces, 3);
        assert_eq!(resolved.size, fixed_math::FIXED_ONE);
        assert_eq!(resolved.lifetime, None);
    }

    /// Bounce(n) : n rebonds, puis fin au mur suivant ; Bounce(0) : fin au premier mur.
    #[test]
    fn bounce_n_rebonds_puis_fin() {
        let mut p = projectile(vec![ProjectileModifier::Bounce(2)]);
        assert!(p.register_wall());
        assert!(p.register_wall());
        assert!(!p.ended);
        assert!(!p.register_wall());
        assert!(p.ended);

        let mut p = projectile(vec![ProjectileModifier::Bounce(0)]);
        assert!(!p.register_wall());
        assert!(p.ended);
    }

    /// Le rebond inverse l'axe qui entre dans le mur : mur vertical à droite (x ≥ 10).
    #[test]
    fn bounce_reflechit_l_axe_du_mur() {
        let old = FixedVec3::new(fx(8.0), fx(0.0), fx(0.0));
        let velocity = FixedVec2::new(fx(4.0), fx(1.0));
        let wall_right = |p: &FixedVec3| p.x >= fx(10.0);
        let (fx_, fy_) = bounce_axes(old, velocity, wall_right);
        assert_eq!((fx_, fy_), (true, false));
        assert_eq!(
            reflect(velocity, fx_, fy_),
            FixedVec2::new(fx(-4.0), fx(1.0))
        );
        // Coin : aucun axe seul n'entre, les deux s'inversent.
        let corner = |p: &FixedVec3| p.x >= fx(10.0) && p.y >= fx(0.5);
        assert_eq!(bounce_axes(old, velocity, corner), (true, true));
    }

    /// Pierce(n) : n personnages traversés, fin au (n+1)-ième ; jamais deux fois le même.
    #[test]
    fn pierce_traverse_n_cibles() {
        let mut p = projectile(vec![ProjectileModifier::Pierce(2)]);
        assert!(!p.register_hit(net(7)));
        assert!(!p.can_hit(&net(7)));
        assert!(p.can_hit(&net(3)));
        assert!(!p.register_hit(net(3)));
        assert!(p.register_hit(net(9)));
        assert_eq!(
            p.hits.iter().map(|h| h.0).collect::<Vec<_>>(),
            vec![3, 7, 9]
        );

        let mut p = projectile(vec![]);
        assert!(p.register_hit(net(1)), "sans Pierce : fin au premier coup");
    }

    /// Size multiplie le rayon de collision.
    #[test]
    fn size_multiplie_le_rayon() {
        let p = projectile(vec![ProjectileModifier::Size(fx(2.5))]);
        assert_eq!(p.radius(fx(5.0)), fx(12.5));
        assert_eq!(projectile(vec![]).radius(fx(5.0)), fx(5.0));
    }

    /// Lifetime(n) : n frames décomptées, fin à la suivante ; Lifetime(0) : fin immédiate.
    #[test]
    fn lifetime_expire_proprement() {
        let mut p = projectile(vec![ProjectileModifier::Lifetime(2)]);
        p.tick_lifetime();
        p.tick_lifetime();
        assert!(!p.ended);
        p.tick_lifetime();
        assert!(p.ended);

        let mut p = projectile(vec![ProjectileModifier::Lifetime(0)]);
        p.tick_lifetime();
        assert!(p.ended);

        let mut p = projectile(vec![]);
        for _ in 0..1000 {
            p.tick_lifetime();
        }
        assert!(!p.ended, "sans Lifetime, seule la portée termine");
    }

    /// Homing : la direction tourne vers la cible, la vitesse scalaire est conservée.
    #[test]
    fn homing_tourne_vers_la_cible_a_vitesse_constante() {
        let velocity = FixedVec2::new(fx(5.0), fx(0.0));
        let steered = steer_towards(velocity, FixedVec2::new(fx(0.0), fx(100.0)), fx(0.5));
        assert!(steered.y > Fixed::ZERO && steered.x > Fixed::ZERO);
        let diff = (steered.length() - velocity.length()).abs();
        assert!(diff < fx(0.01), "vitesse conservée : {}", steered.length());
        // Sans cible (direction nulle) ou à l'arrêt : inchangé.
        assert_eq!(steer_towards(velocity, FixedVec2::ZERO, fx(0.5)), velocity);
        assert_eq!(
            steer_towards(FixedVec2::ZERO, FixedVec2::new(fx(1.0), fx(0.0)), fx(0.5)),
            FixedVec2::ZERO
        );
    }

    /// Gravity(g) : g unités/s² ajoutées à la vitesse verticale, g/3600 par frame.
    #[test]
    fn gravity_acceleration_verticale_constante() {
        let mut velocity = FixedVec2::new(fx(2.0), fx(0.0));
        for _ in 0..60 {
            velocity = apply_gravity(velocity, fx(-360.0));
        }
        assert_eq!(velocity.x, fx(2.0));
        assert!((velocity.y - fx(-6.0)).abs() < fx(0.01), "{}", velocity.y);
    }

    /// Ring : `count` directions régulières à partir de l'avant, vitesse du pattern.
    #[test]
    fn ring_repartit_count_directions() {
        let ring = Pattern::Ring {
            count: 4,
            speed: fx(300.0),
            projectile: "eclat".into(),
            every: 0,
        };
        let shots = pattern_shots(&ring, FixedVec2::new(fx(1.0), fx(0.0)), None);
        assert_eq!(shots.len(), 4);
        assert!(shots.iter().all(|s| s.speed == Some(fx(300.0))));
        let d = |i: usize| shots[i].direction;
        assert!((d(0).x - fx(1.0)).abs() < fx(0.01) && d(0).y.abs() < fx(0.01));
        assert!(d(1).x.abs() < fx(0.01) && (d(1).y - fx(1.0)).abs() < fx(0.01));
        assert!((d(2).x + fx(1.0)).abs() < fx(0.01));
        assert!((d(3).y + fx(1.0)).abs() < fx(0.01));
    }

    /// Spread/Aimed : éventail centré, bornes comprises ; Aimed vise la cible si elle existe ;
    /// Sequence concatène, Telegraph/Wait ne tirent rien.
    #[test]
    fn spread_aimed_sequence() {
        let forward = FixedVec2::new(fx(1.0), fx(0.0));
        let spread = Pattern::Spread {
            count: 3,
            spread: fx(1.0),
            projectile: "p".into(),
        };
        let shots = pattern_shots(&spread, forward, None);
        assert_eq!(shots.len(), 3);
        assert!(shots[1].direction.y.abs() < fx(0.01));
        assert!(shots[0].direction.y < Fixed::ZERO && shots[2].direction.y > Fixed::ZERO);

        let aimed = Pattern::Aimed {
            count: 1,
            spread: Fixed::ZERO,
            projectile: "p".into(),
        };
        let up = FixedVec2::new(fx(0.0), fx(3.0));
        let shot = &pattern_shots(&aimed, forward, Some(up))[0];
        assert!((shot.direction.y - fx(1.0)).abs() < fx(0.01));

        let sequence = Pattern::Sequence(vec![
            Pattern::Telegraph(30),
            aimed,
            Pattern::Wait(5),
            spread,
        ]);
        assert_eq!(pattern_shots(&sequence, forward, None).len(), 4);
    }

    /// Explosion : à vitesse nulle, `forward` nul -> axe +x, rien ne panique.
    #[test]
    fn pattern_a_l_arret() {
        let ring = Pattern::Ring {
            count: 2,
            speed: fx(10.0),
            projectile: "p".into(),
            every: 0,
        };
        let shots = pattern_shots(&ring, FixedVec2::ZERO, None);
        assert!((shots[0].direction.x - fx(1.0)).abs() < fx(0.01));
    }
}
