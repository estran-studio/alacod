pub mod ui;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_kira_audio::SpatialAudioReceiver;

#[cfg(feature = "lighting")]
use bevy_light_2d::light::AmbientLight2d;

use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use ui::CameraDebugUIPlugin;

use crate::character::player::{control::PlayerAction, LocalPlayer, Player};

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct CameraSettingsAsset(pub CameraSettings);

// Plugin to add all camera systems
pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>()
            .add_plugins(CameraDebugUIPlugin)
            .add_plugins(RonAssetPlugin::<CameraSettingsAsset>::new(&[".ron"]))
            .add_systems(Startup, (setup_camera, load_camera_settings))
            .add_systems(
                Update,
                (
                    character_visuals_update_system,
                    camera_control_system,
                    player_indicator_system,
                    camera_input_system,
                ),
            );
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CameraSettings {
    // How fast the camera moves in free mode
    pub free_move_speed: f32,
    // The edge size for mouse detection in free mode (0.0 to 1.0)
    pub edge_margin: f32,
    // How quickly the camera interpolates to target positions
    pub lerp_speed: f32,
    // Maximum zoom out in players lock mode
    pub max_zoom_out: f32,
    // Minimum zoom level
    pub min_zoom: f32,
    // Default zoom level for player mode
    pub default_player_zoom: f32,
    // Padding around all players for players lock mode
    pub player_padding: f32,
    // Arrow indicator size
    pub indicator_size: f32,
    // Arrow distance from screen edge
    pub indicator_edge_distance: f32,
    // Whether to use screen edge detection for camera movement
    pub use_edge_detection: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            free_move_speed: 500.0,
            edge_margin: 0.05,
            lerp_speed: 5.0,
            max_zoom_out: 15.0,
            min_zoom: 5.0,
            default_player_zoom: 5.0, // Default zoom when in player mode
            player_padding: 100.0,
            indicator_size: 20.0,
            indicator_edge_distance: 20.0,
            use_edge_detection: true,
        }
    }
}

// Track which camera mode is active
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    #[default]
    PlayerLock,
    PlayersLock,
    Unlock,
}

// Component to mark the camera entity
#[derive(Component, Default)]
pub struct GameCamera {
    pub mode: CameraMode,
    pub target_player_id: Option<Entity>,
    pub target_position: Vec2,
    pub target_zoom: f32,
}

// Marker for indicator arrows
#[derive(Component)]
pub struct PlayerIndicator {
    pub player_entity: Entity,
}

/// Actions du joueur local qui contrôle la caméra : celui au plus petit handle
/// (plusieurs joueurs locaux partagent une session synctest).
fn camera_owner_actions<'a>(
    action_query: &'a Query<(&Player, &ActionState<PlayerAction>), With<LocalPlayer>>,
) -> Option<&'a ActionState<PlayerAction>> {
    action_query
        .iter()
        .min_by_key(|(player, _)| player.handle)
        .map(|(_, actions)| actions)
}

// System to handle camera input
fn camera_input_system(
    action_query: Query<(&Player, &ActionState<PlayerAction>), With<LocalPlayer>>,
    mut camera_query: Query<&mut GameCamera>,
    _player_query: Query<(Entity, &Player)>,
) {
    let Some(action_state) = camera_owner_actions(&action_query) else {
        return;
    };

    let mut camera = if let Ok(cam) = camera_query.single_mut() {
        cam
    } else {
        return;
    };

    // Handle mode switching
    if action_state.just_pressed(&PlayerAction::SwitchLockMode) {
        if matches!(camera.mode, CameraMode::PlayerLock) {
            camera.mode = CameraMode::PlayersLock;
        } else {
            camera.mode = CameraMode::PlayerLock;
        }
    } else if action_state.just_pressed(&PlayerAction::SwitchToUnlockMode) {
        if matches!(camera.mode, CameraMode::Unlock) {
            camera.mode = CameraMode::PlayerLock;
        } else {
            camera.mode = CameraMode::Unlock;
        }
    }

    // Handle player switching in PlayerLock mode
    /*
    if action_state.just_pressed(PlayerAction::SwitchTargetPlayer) && camera.mode == CameraMode::PlayerLock {
        // Collect all player entities
        let players: Vec<Entity> = player_query
            .iter()
            .map(|(entity, _)| entity)
            .collect();

        if players.is_empty() {
            return;
        }

        // Find the index of the current target
        let current_index = if let Some(target) = camera.target_player_id {
            players.iter().position(|&p| p == target).unwrap_or(0)
        } else {
            0
        };

        // Switch to the next player (or wrap around)
        let next_index = (current_index + 1) % players.len();
        camera.target_player_id = Some(players[next_index]);
    }
    */
}

// Main camera control system
fn camera_control_system(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    windows: Query<&Window>,
    action_query: Query<(&Player, &ActionState<PlayerAction>), With<LocalPlayer>>,
    mut camera_query: Query<
        (&mut GameCamera, &mut Transform, &mut Projection),
        Without<Player>,
    >,
    player_query: Query<(Entity, &Transform, &Player, Option<&LocalPlayer>), Without<GameCamera>>,
) {
    // Get the primary window for dimensions
    let window = windows.single().unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    // Get mouse position normalized to -1.0 to 1.0 range
    let mouse_position = if let Some(position) = window.cursor_position() {
        Vec2::new(
            (position.x / window.width()) * 2.0 - 1.0,
            ((window.height() - position.y) / window.height()) * 2.0 - 1.0,
        )
    } else {
        Vec2::ZERO
    };

    let Some(action_state) = camera_owner_actions(&action_query) else {
        return;
    };

    let (mut camera, mut camera_transform, mut projection) =
        if let Ok(cam) = camera_query.single_mut() {
            cam
        } else {
            return;
        };

    // Find the local player if not already set (smallest handle)
    if camera.target_player_id.is_none() {
        camera.target_player_id = player_query
            .iter()
            .filter(|(_, _, _, local)| local.is_some())
            .min_by_key(|(_, _, player, _)| player.handle)
            .map(|(entity, _, _, _)| entity);
    }

    // Calculate target position and zoom based on camera mode
    match camera.mode {
        CameraMode::PlayerLock => {
            if let Some(target_entity) = camera.target_player_id {
                if let Ok((_, transform, _, _)) = player_query.get(target_entity) {
                    // Lock to the target player - explicitly set both X and Y
                    let player_pos = transform.translation.truncate();
                    camera.target_position.x = player_pos.x;
                    camera.target_position.y = player_pos.y;
                    camera.target_zoom = settings.min_zoom;
                }
            }
        }
        CameraMode::PlayersLock => {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut player_count = 0;

            // Find bounds of all players
            for (_, transform, _, _) in player_query.iter() {
                let pos = transform.translation.truncate();
                min_x = min_x.min(pos.x);
                max_x = max_x.max(pos.x);
                min_y = min_y.min(pos.y);
                max_y = max_y.max(pos.y);
                player_count += 1;
            }

            if player_count > 0 {
                // Add padding
                min_x -= settings.player_padding;
                max_x += settings.player_padding;
                min_y -= settings.player_padding;
                max_y += settings.player_padding;

                // Calculate the center and size
                let center = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
                let size = Vec2::new(max_x - min_x, max_y - min_y);

                // Calculate required zoom to fit all players
                let width_ratio = size.x / window_size.x;
                let height_ratio = size.y / window_size.y;

                // Take the larger ratio to ensure both dimensions fit
                let required_zoom = width_ratio.max(height_ratio);

                // Add a small margin factor
                let target_zoom = required_zoom * 1.1;

                // Clamp to our zoom limits
                camera.target_zoom = target_zoom.clamp(settings.min_zoom, settings.max_zoom_out);

                // Always center on all players
                camera.target_position = center;
            }
        }
        CameraMode::Unlock => {
            let mut move_dir = Vec2::ZERO;

            // Handle keyboard input for camera movement
            if action_state.pressed(&PlayerAction::MoveCameraUp) {
                move_dir.y += 1.0;
            }
            if action_state.pressed(&PlayerAction::MoveCameraDown) {
                move_dir.y -= 1.0;
            }
            if action_state.pressed(&PlayerAction::MoveCameraLeft) {
                move_dir.x -= 1.0;
            }
            if action_state.pressed(&PlayerAction::MoveCameraRight) {
                move_dir.x += 1.0;
            }

            // Also handle edge detection if enabled
            if settings.use_edge_detection {
                let edge_margin = settings.edge_margin;

                if mouse_position.x > 1.0 - edge_margin {
                    move_dir.x += 1.0;
                }
                if mouse_position.x < -1.0 + edge_margin {
                    move_dir.x -= 1.0;
                }
                if mouse_position.y > 1.0 - edge_margin {
                    move_dir.y += 1.0;
                }
                if mouse_position.y < -1.0 + edge_margin {
                    move_dir.y -= 1.0;
                }
            }

            // Normalize direction if moving diagonally
            if move_dir.length_squared() > 0.0 {
                move_dir = move_dir.normalize();
            }

            // Apply movement
            let delta = move_dir * settings.free_move_speed * time.delta().as_secs_f32();
            camera.target_position += delta;

            // Keep zoom at min level in free mode
            camera.target_zoom = settings.min_zoom;
        }
    }

    // Smoothly interpolate camera position
    let current_pos = camera_transform.translation.truncate();
    let lerp_factor = settings.lerp_speed * time.delta().as_secs_f32();

    // Explicitly apply lerp to both X and Y components
    let new_x = current_pos.x + (camera.target_position.x - current_pos.x) * lerp_factor;
    let new_y = current_pos.y + (camera.target_position.y - current_pos.y) * lerp_factor;

    camera_transform.translation.x = new_x;
    camera_transform.translation.y = new_y;

    // Apply zoom by updating the orthographic projection scale
    // This is what controls how much of the world is visible
    match projection.as_mut() {
        Projection::Orthographic(orth) => {
            let current_zoom = orth.scale;
            let new_zoom = current_zoom + (camera.target_zoom - current_zoom) * lerp_factor;
            orth.scale = new_zoom;
        },
        _ => {}
    };
}

// System to handle player indicators for off-screen players
fn player_indicator_system(
    mut commands: Commands,
    settings: Res<CameraSettings>,
    windows: Query<&Window>,
    camera_query: Query<(&GameCamera, &Transform, &Projection)>,
    player_query: Query<(Entity, &Transform, &Player)>,
    indicator_query: Query<Entity, With<PlayerIndicator>>,
) {
    // Remove all existing indicators
    for entity in indicator_query.iter() {
        commands.entity(entity).despawn();
    }

    let window = windows.single().unwrap();
    let (camera, camera_transform, projection) = if let Ok(cam) = camera_query.single() {
        cam
    } else {
        return;
    };

    let projection = match projection {
        Projection::Orthographic(o) => o,
        _ => {
            return;
        }
    };

    // Calculate visible screen rectangle in world space
    let camera_pos = camera_transform.translation.truncate();
    let half_width = (window.width() * projection.scale) / 2.0;
    let half_height = (window.height() * projection.scale) / 2.0;

    let visible_rect = Rect {
        min: Vec2::new(camera_pos.x - half_width, camera_pos.y - half_height),
        max: Vec2::new(camera_pos.x + half_width, camera_pos.y + half_height),
    };

    // Check each player and create indicators for those off-screen
    for (player_entity, player_transform, player_info) in player_query.iter() {
        // Skip the currently targeted player in player lock mode
        if camera.mode == CameraMode::PlayerLock && camera.target_player_id == Some(player_entity) {
            continue;
        }

        let player_pos = player_transform.translation.truncate();

        // Check if player is outside the visible area
        if !visible_rect.contains(player_pos) {
            // Determine where to place the indicator based on player position relative to screen

            // Calculate relative position to screen
            let relative_pos = player_pos - camera_pos;
            let angle_to_player = relative_pos.y.atan2(relative_pos.x);

            // Determine which edge to place the indicator on
            // We'll use the angle to determine the closest edge

            // Calculate aspect ratio to handle rectangular screens properly
            let aspect_ratio = window.width() / window.height();

            // Calculate normalized slope based on angle
            let angle_tangent = angle_to_player.tan();
            let _normalized_tangent = angle_tangent / aspect_ratio;

            // Determine which side of the screen to place the indicator
            let (indicator_pos, final_angle) = if angle_to_player.abs() < std::f32::consts::PI / 4.0
            {
                // Right side of screen
                (
                    Vec2::new(
                        visible_rect.max.x - settings.indicator_edge_distance,
                        camera_pos.y + (visible_rect.max.x - camera_pos.x) * angle_tangent,
                    ),
                    angle_to_player,
                )
            } else if angle_to_player.abs() > 3.0 * std::f32::consts::PI / 4.0 {
                // Left side of screen
                (
                    Vec2::new(
                        visible_rect.min.x + settings.indicator_edge_distance,
                        camera_pos.y + (camera_pos.x - visible_rect.min.x) * angle_tangent,
                    ),
                    angle_to_player,
                )
            } else if angle_to_player > 0.0 {
                // Top side of screen
                (
                    Vec2::new(
                        camera_pos.x + (visible_rect.max.y - camera_pos.y) / angle_tangent,
                        visible_rect.max.y - settings.indicator_edge_distance,
                    ),
                    angle_to_player,
                )
            } else {
                // Bottom side of screen
                (
                    Vec2::new(
                        camera_pos.x + (camera_pos.y - visible_rect.min.y) / angle_tangent,
                        visible_rect.min.y + settings.indicator_edge_distance,
                    ),
                    angle_to_player,
                )
            };

            // Make sure the indicator is within the screen bounds
            let clamped_pos = Vec2::new(
                indicator_pos.x.clamp(
                    visible_rect.min.x + settings.indicator_edge_distance,
                    visible_rect.max.x - settings.indicator_edge_distance,
                ),
                indicator_pos.y.clamp(
                    visible_rect.min.y + settings.indicator_edge_distance,
                    visible_rect.max.y - settings.indicator_edge_distance,
                ),
            );

            // Spawn the indicator
            commands.spawn((
                Sprite {
                    color: player_info.color,
                    custom_size: Some(Vec2::splat(settings.indicator_size)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(clamped_pos.x, clamped_pos.y, 10.0))
                    .with_rotation(Quat::from_rotation_z(final_angle)),
                PlayerIndicator { player_entity },
            ));
        }
    }
}

/// Garde le handle de `camera.ron` chargé ; les réglages sont appliqués à la réception
/// de l'asset (et à chaque modification) par `character_visuals_update_system`.
#[derive(Resource)]
struct CameraSettingsHandle(#[allow(dead_code)] Handle<CameraSettingsAsset>);

fn load_camera_settings(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CameraSettingsHandle(asset_server.load("camera.ron")));
}

fn character_visuals_update_system(
    mut ev_asset: MessageReader<AssetEvent<CameraSettingsAsset>>,
    camera_asset: Res<Assets<CameraSettingsAsset>>,
    mut r_camera: ResMut<CameraSettings>,
    _asset_server: Res<AssetServer>,
    _camera_query: Query<(&mut GameCamera, &mut Transform, &mut Projection)>,
) {
    for event in ev_asset.read() {
        if let AssetEvent::Added { id } = event {
            if let Some(camera_settings) = camera_asset.get(*id) {
                *r_camera = camera_settings.0.clone();
            }
        }
        if let AssetEvent::Modified { id } = event {
            if let Some(camera_settings) = camera_asset.get(*id) {
                *r_camera = camera_settings.0.clone();
            }
        }
    }
}

// Helper struct for defining a 2D rectangle
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

// Example of how to set up the camera in your game
pub fn setup_camera(mut commands: Commands, settings: Res<CameraSettings>) {
    // Spawn the camera itself
    #[cfg(feature = "lighting")]
    {
        commands.spawn((
            Camera2d::default(),
            SpatialAudioReceiver,
            AmbientLight2d{
                brightness: 0.4,
                ..Default::default()
            },
            GameCamera {
                mode: CameraMode::PlayerLock,
                target_player_id: None,
                target_position: Vec2::ZERO,
                target_zoom: settings.default_player_zoom, // Use the default player zoom
            },
        ));
    }
    
    #[cfg(not(feature = "lighting"))]
    {
        commands.spawn((
            Camera2d::default(),
            SpatialAudioReceiver,
            GameCamera {
                mode: CameraMode::PlayerLock,
                target_player_id: None,
                target_position: Vec2::ZERO,
                target_zoom: settings.default_player_zoom, // Use the default player zoom
            },
        ));
    }
}
