use animation::SpriteSheetConfig;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use utils::net_id::GgrsNetIdFactory;
#[cfg(feature = "harmonium")]
use harmonium_bevy::components::HarmoniumTag;

use crate::{
    character::{config::CharacterConfig, create::create_character},
    collider::{CollisionLayer, CollisionSettings},
    global_asset::GlobalAsset,
    weapons::{melee::{spawn_melee_weapon_for_character, MeleeWeaponsConfig}, WeaponInventory, WeaponsConfig},
};

use super::{
    ai::{
        pathing::{EnemyPath, WallSlideTracker},
        state::{EnemyAiConfig, EnemyTarget, MonsterState},
    },
    Enemy,
};

/// Spawns an enemy entity and returns it.
///
/// Returns the spawned Entity so callers can attach additional components
/// (e.g., WaveEnemy for wave tracking).
pub fn spawn_enemy(
    enemy_type_name: String,
    position: fixed_math::FixedVec3,
    commands: &mut Commands,
    _weapons_asset: &Res<Assets<WeaponsConfig>>,
    melee_weapons_asset: &Res<Assets<MeleeWeaponsConfig>>,
    characters_asset: &Res<Assets<CharacterConfig>>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    spritesheet_assets: &Res<Assets<SpriteSheetConfig>>,

    global_assets: &Res<GlobalAsset>,
    collision_settings: &Res<CollisionSettings>,

    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Entity {
    let entity = create_character(
        commands,
        global_assets,
        characters_asset,
        asset_server,
        texture_atlas_layouts,
        spritesheet_assets,
        enemy_type_name,
        None,
        (LinearRgba::RED).into(),
        position,
        CollisionLayer(collision_settings.enemy_layer),
        id_factory,
    );

    let inventory = WeaponInventory::default();

    // Give the enemy a melee weapon (zombie claws, fallback to bare hands)
    if let Some(melee_weapons_config) = melee_weapons_asset.get(&global_assets.melee_weapons) {
        if let Some(weapon) = melee_weapons_config.0.get("zombie_claws").or_else(|| melee_weapons_config.0.get("bare_hands")) {
            spawn_melee_weapon_for_character(
                commands,
                entity,
                weapon.clone(),
                id_factory,
            );
        }
    }

    commands
        .entity(entity)
        .insert((
            inventory,
            EnemyPath::default(),
            WallSlideTracker::default(),
            Enemy::default(),
            // AI components for flow field navigation and combat
            EnemyAiConfig::zombie(),
            EnemyTarget::default(),
            MonsterState::default(),
        ));

    #[cfg(feature = "harmonium")]
    commands
        .entity(entity)
        .insert(HarmoniumTag::new(&["danger", "combat", "monster"], 1.0));

    entity
}
