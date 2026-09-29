use animation::AnimationStateBundle;
use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_kira_audio::prelude::*;
use utils::net_id::GgrsNetIdFactory;

use crate::{
    character::{config::CharacterConfigHandles, movement::Velocity},
    collider::{Collider, CollisionLayer},
    global_asset::GlobalAsset,
    weapons::melee::MeleeAttackState,
};

use bevy_ggrs::Rollback;

use super::{
    config::CharacterConfig,
    dash::DashState,
    health::{Health, HealthRegen},
    movement::SprintState,
    visuals::CharacterAppearance,
    Character,
};

pub fn create_character(
    commands: &mut Commands,
    global_assets: &Res<GlobalAsset>,
    character_asset: &Res<Assets<CharacterConfig>>,

    config_name: String,

    skin: Option<String>,
    color_health_bar: Color,
    translation: fixed_math::FixedVec3,

    collision_layer: CollisionLayer,
    id_factory: &mut ResMut<GgrsNetIdFactory>,
) -> Entity {
    let handle = global_assets.character_configs.get(&config_name).unwrap();
    let config = character_asset.get(handle).unwrap();

    let player_config_handle = global_assets
        .character_configs
        .get(&config.asset_name_ref)
        .unwrap()
        .clone();

    // BTreeMap (pas HashMap) : `AnimationStateBundle` place ces layers dans `ActiveLayers`,
    // un composant rollback dont l'ordre d'itération doit être stable entre clients.
    let starting_layers: std::collections::BTreeMap<String, String> = config
        .skins
        .get(skin.as_deref().unwrap_or(&config.starting_skin))
        .unwrap()
        .layers
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let transform_fixed = fixed_math::FixedTransform3D::new(
        translation,
        fixed_math::FixedMat3::IDENTITY,
        fixed_math::FixedVec3::splat(config.scale),
    );

    // Entité logique uniquement : les sprites et la barre de vie sont ajoutés par
    // la présentation à partir de CharacterAppearance (voir character::visuals).
    let entity = commands
        .spawn((
            transform_fixed.to_bevy_transform(),
            Visibility::default(),
            AnimationStateBundle::new(starting_layers),
            CharacterAppearance {
                config_name: config_name.clone(),
                skin: skin.clone(),
                health_bar_color: color_health_bar,
            },
        ))
        .id();

    // Apply character scale to collider dimensions
    let mut collider: Collider = (&config.collider).into();
    match &mut collider.shape {
        crate::collider::ColliderShape::Circle { radius } => {
            *radius = radius.saturating_mul(config.scale);
        }
        crate::collider::ColliderShape::Rectangle { width, height } => {
            *width = width.saturating_mul(config.scale);
            *height = height.saturating_mul(config.scale);
        }
    }
    // Also scale the collider offset
    collider.offset.x = collider.offset.x.saturating_mul(config.scale);
    collider.offset.y = collider.offset.y.saturating_mul(config.scale);
    collider.offset.z = collider.offset.z.saturating_mul(config.scale);
    
    let health: Health = config.base_health.clone().into();
    
    // Add gameplay components to the visual entity
    commands.entity(entity).insert((
        transform_fixed,
        SpatialAudioEmitter { instances: vec![] },
        Velocity { main: fixed_math::FixedVec2::ZERO, knockback: fixed_math::FixedVec2::ZERO },
        SprintState::default(),
        DashState::default(),
        MeleeAttackState::default(),
        collider,
        health,
        collision_layer,
        Character,
        CharacterConfigHandles {
            config: player_config_handle.clone(),
        },
        id_factory.next(config_name),
    ));

    // Add HealthRegen component if configured
    if let (Some(regen_rate), Some(regen_delay_frames)) = 
        (config.base_health.regen_rate, config.base_health.regen_delay_frames) 
    {
        commands.entity(entity).insert(HealthRegen {
            last_damage_frame: 0,
            regen_rate,
            regen_delay_frames,
        });
    }

    commands.entity(entity).insert(Rollback);

    entity
}
