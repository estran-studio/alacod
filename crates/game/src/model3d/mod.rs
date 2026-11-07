//! Rendu 3D en direct « façon sprite » de ce qui bouge (R-1, `docs/conventions.md` § 111).
//!
//! Un personnage dont l'`asset_name_ref` a une entrée dans le fichier `Model` du jeu
//! (`content::models`) est dessiné par son modèle animé (`anime.glb`) au lieu de sa planche :
//!
//! 1. **Un banc par personnage** : le modèle est posé à l'origine d'un monde 3D qui n'est vu que par sa
//!    caméra (calque de rendu propre au personnage), orthographique, à l'élévation du jeu, sans lacet,
//!    cadrée à `units_per_meter` unités du monde par mètre. Elle dessine dans une image au fond
//!    transparent, sur-échantillonnée ([`SUPERSAMPLE`]).
//! 2. **La pose** : un quad enfant du personnage (donc au `z` du personnage, trié avec le décor) affiche
//!    l'image avec le contour d'un pixel de sprite et l'ombre ([`materials::ModelPoseMaterial`]). Le pied
//!    du modèle tombe exactement sur la position 2D : le sol du monde 2D est calé sur la caméra
//!    (1 m de profondeur = `sin(élévation)` m de largeur à l'écran).
//! 3. **Le matériau** du corps porte le shader du banc ([`materials::FaconSpriteMaterial`]) sur des
//!    normales lissées par position ; l'ombre est le maillage projeté au sol le long de la lumière.
//! 4. **L'animation** ([`animate`]) choisit le clip d'après l'état de la simulation déjà là (vitesse,
//!    visée, tir, détente, rechargement, coup, esquive), le fond avec le précédent et montre l'arme du
//!    manifeste dans la main.
//!
//! Présentation pure : `PresentationPlugin`, `Update`/`PostUpdate`, absent en headless. Rien n'est écrit
//! dans la simulation ; aucune trace ne bouge.

pub mod animate;
pub mod materials;
pub mod placement;
pub mod pose;

use std::collections::BTreeSet;

use bevy::{
    asset::RenderAssetUsages,
    camera::{
        visibility::{NoFrustumCulling, RenderLayers},
        RenderTarget, ScalingMode,
    },
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    gltf::{Gltf, GltfAssetLabel},
    mesh::{Indices, VertexAttributeValues},
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::{render_resource::TextureFormat, view::Msaa},
    world_serialization::WorldAssetRoot,
};
use content::{models::ModelsTable, Registry};

use crate::camera::GameCamera;
use materials::{FaconSpriteMaterial, ModelPoseMaterial, ModelShadowMaterial};

/// Texels de l'image d'un personnage par pixel de l'écran, sur chaque axe : la passe de pose en fait la
/// moyenne (anticrénelage) et distingue le corps de l'ombre texel par texel.
pub const SUPERSAMPLE: u32 = 2;
/// Côté maximal de l'image d'un personnage (pixels de l'écran), quel que soit le zoom.
const MAX_SIDE_PX: f32 = 1024.0;
/// Distance de la caméra du banc à l'origine (m) : hors de tout modèle.
const CAMERA_DISTANCE: f32 = 30.0;

pub struct ModelRenderPlugin;

impl Plugin for ModelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(materials::ModelMaterialsPlugin)
            .init_resource::<ModelLayers>()
            .init_resource::<PreparedAssets>()
            .add_systems(
                Update,
                (spawn_rigs, finish_rigs, resize_rigs, release_rigs).chain(),
            )
            .add_systems(
                PostUpdate,
                animate::animate_models
                    .after(crate::ui::weapon_visuals::aim_enemy_weapons)
                    .after(bevy_fixed::fixed_math::sync_bevy_transforms_from_fixed)
                    .before(bevy::app::AnimationSystems),
            )
            .add_systems(
                PostUpdate,
                (placement::offset_model_bullets, placement::y_sort)
                    .chain()
                    .after(crate::display_interpolation::interpolate_transforms)
                    .after(bevy_fixed::fixed_math::sync_bevy_transforms_from_fixed)
                    .before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

/// Posé par `character::visuals` sur un personnage dessiné par un modèle (id de la table `Model`).
#[derive(Component, Clone, Debug)]
pub struct CharacterModel(pub String);

/// Le banc 3D d'un personnage (présentation, hors rollback).
#[derive(Component)]
pub struct ModelRig {
    pub model: String,
    pub layer: usize,
    pub camera: Entity,
    pub root: Entity,
    pub quad: Entity,
    pub image: Handle<Image>,
    pub pose_material: Handle<ModelPoseMaterial>,
    gltf: Handle<Gltf>,
    /// Pixels de l'écran par unité du monde de l'image actuelle.
    pub px_per_unit: f32,
    pub ready: Option<RigReady>,
    pub state: animate::RigState,
}

/// Ce que l'animation trouve dans l'instance du modèle, une fois chargée.
pub struct RigReady {
    pub player: Entity,
    /// Clip -> (nœud du graphe, durée en s).
    pub clips: HashMap<String, (AnimationNodeIndex, f32)>,
    /// Nœuds nommés du modèle (objets et repères des armes du manifeste).
    pub nodes: HashMap<String, Entity>,
}

/// Lien d'une entité du banc (caméra, racine du modèle) vers son personnage ; la caméra porte le
/// calque, rendu quand elle disparaît.
#[derive(Component)]
pub struct RigPart {
    pub owner: Entity,
    pub layer: Option<usize>,
}

/// Calques de rendu pris par les bancs (le calque 0 est celui du jeu).
#[derive(Resource, Default)]
struct ModelLayers(BTreeSet<usize>);

impl ModelLayers {
    fn take(&mut self) -> usize {
        let layer = (1..).find(|n| !self.0.contains(n)).unwrap_or(1);
        self.0.insert(layer);
        layer
    }
}

/// Maillages déjà lissés et matériaux déjà convertis (partagés par les instances d'un modèle).
#[derive(Resource, Default)]
struct PreparedAssets {
    meshes: HashSet<AssetId<Mesh>>,
    materials: HashMap<AssetId<StandardMaterial>, Handle<FaconSpriteMaterial>>,
    shadow: Option<(Vec3, Handle<ModelShadowMaterial>)>,
}

/// Le calage du sol 2D sur la caméra : `(cos e, sin e)` de l'élévation.
pub fn elevation(table: &ModelsTable) -> (f32, f32) {
    let e = table.def.elevation_deg.to_radians();
    (e.cos(), e.sin())
}

/// Pixels de l'écran par unité du monde (zoom de la caméra du jeu et facteur d'échelle de sa cible),
/// arrondis au quart d'octave : l'image d'un personnage ne change de taille qu'à ces crans.
fn screen_px_per_unit(cameras: &Query<(&Camera, &Projection), With<GameCamera>>) -> f32 {
    let (scale_factor, scale) = cameras
        .iter()
        .next()
        .map(|(camera, projection)| {
            let scale = match projection {
                Projection::Orthographic(o) => o.scale,
                _ => 1.0,
            };
            (camera.target_scaling_factor().unwrap_or(1.0), scale)
        })
        .unwrap_or((1.0, 1.0));
    let raw = (scale_factor / scale.max(1e-3)).max(0.05);
    2f32.powf((raw.log2() * 4.0).round() / 4.0)
}

/// Taille de l'image (pixels de l'écran) d'un modèle au cadre `frame_m`.
fn image_size(table: &ModelsTable, frame_m: (f32, f32), px_per_unit: f32) -> UVec2 {
    let u = table.def.units_per_meter;
    let side = |m: f32| (m * u * px_per_unit).ceil().clamp(1.0, MAX_SIDE_PX) as u32;
    UVec2::new(side(frame_m.0), side(frame_m.1))
}

fn pose_params(table: &ModelsTable, size: UVec2, px_per_unit: f32) -> (Vec4, Vec4) {
    let look = &table.def.look;
    let radius = (px_per_unit * table.def.units_per_meter / look.outline_px_per_m).clamp(1.0, 3.0);
    let shadow = look.shadow.as_ref().map_or(0.0, |s| s.opacity);
    (
        Vec4::new(size.x as f32, size.y as f32, radius, look.outline),
        Vec4::new(shadow, SUPERSAMPLE as f32, 0.0, 0.0),
    )
}

fn target_image(size: UVec2) -> Image {
    let mut image = Image::new_target_texture(
        size.x * SUPERSAMPLE,
        size.y * SUPERSAMPLE,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image
}

/// Crée le banc des personnages à modèle qui n'en ont pas : image, caméra, instance du modèle, quad.
#[allow(clippy::too_many_arguments)]
fn spawn_rigs(
    mut commands: Commands,
    registry: Option<Res<Registry>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pose_materials: ResMut<Assets<ModelPoseMaterial>>,
    mut layers: ResMut<ModelLayers>,
    cameras: Query<(&Camera, &Projection), With<GameCamera>>,
    characters: Query<(Entity, &CharacterModel), Without<ModelRig>>,
) {
    let Some(table) = registry.as_ref().and_then(|r| r.models.as_ref()) else {
        return;
    };
    let px_per_unit = screen_px_per_unit(&cameras);
    for (character, CharacterModel(id)) in &characters {
        let Some(model) = table.def.models.get(id) else {
            continue;
        };
        let u = table.def.units_per_meter;
        let size = image_size(table, model.frame_m, px_per_unit);
        let image = images.add(target_image(size));
        let layer = layers.take();
        let (cos_e, sin_e) = elevation(table);

        let camera = commands
            .spawn((
                Camera3d::default(),
                Camera {
                    // Avant la caméra du jeu : l'image posée est celle de cette image-ci.
                    order: -1,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
                RenderTarget::Image(image.clone().into()),
                Projection::Orthographic(OrthographicProjection {
                    near: 0.0,
                    far: 2.0 * CAMERA_DISTANCE,
                    viewport_origin: Vec2::new(model.pivot.0, model.pivot.1),
                    scaling_mode: ScalingMode::Fixed {
                        width: model.frame_m.0,
                        height: model.frame_m.1,
                    },
                    ..OrthographicProjection::default_3d()
                }),
                Transform::from_translation(Vec3::new(0.0, sin_e, cos_e) * CAMERA_DISTANCE)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                Msaa::Off,
                Tonemapping::None,
                DebandDither::Disabled,
                RenderLayers::layer(layer),
                RigPart {
                    owner: character,
                    layer: Some(layer),
                },
            ))
            .id();
        let root = commands
            .spawn((
                WorldAssetRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset(model.glb.clone())),
                ),
                Transform::IDENTITY,
                Visibility::default(),
                RenderLayers::layer(layer),
                RigPart {
                    owner: character,
                    layer: None,
                },
            ))
            .id();
        let (params, extra) = pose_params(table, size, px_per_unit);
        let pose_material = pose_materials.add(ModelPoseMaterial {
            params,
            extra,
            image: image.clone(),
        });
        let (w, h) = (model.frame_m.0 * u, model.frame_m.1 * u);
        let quad = commands
            .spawn((
                Mesh2d(meshes.add(Rectangle::new(w, h))),
                MeshMaterial2d(pose_material.clone()),
                Transform::from_xyz(
                    (0.5 - model.pivot.0) * w,
                    (0.5 - model.pivot.1) * h,
                    model.offset_z,
                ),
            ))
            .id();
        commands.entity(character).add_child(quad);
        commands.entity(character).try_insert(ModelRig {
            model: id.clone(),
            layer,
            camera,
            root,
            quad,
            image,
            pose_material,
            gltf: asset_server.load(model.glb.clone()),
            px_per_unit,
            ready: None,
            state: default(),
        });
    }
}

/// Une fois le modèle instancié et ses clips chargés : matériau « façon sprite » et normales lissées
/// sur chaque maillage, ombre, calque du banc, graphe d'animation de tous les clips.
#[allow(clippy::too_many_arguments)]
fn finish_rigs(
    mut commands: Commands,
    registry: Option<Res<Registry>>,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut meshes: ResMut<Assets<Mesh>>,
    standard: Res<Assets<StandardMaterial>>,
    mut facon: ResMut<Assets<FaconSpriteMaterial>>,
    mut shadows: ResMut<Assets<ModelShadowMaterial>>,
    mut prepared: ResMut<PreparedAssets>,
    mut rigs: Query<&mut ModelRig>,
    children: Query<&Children>,
    players: Query<(), With<AnimationPlayer>>,
    names: Query<&Name>,
    model_meshes: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>)>,
    skins: Query<&bevy::mesh::skinning::SkinnedMesh>,
) {
    let Some(table) = registry.as_ref().and_then(|r| r.models.as_ref()) else {
        return;
    };
    let look = &table.def.look;
    let light = Vec3::from(look.light).normalize_or_zero();
    for mut rig in rigs.iter_mut().filter(|rig| rig.ready.is_none()) {
        let Some(gltf) = gltfs.get(&rig.gltf) else {
            continue;
        };
        let Some(player) = children
            .iter_descendants(rig.root)
            .find(|e| players.contains(*e))
        else {
            continue;
        };
        let layer = RenderLayers::layer(rig.layer);

        // Graphe : un nœud par clip nommé (le nom après un éventuel « Armature| »).
        let mut graph = AnimationGraph::new();
        let mut clip_nodes = HashMap::default();
        for (name, handle) in &gltf.named_animations {
            let name = name.rsplit('|').next().unwrap_or(name).to_string();
            let duration = clips.get(handle).map_or(1.0, |c| c.duration()).max(1e-3);
            let node = graph.add_clip(handle.clone(), 1.0, graph.root);
            clip_nodes.insert(name, (node, duration));
        }
        commands
            .entity(player)
            .insert(AnimationGraphHandle(graphs.add(graph)));

        let shadow_material = look.shadow.as_ref().map(|s| {
            let direction = Vec3::from(s.direction).normalize_or_zero();
            match &prepared.shadow {
                Some((d, handle)) if *d == direction => handle.clone(),
                _ => {
                    let handle = shadows.add(ModelShadowMaterial {
                        direction: direction.extend(0.0),
                    });
                    prepared.shadow = Some((direction, handle.clone()));
                    handle
                }
            }
        });

        let mut nodes = HashMap::default();
        for entity in children.iter_descendants(rig.root).collect::<Vec<_>>() {
            if let Ok(name) = names.get(entity) {
                nodes.insert(name.as_str().to_string(), entity);
            }
            let mut e = commands.entity(entity);
            e.insert(layer.clone());
            let Ok((mesh, material)) = model_meshes.get(entity) else {
                continue;
            };
            if prepared.meshes.insert(mesh.0.id()) {
                if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
                    smooth_normals(&mut mesh);
                }
            }
            let cel = prepared
                .materials
                .entry(material.0.id())
                .or_insert_with(|| {
                    let source = standard.get(&material.0);
                    let texture = source.and_then(|m| m.base_color_texture.clone());
                    let color = source.map_or(Color::WHITE, |m| m.base_color).to_linear();
                    facon.add(FaconSpriteMaterial {
                        color: Vec4::new(
                            color.red,
                            color.green,
                            color.blue,
                            if texture.is_some() { 1.0 } else { 0.0 },
                        ),
                        emissive: Vec4::ZERO,
                        light: light.extend(0.0),
                        ramp: Vec4::new(look.shade, look.highlight, 0.0, 0.0),
                        texture,
                    })
                })
                .clone();
            e.remove::<MeshMaterial3d<StandardMaterial>>()
                .insert((MeshMaterial3d(cel), NoFrustumCulling));
            if let Some(shadow) = &shadow_material {
                let mut copy = e.commands();
                let child = copy
                    .spawn((
                        Mesh3d(mesh.0.clone()),
                        MeshMaterial3d(shadow.clone()),
                        Transform::IDENTITY,
                        layer.clone(),
                        NoFrustumCulling,
                    ))
                    .id();
                // Un maillage à squelette : l'ombre suit le même squelette.
                if let Ok(skin) = skins.get(entity) {
                    copy.entity(child).insert(skin.clone());
                }
                copy.entity(entity).add_child(child);
            }
        }
        rig.ready = Some(RigReady {
            player,
            clips: clip_nodes,
            nodes,
        });
    }
}

/// Normales lissées par position (`facon-sprite.js`, `normalesLissees`) : la normale d'un sommet est la
/// somme des faces qui touchent sa position, chaque face retournée au besoin pour s'accorder à la somme
/// (le shader tourne ensuite la normale vers la caméra). Les coutures d'UV et les facettes ne cassent
/// plus le modelé ; les triangles des maillages générés sont orientés au hasard.
pub fn smooth_normals(mesh: &mut Mesh) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION).cloned()
    else {
        return;
    };
    let key = |p: &[f32; 3]| {
        (
            (p[0] * 1e4).round() as i64,
            (p[1] * 1e4).round() as i64,
            (p[2] * 1e4).round() as i64,
        )
    };
    let indices: Vec<usize> = match mesh.indices() {
        Some(Indices::U16(v)) => v.iter().map(|&i| i as usize).collect(),
        Some(Indices::U32(v)) => v.iter().map(|&i| i as usize).collect(),
        None => (0..positions.len()).collect(),
    };
    let mut sums: std::collections::BTreeMap<(i64, i64, i64), Vec3> = Default::default();
    for tri in indices.chunks_exact(3) {
        let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| Vec3::from(positions[i]));
        let face = (c - b).cross(a - b);
        for &i in tri {
            sums.entry(key(&positions[i]))
                .and_modify(|s| *s += if s.dot(face) < 0.0 { -face } else { face })
                .or_insert(face);
        }
    }
    let normals: Vec<[f32; 3]> = positions
        .iter()
        .map(|p| {
            sums.get(&key(p))
                .map_or(Vec3::Y, |s| s.normalize_or(Vec3::Y))
                .to_array()
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
}

/// Suit le zoom : l'image d'un personnage garde un texel par pixel de l'écran (× [`SUPERSAMPLE`]).
fn resize_rigs(
    registry: Option<Res<Registry>>,
    cameras: Query<(&Camera, &Projection), With<GameCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut pose_materials: ResMut<Assets<ModelPoseMaterial>>,
    mut rigs: Query<&mut ModelRig>,
) {
    let Some(table) = registry.as_ref().and_then(|r| r.models.as_ref()) else {
        return;
    };
    let px_per_unit = screen_px_per_unit(&cameras);
    for mut rig in &mut rigs {
        if rig.px_per_unit == px_per_unit {
            continue;
        }
        let Some(model) = table.def.models.get(&rig.model) else {
            continue;
        };
        rig.px_per_unit = px_per_unit;
        let size = image_size(table, model.frame_m, px_per_unit);
        if let Some(mut image) = images.get_mut(&rig.image) {
            *image = target_image(size);
        }
        if let Some(mut material) = pose_materials.get_mut(&rig.pose_material) {
            (material.params, material.extra) = pose_params(table, size, px_per_unit);
        }
    }
}

/// Le banc suit son personnage : caméra éteinte tant qu'il est désactivé (`despawn_rollback`,
/// ressuscitable), banc détruit avec lui (le quad, enfant, part avec le personnage) et calque rendu.
fn release_rigs(
    mut commands: Commands,
    mut layers: ResMut<ModelLayers>,
    parts: Query<(Entity, &RigPart)>,
    alive: Query<(), With<ModelRig>>,
    disabled: Query<(), With<bevy_ggrs::RollbackDespawned>>,
    mut cameras: Query<&mut Camera>,
) {
    for (entity, part) in &parts {
        let shown = alive.contains(part.owner);
        if shown || disabled.contains(part.owner) {
            if let Ok(mut camera) = cameras.get_mut(entity) {
                if camera.is_active != shown {
                    camera.is_active = shown;
                }
            }
            continue;
        }
        commands.entity(entity).despawn();
        if let Some(layer) = part.layer {
            layers.0.remove(&layer);
        }
    }
}
