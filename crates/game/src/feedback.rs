//! Feedback minimal : flash blanc à l'impact, secousse de caméra, sons.
//!
//! Tous les effets de présentation (non-rollback) sont pilotés par des événements de simulation
//! lus en PostUpdate/Update après la frame GGRS. Les composants et ressources du feedback
//! n'existent qu'en mode rendu (PresentationPlugin, hors headless).

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_kira_audio::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;
use utils::{frame::FrameCount, net_id::GgrsNetId};

use crate::character::player::{LocalPlayer, Player};
use crate::core::is_headless;
use crate::frame_events::FrameEvents;
use sim_core::damage::DamageEvent;

/// Configuration du feedback chargée depuis `feedback.ron`.
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct FeedbackConfig {
    pub hit_flash: HitFlashConfig,
    pub shake: ShakeConfig,
    pub sounds: BTreeMap<String, String>, // Clés : "shot", "reload", etc. Valeurs : chemins sons
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HitFlashConfig {
    pub frames: u32,
    pub color: (f32, f32, f32),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShakeConfig {
    pub frames: u32,
    pub amplitude: f32,
}

/// Composant non-rollback marquant une entité en flash blanc.
/// Appliqué en PostUpdate quand un DamageEvent cible cette entité.
#[derive(Component, Debug)]
pub struct HitFlash {
    pub until_frame: u32,
    pub original_color: Color,
}

/// Composant non-rollback appliqué à la caméra pour une secousse.
/// Appliqué en PostUpdate quand un joueur local reçoit des dégâts.
#[derive(Component, Debug)]
pub struct CameraShake {
    pub until_frame: u32,
    pub amplitude: f32,
}

/// Ressource contenant la FeedbackConfig chargée.
#[derive(Resource, Debug, Clone)]
pub struct FeedbackConfigLoaded(pub FeedbackConfig);

/// Plugin de feedback : charge config RON, enregistre les systèmes.
pub struct FeedbackPlugin;

impl Plugin for FeedbackPlugin {
    fn build(&self, app: &mut App) {
        // Pas de plugin si headless
        if is_headless() {
            return;
        }

        app.add_plugins(RonAssetPlugin::<FeedbackConfig>::new(&["ron"]));

        // Charger la config au démarrage
        app.add_systems(Startup, load_feedback_config);
        // Publie `FeedbackConfigLoaded` une fois l'asset chargé (et à chaque modification) :
        // sans ce système, aucun effet ne s'appliquait jamais (config toujours absente).
        app.add_systems(Update, publish_loaded_feedback_config);

        // Systèmes de présentation (PostUpdate, hors GgrsSchedule)
        app.add_systems(
            PostUpdate,
            (
                apply_hit_flash_from_damage_events,
                update_hit_flash_colors.after(apply_hit_flash_from_damage_events),
                cleanup_expired_hit_flash.after(update_hit_flash_colors),
                apply_camera_shake_from_damage_events,
                // Avant la propagation des transforms : sinon le décalage n'est appliqué au
                // `Transform` qu'après le calcul du `GlobalTransform` rendu, et le suivi de
                // caméra (`Update`) l'écrase à la frame suivante : la secousse n'est jamais vue.
                update_camera_shake_offset
                    .after(apply_camera_shake_from_damage_events)
                    .before(bevy::transform::TransformSystems::Propagate),
                cleanup_expired_camera_shake.after(update_camera_shake_offset),
                play_shot_sound_for_new_bullets,
                play_reload_sound_for_reload_events,
            ),
        );
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
            commands.insert_resource(FeedbackConfigLoaded(config.clone()));
        }
    }
}

/// Système appliquant HitFlash selon les DamageEvents de la frame courante.
fn apply_hit_flash_from_damage_events(
    mut commands: Commands,
    damages: Res<FrameEvents<DamageEvent>>,
    frame_count: Res<FrameCount>,
    config: Option<Res<FeedbackConfigLoaded>>,
    target_query: Query<(&GgrsNetId, Entity, &Sprite)>,
) {
    if let Some(cfg) = config {
        let flash_until = frame_count.frame + cfg.0.hit_flash.frames;

        // Construire map net_id -> Entity
        let mut entity_by_net_id: BTreeMap<usize, Entity> = BTreeMap::new();
        for (net_id, entity, _sprite) in target_query.iter() {
            entity_by_net_id.insert(net_id.0, entity);
        }

        // Parcourir tous les DamageEvents de la frame courante
        for damage_event in damages.iter() {
            info!(
                "feedback f{} flash {}",
                frame_count.frame, damage_event.target.0
            );

            // Chercher l'entité cible via net_id
            if let Some(entity) = entity_by_net_id.get(&damage_event.target.0) {
                commands.entity(*entity).insert(HitFlash {
                    until_frame: flash_until,
                    original_color: Color::WHITE,
                });
            }
        }
    }
}

/// Système mettant à jour les teintes des sprites avec HitFlash.
fn update_hit_flash_colors(mut query: Query<(&mut Sprite, &HitFlash)>) {
    for (mut sprite, _flash) in query.iter_mut() {
        sprite.color = Color::WHITE;
    }
}

/// Système nettoyant les composants HitFlash expirés.
fn cleanup_expired_hit_flash(
    mut commands: Commands,
    query: Query<(Entity, &HitFlash)>,
    frame_count: Res<FrameCount>,
) {
    for (entity, flash) in query.iter() {
        if frame_count.frame >= flash.until_frame {
            commands.entity(entity).remove::<HitFlash>();
            // Restaurer la couleur d'origine (mais en l'absence de tracking séparé,
            // on se contente de supprimer le composant)
        }
    }
}

/// Système appliquant CameraShake quand un joueur local prend des dégâts.
fn apply_camera_shake_from_damage_events(
    mut commands: Commands,
    damages: Res<FrameEvents<DamageEvent>>,
    frame_count: Res<FrameCount>,
    config: Option<Res<FeedbackConfigLoaded>>,
    local_players: Query<(&GgrsNetId, &Player), With<LocalPlayer>>,
    // La caméra de jeu (`GameCamera`, suivi des joueurs), pas n'importe quelle `Camera` : en
    // capture il en existe plusieurs et secouer la mauvaise ne se voit jamais.
    camera_query: Query<Entity, With<crate::camera::GameCamera>>,
) {
    if let Some(cfg) = config {
        let shake_until = frame_count.frame + cfg.0.shake.frames;

        for damage_event in damages.iter() {
            // Vérifier si la cible est un joueur local
            for (net_id, _player) in local_players.iter() {
                if *net_id == damage_event.target {
                    info!(
                        "feedback f{} shake {}",
                        frame_count.frame, damage_event.target.0
                    );

                    // Appliquer CameraShake à la caméra
                    if let Some(camera_entity) = camera_query.iter().next() {
                        commands.entity(camera_entity).insert(CameraShake {
                            until_frame: shake_until,
                            amplitude: cfg.0.shake.amplitude,
                        });
                    }
                }
            }
        }
    }
}

/// Système mettant à jour le décalage de caméra pour la secousse.
/// Applique un motif déterministe indexé par frame.
fn update_camera_shake_offset(
    mut query: Query<(&mut Transform, &CameraShake)>,
    frame_count: Res<FrameCount>,
) {
    for (mut transform, shake) in query.iter_mut() {
        if frame_count.frame < shake.until_frame {
            let frames_remaining = (shake.until_frame - frame_count.frame) as f32;
            // Amplitude décroissante : motif déterministe
            let decay = 1.0 - (frames_remaining / (shake.amplitude as f32 + 1.0)).min(1.0);
            let offset = shake.amplitude
                * decay
                * if (frame_count.frame % 2) == 0 {
                    1.0
                } else {
                    -1.0
                };

            if (frame_count.frame / 2) % 2 == 0 {
                transform.translation.x += offset;
            } else {
                transform.translation.y += offset;
            }
        }
    }
}

/// Système nettoyant les composants CameraShake expirés.
fn cleanup_expired_camera_shake(
    mut commands: Commands,
    query: Query<(Entity, &CameraShake)>,
    frame_count: Res<FrameCount>,
) {
    for (entity, shake) in query.iter() {
        if frame_count.frame >= shake.until_frame {
            commands.entity(entity).remove::<CameraShake>();
        }
    }
}

/// Un son de tir est coupé (en fondu) au bout de cette durée. `sounds/machine-gun.ogg` est un
/// enregistrement de tir soutenu de 17 s, pas un coup unique : joué en entier à chaque balle, il
/// se superposait des dizaines de fois. À retirer quand un échantillon de coup unique le remplace.
const SHOT_SOUND_MAX: Duration = Duration::from_millis(250);
const SHOT_SOUND_FADE: Duration = Duration::from_millis(40);
/// Frames pendant lesquelles un tir déjà joué reste mémorisé (`ShotSounds::played`).
const SHOT_SOUND_MEMORY_FRAMES: u32 = 120;

/// État du système de son de tir (présentation, hors rollback).
#[derive(Default)]
struct ShotSounds {
    /// Tirs déjà joués, par (tireur, frame de création). Le rollback de la session locale détruit
    /// puis recrée les balles des dernières frames à chaque image rendue : elles redeviennent
    /// `Added<Bullet>` et rejoueraient leur son. Une clé par tir et non par balle : un fusil à
    /// pompe ne joue pas un son par plomb.
    played: BTreeSet<(usize, u32)>,
    /// Instances lancées avec leur échéance (secondes de `Time<Real>`), à couper en fondu.
    playing: Vec<(Handle<AudioInstance>, f64)>,
}

/// Système jouant le son de tir pour chaque nouveau tir d'un joueur local.
#[allow(clippy::too_many_arguments)]
fn play_shot_sound_for_new_bullets(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut instances: ResMut<Assets<AudioInstance>>,
    time: Res<Time<Real>>,
    config: Option<Res<FeedbackConfigLoaded>>,
    local_players: Query<&GgrsNetId, With<LocalPlayer>>,
    new_bullets: Query<&crate::weapons::Bullet, Added<crate::weapons::Bullet>>,
    frame_count: Res<FrameCount>,
    mut shots: Local<ShotSounds>,
) {
    let now = time.elapsed_secs_f64();
    shots.playing.retain(|(handle, deadline)| {
        if now < *deadline {
            return true;
        }
        match instances.get_mut_untracked(handle) {
            Some(instance) => {
                instance.stop(AudioTween::linear(SHOT_SOUND_FADE));
                false
            }
            // L'instance n'existe pas encore (la lecture est traitée à l'image suivante) ;
            // une seconde après l'échéance, elle est finie depuis longtemps.
            None => now < *deadline + 1.0,
        }
    });

    // Oublie les tirs anciens, et ceux d'une partie précédente (le compteur repart de 0).
    let frame = frame_count.frame;
    shots.played.retain(|&(_, created_at)| {
        created_at <= frame && frame - created_at <= SHOT_SOUND_MEMORY_FRAMES
    });

    let Some(cfg) = config else {
        return;
    };
    let Some(sound_path) = cfg.0.sounds.get("shot") else {
        return;
    };
    let local_net_ids: Vec<_> = local_players.iter().map(|n| n.0).collect();

    for bullet in new_bullets.iter() {
        if local_net_ids.contains(&bullet.source.0)
            && shots.played.insert((bullet.source.0, bullet.created_at))
        {
            info!("feedback f{} sound shot", frame);
            let handle = audio.play(asset_server.load(sound_path)).handle();
            shots
                .playing
                .push((handle, now + SHOT_SOUND_MAX.as_secs_f64()));
        }
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
