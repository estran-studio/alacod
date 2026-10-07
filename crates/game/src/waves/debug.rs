//! Debug UI for wave system visualization.

use bevy::prelude::*;

use super::{WaveModeEnabled, WavePhase, WaveState};

/// Marker component for wave debug UI text.
#[derive(Component)]
struct WaveDebugText;

/// Resource to toggle wave debug UI visibility.
#[derive(Resource, Default)]
pub struct WaveDebugEnabled(pub bool);

/// Sets up the wave debug UI (matches existing weapon UI style).
fn setup_wave_debug_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    // Position above weapon UI (which uses bottom 3-37px)
    commands.spawn((
        WaveDebugText,
        Text::new("Wave: --"),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(16.0),
            ..Default::default()
        },
        TextLayout::justify(Justify::Left),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(54.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));
}

/// Updates the wave debug UI text.
fn update_wave_debug_text(
    wave_state: Res<WaveState>,
    wave_mode: Res<WaveModeEnabled>,
    debug_enabled: Res<WaveDebugEnabled>,
    run: Option<Res<run::Run>>,
    mut query: Query<(&mut Text, &mut Visibility), With<WaveDebugText>>,
) {
    // m1-cloture-videos-digest : l'overlay de vagues n'a de sens qu'en mode `Waves` (le runner
    // de scénarios active `WaveModeEnabled` partout) ; présentation seule.
    let waves_run = run
        .as_deref()
        .is_none_or(|run| matches!(run.mode, run::RunMode::Waves { .. }));
    for (mut text, mut visibility) in &mut query {
        // Hide if debug disabled, wave mode disabled, or the run is not a wave run
        if !debug_enabled.0 || !wave_mode.0 || !waves_run {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;

        let phase_str = match wave_state.phase {
            WavePhase::NotStarted => "---",
            WavePhase::GracePeriod => "PREP",
            WavePhase::Spawning => "SPAWN",
            WavePhase::InProgress => "FIGHT",
            WavePhase::WaveComplete => "CLEAR",
        };

        // Compact single-line format matching frame counter style
        // Both Spawned and Kills use wave_enemy_count as denominator for consistency
        text.0 = format!(
            "Wave {:>2} | {} | Spawned: {:>2}/{:<2} | Kills: {:>2}/{:<2} | Total: {:>4}",
            wave_state.current_wave,
            phase_str,
            wave_state.enemies_spawned_this_wave,
            wave_state.wave_enemy_count,
            wave_state.wave_enemies_killed,
            wave_state.wave_enemy_count,
            wave_state.total_enemies_killed,
        );
    }
}

/// Toggles wave debug UI with F3 key.
fn toggle_wave_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_enabled: ResMut<WaveDebugEnabled>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_enabled.0 = !debug_enabled.0;
        info!(
            "Wave debug UI: {}",
            if debug_enabled.0 { "ON" } else { "OFF" }
        );
    }
}

/// Plugin that adds wave debug UI.
pub struct WaveDebugPlugin;

impl Plugin for WaveDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveDebugEnabled>();
        app.add_systems(Startup, setup_wave_debug_ui);
        app.add_systems(Update, (toggle_wave_debug, update_wave_debug_text));
    }
}
