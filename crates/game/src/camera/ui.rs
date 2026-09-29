use bevy::prelude::*;
use bevy::text::Justify;

use crate::camera::{CameraMode, GameCamera};

// Marker component for camera debug text
#[derive(Component)]
struct CameraDebugText;

// Setup system for camera debug UI
fn setup_camera_debug_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands.spawn((
        CameraDebugText,
        Text::new("Camera: Mode, Zoom"),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(16.0),
            ..Default::default()
        },
        TextLayout::justify(Justify::Left),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));
}

// Update system for camera debug text
fn update_camera_debug_text(
    camera_query: Query<(&GameCamera, &Projection)>,
    mut text_query: Query<&mut Text, With<CameraDebugText>>,
) {
    // Get camera component and projection
    if let Ok((camera, projection)) = camera_query.single() {
        // Get the text component
        if let Ok(mut text) = text_query.single_mut() {
            // Format mode as a string
            let mode_str = match camera.mode {
                CameraMode::PlayerLock => "PlayerLock",
                CameraMode::PlayersLock => "PlayersLock",
                CameraMode::Unlock => "Unlock",
            };

            // Update the text
            text.0 = format!(
                "Camera: {} | Zoom: {:.2} | Target: {:.2} | Pos: ({:.1}, {:.1})",
                mode_str,
                match projection {
                    Projection::Orthographic(o) => o.scale,
                    _ => 0.,
                },
                camera.target_zoom,
                camera.target_position.x,
                camera.target_position.y
            );
        }
    }
}

// Plugin to add camera debug UI
pub struct CameraDebugUIPlugin;

impl Plugin for CameraDebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera_debug_ui)
            .add_systems(Update, update_camera_debug_text);
    }
}
