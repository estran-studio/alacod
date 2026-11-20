//! Attentes fines sur la présentation d'un personnage à modèle (T-3b, `docs/conventions.md` § 123) : `Clip`,
//! `ClipProgress`, `ClipSpeed`, `ModelYaw`, et `TelegraphAim` (le cône annoncé d'un émetteur suit sa cible).
//!
//! Elles lisent le composant de P-1d, [`ModelPlayback`] (décidé par `decide_poses`, aussi en headless), sans le réécrire,
//! et l'état de l'émetteur ; rien n'est écrit dans la simulation. Les angles sont comparés en `f32` : lecture entre deux
//! updates, jamais dans `GgrsSchedule`.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::emitter::Emitter;
use game::character::player::Player;
use game::model3d::playback::{yaw_from_screen, ModelPlayback};
use game::replay::{DistanceTarget, Expectation, YawTarget};
use sim_core::animation_state::FacingDirection;

/// Tolérance par défaut de `ModelYaw` : la même des 8 directions (demi-secteur).
pub const DEFAULT_YAW_TOLERANCE_DEG: f32 = 22.5;

/// Clip vu à une frame (pour `ClipSpeed`, mesurée d'une frame à la suivante).
#[derive(Clone, Debug)]
struct Seen {
    frame: u32,
    clip: String,
    time: f32,
    duration: f32,
}

/// Le clip de chaque personnage à modèle à la dernière frame relevée.
#[derive(Default)]
pub struct PlaybackHistory {
    last: BTreeMap<Entity, Seen>,
}

impl PlaybackHistory {
    /// Relève le clip joué par chaque personnage à modèle (après les attentes de la frame).
    pub fn record(&mut self, world: &mut World, frame: u32) {
        self.last.clear();
        for (entity, playback) in world.query::<(Entity, &ModelPlayback)>().iter(world) {
            if let (Some(clip), Some(time), Some(duration)) =
                (playback.clip(), playback.time(), playback.duration())
            {
                self.last.insert(
                    entity,
                    Seen {
                        frame,
                        clip: clip.to_string(),
                        time,
                        duration,
                    },
                );
            }
        }
    }
}

/// Entité du personnage désigné : un joueur par son handle, ou une entité rollback.
fn who_entity(world: &mut World, who: &DistanceTarget) -> Result<Entity, String> {
    match who {
        DistanceTarget::Player(handle) => world
            .query::<(Entity, &Player)>()
            .iter(world)
            .find(|(_, player)| player.handle == *handle)
            .map(|(entity, _)| entity)
            .ok_or_else(|| format!("joueur {handle} absent")),
        DistanceTarget::Entity(entity) => {
            crate::runner::resolve_entity(world, entity).ok_or_else(|| "entité absente".into())
        }
    }
}

fn playback<'w>(world: &'w World, entity: Entity) -> Result<&'w ModelPlayback, String> {
    world
        .get::<ModelPlayback>(entity)
        .ok_or_else(|| "personnage sans modèle (aucun ModelPlayback)".to_string())
}

/// Le clip joué, résolu (sinon la cause de P-1d, `unresolved`).
fn playing(playback: &ModelPlayback) -> Result<(&str, f32, f32), String> {
    if let Some(reason) = &playback.unresolved {
        return Err(format!("pose non résolue : {reason}"));
    }
    match (playback.clip(), playback.time(), playback.duration()) {
        (Some(clip), Some(time), Some(duration)) => Ok((clip, time, duration)),
        _ => Err("aucun clip joué".into()),
    }
}

fn position(world: &World, entity: Entity) -> Option<Vec2> {
    world.get::<FixedTransform3D>(entity).map(|t| {
        Vec2::new(
            t.translation.x.to_num::<f32>(),
            t.translation.y.to_num::<f32>(),
        )
    })
}

fn player_position(world: &mut World, handle: usize) -> Result<Vec2, String> {
    let entity = who_entity(world, &DistanceTarget::Player(handle))?;
    position(world, entity).ok_or_else(|| format!("joueur {handle} sans position"))
}

/// Direction de l'écran (y vers le haut) d'une des 8 directions.
fn direction_vector(direction: FacingDirection) -> Vec2 {
    let degrees: f32 = match direction {
        FacingDirection::Right => 0.0,
        FacingDirection::UpRight => 45.0,
        FacingDirection::Up => 90.0,
        FacingDirection::UpLeft => 135.0,
        FacingDirection::Left => 180.0,
        FacingDirection::DownLeft => 225.0,
        FacingDirection::Down => 270.0,
        FacingDirection::DownRight => 315.0,
    };
    Vec2::from_angle(degrees.to_radians())
}

/// Écart d'angle ramené dans `[-π, π]`.
fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn bounds(name: &str, value: f32, min: Option<f32>, max: Option<f32>) -> Result<(), String> {
    if let Some(min) = min {
        if value < min {
            return Err(format!("{name} {value:.3} < min {min}"));
        }
    }
    if let Some(max) = max {
        if value > max {
            return Err(format!("{name} {value:.3} > max {max}"));
        }
    }
    Ok(())
}

/// `Clip`, `ClipProgress`, `ModelYaw`, `TelegraphAim` (une frame suffit). `ClipSpeed` passe par [`clip_speed`].
pub fn check(world: &mut World, expectation: &Expectation) -> Result<(), String> {
    match expectation {
        Expectation::Clip {
            who, clip, once, ..
        } => {
            let entity = who_entity(world, who)?;
            let playback = playback(world, entity)?;
            let (playing_clip, time, duration) = playing(playback)?;
            if playing_clip != clip {
                return Err(format!(
                    "clip joué `{playing_clip}` ({time:.3} / {duration:.3} s), `{clip}` attendu"
                ));
            }
            match once {
                Some(true) if playback.looping() => {
                    Err(format!("`{clip}` boucle, joué une fois attendu"))
                }
                Some(false) if !playback.looping() => {
                    Err(format!("`{clip}` joué une fois, une boucle attendue"))
                }
                _ => Ok(()),
            }
        }
        Expectation::ClipProgress { who, min, max, .. } => {
            let entity = who_entity(world, who)?;
            let (clip, time, duration) = playing(playback(world, entity)?)?;
            bounds(
                &format!("progression de `{clip}`"),
                time / duration.max(1e-6),
                *min,
                *max,
            )
        }
        Expectation::ModelYaw {
            who,
            toward,
            tolerance_deg,
            ..
        } => {
            let entity = who_entity(world, who)?;
            let from = position(world, entity).ok_or("personnage sans position")?;
            let direction = match toward {
                YawTarget::Direction(direction) => direction_vector(*direction),
                YawTarget::Player(handle) => player_position(world, *handle)? - from,
            };
            let sin_e = world
                .get_resource::<content::Registry>()
                .and_then(|registry| registry.models.as_ref())
                .map(|table| game::model3d::elevation(table).1)
                .ok_or("aucune table de modèles")?;
            let expected =
                yaw_from_screen(direction, sin_e).ok_or("direction nulle (cible confondue)")?;
            let yaw = playback(world, entity)?.yaw;
            let gap = wrap(yaw - expected).abs().to_degrees();
            let tolerance = tolerance_deg.unwrap_or(DEFAULT_YAW_TOLERANCE_DEG);
            if gap <= tolerance {
                Ok(())
            } else {
                Err(format!(
                    "cap du corps {:.1}°, {:.1}° attendu (écart {gap:.1}° > {tolerance}°)",
                    yaw.to_degrees(),
                    expected.to_degrees()
                ))
            }
        }
        Expectation::TelegraphAim {
            entity,
            handle,
            max_deg,
            ..
        } => {
            let shooter = crate::runner::resolve_entity(world, entity).ok_or("entité absente")?;
            let from = position(world, shooter).ok_or("entité sans position")?;
            let to = player_position(world, *handle)?;
            let emitter = world
                .get::<Emitter>(shooter)
                .ok_or("aucun émetteur (pas de salve en cours)")?;
            if emitter.telegraphing().is_none() {
                return Err("l'émetteur n'est pas dans un Telegraph".into());
            }
            let aim = Vec2::new(emitter.aim.x.to_num::<f32>(), emitter.aim.y.to_num::<f32>());
            let toward = to - from;
            if aim.length_squared() < 1e-8 || toward.length_squared() < 1e-8 {
                return Err("visée ou direction nulle".into());
            }
            let gap = aim.angle_to(toward).abs().to_degrees();
            if gap <= *max_deg {
                Ok(())
            } else {
                Err(format!(
                    "le cône vise à {gap:.1}° du joueur {handle} (> {max_deg}°)"
                ))
            }
        }
        _ => Err("attente hors du module des modèles".into()),
    }
}

/// `ClipSpeed` : secondes de clip par seconde de jeu entre la frame relevée (`at_frame - 1`) et celle-ci.
pub fn clip_speed(
    world: &mut World,
    expectation: &Expectation,
    history: &PlaybackHistory,
    frame: u32,
) -> Result<(), String> {
    let Expectation::ClipSpeed { who, min, max, .. } = expectation else {
        return Err("attente ClipSpeed attendue".into());
    };
    let entity = who_entity(world, who)?;
    let (clip, time, duration) = playing(playback(world, entity)?)?;
    let before = history
        .last
        .get(&entity)
        .filter(|seen| seen.frame + 1 == frame)
        .ok_or("aucun clip relevé à la frame précédente")?;
    if before.clip != clip {
        return Err(format!(
            "le clip change (`{}` → `{clip}`) : vitesse non mesurable",
            before.clip
        ));
    }
    let mut advanced = time - before.time;
    // Un tour de boucle entre les deux frames.
    if advanced < 0.0 && before.duration > 0.0 {
        advanced += duration;
    }
    let speed = advanced * game::model3d::playback::SIM_HZ;
    bounds(&format!("vitesse de `{clip}`"), speed, *min, *max)
}
