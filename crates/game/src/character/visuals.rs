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

/// Hash manuel : exclut `health_bar_color` (présentation, `Color` est `f32` et
/// n'implémente pas `Hash`) ; seuls `config_name`/`skin` (le choix de skin, qui affecte
/// bien la simulation via le spawn des layers) contribuent au checksum.
impl std::hash::Hash for CharacterAppearance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.config_name.hash(state);
        self.skin.hash(state);
    }
}

/// Teinte de base des calques (D66, `CharacterConfig::marker`) : relue par
/// `feedback::apply_layer_colors` à la place du blanc. Présentation, hors rollback.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct MarkerTint(pub Color);

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
            let layer_entity = create_child_sprite(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                entity,
                spritesheet_config,
                0,
            );
            // D66 : échelle du marqueur (ombre comprise, le personnage grossit d'un bloc).
            if let Some(scale) = config.marker.as_ref().and_then(|m| m.scale) {
                commands.entity(layer_entity).insert(Transform {
                    translation: Vec3::new(
                        spritesheet_config.offset_x,
                        spritesheet_config.offset_y,
                        spritesheet_config.offset_z,
                    ),
                    scale: Vec3::splat(spritesheet_config.scale * scale),
                    ..default()
                });
            }
        }
        if let Some((r, g, b)) = config.marker.as_ref().and_then(|m| m.tint) {
            commands
                .entity(entity)
                .insert(MarkerTint(Color::srgb(r, g, b)));
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

/// D66 (présentation) : un ennemi **stationnaire** regarde sa cible. `FacingDirection` (rollback)
/// ne suit que la vitesse, nulle pour une tourelle : le sprite gardait son orientation de
/// création. Direction : visée figée de l'émetteur pendant un pattern, sinon cible de
/// déplacement (`EnemyPath`) ; ne retourne que le `flip_x` des calques (comme `set_sprite_flip`,
/// dont ce système passe après). Ne touche ni l'état ni les traces.
pub fn face_stationary_enemies(
    enemies: Query<
        (
            &bevy_fixed::fixed_math::FixedTransform3D,
            &crate::character::enemy::ai::state::EnemyAiConfig,
            Option<&combat::emitter::Emitter>,
            Option<&crate::character::enemy::ai::pathing::EnemyPath>,
            &Children,
        ),
        With<combat::actors::Enemy>,
    >,
    mut sprites: Query<&mut Sprite>,
) {
    for (transform, ai, emitter, path, children) in &enemies {
        if !ai.stationary {
            continue;
        }
        let dx = match (emitter, path) {
            (Some(emitter), _) if emitter.aim != bevy_fixed::fixed_math::FixedVec2::ZERO => {
                emitter.aim.x.to_num::<f32>()
            }
            (_, Some(path)) => (path.target_position.x - transform.translation.x).to_num::<f32>(),
            _ => continue,
        };
        if dx.abs() < 1e-3 {
            continue;
        }
        for child in children.iter() {
            if let Ok(mut sprite) = sprites.get_mut(child) {
                sprite.flip_x = dx < 0.0;
            }
        }
    }
}
