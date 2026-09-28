//! Présentation des personnages : sprites animés et barre de vie.
//!
//! La simulation ne crée que l'entité logique avec un [`CharacterAppearance`].
//! [`attach_character_visuals`] y ajoute les sprites dès qu'il voit une entité sans
//! visuels, y compris une entité recréée par un rollback.

use animation::{create_child_sprite, AnimationVisualsBundle, SpriteSheetConfig};
use bevy::prelude::*;

use crate::global_asset::GlobalAsset;

use super::{config::CharacterConfig, health::ui::HealthBar};

/// Apparence d'un personnage : donnée par la simulation, dessinée par la présentation.
#[derive(Component, Clone, Debug)]
pub struct CharacterAppearance {
    /// Clé de `GlobalAsset::character_configs`.
    pub config_name: String,
    /// Skin de la config ; `None` pour le skin de départ.
    pub skin: Option<String>,
    pub health_bar_color: Color,
}

/// Marque une entité dont les visuels ont été créés (hors rollback).
#[derive(Component)]
pub struct VisualsAttached;

pub fn attach_character_visuals(
    mut commands: Commands,
    global_assets: Res<GlobalAsset>,
    character_asset: Res<Assets<CharacterConfig>>,
    spritesheet_assets: Res<Assets<SpriteSheetConfig>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    characters: Query<(Entity, &CharacterAppearance), Without<VisualsAttached>>,
) {
    for (entity, appearance) in characters.iter() {
        let Some(config) = global_assets
            .character_configs
            .get(&appearance.config_name)
            .and_then(|handle| character_asset.get(handle))
        else {
            continue;
        };
        let (Some(map_layers), Some(animation_handle)) = (
            global_assets.spritesheets.get(&config.asset_name_ref),
            global_assets.animations.get(&config.asset_name_ref),
        ) else {
            continue;
        };
        let Some(skin) = config
            .skins
            .get(appearance.skin.as_deref().unwrap_or(&config.starting_skin))
        else {
            continue;
        };

        commands.entity(entity).insert((
            AnimationVisualsBundle::new(map_layers.clone(), animation_handle.clone(), 0),
            VisualsAttached,
        ));

        for layer in skin.layers.keys() {
            let Some(spritesheet_config) = map_layers
                .get(layer)
                .and_then(|handle| spritesheet_assets.get(handle))
            else {
                continue;
            };
            create_child_sprite(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                entity,
                spritesheet_config,
                0,
            );
        }

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                HealthBar,
                Sprite {
                    color: appearance.health_bar_color,
                    custom_size: Some(Vec2::new(30.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 10.0, 0.1)),
            ));
        });
    }
}
