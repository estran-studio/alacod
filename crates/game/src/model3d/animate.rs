//! Animation d'un modèle d'après l'état de la simulation (R-1, `docs/conventions.md` § 111) : lecture
//! de l'état (déjà rollback, jamais écrit), choix du clip ([`super::pose`]), fondu avec le clip
//! précédent, cap du corps, arme du manifeste dans la main, bouche du canon.
//!
//! Le cap suit la visée (rotation de l'arme active, comme le sprite de l'arme), la direction de
//! l'esquive pendant l'esquive, sinon la marche, sinon `FacingDirection`. Les gestes sont lus à la
//! position que donne l'état de la simulation (avancement du coup, du rechargement, temps depuis le
//! tir) : un rollback qui corrige l'état corrige la pose à l'image suivante.

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::{
    actors::DashState,
    weapons::{melee::MeleeAttackState, melee::MeleeWeapon, Weapon, WeaponInventory, WeaponState},
};
use content::{
    models::{AnimeManifest, HAND_SIDE, TWO_HANDED_GRIP, WEAPON_SIDE},
    Registry,
};
use sim_core::animation_state::FacingDirection;
use utils::frame::FrameCount;

use super::{
    elevation,
    pose::{self, PoseChoice, PoseInput, Timing},
    ModelRig,
};

/// Fréquence de la simulation : les frames lues dans l'état deviennent des secondes.
const SIM_HZ: f32 = 60.0;

/// État de présentation d'un modèle : pistes en cours, mémoire des entrées.
#[derive(Default)]
pub struct RigState {
    current: Option<Track>,
    fading: Vec<Track>,
    fade: f32,
    last: Option<(Vec2, u32)>,
    /// Vitesse au sol (unités du monde par seconde), d'une frame simulée à l'autre.
    velocity: Vec2,
    trigger_from: Option<u32>,
    dashing: bool,
    dashes: u32,
    /// Cap du corps (radians autour de la verticale, 0 : face à la caméra).
    pub yaw: f32,
    /// La bouche du canon à l'écran, mesurée à l'image précédente.
    pub muzzle: Option<Muzzle>,
}

/// Bouche du canon d'un modèle : décalage à l'écran depuis le pied (unités du monde) et la part de ce
/// décalage due à sa hauteur au-dessus du sol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Muzzle {
    pub screen: Vec2,
    pub height: f32,
}

#[derive(Clone, Debug)]
struct Track {
    clip: String,
    node: AnimationNodeIndex,
    duration: f32,
    /// Position dans le clip (s).
    time: f32,
    weight: f32,
}

/// Cap 3D (radians, autour de +Y, 0 vers +Z) d'une direction de l'écran `(dx, dy)` (y vers le haut) :
/// la profondeur du sol est l'écran divisé par `sin e`.
pub fn yaw_from_screen(d: Vec2, sin_e: f32) -> Option<f32> {
    (d.length_squared() > 1e-8).then(|| d.x.atan2(-d.y / sin_e))
}

/// Une direction de l'écran vue au sol, en mètres : `(x, profondeur vers la caméra)`.
fn ground(d: Vec2, units_per_meter: f32, sin_e: f32) -> Vec2 {
    Vec2::new(d.x / units_per_meter, -d.y / (units_per_meter * sin_e))
}

fn facing_vector(facing: &FacingDirection) -> Vec2 {
    let angle = match facing {
        FacingDirection::Right => 0.0,
        FacingDirection::UpRight => 45.0,
        FacingDirection::Up => 90.0,
        FacingDirection::UpLeft => 135.0,
        FacingDirection::Left => 180.0,
        FacingDirection::DownLeft => 225.0,
        FacingDirection::Down => 270.0,
        FacingDirection::DownRight => 315.0,
    } as f32;
    Vec2::from_angle(angle.to_radians())
}

fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
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
    time: Res<Time>,
    frame: Option<Res<FrameCount>>,
    registry: Option<Res<Registry>>,
    mut rigs: Query<(
        &mut ModelRig,
        &FixedTransform3D,
        Option<&DashState>,
        Option<&WeaponInventory>,
        Option<&MeleeAttackState>,
        Option<&FacingDirection>,
        Option<&Children>,
    )>,
    weapons: Query<(&Weapon, &WeaponState, &Transform)>,
    melee: Query<&MeleeWeapon>,
    mut roots: Query<&mut Transform, (Without<Weapon>, Without<ModelRig>)>,
    mut players: Query<&mut AnimationPlayer>,
    mut visibility: Query<&mut Visibility>,
    globals: Query<&GlobalTransform>,
) {
    let Some(table) = registry.as_ref().and_then(|r| r.models.as_ref()) else {
        return;
    };
    let f = frame.map_or(0, |frame| frame.frame);
    let dt = time.delta_secs();
    let u = table.def.units_per_meter;
    let (cos_e, sin_e) = elevation(table);

    for (mut rig, transform, dash, inventory, melee_state, facing, children) in &mut rigs {
        let rig = &mut *rig;
        let Some(ready) = rig.ready.as_ref() else {
            continue;
        };
        let Some((model, manifest)) = table.get(&rig.model) else {
            continue;
        };
        let state = &mut rig.state;

        // Vitesse : déplacement d'une frame simulée à l'autre (la simulation est en `Fixed`, l'affichage
        // n'en lit qu'une copie).
        let position = Vec2::new(
            transform.translation.x.to_num::<f32>(),
            transform.translation.y.to_num::<f32>(),
        );
        match state.last {
            Some((p0, f0)) if f > f0 && f - f0 <= 10 => {
                state.velocity = (position - p0) * SIM_HZ / (f - f0) as f32;
                state.last = Some((position, f));
            }
            Some((_, f0)) if f == f0 => {}
            _ => {
                state.velocity = Vec2::ZERO;
                state.last = Some((position, f));
            }
        }

        // Arme active : poses, visée, tir, rechargement.
        let active = inventory
            .and_then(|inv| inv.weapons.get(inv.active_weapon_index))
            .and_then(|(entity, _)| weapons.get(*entity).ok());
        let aim = active.map(|(_, _, t)| (t.rotation * Vec3::X).truncate());

        // Cap : l'esquive, la visée, la marche, l'orientation de la simulation.
        let dashing = dash.filter(|d| d.is_dashing);
        let heading = dashing
            .map(|d| Vec2::new(d.dash_direction.x.to_num(), d.dash_direction.y.to_num()))
            .or(aim)
            .filter(|d| d.length_squared() > 1e-8)
            .or_else(|| (state.velocity.length() > 1.0).then_some(state.velocity))
            .or_else(|| facing.map(facing_vector));
        if let Some(yaw) = heading.and_then(|d| yaw_from_screen(d, sin_e)) {
            state.yaw = yaw;
        }

        let walk = ground(state.velocity, u, sin_e);
        let mut input = PoseInput {
            speed: walk.length(),
            rel: if walk.length_squared() > 1e-8 {
                wrap(walk.x.atan2(walk.y) - state.yaw)
            } else {
                0.0
            },
            ..default()
        };
        match dashing {
            Some(d) => {
                if !state.dashing {
                    state.dashes = state.dashes.wrapping_add(1);
                }
                let total = d.dash_duration.max(1) as f32;
                let done = d.dash_duration.saturating_sub(d.dash_frames_remaining) as f32;
                input.dash = Some((state.dashes, done / total));
            }
            None => {}
        }
        state.dashing = dashing.is_some();
        if let Some(attack) = melee_state.filter(|m| m.is_attacking) {
            let started = attack.attack_started_frame.unwrap_or(f);
            let frames = children
                .into_iter()
                .flatten()
                .find_map(|c| melee.get(*c).ok())
                .map_or(20, |w| w.config.attack_duration_frames.max(1));
            input.melee = Some((
                f.saturating_sub(started) as f32 / SIM_HZ,
                frames as f32 / SIM_HZ,
            ));
        }
        if let Some((weapon, weapon_state, _)) = active {
            if weapon_state.last_fire_frame > 0 && f >= weapon_state.last_fire_frame {
                input.since_shot = Some((f - weapon_state.last_fire_frame) as f32 / SIM_HZ);
            }
            input.trigger = weapon_state.is_firing;
            if weapon_state.is_firing {
                let from = *state.trigger_from.get_or_insert(f);
                input.trigger_for = f.saturating_sub(from) as f32 / SIM_HZ;
            } else {
                state.trigger_from = None;
            }
            if let Some(end) = inventory.and_then(|inv| inv.reloading_ending_frame) {
                let seconds = weapon
                    .config
                    .firing_modes
                    .get(&weapon_state.active_mode)
                    .map_or(1.0, |mode| mode.reload_time_seconds.to_num::<f32>());
                let frames = (seconds * SIM_HZ).ceil().max(1.0);
                if end > f {
                    let left = (end - f) as f32;
                    input.reload = Some(((1.0 - left / frames).clamp(0.0, 1.0), seconds));
                }
            }
        } else {
            state.trigger_from = None;
        }

        let weapon_poses = active.and_then(|(w, _, _)| model.weapons.get(&w.config.name));
        let poses = match weapon_poses {
            Some(p) => p.over(&model.poses),
            None => model.poses.clone(),
        };
        let choice = pose::choose(model, &poses, manifest, &input);
        if let Some(choice) = &choice {
            step(state, choice, &ready.clips, dt);
        }

        if let Ok(mut root) = roots.get_mut(rig.root) {
            root.rotation = Quat::from_rotation_y(state.yaw);
        }
        if let Ok(mut player) = players.get_mut(ready.player) {
            apply(state, &mut player);
        }

        // L'arme du manifeste dans la main, du côté de la prise du clip en cours ; un objet porté (ni à la
        // main ni à l'os de l'arme : la clé à la ceinture) se montre dans les clips qui le nomment.
        let clip = state.current.as_ref().map(|t| t.clip.as_str()).unwrap_or("");
        let object = choice.as_ref().and_then(|c| c.object.as_deref());
        for (id, weapon) in &manifest.weapons {
            let side = (Some(id.as_str()) == object)
                .then(|| held_side(manifest, id, clip))
                .flatten();
            let worn = is_worn(weapon) && weapon.animations.iter().any(|a| a == clip);
            for (object_side, node) in &weapon.objects {
                if let Some(mut v) = ready.nodes.get(node).and_then(|e| visibility.get_mut(*e).ok())
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
        state.muzzle = object
            .zip(poses.muzzle.as_deref())
            .and_then(|(object, marker)| {
                let side = held_side(manifest, object, clip)?;
                let node = manifest.weapons.get(object)?.markers.get(side)?.get(marker)?;
                let p = globals.get(*ready.nodes.get(node)?).ok()?.translation();
                Some(Muzzle {
                    screen: Vec2::new(p.x, p.y * cos_e - p.z * sin_e) * u,
                    height: p.y * cos_e * u,
                })
            });
    }
}

/// Avance les pistes : un nouveau clip entre en fondu (à la même phase si c'est une allure), les
/// autres sortent.
fn step(
    state: &mut RigState,
    choice: &PoseChoice,
    clips: &HashMap<String, (AnimationNodeIndex, f32)>,
    dt: f32,
) {
    let Some(&(node, duration)) = clips.get(&choice.clip) else {
        return;
    };
    if state.current.as_ref().map(|t| &t.clip) != Some(&choice.clip) {
        let phase = state
            .current
            .as_ref()
            .map(|t| t.time / t.duration)
            .unwrap_or(0.0);
        let resumed = state
            .fading
            .iter()
            .position(|t| t.clip == choice.clip)
            .map(|i| state.fading.remove(i));
        let mut track = resumed.unwrap_or(Track {
            clip: choice.clip.clone(),
            node,
            duration,
            time: 0.0,
            weight: 0.0,
        });
        if matches!(choice.timing, Timing::Loop(_)) && track.weight == 0.0 {
            track.time = phase.fract() * duration;
        }
        if let Some(old) = state.current.replace(track) {
            state.fading.push(old);
        }
        state.fade = choice.fade.max(1e-3);
    }
    let fade = state.fade;
    if let Some(track) = state.current.as_mut() {
        match choice.timing {
            Timing::Loop(rate) => {
                track.time = (track.time + dt * rate).rem_euclid(track.duration);
            }
            Timing::At(u) => track.time = u * track.duration,
        }
        track.weight = (track.weight + dt / fade).min(1.0);
    }
    for track in &mut state.fading {
        track.weight -= dt / fade;
    }
    state.fading.retain(|t| t.weight > 0.0);
}

/// Pose les pistes sur le lecteur d'animation : chacune à sa position et son poids, figée (la position
/// vient de l'état, pas du temps du lecteur) ; les autres clips s'arrêtent.
fn apply(state: &RigState, player: &mut AnimationPlayer) {
    let tracks: Vec<&Track> = state.current.iter().chain(state.fading.iter()).collect();
    let stale: Vec<AnimationNodeIndex> = player
        .playing_animations()
        .map(|(node, _)| *node)
        .filter(|node| !tracks.iter().any(|t| t.node == *node))
        .collect();
    for node in stale {
        player.stop(node);
    }
    for track in tracks {
        player
            .play(track.node)
            .set_weight(track.weight.max(1e-4))
            .seek_to(track.time)
            .pause();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screen_heading_maps_to_the_ground_yaw() {
        let s = std::f32::consts::FRAC_1_SQRT_2;
        // Vers le bas de l'écran : face à la caméra ; vers le haut : de dos ; à droite : +X.
        assert!(yaw_from_screen(Vec2::new(0.0, -1.0), s).unwrap().abs() < 1e-6);
        assert!((yaw_from_screen(Vec2::new(0.0, 1.0), s).unwrap().abs() - std::f32::consts::PI).abs() < 1e-6);
        assert!((yaw_from_screen(Vec2::new(1.0, 0.0), s).unwrap() - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // Une diagonale de l'écran est plus profonde au sol : le cap s'ouvre vers la profondeur.
        let diag = yaw_from_screen(Vec2::new(1.0, -1.0), s).unwrap();
        assert!((diag - (1.0f32).atan2(1.0 / s)).abs() < 1e-6);
        assert_eq!(yaw_from_screen(Vec2::ZERO, s), None);
    }

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
