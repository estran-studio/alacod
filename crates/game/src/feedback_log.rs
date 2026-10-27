//! Journal de feedback (T2.13, T1.17, M2-T19, `docs/conventions.md` §7, §31 et §55) : la moitié
//! **simulation** de `feedback.rs` (D69 étape 2, E5b). Aucun asset, aucun `Handle`, aucun type de
//! rendu : `ECS` seul.
//!
//! **Présentation seule dans ses effets** : rien ici n'est rollback, rien n'entre dans le checksum
//! ni dans les traces, et la simulation ne lit jamais [`FeedbackLog`].
//!
//! [`FeedbackLogPlugin`] (toujours actif, headless compris) remplit [`FeedbackLog`] depuis les dégâts
//! et les télégraphes de la frame simulée, avec la configuration [`FeedbackConfigLoaded`] que
//! l'application publie (`feedback::FeedbackAssetsPlugin` : chargeur de `ui/feedback.ron`). C'est la
//! source de vérité : le rendu (`feedback::FeedbackPlugin`) applique ces indices, et
//! `scenario::events` les relit pour en faire des moments clés `feedback` (preuve sans écran).

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::emitter::Emitter;
use combat::projectile::Pattern;
use serde::Serialize;
use sim_core::tag::Tag;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::{frame::FrameCount, net_id::GgrsNetId};

pub use content::feedback::{
    BlankFlashConfig, BossEndConfig, ExplosionFxConfig, FeedbackOverride, FeedbackSettings,
    HitFlashConfig, HitStopScope, PatternTelegraphConfig, ResolvedFeedback, ShakeConfig,
    TelegraphConfig,
};

use crate::character::enemy::ai::state::{BehaviorRuntime, ChargePhase, EnemyAiConfig};
use crate::character::health::Death;
use crate::character::player::{LocalPlayer, Player};
use crate::frame_events::FrameEvents;
use items::{Inventory, ItemTable};
use sim_core::damage::DamageEvent;
use world::{RoomState, RoomStates};

/// Ressource contenant la configuration chargée (absente tant que l'asset ne l'est pas).
/// Publiée par l'application (`feedback::publish_loaded_feedback_config`) ; lue ici et par le rendu.
#[derive(Resource, Debug, Clone)]
pub struct FeedbackConfigLoaded(pub FeedbackSettings);

// ---------------------------------------------------------------------------------------
// Journal (headless compris)
// ---------------------------------------------------------------------------------------

/// Genre d'indice de feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FeedbackKind {
    /// Flash sur les calques de la cible (`value` : frames).
    Flash,
    /// Secousse de caméra, cible = joueur local (`value` : frames, `extra` : amplitude).
    Shake,
    /// Hit stop, source ou cible = joueur local (`value` : frames de rendu, `extra` : 1 pour un
    /// gel de la **cible seule** — `hit_stop_scope: Target`, X-3 —, 0 pour le gel global).
    HitStop,
    /// Début d'un télégraphe au sol (`value` : frames restantes, `extra` : rayon).
    Telegraph,
    /// Chiffre de dégâts au-dessus de la cible (`value` : montant).
    Number,
    /// Forme du pattern annoncé par un télégraphe d'émetteur (M2-T19, `pattern` : la forme,
    /// `value` : frames restantes, `extra` : longueur des rayons).
    PatternTelegraph,
    /// Un joueur a utilisé un blank (M2-T19) : charges de blank en baisse (`value` : handle
    /// du joueur, `extra` : rayon du blank).
    Blank,
    /// Mort d'un boss dont la salle passe `Cleared` (M2-T19) : `net_id` du boss, `position`
    /// du boss, `value` : frames de rendu gelées.
    BossEnd,
    /// Première apparition d'une larme à modificateurs (M3-T17) : `net_id` du projectile,
    /// `note` : « guidée, spectrale, poison ».
    Tear,
    /// Un familier ou orbital apparaît (M3-T17) : `net_id` du familier, `value` : rang,
    /// `extra` : `GgrsNetId` du porteur, `note` : « suiveur » ou « orbital ».
    Familiar,
    /// Un champion (variante à `tint`) apparaît (M3-T17) : `net_id`, `note` : nom de la variante.
    Champion,
    /// Une bombe est posée (M3-T17) : `net_id`, `value` : frames de mèche.
    BombFuse,
    /// Une explosion a eu lieu (M3-T17) : `position`, `extra` : rayon, `value` : 1 si elle
    /// `destroys`, sinon 0.
    Explosion,
    /// Un rocher vient d'être cassé (M3-T17) : `position`, `note` : « x,y » de sa case.
    RockBroken,
    /// Un mur secret vient de s'ouvrir (M3-T17) : `position` de l'explosion qui l'ouvre,
    /// `note` : salle.
    SecretWall,
    /// Un joueur vient de se transformer (suivi Q10, §44) : `net_id` du joueur, `note` : id de la
    /// synergie à `requires_tag`, `color` : sa teinte.
    Transformation,
}

impl FeedbackKind {
    /// Libellé des moments clés `feedback` (`scenario::events`).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Flash => "flash",
            Self::Shake => "secousse",
            Self::HitStop => "hit stop",
            Self::Telegraph => "télégraphe",
            Self::Number => "chiffre",
            Self::PatternTelegraph => "télégraphe",
            Self::Blank => "blank",
            Self::BossEnd => "fin de boss",
            Self::Tear => "larme",
            Self::Familiar => "familier",
            Self::Champion => "champion",
            Self::BombFuse => "mèche",
            Self::Explosion => "explosion",
            Self::RockBroken => "rocher cassé",
            Self::SecretWall => "mur secret",
            Self::Transformation => "transformation",
        }
    }
}

/// Un indice de feedback : ce que le rendu doit montrer, et à quel moment de la simulation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FeedbackCue {
    pub kind: FeedbackKind,
    /// Frame de simulation qui l'a produit.
    pub frame: u32,
    /// `GgrsNetId` de l'entité concernée (cible du coup, porteur du télégraphe).
    pub net_id: usize,
    /// Position monde (cible du coup, centre du télégraphe).
    pub position: Vec2,
    pub value: f32,
    pub extra: f32,
    /// Couleur du flash (multiplicateur linéaire) ; `(1, 1, 1)` pour les autres genres.
    pub color: (f32, f32, f32),
    /// Forme du pattern ([`FeedbackKind::PatternTelegraph`] seulement).
    pub pattern: Option<PatternShape>,
    /// Description libre (M3-T17, §82) : modificateurs d'une larme, rang d'un familier, nom d'une
    /// variante, salle d'un mur secret. `None` pour les genres qui n'en ont pas besoin.
    pub note: Option<String>,
}

/// Nombre d'indices gardés dans [`FeedbackLog::history`].
pub const FEEDBACK_HISTORY_LEN: usize = 2048;

/// Journal de présentation du feedback (T1.17). Hors rollback, hors trace.
///
/// Rempli une fois par **nouvelle** frame de simulation (`FrameCount` plus grand que la
/// dernière frame journalisée) : un tick de rendu sans frame simulée ne produit rien, et les
/// frames rejouées par un rollback ne sont pas journalisées deux fois. Limites connues
/// (présentation, sans conséquence) : un tick qui simule plusieurs frames ne voit que les
/// événements de la dernière (`FrameEvents` n'en garde qu'une) ; un coup prédit puis annulé
/// par un rollback p2p reste dans le journal.
#[derive(Resource, Debug, Default)]
pub struct FeedbackLog {
    /// Indices de la frame simulée pendant ce tick (vide sinon).
    pub frame_cues: Vec<FeedbackCue>,
    /// Derniers indices, du plus ancien au plus récent ([`FEEDBACK_HISTORY_LEN`] au plus).
    pub history: VecDeque<FeedbackCue>,
    last_frame: Option<u32>,
}

impl FeedbackLog {
    pub(crate) fn push(&mut self, cue: FeedbackCue) {
        if self.history.len() == FEEDBACK_HISTORY_LEN {
            self.history.pop_front();
        }
        self.history.push_back(cue.clone());
        self.frame_cues.push(cue);
    }
}

// ---------------------------------------------------------------------------------------
// M2-T19 : fonctions pures (formes de télégraphe, anneau du blank, zoom de fin de boss)
// ---------------------------------------------------------------------------------------

/// Genre de pattern dont la forme est télégraphiée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PatternKind {
    Fan,
    Spread,
    Aimed,
    Spiral,
    Burst,
    Scatter,
    Ring,
}

impl PatternKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Fan => "fan",
            Self::Spread => "spread",
            Self::Aimed => "aimed",
            Self::Spiral => "spiral",
            Self::Burst => "burst",
            Self::Scatter => "scatter",
            Self::Ring => "ring",
        }
    }
}

/// Forme à dessiner pendant un télégraphe : des rayons (angles en radians, depuis `origin`,
/// sur `length` pixels) et, pour un cône (`Burst`, `Scatter`), son axe et son ouverture totale.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PatternShape {
    pub kind: PatternKind,
    pub origin: Vec2,
    pub length: f32,
    pub rays: Vec<f32>,
    /// `(angle de l'axe, ouverture totale)` d'un cône.
    pub cone: Option<(f32, f32)>,
}

/// Angles d'un éventail de `count` rayons répartis sur `spread` autour de `center` (même
/// répartition que `combat::projectile::fan_angles`, en `f32` : présentation seule).
pub fn fan_rays(center: f32, count: u32, spread: f32) -> Vec<f32> {
    if count <= 1 {
        return vec![center; count as usize];
    }
    let step = spread / (count - 1) as f32;
    (0..count)
        .map(|i| center - spread / 2.0 + step * i as f32)
        .collect()
}

/// Longueur (px) des rayons du télégraphe d'une couronne (`Ring`) : courts, autour de l'émetteur.
pub const RING_RAY_LENGTH: f32 = 40.0;

/// Forme du prochain pattern d'un émetteur en télégraphe : la première étape qui tire, avant
/// le prochain `Telegraph`/`Wait`, parmi `Fan`, `Spread`, `Aimed` (rayons), `Spiral` (rayons de
/// la première salve : l'amorce), `Burst` et `Scatter` (cône), `Ring` (D67 : rayons courts
/// dans les directions de tir de la salve). `aim` : visée figée de l'émetteur.
pub fn pattern_shape(
    steps: &[Pattern],
    next_step: usize,
    aim: Vec2,
    origin: Vec2,
    length: f32,
) -> Option<PatternShape> {
    let center = if aim == Vec2::ZERO {
        0.0
    } else {
        aim.y.atan2(aim.x)
    };
    let to_f32 = |value: bevy_fixed::fixed_math::Fixed| value.to_num::<f32>();
    let shape = |kind, rays: Vec<f32>, cone| PatternShape {
        kind,
        origin,
        length,
        rays,
        cone,
    };
    for step in steps.iter().skip(next_step) {
        match step {
            Pattern::Telegraph(_) | Pattern::Wait(_) => return None,
            Pattern::Fan { count, spread, .. } => {
                return Some(shape(
                    PatternKind::Fan,
                    fan_rays(center, *count, to_f32(*spread)),
                    None,
                ))
            }
            Pattern::Spread { count, spread, .. } => {
                return Some(shape(
                    PatternKind::Spread,
                    fan_rays(center, *count, to_f32(*spread)),
                    None,
                ))
            }
            Pattern::Aimed { count, spread, .. } => {
                return Some(shape(
                    PatternKind::Aimed,
                    fan_rays(center, *count, to_f32(*spread)),
                    None,
                ))
            }
            // D67 : la salve en couronne se dit par des rayons **courts** autour de l'émetteur,
            // dans les directions de tir (départ = visée, pas TAU / count : `combat::projectile`),
            // pas par un grand cercle qui ressemble à une zone de dégâts.
            Pattern::Ring { count, .. } => {
                let arm = std::f32::consts::TAU / (*count).max(1) as f32;
                let rays = (0..*count).map(|i| center + arm * i as f32).collect();
                let mut ring = shape(PatternKind::Ring, rays, None);
                ring.length = length.min(RING_RAY_LENGTH);
                return Some(ring);
            }
            Pattern::Spiral { count, .. } => {
                let arm = std::f32::consts::TAU / (*count).max(1) as f32;
                let rays = (0..*count).map(|i| center + arm * i as f32).collect();
                return Some(shape(PatternKind::Spiral, rays, None));
            }
            Pattern::Burst { spread, .. } => {
                let spread = to_f32(*spread);
                return Some(shape(
                    PatternKind::Burst,
                    vec![center - spread / 2.0, center + spread / 2.0],
                    Some((center, spread)),
                ));
            }
            Pattern::Scatter { spread, .. } => {
                let spread = to_f32(*spread);
                return Some(shape(
                    PatternKind::Scatter,
                    vec![center - spread / 2.0, center + spread / 2.0],
                    Some((center, spread)),
                ));
            }
            _ => {}
        }
    }
    None
}

/// Libellé du moment clé d'une forme : « fan 31 (5 rayons) », « burst 31 (cône 0.60 rad) ».
pub fn pattern_label(net_id: usize, shape: &PatternShape) -> String {
    match shape.cone {
        Some((_, spread)) => format!("{} {} (cône {:.2} rad)", shape.kind.name(), net_id, spread),
        None => format!(
            "{} {} ({} rayons)",
            shape.kind.name(),
            net_id,
            shape.rays.len()
        ),
    }
}

/// Rayon de l'anneau du blank `age` frames après l'usage : s'étend vite puis ralentit
/// (ease-out) jusqu'à `radius`, `None` une fois `frames` écoulées.
pub fn blank_ring_radius(radius: f32, age: u32, frames: u32) -> Option<f32> {
    if frames == 0 || age >= frames {
        return None;
    }
    let remaining = 1.0 - age as f32 / frames as f32;
    Some(radius * (1.0 - remaining * remaining))
}

/// Opacité d'un flash `age` frames après son départ : linéaire de `alpha` à 0 sur `frames`.
pub fn flash_alpha(alpha: f32, age: u32, frames: u32) -> f32 {
    if frames == 0 || age >= frames {
        return 0.0;
    }
    alpha * (1.0 - age as f32 / frames as f32)
}

/// Poids du zoom de fin de boss (0 à 1) : monte pendant le premier cinquième de `frames`, puis
/// redescend en douceur à 0.
pub fn boss_zoom_weight(age: u32, frames: u32) -> f32 {
    if frames == 0 || age >= frames {
        return 0.0;
    }
    let p = age as f32 / frames as f32;
    if p < 0.2 {
        p / 0.2
    } else {
        1.0 - (p - 0.2) / 0.8
    }
}

/// Facteur de zoom avant (≥ 1) à l'instant `age` : `1 + (zoom - 1) × poids`.
pub fn boss_zoom_factor(zoom: f32, age: u32, frames: u32) -> f32 {
    1.0 + (zoom - 1.0).max(0.0) * boss_zoom_weight(age, frames)
}

/// Cercle de télégraphe au sol, dérivé de l'état de simulation (lecture seule).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelegraphShape {
    pub center: Vec2,
    pub radius: f32,
    /// Frames avant le tir ou la ruée.
    pub remaining: u32,
}

/// Portée de tir (`Shoot.range`) d'un ennemi à distance, en pixels.
pub fn ranged_range(ai: Option<&EnemyAiConfig>) -> Option<f32> {
    ai?.ranged.as_ref().map(|r| r.range.to_num::<f32>())
}

/// Télégraphe en cours d'un personnage : phase `Telegraph` d'une `Charge` (cercle au point
/// visé, figé, de rayon `attack_range`), sinon pas `Telegraph` d'un émetteur (cercle autour
/// du porteur, de rayon la portée du tir annoncé).
pub fn telegraph_shape(
    now: u32,
    position: Vec2,
    emitter: Option<&Emitter>,
    charge: Option<(&ChargePhase, &EnemyAiConfig)>,
    ranged_range: Option<f32>,
) -> Option<TelegraphShape> {
    if let Some((ChargePhase::Telegraph { until, target }, ai_config)) = charge {
        return Some(TelegraphShape {
            center: Vec2::new(target.0.to_num::<f32>(), target.1.to_num::<f32>()),
            radius: ai_config.attack_range.to_num::<f32>(),
            remaining: until.saturating_sub(now),
        });
    }
    let emitter = emitter?;
    let remaining = emitter.telegraphing()?;
    // D57 : la portée du tir annoncé, pas la plus grande de la table (le télégraphe couvrait la
    // carte entière pour `roi_rat`) ; repli sur la table si le pattern n'est pas résolu.
    // et bornée par la portée de tir de l'ennemi (`Shoot.range`, 260 px pour `roi_rat`) : la
    // zone lisible, pas la distance parcourue par la balle.
    let radius = emitter
        .telegraphed_range()
        .or_else(|| emitter.max_range())
        .map_or(0.0, |range| range.to_num::<f32>());
    let radius = ranged_range.map_or(radius, |limit| radius.min(limit));
    Some(TelegraphShape {
        center: position,
        radius,
        remaining,
    })
}

/// Arme d'un coup, pour `by_weapon` : `DamageEvent` ne la porte pas (il est tracé : lui
/// ajouter un champ ferait bouger toutes les traces). Approximation (limite connue, §31) :
/// l'émetteur de la source s'il en a un (`Emitter.weapon`), sinon l'arme active de son
/// inventaire **au moment du coup** (une balle tirée avant un changement d'arme est
/// attribuée à la nouvelle) ; aucune pour la mêlée (tag `melee`).
fn weapon_of_hit(
    event: &DamageEvent,
    emitter: Option<&Emitter>,
    inventory: Option<&crate::weapons::WeaponInventory>,
) -> Option<String> {
    if event.tags.has(&Tag::new("melee")) {
        return None;
    }
    if let Some(emitter) = emitter {
        return Some(emitter.weapon.clone());
    }
    inventory
        .and_then(|inventory| inventory.weapons.get(inventory.active_weapon_index))
        .map(|(_, weapon)| weapon.config.name.clone())
}

/// Plugin du journal de feedback, actif en headless comme en rendu. Sans asset : la configuration
/// vient de [`FeedbackConfigLoaded`], publiée par `feedback::FeedbackAssetsPlugin`.
pub struct FeedbackLogPlugin;

impl Plugin for FeedbackLogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeedbackLog>();
        app.add_systems(
            PostUpdate,
            (
                record_feedback_cues,
                // M3-T17 (§82) : après le journal v1/v2, qui vide `frame_cues` à chaque tick.
                crate::feedback_v3::record_v3_cues.after(record_feedback_cues),
            ),
        );
    }
}

/// Une entité rollback vue par le journal : position, joueur local, de quoi deviner l'arme.
type CueSubject<'a> = (
    &'a GgrsNetId,
    &'a FixedTransform3D,
    Has<LocalPlayer>,
    Option<&'a crate::weapons::WeaponInventory>,
    Option<&'a Emitter>,
    Option<&'a BehaviorRuntime>,
    Option<&'a EnemyAiConfig>,
);

/// Remplit [`FeedbackLog`] pour la frame simulée pendant ce tick (rien si aucune nouvelle
/// frame). Par coup : flash sur la cible ; chiffre si `damage_numbers`. Par frame, au plus
/// une secousse (cible = joueur local, ou tireur = joueur local avec une secousse de
/// surcharge `by_weapon`/`by_kind`) et un hit stop (source ou cible = joueur local, au moins
/// une frame), le plus fort des coups de la frame. Par télégraphe : un indice à son début.
fn record_feedback_cues(
    frame: Res<FrameCount>,
    damages: Res<FrameEvents<DamageEvent>>,
    config: Option<Res<FeedbackConfigLoaded>>,
    mut log: ResMut<FeedbackLog>,
    subjects: Query<CueSubject>,
    mut telegraphing: Local<BTreeSet<usize>>,
    // M2-T19 : mémoires de présentation (état d'avant, pour détecter une baisse ou une disparition).
    t19: T19Inputs,
    mut memory: Local<T19Memory>,
) {
    log.frame_cues.clear();
    let now = frame.frame;
    match log.last_frame {
        Some(last) if now == last => return,
        // Nouvelle partie : le compteur de frames repart de zéro.
        Some(last) if now < last => {
            telegraphing.clear();
            *memory = T19Memory::default();
        }
        _ => {}
    }
    log.last_frame = Some(now);

    let by_net_id: BTreeMap<usize, _> = subjects.iter().map(|s| (s.0 .0, s)).collect();
    let position_of = |net_id: usize| {
        by_net_id.get(&net_id).map(|(_, transform, ..)| {
            Vec2::new(
                transform.translation.x.to_num::<f32>(),
                transform.translation.y.to_num::<f32>(),
            )
        })
    };
    let is_local = |net_id: usize| by_net_id.get(&net_id).is_some_and(|s| s.2);

    if let Some(config) = &config {
        let settings = &config.0;
        // Une secousse et un hit stop par frame au plus (le plus fort), même quand plusieurs
        // coups tombent ensemble (éclats d'une grenade) : (cible, position, frames, amplitude).
        let mut shake: Option<(usize, Vec2, u32, f32)> = None;
        let mut hit_stop: Option<(usize, Vec2, u32, HitStopScope)> = None;
        for event in damages.iter() {
            let target = event.target.0;
            let source = by_net_id.get(&event.source.0);
            let weapon = weapon_of_hit(event, source.and_then(|s| s.4), source.and_then(|s| s.3));
            let resolved = settings.resolve(&event.kind, weapon.as_deref());
            let position = position_of(target).unwrap_or(Vec2::ZERO);
            log.push(FeedbackCue {
                kind: FeedbackKind::Flash,
                frame: now,
                net_id: target,
                position,
                value: resolved.hit_flash.frames as f32,
                extra: 0.0,
                color: resolved.hit_flash.color,
                pattern: None,
                note: None,
            });
            if settings.damage_numbers {
                log.push(FeedbackCue {
                    kind: FeedbackKind::Number,
                    frame: now,
                    net_id: target,
                    position,
                    value: event.amount.to_num::<f32>(),
                    extra: 0.0,
                    color: (1.0, 1.0, 1.0),
                    pattern: None,
                    note: None,
                });
            }
            let local_hit = is_local(target);
            let local_shooter = is_local(event.source.0);
            if (local_hit || (local_shooter && resolved.shake_overridden))
                && shake.is_none_or(|(_, _, _, amplitude)| resolved.shake.amplitude > amplitude)
            {
                shake = Some((
                    target,
                    position,
                    resolved.shake.frames,
                    resolved.shake.amplitude,
                ));
            }
            // Gel de la cible seule : jamais le joueur local (un coup qu'il reçoit ne fige personne).
            let target_only = resolved.hit_stop_scope == HitStopScope::Target;
            if resolved.hit_stop_frames > 0
                && (local_hit || local_shooter)
                && !(target_only && local_hit)
                && hit_stop.is_none_or(|(_, _, frames, _)| resolved.hit_stop_frames > frames)
            {
                hit_stop = Some((
                    target,
                    position,
                    resolved.hit_stop_frames,
                    resolved.hit_stop_scope,
                ));
            }
        }
        if let Some((net_id, position, frames, amplitude)) = shake {
            log.push(FeedbackCue {
                kind: FeedbackKind::Shake,
                frame: now,
                net_id,
                position,
                value: frames as f32,
                extra: amplitude,
                color: (1.0, 1.0, 1.0),
                pattern: None,
                note: None,
            });
        }
        if let Some((net_id, position, frames, scope)) = hit_stop {
            log.push(FeedbackCue {
                kind: FeedbackKind::HitStop,
                frame: now,
                net_id,
                position,
                value: frames as f32,
                extra: if scope == HitStopScope::Target {
                    1.0
                } else {
                    0.0
                },
                color: (1.0, 1.0, 1.0),
                pattern: None,
                note: None,
            });
        }
    }

    record_t19_cues(&mut log, now, config.as_deref(), &t19, &mut memory);

    let mut current = BTreeSet::new();
    let mut current_patterns = BTreeSet::new();
    for (net_id, transform, _, _, emitter, runtime, ai_config) in by_net_id.values() {
        let position = Vec2::new(
            transform.translation.x.to_num::<f32>(),
            transform.translation.y.to_num::<f32>(),
        );
        let charge = runtime.zip(*ai_config).map(|(r, ai)| (&r.charge, ai));
        let Some(shape) =
            telegraph_shape(now, position, *emitter, charge, ranged_range(*ai_config))
        else {
            continue;
        };
        current.insert(net_id.0);
        if !telegraphing.contains(&net_id.0) {
            log.push(FeedbackCue {
                kind: FeedbackKind::Telegraph,
                frame: now,
                net_id: net_id.0,
                position: shape.center,
                value: shape.remaining as f32,
                extra: shape.radius,
                color: (1.0, 1.0, 1.0),
                pattern: None,
                note: None,
            });
        }
        // M2-T19 : la forme du pattern annoncé, une fois par télégraphe (clé : émetteur et
        // étape, un même émetteur enchaîne plusieurs télégraphes).
        let enabled = config
            .as_ref()
            .is_none_or(|config| config.0.pattern_telegraph.enabled);
        if let (true, Some(emitter)) = (enabled, *emitter) {
            let key = (net_id.0, emitter.next_step);
            current_patterns.insert(key);
            if !memory.patterns.contains(&key) {
                let aim = Vec2::new(emitter.aim.x.to_num::<f32>(), emitter.aim.y.to_num::<f32>());
                if let Some(pattern) = pattern_shape(
                    &emitter.steps,
                    emitter.next_step as usize,
                    aim,
                    position,
                    shape.radius,
                ) {
                    log.push(FeedbackCue {
                        kind: FeedbackKind::PatternTelegraph,
                        frame: now,
                        net_id: net_id.0,
                        position,
                        value: shape.remaining as f32,
                        extra: shape.radius,
                        color: (1.0, 1.0, 1.0),
                        pattern: Some(pattern),
                        note: None,
                    });
                }
            }
        }
    }
    *telegraphing = current;
    memory.patterns = current_patterns;
}

/// Ce que [`record_feedback_cues`] lit en plus pour le blank et la fin de boss (M2-T19).
#[derive(bevy::ecs::system::SystemParam)]
struct T19Inputs<'w, 's> {
    players: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static Player,
            &'static Inventory,
            &'static FixedTransform3D,
        ),
    >,
    items: Option<Res<'w, ItemTable>>,
    bosses: Query<
        'w,
        's,
        (&'static GgrsNetId, &'static FixedTransform3D),
        (With<crate::boss::BossPlan>, Without<Death>),
    >,
    rooms: Option<Res<'w, RoomStates>>,
}

/// Mémoire d'un tick à l'autre : tout se dérive de l'état, jamais d'un événement.
#[derive(Default)]
struct T19Memory {
    /// Télégraphes de forme déjà journalisés, par (émetteur, étape).
    patterns: BTreeSet<(usize, u32)>,
    /// Charges de blank par joueur au tick précédent.
    blanks: BTreeMap<usize, u32>,
    /// Boss vivants au tick précédent, avec leur position.
    bosses: BTreeMap<usize, Vec2>,
    /// Nombre de salles `Cleared` au tick précédent.
    cleared: usize,
}

/// Blank utilisé (charges en baisse) et fin de boss (boss disparu **et** une salle vient de
/// passer `Cleared`). Rien n'est lu d'un événement : un rollback qui rejoue la même situation
/// donne les mêmes indices.
fn record_t19_cues(
    log: &mut FeedbackLog,
    now: u32,
    config: Option<&FeedbackConfigLoaded>,
    inputs: &T19Inputs,
    memory: &mut T19Memory,
) {
    // Blank : l'objet blank est le premier de la table (ordre des ids) à porter la donnée.
    let blank_item = inputs.items.as_ref().and_then(|table| {
        table
            .0
            .iter()
            .find_map(|(id, def)| def.blank.map(|b| (id, b)))
    });
    let mut blanks = BTreeMap::new();
    if let Some((id, spec)) = blank_item {
        for (net_id, player, inventory, transform) in &inputs.players {
            let charges = inventory.consumable(id);
            blanks.insert(net_id.0, charges);
            let before = memory.blanks.get(&net_id.0).copied().unwrap_or(charges);
            if charges < before && config.is_none_or(|config| config.0.blank.enabled) {
                log.push(FeedbackCue {
                    kind: FeedbackKind::Blank,
                    frame: now,
                    net_id: net_id.0,
                    position: Vec2::new(
                        transform.translation.x.to_num::<f32>(),
                        transform.translation.y.to_num::<f32>(),
                    ),
                    value: player.handle as f32,
                    extra: spec.radius.to_num::<f32>(),
                    color: (1.0, 1.0, 1.0),
                    pattern: None,
                    note: None,
                });
            }
        }
    }
    memory.blanks = blanks;

    // Fin de boss.
    let cleared = inputs.rooms.as_ref().map_or(0, |rooms| {
        rooms
            .rooms
            .values()
            .filter(|state| **state == RoomState::Cleared)
            .count()
    });
    let alive: BTreeMap<usize, Vec2> = inputs
        .bosses
        .iter()
        .map(|(net_id, transform)| {
            (
                net_id.0,
                Vec2::new(
                    transform.translation.x.to_num::<f32>(),
                    transform.translation.y.to_num::<f32>(),
                ),
            )
        })
        .collect();
    if cleared > memory.cleared {
        if let Some(boss_end) = config
            .map(|config| config.0.boss_end)
            .or_else(|| Some(BossEndConfig::default()))
            .filter(|boss_end| boss_end.enabled)
        {
            for (net_id, position) in &memory.bosses {
                if !alive.contains_key(net_id) {
                    log.push(FeedbackCue {
                        kind: FeedbackKind::BossEnd,
                        frame: now,
                        net_id: *net_id,
                        position: *position,
                        value: boss_end.freeze_frames as f32,
                        extra: boss_end.zoom,
                        color: boss_end.flash_color,
                        pattern: None,
                        note: None,
                    });
                }
            }
        }
    }
    memory.cleared = cleared;
    memory.bosses = alive;
}

#[cfg(test)]
mod telegraph_tests {
    use super::*;
    use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
    use combat::projectile::{Pattern, ProjectileTable};
    use sim_core::damage::FriendlyFire;

    fn emitter(pattern: Pattern) -> Emitter {
        let table: ProjectileTable = ron::from_str(
            r#"{"court": (damage: "1.0", speed: "100.0", range: "60.0"),
                "long": (damage: "1.0", speed: "100.0", range: "150.0")}"#,
        )
        .unwrap();
        Emitter::new(
            "test",
            &pattern,
            "arsenal",
            std::sync::Arc::new(table),
            fixed_math::FIXED_ONE,
            FriendlyFire::Never,
            FixedVec2::new(fixed_math::FIXED_ONE, Fixed::ZERO),
            0,
        )
    }

    fn fixed(value: f32) -> Fixed {
        fixed_math::new(value)
    }

    #[test]
    fn forme_fan_rayons_repartis_autour_de_la_visee() {
        let steps = vec![
            Pattern::Telegraph(20),
            Pattern::Fan {
                count: 5,
                spread: fixed(1.0),
                speed: Fixed::ZERO,
                projectile: "p".into(),
            },
        ];
        let shape = pattern_shape(&steps, 1, Vec2::X, Vec2::new(2.0, 3.0), 90.0).unwrap();
        assert_eq!(shape.kind, PatternKind::Fan);
        assert_eq!(shape.rays.len(), 5);
        assert!((shape.rays[0] + 0.5).abs() < 1e-5 && (shape.rays[4] - 0.5).abs() < 1e-5);
        assert!(shape.rays[2].abs() < 1e-5);
        assert_eq!(
            (shape.origin, shape.length, shape.cone),
            (Vec2::new(2.0, 3.0), 90.0, None)
        );
        assert_eq!(pattern_label(31, &shape), "fan 31 (5 rayons)");
        // Visée vers le haut : l'éventail tourne avec elle.
        let up = pattern_shape(&steps, 1, Vec2::Y, Vec2::ZERO, 10.0).unwrap();
        assert!((up.rays[2] - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn forme_spirale_amorce_et_burst_cone() {
        let spiral = vec![Pattern::Spiral {
            count: 4,
            step: fixed(0.3),
            shots: 10,
            every: 3,
            speed: Fixed::ZERO,
            projectile: "p".into(),
        }];
        let shape = pattern_shape(&spiral, 0, Vec2::X, Vec2::ZERO, 50.0).unwrap();
        assert_eq!(shape.kind, PatternKind::Spiral);
        assert_eq!(shape.rays.len(), 4);
        assert!((shape.rays[1] - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        let burst = vec![Pattern::Burst {
            count: 3,
            per_burst: 3,
            interval: 5,
            spread: fixed(0.5),
            projectile: "p".into(),
        }];
        let shape = pattern_shape(&burst, 0, Vec2::X, Vec2::ZERO, 50.0).unwrap();
        assert_eq!(shape.kind, PatternKind::Burst);
        assert_eq!(shape.rays.len(), 2);
        let (axis, spread) = shape.cone.unwrap();
        assert!(axis.abs() < 1e-5 && (spread - 0.5).abs() < 1e-3);
        assert_eq!(pattern_label(7, &shape), "burst 7 (cône 0.50 rad)");
    }

    #[test]
    fn forme_ring_rayons_courts_dans_les_directions_de_tir() {
        let ring = Pattern::Ring {
            count: 8,
            speed: fixed(60.0),
            projectile: "p".into(),
            every: 0,
        };
        let shape = pattern_shape(&[ring.clone()], 0, Vec2::X, Vec2::new(1.0, 2.0), 260.0).unwrap();
        assert_eq!(shape.kind, PatternKind::Ring);
        assert_eq!(shape.rays.len(), 8);
        assert_eq!(shape.length, RING_RAY_LENGTH);
        assert_eq!(shape.cone, None);
        assert!((shape.rays[2] - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert_eq!(pattern_label(220, &shape), "ring 220 (8 rayons)");
        // Départ sur la visée.
        let up = pattern_shape(&[ring], 0, Vec2::Y, Vec2::ZERO, 10.0).unwrap();
        assert!((up.rays[0] - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert_eq!(up.length, 10.0);
    }

    #[test]
    fn forme_absente_apres_une_etape_bloquante() {
        let ring = Pattern::Ring {
            count: 8,
            speed: fixed(60.0),
            projectile: "p".into(),
            every: 0,
        };
        let fan = Pattern::Fan {
            count: 3,
            spread: fixed(1.0),
            speed: Fixed::ZERO,
            projectile: "p".into(),
        };
        // Le prochain tir est derrière un autre télégraphe : ce n'est pas celui qu'on annonce.
        let steps = vec![ring, Pattern::Wait(5), fan];
        assert_eq!(pattern_shape(&steps, 1, Vec2::X, Vec2::ZERO, 10.0), None);
    }

    #[test]
    fn anneau_du_blank_s_etend_jusqu_au_rayon_puis_disparait() {
        assert_eq!(blank_ring_radius(160.0, 0, 24), Some(0.0));
        let mid = blank_ring_radius(160.0, 12, 24).unwrap();
        assert!(mid > 80.0 && mid < 160.0, "ease-out : {mid}");
        let mut last = 0.0;
        for age in 0..24 {
            let radius = blank_ring_radius(160.0, age, 24).unwrap();
            assert!(radius >= last && radius <= 160.0);
            last = radius;
        }
        assert_eq!(blank_ring_radius(160.0, 24, 24), None);
        assert_eq!(blank_ring_radius(160.0, 0, 0), None);
        assert_eq!(flash_alpha(0.5, 0, 10), 0.5);
        assert_eq!(flash_alpha(0.5, 5, 10), 0.25);
        assert_eq!(flash_alpha(0.5, 10, 10), 0.0);
    }

    #[test]
    fn zoom_de_fin_de_boss_monte_puis_revient_a_un() {
        assert_eq!(boss_zoom_factor(1.5, 0, 60), 1.0);
        let peak = boss_zoom_factor(1.5, 12, 60);
        assert!((peak - 1.5).abs() < 1e-5);
        let mut last = peak;
        for age in 13..60 {
            let factor = boss_zoom_factor(1.5, age, 60);
            assert!(factor <= last && factor >= 1.0);
            last = factor;
        }
        assert_eq!(boss_zoom_factor(1.5, 60, 60), 1.0);
        // Un zoom < 1 est ramené à l'identité (jamais d'éloignement).
        assert_eq!(boss_zoom_factor(0.5, 12, 60), 1.0);
    }

    #[test]
    fn charge_cercle_au_point_vise_rayon_de_contact() {
        let ai = EnemyAiConfig {
            attack_range: fixed_math::new(40.0),
            ..Default::default()
        };
        let phase = ChargePhase::Telegraph {
            until: 90,
            target: (fixed_math::new(100.0), fixed_math::new(-20.0)),
        };
        let shape =
            telegraph_shape(70, Vec2::new(5.0, 5.0), None, Some((&phase, &ai)), None).unwrap();
        assert_eq!(shape.center, Vec2::new(100.0, -20.0));
        assert_eq!(shape.radius, 40.0);
        assert_eq!(shape.remaining, 20);
        assert_eq!(
            telegraph_shape(70, Vec2::ZERO, None, Some((&ChargePhase::Idle, &ai)), None),
            None
        );
    }

    #[test]
    fn emetteur_cercle_autour_du_porteur_rayon_de_la_plus_longue_portee() {
        let mut e = emitter(Pattern::Sequence(vec![
            Pattern::Telegraph(30),
            Pattern::Named("x".into()),
        ]));
        let mut never = || -> Fixed { panic!() };
        assert!(telegraph_shape(0, Vec2::ZERO, Some(&e), None, None).is_none());
        let _ = e.tick(1, &mut never);
        let shape = telegraph_shape(1, Vec2::new(3.0, 4.0), Some(&e), None, None).unwrap();
        assert_eq!(shape.center, Vec2::new(3.0, 4.0));
        assert_eq!(shape.radius, 150.0);
        assert_eq!(shape.remaining, 30);
    }

    #[test]
    fn emetteur_rayon_de_la_portee_du_tir_annonce() {
        // Deux salves : la première annoncée (courte), la seconde après un autre télégraphe.
        let mut e = emitter(Pattern::Sequence(vec![
            Pattern::Telegraph(30),
            Pattern::Ring {
                count: 8,
                speed: fixed_math::new(60.0),
                projectile: "court".into(),
                every: 0,
            },
            Pattern::Telegraph(20),
            Pattern::Aimed {
                count: 1,
                spread: Fixed::ZERO,
                projectile: "long".into(),
            },
        ]));
        let mut never = || -> Fixed { panic!() };
        let _ = e.tick(1, &mut never);
        let shape = telegraph_shape(1, Vec2::ZERO, Some(&e), None, None).unwrap();
        assert_eq!(shape.radius, 60.0);
        // Bornée par la portée de tir de l'ennemi.
        let bounded = telegraph_shape(1, Vec2::ZERO, Some(&e), None, Some(40.0)).unwrap();
        assert_eq!(bounded.radius, 40.0);
    }
}
