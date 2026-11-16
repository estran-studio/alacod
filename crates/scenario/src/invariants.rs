//! Invariants vérifiés à chaque frame simulée, sans rien écrire dans le scénario (plan §9.4).
//! Une violation produit une failure « invariant <nom> : frame N : détail ». Chaque invariant se
//! désactive par scénario (`invariants: (joueur_hors_mur: false)`, voir [`Invariants`]).

use bevy::ecs::query::{QueryState, With};
use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedTransform3D};
use bevy_ggrs::Rollback;
use game::character::health::Health;
use game::character::player::Player;
use game::collider::{is_colliding, Collider, Wall};
use game::model3d::playback::{sim_activity, ModelPlayback};
pub use game::replay::Invariants;
use std::collections::{BTreeMap, BTreeSet};
use utils::net_id::GgrsNetId;

/// Frames d'un même état de la simulation avant qu'on lui demande son clip (le fondu et les gestes d'une frame passent).
const ANIMATION_SETTLE_FRAMES: u32 = 6;
/// Frames sans que la position du clip joué bouge avant de le dire figé.
const ANIMATION_STUCK_FRAMES: u32 = 30;
/// Déplacement (unités du monde par frame) au-delà duquel un personnage marche.
const ANIMATION_MOVING_UNITS: f32 = 0.15;

/// Mémoire de l'invariant `animation_vivante` pour un personnage à modèle.
#[derive(Default)]
struct AnimationHistory {
    last_frame: Option<u32>,
    last_position: Option<(f32, f32)>,
    /// Clips joués au repos (aucune activité de la simulation, à l'arrêt).
    rest: BTreeSet<String>,
    label: Option<&'static str>,
    label_since: u32,
    clip: Option<String>,
    last_time: Option<f32>,
    stuck_for: u32,
    reported: BTreeSet<String>,
}

/// Les requêtes des invariants, construites une fois par partie (pas à chaque frame).
pub struct InvariantQueries {
    santes: QueryState<(&'static GgrsNetId, &'static Health), With<Rollback>>,
    net_ids: QueryState<&'static GgrsNetId, With<Rollback>>,
    joueurs: QueryState<
        (
            &'static Player,
            &'static FixedTransform3D,
            &'static Collider,
        ),
        With<Rollback>,
    >,
    murs: QueryState<(&'static FixedTransform3D, &'static Collider), With<Wall>>,
    animes: QueryState<(Entity, &'static ModelPlayback, &'static FixedTransform3D)>,
    historiques: BTreeMap<Entity, AnimationHistory>,
}

impl InvariantQueries {
    pub fn new(world: &mut World) -> Self {
        Self {
            santes: world.query_filtered(),
            net_ids: world.query_filtered(),
            joueurs: world.query_filtered(),
            murs: world.query_filtered(),
            animes: world.query(),
            historiques: BTreeMap::new(),
        }
    }

    /// Vérifie les invariants actifs après la frame `frame` ; une violation au plus par invariant.
    pub fn check(&mut self, world: &mut World, config: &Invariants, frame: u32) -> Vec<String> {
        let mut failures = Vec::new();

        if config.sante_bornee {
            for (net_id, health) in self.santes.iter(world) {
                if health.current < Fixed::ZERO || health.current > health.max {
                    failures.push(format!(
                        "invariant sante_bornee : frame {frame} : net_id {} : santé {} hors de [0, {}]",
                        net_id.0, health.current, health.max
                    ));
                    break;
                }
            }
        }

        if config.net_ids_uniques {
            let mut vus = BTreeSet::new();
            for net_id in self.net_ids.iter(world) {
                if !vus.insert(net_id.0) {
                    failures.push(format!(
                        "invariant net_ids_uniques : frame {frame} : net_id {} apparaît deux fois",
                        net_id.0
                    ));
                    break;
                }
            }
        }

        if config.joueur_hors_mur {
            // Peu de joueurs, beaucoup de murs : on copie les joueurs, pas les murs.
            let joueurs: Vec<(usize, FixedTransform3D, Collider)> = self
                .joueurs
                .iter(world)
                .map(|(player, transform, collider)| {
                    (player.handle, transform.clone(), collider.clone())
                })
                .collect();
            'joueurs: for (handle, transform, collider) in &joueurs {
                for (mur_transform, mur_collider) in self.murs.iter(world) {
                    if is_colliding(
                        &transform.translation,
                        collider,
                        &mur_transform.translation,
                        mur_collider,
                    ) {
                        failures.push(format!(
                            "invariant joueur_hors_mur : frame {frame} : joueur {handle} chevauche un mur en ({}, {})",
                            transform.translation.x, transform.translation.y
                        ));
                        break 'joueurs;
                    }
                }
            }
        }

        if config.animation_vivante {
            self.check_animation(world, frame, &mut failures);
        }

        failures
    }

    /// `animation_vivante` : pour chaque personnage à modèle (`ModelPlayback`, décidé hors rendu), à chaque état de la
    /// simulation qui dure (esquive, coup, séquence de tir, rechargement, coup reçu, marche…) : un clip est résolu
    /// (`unresolved` vide), il **n'est pas** celui que le personnage joue au repos, et sa position **avance**. Rien à
    /// écrire dans le scénario : l'état vient des composants de simulation (`sim_activity`), le clip du composant de
    /// présentation. Une violation au plus par personnage et par cause.
    fn check_animation(&mut self, world: &mut World, frame: u32, failures: &mut Vec<String>) {
        struct Seen {
            entity: Entity,
            model: String,
            clip: Option<String>,
            time: Option<f32>,
            unresolved: Option<String>,
            position: (f32, f32),
        }
        let seen: Vec<Seen> = self
            .animes
            .iter(world)
            .map(|(entity, playback, transform)| Seen {
                entity,
                model: playback.model.clone(),
                clip: playback.clip().map(str::to_string),
                time: playback.time(),
                unresolved: playback.unresolved.clone(),
                position: (
                    transform.translation.x.to_num::<f32>(),
                    transform.translation.y.to_num::<f32>(),
                ),
            })
            .collect();
        let alive: BTreeSet<Entity> = seen.iter().map(|s| s.entity).collect();
        self.historiques.retain(|entity, _| alive.contains(entity));
        for s in seen {
            let net_id = world
                .get::<GgrsNetId>(s.entity)
                .map_or_else(|| "?".to_string(), |id| id.0.to_string());
            let who = format!("{} (net_id {net_id})", s.model);
            let activity = sim_activity(world, s.entity, frame);
            let h = self.historiques.entry(s.entity).or_default();
            if h.last_frame == Some(frame) {
                continue;
            }
            h.last_frame = Some(frame);
            let mut fail = |h: &mut AnimationHistory, cause: &str, detail: String| {
                if h.reported.insert(cause.to_string()) {
                    failures.push(format!(
                        "invariant animation_vivante : frame {frame} : {who} : {detail}"
                    ));
                }
            };
            if let Some(why) = &s.unresolved {
                fail(h, "non résolu", why.clone());
                continue;
            }
            let Some(clip) = s.clip.clone() else {
                continue;
            };
            let moving = h.last_position.is_some_and(|(x, y)| {
                (s.position.0 - x).hypot(s.position.1 - y) > ANIMATION_MOVING_UNITS
            });
            h.last_position = Some(s.position);
            let label = activity.or(moving.then_some("déplacement"));
            if label != h.label {
                h.label = label;
                h.label_since = frame;
            }
            let lasting = frame.saturating_sub(h.label_since) >= ANIMATION_SETTLE_FRAMES;
            match label {
                None if lasting => {
                    h.rest.insert(clip.clone());
                }
                Some(label) if lasting && h.rest.contains(&clip) => {
                    let detail = format!(
                        "état « {label} » depuis {} frames, mais le clip joué `{clip}` est celui de l'attente",
                        frame - h.label_since
                    );
                    fail(h, label, detail);
                }
                _ => {}
            }
            // La position du clip avance tant que le clip joué ne change pas.
            if h.clip.as_ref() == Some(&clip) && h.last_time == s.time {
                h.stuck_for += 1;
                if h.stuck_for >= ANIMATION_STUCK_FRAMES {
                    let detail = format!(
                        "le clip `{clip}` est figé à {:.3} s depuis {} frames{}",
                        s.time.unwrap_or(0.0),
                        h.stuck_for,
                        label.map_or(String::new(), |l| format!(" (état « {l} »)")),
                    );
                    fail(h, "figé", detail);
                }
            } else {
                h.stuck_for = 0;
            }
            h.clip = Some(clip);
            h.last_time = s.time;
        }
    }
}
