//! Visuels de tir et de mort (S1-6, `docs/conventions.md` §94) : flash de bouche, recul visuel de
//! l'arme, éclats à la mort d'un ennemi. Réglés par jeu dans `ui/feedback.ron` (`muzzle_flash`,
//! `recoil`, `death`, surcharges `by_weapon`).
//!
//! **Présentation seule**, dérivée de l'état dans `Update`/`PostUpdate` : une balle apparue
//! (`Added<Bullet>`), un ennemi disparu. Rien n'est rollback, rien n'entre dans le checksum ; le
//! recul ne déplace que le `Transform` bevy du sprite de l'arme, réécrit à chaque image par la
//! recopie de `FixedTransform3D` : la trajectoire des balles ne change pas.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedTransform3D};
use utils::{frame::FrameCount, net_id::GgrsNetId};

use crate::feedback_log::FeedbackConfigLoaded;

/// Flash de bouche à l'écran : né à la frame `born`, estompé en `frames` frames de simulation.
#[derive(Component, Debug)]
pub struct MuzzleFlash {
    born: u32,
    frames: u32,
    color: Color,
}

/// Recul en cours sur le sprite d'une arme : décalage de `distance` px opposé à `direction`,
/// ramené à zéro en `frames` frames de simulation.
#[derive(Component, Debug)]
pub struct WeaponKick {
    born: u32,
    frames: u32,
    distance: f32,
    direction: Vec2,
}

/// Frames pendant lesquelles un tir déjà montré reste mémorisé : le rollback de la session locale
/// détruit puis recrée les balles des dernières frames, elles redeviennent `Added<Bullet>`.
const SHOT_MEMORY_FRAMES: u32 = 120;

/// Hauteur d'affichage du flash (au-dessus des personnages, comme les éclats).
const FLASH_Z: f32 = 46.0;

/// Part restante d'un effet de `frames` frames né à `born`, vue à la frame `now` ; `None` s'il est
/// fini ou né d'une frame que la simulation a depuis rejouée plus tôt.
pub fn remaining(born: u32, frames: u32, now: u32) -> Option<f32> {
    let age = now.checked_sub(born)?;
    (age < frames).then(|| 1.0 - age as f32 / frames as f32)
}

/// Décalage du sprite de l'arme quand il reste `left` (de 1 à 0) du recul : plein recul au tir,
/// en `left²` ensuite (retour rapide d'abord, puis en douceur).
pub fn kick_offset(direction: Vec2, distance: f32, left: f32) -> Vec2 {
    -direction * distance * left * left
}

/// Un flash et un recul par tir (tireur, frame) : un fusil à pompe en tire huit plombs.
fn spawn_shot_visuals(
    mut commands: Commands,
    frame: Res<FrameCount>,
    config: Option<Res<FeedbackConfigLoaded>>,
    shooters: Query<(&GgrsNetId, &crate::weapons::WeaponInventory)>,
    new_bullets: Query<(&crate::weapons::Bullet, &FixedTransform3D), Added<crate::weapons::Bullet>>,
    mut seen: Local<BTreeSet<(usize, u32)>>,
) {
    let now = frame.frame;
    seen.retain(|&(_, at)| at <= now && now - at <= SHOT_MEMORY_FRAMES);
    let Some(config) = config else {
        return;
    };
    if config.0.muzzle_flash.is_none()
        && config.0.recoil.is_none()
        && config
            .0
            .by_weapon
            .values()
            .all(|entry| entry.muzzle_flash.is_none() && entry.recoil.is_none())
    {
        return;
    }
    let mut inventories: Option<BTreeMap<usize, &crate::weapons::WeaponInventory>> = None;
    for (bullet, transform) in &new_bullets {
        if !seen.insert((bullet.source.0, bullet.created_at)) {
            continue;
        }
        let inventories = inventories.get_or_insert_with(|| {
            shooters
                .iter()
                .map(|(net_id, inventory)| (net_id.0, inventory))
                .collect()
        });
        let active = inventories
            .get(&bullet.source.0)
            .and_then(|inventory| inventory.weapons.get(inventory.active_weapon_index));
        let (flash, recoil) = config
            .0
            .resolve_shot(active.map(|(_, weapon)| weapon.config.name.as_str()));
        // `velocity` : déplacement par frame simulée.
        let velocity = Vec2::new(
            bullet.velocity.x.to_num::<f32>(),
            bullet.velocity.y.to_num::<f32>(),
        );
        let direction = velocity.normalize_or_zero();
        if let Some(flash) = flash {
            // La balle a pu avancer depuis sa création : le flash naît à la sortie du canon.
            let age = now.saturating_sub(bullet.created_at) as f32;
            let origin =
                fixed_math::fixed_to_vec3(transform.translation).truncate() - velocity * age;
            let (r, g, b, a) = flash.color;
            let color = Color::linear_rgba(r, g, b, a);
            commands.spawn((
                MuzzleFlash {
                    born: bullet.created_at,
                    frames: flash.frames,
                    color,
                },
                Sprite::from_color(color, Vec2::new(flash.size, flash.size * 2.0 / 3.0)),
                Transform::from_xyz(origin.x, origin.y, FLASH_Z)
                    .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ));
        }
        if let (Some(recoil), Some((weapon, _))) = (recoil, active) {
            if recoil.distance > 0.0 && direction != Vec2::ZERO {
                commands.entity(*weapon).try_insert(WeaponKick {
                    born: bullet.created_at,
                    frames: recoil.frames,
                    distance: recoil.distance,
                    direction,
                });
            }
        }
    }
}

/// Estompe les flashs ; retire ceux qui sont finis.
fn animate_muzzle_flashes(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut flashes: Query<(Entity, &MuzzleFlash, &mut Sprite)>,
) {
    for (entity, flash, mut sprite) in &mut flashes {
        match remaining(flash.born, flash.frames, frame.frame) {
            Some(left) => sprite.color = flash.color.with_alpha(flash.color.alpha() * left),
            None => commands.entity(entity).despawn(),
        }
    }
}

/// Décale le sprite de l'arme pendant son recul. Après la recopie `FixedTransform3D` → `Transform`
/// et l'interpolation (le décalage ne s'accumule pas d'une image à l'autre), avant la propagation.
fn apply_weapon_kicks(
    mut commands: Commands,
    frame: Res<FrameCount>,
    mut weapons: Query<(Entity, &WeaponKick, &mut Transform)>,
) {
    for (entity, kick, mut transform) in &mut weapons {
        match remaining(kick.born, kick.frames, frame.frame) {
            Some(left) => {
                let offset = kick_offset(kick.direction, kick.distance, left);
                transform.translation.x += offset.x;
                transform.translation.y += offset.y;
            }
            None => {
                commands.entity(entity).try_remove::<WeaponKick>();
            }
        }
    }
}

/// Éclats à la mort d'un ennemi : un `GgrsNetId` d'ennemi présent à l'image précédente et absent
/// à celle-ci (même règle que le son `death`), à sa dernière position. Un retour de frame
/// (nouvelle partie) réarme sans rien montrer.
fn spawn_death_bursts(
    mut commands: Commands,
    frame: Res<FrameCount>,
    config: Option<Res<FeedbackConfigLoaded>>,
    enemies: Query<(&GgrsNetId, &FixedTransform3D), With<crate::character::enemy::Enemy>>,
    mut previous: Local<BTreeMap<usize, Vec2>>,
    mut last_frame: Local<Option<u32>>,
) {
    let Some(death) = config
        .map(|config| config.0.death)
        .filter(|death| death.enabled)
    else {
        return;
    };
    let now: BTreeMap<usize, Vec2> = enemies
        .iter()
        .map(|(net_id, transform)| {
            (
                net_id.0,
                fixed_math::fixed_to_vec3(transform.translation).truncate(),
            )
        })
        .collect();
    let rewound = last_frame.is_some_and(|last| frame.frame < last);
    *last_frame = Some(frame.frame);
    let before = std::mem::replace(&mut *previous, now);
    if rewound {
        return;
    }
    let (r, g, b, a) = death.color;
    for (net_id, position) in before {
        if !previous.contains_key(&net_id) {
            crate::feedback::spawn_debris(
                &mut commands,
                frame.frame,
                position,
                death.count,
                Color::linear_rgba(r, g, b, a),
                death.size,
            );
        }
    }
}

/// Visuels de tir et de mort (ajouté par `feedback::FeedbackPlugin`, rendu seulement).
pub struct ShotFeedbackPlugin;

impl Plugin for ShotFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_shot_visuals,
                animate_muzzle_flashes,
                spawn_death_bursts,
            ),
        )
        .add_systems(
            PostUpdate,
            apply_weapon_kicks
                .after(crate::display_interpolation::interpolate_transforms)
                .after(fixed_math::sync_bevy_transforms_from_fixed)
                .before(bevy::transform::TransformSystems::Propagate),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_effet_dure_ses_frames_et_s_efface_au_retour_en_arriere() {
        assert_eq!(remaining(10, 4, 10), Some(1.0));
        assert_eq!(remaining(10, 4, 12), Some(0.5));
        assert_eq!(remaining(10, 4, 14), None);
        assert_eq!(remaining(10, 4, 9), None);
    }

    #[test]
    fn le_recul_s_oppose_au_tir_et_revient() {
        let right = Vec2::X;
        assert_eq!(kick_offset(right, 3.0, 1.0), Vec2::new(-3.0, 0.0));
        assert_eq!(kick_offset(right, 3.0, 0.5), Vec2::new(-0.75, 0.0));
        assert_eq!(kick_offset(right, 3.0, 0.0).x, 0.0);
    }
}
