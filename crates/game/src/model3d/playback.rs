//! Décision de pose d'un personnage à modèle, **hors rendu** (P-1d, `docs/conventions.md` § 122).
//!
//! `decide_poses` lit l'état de la simulation de chaque personnage dont le `asset_name_ref` a une entrée dans la table
//! `Model`, choisit son clip ([`super::pose::choose`]), le fond avec le précédent et avance sa position, puis range le
//! résultat dans le composant [`ModelPlayback`]. Il tourne aussi en headless (aucune image, caméra ni GLB : le registre et
//! les manifestes `anime.json` suffisent) : le runner de scénarios y lit **quel clip est joué, à quelle position**, et
//! l'invariant `animation_vivante` échoue quand un personnage dont l'état de la simulation change garde un clip figé
//! ou celui de l'attente. `animate_models` ne fait plus qu'appliquer ce composant au lecteur d'animation et aux objets
//! du modèle (rendu seul).
//!
//! Rien ici n'est rollbacké ni écrit dans la simulation.

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::{
    actors::{CursorPosition, DashState},
    emitter::Emitter,
    weapons::{melee::MeleeAttackState, melee::MeleeWeapon, Weapon, WeaponInventory, WeaponState},
};
use content::Registry;
use sim_core::animation_state::FacingDirection;
use utils::frame::FrameCount;

use super::{
    elevation,
    pose::{self, Fades, PoseChoice, PoseInput, Timing},
    shooter,
};
use crate::character::config::{CharacterConfigKey, CharacterConfigs};
use crate::character::enemy::ai::{cycle::RangedCycle, state::RangedAttackState, EnemyAiConfig};

/// Fréquence de la simulation : les frames lues dans l'état deviennent des secondes.
pub const SIM_HZ: f32 = 60.0;

/// D'où vient le temps d'avancement des clips.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackClock {
    /// Le temps de l'affichage (fenêtre) : les clips avancent à chaque image dessinée.
    Display,
    /// Une frame simulée = 1/60 s (headless : pas de fenêtre, le temps réel ne dit rien du jeu).
    Simulated,
}

/// Présentation d'un personnage à modèle : quel clip joue, à quelle position, depuis quelles entrées
/// (présentation, hors rollback, écrite aussi en headless).
#[derive(Component, Debug, Clone)]
pub struct ModelPlayback {
    /// Id du modèle (`asset_name_ref` du personnage).
    pub model: String,
    pub(super) current: Option<Track>,
    pub(super) fading: Vec<Track>,
    fade: f32,
    last: Option<(Vec2, u32)>,
    /// Vitesse au sol (unités du monde par seconde), d'une frame simulée à l'autre.
    velocity: Vec2,
    trigger_from: Option<u32>,
    dashing: bool,
    dashes: u32,
    last_frame: Option<u32>,
    /// Cap du corps (radians autour de la verticale, 0 : face à la caméra).
    pub yaw: f32,
    /// Arme du manifeste montrée dans la main (celle de la pose choisie).
    pub object: Option<String>,
    /// Repère de la bouche du canon de la pose choisie.
    pub muzzle_marker: Option<String>,
    /// Pourquoi aucun clip ne joue : pose déclarée absente du manifeste, ou aucune pose ne convient.
    pub unresolved: Option<String>,
    /// La dernière image d'un geste joué une fois est **tenue exprès** tant que l'état de la simulation dure (le tir d'un
    /// tireur posté jusqu'à la fin de sa séquence) : sa position ne bouge plus, sans que le lecteur soit bloqué.
    pub held: bool,
    /// Le personnage marche selon la pose choisie (vitesse au sol ≥ `pose::STILL_BELOW`) : la même mesure que la
    /// sélection du clip, lue par l'invariant `animation_vivante` au lieu d'un second seuil.
    pub moving: bool,
}

#[derive(Clone, Debug)]
pub(super) struct Track {
    pub(super) clip: String,
    pub(super) duration: f32,
    /// Position dans le clip (s).
    pub(super) time: f32,
    pub(super) weight: f32,
    /// Clip joué en boucle (attente, allure, en joue) ; un geste joué une fois (`Timing::At`) ne l'est pas.
    pub(super) looping: bool,
}

impl ModelPlayback {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            current: None,
            fading: Vec::new(),
            fade: 0.0,
            last: None,
            velocity: Vec2::ZERO,
            trigger_from: None,
            dashing: false,
            dashes: 0,
            last_frame: None,
            yaw: 0.0,
            object: None,
            muzzle_marker: None,
            unresolved: None,
            held: false,
            moving: false,
        }
    }

    /// Une pose imposée (clip, durée et position en s) : pour éprouver l'invariant `animation_vivante` sans jeu ni
    /// registre ; le jeu, lui, ne passe que par [`decide_poses`].
    pub fn playing(model: &str, clip: &str, duration: f32, time: f32) -> Self {
        let mut playback = Self::new(model);
        playback.current = Some(Track {
            clip: clip.to_string(),
            duration,
            time,
            weight: 1.0,
            looping: true,
        });
        playback
    }

    /// Déplace la position du clip joué (voir [`Self::playing`]).
    pub fn set_time(&mut self, time: f32) {
        if let Some(track) = self.current.as_mut() {
            track.time = time;
        }
    }

    /// Le clip joué (celui vers lequel le fondu va).
    pub fn clip(&self) -> Option<&str> {
        self.current.as_ref().map(|t| t.clip.as_str())
    }

    /// Position dans le clip joué (s).
    pub fn time(&self) -> Option<f32> {
        self.current.as_ref().map(|t| t.time)
    }

    /// Durée du clip joué (s).
    pub fn duration(&self) -> Option<f32> {
        self.current.as_ref().map(|t| t.duration)
    }

    /// Le clip joué boucle-t-il (attente, allure, en joue) ? Un geste joué une fois (tir, épaulé, rechargement) : non.
    pub fn looping(&self) -> bool {
        self.current.as_ref().is_some_and(|t| t.looping)
    }
}

/// Pose un [`ModelPlayback`] sur chaque personnage dont le modèle est déclaré (une fois, à sa création).
pub fn attach_playback(
    mut commands: Commands,
    registry: Option<Res<Registry>>,
    configs: Option<Res<CharacterConfigs>>,
    created: Query<(Entity, &CharacterConfigKey), Added<CharacterConfigKey>>,
) {
    let (Some(table), Some(configs)) = (registry.as_ref().and_then(|r| r.models.as_ref()), configs)
    else {
        return;
    };
    for (entity, key) in &created {
        let Some(config) = configs.get(&key.0) else {
            continue;
        };
        if table.get(&config.asset_name_ref).is_some() {
            commands
                .entity(entity)
                .try_insert(ModelPlayback::new(&config.asset_name_ref));
        }
    }
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

/// Direction (écran) vers laquelle le corps se tourne. Un personnage à curseur (le joueur) : l'esquive, la
/// visée si le curseur vise, la marche, puis le regard. Un autre (ennemi) : l'esquive, puis le regard de la
/// simulation (`FacingDirection`, qui suit sa cible), puis la marche : il ne tourne pas le dos à sa cible
/// quand il recule ou revient à son poste (banc `armes.js` A:686 : `vers` le joueur, hors « touché »).
pub fn body_heading(
    dash: Option<Vec2>,
    aim: Option<Vec2>,
    velocity: Vec2,
    facing: Option<Vec2>,
    has_cursor: bool,
) -> Option<Vec2> {
    let usable = |d: &Vec2| d.length_squared() > 1e-8;
    let walking = (velocity.length() > 1.0).then_some(velocity);
    let ordered = if has_cursor {
        [dash, aim, walking, facing]
    } else {
        [dash, facing, walking, None]
    };
    ordered.into_iter().flatten().find(usable)
}

fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// Ce que `decide_poses` lit d'un tireur posté (fantassin) : séquence de tir, refroidissement, cycle.
type Posted<'a> = (
    Option<&'a Emitter>,
    Option<&'a RangedAttackState>,
    Option<&'a EnemyAiConfig>,
    Option<&'a RangedCycle>,
);

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn decide_poses(
    time: Res<Time>,
    clock: Option<Res<PlaybackClock>>,
    frame: Option<Res<FrameCount>>,
    registry: Option<Res<Registry>>,
    mut characters: Query<(
        &mut ModelPlayback,
        &FixedTransform3D,
        Option<&DashState>,
        Option<&WeaponInventory>,
        Option<&MeleeAttackState>,
        Option<&FacingDirection>,
        Option<&Children>,
        // Visée du joueur (absente des ennemis : leur regard vient de `FacingDirection`).
        Option<&CursorPosition>,
        // Tireur posté (P-1d) : son geste vient de sa séquence d'émetteur et de son refroidissement.
        Posted<'_>,
    )>,
    weapons: Query<(&Weapon, &WeaponState, &Transform)>,
    melee: Query<&MeleeWeapon>,
) {
    let Some(table) = registry.as_ref().and_then(|r| r.models.as_ref()) else {
        return;
    };
    let f = frame.map_or(0, |frame| frame.frame);
    let simulated = clock.is_some_and(|c| *c == PlaybackClock::Simulated);
    let u = table.def.units_per_meter;
    let (_, sin_e) = elevation(table);

    for (mut playback, transform, dash, inventory, melee_state, facing, children, cursor, posted) in
        &mut characters
    {
        let state = &mut *playback;
        let Some((model, manifest)) = table.get(&state.model) else {
            continue;
        };
        // Le temps des clips : l'image dessinée (fenêtre) ou la frame simulée (headless, 1/60 s ; jamais en arrière).
        let dt = if simulated {
            match state.last_frame {
                Some(last) if f > last => (f - last) as f32 / SIM_HZ,
                Some(_) => 0.0,
                None => 0.0,
            }
        } else {
            time.delta_secs()
        };
        state.last_frame = Some(match state.last_frame {
            Some(last) if simulated => last.max(f),
            _ => f,
        });

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

        // Un tireur posté (fantassin) n'a ni détente ni chargeur : ses gestes se lisent dans sa séquence et
        // son refroidissement (P-1d, `shooter::read`).
        let shot = match posted {
            (emitter, Some(ranged), Some(ai), cycle) => ai.ranged.as_ref().map(|r| {
                let stagger = ai.ranged_cycle.as_ref().and_then(|c| c.stagger_frames);
                shooter::read(
                    f,
                    emitter,
                    ranged,
                    r.cooldown_frames,
                    cycle.map(|c| (c, stagger)),
                )
            }),
            _ => None,
        };

        // Cap : l'esquive, la visée, la marche, l'orientation de la simulation. Un personnage sans curseur
        // (l'ennemi) tourne vers sa cible, sauf sous le coup (banc `armes.js` A:686 : « touché » garde le cap),
        // arrondi aux directions du corps du modèle (`arrondir`, 8 par défaut) ; le cap du joueur reste exact
        // tant que sa revue (tranche suivante de P-1d) ne l'a pas observé : le Frère est validé tel quel.
        let dashing = dash.filter(|d| d.is_dashing);
        let dash_heading =
            dashing.map(|d| Vec2::new(d.dash_direction.x.to_num(), d.dash_direction.y.to_num()));
        let heading = body_heading(
            dash_heading,
            aim,
            state.velocity,
            facing.map(facing_vector),
            cursor.is_some(),
        );
        let stunned = shot.is_some_and(|s| s.hit.is_some());
        if let Some(yaw) = heading.and_then(|d| yaw_from_screen(d, sin_e)) {
            if cursor.is_some() {
                state.yaw = yaw;
            } else if !stunned {
                state.yaw = model.body_directions.round(yaw);
            }
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
        state.moving = input.speed >= pose::STILL_BELOW;
        if let Some(d) = dashing {
            if !state.dashing {
                state.dashes = state.dashes.wrapping_add(1);
            }
            let total = d.dash_duration.max(1) as f32;
            let done = d.dash_duration.saturating_sub(d.dash_frames_remaining) as f32;
            // Rang de l'esquive (0 pour la première) : la première joue `dash[0]`, comme le banc
            // (`armes.js` : `J.pas` part de −1 ; P-1e, § 125 : le Frère commençait sur `gigue-02`).
            input.dash = Some((state.dashes.wrapping_sub(1), done / total));
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
            // P-1 (D199) : l'épaulé de la simulation (`WeaponGesture::raise_frames`) joue la pose
            // `shoulder` même si la détente est déjà relâchée.
            let held = weapon_state.is_firing || weapon_state.raise_ends.is_some();
            input.trigger = held;
            if held {
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
        if let Some(read) = shot {
            input.fades = Fades::SHOOTER;
            input.hit = read.hit.map(|(done, total)| done as f32 / total as f32);
            input.trigger = read.raising_for.is_some();
            input.trigger_for = read.raising_for.map_or(0.0, |n| n as f32 / SIM_HZ);
            input.since_shot = read.since_shot.map(|n| n as f32 / SIM_HZ);
            input.reload = read
                .reload
                .map(|(done, total)| (done as f32 / total as f32, total as f32 / SIM_HZ));
        }

        // Les poses de l'arme en main (`models.ron`, `weapons`) : l'arme active d'un joueur, ou l'arme de la règle `Shoot`
        // d'un tireur posté ; le lint `MissingPose` suit la même résolution.
        let weapon_name = active.map(|(w, _, _)| w.config.name.as_str()).or_else(|| {
            shot.and(posted.2)
                .and_then(|ai| ai.ranged.as_ref())
                .map(|r| r.weapon.as_str())
        });
        let mut poses = match weapon_name.and_then(|name| model.weapons.get(name)) {
            Some(p) => p.over(&model.poses),
            None => model.poses.clone(),
        };
        // P-1e (§ 125) : une arme à gestes (`WeaponConfig::gesture`) étire ses clips d'épaulé et de tir sur la durée de
        // ses gestes, comme le banc (`armes.js` `lancer` : clip lu à `durée du clip / durée du geste`) ; lus à la durée
        // du clip, le tir du Frère durait 34 frames au lieu de 18, cachait le début du rechargement (entré à 23 %), et
        // l'épaulé n'en montrait que 36 %. Une `duration_s` écrite dans `models.ron` reste prioritaire.
        if let Some(gesture) = active.and_then(|(w, _, _)| w.config.gesture.as_ref()) {
            for (pose, frames) in [
                (&mut poses.shoulder, gesture.raise_frames),
                (&mut poses.shoot, gesture.fire_frames),
            ] {
                if let Some(g) = pose
                    .as_mut()
                    .filter(|g| g.duration_s.is_none() && frames > 0)
                {
                    g.duration_s = Some(frames as f32 / SIM_HZ);
                }
            }
            // La simulation lance le rechargement à la fin du geste de tir, mais il n'est visible ici qu'à la frame
            // suivante : le tir tient sa dernière image cette frame-là au lieu de montrer l'attente (au banc, `tir`
            // enchaîne sur `rechargement` sans image entre les deux).
            let total = gesture.fire_frames as f32 / SIM_HZ;
            let gap = input
                .since_shot
                .is_some_and(|s| s >= total && s < total + 1.5 / SIM_HZ);
            if gesture.fire_frames > 0 && input.reload.is_none() && gap {
                input.since_shot = Some(total * 0.999);
            }
        }
        // Le geste du tir d'un tireur posté tient sa dernière image tant que sa séquence dure : une attente d'après tir
        // plus longue que le clip (`Wait(35)` pour un clip de 32,5 frames) ne doit pas montrer l'attente entre le tir et
        // le rechargement (banc `armes.js` A:694 : `tir` enchaîne sur `recharge`). `held` le dit à l'invariant.
        state.held = false;
        if let (Some(since), true) = (
            input.since_shot,
            shot.is_some_and(|s| s.since_shot.is_some()),
        ) {
            let total = poses.shoot.as_ref().and_then(|g| {
                g.duration_s
                    .or_else(|| g.still.as_deref().and_then(|c| manifest.duration(c)))
            });
            if let Some(total) = total {
                state.held = since >= total * 0.999;
                input.since_shot = Some(since.min(total * 0.999));
            }
        }
        let choice = pose::choose(model, &poses, manifest, &input);
        state.object = choice.as_ref().and_then(|c| c.object.clone());
        state.muzzle_marker = poses.muzzle.clone();
        match &choice {
            Some(choice) => match manifest.duration(&choice.clip) {
                Some(duration) => {
                    state.unresolved = None;
                    step(state, choice, duration, dt);
                }
                None => {
                    state.unresolved = Some(format!(
                        "le clip `{}` de la pose n'est pas dans le manifeste du modèle `{}`",
                        choice.clip, state.model
                    ));
                }
            },
            None => {
                state.unresolved = Some(format!(
                    "aucune pose du modèle `{}` ne convient à l'état du personnage",
                    state.model
                ));
            }
        }
    }
}

/// Avance les pistes : un nouveau clip entre en fondu (à la même phase si c'est une allure), les
/// autres sortent.
fn step(state: &mut ModelPlayback, choice: &PoseChoice, duration: f32, dt: f32) {
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
            duration,
            time: 0.0,
            weight: 0.0,
            looping: false,
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
        track.looping = matches!(choice.timing, Timing::Loop(_));
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

/// Ce que fait la simulation d'un personnage à modèle, lu **sur ses composants de simulation** (pas sur les entrées de la
/// pose) : l'invariant `animation_vivante` s'en sert pour demander qu'un état actif ne joue pas le clip de l'attente.
/// `None` : au repos. Les gestes de plus d'une frame seulement (un tir d'une frame ne compte pas).
pub fn sim_activity(world: &World, entity: Entity, frame: u32) -> Option<&'static str> {
    let entity = world.get_entity(entity).ok()?;
    if entity.get::<DashState>().is_some_and(|d| d.is_dashing) {
        return Some("esquive");
    }
    if entity
        .get::<MeleeAttackState>()
        .is_some_and(|m| m.is_attacking)
    {
        return Some("coup de mêlée");
    }
    if let Some(ranged) = entity.get::<RangedAttackState>() {
        let stagger = entity
            .get::<RangedCycle>()
            .is_some_and(|c| frame < c.staggered_until);
        if stagger {
            return Some("coup reçu");
        }
        if entity.get::<Emitter>().is_some() {
            return Some("séquence de tir");
        }
        // Le rechargement est la fin du refroidissement de la règle `Shoot` (jamais l'attente avant la première salve).
        let cooldown = entity
            .get::<EnemyAiConfig>()
            .and_then(|ai| ai.ranged.as_ref())
            .map_or(0, |r| r.cooldown_frames);
        let until = shooter::reload_end(ranged, entity.get::<RangedCycle>());
        if cooldown > 0 && ranged.target.is_none() && frame < until && until - frame <= cooldown {
            return Some("rechargement");
        }
    }
    if let Some(inventory) = entity.get::<WeaponInventory>() {
        if inventory
            .reloading_ending_frame
            .is_some_and(|end| end > frame)
        {
            return Some("rechargement");
        }
        let held = inventory
            .weapons
            .get(inventory.active_weapon_index)
            .and_then(|(weapon, _)| world.get::<WeaponState>(*weapon))
            .is_some_and(|w| w.is_firing || w.raise_ends.is_some_and(|end| end > frame));
        if held {
            return Some("épaulé ou tir tenu");
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le joueur (à curseur) garde l'ordre d'avant, validé : l'esquive, la visée, la marche, le regard. Un
    /// ennemi garde le regard de la simulation (sa cible) même s'il recule ou revient à son poste.
    #[test]
    fn the_body_follows_the_cursor_then_the_walk_and_an_enemy_its_target() {
        let (right, left, up) = (Vec2::X * 2.0, Vec2::NEG_X * 2.0, Vec2::Y);
        // Joueur : l'esquive, la visée, la marche, le regard.
        assert_eq!(
            body_heading(Some(up), Some(right), left, Some(left), true),
            Some(up)
        );
        assert_eq!(
            body_heading(None, Some(right), left, Some(left), true),
            Some(right)
        );
        assert_eq!(body_heading(None, None, left, Some(up), true), Some(left));
        assert_eq!(
            body_heading(None, None, Vec2::ZERO, Some(up), true),
            Some(up)
        );
        // Ennemi : le regard (vers sa cible) avant la marche (le recul, le retour au poste).
        assert_eq!(
            body_heading(None, None, right, Some(left), false),
            Some(left)
        );
        assert_eq!(body_heading(None, None, right, None, false), Some(right));
        assert_eq!(body_heading(None, None, Vec2::ZERO, None, false), None);
    }

    #[test]
    fn the_screen_heading_maps_to_the_ground_yaw() {
        let s = std::f32::consts::FRAC_1_SQRT_2;
        // Vers le bas de l'écran : face à la caméra ; vers le haut : de dos ; à droite : +X.
        assert!(yaw_from_screen(Vec2::new(0.0, -1.0), s).unwrap().abs() < 1e-6);
        assert!(
            (yaw_from_screen(Vec2::new(0.0, 1.0), s).unwrap().abs() - std::f32::consts::PI).abs()
                < 1e-6
        );
        assert!(
            (yaw_from_screen(Vec2::new(1.0, 0.0), s).unwrap() - std::f32::consts::FRAC_PI_2).abs()
                < 1e-6
        );
        // Une diagonale de l'écran est plus profonde au sol : le cap s'ouvre vers la profondeur.
        let diag = yaw_from_screen(Vec2::new(1.0, -1.0), s).unwrap();
        assert!((diag - (1.0f32).atan2(1.0 / s)).abs() < 1e-6);
        assert_eq!(yaw_from_screen(Vec2::ZERO, s), None);
    }
}
