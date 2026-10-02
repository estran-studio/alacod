//! Présentation des armes et de la mêlée, dérivée de l'état de combat.
use crate::{character::visuals::VisualsAttached, global_asset::GlobalAsset};
use animation::{create_child_sprite, AnimationVisualsBundle, FacingDirection, SpriteSheetConfig};
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_fixed::fixed_math;
use combat::weapons::melee::*;
use combat::weapons::*;
use utils::{frame::FrameCount, net_id::GgrsNetId};

/// Présentation : ajoute le sprite animé des armes qui n'en ont pas encore
/// (nouvelles ou recréées par un rollback).
pub fn attach_weapon_visuals(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    spritesheet_assets: Res<Assets<SpriteSheetConfig>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    weapons: Query<(Entity, &Weapon), Without<VisualsAttached>>,
) {
    for (entity, weapon) in weapons.iter() {
        let name = &weapon.sprite_config.name;
        let (Some(map_layers), Some(animation_handle)) = (
            global_assets.spritesheets.get(name),
            global_assets.animations.get(name),
        ) else {
            continue;
        };
        let Some(spritesheet_config) = map_layers
            .get("body")
            .and_then(|handle| spritesheet_assets.get(handle))
        else {
            continue;
        };

        commands.entity(entity).insert((
            AnimationVisualsBundle::new(
                map_layers.clone(),
                animation_handle.clone(),
                weapon.sprite_config.index,
            ),
            VisualsAttached,
        ));
        create_child_sprite(
            &mut commands,
            &asset_server,
            &mut texture_atlas_layouts,
            entity,
            spritesheet_config,
            0,
        );
    }
}

pub fn update_weapon_sprite_direction(
    mut query_sprite: Query<&mut Sprite>,
    query_players: Query<(&Children, &FacingDirection)>,
    query_weapons: Query<&Children, With<ActiveWeapon>>,
) {
    for (childs, direction) in query_players.iter() {
        for child in childs.iter() {
            if let Ok(childs) = query_weapons.get(child.clone()) {
                for child in childs.iter() {
                    if let Ok(mut sprite) = query_sprite.get_mut(child.clone()) {
                        // Flip sprite based on facing direction
                        sprite.flip_y = direction.should_flip_x();
                    }
                }
            }
        }
    }
}

/// Sprites des armes, effets de slash et UI de debug. Ajouté par `PresentationPlugin`.
pub struct WeaponPresentationPlugin;

impl Plugin for WeaponPresentationPlugin {
    fn build(&self, app: &mut App) {
        // Only include the debug UI plugin when the `debug_ui` feature is enabled.
        // This keeps Egui / WorldInspector out of production builds unless explicitly requested.
        #[cfg(feature = "debug_ui")]
        app.add_plugins(crate::ui::weapon_hud::WeaponDebugUIPlugin);

        app.add_systems(
            Update,
            (
                attach_weapon_visuals,
                update_weapon_sprite_direction,
                spawn_slash_effects,
                update_slash_effects,
            ),
        );
    }
}
// SLASH VISUAL EFFECT
#[derive(Component)]
pub struct SlashEffect {
    pub start_frame: u32,
    pub duration_frames: u32,
    pub frame_duration: u32, // Frames per animation frame
    pub animation_start: usize,
    pub animation_end: usize, // Inclusive
}

// SYSTEM: UPDATE SLASH VISUAL EFFECTS
/// Présentation : crée l'effet de slash d'une hitbox de mêlée. Une hitbox recréée
/// ou resimulée par un rollback (même GgrsNetId) ne produit pas de second effet.
pub fn spawn_slash_effects(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    spritesheet_assets: Res<Assets<SpriteSheetConfig>>,
    animation_configs: Res<Assets<animation::AnimationMapConfig>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    frame: Res<FrameCount>,
    hitboxes: Query<(&GgrsNetId, &MeleeHitbox, &fixed_math::FixedTransform3D)>,
    mut shown: Local<HashMap<usize, u32>>,
) {
    // Oublier les hitbox trop anciennes pour être encore rejouées par un rollback
    shown.retain(|_, created_frame| frame.frame.saturating_sub(*created_frame) < 600);

    let (Some(slash_config), Some(anim_config)) = (
        spritesheet_assets.get(&global_assets.slash_effect_spritesheet),
        animation_configs.get(&global_assets.slash_effect_animation),
    ) else {
        return;
    };

    for (net_id, hitbox, hitbox_transform) in hitboxes.iter() {
        if shown.insert(net_id.0, hitbox.created_frame).is_some() {
            continue;
        }

        let texture_handle: Handle<Image> = asset_server.load(&slash_config.path);
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(slash_config.tile_size.0, slash_config.tile_size.1),
            slash_config.columns,
            slash_config.rows,
            None,
            None,
        );
        let layout_handle = texture_atlas_layouts.add(layout);

        // Get animation configuration dynamically
        let slash_anim = anim_config
            .animations
            .get("slash")
            .expect("slash animation not found in config");
        let columns = slash_config.columns;
        let (start, end) = slash_anim.to_absolute(columns);
        let frame_duration = anim_config.frame_duration as u32;
        let animation_frame_count = (end - start + 1) as u32;
        let total_duration = animation_frame_count * frame_duration;

        // Position effect at hitbox location
        let mut effect_transform = hitbox_transform.to_bevy_transform();
        effect_transform.translation.z = 5.0; // Place above everything

        // For 8-directional slashes, we need to handle flipping carefully
        // The sprite is designed for right-facing attacks (0 degrees)
        // For left-facing, we flip and use the opposite angle
        let (flip_x, rotation_angle) = match hitbox.facing {
            FacingDirection::Right => (false, 0.0),
            FacingDirection::UpRight => (false, std::f32::consts::PI / 4.0),
            FacingDirection::Up => (false, std::f32::consts::PI / 2.0),
            FacingDirection::UpLeft => (true, -std::f32::consts::PI / 4.0), // Flip + negative angle for upper left
            FacingDirection::Left => (true, 0.0),                           // Flip + 0° for left
            FacingDirection::DownLeft => (true, std::f32::consts::PI / 4.0), // Flip + positive angle for lower left
            FacingDirection::Down => (false, -std::f32::consts::PI / 2.0),
            FacingDirection::DownRight => (false, -std::f32::consts::PI / 4.0),
        };

        effect_transform.rotation = Quat::from_rotation_z(rotation_angle);

        commands.spawn((
            SlashEffect {
                start_frame: hitbox.created_frame,
                duration_frames: total_duration,
                frame_duration,
                animation_start: start,
                animation_end: end,
            },
            Sprite {
                image: texture_handle,
                texture_atlas: Some(TextureAtlas {
                    layout: layout_handle,
                    index: start, // Start at the correct animation frame
                }),
                flip_x,
                flip_y: false,
                ..default()
            },
            effect_transform,
        ));
    }
}

pub fn update_slash_effects(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut slash_query: Query<(Entity, &SlashEffect, &mut Sprite)>,
) {
    for (entity, slash_effect, mut sprite) in slash_query.iter_mut() {
        let frames_alive = frame.frame - slash_effect.start_frame;

        // Calculate current animation frame dynamically based on SlashEffect config
        let animation_progress = frames_alive / slash_effect.frame_duration;
        let animation_frame_count =
            (slash_effect.animation_end - slash_effect.animation_start + 1) as u32;
        let current_animation_frame = animation_progress.min(animation_frame_count - 1) as usize;
        let sprite_index = slash_effect.animation_start + current_animation_frame;

        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = sprite_index;
        }

        // Despawn when animation is complete
        if frames_alive >= slash_effect.duration_frames {
            commands.entity(entity).despawn();
        }
    }
}
