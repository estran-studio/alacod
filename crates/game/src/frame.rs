use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use utils::frame::FrameCount;

use crate::character::enemy::Enemy;
use crate::core::GameInfo;

// You can also register resources.

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

// DEBUG

#[derive(Component)]
struct FrameCountText;

fn setup_frame_counter_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        FrameCountText,
        Text::new("Frame Count"),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(16.0),
            ..Default::default()
        },
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
    ));
}

fn update_frame_counter_text(
    frame_count: Res<FrameCount>,
    game_info: Res<GameInfo>,
    diagnostics: Res<DiagnosticsStore>,
    enemy_query: Query<(), With<Enemy>>,
    mut query: Query<&mut Text, With<FrameCountText>>,
) {
    let enemy_count = enemy_query.iter().count();

    for mut text in &mut query {
        let fps_text = if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                format!("{:>5.1}", value)
            } else {
                "  ...".to_string()
            }
        } else {
            "  ...".to_string()
        };

        text.0 = format!(
            "{} : {:>8} | FPS: {} | E: {}",
            game_info.version, frame_count.frame, fps_text, enemy_count
        );
    }
}

pub struct FrameDebugUIPlugin;

impl Plugin for FrameDebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_frame_counter_ui);
        app.add_systems(Update, update_frame_counter_text);
    }
}
