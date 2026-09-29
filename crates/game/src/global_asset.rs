use animation::{AnimationMapConfig, SpriteSheetConfig};
use bevy::{platform::collections::hash_map::HashMap, prelude::*};
use utils::bmap;

use crate::{
    character::config::CharacterConfig,
    core::{AppState, OnlineState},
    waves::WaveConfig,
    weapons::{melee::MeleeWeaponsConfig, WeaponsConfig},
};

const PLAYER_SPRITESHEET_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_sheet.ron";
const PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH: &str =
    "ZombieShooter/Sprites/Character/shirt_1_sheet.ron";
const PLAYER_HAIR_SPRITESHEET_CONFIG_PATH: &str =
    "ZombieShooter/Sprites/Character/hair_1_sheet.ron";
const PLAYER_ANIMATIONS_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_animation.ron";
const PLAYER_CONFIG_PATH: &str = "ZombieShooter/Sprites/Character/player_config.ron";

#[derive(Resource)]
pub struct GlobalAsset {
    pub spritesheets: HashMap<String, HashMap<String, Handle<SpriteSheetConfig>>>,
    pub animations: HashMap<String, Handle<AnimationMapConfig>>,
    pub character_configs: HashMap<String, Handle<CharacterConfig>>,
    pub weapons: Handle<WeaponsConfig>,
    pub melee_weapons: Handle<MeleeWeaponsConfig>,

    // Visual effects
    pub slash_effect_spritesheet: Handle<SpriteSheetConfig>,
    pub slash_effect_animation: Handle<AnimationMapConfig>,

    // Wave spawning config (optional - only loaded when wave mode is used)
    pub wave_config: Option<Handle<WaveConfig>>,
}

impl GlobalAsset {
    pub fn create(asset_server: &AssetServer) -> Self {
        Self {
            spritesheets: bmap!(
                "player" => bmap!(
                    "body" => asset_server.load(PLAYER_SPRITESHEET_CONFIG_PATH),
                    "shirt" => asset_server.load(PLAYER_SHIRT_SPRITESHEET_CONFIG_PATH),
                    "hair" => asset_server.load(PLAYER_HAIR_SPRITESHEET_CONFIG_PATH),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
                "shotgun" => bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Character/shotgun_sheet.ron")
                ),
                "pistol" => bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Character/pistol_sheet.ron")
                ),
                "machine_gun" => bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Character/machine_gun_sheet.ron")
                ),
                "zombie_1" => bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_sheet.ron"),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
                "zombie_2" => bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_hard_sheet.ron"),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                ),
                "zombie_full" => bmap!(
                    "body" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_full_sheet.ron"),
                    "shadow" => asset_server.load("ZombieShooter/Sprites/Character/shadow_sheet.ron")
                )
            ),
            animations: bmap!(
                "player" => asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
                "machine_gun" => asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
                "pistol" => asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
                "shotgun" => asset_server.load(PLAYER_ANIMATIONS_CONFIG_PATH),
                "zombie_1" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_animation.ron"),
                "zombie_2" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_animation.ron"),
                "zombie_full" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_full_animation.ron")
            ),
            character_configs: bmap!(
                "player" => asset_server.load(PLAYER_CONFIG_PATH),
                "zombie_1" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_config.ron"),
                "zombie_2" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_hard_config.ron"),
                "zombie_full" => asset_server.load("ZombieShooter/Sprites/Zombie/zombie_full_config.ron")
            ),
            weapons: asset_server.load("ZombieShooter/Sprites/Character/weapons.ron"),
            melee_weapons: asset_server.load("weapons/melee/melee_weapons.ron"),

            // Visual effects
            slash_effect_spritesheet: asset_server
                .load("ZombieShooter/Sprites/Character/slash_sheet.ron"),
            slash_effect_animation: asset_server
                .load("ZombieShooter/Sprites/Character/slash_animation.ron"),

            // Wave spawning config
            wave_config: Some(asset_server.load("waves/wave_config.ron")),
        }
    }
}

pub fn add_global_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    let global_asset = GlobalAsset::create(&asset_server);

    commands.insert_resource(global_asset);
}

pub fn loading_asset_system(
    mut app_state: ResMut<NextState<AppState>>,
    online: Res<OnlineState>,
    global_assets: Res<GlobalAsset>,
    asset_server: Res<AssetServer>,
) {
    for (_, v) in global_assets.spritesheets.iter() {
        for (_, handle) in v.iter() {
            if !asset_server.load_state(handle).is_loaded() {
                return;
            }
        }
    }

    for (_, handle) in global_assets.animations.iter() {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }

    for (_, handle) in global_assets.character_configs.iter() {
        if !asset_server.load_state(handle).is_loaded() {
            return;
        }
    }

    if !asset_server.load_state(&global_assets.weapons).is_loaded() {
        return;
    }
    if !asset_server
        .load_state(&global_assets.melee_weapons)
        .is_loaded()
    {
        return;
    }

    // Check visual effects
    if !asset_server
        .load_state(&global_assets.slash_effect_spritesheet)
        .is_loaded()
    {
        return;
    }
    if !asset_server
        .load_state(&global_assets.slash_effect_animation)
        .is_loaded()
    {
        return;
    }

    // Check wave config (if loaded)
    if let Some(wave_config) = &global_assets.wave_config {
        if !asset_server.load_state(wave_config).is_loaded() {
            return;
        }
    }

    if matches!(*online, OnlineState::Online) {
        app_state.set(AppState::LobbyOnline);
    } else {
        app_state.set(AppState::LobbyLocal);
    }
    info!("loading of asset is done , now entering lobby");
}
