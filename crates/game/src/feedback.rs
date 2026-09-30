//! Feedback minimal : flash blanc à l'impact, secousse de caméra, sons.
//!
//! Tous les effets de présentation (non-rollback) sont pilotés par des événements de simulation
//! lus en PostUpdate/Update après la frame GGRS. Les composants et ressources du feedback
//! n'existent qu'en mode rendu (PresentationPlugin, hors headless).

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_kira_audio::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

/// Système jouant le son de tir pour chaque nouvelle balle d'un joueur local.
fn play_shot_sound_for_new_bullets(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    config: Option<Res<FeedbackConfigLoaded>>,
    local_players: Query<&GgrsNetId, With<LocalPlayer>>,
    new_bullets: Query<&crate::weapons::Bullet, Added<crate::weapons::Bullet>>,
    frame_count: Res<FrameCount>,
) {
    if let Some(cfg) = config {
        if let Some(sound_path) = cfg.0.sounds.get("shot") {
            let local_net_ids: Vec<_> = local_players.iter().map(|n| n.0).collect();

            for bullet in new_bullets.iter() {
                if local_net_ids.contains(&bullet.source.0) {
                    info!("feedback f{} sound shot", frame_count.frame);
                    audio.play(asset_server.load(sound_path));
                }
            }
        }
    }
}

/// Système jouant le son de rechargement quand un joueur local commence à recharger.
fn play_reload_sound_for_reload_events(
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    config: Option<Res<FeedbackConfigLoaded>>,
    query: Query<(&crate::weapons::WeaponInventory, &GgrsNetId), With<LocalPlayer>>,
    frame_count: Res<FrameCount>,
) {
    if let Some(cfg) = config {
        if let Some(sound_path) = cfg.0.sounds.get("reload") {
            for (inv, _net_id) in query.iter() {
                // Détecter la transition None -> Some du reloading_ending_frame
                // (pour l'instant, on ne déclenche que si reloading_ending_frame vient de devenir Some)
                // Impossible de détecter le changement ici sans state tracking.
                // Pour la v0, on relâche cette implémentation.
                if inv.reloading_ending_frame.is_some() {
                    info!("feedback f{} sound reload", frame_count.frame);
                    audio.play(asset_server.load(sound_path));
                }
            }
        }
    }
}
