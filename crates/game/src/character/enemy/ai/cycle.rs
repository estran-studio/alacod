//! Cycle d'un tireur posté (P-1, `docs/conventions.md` §110) : ce qui entoure la séquence de tir d'un
//! ennemi à distance (`EnemyAiConfig::ranged`, `behavior::ranged_attack`) quand son `ranged.cycle` est
//! posé. Verbe générique, composé par le contenu :
//!
//! - `track_target` : la visée de l'émetteur suit la cible à chaque frame (le télégraphe pivote, le
//!   tir part vers la position courante), au lieu d'être figée au départ de la séquence ;
//! - `face_target` : le regard (8 directions) se tourne vers la cible à chaque frame, sauf sonné ;
//! - `wait_frames: (min, max)` : attente tirée (flux RNG `ranged_cycle`, ordre `GgrsNetId`) ajoutée
//!   au `cooldown_frames` à la fin de chaque séquence et après un coup reçu ;
//! - `first_wait: (base, pas)` : première séquence au plus tôt `base + pas × rang` frames après la
//!   première frame du tireur, le rang étant sa place par `GgrsNetId` parmi les tireurs à cycle ;
//! - `stagger_frames` : un coup reçu (`DamageEvent` visant le tireur) interrompt la séquence
//!   (émetteur retiré, télégraphe caché, tir perdu) ; le tireur reste sonné ce nombre de frames ;
//! - `recoil` : au tir, le tireur recule de `distance` px en `frames` frames, à l'opposé de son
//!   regard, courbe `1 − (1 − v)³` ;
//! - `return_speed` : hors séquence, hors recharge et hors sonné, il revient à son poste (sa position
//!   à sa première frame) à cette vitesse (px/s).
//!
//! État rollback dans [`RangedCycle`] (composant **neutre**, posé à la création seulement si
//! `ranged.cycle` est posé) ; la configuration est statique. Aucun `f32`, ordre `GgrsNetId`, RNG
//! consommé dans l'ordre des tireurs.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2, FixedVec3};
use bevy_fixed::rng::RngStreams;
use combat::emitter::Emitter;
use combat::projectile::Pattern;
use serde::{Deserialize, Serialize};
use sim_core::animation_state::FacingDirection;
use sim_core::damage::DamageEvent;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter, order_mut_iter};

use super::state::{EnemyAiConfig, MonsterState, RangedAttackState};
use crate::character::enemy::Enemy;
use crate::character::player::Player;
use crate::collider::{Collider, CollisionLayer, CollisionSettings, Wall};
use crate::frame_events::FrameEvents;

/// Flux RNG des attentes tirées (`RngStreams`, créé au premier tirage : un jeu sans tireur à cycle
/// ne le voit pas).
pub const RANGED_CYCLE_RNG_STREAM: &str = "ranged_cycle";

/// Recul d'un tireur au moment du tir (P-1).
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangedRecoil {
    /// Distance totale (px), à l'opposé du regard.
    pub distance: Fixed,
    /// Durée (frames) ; la part `v` du recul couvre `1 − (1 − v)³` de la distance.
    pub frames: u32,
}

/// Cycle d'un tireur posté (voir le module). Tous les champs sont optionnels.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RangedCycleConfig {
    #[serde(default)]
    pub track_target: bool,
    #[serde(default)]
    pub face_target: bool,
    /// Attente tirée `[min, max]` (frames, bornes comprises) après chaque séquence et chaque coup reçu.
    #[serde(default)]
    pub wait_frames: Option<(u32, u32)>,
    /// Première séquence au plus tôt `base + pas × rang` frames après la première frame du tireur.
    #[serde(default)]
    pub first_wait: Option<(u32, u32)>,
    #[serde(default)]
    pub stagger_frames: Option<u32>,
    #[serde(default)]
    pub recoil: Option<RangedRecoil>,
    /// Vitesse de retour au poste (px/s).
    #[serde(default)]
    pub return_speed: Option<Fixed>,
}

/// État rollback du cycle (neutre : seulement sur les tireurs à `ranged.cycle`).
#[derive(Component, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct RangedCycle {
    /// Poste (position `(x, y)` à la première frame vue).
    pub home: Option<(Fixed, Fixed)>,
    /// Une séquence était en cours à la fin de la frame précédente.
    pub was_firing: bool,
    /// Première frame où le tireur peut revenir à son poste (fin de la recharge ou du coup reçu).
    pub idle_from: u32,
    /// Sonné jusqu'à cette frame (exclue).
    pub staggered_until: u32,
    /// `Emitter::started_at` de la séquence dont le tir a déjà lancé le recul.
    pub recoiled_for: Option<u32>,
    /// Recul en cours : frame du tir et direction unitaire `(x, y)` du recul.
    pub recoil: Option<(u32, Fixed, Fixed)>,
}

/// Index de la première étape qui tire (ni `Wait` ni `Telegraph`) d'un émetteur.
fn first_shot_step(emitter: &Emitter) -> Option<u32> {
    emitter
        .steps
        .iter()
        .position(|step| !matches!(step, Pattern::Wait(_) | Pattern::Telegraph(_)))
        .map(|index| index as u32)
}

/// Pas (px) de la frame `k` (0 à `n − 1`) d'un recul de `distance` en `n` frames, courbe
/// `1 − (1 − v)³` : `distance × ((n − k)³ − (n − k − 1)³) / n³` (somme exacte : `distance`).
pub fn recoil_step(distance: Fixed, frames: u32, k: u32) -> Fixed {
    if frames == 0 || k >= frames {
        return Fixed::ZERO;
    }
    let n = i64::from(frames);
    let rest = n - i64::from(k);
    let weight = rest * rest * rest - (rest - 1) * (rest - 1) * (rest - 1);
    distance.saturating_mul(Fixed::from_num(weight)) / Fixed::from_num(n * n * n)
}

/// Attente tirée `[min, max]`.
fn draw_wait(rng: &mut RngStreams, wait: Option<(u32, u32)>) -> u32 {
    match wait {
        Some((min, max)) if max > min => rng
            .get_mut(RANGED_CYCLE_RNG_STREAM)
            .next_u32_range_inclusive(min, max),
        Some((min, _)) => min,
        None => 0,
    }
}

/// Le cycle des tireurs postés (voir le module). `RollbackSystemSet::EnemyAI`, après
/// `move_enemies` et **avant** `enemy_attack_system` : un coup reçu ou une première attente sont
/// posés avant que `ranged_attack` décide la séquence de la frame ; la fin d'une séquence (décidée
/// par `ranged_attack` à la frame précédente) reçoit son attente tirée. Lit les `DamageEvent` de la
/// frame (`CollisionDamage`, plus tôt), écrit la visée de l'émetteur (lue au tick suivant), le
/// regard, l'état de tir et la position (recul, retour au poste : mouvement direct, arrêté par les
/// murs). Aucune commande.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn ranged_cycle_system(
    frame: Res<FrameCount>,
    mut rng: ResMut<RngStreams>,
    damage: Option<Res<FrameEvents<DamageEvent>>>,
    settings: Res<CollisionSettings>,
    grids: Res<crate::collision_grid::CollisionGrids>,
    mut query: Query<
        (
            &GgrsNetId,
            &mut fixed_math::FixedTransform3D,
            &EnemyAiConfig,
            &super::state::EnemyTarget,
            &mut MonsterState,
            &mut RangedAttackState,
            &mut RangedCycle,
            Option<&mut Emitter>,
            &mut FacingDirection,
            &Collider,
            &CollisionLayer,
        ),
        (With<Enemy>, Without<world::RoomDormant>),
    >,
    player_query: Query<
        (&GgrsNetId, &fixed_math::FixedTransform3D),
        (With<Player>, Without<Enemy>),
    >,
    wall_query: Query<
        (&fixed_math::FixedTransform3D, &Collider, &CollisionLayer),
        (With<Wall>, Without<Enemy>, Without<Player>),
    >,
) {
    let now = frame.frame;
    // Rang de chaque tireur à cycle (ordre `GgrsNetId`), pour `first_wait`. La requête ne porte que
    // des tireurs à `RangedCycle` (posé seulement si `ranged_cycle` et un tir sont configurés).
    // Calculé seulement à la première frame d'un tireur (aucune allocation ensuite).
    let ranks: Vec<usize> = if query.iter().any(|item| item.6.home.is_none()) {
        order_iter!(query)
            .into_iter()
            .map(|item| item.0 .0)
            .collect()
    } else {
        Vec::new()
    };

    for (
        net_id,
        mut transform,
        ai_config,
        target,
        mut state,
        mut ranged_state,
        mut cycle,
        mut emitter,
        mut facing,
        collider,
        layer,
    ) in order_mut_iter!(query)
    {
        if ai_config.ranged.is_none() {
            continue;
        }
        let Some(config) = ai_config.ranged_cycle.as_ref() else {
            continue;
        };
        let position = transform.translation.truncate();
        if cycle.home.is_none() {
            cycle.home = Some((position.x, position.y));
            if let Some((base, step)) = config.first_wait {
                let rank = ranks.iter().position(|id| *id == net_id.0).unwrap_or(0) as u32;
                ranged_state.ready_at = ranged_state.ready_at.max(
                    now.saturating_add(base)
                        .saturating_add(step.saturating_mul(rank)),
                );
            }
        }
        let target_pos = target.target.as_ref().and_then(|id| {
            player_query
                .iter()
                .find(|(player_id, _)| *player_id == id)
                .map(|(_, t)| t.translation.truncate())
        });

        // Coup reçu : séquence interrompue, sonné.
        let hit = config.stagger_frames.is_some()
            && damage
                .as_ref()
                .is_some_and(|events| events.iter().any(|event| event.target == *net_id));
        if let (Some(stagger), true) = (config.stagger_frames, hit) {
            // Séquence perdue : l'émetteur ne tire plus, son télégraphe disparaît, `emitter_system`
            // le retire à son prochain tick (aucune commande ici : pas de point de synchronisation).
            if let Some(emitter) = emitter.as_deref_mut() {
                emitter.finished = true;
                emitter.telegraph = false;
            }
            let wait = draw_wait(&mut rng, config.wait_frames);
            ranged_state.target = None;
            cycle.staggered_until = now.saturating_add(stagger);
            cycle.idle_from = cycle.staggered_until;
            ranged_state.ready_at = cycle.staggered_until.saturating_add(wait);
            cycle.recoil = None;
            cycle.was_firing = false;
            *state = MonsterState::Chasing;
            info!(
                "ggrs{{f={} ranged_cycle net_id={} staggered until={} ready={}}}",
                now, net_id, cycle.staggered_until, ranged_state.ready_at
            );
            continue;
        }

        let firing = ranged_state.target.is_some();
        // Fin de séquence (cette frame, `ranged_attack`) : attente tirée après la recharge.
        if cycle.was_firing && !firing {
            cycle.idle_from = ranged_state.ready_at;
            let wait = draw_wait(&mut rng, config.wait_frames);
            ranged_state.ready_at = ranged_state.ready_at.saturating_add(wait);
        }
        cycle.was_firing = firing;

        let staggered = now < cycle.staggered_until;
        if let (true, false, Some(target_pos)) = (config.face_target, staggered, target_pos) {
            let toward = target_pos - position;
            if toward != FixedVec2::ZERO {
                *facing = FacingDirection::from_fixed_vector(toward);
            }
        }
        if let Some(emitter) = emitter.as_deref_mut() {
            if let (true, Some(target_pos)) = (config.track_target, target_pos) {
                let aim = (target_pos - position).normalize_or_zero();
                if aim != FixedVec2::ZERO && aim != emitter.aim {
                    emitter.aim = aim;
                }
            }
            // Le tir de la séquence lance le recul (une fois par séquence).
            if let Some(recoil) = &config.recoil {
                let fired = first_shot_step(emitter).is_some_and(|shot| emitter.next_step > shot);
                if fired && cycle.recoiled_for != Some(emitter.started_at) {
                    cycle.recoiled_for = Some(emitter.started_at);
                    let back = FixedVec2::ZERO - combat::weapons::melee::facing_to_fixed(*facing);
                    cycle.recoil = Some((now, back.x, back.y));
                    info!(
                        "ggrs{{f={} ranged_cycle net_id={} recoil frames={}}}",
                        now, net_id, recoil.frames
                    );
                }
            }
        }

        // Mouvement : recul en cours, sinon retour au poste.
        let mut step = FixedVec2::ZERO;
        if let (Some((from, back_x, back_y)), Some(recoil)) = (cycle.recoil, config.recoil.as_ref())
        {
            let k = now.saturating_sub(from);
            if k < recoil.frames {
                step =
                    FixedVec2::new(back_x, back_y) * recoil_step(recoil.distance, recoil.frames, k);
            } else {
                cycle.recoil = None;
            }
        } else if let (Some(speed), Some(home), true, false) = (
            config.return_speed,
            cycle.home,
            // Hors séquence (ni émetteur ni cible de tir : la recharge commence quand
            // `ranged_attack` clôt la séquence) et la recharge finie.
            emitter.is_none() && !firing && now >= cycle.idle_from,
            staggered,
        ) {
            let home = FixedVec2::new(home.0, home.1);
            let (distance, toward) = (home - position).length_and_direction();
            let per_frame = speed / Fixed::from_num(60);
            step = if distance <= per_frame {
                home - position
            } else {
                toward * per_frame
            };
        }
        if step != FixedVec2::ZERO {
            let start = transform.translation;
            let next = FixedVec3::new(start.x + step.x, start.y + step.y, start.z);
            let aabb = crate::collision_grid::collider_aabb(&next, collider);
            let blocked = grids.walls.query_aabb(&aabb).into_iter().any(|entry| {
                wall_query.get(entry.entity).is_ok_and(
                    |(wall_transform, wall_collider, wall_layer)| {
                        settings.layer_matrix[layer.0][wall_layer.0]
                            && crate::collider::step_blocked_by(
                                &start,
                                &next,
                                collider,
                                &wall_transform.translation,
                                wall_collider,
                            )
                    },
                )
            });
            if !blocked {
                transform.translation = next;
            }
        }
    }
}
