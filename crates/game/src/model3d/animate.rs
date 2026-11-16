//! Application d'un modèle animé au rendu (R-1, `docs/conventions.md` § 111) : la pose décidée hors rendu
//! ([`super::playback::ModelPlayback`], écrite par `decide_poses`, aussi en headless) est posée sur le lecteur
//! d'animation du GLB, le cap du corps sur sa racine, l'arme du manifeste dans la main, et la bouche du canon est
//! mesurée. Un rollback qui corrige l'état de la simulation corrige la pose à l'image suivante.

use std::collections::BTreeMap;

use bevy::prelude::*;
use combat::weapons::Weapon;
use content::{
    models::{AnimeManifest, HAND_SIDE, TWO_HANDED_GRIP, WEAPON_SIDE},
    Registry,
};

use super::{
    elevation,
    playback::{ModelPlayback, Track},
    ModelRig,
};

/// Bouche du canon d'un modèle : décalage à l'écran depuis le pied (unités du monde) et la part de ce
/// décalage due à sa hauteur au-dessus du sol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Muzzle {
    pub screen: Vec2,
    pub height: f32,
}

/// Côté de l'arme `weapon` du manifeste tenu dans `clip` (`armer` du banc) : l'os de l'arme pour une
/// prise à deux mains si l'arme en a un, la main sinon ; `None` si le clip ne tient pas cette arme.
pub fn held_side<'a>(manifest: &'a AnimeManifest, weapon: &str, clip: &str) -> Option<&'a str> {
    let w = manifest.weapons.get(weapon)?;
    if !w.animations.iter().any(|a| a == clip) {
        return None;
    }
    let two_hands = manifest.grips.get(clip).map(String::as_str) == Some(TWO_HANDED_GRIP);
    Some(if two_hands && w.objects.contains_key(WEAPON_SIDE) {
        WEAPON_SIDE
    } else {
        HAND_SIDE
    })
}

/// Un objet porté (la clé à la ceinture) : aucun de ses côtés n'est la main ni l'os de l'arme.
pub fn is_worn(weapon: &content::models::ManifestWeapon) -> bool {
    !weapon.objects.is_empty()
        && weapon
            .objects
            .keys()
            .all(|side| side != HAND_SIDE && side != WEAPON_SIDE)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn animate_models(
    registry: Option<Res<Registry>>,
    mut rigs: Query<(&mut ModelRig, &ModelPlayback)>,
    mut roots: Query<&mut Transform, (Without<Weapon>, Without<ModelRig>)>,
    mut players: Query<&mut AnimationPlayer>,
    mut visibility: Query<&mut Visibility>,
    globals: Query<&GlobalTransform>,
) {
    let Some(table) = registry.as_ref().and_then(|r| r.models.as_ref()) else {
        return;
    };
    let u = table.def.units_per_meter;
    let (cos_e, sin_e) = elevation(table);

    for (mut rig, playback) in &mut rigs {
        let rig = &mut *rig;
        let Some(ready) = rig.ready.as_ref() else {
            continue;
        };
        let Some((_, manifest)) = table.get(&rig.model) else {
            continue;
        };

        if let Ok(mut root) = roots.get_mut(rig.root) {
            root.rotation = Quat::from_rotation_y(playback.yaw);
        }
        if let Ok(mut player) = players.get_mut(ready.player) {
            apply(playback, &ready.clips, &mut player);
        }

        // L'arme du manifeste dans la main, du côté de la prise du clip en cours ; un objet porté (ni à la
        // main ni à l'os de l'arme : la clé à la ceinture) se montre dans les clips qui le nomment.
        let clip = playback.clip().unwrap_or("");
        let object = playback.object.as_deref();
        for (id, weapon) in &manifest.weapons {
            let side = (Some(id.as_str()) == object)
                .then(|| held_side(manifest, id, clip))
                .flatten();
            let worn = is_worn(weapon) && weapon.animations.iter().any(|a| a == clip);
            for (object_side, node) in &weapon.objects {
                if let Some(mut v) = ready
                    .nodes
                    .get(node)
                    .and_then(|e| visibility.get_mut(*e).ok())
                {
                    let wanted = if worn || Some(object_side.as_str()) == side {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                    v.set_if_neq(wanted);
                }
            }
        }

        // La bouche du canon (repère du côté tenu), mesurée sur la pose propagée à l'image précédente.
        rig.muzzle = object
            .zip(playback.muzzle_marker.as_deref())
            .and_then(|(object, marker)| {
                let side = held_side(manifest, object, clip)?;
                let node = manifest
                    .weapons
                    .get(object)?
                    .markers
                    .get(side)?
                    .get(marker)?;
                let p = globals.get(*ready.nodes.get(node)?).ok()?.translation();
                Some(Muzzle {
                    screen: Vec2::new(p.x, p.y * cos_e - p.z * sin_e) * u,
                    height: p.y * cos_e * u,
                })
            });
    }
}

/// Pose les pistes sur le lecteur d'animation : chacune à sa position et son poids, figée (la position
/// vient de l'état, pas du temps du lecteur) ; les autres clips s'arrêtent.
fn apply(
    playback: &ModelPlayback,
    clips: &BTreeMap<String, (bevy::animation::graph::AnimationNodeIndex, f32)>,
    player: &mut AnimationPlayer,
) {
    let tracks: Vec<(&Track, bevy::animation::graph::AnimationNodeIndex)> = playback
        .current
        .iter()
        .chain(playback.fading.iter())
        .filter_map(|t| Some((t, clips.get(&t.clip)?.0)))
        .collect();
    let stale: Vec<bevy::animation::graph::AnimationNodeIndex> = player
        .playing_animations()
        .map(|(node, _)| *node)
        .filter(|node| !tracks.iter().any(|(_, n)| n == node))
        .collect();
    for node in stale {
        player.stop(node);
    }
    for (track, node) in tracks {
        player
            .play(node)
            .set_weight(track.weight.max(1e-4))
            .seek_to(track.time)
            .pause();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_two_handed_clip_holds_the_weapon_by_its_bone() {
        let m: AnimeManifest = serde_json::from_str(
            r#"{"fps": 24, "actions": {"aim": [1, 9], "swing": [1, 9], "idle": [1, 9]},
                "armes": {"gun": {"objets": {"main": "gun_hand", "arme": "gun_bone"}, "animations": ["aim", "idle"]},
                          "axe": {"objets": {"main": "axe_hand"}, "animations": ["swing"]}},
                "prises": {"aim": "deux-mains", "idle": "une-main", "swing": "deux-mains"}}"#,
        )
        .unwrap();
        assert_eq!(held_side(&m, "gun", "aim"), Some("arme"));
        assert_eq!(held_side(&m, "gun", "idle"), Some("main"));
        assert_eq!(held_side(&m, "axe", "swing"), Some("main"));
        assert_eq!(held_side(&m, "gun", "swing"), None);
    }
}
