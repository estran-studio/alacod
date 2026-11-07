//! Placement à l'écran (R-1, `docs/conventions.md` § 111) : tri en y des personnages et des balles, et
//! balles qui partent de la bouche du canon des modèles. Présentation : n'écrit que le `Transform`
//! bevy, après sa recopie depuis la simulation (`sync_bevy_transforms_from_fixed`) et l'interpolation
//! d'affichage ; la position simulée de la balle (`FixedTransform3D`) ne change pas.

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::weapons::Bullet;
use content::Registry;
use utils::{frame::FrameCount, net_id::GgrsNetId};

use super::ModelRig;
use crate::character::appearance::CharacterAppearance;

/// `z` retiré par unité de `y` : sur une carte de 5 000 unités, l'écart reste sous 0,5 (sous l'écart
/// entre deux couches de la carte), et un enfant posé à +0,1 (barre de vie) passe devant ses voisins.
pub const Y_SORT_PER_UNIT: f32 = 1e-4;
/// Frames pendant lesquelles l'écart latéral entre la bouche du canon dessinée et le point de tir
/// simulé se résorbe ; la hauteur de la bouche, elle, reste (la balle vole à hauteur du canon, son
/// ombre au sol est la position simulée).
pub const MUZZLE_BLEND_FRAMES: f32 = 8.0;

/// Écart d'une balle de modèle entre sa position simulée et son dessin.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ModelBulletOffset {
    lateral: Vec2,
    height: f32,
    born: u32,
}

/// Le plus bas à l'écran devant (`y_sort` du fichier `Model`) : `z` de la simulation moins `y` ×
/// [`Y_SORT_PER_UNIT`], pour les personnages (corps 3D et décor en sprites) et les balles (à leur pied).
pub fn y_sort(
    registry: Option<Res<Registry>>,
    mut sorted: Query<
        (&FixedTransform3D, &mut Transform),
        Or<(With<CharacterAppearance>, With<Bullet>)>,
    >,
) {
    if !registry
        .as_ref()
        .and_then(|r| r.models.as_ref())
        .is_some_and(|t| t.def.y_sort)
    {
        return;
    }
    for (fixed, mut transform) in &mut sorted {
        let z = fixed.translation.z.to_num::<f32>()
            - fixed.translation.y.to_num::<f32>() * Y_SORT_PER_UNIT;
        if transform.translation.z != z {
            transform.translation.z = z;
        }
    }
}

/// Une balle tirée par un modèle part, à l'écran, de la bouche de son canon (repère du modèle, mesuré
/// à l'image) : décalée de la bouche dessinée, l'écart latéral se résorbe en
/// [`MUZZLE_BLEND_FRAMES`] frames, la hauteur reste.
#[allow(clippy::type_complexity)]
pub fn offset_model_bullets(
    mut commands: Commands,
    frame: Option<Res<FrameCount>>,
    registry: Option<Res<Registry>>,
    mut bullets: Query<(
        Entity,
        &Bullet,
        &FixedTransform3D,
        &mut Transform,
        Option<&ModelBulletOffset>,
    )>,
    shooters: Query<(&GgrsNetId, &ModelRig, &FixedTransform3D)>,
) {
    if registry.as_ref().and_then(|r| r.models.as_ref()).is_none() {
        return;
    }
    let f = frame.map_or(0, |frame| frame.frame);
    for (entity, bullet, fixed, mut transform, offset) in &mut bullets {
        let offset = match offset {
            Some(offset) => *offset,
            None => {
                let offset = shooters
                    .iter()
                    .find(|(id, _, _)| **id == bullet.source)
                    .and_then(|(_, rig, shooter)| {
                        let muzzle = rig.state.muzzle?;
                        let foot = Vec2::new(
                            shooter.translation.x.to_num::<f32>(),
                            shooter.translation.y.to_num::<f32>(),
                        );
                        // Le point de tir simulé : la balle recule des frames déjà jouées.
                        let age = f.saturating_sub(bullet.created_at) as f32;
                        let velocity = Vec2::new(
                            bullet.velocity.x.to_num::<f32>(),
                            bullet.velocity.y.to_num::<f32>(),
                        );
                        let spawn = Vec2::new(
                            fixed.translation.x.to_num::<f32>(),
                            fixed.translation.y.to_num::<f32>(),
                        ) - velocity * age;
                        let drawn = foot + muzzle.screen;
                        Some(ModelBulletOffset {
                            lateral: drawn - spawn - Vec2::Y * muzzle.height,
                            height: muzzle.height,
                            born: bullet.created_at,
                        })
                    })
                    .unwrap_or_default();
                commands.entity(entity).try_insert(offset);
                offset
            }
        };
        if offset.height == 0.0 && offset.lateral == Vec2::ZERO {
            continue;
        }
        let blend = (1.0 - f.saturating_sub(offset.born) as f32 / MUZZLE_BLEND_FRAMES).max(0.0);
        let shift = offset.lateral * blend + Vec2::Y * offset.height;
        transform.translation.x += shift.x;
        transform.translation.y += shift.y;
    }
}
