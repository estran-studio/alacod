use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};
use bevy_fixed::fixed_math;

use crate::core::AppState;
use crate::character::health::Health;
use crate::character::player::{Player, LocalPlayer};
use crate::character::enemy::Enemy;
use crate::waves::state::WaveState;
use crate::weapons::{Weapon, WeaponState, WeaponModesState};

/// HUD Root marker component
#[derive(Component)]
pub struct HudRoot;

/// Widget marker components for updates
#[derive(Component)]
pub struct HudBarWidget {
    pub source: String,
    pub full_width: f32,
}

#[derive(Component)]
pub struct HudTextWidget {
    pub source: String,
}

/// Position anchor for HUD widgets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// HUD widget kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HudWidgetKind {
    Bar { source: String },
    Text { source: String, prefix: Option<String> },
}

/// A single HUD widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudWidget {
    pub kind: HudWidgetKind,
    pub anchor: HudAnchor,
    pub offset: (f32, f32),
    pub size: Option<(f32, f32)>,
    pub color: Option<String>,
    pub font_size: Option<f32>,
}

/// HUD configuration asset
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct HudConfig {
    pub widgets: Vec<HudWidget>,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<HudConfig>::new(&[".ron"]))
            .init_resource::<HudConfigHandle>()
            .add_systems(OnEnter(AppState::InGame), load_hud_config)
            .add_systems(Update, spawn_hud_when_ready.run_if(in_state(AppState::InGame)))
            .add_systems(Update, update_hud_values.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                handle_hud_config_changes.run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), despawn_hud);
    }
}

/// Resource to hold the HUD config asset handle
#[derive(Resource, Default)]
struct HudConfigHandle(Option<Handle<HudConfig>>);

/// Load the HUD config handle
fn load_hud_config(
    asset_server: Res<AssetServer>,
    mut config_handle: ResMut<HudConfigHandle>,
) {
    config_handle.0 = Some(asset_server.load("ui/hud.ron"));
}

/// Spawn the HUD UI tree when config is ready
fn spawn_hud_when_ready(
    mut commands: Commands,
    config_handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    q_hud_root: Query<Entity, With<HudRoot>>,
) {
    if q_hud_root.is_empty() {
        if let Some(handle) = &config_handle.0 {
            if let Some(config) = configs.get(handle) {
                let entity = commands.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    HudRoot,
                )).id();
                spawn_hud_widgets(&mut commands, entity, config);
            }
        }
    }
}

/// Convert anchor enum to bevy UI layout properties
fn anchor_to_node(anchor: HudAnchor, offset: (f32, f32), size: (f32, f32)) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(size.0),
        height: Val::Px(size.1),
        ..default()
    };

    match anchor {
        HudAnchor::TopLeft => {
            node.left = Val::Px(offset.0);
            node.top = Val::Px(offset.1);
        }
        HudAnchor::TopCenter => {
            node.left = Val::Percent(50.0);
            node.margin.left = Val::Px(-size.0 / 2.0);
            node.top = Val::Px(offset.1);
        }
        HudAnchor::TopRight => {
            node.right = Val::Px(offset.0);
            node.top = Val::Px(offset.1);
        }
        HudAnchor::BottomLeft => {
            node.left = Val::Px(offset.0);
            node.bottom = Val::Px(offset.1);
        }
        HudAnchor::BottomCenter => {
            node.left = Val::Percent(50.0);
            node.margin.left = Val::Px(-size.0 / 2.0);
            node.bottom = Val::Px(offset.1);
        }
        HudAnchor::BottomRight => {
            node.right = Val::Px(offset.0);
            node.bottom = Val::Px(offset.1);
        }
    }

    node
}

fn spawn_hud_widgets(commands: &mut Commands, entity: Entity, config: &HudConfig) {
    commands.entity(entity).with_children(|parent| {
        for widget in &config.widgets {
            let size = widget.size.unwrap_or((100.0, 20.0));
            let node = anchor_to_node(widget.anchor, widget.offset, size);
            let font_size = widget.font_size.unwrap_or(16.0);
            let color = parse_color(widget.color.as_deref().unwrap_or("#ffffff"));

            match &widget.kind {
                HudWidgetKind::Bar { source } => {
                    parent.spawn((
                        node,
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        HudBarWidget {
                            source: source.clone(),
                            full_width: size.0,
                        },
                    ));
                }
                HudWidgetKind::Text { source, prefix } => {
                    let text = prefix.clone().unwrap_or_default();
                    parent.spawn((
                        node,
                        Text::new(text),
                        TextFont {
                            font_size: FontSize::Px(font_size),
                            ..default()
                        },
                        TextColor(color),
                        HudTextWidget {
                            source: source.clone(),
                        },
                    ));
                }
            }
        }
    });
}

/// Update HUD values from game state
fn update_hud_values(
    local_players: Query<(&Health, &Player), With<LocalPlayer>>,
    all_players: Query<(), With<Player>>,
    enemies: Query<(), With<Enemy>>,
    wave_state: Res<WaveState>,
    weapons_query: Query<(&Weapon, &WeaponState, &WeaponModesState), With<LocalPlayer>>,
    mut bar_widgets: Query<(&HudBarWidget, &mut Node, &mut BackgroundColor)>,
    mut text_widgets: Query<(&HudTextWidget, &mut Text)>,
) {
    // Gather game state
    let health_info = local_players
        .iter()
        .next()
        .map(|(health, _player)| (health.current, health.max));

    let enemy_count = enemies.iter().count();
    let player_count = all_players.iter().count();
    let wave_num = wave_state.current_wave;

    let weapon_info = weapons_query
        .iter()
        .next()
        .and_then(|(weapon, state, modes)| {
            let name = weapon.config.name.clone();
            let active_mode = modes.modes.get(&state.active_mode);
            Some((name, active_mode.cloned()))
        });

    // Update bar widgets
    for (bar_widget, mut node, mut bg_color) in bar_widgets.iter_mut() {
        match bar_widget.source.as_str() {
            "health" => {
                if let Some((current, max)) = health_info {
                    let ratio = if max > fixed_math::FIXED_ZERO {
                        (current.to_num::<f32>()) / (max.to_num::<f32>())
                    } else {
                        0.0
                    };
                    let ratio = ratio.clamp(0.0, 1.0);

                    // Update bar width based on ratio
                    node.width = Val::Px(bar_widget.full_width * ratio);

                    // Color gradient: red to green
                    let r = (1.0 - ratio).max(0.0);
                    let g = ratio.max(0.0);
                    *bg_color = BackgroundColor(Color::srgba(r, g, 0.0, 0.8));
                }
            }
            _ => {
                // Silently ignore unknown sources
            }
        }
    }

    // Update text widgets
    for (text_widget, mut text) in text_widgets.iter_mut() {
        let prefix_text = text.0.split('\n').next().unwrap_or("").to_string();
        let new_text = match text_widget.source.as_str() {
            "health" => {
                if let Some((current, max)) = health_info {
                    format!(
                        "{}{}/{}",
                        prefix_text,
                        current.to_num::<i32>(),
                        max.to_num::<i32>()
                    )
                } else {
                    prefix_text
                }
            }
            "wave" => format!("{}{}", prefix_text, wave_num),
            "ammo" => {
                if let Some((_name, Some(mode))) = &weapon_info {
                    format!("{}{} | {}", prefix_text, mode.mag_ammo, mode.mag_quantity)
                } else {
                    format!("{}? | ?", prefix_text)
                }
            }
            "weapon" => {
                if let Some((name, _mode)) = &weapon_info {
                    format!("{}{}", prefix_text, name)
                } else {
                    prefix_text
                }
            }
            "enemies" => format!("{}{}", prefix_text, enemy_count),
            "players" => format!("{}{}", prefix_text, player_count),
            _ => {
                // Silently ignore unknown sources
                prefix_text
            }
        };
        text.0 = new_text;
    }
}

// Hot-reload support: detect when HUD config is reloaded
fn handle_hud_config_changes(
    config_handle: Res<HudConfigHandle>,
    configs: Res<Assets<HudConfig>>,
    _q_hud_root: Query<Entity, With<HudRoot>>,
) {
    // Check if the config has changed by checking if the handle's state changed
    if let Some(handle) = &config_handle.0 {
        if let Some(_config) = configs.get(handle) {
            // If config is not in the "loading" state anymore, potentially re-render
            // For now, this is a simplified version - full hot-reload requires event listeners
            // which would be added later
        }
    }
}

fn despawn_hud(mut commands: Commands, q_hud_root: Query<Entity, With<HudRoot>>) {
    for entity in q_hud_root.iter() {
        commands.entity(entity).despawn();
    }
}

fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if let Ok(val) = u32::from_str_radix(hex, 16) {
        let r = ((val >> 16) & 0xFF) as f32 / 255.0;
        let g = ((val >> 8) & 0xFF) as f32 / 255.0;
        let b = (val & 0xFF) as f32 / 255.0;
        Color::srgb(r, g, b)
    } else {
        Color::WHITE
    }
}
