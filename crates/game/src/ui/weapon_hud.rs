use bevy::prelude::*;
use bevy_fixed::math::calculate_time_remaining_seconds;
use combat::inventory::AmmoReserves;
use utils::frame::FrameCount;

use crate::{character::player::LocalPlayer, core::AppState};

use combat::weapons::{WeaponInventory, WeaponModesState, WeaponState};

#[derive(Component)]
struct CurrentWeaponText;

#[derive(Component)]
struct AmmoText;

#[derive(Component)]
struct ReloadingText;

fn setup_weapon_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        ReloadingText,
        Text::new(""),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(16.0),
            ..Default::default()
        },
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(37.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));

    commands.spawn((
        CurrentWeaponText,
        Text::new("Weapon: "),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(16.0),
            ..Default::default()
        },
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));

    commands.spawn((
        AmmoText,
        Text::new("Ammo: "),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(16.0),
            ..Default::default()
        },
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(3.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));
}

fn update_weapons_text(
    frame: Res<FrameCount>,
    q_player: Query<(&WeaponInventory, &AmmoReserves), With<LocalPlayer>>,
    weapon_query: Query<(&WeaponState, &WeaponModesState)>,
    mut q_weapon: Query<&mut Text, (With<CurrentWeaponText>, Without<AmmoText>)>,
    mut q_ammo: Query<&mut Text, (With<AmmoText>, Without<CurrentWeaponText>)>,
    mut q_reloading: Query<
        &mut Text,
        (
            With<ReloadingText>,
            Without<CurrentWeaponText>,
            Without<AmmoText>,
        ),
    >,
) {
    if let Ok((inventory, ammo_reserves)) = q_player.single() {
        // T2.2 (lâcher/ramasser) : l'inventaire peut être vide (toutes les armes au sol).
        if inventory.weapons.is_empty() {
            return;
        }
        let active_weapon = inventory.active_weapon();
        if let Ok((state, modes_state)) = weapon_query.get(active_weapon.0) {
            let active_weapon_state = modes_state.modes.get(&state.active_mode).unwrap();
            if let Ok(mut text) = q_weapon.single_mut() {
                text.0 = format!(
                    "Weapon: {} - {}",
                    active_weapon.1.config.name, state.active_mode
                );
            }
            if let Ok(mut text) = q_ammo.single_mut() {
                // Munitions du chargeur / réserve du type de l'arme active (T2.2 : la
                // réserve remplace l'ancien `mag_quantity` par arme, voir
                // `combat::inventory::AmmoReserves`).
                text.0 = format!(
                    "Ammo: {} / {}",
                    active_weapon_state.mag_ammo,
                    ammo_reserves.get(&active_weapon.1.config.ammo_type)
                )
            }

            if let Ok(mut text) = q_reloading.single_mut() {
                text.0 = if inventory.is_reloading() {
                    format!(
                        "{:.2}s",
                        calculate_time_remaining_seconds(
                            inventory.reloading_ending_frame.unwrap(),
                            frame.frame
                        )
                    )
                } else {
                    String::new()
                };
            }
        }
    }
}

#[derive(Default)]
pub struct WeaponDebugUIPlugin;

impl Plugin for WeaponDebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_weapon_ui);
        app.add_systems(
            Update,
            update_weapons_text.run_if(in_state(AppState::InGame)),
        );
    }
}
