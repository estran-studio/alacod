//! Feedback v1 (T2.13, étendu en T1.17, `docs/conventions.md` §7 et §31) : flash à l'impact,
//! secousse de caméra, hit stop, chiffres de dégâts, cercle de télégraphe, sons.
//!
//! **Présentation seule** : rien ici n'est rollback, rien n'entre dans le checksum ni dans les
//! traces, et la simulation ne lit jamais ces composants ou ressources.
//!
//! Deux plugins :
//! - [`FeedbackLogPlugin`] (toujours actif, headless compris) : charge `ui/feedback.ron` et
//!   remplit [`FeedbackLog`] depuis les dégâts et les télégraphes de la frame simulée. C'est la
//!   source de vérité : le rendu applique ces indices, et `scenario::events` les relit pour en
//!   faire des moments clés `feedback` (preuve sans écran).
//! - [`FeedbackPlugin`] (rendu seulement, `PresentationPlugin`) : applique les indices (flash,
//!   secousse, hit stop, chiffres), dessine les télégraphes et joue les sons.

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D};
use bevy_kira_audio::prelude::*;
use combat::emitter::Emitter;
use combat::projectile::Pattern;
use serde::{Deserialize, Serialize};
use sim_core::tag::Tag;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use utils::{frame::FrameCount, net_id::GgrsNetId};

pub use content::feedback::{
    BlankFlashConfig, BossEndConfig, FeedbackOverride, FeedbackSettings, HitFlashConfig,
    PatternTelegraphConfig, ResolvedFeedback, ShakeConfig, TelegraphConfig,
};

use crate::character::enemy::ai::state::{BehaviorRuntime, ChargePhase, EnemyAiConfig};
use crate::character::health::Death;
use crate::character::player::{LocalPlayer, Player};
use crate::core::is_headless;
use crate::frame_events::FrameEvents;
use items::{Inventory, ItemTable};
use sim_core::damage::DamageEvent;
use world::{RoomState, RoomStates};

/// Asset `ui/feedback.ron` : enveloppe du type partagé avec le lint
/// (`content::feedback::FeedbackSettings`).
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FeedbackConfig(pub FeedbackSettings);

/// Ressource contenant la configuration chargée (absente tant que l'asset ne l'est pas).
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
    /// Hit stop, source ou cible = joueur local (`value` : frames de rendu).
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
    fn push(&mut self, cue: FeedbackCue) {
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

/// Forme du prochain pattern d'un émetteur en télégraphe : la première étape qui tire, avant
/// le prochain `Telegraph`/`Wait`, parmi `Fan`, `Spread`, `Aimed` (rayons), `Spiral` (rayons de
/// la première salve : l'amorce), `Burst` et `Scatter` (cône). Un `Ring` n'a pas de forme
/// propre : le cercle du télégraphe v1 le dit déjà. `aim` : visée figée de l'émetteur.
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
fn ranged_range(ai: Option<&EnemyAiConfig>) -> Option<f32> {
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

/// Plugin du journal de feedback, actif en headless comme en rendu.
pub struct FeedbackLogPlugin;

impl Plugin for FeedbackLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<FeedbackConfig>::new(&["ron"]));
        app.init_resource::<FeedbackLog>();
        // Charger la config au démarrage
        app.add_systems(Startup, load_feedback_config);
        // Publie `FeedbackConfigLoaded` une fois l'asset chargé (et à chaque modification) :
        // sans ce système, aucun effet ne s'appliquait jamais (config toujours absente).
        app.add_systems(Update, publish_loaded_feedback_config);
        app.add_systems(PostUpdate, record_feedback_cues);
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
        let mut hit_stop: Option<(usize, Vec2, u32)> = None;
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
            if resolved.hit_stop_frames > 0
                && (local_hit || local_shooter)
                && hit_stop.is_none_or(|(_, _, frames)| resolved.hit_stop_frames > frames)
            {
                hit_stop = Some((target, position, resolved.hit_stop_frames));
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
            });
        }
        if let Some((net_id, position, frames)) = hit_stop {
            log.push(FeedbackCue {
                kind: FeedbackKind::HitStop,
                frame: now,
                net_id,
                position,
                value: frames as f32,
                extra: 0.0,
                color: (1.0, 1.0, 1.0),
                pattern: None,
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
                    });
                }
            }
        }
    }
    memory.cleared = cleared;
    memory.bosses = alive;
}

/// Ressource intermédiaire : handle du fichier config.
#[derive(Resource)]
struct FeedbackConfigHandle(Handle<FeedbackConfig>);

/// Système de démarrage : charge la FeedbackConfig.
fn load_feedback_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config_handle: Handle<FeedbackConfig> = asset_server.load("ui/feedback.ron");
    commands.insert_resource(FeedbackConfigHandle(config_handle));
}

/// Copie l'asset chargé dans la ressource `FeedbackConfigLoaded` (une fois, puis à chaque
/// `AssetEvent::Modified` pour le rechargement à chaud). Les systèmes d'effets lisent cette
/// ressource ; tant qu'elle est absente, ils ne font rien.
fn publish_loaded_feedback_config(
    mut commands: Commands,
    handle: Option<Res<FeedbackConfigHandle>>,
    assets: Res<Assets<FeedbackConfig>>,
    loaded: Option<Res<FeedbackConfigLoaded>>,
    mut events: MessageReader<AssetEvent<FeedbackConfig>>,
) {
    let Some(handle) = handle else {
        return;
    };
    let modified = events
        .read()
        .any(|event| matches!(event, AssetEvent::Modified { .. }));
    if loaded.is_none() || modified {
        if let Some(config) = assets.get(&handle.0) {
            commands.insert_resource(FeedbackConfigLoaded(config.0.clone()));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Rendu
// ---------------------------------------------------------------------------------------

/// Composant non-rollback : flash en cours sur les calques d'un personnage.
#[derive(Component, Debug)]
pub struct HitFlash {
    pub until_frame: u32,
    pub color: Color,
}

/// Composant non-rollback appliqué à la caméra pour une secousse.
#[derive(Component, Debug)]
pub struct CameraShake {
    pub until_frame: u32,
    pub frames: u32,
    pub amplitude: f32,
    /// Décalage appliqué au tick précédent, retiré avant d'appliquer le suivant : sans ça,
    /// les décalages s'additionnent dès que le suivi de caméra ne les écrase plus (hit stop).
    pub applied: Vec3,
}

/// Chiffre de dégâts flottant (présentation).
#[derive(Component, Debug)]
pub struct DamageNumber {
    pub born_secs: f64,
}

/// Durée de vie d'un chiffre de dégâts et sa vitesse de montée.
const DAMAGE_NUMBER_SECS: f64 = 0.7;
const DAMAGE_NUMBER_RISE: f32 = 30.0;

/// Effets de présentation en cours (M2-T19) : anneaux de blank, flash plein écran, zoom de fin
/// de boss. Dérivés des indices du journal ; la simulation ne les lit jamais.
#[derive(Resource, Default)]
pub struct FeedbackFx {
    rings: Vec<BlankRing>,
    flashes: Vec<ScreenFlashFx>,
    boss: Option<BossZoomFx>,
    /// Décalage appliqué à la caméra au tick précédent (retiré avant le suivi de caméra).
    boss_applied: Option<(f32, Vec3)>,
}

struct BlankRing {
    frame: u32,
    center: Vec2,
    radius: f32,
    frames: u32,
    color: Color,
}

struct ScreenFlashFx {
    frame: u32,
    frames: u32,
    alpha: f32,
    color: (f32, f32, f32),
}

struct BossZoomFx {
    frame: u32,
    frames: u32,
    zoom: f32,
    center: Vec2,
}

/// Voile plein écran du flash (blank, fin de boss).
#[derive(Component)]
struct ScreenFlashNode;

/// Plugin de rendu du feedback (`PresentationPlugin`, absent en headless).
pub struct FeedbackPlugin;

impl Plugin for FeedbackPlugin {
    fn build(&self, app: &mut App) {
        // Pas de plugin si headless
        if is_headless() {
            return;
        }

        app.init_resource::<FeedbackFx>();
        app.add_systems(Startup, spawn_screen_flash);
        // Retire le zoom du tick précédent avant que le suivi de caméra ne relise la caméra.
        app.add_systems(
            Update,
            undo_boss_zoom.before(crate::camera::camera_control_system),
        );
        app.add_systems(
            PostUpdate,
            (
                draw_pattern_shapes,
                draw_blank_rings,
                update_screen_flash.after(apply_feedback_cues),
                apply_boss_zoom
                    .after(apply_feedback_cues)
                    .before(bevy::transform::TransformSystems::Propagate),
            ),
        );
        app.add_systems(
            PostUpdate,
            (
                apply_feedback_cues.after(record_feedback_cues),
                apply_layer_colors.after(apply_feedback_cues),
                cleanup_expired_hit_flash.after(apply_layer_colors),
                // Avant la propagation des transforms : sinon le décalage n'est appliqué au
                // `Transform` qu'après le calcul du `GlobalTransform` rendu, et le suivi de
                // caméra (`Update`) l'écrase à la frame suivante : la secousse n'est jamais vue.
                update_camera_shake_offset
                    .after(apply_feedback_cues)
                    .before(bevy::transform::TransformSystems::Propagate),
                animate_damage_numbers,
                draw_telegraphs,
                play_shot_sound_for_new_bullets,
                play_reload_sound_for_reload_events,
                play_hit_sound,
                play_hurt_sound,
                play_death_sound,
                play_dash_sound,
                play_blank_sound,
                play_door_sound,
            ),
        );
    }
}

/// Applique les indices de la frame : flash (sur l'entité racine, les calques sont colorés
/// par [`apply_layer_colors`]), secousse, hit stop, chiffres. Les télégraphes sont dessinés
/// depuis l'état ([`draw_telegraphs`]).
#[allow(clippy::too_many_arguments)]
fn apply_feedback_cues(
    mut commands: Commands,
    log: Res<FeedbackLog>,
    time: Res<Time<Real>>,
    asset_server: Res<AssetServer>,
    targets: Query<(Entity, &GgrsNetId)>,
    mut cameras: Query<(Entity, Option<&mut CameraShake>), With<crate::camera::GameCamera>>,
    mut freeze: ResMut<animation::AnimationFreeze>,
    fog: Res<crate::room_fog::FogView>,
    fog_map: Option<Res<crate::room_fog::FogMap>>,
    config: Option<Res<FeedbackConfigLoaded>>,
    mut fx: ResMut<FeedbackFx>,
) {
    if log.frame_cues.is_empty() {
        return;
    }
    let settings = config.as_ref().map(|config| &config.0);
    let blank = settings.map(|s| s.blank).unwrap_or_default();
    let boss_end = settings.map(|s| s.boss_end).unwrap_or_default();
    let entity_of: BTreeMap<usize, Entity> = targets
        .iter()
        .map(|(entity, net_id)| (net_id.0, entity))
        .collect();
    for cue in &log.frame_cues {
        info!(
            "feedback f{} {} {} ({})",
            cue.frame,
            cue.kind.label(),
            cue.net_id,
            cue.value
        );
        match cue.kind {
            FeedbackKind::Flash => {
                if let Some(entity) = entity_of.get(&cue.net_id) {
                    let (r, g, b) = cue.color;
                    commands.entity(*entity).insert(HitFlash {
                        until_frame: cue.frame + cue.value as u32,
                        color: Color::linear_rgb(r, g, b),
                    });
                }
            }
            FeedbackKind::Shake => {
                // La caméra de jeu (`GameCamera`, suivi des joueurs), pas n'importe quelle
                // `Camera` : en capture il en existe plusieurs.
                if let Some((camera, shake)) = cameras.iter_mut().next() {
                    let until_frame = cue.frame + cue.value as u32;
                    match shake {
                        Some(mut shake) => {
                            shake.until_frame = until_frame;
                            shake.frames = cue.value as u32;
                            shake.amplitude = cue.extra;
                        }
                        None => {
                            commands.entity(camera).insert(CameraShake {
                                until_frame,
                                frames: cue.value as u32,
                                amplitude: cue.extra,
                                applied: Vec3::ZERO,
                            });
                        }
                    }
                }
            }
            FeedbackKind::HitStop => freeze.freeze_for(cue.value as u32),
            FeedbackKind::Number => {
                if !fog.sees(fog_map.as_deref(), cue.position) {
                    continue;
                }
                commands.spawn((
                    DamageNumber {
                        born_secs: time.elapsed_secs_f64(),
                    },
                    Text2d::new(format!("{}", cue.value.round() as i64)),
                    TextFont {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf").into(),
                        font_size: FontSize::Px(10.0),
                        ..Default::default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(cue.position.x, cue.position.y + 12.0, 50.0),
                ));
            }
            FeedbackKind::Telegraph | FeedbackKind::PatternTelegraph => {}
            FeedbackKind::Blank => {
                let (r, g, b, a) = blank.color;
                fx.rings.push(BlankRing {
                    frame: cue.frame,
                    center: cue.position,
                    radius: cue.extra,
                    frames: blank.ring_frames,
                    color: Color::srgba(r, g, b, a),
                });
                if blank.flash_alpha > 0.0 {
                    fx.flashes.push(ScreenFlashFx {
                        frame: cue.frame,
                        frames: blank.ring_frames,
                        alpha: blank.flash_alpha,
                        color: (r, g, b),
                    });
                }
            }
            FeedbackKind::BossEnd => {
                // Gel de **rendu** (comme le hit stop) : la simulation et `Time<Virtual>` ne
                // sont pas touchés.
                freeze.freeze_for(boss_end.freeze_frames);
                fx.boss = Some(BossZoomFx {
                    frame: cue.frame,
                    frames: boss_end.zoom_frames,
                    zoom: boss_end.zoom,
                    center: cue.position,
                });
                if boss_end.flash_alpha > 0.0 {
                    fx.flashes.push(ScreenFlashFx {
                        frame: cue.frame,
                        frames: boss_end.zoom_frames,
                        alpha: boss_end.flash_alpha,
                        color: cue.color,
                    });
                }
            }
        }
    }
}

/// Voile plein écran, transparent au repos.
fn spawn_screen_flash(mut commands: Commands) {
    commands.spawn((
        ScreenFlashNode,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(900),
    ));
}

/// Opacité du voile : le plus fort des flashs en cours ; retire les flashs finis.
fn update_screen_flash(
    frame: Res<FrameCount>,
    mut fx: ResMut<FeedbackFx>,
    mut nodes: Query<&mut BackgroundColor, With<ScreenFlashNode>>,
) {
    let now = frame.frame;
    fx.flashes
        .retain(|flash| now >= flash.frame && now - flash.frame < flash.frames);
    let strongest = fx
        .flashes
        .iter()
        .map(|flash| {
            (
                flash_alpha(flash.alpha, now - flash.frame, flash.frames),
                flash.color,
            )
        })
        .max_by(|a, b| a.0.total_cmp(&b.0));
    for mut background in &mut nodes {
        background.0 = match strongest {
            Some((alpha, (r, g, b))) => Color::srgba(r, g, b, alpha),
            None => Color::NONE,
        };
    }
}

/// Anneau du blank : s'étend jusqu'au rayon du blank et s'estompe.
fn draw_blank_rings(mut gizmos: Gizmos, frame: Res<FrameCount>, mut fx: ResMut<FeedbackFx>) {
    let now = frame.frame;
    fx.rings
        .retain(|ring| now >= ring.frame && now - ring.frame < ring.frames);
    for ring in &fx.rings {
        let age = now - ring.frame;
        if let Some(radius) = blank_ring_radius(ring.radius, age, ring.frames) {
            let fade = 1.0 - age as f32 / ring.frames as f32;
            gizmos.circle_2d(
                Isometry2d::from_translation(ring.center),
                radius,
                ring.color.with_alpha(ring.color.alpha() * fade),
            );
        }
    }
}

/// Retire le décalage de zoom appliqué au tick précédent (avant le suivi de caméra).
fn undo_boss_zoom(
    mut fx: ResMut<FeedbackFx>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<crate::camera::GameCamera>>,
) {
    let Some((ratio, delta)) = fx.boss_applied.take() else {
        return;
    };
    for (mut transform, mut projection) in &mut cameras {
        transform.translation -= delta;
        if let Projection::Orthographic(orthographic) = projection.as_mut() {
            orthographic.scale /= ratio;
        }
    }
}

/// Zoom sur la position du boss mort : rapproche l'échelle de projection et décale la caméra
/// vers le boss, selon [`boss_zoom_weight`]. Présentation seule (`Transform`, `Projection`).
fn apply_boss_zoom(
    frame: Res<FrameCount>,
    mut fx: ResMut<FeedbackFx>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<crate::camera::GameCamera>>,
) {
    let now = frame.frame;
    let Some(boss) = &fx.boss else {
        return;
    };
    if now < boss.frame || now - boss.frame >= boss.frames {
        fx.boss = None;
        return;
    }
    let age = now - boss.frame;
    let factor = boss_zoom_factor(boss.zoom, age, boss.frames);
    let weight = boss_zoom_weight(age, boss.frames);
    let (center, ratio) = (boss.center, 1.0 / factor);
    let mut applied = None;
    for (mut transform, mut projection) in &mut cameras {
        let Projection::Orthographic(orthographic) = projection.as_mut() else {
            continue;
        };
        orthographic.scale *= ratio;
        let delta = ((center - transform.translation.truncate()) * weight).extend(0.0);
        transform.translation += delta;
        applied = Some((ratio, delta));
    }
    fx.boss_applied = applied;
}

/// Rayons, amorce ou cône du pattern annoncé, pendant le `Telegraph` d'un émetteur.
fn draw_pattern_shapes(
    mut gizmos: Gizmos,
    frame: Res<FrameCount>,
    config: Option<Res<FeedbackConfigLoaded>>,
    subjects: Query<(&FixedTransform3D, &Emitter, Option<&EnemyAiConfig>)>,
    fog: Res<crate::room_fog::FogView>,
    fog_map: Option<Res<crate::room_fog::FogMap>>,
) {
    let settings = config.as_ref().map(|config| &config.0);
    let pattern = settings.map(|s| s.pattern_telegraph).unwrap_or_default();
    if !pattern.enabled {
        return;
    }
    let (r, g, b, a) = pattern
        .color
        .or_else(|| settings.map(|s| s.telegraph.color))
        .unwrap_or(TelegraphConfig::default().color);
    for (transform, emitter, ai_config) in &subjects {
        let position = Vec2::new(
            transform.translation.x.to_num::<f32>(),
            transform.translation.y.to_num::<f32>(),
        );
        if !fog.sees(fog_map.as_deref(), position) {
            continue;
        }
        let Some(shape) = telegraph_shape(
            frame.frame,
            position,
            Some(emitter),
            None,
            ranged_range(ai_config),
        ) else {
            continue;
        };
        let aim = Vec2::new(emitter.aim.x.to_num::<f32>(), emitter.aim.y.to_num::<f32>());
        let Some(pattern_shape) = pattern_shape(
            &emitter.steps,
            emitter.next_step as usize,
            aim,
            position,
            shape.radius,
        ) else {
            continue;
        };
        let color = Color::srgba(r, g, b, a);
        for angle in &pattern_shape.rays {
            let end = position + Vec2::from_angle(*angle) * pattern_shape.length;
            gizmos.line_2d(position, end, color);
        }
        if let Some((center, spread)) = pattern_shape.cone {
            let points = (0..=12).map(|i| {
                let angle = center - spread / 2.0 + spread * i as f32 / 12.0;
                position + Vec2::from_angle(angle) * pattern_shape.length
            });
            gizmos.linestrip_2d(points, color.with_alpha(a * 0.6));
        }
    }
}

/// Teinte d'un statut (T1.3, `docs/conventions.md` §19) : le plus contraignant gagne
/// (gel, étourdi, brûlure, lenteur).
fn status_tint(statuses: &combat::status::Statuses) -> Color {
    use combat::status::StatusDef;
    let has = |kind: StatusDef| statuses.0.iter().any(|entry| entry.status == kind);
    if has(StatusDef::Freeze) {
        Color::srgb(0.55, 0.8, 1.0)
    } else if has(StatusDef::Stun) {
        Color::srgb(1.0, 1.0, 0.45)
    } else if has(StatusDef::Burn) {
        Color::srgb(1.0, 0.55, 0.35)
    } else if has(StatusDef::Slow) {
        Color::srgb(0.6, 0.6, 0.9)
    } else {
        Color::WHITE
    }
}

/// Couleur des calques (sprites enfants) de chaque personnage, relue chaque tick depuis
/// l'état : flash en cours, sinon teinte du statut le plus contraignant (T1.3), sinon blanc.
/// T1.17 : le flash vise les calques **enfants** (l'ancien système cherchait un `Sprite` sur
/// l'entité racine, qui n'en a pas : aucun flash n'était jamais visible).
fn apply_layer_colors(
    frame: Res<FrameCount>,
    characters: Query<
        (
            &Children,
            Option<&HitFlash>,
            Option<&combat::status::Statuses>,
        ),
        With<combat::actors::Health>,
    >,
    mut sprites: Query<&mut Sprite>,
) {
    for (layers, flash, statuses) in &characters {
        let color = match flash {
            Some(flash) if frame.frame < flash.until_frame => flash.color,
            _ => statuses.map(status_tint).unwrap_or(Color::WHITE),
        };
        for layer in layers.iter() {
            if let Ok(mut sprite) = sprites.get_mut(layer) {
                if sprite.color != color {
                    sprite.color = color;
                }
            }
        }
    }
}

/// Retire les flashs expirés (la couleur est remise par [`apply_layer_colors`]).
fn cleanup_expired_hit_flash(
    mut commands: Commands,
    query: Query<(Entity, &HitFlash)>,
    frame_count: Res<FrameCount>,
) {
    for (entity, flash) in query.iter() {
        if frame_count.frame >= flash.until_frame {
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

/// Décalage de caméra de la secousse : motif déterministe indexé par frame, amplitude
/// décroissante sur la durée. Retire d'abord le décalage du tick précédent ; retire le
/// composant à l'échéance.
fn update_camera_shake_offset(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut CameraShake)>,
    frame_count: Res<FrameCount>,
) {
    let frame = frame_count.frame;
    for (entity, mut transform, mut shake) in query.iter_mut() {
        transform.translation -= shake.applied;
        shake.applied = Vec3::ZERO;
        if frame >= shake.until_frame {
            commands.entity(entity).remove::<CameraShake>();
            continue;
        }
        let remaining = (shake.until_frame - frame) as f32;
        let strength = (remaining / shake.frames.max(1) as f32).min(1.0);
        let sign = if frame % 2 == 0 { 1.0 } else { -1.0 };
        let offset = shake.amplitude * strength * sign;
        shake.applied = if (frame / 2) % 2 == 0 {
            Vec3::new(offset, 0.0, 0.0)
        } else {
            Vec3::new(0.0, offset, 0.0)
        };
        transform.translation += shake.applied;
    }
}

/// Fait monter et disparaître les chiffres de dégâts (temps réel).
fn animate_damage_numbers(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut numbers: Query<(Entity, &DamageNumber, &mut Transform, &mut TextColor)>,
    fog: Res<crate::room_fog::FogView>,
    fog_map: Option<Res<crate::room_fog::FogMap>>,
) {
    let now = time.elapsed_secs_f64();
    for (entity, number, mut transform, mut color) in numbers.iter_mut() {
        let age = now - number.born_secs;
        if age >= DAMAGE_NUMBER_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += DAMAGE_NUMBER_RISE * time.delta_secs();
        let alpha = if fog.sees(fog_map.as_deref(), transform.translation.truncate()) {
            1.0 - (age / DAMAGE_NUMBER_SECS) as f32
        } else {
            0.0
        };
        color.0 = color.0.with_alpha(alpha);
    }
}

/// Cercle de télégraphe au sol : contour au rayon complet, disque intérieur qui grandit
/// jusqu'au déclenchement. Relu chaque tick depuis l'état (lecture seule).
fn draw_telegraphs(
    mut gizmos: Gizmos,
    frame: Res<FrameCount>,
    config: Option<Res<FeedbackConfigLoaded>>,
    subjects: Query<(
        &GgrsNetId,
        &FixedTransform3D,
        Option<&Emitter>,
        Option<&BehaviorRuntime>,
        Option<&EnemyAiConfig>,
    )>,
    mut started: Local<BTreeMap<usize, u32>>,
    fog: Res<crate::room_fog::FogView>,
    fog_map: Option<Res<crate::room_fog::FogMap>>,
) {
    let (r, g, b, a) = config
        .map(|config| config.0.telegraph.color)
        .unwrap_or(TelegraphConfig::default().color);
    let color = Color::srgba(r, g, b, a);
    let mut seen = BTreeMap::new();
    for (net_id, transform, emitter, runtime, ai_config) in &subjects {
        let position = Vec2::new(
            transform.translation.x.to_num::<f32>(),
            transform.translation.y.to_num::<f32>(),
        );
        if !fog.sees(fog_map.as_deref(), position) {
            continue;
        }
        let charge = runtime.zip(ai_config).map(|(r, ai)| (&r.charge, ai));
        let Some(shape) = telegraph_shape(
            frame.frame,
            position,
            emitter,
            charge,
            ranged_range(ai_config),
        ) else {
            continue;
        };
        // Durée totale : frames restantes au premier tick vu.
        let total = *started.get(&net_id.0).unwrap_or(&shape.remaining);
        seen.insert(net_id.0, total.max(shape.remaining));
        let progress = 1.0 - shape.remaining as f32 / total.max(1) as f32;
        let iso = Isometry2d::from_translation(shape.center);
        gizmos.circle_2d(iso, shape.radius, color);
        gizmos.circle_2d(iso, shape.radius * progress, color.with_alpha(a * 0.5));
    }
    *started = seen;
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
    fn forme_absente_pour_ring_et_apres_une_etape_bloquante() {
        let ring = Pattern::Ring {
            count: 8,
            speed: fixed(60.0),
            projectile: "p".into(),
            every: 0,
        };
        assert_eq!(
            pattern_shape(&[ring.clone()], 0, Vec2::X, Vec2::ZERO, 10.0),
            None
        );
        let fan = Pattern::Fan {
            count: 3,
            spread: fixed(1.0),
            speed: Fixed::ZERO,
            projectile: "p".into(),
        };
        // Le prochain tir est derrière un autre télégraphe : ce n'est pas celui qu'on annonce.
        let steps = vec![ring, Pattern::Wait(5), fan];
        assert_eq!(pattern_shape(&steps, 0, Vec2::X, Vec2::ZERO, 10.0), None);
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

/// Frames pendant lesquelles un tir déjà joué reste mémorisé (`played`, voir
/// `play_shot_sound_for_new_bullets`).
const SHOT_SOUND_MEMORY_FRAMES: u32 = 120;

/// Clé de `sounds` du son de tir d'un mode de tir (lot A, `docs/plan-assets.md`) : la famille
/// d'arme se déduit du mode actif de l'arme (`Shotgun` : fusil à pompe ; `Automatic`/`Burst` :
/// mitraillette ; `Manual` : pistolet).
fn shot_sound_key(mode: &crate::weapons::FiringMode) -> &'static str {
    use crate::weapons::FiringMode;
    match mode {
        FiringMode::Shotgun { .. } => "shot_shotgun",
        FiringMode::Automatic {} | FiringMode::Burst { .. } => "shot_smg",
        FiringMode::Manual {} => "shot_pistol",
    }
}

/// Son de tir : la clé imposée à l'arme (`shot_sounds`), sinon celle de la famille de son mode
/// de tir, sinon `shot` (repli), sinon aucun son.
fn shot_sound_path<'a>(
    settings: &'a FeedbackSettings,
    weapon: Option<&str>,
    mode: Option<&crate::weapons::FiringMode>,
) -> Option<&'a String> {
    let sounds = &settings.sounds;
    weapon
        .and_then(|name| settings.shot_sounds.get(name))
        .and_then(|key| sounds.get(key))
        .or_else(|| mode.and_then(|mode| sounds.get(shot_sound_key(mode))))
        .or_else(|| sounds.get("shot"))
}

/// Nom et mode de tir actif de l'arme active de cet inventaire.
fn active_weapon_mode<'a>(
    inventory: &'a crate::weapons::WeaponInventory,
    weapon_states: &Query<&crate::weapons::WeaponState>,
) -> Option<(&'a str, Option<&'a crate::weapons::FiringMode>)> {
    let (entity, weapon) = inventory.weapons.get(inventory.active_weapon_index)?;
    let mode = weapon_states.get(*entity).ok().and_then(|state| {
        weapon
            .config
            .firing_modes
            .get(&state.active_mode)
            .map(|mode| &mode.firing_mode)
    });
    Some((weapon.config.name.as_str(), mode))
}

/// Système jouant le son de tir pour chaque nouveau tir d'un joueur local.
///
/// Un son par (tireur, frame de création) : le rollback de la session locale détruit puis
/// recrée les balles des dernières frames à chaque image rendue, elles redeviennent
/// `Added<Bullet>` et rejoueraient leur son ; un fusil à pompe ne joue pas un son par plomb.
/// Le son est celui de l'arme active du tireur **à l'image rendue** (`shot_sounds`, sinon la
/// famille de son mode actif) : la balle ne porte pas son arme, lui ajouter un champ ferait
/// bouger les traces.
#[allow(clippy::too_many_arguments)]
fn play_shot_sound_for_new_bullets(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    config: Option<Res<FeedbackConfigLoaded>>,
    local_players: Query<(&GgrsNetId, &crate::weapons::WeaponInventory), With<LocalPlayer>>,
    weapon_states: Query<&crate::weapons::WeaponState>,
    new_bullets: Query<&crate::weapons::Bullet, Added<crate::weapons::Bullet>>,
    frame_count: Res<FrameCount>,
    mut played: Local<BTreeSet<(usize, u32)>>,
) {
    // Oublie les tirs anciens, et ceux d'une partie précédente (le compteur repart de 0).
    let frame = frame_count.frame;
    played.retain(|&(_, created_at)| {
        created_at <= frame && frame - created_at <= SHOT_SOUND_MEMORY_FRAMES
    });

    let Some(cfg) = config else {
        return;
    };
    for bullet in new_bullets.iter() {
        let Some((_, inventory)) = local_players
            .iter()
            .find(|(net_id, _)| net_id.0 == bullet.source.0)
        else {
            continue;
        };
        if !played.insert((bullet.source.0, bullet.created_at)) {
            continue;
        }
        let active = active_weapon_mode(inventory, &weapon_states);
        let weapon = active.map(|(name, _)| name);
        let mode = active.and_then(|(_, mode)| mode);
        let Some(sound_path) = shot_sound_path(&cfg.0, weapon, mode) else {
            continue;
        };
        info!("feedback f{} sound {}", frame, sound_path);
        audio.play(asset_server.load(sound_path));
    }
}

#[cfg(test)]
mod shot_sound_tests {
    use super::*;
    use crate::weapons::FiringMode;

    #[test]
    fn arme_puis_famille_puis_repli_sur_shot() {
        let shotgun = FiringMode::Shotgun {
            pellet_count: 8,
            spread_angle: Fixed::ZERO,
        };
        let automatic = FiringMode::Automatic {};
        let mut settings: FeedbackSettings = ron::from_str(
            r#"(
                hit_flash: (frames: 4, color: (1.0, 1.0, 1.0)),
                shake: (frames: 8, amplitude: 12.0),
                sounds: {
                    "shot": "repli.ogg",
                    "shot_pistol": "pistolet.ogg",
                    "shot_shotgun": "pompe.ogg",
                    "shot_smg": "smg.ogg",
                },
                shot_sounds: {"pistolet": "shot_pistol", "muet": "absente"},
            )"#,
        )
        .unwrap();
        let path = |s: &FeedbackSettings, weapon, mode| {
            shot_sound_path(s, weapon, mode).map(|p| p.to_string())
        };
        // La clé imposée à l'arme l'emporte sur la famille de son mode.
        assert_eq!(
            path(&settings, Some("pistolet"), Some(&automatic)).as_deref(),
            Some("pistolet.ogg")
        );
        // Clé imposée absente de `sounds` (refusée par le lint) : la famille reprend.
        assert_eq!(
            path(&settings, Some("muet"), Some(&shotgun)).as_deref(),
            Some("pompe.ogg")
        );
        assert_eq!(
            path(&settings, Some("mitraillette"), Some(&automatic)).as_deref(),
            Some("smg.ogg")
        );
        // Famille absente : repli sur `shot` ; mode inconnu aussi.
        settings.sounds.remove("shot_smg");
        assert_eq!(
            path(&settings, None, Some(&automatic)).as_deref(),
            Some("repli.ogg")
        );
        assert_eq!(path(&settings, None, None).as_deref(), Some("repli.ogg"));
        // Ni la famille ni `shot` : aucun son.
        settings.sounds.clear();
        assert_eq!(path(&settings, None, Some(&shotgun)), None);
    }
}

/// Système jouant le son de rechargement quand un joueur local commence à recharger.
///
/// Déclenché une fois, au passage « ne recharge pas » → « recharge » d'une image rendue à
/// l'autre : `reloading_ending_frame` reste `Some` pendant tout le rechargement, et le jouer à
/// chaque image superposait le même son des dizaines de fois.
fn play_reload_sound_for_reload_events(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    config: Option<Res<FeedbackConfigLoaded>>,
    query: Query<(&crate::weapons::WeaponInventory, &GgrsNetId), With<LocalPlayer>>,
    frame_count: Res<FrameCount>,
    mut reloading: Local<BTreeSet<usize>>,
) {
    let now: BTreeSet<usize> = query
        .iter()
        .filter(|(inv, _)| inv.reloading_ending_frame.is_some())
        .map(|(_, net_id)| net_id.0)
        .collect();
    let started = now.difference(&reloading).count();
    *reloading = now;

    let Some(cfg) = config else {
        return;
    };
    let Some(sound_path) = cfg.0.sounds.get("reload") else {
        return;
    };
    for _ in 0..started {
        info!("feedback f{} sound reload", frame_count.frame);
        audio.play(asset_server.load(sound_path));
    }
}

// ---------------------------------------------------------------------------------------
// Sons d'événements (lot B de `docs/plan-assets.md`) : présentation seule, dérivés de l'état
// ou des `FrameEvents` de la dernière frame simulée, jamais écrits dans la simulation.
// ---------------------------------------------------------------------------------------

/// Lecteur des sons de `ui/feedback.ron` par clé.
#[derive(bevy::ecs::system::SystemParam)]
struct FeedbackSounds<'w> {
    audio: Res<'w, Audio>,
    asset_server: Res<'w, AssetServer>,
    config: Option<Res<'w, FeedbackConfigLoaded>>,
    frame: Res<'w, FrameCount>,
}

impl FeedbackSounds<'_> {
    /// Joue le son `key` (rien si la config n'est pas chargée ou si la clé manque).
    fn play(&self, key: &str) {
        let Some(path) = self.config.as_ref().and_then(|cfg| cfg.0.sounds.get(key)) else {
            return;
        };
        info!("feedback f{} sound {}", self.frame.frame, key);
        self.audio.play(self.asset_server.load(path));
    }
}

/// `hit` : un coup porté par un joueur local à une autre entité, un son par frame simulée
/// (les plombs d'un fusil à pompe ou une explosion n'en jouent qu'un). Gardé par la frame de
/// l'événement : une image rendue sans nouvelle frame simulée voit encore les mêmes
/// événements. L'événement précède la résolution : un coup annulé (invulnérabilité) sonne.
fn play_hit_sound(
    sounds: FeedbackSounds,
    damage: Res<FrameEvents<DamageEvent>>,
    local_players: Query<&GgrsNetId, With<LocalPlayer>>,
    mut last_frame: Local<Option<u32>>,
) {
    let local: BTreeSet<usize> = local_players.iter().map(|n| n.0).collect();
    let Some(hit) = damage
        .iter()
        .find(|e| local.contains(&e.source.0) && !local.contains(&e.target.0))
    else {
        return;
    };
    if *last_frame == Some(hit.frame) {
        return;
    }
    *last_frame = Some(hit.frame);
    sounds.play("hit");
}

/// Écart minimal, en frames simulées, entre deux sons `hurt` : les PV d'un joueur peuvent
/// baisser deux frames de suite (vu en gungeon au contact d'un ennemi, frames 522 et 523), et le
/// même son rejoué à 16 ms d'écart s'entend comme un écho.
const HURT_SOUND_MIN_FRAMES: u32 = 15;

/// `hurt` : la santé d'un joueur local baisse d'une image rendue à l'autre, à `max` égal (une
/// baisse de `max` qui rabote `current` n'est pas un coup). Au plus un son toutes les
/// [`HURT_SOUND_MIN_FRAMES`] frames.
fn play_hurt_sound(
    sounds: FeedbackSounds,
    players: Query<(&GgrsNetId, &crate::character::health::Health), With<LocalPlayer>>,
    mut previous: Local<BTreeMap<usize, (Fixed, Fixed)>>,
    mut last_played: Local<Option<u32>>,
) {
    let now: BTreeMap<usize, (Fixed, Fixed)> = players
        .iter()
        .map(|(net_id, health)| (net_id.0, (health.current, health.max)))
        .collect();
    let hurt = now.iter().any(|(net_id, (current, max))| {
        previous
            .get(net_id)
            .is_some_and(|(before, max_before)| current < before && max == max_before)
    });
    *previous = now;
    let frame = sounds.frame.frame;
    // Une frame plus petite que la dernière jouée : nouvelle partie.
    let ready =
        last_played.is_none_or(|last| frame < last || frame - last >= HURT_SOUND_MIN_FRAMES);
    if hurt && ready {
        *last_played = Some(frame);
        sounds.play("hurt");
    }
}

/// `death` : un ennemi tué. `Death` est posé puis l'entité est détruite par
/// `despawn_rollback` : désactivée (`RollbackDespawned`), elle n'est vue que par une requête
/// qui nomme ce composant, jusqu'à la confirmation de sa frame. Un son par image, quel que
/// soit le nombre de morts ; chaque ennemi une fois (le rollback local la ressuscite puis la
/// redétruit dans la même image).
fn play_death_sound(
    sounds: FeedbackSounds,
    dead: Query<
        &GgrsNetId,
        (
            With<crate::character::enemy::Enemy>,
            With<crate::character::health::Death>,
            With<bevy_ggrs::RollbackDespawned>,
        ),
    >,
    mut played: Local<BTreeSet<usize>>,
) {
    let now: BTreeSet<usize> = dead.iter().map(|n| n.0).collect();
    let new_death = now.difference(&played).next().is_some();
    *played = now;
    if new_death {
        sounds.play("death");
    }
}

/// `dash` : un joueur local commence un dash (front montant de `DashState::is_dashing`).
fn play_dash_sound(
    sounds: FeedbackSounds,
    players: Query<(&GgrsNetId, &crate::character::dash::DashState), With<LocalPlayer>>,
    mut dashing: Local<BTreeSet<usize>>,
) {
    let now: BTreeSet<usize> = players
        .iter()
        .filter(|(_, dash)| dash.is_dashing)
        .map(|(net_id, _)| net_id.0)
        .collect();
    let started = now.difference(&dashing).next().is_some();
    *dashing = now;
    if started {
        sounds.play("dash");
    }
}

/// `blank` : un joueur local utilise un blank (`FrameEvents<BlankUsed>`, émis seulement s'il
/// lui en restait un), gardé par la frame de l'événement comme `hit`.
fn play_blank_sound(
    sounds: FeedbackSounds,
    blanks: Option<Res<FrameEvents<items::BlankUsed>>>,
    local_players: Query<&crate::character::player::Player, With<LocalPlayer>>,
    mut last_frame: Local<Option<u32>>,
) {
    let Some(blanks) = blanks else {
        return;
    };
    let local: BTreeSet<usize> = local_players.iter().map(|p| p.handle).collect();
    let Some(blank) = blanks.iter().find(|b| local.contains(&b.player_handle)) else {
        return;
    };
    if *last_frame == Some(blank.frame) {
        return;
    }
    *last_frame = Some(blank.frame);
    sounds.play("blank");
}

/// `door` : une porte vue fermée (avec collider) à l'image précédente est ouverte (porte
/// achetée de zombies, salle nettoyée de gungeon). Un son par image. Pas de son pour
/// l'ouverture d'office des portes de salle (`RoomStates::doors_opened` qui passe à `true`, au
/// début de chaque étage), ni à la première image (référence).
fn play_door_sound(
    sounds: FeedbackSounds,
    doors: Query<
        (&GgrsNetId, Has<crate::collider::Collider>),
        With<map::game::entity::map::door::DoorComponent>,
    >,
    rooms: Option<Res<world::rooms::RoomStates>>,
    mut previous: Local<Option<(BTreeSet<usize>, bool)>>,
) {
    let now_closed: BTreeSet<usize> = doors
        .iter()
        .filter(|(_, has_collider)| *has_collider)
        .map(|(net_id, _)| net_id.0)
        .collect();
    let doors_opened = rooms.is_some_and(|rooms| rooms.doors_opened);
    let opened = previous.as_ref().is_some_and(|(closed, opened_before)| {
        let initial_opening = doors_opened && !opened_before;
        !initial_opening
            && doors
                .iter()
                .any(|(net_id, has_collider)| !has_collider && closed.contains(&net_id.0))
    });
    *previous = Some((now_closed, doors_opened));
    if opened {
        sounds.play("door");
    }
}
