//! Matériaux du rendu « façon sprite » (R-1, `docs/conventions.md` § 111) : le corps (rampe de lumière),
//! son ombre portée et la pose de son image sur le décor (contour). Shaders embarqués (natif et web).

use bevy::{
    asset::embedded_asset,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

const FACON_SPRITE_SHADER: &str = "embedded://game/model3d/facon_sprite.wgsl";
const SHADOW_SHADER: &str = "embedded://game/model3d/model_shadow.wgsl";
const POSE_SHADER: &str = "embedded://game/model3d/model_pose.wgsl";

pub struct ModelMaterialsPlugin;

impl Plugin for ModelMaterialsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "facon_sprite.wgsl");
        embedded_asset!(app, "model_shadow.wgsl");
        embedded_asset!(app, "model_pose.wgsl");
        app.add_plugins((
            MaterialPlugin::<FaconSpriteMaterial>::default(),
            MaterialPlugin::<ModelShadowMaterial>::default(),
            Material2dPlugin::<ModelPoseMaterial>::default(),
        ));
    }
}

/// Le corps : couleur de la texture (sRGB) × rampe `ramp.x` (ombre) → `ramp.y` (reflet).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct FaconSpriteMaterial {
    /// rgb : couleur sans texture ; a : 1 si `texture` est posée.
    #[uniform(0)]
    pub color: Vec4,
    /// rgb : émission (flash d'un coup), sRGB.
    #[uniform(0)]
    pub emissive: Vec4,
    /// xyz : lumière, repère de la vue.
    #[uniform(0)]
    pub light: Vec4,
    /// x : ombre, y : reflet.
    #[uniform(0)]
    pub ramp: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Material for FaconSpriteMaterial {
    fn fragment_shader() -> ShaderRef {
        FACON_SPRITE_SHADER.into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Les maillages des modèles générés n'ont pas d'« extérieur » fiable : aucune face cachée.
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// L'ombre : le maillage projeté au sol le long de `direction` (vers la lumière, repère du monde).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ModelShadowMaterial {
    #[uniform(0)]
    pub direction: Vec4,
}

impl Material for ModelShadowMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADOW_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADOW_SHADER.into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// La pose de l'image d'un personnage sur le décor : contour et ombre (`model_pose.wgsl`).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ModelPoseMaterial {
    /// xy : taille de sortie (pixels), z : rayon du contour (pixels), w : teinte du contour.
    #[uniform(0)]
    pub params: Vec4,
    /// x : opacité de l'ombre, y : texels par pixel de sortie.
    #[uniform(0)]
    pub extra: Vec4,
    #[texture(1, sample_type = "float", filterable = false)]
    pub image: Handle<Image>,
}

impl Material2d for ModelPoseMaterial {
    fn fragment_shader() -> ShaderRef {
        POSE_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
