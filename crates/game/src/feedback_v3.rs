//! Feedback v3 (M3-T17, `docs/conventions.md` §82) : larmes lisibles, familiers, champions,
//! bombes et explosions. Complète [`crate::feedback`] (§31, §55).
//!
//! **Présentation seule.** Rien ici n'est rollback, rien n'entre dans le checksum ni dans les
//! traces ; la simulation ne lit jamais ces fonctions ni ces composants. Tout est **dérivé de
//! l'état** (composants `Projectile`, `Bomb`, `FamiliarOf`, `Variant`, ressource `DoorLocks`) ou des
//! `FrameEvents` transitoires que la simulation émet déjà (`Exploded`, `RockBroken`) : un rollback
//! qui rejoue la même situation redonne les mêmes indices.
//!
//! Deux parties :
//! - les **fonctions pures** (look d'une larme, clignotement d'une mèche, débris, teinte d'un
//!   champion), testées sans `App` ;
//! - [`record_v3_cues`] : remplit [`FeedbackLog`] (headless compris), source de vérité des moments
//!   clés `feedback` (preuve sans écran) ; le rendu applique ensuite les indices
//!   (`feedback::FeedbackPlugin`, `world_sprites`).

use std::collections::BTreeSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::projectile::Projectile;
use combat::status::{StatusDef, StatusLibrary};
use combat::weapons::Bullet;
use effects::Action;
use utils::{frame::FrameCount, net_id::GgrsNetId};
use world::{Bomb, DoorLocks, Exploded};

use crate::bombs::RockBroken;
use crate::character::config::CharacterConfig;
use crate::character::variant::Variant;
use crate::character::visuals::CharacterAppearance;
use crate::familiars::{FamiliarBehaviors, FamiliarMotion};
use crate::feedback::{FeedbackCue, FeedbackKind, FeedbackLog};
use crate::frame_events::FrameEvents;
use crate::global_asset::GlobalAsset;
use behaviors::FamiliarOf;

// ---------------------------------------------------------------------------------------
// Larmes
// ---------------------------------------------------------------------------------------

/// Couleur de base d'une balle de joueur ou d'allié (celle de `world_sprites::skin_bullets`).
pub const PLAYER_BULLET_COLOR: (f32, f32, f32) = (1.0, 0.9, 0.4);
/// Couleur de base d'une balle ennemie.
pub const ENEMY_BULLET_COLOR: (f32, f32, f32) = (1.0, 0.35, 0.3);
/// Couleur d'une larme guidée (`Homing`) sans statut.
pub const HOMING_COLOR: (f32, f32, f32) = (0.45, 1.0, 0.9);
/// Opacité d'une larme spectrale (elle traverse les murs : elle se lit « fantôme »).
pub const SPECTRAL_ALPHA: f32 = 0.5;
/// Facteur de taille d'une larme qui se divise à l'impact.
pub const SPLIT_SCALE: f32 = 1.25;
/// Facteur de taille d'une larme perforante.
pub const PIERCE_SCALE: f32 = 1.15;
/// Facteur de taille par génération d'un projectile né d'un autre (`Split`, `on_expire`).
pub const GENERATION_SCALE: f32 = 0.8;
/// Taille minimale (facteur) d'une larme de génération avancée.
pub const MIN_TEAR_SCALE: f32 = 0.5;

/// Ce que la présentation montre d'une larme : couleur, opacité, facteur de taille, et la liste
/// lisible de ses modificateurs (moment clé).
#[derive(Debug, Clone, PartialEq)]
pub struct TearLook {
    pub color: (f32, f32, f32),
    pub alpha: f32,
    /// Multiplie la taille du sprite posée par `Size` (déjà résolue dans [`Projectile::size`]).
    pub scale: f32,
    /// « guidée », « spectrale », « orbite », « division », « perforante », nom du statut posé,
    /// « génération n ».
    pub traits: Vec<String>,
}

impl TearLook {
    /// Une larme sans rien de particulier : le rendu d'avant est gardé tel quel.
    pub fn is_plain(&self) -> bool {
        self.traits.is_empty()
    }
}

/// Teinte d'un statut posé par une larme (mêmes teintes que `feedback::status_tint`, celles du
/// personnage qui le subit : la larme annonce ce qu'elle va faire).
pub fn status_color(kind: StatusDef) -> (f32, f32, f32) {
    match kind {
        StatusDef::Freeze => (0.55, 0.8, 1.0),
        StatusDef::Stun => (1.0, 1.0, 0.45),
        StatusDef::Burn => (1.0, 0.55, 0.35),
        StatusDef::Poison => (0.55, 0.9, 0.4),
        StatusDef::Fear => (0.75, 0.55, 0.9),
        StatusDef::Charm => (1.0, 0.6, 0.8),
        StatusDef::Slow => (0.6, 0.6, 0.9),
    }
}

/// Nom lisible d'un statut (moments clés).
pub fn status_name(kind: StatusDef) -> &'static str {
    match kind {
        StatusDef::Freeze => "gel",
        StatusDef::Stun => "étourdissement",
        StatusDef::Burn => "brûlure",
        StatusDef::Poison => "poison",
        StatusDef::Fear => "peur",
        StatusDef::Charm => "charme",
        StatusDef::Slow => "lenteur",
    }
}

/// Statuts posés par l'`on_hit` d'un projectile (`Chance` déballée), dans l'ordre des actions ;
/// un statut inconnu de la bibliothèque est ignoré (le lint le refuse avant le jeu).
pub fn on_hit_statuses(actions: &[Action], library: &StatusLibrary) -> Vec<StatusDef> {
    fn walk(action: &Action, library: &StatusLibrary, out: &mut Vec<StatusDef>) {
        match action {
            Action::ApplyStatus { status, .. } => {
                if let Some(spec) = library.statuses.get(status) {
                    if !out.contains(&spec.kind) {
                        out.push(spec.kind);
                    }
                }
            }
            Action::Chance { then, .. } => walk(then, library, out),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for action in actions {
        walk(action, library, &mut out);
    }
    out
}

/// Look d'une larme d'après son état résolu : le statut posé donne la couleur (le premier), à
/// défaut le guidage ; `Spectral` la rend translucide ; `Split` et `Pierce` la grossissent, chaque
/// génération la rapetisse. `base` : couleur de l'équipe, gardée sans statut ni guidage.
pub fn tear_look(
    projectile: &Projectile,
    statuses: &[StatusDef],
    base: (f32, f32, f32),
) -> TearLook {
    let mut traits: Vec<String> = Vec::new();
    let mut color = base;
    let mut scale = 1.0;
    if projectile.homing.is_some() {
        traits.push("guidée".into());
        color = HOMING_COLOR;
    }
    if projectile.spectral {
        traits.push("spectrale".into());
    }
    if projectile.orbit.is_some() {
        traits.push("orbite".into());
    }
    if projectile.split.is_some() {
        traits.push("division".into());
        scale *= SPLIT_SCALE;
    }
    if projectile.pierces_left > 0 {
        traits.push("perforante".into());
        scale *= PIERCE_SCALE;
    }
    for (index, kind) in statuses.iter().enumerate() {
        traits.push(status_name(*kind).into());
        if index == 0 {
            color = status_color(*kind);
        }
    }
    if projectile.generation > 0 {
        traits.push(format!("génération {}", projectile.generation));
        scale *= GENERATION_SCALE.powi(projectile.generation as i32);
    }
    TearLook {
        color,
        alpha: if projectile.spectral {
            SPECTRAL_ALPHA
        } else {
            1.0
        },
        scale: scale.max(MIN_TEAR_SCALE),
        traits,
    }
}

// ---------------------------------------------------------------------------------------
// Bombe : mèche
// ---------------------------------------------------------------------------------------

/// État du clignotement d'une mèche.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuseBlink {
    /// Phase « allumée » (rouge vif) ; sinon teinte normale.
    pub lit: bool,
    /// Facteur de taille : la bombe gonfle un peu en s'approchant de l'explosion.
    pub scale: f32,
}

/// Demi-période du clignotement (frames) avec `remaining` frames de mèche : 8 loin de
/// l'explosion, de plus en plus court, 2 sur la fin.
pub fn blink_half_period(remaining: u32) -> u32 {
    (remaining / 4).clamp(2, 8)
}

/// Clignotement d'une mèche avec `remaining` frames avant l'explosion : le rythme s'accélère
/// (demi-période de [`blink_half_period`]), la taille grandit jusqu'à +25 % sur les 30 dernières
/// frames.
pub fn fuse_blink(remaining: u32) -> FuseBlink {
    let half = blink_half_period(remaining);
    let lit = (remaining / half) % 2 == 0;
    let urgency = 1.0 - (remaining.min(30) as f32 / 30.0);
    FuseBlink {
        lit,
        scale: 1.0 + 0.25 * urgency,
    }
}

/// Couleur d'une bombe selon le clignotement (teinte multipliée à l'image).
pub fn fuse_color(blink: FuseBlink) -> (f32, f32, f32) {
    if blink.lit {
        (1.0, 0.25, 0.2)
    } else {
        (0.85, 0.85, 0.85)
    }
}

// ---------------------------------------------------------------------------------------
// Explosion, débris
// ---------------------------------------------------------------------------------------

/// Durée de vie d'un débris, en frames de simulation.
pub const DEBRIS_FRAMES: u32 = 24;
/// Nombre de débris d'un rocher cassé.
pub const ROCK_DEBRIS: u32 = 6;
/// Nombre de débris de poussière d'un mur secret qui s'ouvre.
pub const WALL_DEBRIS: u32 = 12;

/// Secousse d'une explosion de rayon `radius` : `(frames, amplitude)`, amplitude proportionnelle au
/// rayon (`shake_amplitude` pour 100 unités, plafonnée à 3×, au moins 1 unité tant qu'elle est
/// active).
pub fn explosion_shake(radius: f32, frames: u32, amplitude_per_100: f32) -> Option<(u32, f32)> {
    if frames == 0 || amplitude_per_100 <= 0.0 {
        return None;
    }
    let amplitude = (amplitude_per_100 * radius / 100.0).clamp(1.0, amplitude_per_100 * 3.0);
    Some((frames, amplitude))
}

/// Décalage d'un débris `age` frames après la casse, relatif au centre : direction répartie sur
/// le cercle (`index` sur `count`, décalée d'un demi-pas pour ne pas s'aligner sur les axes),
/// vitesse variable selon l'indice, freinage en `1 - (1 - p)²`. Déterministe, sans RNG.
pub fn debris_offset(index: u32, count: u32, age: u32) -> Vec2 {
    if count == 0 || age >= DEBRIS_FRAMES {
        return Vec2::ZERO;
    }
    let step = std::f32::consts::TAU / count as f32;
    let angle = step * (index as f32 + 0.5);
    let reach = 10.0 + 6.0 * (index % 3) as f32;
    let p = age as f32 / DEBRIS_FRAMES as f32;
    Vec2::from_angle(angle) * reach * (1.0 - (1.0 - p) * (1.0 - p))
}

/// Opacité d'un débris : pleine sur la première moitié de sa vie, puis linéaire jusqu'à 0.
pub fn debris_alpha(age: u32) -> f32 {
    if age >= DEBRIS_FRAMES {
        return 0.0;
    }
    let p = age as f32 / DEBRIS_FRAMES as f32;
    if p < 0.5 {
        1.0
    } else {
        2.0 * (1.0 - p)
    }
}

// ---------------------------------------------------------------------------------------
// Familiers, champions
// ---------------------------------------------------------------------------------------

/// « suiveur » ou « orbital » : le mouvement d'un familier (le premier de sa liste), « immobile »
/// sans mouvement.
pub fn familiar_kind(behaviors: &FamiliarBehaviors) -> &'static str {
    match behaviors.motion() {
        Some(FamiliarMotion::Follow { .. }) => "suiveur",
        Some(FamiliarMotion::Orbit { .. }) => "orbital",
        None => "immobile",
    }
}

/// Angle (radians) d'un orbital : du porteur vers lui, la pointe du sprite vers l'extérieur.
pub fn outward_angle(owner: Vec2, familiar: Vec2) -> f32 {
    let d = familiar - owner;
    if d == Vec2::ZERO {
        0.0
    } else {
        d.y.atan2(d.x)
    }
}

/// Un suiveur regarde-t-il vers la gauche ? Vers le côté où va son porteur ; à défaut (porteur
/// immobile), vers le porteur.
pub fn follower_faces_left(owner_dx: f32, to_owner_dx: f32) -> Option<bool> {
    if owner_dx.abs() > 0.05 {
        Some(owner_dx < 0.0)
    } else if to_owner_dx.abs() > 0.5 {
        Some(to_owner_dx < 0.0)
    } else {
        None
    }
}

/// Teinte `#rrggbb` d'une variante, en composantes 0 à 1.
pub fn parse_tint(tint: &str) -> Option<(f32, f32, f32)> {
    let hex = tint.strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map(|v| v as f32 / 255.0);
    Some((channel(0).ok()?, channel(2).ok()?, channel(4).ok()?))
}

/// Teinte de la variante `variant` du personnage `config_name`, si la table en déclare une.
pub fn variant_tint(
    global: &GlobalAsset,
    configs: &Assets<CharacterConfig>,
    config_name: &str,
    variant: &str,
) -> Option<(f32, f32, f32)> {
    let config = configs.get(global.character_configs.get(config_name)?)?;
    let tint = config
        .variants
        .as_ref()?
        .table
        .get(variant)?
        .tint
        .as_deref()?;
    parse_tint(tint)
}

// ---------------------------------------------------------------------------------------
// Journal
// ---------------------------------------------------------------------------------------

/// Ce que [`record_v3_cues`] lit.
#[derive(SystemParam)]
pub struct V3Inputs<'w, 's> {
    tears: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static Projectile,
            &'static Bullet,
            &'static FixedTransform3D,
        ),
    >,
    familiars: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static FamiliarOf,
            &'static FamiliarBehaviors,
            &'static FixedTransform3D,
        ),
    >,
    champions: Query<
        'w,
        's,
        (
            &'static GgrsNetId,
            &'static Variant,
            &'static CharacterAppearance,
            &'static FixedTransform3D,
        ),
    >,
    bombs: Query<'w, 's, (&'static GgrsNetId, &'static Bomb, &'static FixedTransform3D)>,
    library: Option<Res<'w, StatusLibrary>>,
    global: Option<Res<'w, GlobalAsset>>,
    configs: Option<Res<'w, Assets<CharacterConfig>>>,
    exploded: Option<Res<'w, FrameEvents<Exploded>>>,
    rocks: Option<Res<'w, FrameEvents<RockBroken>>>,
    locks: Option<Res<'w, DoorLocks>>,
}

/// Mémoire d'un tick à l'autre : tout se dérive de l'état, jamais d'un événement.
#[derive(Default)]
pub struct V3Memory {
    last_frame: Option<u32>,
    tears: BTreeSet<usize>,
    familiars: BTreeSet<usize>,
    champions: BTreeSet<usize>,
    bombs: BTreeSet<usize>,
    open_rooms: BTreeSet<String>,
}

fn xy(transform: &FixedTransform3D) -> Vec2 {
    Vec2::new(
        transform.translation.x.to_num::<f32>(),
        transform.translation.y.to_num::<f32>(),
    )
}

fn cue(kind: FeedbackKind, frame: u32, net_id: usize, position: Vec2) -> FeedbackCue {
    FeedbackCue {
        kind,
        frame,
        net_id,
        position,
        value: 0.0,
        extra: 0.0,
        color: (1.0, 1.0, 1.0),
        pattern: None,
        note: None,
    }
}

/// Indices de M3-T17 pour la frame simulée de ce tick : une apparition = un indice (larme à
/// modificateurs, familier, champion, bombe), une explosion ou un rocher cassé = un indice par
/// événement, un mur secret ouvert = un indice par salle. S'exécute après
/// `feedback::record_feedback_cues` (qui vide `frame_cues` à chaque tick) et applique la même règle :
/// rien sans nouvelle frame simulée, mémoire remise à zéro quand le compteur de frames recule.
pub fn record_v3_cues(
    frame: Res<FrameCount>,
    mut log: ResMut<FeedbackLog>,
    inputs: V3Inputs,
    mut memory: Local<V3Memory>,
) {
    let now = frame.frame;
    match memory.last_frame {
        Some(last) if now == last => return,
        Some(last) if now < last => *memory = V3Memory::default(),
        _ => {}
    }
    memory.last_frame = Some(now);

    // Larmes : un indice quand une larme à modificateurs apparaît (ordre `GgrsNetId`).
    let empty = StatusLibrary::default();
    let library = inputs.library.as_deref().unwrap_or(&empty);
    let mut tears: Vec<_> = inputs.tears.iter().collect();
    tears.sort_by_key(|(net_id, ..)| net_id.0);
    for (net_id, projectile, _, transform) in tears {
        if memory.tears.contains(&net_id.0) {
            continue;
        }
        let statuses = on_hit_statuses(&projectile.on_hit, library);
        let look = tear_look(projectile, &statuses, PLAYER_BULLET_COLOR);
        if look.is_plain() {
            continue;
        }
        let mut c = cue(FeedbackKind::Tear, now, net_id.0, xy(transform));
        c.value = look.scale;
        c.color = look.color;
        c.note = Some(look.traits.join(", "));
        log.push(c);
    }
    // Les larmes plates restent en mémoire aussi : elles ne sont pas réévaluées à chaque tick.
    memory.tears = inputs.tears.iter().map(|(net_id, ..)| net_id.0).collect();

    // Familiers.
    let mut current = BTreeSet::new();
    let mut familiars: Vec<_> = inputs.familiars.iter().collect();
    familiars.sort_by_key(|(net_id, ..)| net_id.0);
    for (net_id, of, behaviors, transform) in familiars {
        current.insert(net_id.0);
        if memory.familiars.contains(&net_id.0) {
            continue;
        }
        let mut c = cue(FeedbackKind::Familiar, now, net_id.0, xy(transform));
        c.value = of.rank as f32;
        c.extra = of.owner.0 as f32;
        c.note = Some(familiar_kind(behaviors).to_string());
        log.push(c);
    }
    memory.familiars = current;

    // Champions : une variante à `tint`.
    let mut current = BTreeSet::new();
    if let (Some(global), Some(configs)) = (&inputs.global, &inputs.configs) {
        let mut champions: Vec<_> = inputs.champions.iter().collect();
        champions.sort_by_key(|(net_id, ..)| net_id.0);
        for (net_id, variant, appearance, transform) in champions {
            let Some(tint) = variant_tint(global, configs, &appearance.config_name, &variant.0)
            else {
                continue;
            };
            current.insert(net_id.0);
            if memory.champions.contains(&net_id.0) {
                continue;
            }
            let mut c = cue(FeedbackKind::Champion, now, net_id.0, xy(transform));
            c.color = tint;
            c.note = Some(variant.0.clone());
            log.push(c);
        }
    }
    memory.champions = current;

    // Bombes posées : mèche (frames restantes à la pose).
    let mut current = BTreeSet::new();
    let mut bombs: Vec<_> = inputs.bombs.iter().collect();
    bombs.sort_by_key(|(net_id, ..)| net_id.0);
    for (net_id, bomb, transform) in bombs {
        current.insert(net_id.0);
        if memory.bombs.contains(&net_id.0) {
            continue;
        }
        let mut c = cue(FeedbackKind::BombFuse, now, net_id.0, xy(transform));
        c.value = bomb.fuse_until.saturating_sub(now) as f32;
        log.push(c);
    }
    memory.bombs = current;

    // Explosions de la frame (`FrameEvents` transitoire, déjà émis par la simulation).
    let mut destroying: Option<Vec2> = None;
    if let Some(exploded) = &inputs.exploded {
        for event in exploded.iter() {
            let position = Vec2::new(event.x.to_num::<f32>(), event.y.to_num::<f32>());
            let mut c = cue(FeedbackKind::Explosion, now, 0, position);
            c.extra = event.radius.to_num::<f32>();
            c.value = if event.destroys { 1.0 } else { 0.0 };
            log.push(c);
            if event.destroys && destroying.is_none() {
                destroying = Some(position);
            }
        }
    }
    if let Some(rocks) = &inputs.rocks {
        for rock in rocks.iter() {
            let position = Vec2::new(rock.x.to_num::<f32>(), rock.y.to_num::<f32>());
            let mut c = cue(FeedbackKind::RockBroken, now, 0, position);
            c.value = if rock.loot { 1.0 } else { 0.0 };
            c.note = Some(format!("{},{}", rock.cell.0, rock.cell.1));
            log.push(c);
        }
    }

    // Mur secret : une salle entre dans `DoorLocks` (état, pas événement : un rollback qui
    // retombe avant l'ouverture la fait disparaître de l'ensemble et la mémoire suit).
    let open: BTreeSet<String> = inputs
        .locks
        .as_ref()
        .map(|locks| locks.0.clone())
        .unwrap_or_default();
    for room in open.difference(&memory.open_rooms) {
        let mut c = cue(
            FeedbackKind::SecretWall,
            now,
            0,
            destroying.unwrap_or(Vec2::ZERO),
        );
        c.value = if destroying.is_some() { 1.0 } else { 0.0 };
        c.note = Some(room.clone());
        log.push(c);
    }
    memory.open_rooms = open;
}

/// Étiquette (moment clé) d'un indice de M3-T17, `None` pour les autres genres.
pub fn cue_label(cue: &FeedbackCue) -> Option<String> {
    let note = cue.note.as_deref().unwrap_or("");
    Some(match cue.kind {
        FeedbackKind::Tear => format!("{} {} ({})", cue.kind.label(), cue.net_id, note),
        FeedbackKind::Familiar => format!(
            "{} {} ({}, porteur {}, rang {})",
            cue.kind.label(),
            cue.net_id,
            note,
            cue.extra,
            cue.value
        ),
        FeedbackKind::Champion => format!("{} {} ({})", cue.kind.label(), cue.net_id, note),
        FeedbackKind::BombFuse => {
            format!("{} {} ({} frames)", cue.kind.label(), cue.net_id, cue.value)
        }
        FeedbackKind::Explosion => format!("anneau d'explosion (rayon {})", cue.extra),
        FeedbackKind::RockBroken => format!("{} ({})", cue.kind.label(), note),
        FeedbackKind::SecretWall => format!("{} ouvert ({})", cue.kind.label(), note),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::Fixed;

    fn projectile() -> Projectile {
        use combat::projectile::{ProjectileSpec, ProjectileTable};
        let spec = ProjectileSpec {
            modifiers: Vec::new(),
            on_hit: Vec::new(),
            on_expire: Vec::new(),
        };
        Projectile::new(
            "test".into(),
            &spec,
            std::sync::Arc::new(ProjectileTable::new()),
            Fixed::ONE,
            0,
        )
    }

    #[test]
    fn une_larme_ordinaire_est_plate() {
        let look = tear_look(&projectile(), &[], PLAYER_BULLET_COLOR);
        assert!(look.is_plain());
        assert_eq!(look.color, PLAYER_BULLET_COLOR);
        assert_eq!(look.alpha, 1.0);
        assert_eq!(look.scale, 1.0);
    }

    #[test]
    fn guidage_spectral_et_statut() {
        let mut p = projectile();
        p.homing = Some(Fixed::from_num(1));
        let look = tear_look(&p, &[], PLAYER_BULLET_COLOR);
        assert_eq!(look.color, HOMING_COLOR);
        assert_eq!(look.traits, vec!["guidée"]);

        // Le statut l'emporte sur le guidage pour la couleur ; spectral : translucide.
        p.spectral = true;
        let look = tear_look(&p, &[StatusDef::Poison], PLAYER_BULLET_COLOR);
        assert_eq!(look.color, status_color(StatusDef::Poison));
        assert_eq!(look.alpha, SPECTRAL_ALPHA);
        assert_eq!(look.traits, vec!["guidée", "spectrale", "poison"]);
    }

    #[test]
    fn taille_division_et_generations() {
        let mut p = projectile();
        p.split = Some(combat::projectile::ResolvedSplit {
            count: 3,
            spread: Fixed::from_num(1),
            child: None,
        });
        let parent = tear_look(&p, &[], PLAYER_BULLET_COLOR);
        assert!((parent.scale - SPLIT_SCALE).abs() < 1e-6);
        // Un enfant (génération 1, sans split) est plus petit que la base.
        let mut child = projectile();
        child.generation = 1;
        let child = tear_look(&child, &[], PLAYER_BULLET_COLOR);
        assert!((child.scale - GENERATION_SCALE).abs() < 1e-6);
        // Plancher.
        let mut deep = projectile();
        deep.generation = 8;
        assert_eq!(
            tear_look(&deep, &[], PLAYER_BULLET_COLOR).scale,
            MIN_TEAR_SCALE
        );
    }

    #[test]
    fn la_meche_accelere_et_grossit() {
        assert_eq!(blink_half_period(100), 8);
        assert_eq!(blink_half_period(20), 5);
        assert_eq!(blink_half_period(3), 2);
        // Les changements de phase se rapprochent : compte des bascules sur 40 frames.
        let flips = |range: std::ops::Range<u32>| {
            range
                .map(|r| fuse_blink(r).lit)
                .collect::<Vec<_>>()
                .windows(2)
                .filter(|w| w[0] != w[1])
                .count()
        };
        assert!(flips(1..21) > flips(40..60));
        assert_eq!(fuse_blink(40).scale, 1.0);
        assert!(fuse_blink(1).scale > fuse_blink(15).scale);
        assert!(fuse_blink(0).scale <= 1.25 + 1e-6);
    }

    #[test]
    fn secousse_proportionnelle_et_bornee() {
        assert_eq!(explosion_shake(100.0, 10, 8.0), Some((10, 8.0)));
        assert_eq!(explosion_shake(50.0, 10, 8.0), Some((10, 4.0)));
        assert_eq!(explosion_shake(1000.0, 10, 8.0), Some((10, 24.0)));
        assert_eq!(explosion_shake(1.0, 10, 8.0), Some((10, 1.0)));
        assert_eq!(explosion_shake(100.0, 0, 8.0), None);
        assert_eq!(explosion_shake(100.0, 10, 0.0), None);
    }

    #[test]
    fn debris_partent_en_etoile_et_s_estompent() {
        assert_eq!(debris_offset(0, 6, 0), Vec2::ZERO);
        assert_eq!(debris_offset(0, 6, DEBRIS_FRAMES), Vec2::ZERO);
        // Deux débris ne partent pas dans la même direction.
        let a = debris_offset(0, 6, 12);
        let b = debris_offset(3, 6, 12);
        assert!(a.dot(b) < 0.0);
        // Ils s'éloignent tant qu'ils vivent.
        assert!(debris_offset(1, 6, 20).length() > debris_offset(1, 6, 5).length());
        assert_eq!(debris_alpha(0), 1.0);
        assert_eq!(debris_alpha(DEBRIS_FRAMES), 0.0);
        assert!(debris_alpha(18) < 1.0 && debris_alpha(18) > 0.0);
    }

    #[test]
    fn teinte_de_variante() {
        assert_eq!(parse_tint("#ff0000"), Some((1.0, 0.0, 0.0)));
        let (r, g, b) = parse_tint("#c0392b").unwrap();
        assert!((r - 192.0 / 255.0).abs() < 1e-6);
        assert!((g - 57.0 / 255.0).abs() < 1e-6);
        assert!((b - 43.0 / 255.0).abs() < 1e-6);
        assert_eq!(parse_tint("ff0000"), None);
        assert_eq!(parse_tint("#ff00"), None);
        assert_eq!(parse_tint("#gg0000"), None);
    }

    #[test]
    fn orientation_des_familiers() {
        assert!(
            (outward_angle(Vec2::ZERO, Vec2::new(0.0, 5.0)) - std::f32::consts::FRAC_PI_2).abs()
                < 1e-6
        );
        assert_eq!(outward_angle(Vec2::ONE, Vec2::ONE), 0.0);
        assert_eq!(follower_faces_left(-1.0, 10.0), Some(true));
        assert_eq!(follower_faces_left(0.0, 10.0), Some(false));
        assert_eq!(follower_faces_left(0.0, -10.0), Some(true));
        assert_eq!(follower_faces_left(0.0, 0.1), None);
    }
}
