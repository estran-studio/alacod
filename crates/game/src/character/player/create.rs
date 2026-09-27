use animation::SpriteSheetConfig;
use bevy::prelude::*;

#[cfg(feature = "lighting")]
use bevy_light_2d::light::PointLight2d;

use leafwing_input_manager::prelude::ActionState;
use utils::net_id::GgrsNetIdFactory;
#[cfg(feature = "harmonium")]
use harmonium_bevy::components::AiDriver;

use crate::{
    character::{config::CharacterConfig, create::create_character},
    collider::{CollisionLayer, CollisionSettings},
    global_asset::GlobalAsset,
    weapons::{melee::{spawn_melee_weapon_for_character, MeleeWeaponsConfig}, spawn_weapon_for_player, WeaponInventory, WeaponsConfig},
};

use super::{
    control::{get_input_map, PlayerAction},
    input::CursorPosition,
    LocalPlayer, Player,
};
use bevy_fixed::fixed_math::{self, FixedVec3};

const PLAYER_COLORS: &[LinearRgba] = &[
    LinearRgba::RED,
    LinearRgba::BLUE,
    LinearRgba::GREEN,
    LinearRgba::BLACK,
];

pub fn create_player(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,
    weapons_asset: &Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: &Res<Assets<MeleeWeaponsConfig>>,
    character_asset: &Res<Assets<CharacterConfig>>,
    collision_settings: &Res<CollisionSettings>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    sprint_sheet_assets: &Res<Assets<SpriteSheetConfig>>,

    position: FixedVec3,

    local: bool,
    handle: usize,
    name: String,
    pubkey: String,

    id_factory: &mut ResMut<GgrsNetIdFactory>,
) {
    let player_name = name;
    let player_pubkey = pubkey;

    // Use "player" as the character config name (defined in global_asset.rs)
    let config_name = "player".to_string();

    let entity = create_character(
        commands,
        global_assets,
        character_asset,
        asset_server,
        texture_atlas_layouts,
        sprint_sheet_assets,
        config_name,
        Some(if handle == 0 { "1" } else { "2" }.into()),
        (LinearRgba::GREEN).into(),
        position,
        CollisionLayer(collision_settings.player_layer),
        id_factory,
    );
    if local {
        commands.entity(entity).insert((
            LocalPlayer {},
            ActionState::<PlayerAction>::default(),
            get_input_map(),
        ));

        #[cfg(feature = "harmonium")]
        commands.entity(entity).insert(AiDriver {
            ai_influence: 1.0,
            detection_radius: 300.0,
            ..default()
        });
    }

    let mut inventory = WeaponInventory::default();

    if let Some(weapons_config) = weapons_asset.get(&global_assets.weapons) {
        let mut keys: Vec<&String> = weapons_config.0.keys().collect();
        keys.sort();
        for (i, k) in keys.iter().enumerate() {
            spawn_weapon_for_player(
                commands,
                global_assets,
                asset_server,
                texture_atlas_layouts,
                sprint_sheet_assets,
                i == 0,
                entity,
                weapons_config.0.get(*k).unwrap().clone(),
                &mut inventory,
                id_factory,
            );
        }
    }

    // Add a default melee weapon (bare hands) to all players
    if let Some(melee_weapons_config) = melee_weapons_asset.get(&global_assets.melee_weapons) {
        if let Some(bare_hands) = melee_weapons_config.0.get("bare_hands") {
            spawn_melee_weapon_for_character(
                commands,
                entity,
                bare_hands.clone(),
                id_factory,
            );
        }
    }

    #[cfg(feature = "lighting")]
    {
        commands.entity(entity).insert((
            inventory,
            CursorPosition::default(),
            super::input::InteractionInput::default(),
            crate::interaction::Interactor,
            Player {
                handle,
                color: PLAYER_COLORS[handle].into(),
                name: player_name.clone(),
                pubkey: player_pubkey.clone(),
            },
            PointLight2d {
                radius: 200.,
                cast_shadows: false,
                falloff: 4.,
                ..default()
            },
        ));
    }

    #[cfg(not(feature = "lighting"))]
    {
        commands.entity(entity).insert((
            inventory,
            CursorPosition::default(),
            super::input::InteractionInput::default(),
            crate::interaction::Interactor,
            Player {
                handle,
                color: PLAYER_COLORS[handle].into(),
                name: player_name,
                pubkey: player_pubkey,
            },
        ));
    }
}
