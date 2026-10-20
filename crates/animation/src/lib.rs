use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*, reflect::TypePath, sprite::Anchor};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;
use sim_core::animation_state::{AnimationState, FacingDirection};

// CONFIG

// 1a. Define your custom enum that CAN be deserialized
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")] // Allows "BottomCenter" in RON file
pub enum ConfigurableAnchor {
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
    // Add Custom(Vec2) if you need it, requires slightly more complex mapping
}

impl ConfigurableAnchor {
    pub fn to_anchor(&self) -> Anchor {
        match self {
            ConfigurableAnchor::Center => Anchor::CENTER,
            ConfigurableAnchor::BottomLeft => Anchor::BOTTOM_LEFT,
            ConfigurableAnchor::BottomCenter => Anchor::BOTTOM_CENTER,
            ConfigurableAnchor::BottomRight => Anchor::BOTTOM_RIGHT,
            ConfigurableAnchor::CenterLeft => Anchor::CENTER_LEFT,
            ConfigurableAnchor::CenterRight => Anchor::CENTER_RIGHT,
            ConfigurableAnchor::TopLeft => Anchor::TOP_LEFT,
            ConfigurableAnchor::TopCenter => Anchor::TOP_CENTER,
            ConfigurableAnchor::TopRight => Anchor::TOP_RIGHT,
            // Add Custom case here if you defined it
        }
    }
}

// -- Sprite Sheet Layout Configuration --
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct SpriteSheetConfig {
    pub path: String,
    pub tile_size: (u32, u32),
    pub columns: u32,
    pub rows: u32,
    pub name: String,
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub animated: bool,
    pub anchor: ConfigurableAnchor,
    /// L'image porte sa couleur (projectiles de `m2-lisibilite-gungeon`) : le rendu ne la teinte pas par
    /// équipe et sa taille affichée suit le dégât. Faux par défaut : comportement d'avant.
    #[serde(default)]
    pub own_colors: bool,
}

// -- Animation Definition Configuration --
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AnimationIndices {
    // Direct sprite indices (backwards compatible)
    Direct {
        start: usize,
        end: usize, // Inclusive end index
    },
    // Row-based specification
    Row {
        row: usize,
        end: usize, // Number of frames in the row (offset from row start)
    },
}

impl AnimationIndices {
    /// Converts the animation indices to absolute start/end sprite indices
    /// based on the sprite sheet configuration
    pub fn to_absolute(&self, columns: u32) -> (usize, usize) {
        match self {
            AnimationIndices::Direct { start, end } => (*start, *end),
            AnimationIndices::Row { row, end } => {
                let row_start = row * columns as usize;
                let row_end = row_start + end;
                (row_start, row_end)
            }
        }
    }
}

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct AnimationMapConfig {
    pub frame_duration: u64,
    pub animations: HashMap<String, AnimationIndices>,
    // Optional: Store columns count if needed for row-based animations
    // This can be provided or inferred from the associated sprite sheet
    #[serde(default)]
    pub columns: Option<u32>,
}

// COMPONENT
#[derive(Component, Default, Clone, Debug)]
pub struct LayerName {
    pub name: String,
}

#[derive(Component)]
pub struct AnimatedLayer {}

#[derive(Component)]
pub struct ColoredLayer {}

// Handles are loaded once, assume they don't change and don't need rollback/reflection
#[derive(Component)]
pub struct CharacterAnimationHandles {
    pub spritesheets: HashMap<String, Handle<SpriteSheetConfig>>,
    pub animations: Handle<AnimationMapConfig>,
    pub starting_index: usize,
}

#[derive(Component)]
struct AnimationTimer {
    frame_timer: Timer,
}

/// Partie visuelle de l'animation (handles et timer), ajoutée par la présentation
/// à côté d'un [`sim_core::animation_state::AnimationStateBundle`].
#[derive(Bundle)]
pub struct AnimationVisualsBundle {
    handles: CharacterAnimationHandles,
    timer: AnimationTimer,
}

impl AnimationVisualsBundle {
    pub fn new(
        spritesheets: HashMap<String, Handle<SpriteSheetConfig>>,
        animations: Handle<AnimationMapConfig>,
        starting_index: usize,
    ) -> Self {
        Self {
            handles: CharacterAnimationHandles {
                spritesheets,
                animations,
                starting_index,
            },
            timer: AnimationTimer {
                frame_timer: Timer::from_seconds(1., TimerMode::Repeating),
            },
        }
    }
}

/// Gel de présentation des animations (hit stop, T1.17, `docs/conventions.md` §31) : tant
/// qu'il est actif, `animate_sprite_system` n'avance plus les timers ni les images. Compté en
/// **ticks de rendu** (un par `Update`), jamais en frames de simulation : la simulation
/// (rollback, p2p) continue. Posé par `game::feedback`, lu aussi par le suivi de caméra.
#[derive(Resource, Debug, Default)]
pub struct AnimationFreeze {
    tick: u64,
    until_tick: u64,
}

impl AnimationFreeze {
    /// Gèle les `frames` prochains ticks de rendu (prolonge un gel en cours, ne le raccourcit
    /// jamais).
    pub fn freeze_for(&mut self, frames: u32) {
        self.until_tick = self.until_tick.max(self.tick + 1 + u64::from(frames));
    }

    pub fn is_frozen(&self) -> bool {
        self.tick < self.until_tick
    }
}

fn advance_animation_freeze(mut freeze: ResMut<AnimationFreeze>) {
    freeze.tick += 1;
}

// Animates sprite based on AnimationState
fn animate_sprite_system(
    time: Res<Time>,
    freeze: Res<AnimationFreeze>,
    animation_configs: Res<Assets<AnimationMapConfig>>,
    spritesheet_configs: Res<Assets<SpriteSheetConfig>>,
    mut query: Query<(
        &Children,
        &CharacterAnimationHandles,
        &mut AnimationTimer,
        &AnimationState,
    )>,
    mut query_sprites: Query<(&mut Sprite, &LayerName), With<AnimatedLayer>>,
) {
    if freeze.is_frozen() {
        return;
    }
    for (childs, config_handles, mut timer, state) in query.iter_mut() {
        if let Some(anim_config) = animation_configs.get(&config_handles.animations) {
            // Try to get columns count from animation config or first spritesheet
            let columns = anim_config
                .columns
                .or_else(|| {
                    config_handles
                        .spritesheets
                        .values()
                        .next()
                        .and_then(|handle| {
                            spritesheet_configs.get(handle).map(|config| config.columns)
                        })
                })
                .unwrap_or(1); // Default to 1 if we can't determine

            timer.frame_timer.tick(time.delta());
            if timer.frame_timer.just_finished() {
                for child in childs.iter() {
                    if let Ok((mut sprite, _)) = query_sprites.get_mut(child.clone()) {
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            if let Some(indices) = anim_config.animations.get(&state.0) {
                                let (start_index, end_index) = indices.to_absolute(columns);
                                if atlas.index < start_index || atlas.index > end_index {
                                    atlas.index = start_index;
                                } else {
                                    atlas.index = (atlas.index + 1 - start_index)
                                        % (end_index - start_index + 1)
                                        + start_index;
                                }
                            } else {
                                atlas.index = anim_config.animations.get("Idle").map_or(0, |idx| {
                                    let (start, _) = idx.to_absolute(columns);
                                    start
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

// Updates animation timer duration if AnimationMapConfig reloads
fn check_animation_config_reload_system(
    mut ev_asset: MessageReader<AssetEvent<AnimationMapConfig>>,
    animation_configs: Res<Assets<AnimationMapConfig>>,
    mut query: Query<(&CharacterAnimationHandles, &mut AnimationTimer)>,
    asset_server: Res<AssetServer>,
) {
    let mut updates_needed = HashMap::new(); // Handle ID -> new duration

    // Collect updates needed from asset events
    for event in ev_asset.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(config) = animation_configs.get(*id) {
                    updates_needed.insert(*id, config.frame_duration);
                }
            }
            _ => {}
        }
    }

    // Apply updates to relevant entities
    for (config_handles, mut anim_timer) in query.iter_mut() {
        if let Some(new_duration) = updates_needed.get(&config_handles.animations.id()) {
            anim_timer
                .frame_timer
                .set_duration(Duration::from_millis(*new_duration));
            anim_timer.frame_timer.reset();
        }
        // Apply initial duration after startup load (if needed)
        else if anim_timer.frame_timer.duration().as_secs_f32() == 1.0 {
            // Check default
            if asset_server
                .load_state(&config_handles.animations)
                .is_loaded()
            {
                if let Some(config) = animation_configs.get(&config_handles.animations) {
                    anim_timer
                        .frame_timer
                        .set_duration(Duration::from_millis(config.frame_duration));
                    anim_timer.frame_timer.reset();
                }
            }
        }
    }
}

fn character_visuals_update_system(
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    spritesheet_configs: Res<Assets<SpriteSheetConfig>>,
    mut ev_asset: MessageReader<AssetEvent<SpriteSheetConfig>>,
    query: Query<(&Children, Entity, &CharacterAnimationHandles)>,
    mut query_sprite: Query<(&mut Sprite, &mut Transform, &mut Anchor, &LayerName)>,
) {
    for event in ev_asset.read() {
        if let AssetEvent::Modified { id } | AssetEvent::Added { id } = event {
            // Find entities using the modified spritesheet config
            for (childs, _entity, config_handle) in query.iter() {
                for handle in config_handle.spritesheets.values() {
                    if handle.id() == *id {
                        if let Some(new_config) = spritesheet_configs.get(handle) {
                            info!("Spritesheet config modified {}", new_config.name,);
                            let new_layout = TextureAtlasLayout::from_grid(
                                UVec2::new(new_config.tile_size.0, new_config.tile_size.1),
                                new_config.columns,
                                new_config.rows,
                                None,
                                None,
                            );

                            for child in childs.iter() {
                                if let Ok((mut sprite, mut transform, mut anchor, layer_name)) =
                                    query_sprite.get_mut(child.clone())
                                {
                                    if layer_name.name == new_config.name {
                                        sprite.texture_atlas = Some(TextureAtlas {
                                            layout: texture_atlas_layouts.add(new_layout.clone()),
                                            index: config_handle.starting_index,
                                        });
                                        transform.translation.x = new_config.offset_x;
                                        transform.translation.z = new_config.offset_z;
                                        transform.translation.y = new_config.offset_y;
                                        transform.scale = Vec3::splat(new_config.scale);
                                        sprite.image = asset_server.load(&new_config.path);
                                        *anchor = new_config.anchor.to_anchor();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// SYSTEM THAT RUN ON THE BEVY SCHEDULE FOR SYNCH

pub fn set_sprite_flip(
    query: Query<(&Children, &FacingDirection)>,
    mut sprite_query: Query<&mut Sprite>,
) {
    for (childrens, direction) in query.iter() {
        for child in childrens.iter() {
            if let Ok(mut sprite) = sprite_query.get_mut(child.clone()) {
                // Flip sprite horizontally for left-facing directions
                sprite.flip_x = direction.should_flip_x();
            }
        }
    }
}

pub fn create_child_sprite(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,

    parent_entity: Entity,
    spritesheet_config: &SpriteSheetConfig,
    current_frame_index: usize,
) -> Entity {
    let texture_handle: Handle<Image> = asset_server.load(&spritesheet_config.path);
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(
            spritesheet_config.tile_size.0,
            spritesheet_config.tile_size.1,
        ),
        spritesheet_config.columns,
        spritesheet_config.rows,
        None,
        None,
    );
    let layout_handle = texture_atlas_layouts.add(layout);

    let mut entity_commands = commands.spawn((
        Sprite {
            image: texture_handle.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: layout_handle.clone(),
                index: current_frame_index,
            }),
            ..default()
        },
        spritesheet_config.anchor.to_anchor(),
        Transform::from_scale(Vec3::splat(spritesheet_config.scale)).with_translation(Vec3::new(
            spritesheet_config.offset_x,
            spritesheet_config.offset_y,
            spritesheet_config.offset_z,
        )),
        //.with_rotation(Quat::IDENTITY),
        LayerName {
            name: spritesheet_config.name.clone(),
        },
    ));

    if spritesheet_config.animated {
        entity_commands.insert(AnimatedLayer {});
    }

    let sprite = entity_commands.id();

    // Le parent a pu mourir et être détruit avant l'application différée des
    // commandes (même course que les `try_insert` des visuels) : `add_child`
    // paniquait (« Entity despawned »). Si le parent est parti, l'enfant
    // orphelin est détruit plutôt que laissé seul à l'origine.
    commands.queue_silenced(move |world: &mut World| {
        if let Ok(mut parent) = world.get_entity_mut(parent_entity) {
            parent.add_child(sprite);
        } else if let Ok(mut orphan) = world.get_entity_mut(sprite) {
            orphan.despawn();
        }
    });

    sprite
}

// PLUGIN

pub struct D2AnimationPlugin;

impl Plugin for D2AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<SpriteSheetConfig>::new(&["ron"]));
        app.add_plugins(RonAssetPlugin::<AnimationMapConfig>::new(&["ron"]));

        // `AnimationState`, `FacingDirection` et `ActiveLayers` (rollback) sont enregistrés par
        // `sim_core::animation_state::AnimationStatePlugin`, ajouté avant ce plugin (`game::core`).

        app.init_resource::<AnimationFreeze>()
            .add_systems(First, advance_animation_freeze);

        app.add_systems(
            Update,
            (
                character_visuals_update_system,
                animate_sprite_system.after(character_visuals_update_system),
                check_animation_config_reload_system.after(animate_sprite_system),
            ),
        );
    }
}

#[cfg(test)]
mod freeze_tests {
    use super::AnimationFreeze;

    #[test]
    fn gele_les_n_ticks_suivants_puis_reprend() {
        let mut freeze = AnimationFreeze::default();
        assert!(!freeze.is_frozen());
        freeze.freeze_for(2);
        // Le tick où le gel est posé n'est pas gelé (les animations ont déjà avancé).
        let frozen: Vec<bool> = (0..4)
            .map(|_| {
                freeze.tick += 1;
                freeze.is_frozen()
            })
            .collect();
        assert_eq!(frozen, [true, true, false, false]);
    }

    #[test]
    fn un_gel_plus_court_ne_raccourcit_pas() {
        let mut freeze = AnimationFreeze::default();
        freeze.freeze_for(5);
        freeze.tick += 1;
        freeze.freeze_for(1);
        assert_eq!(freeze.until_tick, 6);
    }
}
