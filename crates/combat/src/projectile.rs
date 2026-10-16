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
use bevy_fixed::rng::RngStreams;
use bevy_ggrs::{Rollback, RollbackDespawnCommandExtension};
use effects::Action;
use serde::{Deserialize, Serialize};
use sim_core::{
    damage::{DamageEvent, DamageKind, FriendlyFire},
    frame_events::FrameEvents,
    modifier::{Modifier, ModifierOp, ModifierSource, Modifiers},
    stats::StatId,
    tag::Tags,
    team::Team,
};
use utils::{
    frame::FrameCount,
    net_id::{GgrsNetId, GgrsNetIdFactory},
    order_iter, order_mut_iter,
};

use crate::status::{StatusDef, StatusLibrary, Statuses};
use crate::{
    actors::Health,
    collider::{is_colliding, Collider, CollisionLayer, CollisionSettings, Wall},
    collision_grid::{collider_aabb, union_aabb, CollisionGrids},
    team::team_allows_hit,
    weapons::{Bullet, BulletType},
};
/// Contrats déplacés dans `effects::projectile` par M3-T0a (§60), réexportés ici : chemins
/// inchangés pour tout le code existant.
pub use effects::projectile::{
    ExpireAction, Pattern, ProjectileModifier, ProjectileMods, ProjectileSpec,
};

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

/// `Split` résolu (M3-T0a, exécuté par M3-T1, §66).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedSplit {
    pub count: u32,
    pub spread: Fixed,
    pub child: Option<String>,
}

/// `Orbit` résolu (M3-T0a, exécuté par M3-T1, §66).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedOrbit {
    pub radius: Fixed,
    pub frames_per_turn: u32,
}

/// Valeurs résolues d'une liste de modificateurs. Un même modificateur répété : le dernier
/// l'emporte (le lint refuse les doublons).
///
/// M3-T0a : plus `Copy` (`split` porte l'id du projectile enfant) ; M3-T1 : `spectral`, `split`
/// et `orbit` sont lus par [`Projectile`] et exécutés par les systèmes de projectiles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedModifiers {
    pub bounces: u32,
    pub pierces: u32,
    pub size: Fixed,
    pub lifetime: Option<u32>,
    pub homing: Option<Fixed>,
    pub gravity: Option<Fixed>,
    pub spectral: bool,
    pub split: Option<ResolvedSplit>,
    pub orbit: Option<ResolvedOrbit>,
}

pub fn resolve_modifiers(modifiers: &[ProjectileModifier]) -> ResolvedModifiers {
    let mut resolved = ResolvedModifiers {
        bounces: 0,
        pierces: 0,
        size: fixed_math::FIXED_ONE,
        lifetime: None,
        homing: None,
        gravity: None,
        spectral: false,
        split: None,
        orbit: None,
    };
    for modifier in modifiers {
        match modifier {
            ProjectileModifier::Bounce(n) => resolved.bounces = *n,
            ProjectileModifier::Pierce(n) => resolved.pierces = *n,
            ProjectileModifier::Size(factor) => resolved.size = *factor,
            ProjectileModifier::Lifetime(frames) => resolved.lifetime = Some(*frames),
            ProjectileModifier::Homing(force) => resolved.homing = Some(*force),
            ProjectileModifier::Gravity(accel) => resolved.gravity = Some(*accel),
            ProjectileModifier::Spectral => resolved.spectral = true,
            ProjectileModifier::Split {
                count,
                spread,
                child,
            } => {
                resolved.split = Some(ResolvedSplit {
                    count: *count,
                    spread: *spread,
                    child: child.clone(),
                })
            }
            ProjectileModifier::Orbit {
                radius,
                frames_per_turn,
            } => {
                resolved.orbit = Some(ResolvedOrbit {
                    radius: *radius,
                    frames_per_turn: *frames_per_turn,
                })
            }
        }
    }
    resolved
}

/// M3-T0a (`docs/conventions.md` §60, règles retenues par William le 2026-10-15, Q3) : compose
/// les modificateurs de l'arme (`base`) avec ceux apportés par d'autres sources (`ajouts`), dans
/// l'ordre déterministe fixé par l'appelant : arme, passifs dans l'ordre de ramassage (un passif
/// en double compte deux fois), actif, synergies par id (voir `items::projectile_sources`).
///
/// Chaque liste est d'abord résolue seule ([`resolve_modifiers`] : dans une même liste, le
/// dernier l'emporte, et le lint y refuse les doublons) ; les sources se combinent ensuite :
///
/// | Modificateur | Règle entre sources |
/// |---|---|
/// | `Bounce(n)`, `Pierce(n)` | addition (saturante) |
/// | `Size(f)` | produit |
/// | `Lifetime(n)` | maximum des sources qui en ont un |
/// | `Homing(f)` | somme plafonnée à 1 |
/// | `Gravity(g)` | somme |
/// | `Spectral` | présence (une source suffit) |
/// | `Split` | addition des `count` ; `spread` et `child` de la première source qui en a un |
/// | `Orbit` | présence ; paramètres de la première source qui en a un |
///
/// Sans ajout, le résultat est exactement `resolve_modifiers(base)` (aucun calcul de plus, pas
/// même un produit par 1). Fonction pure, `Fixed` seulement, opérations saturantes.
pub fn compose(base: &[ProjectileModifier], ajouts: &[&[ProjectileModifier]]) -> ResolvedModifiers {
    let mut out = resolve_modifiers(base);
    for source in ajouts {
        let add = resolve_modifiers(source);
        out.bounces = out.bounces.saturating_add(add.bounces);
        out.pierces = out.pierces.saturating_add(add.pierces);
        out.size = out.size.saturating_mul(add.size);
        out.lifetime = match (out.lifetime, add.lifetime) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        out.homing = match (out.homing, add.homing) {
            (Some(a), Some(b)) => Some(a.saturating_add(b).min(fixed_math::FIXED_ONE)),
            (a, b) => a.or(b),
        };
        out.gravity = match (out.gravity, add.gravity) {
            (Some(a), Some(b)) => Some(a.saturating_add(b)),
            (a, b) => a.or(b),
        };
        out.spectral |= add.spectral;
        out.split = match (out.split, add.split) {
            (Some(mut a), Some(b)) => {
                a.count = a.count.saturating_add(b.count);
                Some(a)
            }
            (a, b) => a.or(b),
        };
        out.orbit = out.orbit.or(add.orbit);
    }
    out
}

/// Orbite d'un projectile `Orbit` (M3-T1, §66) : le contrat résolu et la phase de départ (angle
/// de la direction de tir, fixé à l'apparition par [`Projectile::start_orbit`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OrbitState {
    pub radius: Fixed,
    pub frames_per_turn: u32,
    pub phase: Fixed,
}

/// Frames qu'un projectile `Orbit` met à atteindre son rayon (il part du tireur).
pub const ORBIT_RAMP_FRAMES: u32 = 10;

/// État rollback d'un projectile composable, posé à côté de `weapons::Bullet`.
///
/// `Hash` manuel : les champs de M3-T1 (`spectral`, `split`, `orbit`, `luck`) ne sont hachés que
/// s'ils sont actifs, donc le hash d'un projectile d'avant M3-T1 est exactement celui d'avant
/// (aucune trace existante ne bouge).
#[derive(Component, Clone)]
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
    /// M3-T1 : traverse murs et obstacles (`Spectral`), pas les personnages.
    pub spectral: bool,
    /// M3-T1 : à l'impact sur un personnage, `count` projectiles en éventail (`Split`).
    pub split: Option<ResolvedSplit>,
    /// M3-T1 : tourne autour du tireur (`Orbit`).
    pub orbit: Option<OrbitState>,
    /// M3-T1 : stat `Luck` du tireur au tir, pour `Action::Chance` (0 par défaut).
    pub luck: Fixed,
}

impl std::hash::Hash for Projectile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Champs d'avant M3-T1, dans l'ordre historique du `derive(Hash)`.
        self.id.hash(state);
        self.bounces_left.hash(state);
        self.pierces_left.hash(state);
        self.size.hash(state);
        self.lifetime_left.hash(state);
        self.homing.hash(state);
        self.gravity.hash(state);
        self.hits.hash(state);
        self.ended.hash(state);
        self.generation.hash(state);
        self.damage_mult.hash(state);
        self.on_hit.hash(state);
        self.on_expire.hash(state);
        self.table.hash(state);
        // M3-T1 : seulement s'il y a quelque chose (jamais pour un projectile d'arme existante).
        if self.spectral || self.split.is_some() || self.orbit.is_some() || self.luck != Fixed::ZERO
        {
            self.spectral.hash(state);
            self.split.hash(state);
            self.orbit.hash(state);
            self.luck.hash(state);
        }
    }
}

/// Debug compact : la table n'est donnée que par ses clés (elle est identique pour toute la
/// descendance d'un tir, la répéter à chaque ligne de la trace détaillée n'apprend rien).
impl std::fmt::Debug for Projectile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Projectile");
        debug
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
            .field("table", &self.table.keys().collect::<Vec<_>>());
        // M3-T1 : seulement s'ils sont actifs (le dump détaillé d'un projectile d'avant ne change pas)
        if self.spectral || self.split.is_some() || self.orbit.is_some() || self.luck != Fixed::ZERO
        {
            debug
                .field("spectral", &self.spectral)
                .field("split", &self.split)
                .field("orbit", &self.orbit)
                .field("luck", &self.luck);
        }
        debug.finish()
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
        Self::from_resolved(
            id,
            resolve_modifiers(&spec.modifiers),
            spec.on_hit.clone(),
            spec.on_expire.clone(),
            table,
            damage_mult,
            generation,
        )
    }

    /// Projectile tiré, composé (M3-T1, §66) : les modificateurs de `base` (le mode de tir de
    /// l'arme) et ceux des `extras` (`ProjectileMods` des objets et synergies, dans l'ordre de
    /// `items::projectile_mods`) sont combinés par [`compose`] ; `on_hit` et `on_expire` s'y
    /// ajoutent à ceux de l'arme, dans le même ordre. Sans `extras` : exactement
    /// [`Projectile::new`].
    pub fn composed(
        id: String,
        base: &ProjectileSpec,
        extras: &[&ProjectileMods],
        table: Arc<ProjectileTable>,
        damage_mult: Fixed,
    ) -> Self {
        let lists: Vec<&[ProjectileModifier]> = extras
            .iter()
            .map(|mods| mods.modifiers.as_slice())
            .collect();
        let mut on_hit = base.on_hit.clone();
        let mut on_expire = base.on_expire.clone();
        for mods in extras {
            on_hit.extend(mods.on_hit.iter().cloned());
            on_expire.extend(mods.on_expire.iter().cloned());
        }
        Self::from_resolved(
            id,
            compose(&base.modifiers, &lists),
            on_hit,
            on_expire,
            table,
            damage_mult,
            0,
        )
    }

    /// Un `on_hit` contient une `Action::Chance` (la stat `Luck` du tireur est alors lue au tir).
    pub fn has_chance(&self) -> bool {
        self.on_hit
            .iter()
            .any(|action| matches!(action, Action::Chance { .. }))
    }

    fn from_resolved(
        id: String,
        resolved: ResolvedModifiers,
        on_hit: Vec<Action>,
        on_expire: Vec<ExpireAction>,
        table: Arc<ProjectileTable>,
        damage_mult: Fixed,
        generation: u8,
    ) -> Self {
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
            on_hit,
            on_expire,
            table,
            spectral: resolved.spectral,
            split: resolved.split,
            orbit: resolved.orbit.map(|orbit| OrbitState {
                radius: orbit.radius,
                frames_per_turn: orbit.frames_per_turn,
                phase: Fixed::ZERO,
            }),
            luck: Fixed::ZERO,
        }
    }

    /// Fixe la phase de l'orbite sur la direction de tir (`velocity` à l'apparition) : le projectile
    /// part du côté où il a été tiré. Sans `Orbit`, ne fait rien.
    pub fn start_orbit(&mut self, velocity: FixedVec2) {
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.phase = angle_of(velocity);
        }
    }

    /// Chances résolues pour un coup (M3-T1, §66) : chaque `Action::Chance` de `on_hit` est
    /// tirée (RNG local, voir [`resolve_chances`]).
    pub fn resolved_on_hit(&self, run_seed: u32, net_id: usize) -> Vec<Action> {
        resolve_chances(&self.on_hit, run_seed, net_id, self.luck)
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

/// Mur touché par une balle ou un projectile (M2-T11) : émis par les deux systèmes de collision
/// avec le `GgrsNetId` du mur (le plus petit des murs en contact). Lu par `game::props` (une
/// table renversée compte les balles qu'elle encaisse). File transitoire (hors checksum et hors
/// snapshot, `add_frame_events_transient`) : produite et lue dans la même frame.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BulletWallHit {
    pub wall: GgrsNetId,
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

/// `on_hit` d'un coup, chances résolues (M3-T1, §66). Une `Action::Chance` dont le tirage réussit
/// est remplacée par son `then` ; celle qui échoue est **laissée telle quelle** (les systèmes
/// d'application l'ignorent), ce qui garde le rang de chaque action dans la liste (la source des
/// modificateurs posés, `projectile:<id>:<rang>`, ne dépend donc pas du hasard). Tirage :
/// [`effects::chance::roll_chance`], RNG local de graine `(run_seed, net_id du projectile, rang)`,
/// chance = `chance_with_luck(base, luck, per_luck)`. Les autres actions passent inchangées.
pub fn resolve_chances(
    on_hit: &[Action],
    run_seed: u32,
    net_id: usize,
    luck: Fixed,
) -> Vec<Action> {
    on_hit
        .iter()
        .enumerate()
        .map(|(rank, action)| match action {
            Action::Chance {
                base,
                per_luck,
                then,
            } => {
                let chance = effects::chance_with_luck(*base, luck, *per_luck);
                if effects::chance::roll_chance(run_seed, net_id, rank as u32, chance) {
                    (**then).clone()
                } else {
                    action.clone()
                }
            }
            other => other.clone(),
        })
        .collect()
}

/// Position d'un projectile `Orbit` autour de `center` après `age` frames (M3-T1) : angle
/// `phase + 2π · (age mod frames_per_turn) / frames_per_turn`, rayon qui croît linéairement de 0
/// à `radius` pendant [`ORBIT_RAMP_FRAMES`]. Pure, `Fixed` seulement.
pub fn orbit_position(center: FixedVec2, orbit: &OrbitState, age: u32) -> FixedVec2 {
    let per_turn = orbit.frames_per_turn.max(1);
    let fraction =
        Fixed::saturating_from_num(age % per_turn) / Fixed::saturating_from_num(per_turn);
    let angle = orbit
        .phase
        .saturating_add(fraction.saturating_mul(fixed_math::FIXED_TAU));
    let ramp = Fixed::from_num(age.min(ORBIT_RAMP_FRAMES)) / Fixed::from_num(ORBIT_RAMP_FRAMES);
    let radius = orbit.radius.saturating_mul(ramp);
    let direction = direction_of(angle);
    FixedVec2::new(
        center.x.saturating_add(direction.x.saturating_mul(radius)),
        center.y.saturating_add(direction.y.saturating_mul(radius)),
    )
}

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

/// Direction unitaire d'un angle. Le CORDIC de `fixed_math` est juste à 5e-5 près sur
/// `[-3π/2, 3π/2]`, puis perd sa précision (1e-4 dès 5,24 rad, 7e-3 en 2π, mesuré dans
/// `bevy_fixed`) et **déborde** (panique `fixed`) au-delà de `2π + 0,03` (ex. 7,07 rad : le 8e
/// rayon d'une couronne dont le premier part à π/2, cas d'une tourelle qui vise vers le haut,
/// T1.2). L'angle est donc replié par tours de `2π` jusque dans `[-3π/2, 3π/2]` (D34). Les angles
/// déjà dans cet intervalle passent tels quels, au bit près ; seuls changent ceux de `]3π/2, 2π+]`
/// et leurs symétriques (ex. le dernier rayon d'une couronne de 24 : 6,02 rad, écart 2e-3 avant).
pub(crate) fn direction_of(angle: Fixed) -> FixedVec2 {
    let limit = fixed_math::FIXED_PI + fixed_math::FIXED_PI.saturating_div(Fixed::from_num(2));
    let mut angle = angle;
    while angle > limit {
        angle -= fixed_math::FIXED_TAU;
    }
    while angle < -limit {
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
        Pattern::Fan {
            count,
            spread,
            speed,
            projectile,
        } => {
            for angle in fan_angles(angle_of(forward), *count, *spread) {
                shots.push(Shot {
                    projectile: projectile.clone(),
                    direction: direction_of(angle),
                    speed: (*speed > Fixed::ZERO).then_some(*speed),
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
        Pattern::Telegraph(_)
        | Pattern::Wait(_)
        | Pattern::Scatter { .. }
        | Pattern::Spiral { .. }
        | Pattern::Burst { .. }
        | Pattern::Named(_) => {}
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
            || net_id.0 == bullet.source.0
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
    let mut projectile = Projectile::new(
        shot.projectile.clone(),
        &def.spec(),
        parent.projectile.table.clone(),
        parent.projectile.damage_mult,
        generation,
    );
    // M3-T1 : la chance du tireur suit la descendance.
    projectile.luck = parent.projectile.luck;
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
            shot_size: fixed_math::FIXED_ONE,
        },
    ))
}

/// `Split` (M3-T1, §66) : à l'impact du projectile `parent` sur un personnage, fait naître `count`
/// projectiles en éventail de `spread` radians autour de sa direction de vol, à sa position.
///
/// - `child: Some(id)` : l'entrée `id` de la table `projectiles` de l'arme (dégâts, vitesse,
///   portée et modificateurs de sa définition) ; id absent de la table (un objet n'a pas de table :
///   le lint le refuse pour un objet) : ignoré avec un `warn!`.
/// - `child: None` : le projectile par défaut du porteur, c'est-à-dire une copie du parent (mêmes
///   dégâts, vitesse et portée, modificateurs et `on_hit` **restants**, sans `Split` ni `Orbit` ni
///   `on_expire`).
///
/// Dans les deux cas, l'enfant garde la liste des personnages déjà touchés par le parent (il ne
/// retouche pas la cible qu'il quitte) et compte une génération de plus ([`MAX_GENERATION`]).
/// Les `GgrsNetId` sont alloués dans l'ordre de l'éventail, après ceux des projectiles de
/// `GgrsNetId` plus petit (l'appelant itère dans cet ordre).
pub fn spawn_split_children(
    commands: &mut Commands,
    parent: &Parent,
    split: &ResolvedSplit,
    frame: u32,
    collision_settings: &CollisionSettings,
    id_factory: &mut GgrsNetIdFactory,
) {
    let generation = parent.projectile.generation + 1;
    if generation > MAX_GENERATION {
        warn!(
            "projectile {} : Split au-delà de {} générations, ignoré",
            parent.projectile.id, MAX_GENERATION
        );
        return;
    }
    let forward = parent.bullet.velocity.normalize_or_zero();
    let forward = if forward == FixedVec2::ZERO {
        FixedVec2::new(fixed_math::FIXED_ONE, Fixed::ZERO)
    } else {
        forward
    };
    for angle in fan_angles(angle_of(forward), split.count, split.spread) {
        let direction = direction_of(angle);
        let (mut projectile, bullet_type, damage, speed, range, label) = match &split.child {
            Some(id) => {
                let Some(def) = parent.projectile.table.get(id) else {
                    warn!(
                        "projectile {} : Split : enfant « {} » absent de la table projectiles de l'arme, ignoré",
                        parent.projectile.id, id
                    );
                    return;
                };
                let mut child = Projectile::new(
                    id.clone(),
                    &def.spec(),
                    parent.projectile.table.clone(),
                    parent.projectile.damage_mult,
                    generation,
                );
                child.luck = parent.projectile.luck;
                (
                    child,
                    BulletType::Standard {
                        damage: def.damage,
                        speed: def.speed,
                    },
                    def.damage.saturating_mul(parent.projectile.damage_mult),
                    def.speed,
                    def.range,
                    id.clone(),
                )
            }
            None => {
                let mut child = parent.projectile.clone();
                child.split = None;
                child.orbit = None;
                child.on_expire = Vec::new();
                child.ended = false;
                child.generation = generation;
                (
                    child,
                    parent.bullet.bullet_type,
                    parent.bullet.damage,
                    bullet_type_speed(&parent.bullet.bullet_type),
                    parent.bullet.range,
                    parent.projectile.id.clone(),
                )
            }
        };
        // L'enfant ne retouche pas ce que le parent a déjà touché.
        projectile.hits = parent.projectile.hits.clone();
        let velocity = FixedVec2::new(
            direction.x.saturating_mul(speed) / Fixed::from_num(60),
            direction.y.saturating_mul(speed) / Fixed::from_num(60),
        );
        crate::weapons::spawn_bullet(
            commands,
            collision_settings,
            id_factory,
            crate::weapons::BulletSpawn {
                position: parent.position,
                rotation: fixed_math::FixedMat3::IDENTITY,
                velocity,
                bullet_type,
                damage,
                range,
                player_handle: parent.bullet.player_handle,
                created_at: frame,
                source: &parent.bullet.source,
                source_team: parent.bullet.source_team,
                source_tags: &parent.bullet.tags,
                friendly_fire: parent.bullet.friendly_fire,
                projectile: Some(projectile),
                id_label: label,
                shot_size: fixed_math::FIXED_ONE,
            },
        );
    }
}

/// Vitesse (unités/seconde) d'un type de balle.
fn bullet_type_speed(bullet_type: &BulletType) -> Fixed {
    match bullet_type {
        BulletType::Standard { speed, .. }
        | BulletType::Explosive { speed, .. }
        | BulletType::Piercing { speed, .. } => *speed,
    }
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
        &'static Health,
        Option<&'static crate::actors::DashExtra>,
    ),
    (Without<Bullet>, With<Rollback>),
>;

/// Collisions des projectiles composables. Personnages d'abord (tous ceux en contact cette
/// frame, par `GgrsNetId`, tant que `Pierce` le permet ; jamais deux fois le même), puis
/// murs (`Bounce` : vitesse réfléchie et retour à la position d'avant le déplacement ; sinon
/// fin). Mêmes filtres que les balles ordinaires : murs par `layer_matrix`, personnages par
/// `team_allows_hit` (`Team::Neutral` arrête le projectile sans être blessé). Un personnage
/// invulnérable (i-frames du dash, `Health::is_invulnerable_at`) est traversé sans être
/// touché ; `Homing` et `Aimed` le visent toujours.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn projectile_collision_system(
    mut commands: Commands,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    // M3-T1 : lecture seule de la graine de run (tirage de chance local, jamais un flux)
    rng_streams: Res<RngStreams>,
    frame: Res<FrameCount>,
    settings: Res<CollisionSettings>,
    grids: Res<CollisionGrids>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    mut hit_events: ResMut<FrameEvents<ProjectileHit>>,
    mut wall_hit_events: ResMut<FrameEvents<ProjectileWallHit>>,
    mut bullet_wall_hits: ResMut<FrameEvents<BulletWallHit>>,
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
        (
            &fixed_math::FixedTransform3D,
            &Collider,
            &CollisionLayer,
            &GgrsNetId,
        ),
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
            let Ok((
                target_id,
                target_transform,
                target_collider,
                target_team,
                target_health,
                target_dash,
            )) = target_query.get(entry.entity)
            else {
                continue;
            };
            // D54 : le tir ami (`Always`, `Cursed`) ne touche jamais son propre tireur.
            if target_id.0 == bullet.source.0
                || !team_allows_hit(
                    bullet.source_team,
                    *target_team,
                    bullet.friendly_fire,
                    &bullet.tags,
                )
                || !projectile.can_hit(target_id)
                || target_health.is_invulnerable_at(frame.frame)
                || target_dash.is_some_and(|dash| dash.projectile_invulnerable_at(frame.frame))
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
                    // M3-T1 : chances résolues ici (le projectile seul connaît son net id)
                    actions: projectile.resolved_on_hit(rng_streams.run_seed, net_id.0),
                    frame: frame.frame,
                });
            }
            let ended = projectile.register_hit(target);
            // M3-T1 : `Split` à l'impact (après `register_hit` : l'enfant hérite de la cible
            // touchée et des perforations restantes).
            if let Some(split) = projectile.split.clone() {
                let parent = Parent {
                    bullet: &bullet,
                    projectile: &projectile,
                    position: transform.translation,
                };
                spawn_split_children(
                    &mut commands,
                    &parent,
                    &split,
                    frame.frame,
                    &settings,
                    &mut id_factory,
                );
            }
            if ended {
                break;
            }
        }
        if projectile.ended {
            continue;
        }

        // M3-T1 : `Spectral` traverse murs et obstacles (jamais les personnages, traités plus
        // haut).
        if projectile.spectral {
            continue;
        }

        // Murs.
        let walls: Vec<_> = grids
            .walls
            .query_aabb(&swept)
            .into_iter()
            .filter_map(|entry| wall_query.get(entry.entity).ok())
            .filter(|(_, _, wall_layer, _)| settings.layer_matrix[layer.0][wall_layer.0])
            .collect();
        let collides_at = |pos: &FixedVec3| {
            walls.iter().any(|(wall_transform, wall_collider, _, _)| {
                is_colliding(pos, collider, &wall_transform.translation, wall_collider)
            })
        };
        if !collides_at(&transform.translation) {
            continue;
        }
        // M2-T11 : le mur touché (le plus petit `GgrsNetId` en contact)
        if let Some(wall_id) = walls
            .iter()
            .filter(|(wall_transform, wall_collider, _, _)| {
                is_colliding(
                    &transform.translation,
                    collider,
                    &wall_transform.translation,
                    wall_collider,
                )
            })
            .min_by_key(|(_, _, _, id)| id.0)
            .map(|(_, _, _, id)| (*id).clone())
        {
            bullet_wall_hits.send(BulletWallHit {
                wall: wall_id,
                frame: frame.frame,
            });
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
                .map(|(id, t, _, team, _, _)| (id, t, team)),
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
        // M3-T1 : `Orbit` tourne autour du tireur au lieu d'avancer : la vitesse de la frame
        // suivante est l'écart jusqu'à la position d'orbite de la frame suivante. Le tireur
        // disparu (mort, détruit) termine le projectile ; ni gravité ni guidage en orbite.
        if let Some(orbit) = &projectile.orbit {
            let center = target_query
                .iter()
                .find(|(id, ..)| id.0 == bullet.source.0)
                .map(|(_, shooter, ..)| shooter.translation.truncate());
            match center {
                Some(center) => {
                    let age = (frame.frame + 1).saturating_sub(bullet.created_at);
                    let next = orbit_position(center, orbit, age);
                    bullet.velocity = FixedVec2::new(
                        next.x.saturating_sub(transform.translation.x),
                        next.y.saturating_sub(transform.translation.y),
                    );
                }
                None => bullet.range = Fixed::ZERO,
            }
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
                    .map(|(id, t, _, team, _, _)| (id, t, team)),
            ) {
                bullet.velocity = steer_towards(bullet.velocity, to_target, force);
            }
        }
    }
}

/// `on_hit` : chaque action à modificateur (`Action::as_modifier`) est posée sur la cible
/// touchée (si elle a des `Modifiers`), sous la source `projectile:<id>:<rang>`. Un nouveau
/// coup du même projectile rafraîchit le modificateur au lieu de l'empiler.
#[allow(clippy::too_many_arguments)]
pub fn apply_projectile_on_hit_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    events: Res<FrameEvents<ProjectileHit>>,
    library: Res<StatusLibrary>,
    mut target_query: Query<
        (&GgrsNetId, Entity, &mut Modifiers, Option<&mut Statuses>),
        With<Rollback>,
    >,
    teams: Query<(&GgrsNetId, &Team), With<Rollback>>,
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

    let team_of: BTreeMap<usize, Team> = teams.iter().map(|(id, team)| (id.0, *team)).collect();
    // Statuts posés sur une cible qui n'en portait pas : un seul `insert` par cible, à la fin
    let mut new_statuses: BTreeMap<usize, (Entity, Statuses)> = BTreeMap::new();
    let mut by_net_id: BTreeMap<usize, (Entity, Mut<Modifiers>, Option<Mut<Statuses>>)> =
        target_query
            .iter_mut()
            .map(|(net_id, entity, modifiers, statuses)| (net_id.0, (entity, modifiers, statuses)))
            .collect();
    for event in events.iter() {
        let Some((entity, modifiers, statuses)) = by_net_id.get_mut(&event.target.0) else {
            continue;
        };
        for (rank, action) in event.actions.iter().enumerate() {
            if let Action::ApplyStatus { status, stacks } = action {
                // T1.3 (§19) : statut posé sur le personnage touché
                let Some(spec) = library.statuses.get(status) else {
                    warn!("statut « {status} » inconnu (voir `alacod lint`)");
                    continue;
                };
                let source = Some((event.source.clone(), team_of.get(&event.source.0).copied()));
                let target_statuses = match statuses.as_deref_mut() {
                    Some(existing) => existing,
                    None => {
                        &mut new_statuses
                            .entry(event.target.0)
                            .or_insert_with(|| (*entity, Statuses::default()))
                            .1
                    }
                };
                target_statuses.apply(status, spec, source, event.frame, *stacks);
                info!(
                    "ggrs{{f={} status target={} id={} stacks={}}}",
                    event.frame,
                    event.target,
                    status,
                    target_statuses.stacks(status)
                );
                if spec.kind == StatusDef::Slow {
                    let until = target_statuses
                        .0
                        .iter()
                        .find(|entry| entry.id == *status)
                        .map(|entry| entry.expires_at_frame);
                    let source = ModifierSource::Named(format!("status:{status}"));
                    modifiers.remove_by_source(&source);
                    for stat in [StatId::MoveSpeed, StatId::EnemyMoveSpeed] {
                        modifiers.push(Modifier {
                            stat,
                            op: ModifierOp::Mul,
                            value: spec.factor,
                            source: source.clone(),
                            until,
                        });
                    }
                }
                continue;
            }
            let source = ModifierSource::Named(format!("projectile:{}:{rank}", event.projectile));
            if let Some(modifier) = action.as_modifier(event.frame, source.clone()) {
                modifiers.remove_by_source(&source);
                modifiers.push(modifier);
            }
        }
    }
    for (_, (entity, statuses)) in new_statuses {
        commands.entity(entity).insert(statuses);
    }
}

/// Statuts (T1.3, `docs/conventions.md` §19) : ticks de `Burn` (un `DamageEvent` de genre
/// `Fire` par tick ; `Poison` : genre `Poison`, M3-T1, crédité à la source, résolu dans la même frame par le résolveur unique),
/// puis retrait des statuts expirés ; un porteur sans statut perd le composant. Après
/// [`apply_projectile_on_hit_system`] (une pose de la frame compte), avant
/// `RollbackSystemSet::CollisionDamage`.
pub fn status_tick_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    library: Res<StatusLibrary>,
    mut damage_events: ResMut<FrameEvents<DamageEvent>>,
    mut carriers: Query<(&GgrsNetId, Entity, &mut Statuses), With<Rollback>>,
) {
    for (net_id, entity, mut statuses) in order_mut_iter!(carriers) {
        for tick in statuses.tick(frame.frame, &library) {
            let tags = Tags::parse(vec!["status".to_string(), tick.id.clone()]);
            damage_events.send(DamageEvent {
                source: tick.source.clone().unwrap_or_else(|| net_id.clone()),
                target: net_id.clone(),
                // M3-T1 : `Poison` inflige du poison, `Burn` du feu
                kind: if tick.status == StatusDef::Poison {
                    DamageKind::Poison
                } else {
                    DamageKind::Fire
                },
                amount: tick.damage,
                frame: frame.frame,
                tags,
                // Source inconnue : le tick touche quand même (règles d'équipe ignorées)
                source_team: tick.source_team.unwrap_or(Team::Enemies),
                friendly_fire: if tick.source_team.is_some() {
                    FriendlyFire::Never
                } else {
                    FriendlyFire::Always
                },
            });
        }
        if statuses.0.is_empty() {
            commands.entity(entity).remove::<Statuses>();
        }
    }
}

/// `Stun` et `Freeze` (T1.3, §19) : vitesse de déplacement remise à zéro chaque frame ;
/// `Freeze` annule aussi le recul. Dans `RollbackSystemSet::Movement`, avant le déplacement.
pub fn status_motion_system(mut query: Query<(&Statuses, &mut crate::actors::Velocity)>) {
    for (statuses, mut velocity) in query.iter_mut() {
        if statuses.incapacitated() {
            velocity.main = FixedVec2::ZERO;
        }
        if statuses.frozen() {
            velocity.knockback = FixedVec2::ZERO;
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
    // -----------------------------------------------------------------------------------
    // M3-T0a (§60) : contrats de larmes et composition
    // -----------------------------------------------------------------------------------

    fn hash_of<T: std::hash::Hash>(value: &T) -> u64 {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Les variantes de contrat sont en fin d'enum : les six variantes d'avant hachent comme
    /// l'enum d'avant M3-T0a (copie locale), le checksum des armes existantes ne bouge pas.
    #[test]
    fn variantes_historiques_gardent_leur_hash() {
        #[derive(Hash)]
        enum Avant {
            Bounce(u32),
            Pierce(u32),
            Size(Fixed),
            Lifetime(u32),
            Homing(Fixed),
            Gravity(Fixed),
        }
        let pairs = [
            (hash_of(&Avant::Bounce(2)), ProjectileModifier::Bounce(2)),
            (hash_of(&Avant::Pierce(3)), ProjectileModifier::Pierce(3)),
            (
                hash_of(&Avant::Size(fx(1.5))),
                ProjectileModifier::Size(fx(1.5)),
            ),
            (
                hash_of(&Avant::Lifetime(9)),
                ProjectileModifier::Lifetime(9),
            ),
            (
                hash_of(&Avant::Homing(fx(0.5))),
                ProjectileModifier::Homing(fx(0.5)),
            ),
            (
                hash_of(&Avant::Gravity(fx(-2.0))),
                ProjectileModifier::Gravity(fx(-2.0)),
            ),
        ];
        for (avant, now) in pairs {
            assert_eq!(avant, hash_of(&now), "{now:?}");
        }
    }

    #[test]
    fn contrats_spectral_split_orbit_ron() {
        let values: Vec<ProjectileModifier> = ron::from_str(
            r#"[Spectral, Split(count: 3, spread: "0.6"), Split(count: 2, spread: "0.3", child: Some("eclat")), Orbit(radius: "40", frames_per_turn: 60)]"#,
        )
        .unwrap();
        assert_eq!(
            values,
            vec![
                ProjectileModifier::Spectral,
                ProjectileModifier::Split {
                    count: 3,
                    spread: fx(0.6),
                    child: None
                },
                ProjectileModifier::Split {
                    count: 2,
                    spread: fx(0.3),
                    child: Some("eclat".into())
                },
                ProjectileModifier::Orbit {
                    radius: fx(40.0),
                    frames_per_turn: 60
                },
            ]
        );
    }

    fn all_kinds() -> Vec<ProjectileModifier> {
        vec![
            ProjectileModifier::Bounce(2),
            ProjectileModifier::Pierce(1),
            ProjectileModifier::Size(fx(1.5)),
            ProjectileModifier::Lifetime(30),
            ProjectileModifier::Homing(fx(0.25)),
            ProjectileModifier::Gravity(fx(-60.0)),
            ProjectileModifier::Spectral,
            ProjectileModifier::Split {
                count: 2,
                spread: fx(0.5),
                child: None,
            },
            ProjectileModifier::Orbit {
                radius: fx(32.0),
                frames_per_turn: 90,
            },
        ]
    }

    /// Identité : sans ajout, `compose` rend exactement `resolve_modifiers`, quelle que soit la
    /// liste (vide, chaque modificateur seul, tous ensemble).
    #[test]
    fn compose_sans_ajout_est_resolve_modifiers() {
        assert_eq!(compose(&[], &[]), resolve_modifiers(&[]));
        for modifier in all_kinds() {
            let base = [modifier];
            assert_eq!(compose(&base, &[]), resolve_modifiers(&base));
        }
        let all = all_kinds();
        assert_eq!(compose(&all, &[]), resolve_modifiers(&all));
        // Des sources vides n'ajoutent rien non plus.
        assert_eq!(compose(&all, &[&[], &[]]), resolve_modifiers(&all));
    }

    #[test]
    fn compose_bounce_et_pierce_s_additionnent() {
        use ProjectileModifier::*;
        let r = compose(
            &[Bounce(1), Pierce(2)],
            &[&[Bounce(2)], &[Pierce(3), Bounce(1)]],
        );
        assert_eq!((r.bounces, r.pierces), (4, 5));
        let r = compose(&[], &[&[Bounce(u32::MAX)], &[Bounce(5)]]);
        assert_eq!(r.bounces, u32::MAX, "addition saturante");
    }

    #[test]
    fn compose_size_produit() {
        use ProjectileModifier::*;
        let r = compose(&[Size(fx(2.0))], &[&[Size(fx(1.5))], &[Size(fx(0.5))]]);
        assert_eq!(r.size, fx(1.5));
        // Une source sans Size compte pour 1.
        assert_eq!(compose(&[Size(fx(2.0))], &[&[Bounce(1)]]).size, fx(2.0));
    }

    #[test]
    fn compose_lifetime_maximum() {
        use ProjectileModifier::*;
        assert_eq!(
            compose(&[Lifetime(30)], &[&[Lifetime(90)], &[Lifetime(60)]]).lifetime,
            Some(90)
        );
        assert_eq!(compose(&[], &[&[Lifetime(40)]]).lifetime, Some(40));
        assert_eq!(compose(&[Lifetime(40)], &[&[Bounce(1)]]).lifetime, Some(40));
    }

    #[test]
    fn compose_homing_somme_plafonnee_a_un() {
        use ProjectileModifier::*;
        assert_eq!(
            compose(&[Homing(fx(0.25))], &[&[Homing(fx(0.5))]]).homing,
            Some(fx(0.75))
        );
        assert_eq!(
            compose(
                &[Homing(fx(0.75))],
                &[&[Homing(fx(0.5))], &[Homing(fx(1.0))]]
            )
            .homing,
            Some(fixed_math::FIXED_ONE)
        );
        assert_eq!(compose(&[], &[&[Homing(fx(0.5))]]).homing, Some(fx(0.5)));
    }

    #[test]
    fn compose_gravity_somme() {
        use ProjectileModifier::*;
        assert_eq!(
            compose(
                &[Gravity(fx(-60.0))],
                &[&[Gravity(fx(20.0))], &[Gravity(fx(-10.0))]]
            )
            .gravity,
            Some(fx(-50.0))
        );
        assert_eq!(compose(&[], &[]).gravity, None);
    }

    #[test]
    fn compose_spectral_orbit_presence_split_additionne() {
        use ProjectileModifier::*;
        let r = compose(&[], &[&[Bounce(1)], &[Spectral], &[Spectral]]);
        assert!(r.spectral);
        let r = compose(
            &[Split {
                count: 2,
                spread: fx(0.5),
                child: None,
            }],
            &[&[Split {
                count: 3,
                spread: fx(1.0),
                child: Some("eclat".into()),
            }]],
        );
        assert_eq!(
            r.split,
            Some(ResolvedSplit {
                count: 5,
                spread: fx(0.5),
                child: None
            }),
            "branches additionnées, forme de la première source"
        );
        let r = compose(
            &[],
            &[
                &[Orbit {
                    radius: fx(30.0),
                    frames_per_turn: 60,
                }],
                &[Orbit {
                    radius: fx(50.0),
                    frames_per_turn: 120,
                }],
            ],
        );
        assert_eq!(
            r.orbit,
            Some(ResolvedOrbit {
                radius: fx(30.0),
                frames_per_turn: 60
            })
        );
    }

    /// L'ordre des sources ne change que ce qui dépend de la « première source » (forme de
    /// `Split`, paramètres d'`Orbit`) ; un passif en double (même source deux fois) compte deux
    /// fois.
    #[test]
    fn compose_ordre_des_sources_et_doublons() {
        use ProjectileModifier::*;
        let a: &[ProjectileModifier] = &[
            Bounce(1),
            Orbit {
                radius: fx(20.0),
                frames_per_turn: 30,
            },
        ];
        let b: &[ProjectileModifier] = &[
            Bounce(2),
            Orbit {
                radius: fx(40.0),
                frames_per_turn: 60,
            },
        ];
        let ab = compose(&[], &[a, b]);
        let ba = compose(&[], &[b, a]);
        assert_eq!(ab.bounces, ba.bounces);
        assert_eq!(ab.orbit.unwrap().radius, fx(20.0));
        assert_eq!(ba.orbit.unwrap().radius, fx(40.0));
        let twice = compose(
            &[Size(fx(1.0))],
            &[&[Size(fx(1.5)), Pierce(1)], &[Size(fx(1.5)), Pierce(1)]],
        );
        assert_eq!((twice.size, twice.pierces), (fx(2.25), 2));
        // L'arme passe avant les objets : sa forme de Split l'emporte.
        let weapon: &[ProjectileModifier] = &[Split {
            count: 1,
            spread: fx(0.2),
            child: Some("arme".into()),
        }];
        let item: &[ProjectileModifier] = &[Split {
            count: 1,
            spread: fx(0.9),
            child: Some("objet".into()),
        }];
        let r = compose(weapon, &[item]);
        assert_eq!(r.split.unwrap().child.as_deref(), Some("arme"));
    }

    /// Dans une même liste (une arme), le dernier l'emporte, comme `resolve_modifiers` : la règle
    /// d'empilement ne vaut qu'entre sources.
    #[test]
    fn compose_dans_une_liste_le_dernier_l_emporte() {
        use ProjectileModifier::*;
        assert_eq!(compose(&[Bounce(1), Bounce(3)], &[]).bounces, 3);
        assert_eq!(compose(&[], &[&[Bounce(1), Bounce(3)]]).bounces, 3);
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

    /// D34 : après le repli dans `[-3π/2, 3π/2]`, `direction_of` est juste à 1e-4 près sur tout
    /// `]-4π, 4π]` (avant : 7e-3 près de 2π, panique au-delà de 2π + 0,03 sans le repli de T1.2),
    /// et le dernier rayon d'une couronne de 24 (6,02 rad) aussi.
    #[test]
    fn direction_of_is_accurate_on_wide_range_d34() {
        let pi = std::f64::consts::PI;
        let mut a = -4.0 * pi + 1.0 / 64.0;
        while a <= 4.0 * pi {
            let angle = Fixed::from_num(a);
            let exact = angle.to_num::<f64>();
            let d = direction_of(angle);
            for (got, want) in [(d.x, exact.cos()), (d.y, exact.sin())] {
                let err = (got.to_num::<f64>() - want).abs();
                assert!(err < 1e-4, "angle {exact} : écart {err}");
            }
            a += 1.0 / 64.0;
        }
        let ring = Pattern::Ring {
            count: 24,
            speed: fx(60.0),
            projectile: "brume".into(),
            every: 0,
        };
        let shots = pattern_shots(&ring, FixedVec2::new(fx(1.0), fx(0.0)), None);
        let last = &shots[23].direction;
        let angle = 23.0 * 2.0 * pi / 24.0;
        assert!((last.x.to_num::<f64>() - angle.cos()).abs() < 1e-4);
        assert!((last.y.to_num::<f64>() - angle.sin()).abs() < 1e-4);
    }

    // ---- M3-T1 (§66) : paires de modificateurs, composition de projectile, chance, orbite ----

    /// Toutes les paires ordonnées de variantes **différentes** : aucune règle ne les lie, donc
    /// `compose([a], [[b]])` est la liste `[a, b]` résolue d'un seul bloc, dans les deux ordres.
    #[test]
    fn compose_deux_variantes_differentes_se_cumulent_sans_interaction() {
        let kinds = all_kinds();
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i == j {
                    continue;
                }
                let composed = compose(std::slice::from_ref(a), &[std::slice::from_ref(b)]);
                assert_eq!(
                    composed,
                    resolve_modifiers(&[a.clone(), b.clone()]),
                    "{} puis {}",
                    a.name(),
                    b.name()
                );
                let swapped = compose(std::slice::from_ref(b), &[std::slice::from_ref(a)]);
                assert_eq!(composed, swapped, "{} / {} commutent", a.name(), b.name());
            }
        }
    }

    /// Toutes les paires d'une **même** variante, une valeur attendue par règle de la table du §60.
    #[test]
    fn compose_deux_variantes_identiques_suivent_leur_regle() {
        use ProjectileModifier::*;
        let pair =
            |m: ProjectileModifier| compose(std::slice::from_ref(&m), &[std::slice::from_ref(&m)]);
        let r = pair(Bounce(2));
        assert_eq!(r.bounces, 4, "Bounce : addition");
        let r = pair(Pierce(1));
        assert_eq!(r.pierces, 2, "Pierce : addition");
        let r = pair(Size(fx(1.5)));
        assert_eq!(r.size, fx(2.25), "Size : produit");
        let r = pair(Lifetime(30));
        assert_eq!(r.lifetime, Some(30), "Lifetime : maximum");
        let r = pair(Homing(fx(0.25)));
        assert_eq!(r.homing, Some(fx(0.5)), "Homing : somme");
        let r = pair(Homing(fx(0.75)));
        assert_eq!(r.homing, Some(fx(1.0)), "Homing : plafonné à 1");
        let r = pair(Gravity(fx(-60.0)));
        assert_eq!(r.gravity, Some(fx(-120.0)), "Gravity : somme");
        let r = pair(Spectral);
        assert!(r.spectral, "Spectral : présence");
        let r = pair(Split {
            count: 2,
            spread: fx(0.5),
            child: None,
        });
        assert_eq!(r.split.unwrap().count, 4, "Split : addition des branches");
        let r = pair(Orbit {
            radius: fx(32.0),
            frames_per_turn: 90,
        });
        assert_eq!(
            r.orbit,
            Some(ResolvedOrbit {
                radius: fx(32.0),
                frames_per_turn: 90
            }),
            "Orbit : présence, paramètres de la première source"
        );
    }

    /// Les trois étages de `tears_stack_three` : rebond (arme), guidage (passif), division (actif).
    #[test]
    fn compose_trois_sources_rebond_guidage_division() {
        use ProjectileModifier::*;
        let r = compose(
            &[Bounce(2)],
            &[
                &[Homing(fx(0.1))],
                &[Split {
                    count: 2,
                    spread: fx(0.4),
                    child: None,
                }],
            ],
        );
        assert_eq!(r.bounces, 2);
        assert_eq!(r.homing, Some(fx(0.1)));
        assert_eq!(r.split.as_ref().map(|s| s.count), Some(2));
        assert_eq!(r.pierces, 0);
    }

    fn spec_of(modifiers: Vec<ProjectileModifier>, on_hit: Vec<Action>) -> ProjectileSpec {
        ProjectileSpec {
            modifiers,
            on_hit,
            on_expire: Vec::new(),
        }
    }

    fn status_action(id: &str) -> Action {
        Action::ApplyStatus {
            status: id.into(),
            stacks: 1,
        }
    }

    /// `Projectile::composed` : modificateurs composés, `on_hit` et `on_expire` ajoutés dans
    /// l'ordre arme puis objets ; sans objet, `Projectile::new` à l'identique (même hash).
    #[test]
    fn projectile_compose_arme_puis_objets_dans_l_ordre() {
        use std::hash::{Hash, Hasher};
        let table = Arc::new(ProjectileTable::new());
        let hash = |p: &Projectile| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            p.hash(&mut hasher);
            hasher.finish()
        };
        let weapon = spec_of(
            vec![ProjectileModifier::Pierce(1)],
            vec![status_action("brulure")],
        );
        let sans = Projectile::composed("larme".into(), &weapon, &[], table.clone(), fx(1.0));
        let direct = Projectile::new("larme".into(), &weapon, table.clone(), fx(1.0), 0);
        assert_eq!(hash(&sans), hash(&direct), "sans objet : chemin d'origine");

        let item_a = spec_of(
            vec![ProjectileModifier::Pierce(1), ProjectileModifier::Spectral],
            vec![status_action("poison")],
        );
        let item_b = ProjectileSpec {
            modifiers: vec![ProjectileModifier::Bounce(1)],
            on_hit: vec![status_action("peur")],
            on_expire: vec![ExpireAction::DestroyTerrain { radius: fx(8.0) }],
        };
        let p = Projectile::composed("larme".into(), &weapon, &[&item_a, &item_b], table, fx(1.0));
        assert_eq!((p.pierces_left, p.bounces_left), (2, 1));
        assert!(p.spectral);
        assert_eq!(
            p.on_hit,
            vec![
                status_action("brulure"),
                status_action("poison"),
                status_action("peur")
            ]
        );
        assert_eq!(p.on_expire.len(), 1);
        assert_ne!(hash(&p), hash(&direct));
    }

    #[test]
    fn hash_du_projectile_distingue_les_champs_de_larmes() {
        use std::hash::{Hash, Hasher};
        let hash = |p: &Projectile| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            p.hash(&mut hasher);
            hasher.finish()
        };
        let base = Projectile::new(
            "a".into(),
            &ProjectileSpec::default(),
            Arc::new(ProjectileTable::new()),
            fx(1.0),
            0,
        );
        let mut spectral = base.clone();
        spectral.spectral = true;
        let mut lucky = base.clone();
        lucky.luck = fx(2.0);
        let mut orbiting = base.clone();
        orbiting.orbit = Some(OrbitState {
            radius: fx(10.0),
            frames_per_turn: 30,
            phase: fx(0.0),
        });
        let hashes = [&base, &spectral, &lucky, &orbiting].map(hash);
        for i in 0..hashes.len() {
            for j in i + 1..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "états {i} et {j} distincts");
            }
        }
        assert_eq!(hash(&base), hash(&base.clone()));
    }

    #[test]
    fn chance_resolue_laisse_le_rang_et_remplace_en_cas_de_succes() {
        let chance = |base: f32, then: Action| Action::Chance {
            base: fx(base),
            per_luck: fx(0.0),
            then: Box::new(then),
        };
        let on_hit = vec![
            status_action("brulure"),
            chance(1.0, status_action("poison")),
            chance(0.0, status_action("peur")),
        ];
        let out = resolve_chances(&on_hit, 7, 42, fx(0.0));
        assert_eq!(out.len(), 3, "rangs conservés");
        assert_eq!(out[0], status_action("brulure"));
        assert_eq!(out[1], status_action("poison"), "chance 1 : toujours");
        assert_eq!(
            out[2], on_hit[2],
            "chance 0 : action laissée, ignorée plus loin"
        );
        // La chance lit la stat Luck : 0,5 + 5 × 0,1 = 1 : toujours, quelle que soit la graine.
        let lucky = vec![Action::Chance {
            base: fx(0.5),
            per_luck: fx(0.125),
            then: Box::new(status_action("poison")),
        }];
        for net_id in 0..50 {
            assert_eq!(
                resolve_chances(&lucky, 7, net_id, fx(4.0))[0],
                status_action("poison")
            );
        }
        // Même graine, même résultat.
        assert_eq!(
            resolve_chances(&lucky, 3, 9, fx(0.0)),
            resolve_chances(&lucky, 3, 9, fx(0.0))
        );
    }

    #[test]
    fn orbite_tourne_autour_du_centre_a_rayon_constant() {
        let orbit = OrbitState {
            radius: fx(40.0),
            frames_per_turn: 60,
            phase: fx(0.0),
        };
        let center = FixedVec2::new(fx(100.0), fx(-20.0));
        // Départ : sur le centre (rayon nul), puis le rayon croît jusqu'à ORBIT_RAMP_FRAMES.
        assert_eq!(orbit_position(center, &orbit, 0), center);
        let ramp_mid = orbit_position(center, &orbit, ORBIT_RAMP_FRAMES / 2);
        let d_mid = ramp_mid.distance(&center);
        assert!(
            (d_mid - fx(20.0)).abs() < fx(0.5),
            "mi-rampe : rayon {d_mid}"
        );
        for age in [ORBIT_RAMP_FRAMES, 15, 30, 45, 59] {
            let at = orbit_position(center, &orbit, age);
            let d = at.distance(&center);
            assert!((d - fx(40.0)).abs() < fx(0.5), "âge {age} : rayon {d}");
        }
        // Un tour exact : retour au point de départ (phase 0 : +x du centre).
        let first = orbit_position(center, &orbit, 60);
        let start = orbit_position(center, &orbit, 0);
        assert!(
            (first.x - (center.x + fx(40.0))).abs() < fx(0.01)
                && (first.y - center.y).abs() < fx(0.01),
            "âge 60 = âge 0 en angle, rayon plein : {first:?}"
        );
        assert_eq!(start, center);
        // Quart de tour : sur +y.
        let quarter = orbit_position(center, &orbit, 15);
        assert!((quarter.x - center.x).abs() < fx(0.5));
        assert!((quarter.y - (center.y + fx(40.0))).abs() < fx(0.5));
    }

    #[test]
    fn orbite_part_du_cote_du_tir() {
        let mut p = Projectile::new(
            "a".into(),
            &spec_of(
                vec![ProjectileModifier::Orbit {
                    radius: fx(30.0),
                    frames_per_turn: 60,
                }],
                vec![],
            ),
            Arc::new(ProjectileTable::new()),
            fx(1.0),
            0,
        );
        p.start_orbit(FixedVec2::new(fx(0.0), fx(2.0)));
        let phase = p.orbit.unwrap().phase;
        assert!(
            (phase - fixed_math::FIXED_PI / fx(2.0)).abs() < fx(0.01),
            "vers +y : {phase}"
        );
        // Sans Orbit : rien.
        let mut plain = Projectile::new(
            "a".into(),
            &ProjectileSpec::default(),
            Arc::new(ProjectileTable::new()),
            fx(1.0),
            0,
        );
        plain.start_orbit(FixedVec2::new(fx(0.0), fx(2.0)));
        assert!(plain.orbit.is_none());
    }
}
