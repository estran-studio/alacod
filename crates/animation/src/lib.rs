use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*, reflect::TypePath, sprite::Anchor};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;
use sim_core::animation_state::{AnimationState, FacingDirection};
pub use sim_core::sprite_directions::{Compass, RowChoice, SheetDirections};

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
    /// Directions dessinées par la planche (X-4a, `docs/conventions.md` §89) : la direction de chaque
    /// ligne d'un bloc, et si le côté non dessiné est la ligne retournée. Absent : une orientation
    /// tournée vers la droite, retournée pour la gauche (comportement d'avant).
    #[serde(default)]
    pub directions: Option<SheetDirections>,
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

impl AnimationMapConfig {
    /// Clé de l'animation jouée : la clé de présentation `presented` si la carte la définit (sinon elle est
    /// ignorée : « Attack » sans clé « Attack » ne change rien), l'état sinon.
    pub fn playing_key<'a>(&self, state: &'a str, presented: Option<&'a str>) -> &'a str {
        presented
            .filter(|key| self.animations.contains_key(*key))
            .unwrap_or(state)
    }
}

/// Clé d'animation de **présentation** (S1-8, `Update`, jamais rollback ni lue par la simulation) : posée et
/// définie par la carte d'animation, elle remplace `AnimationState` pour choisir l'animation jouée (ex. `"Attack"`
/// d'un zombie qui frappe). Posée par `game` à partir de l'état rollback existant ; sans clé correspondante dans
/// la carte, elle n'a aucun effet (les feuilles publiques de zombies n'ont pas d'`Attack`). Même type que celui
/// de X-4b (branche `wquintal/x-4b-import-sprite`, `"Shoot"`), pour un merge sans second mécanisme.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub struct PresentationClip(pub Option<String>);

// COMPONENT
#[derive(Component, Default, Clone, Debug)]
pub struct LayerName {
    pub name: String,
}

#[derive(Component)]
pub struct AnimatedLayer {}

#[derive(Component)]
pub struct ColoredLayer {}

/// Ligne de direction actuellement appliquée à un calque dont la planche déclare `directions`
/// (présentation seule, posée par [`set_sprite_flip`]). `atlas.index` = image de l'animation
/// + `row × columns` ; les systèmes qui lisent l'image d'animation retirent d'abord ce décalage.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DirectionRow {
    pub row: usize,
}

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
    presented: PresentationClip,
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
            presented: PresentationClip::default(),
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
        &PresentationClip,
    )>,
    mut query_sprites: Query<(&mut Sprite, &LayerName, Option<&DirectionRow>), With<AnimatedLayer>>,
) {
    if freeze.is_frozen() {
        return;
    }
    for (childs, config_handles, mut timer, state, presented) in query.iter_mut() {
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

            // Animation jouée : la clé de présentation si la carte la définit, l'état sinon.
            let key = anim_config.playing_key(&state.0, presented.0.as_deref());
            timer.frame_timer.tick(time.delta());
            if timer.frame_timer.just_finished() {
                for child in childs.iter() {
                    if let Ok((mut sprite, layer, direction)) = query_sprites.get_mut(child.clone())
                    {
                        // Décalage de la ligne de direction (X-4a) : l'image d'animation est
                        // `atlas.index` sans lui, il y est remis après l'avance.
                        let row_offset = direction.map_or(0, |d| {
                            let layer_columns = config_handles
                                .spritesheets
                                .get(&layer.name)
                                .and_then(|h| spritesheet_configs.get(h))
                                .map_or(columns, |c| c.columns);
                            d.row * layer_columns as usize
                        });
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            let local = atlas.index.saturating_sub(row_offset);
                            let next = if let Some(indices) = anim_config.animations.get(key) {
                                let (start_index, end_index) = indices.to_absolute(columns);
                                if local < start_index || local > end_index {
                                    start_index
                                } else {
                                    (local + 1 - start_index) % (end_index - start_index + 1)
                                        + start_index
                                }
                            } else {
                                anim_config.animations.get("Idle").map_or(0, |idx| {
                                    let (start, _) = idx.to_absolute(columns);
                                    start
                                })
                            };
                            atlas.index = next + row_offset;
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
    mut query_sprite: Query<(
        &mut Sprite,
        &mut Transform,
        &mut Anchor,
        &LayerName,
        Option<&mut DirectionRow>,
    )>,
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
                                if let Ok((
                                    mut sprite,
                                    mut transform,
                                    mut anchor,
                                    layer_name,
                                    direction,
                                )) = query_sprite.get_mut(child.clone())
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
                                        // L'atlas repart de `starting_index` : ligne 0.
                                        if let Some(mut direction) = direction {
                                            direction.row = 0;
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
}

// SYSTEM THAT RUN ON THE BEVY SCHEDULE FOR SYNCH

/// Orientation des calques (présentation, `Update`) : retournement horizontal et, pour une planche
/// qui déclare `directions` (X-4a, §89), ligne de la direction dessinée la plus proche de
/// `FacingDirection` ([`SheetDirections::choose`]). Un calque sans déclaration garde le retournement
/// d'avant (`should_flip_x`). La simulation ne lit rien de tout cela.
pub fn set_sprite_flip(
    mut commands: Commands,
    sheets: Res<Assets<SpriteSheetConfig>>,
    query: Query<(
        &Children,
        &FacingDirection,
        Option<&CharacterAnimationHandles>,
    )>,
    mut sprite_query: Query<(&mut Sprite, Option<&LayerName>, Option<&mut DirectionRow>)>,
) {
    for (childrens, direction, handles) in query.iter() {
        for child in childrens.iter() {
            let Ok((mut sprite, layer, row)) = sprite_query.get_mut(child.clone()) else {
                continue;
            };
            let sheet = handles
                .zip(layer)
                .and_then(|(handles, layer)| handles.spritesheets.get(&layer.name))
                .and_then(|handle| sheets.get(handle));
            let declared = sheet.and_then(|sheet| sheet.directions.as_ref().map(|d| (sheet, d)));
            let Some((sheet, directions)) = declared else {
                // Flip sprite horizontally for left-facing directions
                sprite.flip_x = direction.should_flip_x();
                if let (Some(row), Some(sheet)) = (row, sheet) {
                    // Déclaration retirée (rechargement à chaud) : retour à la ligne 0.
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.index = atlas.index.saturating_sub(row.row * sheet.columns as usize);
                    }
                    commands.entity(child).try_remove::<DirectionRow>();
                }
                continue;
            };
            let choice = directions.choose(*direction);
            sprite.flip_x = choice.flip_x;
            match row {
                Some(mut row) => {
                    if row.row != choice.row {
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            let columns = sheet.columns as usize;
                            atlas.index = atlas.index.saturating_sub(row.row * columns)
                                + choice.row * columns;
                        }
                        row.row = choice.row;
                    }
                }
                // Posé à la ligne 0, où l'atlas démarre ; la ligne voulue est appliquée à la frame suivante.
                None => {
                    commands.entity(child).try_insert(DirectionRow::default());
                }
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

    if spritesheet_config.directions.is_some() {
        entity_commands.insert(DirectionRow::default());
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

/// X-4a (§89) : la ligne de direction se pose sur l'index d'atlas sans perdre l'image d'animation.
#[cfg(test)]
mod direction_row_tests {
    use super::*;

    const COLUMNS: u32 = 4;

    fn sheet(directions: Option<SheetDirections>, rows: u32) -> SpriteSheetConfig {
        SpriteSheetConfig {
            path: "x.png".into(),
            tile_size: (8, 8),
            columns: COLUMNS,
            rows,
            name: "body".into(),
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            offset_z: 0.0,
            animated: true,
            anchor: ConfigurableAnchor::Center,
            own_colors: false,
            directions,
        }
    }

    struct Rig {
        app: App,
        parent: Entity,
        layer: Entity,
    }

    fn rig(sheet: SpriteSheetConfig) -> Rig {
        let mut app = App::new();
        app.insert_resource(Assets::<SpriteSheetConfig>::default());
        app.insert_resource(Assets::<AnimationMapConfig>::default());
        app.insert_resource(Time::<()>::default());
        app.init_resource::<AnimationFreeze>();
        let sheet_handle = app
            .world_mut()
            .resource_mut::<Assets<SpriteSheetConfig>>()
            .add(sheet);
        let anim_handle = app
            .world_mut()
            .resource_mut::<Assets<AnimationMapConfig>>()
            .add(AnimationMapConfig {
                frame_duration: 1000,
                animations: [(
                    "Idle".to_string(),
                    AnimationIndices::Direct { start: 0, end: 3 },
                )]
                .into_iter()
                .collect(),
                columns: None,
            });
        let mut sprite = Sprite::default();
        sprite.texture_atlas = Some(TextureAtlas {
            layout: Handle::default(),
            index: 2,
        });
        let layer = app
            .world_mut()
            .spawn((
                sprite,
                LayerName {
                    name: "body".into(),
                },
                AnimatedLayer {},
            ))
            .id();
        let parent = app
            .world_mut()
            .spawn((
                FacingDirection::Right,
                AnimationState("Idle".into()),
                AnimationVisualsBundle::new(
                    [("body".to_string(), sheet_handle)].into_iter().collect(),
                    anim_handle,
                    0,
                ),
            ))
            .add_child(layer)
            .id();
        app.add_systems(Update, (set_sprite_flip, animate_sprite_system).chain());
        Rig { app, parent, layer }
    }

    impl Rig {
        fn face(&mut self, facing: FacingDirection) {
            *self
                .app
                .world_mut()
                .get_mut::<FacingDirection>(self.parent)
                .unwrap() = facing;
        }

        /// Une mise à jour, le temps avançant d'une seconde (le timer d'animation sonne à chaque fois).
        fn tick(&mut self) {
            self.app
                .world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs(1));
            self.app.update();
        }

        /// Une mise à jour sans temps écoulé : le timer d'animation ne sonne pas.
        fn settle(&mut self) {
            self.app
                .world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::ZERO);
            self.app.update();
        }

        fn state(&self) -> (usize, bool, Option<usize>) {
            let sprite = self.app.world().get::<Sprite>(self.layer).unwrap();
            (
                sprite.texture_atlas.as_ref().unwrap().index,
                sprite.flip_x,
                self.app
                    .world()
                    .get::<DirectionRow>(self.layer)
                    .map(|r| r.row),
            )
        }
    }

    #[test]
    fn a_sheet_without_directions_keeps_the_mirror_and_row_zero() {
        let mut rig = rig(sheet(None, 1));
        rig.face(FacingDirection::UpLeft);
        rig.tick();
        // Pas de ligne posée ; seule l'animation avance (2 → 3), retournée pour la gauche.
        assert_eq!(rig.state(), (3, true, None));
        rig.face(FacingDirection::Up);
        rig.tick();
        assert_eq!(rig.state(), (0, false, None));
    }

    #[test]
    fn eight_rows_follow_the_facing_and_keep_the_animation_frame() {
        let mut rig = rig(sheet(Some(SheetDirections::eight()), 8));
        rig.face(FacingDirection::Up);
        // 1re mise à jour : le calque reçoit son composant (ligne 0, l'atlas n'a pas bougé hors animation).
        rig.tick();
        assert_eq!(rig.state(), (3, false, Some(0)));
        // 2e : ligne `N` (2) ; l'image d'animation avance (3 → 0), le décalage est de 2 × 4 colonnes.
        rig.tick();
        assert_eq!(rig.state(), (2 * COLUMNS as usize, false, Some(2)));
        // Tourner ne change pas l'image jouée : ligne `W` (4), même colonne ; aucun retournement.
        rig.face(FacingDirection::Left);
        rig.settle();
        assert_eq!(rig.state(), (4 * COLUMNS as usize, false, Some(4)));
        rig.tick();
        assert_eq!(rig.state(), (4 * COLUMNS as usize + 1, false, Some(4)));
    }

    #[test]
    fn a_mirrored_sheet_flips_the_undrawn_side_on_the_row_it_chose() {
        let five = SheetDirections {
            rows: vec![Compass::S, Compass::SE, Compass::E, Compass::NE, Compass::N],
            mirror: true,
        };
        let mut rig = rig(sheet(Some(five), 5));
        rig.face(FacingDirection::DownLeft);
        rig.tick();
        rig.settle();
        // `SW` = miroir de `SE` (ligne 1), retourné.
        assert_eq!(rig.state(), (COLUMNS as usize + 3, true, Some(1)));
    }
}
