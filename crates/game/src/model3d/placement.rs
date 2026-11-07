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

/// Le plus bas à l'écran devant (`y_sort` du fichier `Model`) : le `z` affiché (celui de la simulation, ou
/// de la couche des entités pour un personnage posé à `z = 0`, `map_ldtk::plugins::lift_characters_to_entity_layer`,
/// ordonné avant) moins `y` × [`Y_SORT_PER_UNIT`], pour les personnages (corps 3D et décor importé en
/// personnage) et les balles (à leur pied). Le `Transform` est réécrit depuis la simulation à chaque image :
/// le retrait ne s'accumule pas.
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
        transform.translation.z -= fixed.translation.y.to_num::<f32>() * Y_SORT_PER_UNIT;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::{Fixed, FixedMat3, FixedVec3};
    use content::models::{ModelsFile, ModelsTable};

    fn registry(y_sort: bool) -> Registry {
        let def: ModelsFile = ron::from_str(&format!(
            "(units_per_meter: 24.0, elevation_deg: 45.0, y_sort: {y_sort}, look: (shade: 0.66, highlight: 1.12, \
             light: (-0.5, 0.75, 0.45), outline: 0.75, outline_px_per_m: 91.0), models: {{}})"
        ))
        .unwrap();
        Registry {
            models: Some(ModelsTable {
                file: "models/models.ron".into(),
                def,
                manifests: Default::default(),
            }),
            ..Default::default()
        }
    }

    /// Un personnage en `(y, z)` : `z` est celui de sa couche (recopié de la simulation).
    fn character(app: &mut App, y: f32, z: f32) -> Entity {
        app.world_mut()
            .spawn((
                CharacterAppearance {
                    config_name: "c".into(),
                    skin: None,
                    health_bar_color: Color::WHITE,
                },
                FixedTransform3D::new(
                    FixedVec3::new(Fixed::ZERO, Fixed::from_num(y), Fixed::from_num(z)),
                    FixedMat3::IDENTITY,
                    FixedVec3::ONE,
                ),
                Transform::from_xyz(0.0, y, z),
            ))
            .id()
    }

    fn z(app: &App, e: Entity) -> f32 {
        app.world().get::<Transform>(e).unwrap().translation.z
    }

    /// À couche égale, le pied le plus bas à l'écran passe devant (le poteau devant le joueur qui monte
    /// derrière lui), sans quitter la couche (écart sous 0,1 : la barre de vie, à +0,1, reste devant son
    /// porteur). Sans `y_sort`, rien ne bouge.
    #[test]
    fn le_pied_le_plus_bas_passe_devant() {
        let mut app = App::new();
        app.insert_resource(registry(true))
            .add_systems(Update, y_sort);
        let player = character(&mut app, -472.0, 3.0);
        let pole = character(&mut app, -488.0, 3.0);
        let behind = character(&mut app, -400.0, 3.0);
        app.update();
        assert!(
            z(&app, pole) > z(&app, player),
            "le poteau (plus bas) devant le joueur"
        );
        assert!(
            z(&app, player) > z(&app, behind),
            "le joueur devant le décor plus haut"
        );
        assert!((z(&app, pole) - z(&app, behind)).abs() < 0.1);

        let mut off = App::new();
        off.insert_resource(registry(false))
            .add_systems(Update, y_sort);
        let player = character(&mut off, -472.0, 3.0);
        let pole = character(&mut off, -488.0, 3.0);
        off.update();
        assert_eq!((z(&off, player), z(&off, pole)), (3.0, 3.0));
    }
}
