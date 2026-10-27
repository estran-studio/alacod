//! Feedback v1 (T2.13, étendu en T1.17, `docs/conventions.md` §7 et §31) : flash à l'impact,
//! secousse de caméra, hit stop, chiffres de dégâts, cercle de télégraphe, sons.
//!
//! **Présentation seule** : rien ici n'est rollback, rien n'entre dans le checksum ni dans les
//! traces, et la simulation ne lit jamais ces composants ou ressources.
//!
//! Le journal (`FeedbackLog`, `FeedbackLogPlugin`, formes et fonctions pures) est la moitié
//! simulation : [`crate::feedback_log`] (D69 étape 2, E5b). Ici, la moitié application, deux plugins :
//! - [`FeedbackAssetsPlugin`] (toujours actif, headless compris) : charge `ui/feedback.ron` et
//!   publie [`FeedbackConfigLoaded`] (lue par le journal, qui est la source de vérité : le rendu
//!   applique ses indices, et `scenario::events` les relit pour en faire des moments clés `feedback`) ;
//! - [`FeedbackPlugin`] (rendu seulement, `PresentationPlugin`) : applique les indices (flash,
//!   secousse, hit stop, chiffres), dessine les télégraphes et joue les sons.

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D};
use bevy_kira_audio::prelude::*;
use combat::emitter::Emitter;
use serde::{Deserialize, Serialize};
use sim_core::damage::DamageEvent;
use sim_core::tag::Tag;
use std::collections::{BTreeMap, BTreeSet};
use utils::{frame::FrameCount, net_id::GgrsNetId};

use crate::character::enemy::ai::state::{BehaviorRuntime, EnemyAiConfig};
use crate::character::player::LocalPlayer;
use crate::feedback_log::{
    blank_ring_radius, boss_zoom_factor, boss_zoom_weight, flash_alpha, pattern_shape,
    ranged_range, telegraph_shape, FeedbackConfigLoaded, FeedbackKind, FeedbackLog,
    FeedbackSettings, PatternKind, TelegraphConfig,
};
use crate::frame_events::FrameEvents;
use crate::session::is_headless;

/// Asset `ui/feedback.ron` : enveloppe du type partagé avec le lint
/// (`content::feedback::FeedbackSettings`).
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FeedbackConfig(pub FeedbackSettings);

/// Plugin de chargement de `ui/feedback.ron` (application, headless compris) : l'enveloppe `Asset`
/// [`FeedbackConfig`], son chargeur RON, et la publication de [`FeedbackConfigLoaded`] que lit
/// [`crate::feedback_log::FeedbackLogPlugin`] (simulation, sans asset).
pub struct FeedbackAssetsPlugin;

impl Plugin for FeedbackAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<FeedbackConfig>::new(&["ron"]));
        // Charger la config au démarrage
        app.add_systems(Startup, load_feedback_config);
        // Publie `FeedbackConfigLoaded` une fois l'asset chargé (et à chaque modification) :
        // sans ce système, aucun effet ne s'appliquait jamais (config toujours absente).
        app.add_systems(Update, publish_loaded_feedback_config);
    }
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
        app.add_systems(Update, (tint_champions, tint_transformations));
        // X-3 : musique calée sur l'horloge de simulation (lecture seule de la simulation).
        app.add_systems(Update, crate::music_sync::sync_music);
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
                apply_feedback_cues.after(crate::feedback_v3::record_v3_cues),
                apply_layer_colors.after(apply_feedback_cues),
                cleanup_expired_hit_flash.after(apply_layer_colors),
                // Avant la propagation des transforms : sinon le décalage n'est appliqué au
                // `Transform` qu'après le calcul du `GlobalTransform` rendu, et le suivi de
                // caméra (`Update`) l'écrase à la frame suivante : la secousse n'est jamais vue.
                update_camera_shake_offset
                    .after(apply_feedback_cues)
                    .before(bevy::transform::TransformSystems::Propagate),
                animate_damage_numbers,
                animate_debris,
                draw_telegraphs,
                play_shot_sound_for_new_bullets,
                play_reload_sound_for_reload_events,
                play_hit_sound,
                play_hurt_sound,
                play_melee_taken_sound,
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
    holds: Query<&animation::AnimationHold>,
    config: Option<Res<FeedbackConfigLoaded>>,
    mut fx: ResMut<FeedbackFx>,
) {
    if log.frame_cues.is_empty() {
        return;
    }
    let settings = config.as_ref().map(|config| &config.0);
    let blank = settings.map(|s| s.blank).unwrap_or_default();
    let boss_end = settings.map(|s| s.boss_end).unwrap_or_default();
    let explosion = settings.map(|s| s.explosion).unwrap_or_default();
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
                shake_camera(
                    &mut commands,
                    &mut cameras,
                    cue.frame,
                    cue.value as u32,
                    cue.extra,
                );
            }
            // `extra` : 1 = hit stop de la cible seule (X-3) : seule son animation s'arrête, la
            // caméra et le reste du monde continuent ; 0 = gel global (T1.17).
            FeedbackKind::HitStop if cue.extra >= 1.0 => {
                if let Some(entity) = entity_of.get(&cue.net_id) {
                    let hold = freeze.hold_for(cue.value as u32, holds.get(*entity).ok());
                    commands.entity(*entity).insert(hold);
                }
            }
            FeedbackKind::HitStop => freeze.freeze_for(cue.value as u32),
            FeedbackKind::Number => {
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
            // M3-T17 : apparitions (larme, familier, champion, bombe) : journal seulement, le
            // rendu les dérive de l'état (`world_sprites`, `style_*`).
            FeedbackKind::Tear
            | FeedbackKind::Familiar
            | FeedbackKind::Champion
            | FeedbackKind::BombFuse
            | FeedbackKind::Transformation => {}
            FeedbackKind::Explosion => {
                if !explosion.enabled {
                    continue;
                }
                let (r, g, b, a) = explosion.color;
                let color = Color::srgba(r, g, b, a);
                // Deux anneaux : le plein rayon et un anneau intérieur plus clair, qui épaissit le
                // front de l'onde.
                for (scale, tint) in [(1.0, color), (0.6, color.mix(&Color::WHITE, 0.6))] {
                    fx.rings.push(BlankRing {
                        frame: cue.frame,
                        center: cue.position,
                        radius: cue.extra * scale,
                        frames: explosion.ring_frames,
                        color: tint,
                    });
                }
                if let Some((frames, amplitude)) = crate::feedback_v3::explosion_shake(
                    cue.extra,
                    explosion.shake_frames,
                    explosion.shake_amplitude,
                ) {
                    shake_camera(&mut commands, &mut cameras, cue.frame, frames, amplitude);
                }
            }
            FeedbackKind::RockBroken => {
                if !explosion.enabled {
                    continue;
                }
                spawn_debris(
                    &mut commands,
                    cue.frame,
                    cue.position,
                    crate::feedback_v3::ROCK_DEBRIS,
                    Color::srgb(0.55, 0.5, 0.45),
                    4.0,
                );
            }
            FeedbackKind::SecretWall => {
                // Nuage de poussière à l'endroit de l'explosion qui a ouvert le mur (sans
                // explosion dans la même frame, rien à situer : l'état seul change).
                if !explosion.enabled || cue.value == 0.0 {
                    continue;
                }
                spawn_debris(
                    &mut commands,
                    cue.frame,
                    cue.position,
                    crate::feedback_v3::WALL_DEBRIS,
                    Color::srgb(0.85, 0.78, 0.62),
                    5.0,
                );
                fx.rings.push(BlankRing {
                    frame: cue.frame,
                    center: cue.position,
                    radius: SECRET_WALL_DUST_RADIUS,
                    frames: explosion.ring_frames * 2,
                    color: Color::srgba(0.85, 0.78, 0.62, 1.0),
                });
            }
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
/// (gel, étourdi, brûlure, poison, peur, lenteur ; M3-T1 : poison et peur).
fn status_tint(statuses: &combat::status::Statuses) -> Color {
    use combat::status::StatusDef;
    let has = |kind: StatusDef| statuses.0.iter().any(|entry| entry.status == kind);
    if has(StatusDef::Freeze) {
        Color::srgb(0.55, 0.8, 1.0)
    } else if has(StatusDef::Stun) {
        Color::srgb(1.0, 1.0, 0.45)
    } else if has(StatusDef::Burn) {
        Color::srgb(1.0, 0.55, 0.35)
    } else if has(StatusDef::Poison) {
        Color::srgb(0.55, 0.9, 0.4)
    } else if has(StatusDef::Charm) {
        Color::srgb(1.0, 0.6, 0.8)
    } else if has(StatusDef::Fear) {
        Color::srgb(0.75, 0.55, 0.9)
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
            Option<&crate::character::visuals::MarkerTint>,
            Option<&ChampionTint>,
            Option<&TransformationTint>,
        ),
        With<combat::actors::Health>,
    >,
    // Les barres de vie gardent leur couleur (rouge pour un ennemi, verte pour un joueur) : la teinte
    // et le flash ne vont qu'aux calques du personnage.
    mut sprites: Query<&mut Sprite, Without<crate::character::health_bar::HealthBar>>,
) {
    for (layers, flash, statuses, marker, champion, transformation) in &characters {
        // D66 : la teinte du marqueur remplace le blanc de base (statuts et flash passent devant).
        // M3-T17 : la teinte de la variante (champion) passe devant celle du marqueur ; suivi Q10 : celle
        // d'une transformation active aussi (elle passe derrière celle d'un champion).
        let base = champion
            .and_then(|champion| champion.0)
            .or(transformation.map(|transformation| transformation.0))
            .or(marker.map(|marker| marker.0))
            .unwrap_or(Color::WHITE);
        let color = match flash {
            Some(flash) if frame.frame < flash.until_frame => flash.color,
            _ => statuses
                .map(status_tint)
                .filter(|tint| *tint != Color::WHITE)
                .unwrap_or(base),
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
) {
    let now = time.elapsed_secs_f64();
    for (entity, number, mut transform, mut color) in numbers.iter_mut() {
        let age = now - number.born_secs;
        if age >= DAMAGE_NUMBER_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += DAMAGE_NUMBER_RISE * time.delta_secs();
        let alpha = 1.0 - (age / DAMAGE_NUMBER_SECS) as f32;
        color.0 = color.0.with_alpha(alpha);
    }
}

/// Teinte `tint` de la variante d'un personnage (M3-T17, §72) : `None` si la variante n'en a pas.
/// Présentation, hors rollback ; posée une fois par [`tint_champions`].
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ChampionTint(pub Option<Color>);

/// Pose [`ChampionTint`] sur un personnage dont la variante est connue (une fois), lue dans la
/// table de variantes de sa fiche.
fn tint_champions(
    mut commands: Commands,
    configs: Res<crate::character::config::CharacterConfigs>,
    champions: Query<
        (
            Entity,
            &crate::character::variant::Variant,
            &crate::character::appearance::CharacterAppearance,
        ),
        Without<ChampionTint>,
    >,
) {
    for (entity, variant, appearance) in &champions {
        let tint = crate::feedback_v3::variant_tint(&configs, &appearance.config_name, &variant.0)
            .map(|(r, g, b)| Color::srgb(r, g, b));
        commands.entity(entity).insert(ChampionTint(tint));
    }
}

/// Teinte d'une transformation active du joueur (suivi Q10, §44) : celle de la première synergie (par id)
/// à `tint` dont les conditions sont remplies. Présentation, hors rollback, relue de l'état chaque tick
/// par [`tint_transformations`] (posée, changée, retirée : aucun événement à rejouer).
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct TransformationTint(pub Color);

/// Pose, change ou retire [`TransformationTint`] selon l'inventaire du joueur (`items::SynergyTable`).
fn tint_transformations(
    mut commands: Commands,
    items: Option<Res<items::ItemTable>>,
    synergies: Option<Res<items::SynergyTable>>,
    players: Query<
        (Entity, &items::Inventory, Option<&TransformationTint>),
        With<crate::character::player::Player>,
    >,
) {
    let (Some(items), Some(synergies)) = (items, synergies) else {
        return;
    };
    for (entity, inventory, current) in &players {
        let wanted = crate::feedback_v3::transformation_tint(&synergies, &items, inventory)
            .map(|(_, (r, g, b))| Color::srgb(r, g, b));
        match (wanted, current) {
            (Some(color), Some(have)) if have.0 == color => {}
            (Some(color), _) => {
                commands.entity(entity).insert(TransformationTint(color));
            }
            (None, Some(_)) => {
                commands.entity(entity).remove::<TransformationTint>();
            }
            (None, None) => {}
        }
    }
}

/// Rayon (unités monde) du nuage de poussière d'un mur secret qui s'ouvre.
const SECRET_WALL_DUST_RADIUS: f32 = 48.0;

/// Secousse de la caméra de jeu (`GameCamera`, suivi des joueurs), pas n'importe quelle `Camera` :
/// en capture il en existe plusieurs.
fn shake_camera(
    commands: &mut Commands,
    cameras: &mut Query<(Entity, Option<&mut CameraShake>), With<crate::camera::GameCamera>>,
    frame: u32,
    frames: u32,
    amplitude: f32,
) {
    let Some((camera, shake)) = cameras.iter_mut().next() else {
        return;
    };
    let until_frame = frame + frames;
    match shake {
        Some(mut shake) => {
            shake.until_frame = until_frame;
            shake.frames = frames;
            shake.amplitude = amplitude;
        }
        None => {
            commands.entity(camera).insert(CameraShake {
                until_frame,
                frames,
                amplitude,
                applied: Vec3::ZERO,
            });
        }
    }
}

/// Éclat d'un rocher cassé ou d'un mur qui s'ouvre (présentation) : position et opacité dérivées
/// de l'âge en frames de simulation ([`crate::feedback_v3::debris_offset`]).
#[derive(Component, Debug)]
pub struct Debris {
    born: u32,
    index: u32,
    count: u32,
    origin: Vec2,
    color: Color,
}

/// Fait naître `count` éclats autour de `origin`, de côté `size`.
fn spawn_debris(
    commands: &mut Commands,
    frame: u32,
    origin: Vec2,
    count: u32,
    color: Color,
    size: f32,
) {
    for index in 0..count {
        commands.spawn((
            Debris {
                born: frame,
                index,
                count,
                origin,
                color,
            },
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_xyz(origin.x, origin.y, 45.0),
        ));
    }
}

/// Déplace et estompe les éclats ; retire ceux dont la vie est finie (ou nés d'une frame que la
/// simulation a depuis rejouée plus tôt).
fn animate_debris(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut debris: Query<(Entity, &Debris, &mut Transform, &mut Sprite)>,
) {
    for (entity, d, mut transform, mut sprite) in &mut debris {
        let age = frame.frame.saturating_sub(d.born);
        if frame.frame < d.born || age >= crate::feedback_v3::DEBRIS_FRAMES {
            commands.entity(entity).despawn();
            continue;
        }
        let at = d.origin + crate::feedback_v3::debris_offset(d.index, d.count, age);
        transform.translation.x = at.x;
        transform.translation.y = at.y;
        sprite.color = d.color.with_alpha(crate::feedback_v3::debris_alpha(age));
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
        // D67 : une couronne (`Ring`) se dit par ses rayons courts (dessinés par
        // `draw_pattern_shapes`), pas par le cercle de portée — il ressemblait à une
        // zone de dégâts et couvrait l'écran. Le cercle reste pour les autres formes.
        if let Some(emitter) = emitter {
            let aim = Vec2::new(emitter.aim.x.to_num::<f32>(), emitter.aim.y.to_num::<f32>());
            if let Some(pattern) = pattern_shape(
                &emitter.steps,
                emitter.next_step as usize,
                aim,
                position,
                shape.radius,
            ) {
                if pattern.kind == PatternKind::Ring {
                    continue;
                }
            }
        }
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

/// Son de tir : `firing` du mode actif dans l'`audio_config` de l'arme (`weapons.ron`), sinon
/// la clé de la famille du mode dans `sounds`, sinon `shot`, sinon aucun son. Renvoie le
/// chemin et la clé de `sounds` dont il prend le volume (`volumes`) : celle de la famille,
/// même quand l'arme la remplace.
fn shot_sound<'a>(
    sounds: &'a BTreeMap<String, String>,
    weapon_audio: Option<&'a crate::weapons::WeaponModeAudio>,
    mode: Option<&crate::weapons::FiringMode>,
) -> Option<(&'a str, &'static str)> {
    let family = mode.map(shot_sound_key);
    if let Some(path) = weapon_audio
        .map(|audio| audio.firing.as_str())
        .filter(|path| !path.is_empty())
    {
        return Some((path, family.unwrap_or("shot")));
    }
    family
        .and_then(|key| sounds.get(key).map(|path| (path.as_str(), key)))
        .or_else(|| sounds.get("shot").map(|path| (path.as_str(), "shot")))
}

/// Son de rechargement : `reloading` du mode actif dans l'`audio_config` de l'arme, sinon
/// `reload`, sinon aucun son (volume : celui de `reload`).
fn reload_sound<'a>(
    sounds: &'a BTreeMap<String, String>,
    weapon_audio: Option<&'a crate::weapons::WeaponModeAudio>,
) -> Option<&'a str> {
    weapon_audio
        .map(|audio| audio.reloading.as_str())
        .filter(|path| !path.is_empty())
        .or_else(|| sounds.get("reload").map(String::as_str))
}

/// Arme active d'un inventaire, vue par les sons : nom, mode actif (nom et genre de tir).
struct ActiveWeaponSound<'a> {
    name: &'a str,
    mode: Option<(&'a str, &'a crate::weapons::FiringMode)>,
}

fn active_weapon_sound<'a>(
    inventory: &'a crate::weapons::WeaponInventory,
    weapon_states: &'a Query<&crate::weapons::WeaponState>,
) -> Option<ActiveWeaponSound<'a>> {
    let (entity, weapon) = inventory.weapons.get(inventory.active_weapon_index)?;
    let mode = weapon_states.get(*entity).ok().and_then(|state| {
        weapon
            .config
            .firing_modes
            .get_key_value(&state.active_mode)
            .map(|(name, mode)| (name.as_str(), &mode.firing_mode))
    });
    Some(ActiveWeaponSound {
        name: weapon.config.name.as_str(),
        mode,
    })
}

/// Sons propres des armes (`audio_config` de `weapons.ron`), lus dans `WeaponRegistry` : le
/// composant rollback `Weapon` ne les porte pas.
#[derive(bevy::ecs::system::SystemParam)]
struct WeaponAudio<'w> {
    weapons: Res<'w, crate::weapons::WeaponRegistry>,
}

impl WeaponAudio<'_> {
    /// Sons du mode `mode` de l'arme `weapon` (aucun si l'arme ou le mode n'en déclare pas).
    fn mode(&self, weapon: &ActiveWeaponSound) -> Option<&crate::weapons::WeaponModeAudio> {
        let (mode, _) = weapon.mode?;
        self.weapons
            .weapons
            .get(weapon.name)?
            .audio_config
            .modes
            .get(mode)
    }
}

/// Système jouant le son de tir pour chaque nouveau tir d'un joueur local.
///
/// Un son par (tireur, frame de création) : le rollback de la session locale détruit puis
/// recrée les balles des dernières frames à chaque image rendue, elles redeviennent
/// `Added<Bullet>` et rejoueraient leur son ; un fusil à pompe ne joue pas un son par plomb.
/// Le son est celui de l'arme active du tireur **à l'image rendue** ([`shot_sound`]) : la balle
/// ne porte pas son arme, lui ajouter un champ ferait bouger les traces.
fn play_shot_sound_for_new_bullets(
    sounds: FeedbackSounds,
    weapon_audio: WeaponAudio,
    local_players: Query<(&GgrsNetId, &crate::weapons::WeaponInventory), With<LocalPlayer>>,
    weapon_states: Query<&crate::weapons::WeaponState>,
    new_bullets: Query<&crate::weapons::Bullet, Added<crate::weapons::Bullet>>,
    mut played: Local<BTreeSet<(usize, u32)>>,
) {
    // Oublie les tirs anciens, et ceux d'une partie précédente (le compteur repart de 0).
    let frame = sounds.frame.frame;
    played.retain(|&(_, created_at)| {
        created_at <= frame && frame - created_at <= SHOT_SOUND_MEMORY_FRAMES
    });

    let Some(cfg) = sounds.config.as_ref() else {
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
        let active = active_weapon_sound(inventory, &weapon_states);
        let own = active.as_ref().and_then(|weapon| weapon_audio.mode(weapon));
        let mode = active
            .as_ref()
            .and_then(|weapon| weapon.mode)
            .map(|(_, m)| m);
        if let Some((path, key)) = shot_sound(&cfg.0.sounds, own, mode) {
            sounds.play_path(path, key);
        }
    }
}

#[cfg(test)]
mod shot_sound_tests {
    use super::*;
    use crate::weapons::{FiringMode, WeaponModeAudio};

    #[test]
    fn arme_puis_famille_puis_repli() {
        let shotgun = FiringMode::Shotgun {
            pellet_count: 8,
            spread_angle: Fixed::ZERO,
        };
        let automatic = FiringMode::Automatic {};
        let mut sounds: BTreeMap<String, String> = [
            ("shot", "repli.ogg"),
            ("shot_shotgun", "pompe.ogg"),
            ("shot_smg", "smg.ogg"),
            ("reload", "recharge.ogg"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        let own = WeaponModeAudio {
            firing: "pistolet.ogg".into(),
            reloading: "chargeur.ogg".into(),
        };
        let silent = WeaponModeAudio::default();
        // Le son du mode de l'arme l'emporte, avec le volume de la famille qu'il remplace.
        assert_eq!(
            shot_sound(&sounds, Some(&own), Some(&automatic)),
            Some(("pistolet.ogg", "shot_smg"))
        );
        assert_eq!(reload_sound(&sounds, Some(&own)), Some("chargeur.ogg"));
        // Champs vides : la famille du mode et `reload`.
        assert_eq!(
            shot_sound(&sounds, Some(&silent), Some(&shotgun)),
            Some(("pompe.ogg", "shot_shotgun"))
        );
        assert_eq!(reload_sound(&sounds, Some(&silent)), Some("recharge.ogg"));
        // Famille absente de `sounds`, ou mode inconnu : repli sur `shot`.
        sounds.remove("shot_smg");
        assert_eq!(
            shot_sound(&sounds, None, Some(&automatic)),
            Some(("repli.ogg", "shot"))
        );
        assert_eq!(shot_sound(&sounds, None, None), Some(("repli.ogg", "shot")));
        // Rien de déclaré : aucun son.
        sounds.clear();
        assert_eq!(shot_sound(&sounds, None, Some(&shotgun)), None);
        assert_eq!(reload_sound(&sounds, None), None);
    }
}

/// Système jouant le son de rechargement quand un joueur local commence à recharger
/// ([`reload_sound`] de son arme active).
///
/// Déclenché une fois, au passage « ne recharge pas » → « recharge » d'une image rendue à
/// l'autre : `reloading_ending_frame` reste `Some` pendant tout le rechargement, et le jouer à
/// chaque image superposait le même son des dizaines de fois.
fn play_reload_sound_for_reload_events(
    sounds: FeedbackSounds,
    weapon_audio: WeaponAudio,
    query: Query<(&crate::weapons::WeaponInventory, &GgrsNetId), With<LocalPlayer>>,
    weapon_states: Query<&crate::weapons::WeaponState>,
    mut reloading: Local<BTreeSet<usize>>,
) {
    let now: BTreeSet<usize> = query
        .iter()
        .filter(|(inv, _)| inv.reloading_ending_frame.is_some())
        .map(|(_, net_id)| net_id.0)
        .collect();
    let started: Vec<usize> = now.difference(&reloading).copied().collect();
    *reloading = now;

    let Some(cfg) = sounds.config.as_ref() else {
        return;
    };
    for (inventory, _) in query
        .iter()
        .filter(|(_, net_id)| started.contains(&net_id.0))
    {
        let active = active_weapon_sound(inventory, &weapon_states);
        let own = active.as_ref().and_then(|weapon| weapon_audio.mode(weapon));
        if let Some(path) = reload_sound(&cfg.0.sounds, own) {
            sounds.play_path(path, "reload");
        }
    }
}

// ---------------------------------------------------------------------------------------
// Sons d'événements (lot B de `docs/plan-assets.md`) : présentation seule, dérivés de l'état
// ou des `FrameEvents` de la dernière frame simulée, jamais écrits dans la simulation.
// ---------------------------------------------------------------------------------------

/// Lecteur des sons de `ui/feedback.ron`, au volume de leur clé (`volumes`, en dB).
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
        self.play_path(path, key);
    }

    /// Joue le fichier `path` au volume de la clé `key` (0 dB si `volumes` ne la règle pas).
    /// `ALACOD_MUTE=1` : rien n'est joué (captures, `play_scenario`, tests sur un poste de
    /// travail) ; le journal garde la ligne `sound`, les moments clés restent détectés.
    fn play_path(&self, path: &str, key: &str) {
        let volume_db = self
            .config
            .as_ref()
            .and_then(|cfg| cfg.0.volumes.get(key).copied())
            .unwrap_or(0.0);
        info!("feedback f{} sound {} {}", self.frame.frame, key, path);
        if muted() {
            return;
        }
        self.audio
            .play(self.asset_server.load(path.to_string()))
            .with_volume(volume_db);
    }
}

/// `ALACOD_MUTE=1` coupe tous les sons de feedback (présentation seule).
pub(crate) fn muted() -> bool {
    std::env::var("ALACOD_MUTE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// `hit` : un coup porté par un joueur local à une autre entité, un son par frame simulée
/// (les plombs d'un fusil à pompe ou une explosion n'en jouent qu'un). Un coup de
/// mêlée (tag `melee` de l'événement) joue `melee` : le `hit` générique, très court
/// et mou, sonnait comme une balle de ping-pong au corps à corps (retour de William,
/// 2026-10-16). Gardé par la frame de
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
    let key = if hit.tags.has(&Tag::new("melee")) {
        "melee"
    } else {
        "hit"
    };
    sounds.play(key);
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
    damage: Res<FrameEvents<DamageEvent>>,
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
    // Un coup de mêlée reçu joue `melee` (play_melee_taken_sound), pas `hurt` : les
    // deux le même tick se masquent et le `hurt` générique sonnait faux au contact.
    let melee_taken = damage.iter().any(|e| {
        now.contains_key(&e.target.0)
            && !now.contains_key(&e.source.0)
            && e.tags.has(&Tag::new("melee"))
    });
    *previous = now;
    let frame = sounds.frame.frame;
    // Une frame plus petite que la dernière jouée : nouvelle partie.
    let ready =
        last_played.is_none_or(|last| frame < last || frame - last >= HURT_SOUND_MIN_FRAMES);
    if hurt && !melee_taken && ready {
        *last_played = Some(frame);
        sounds.play("hurt");
    }
}

/// `melee` reçu : un ennemi frappe un joueur local au corps à corps (événement de
/// dégât tagué `melee` dont la cible est locale). C'est ce son que William entendait
/// « comme une balle de ping-pong » (2026-10-16) : avant, seul le `hurt` générique
/// jouait. Un son par frame d'événement, comme `hit`.
fn play_melee_taken_sound(
    sounds: FeedbackSounds,
    damage: Res<FrameEvents<DamageEvent>>,
    local_players: Query<&GgrsNetId, With<LocalPlayer>>,
    mut last_frame: Local<Option<u32>>,
) {
    let local: BTreeSet<usize> = local_players.iter().map(|n| n.0).collect();
    let Some(event) = damage.iter().find(|e| {
        local.contains(&e.target.0)
            && !local.contains(&e.source.0)
            && e.tags.has(&Tag::new("melee"))
    }) else {
        return;
    };
    if *last_frame == Some(event.frame) {
        return;
    }
    *last_frame = Some(event.frame);
    sounds.play("melee");
}

/// `death` : un ennemi tué, dérivé de l'état dans `Update` (jamais d'un événement de
/// simulation) : un `GgrsNetId` d'ennemi présent à l'image précédente et absent à
/// celle-ci est mort. L'ancienne lecture (`Death` + `RollbackDespawned`) ne voyait
/// rien en partie locale : la frame y est déjà confirmée, `despawn_rollback` détruit
/// l'entité dans la simulation avant que la présentation la voie (constaté en partie
/// réelle le 2026-10-16 : aucun `sound death` au journal malgré les zombies tués).
/// En ligne, la disparition a lieu à la destruction confirmée : même déclencheur.
/// Un son par image, quel que soit le nombre de morts. Un retour de frame (nouvelle
/// partie) réarme l'ensemble sans jouer : les disparitions y sont des déchargements.
fn play_death_sound(
    sounds: FeedbackSounds,
    enemies: Query<&GgrsNetId, With<crate::character::enemy::Enemy>>,
    mut previous: Local<BTreeSet<usize>>,
    mut last_frame: Local<Option<u32>>,
) {
    let now: BTreeSet<usize> = enemies.iter().map(|n| n.0).collect();
    let frame = sounds.frame.frame;
    if last_frame.is_some_and(|last| frame < last) {
        *previous = now;
        *last_frame = Some(frame);
        return;
    }
    *last_frame = Some(frame);
    let died = previous.difference(&now).next().is_some();
    *previous = now;
    if died {
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
