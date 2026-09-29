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
pub use game::replay::Invariants;
use std::collections::BTreeSet;
use utils::net_id::GgrsNetId;

/// Les requêtes des invariants, construites une fois par partie (pas à chaque frame).
pub struct InvariantQueries {
    santes: QueryState<(&'static GgrsNetId, &'static Health), With<Rollback>>,
    net_ids: QueryState<&'static GgrsNetId, With<Rollback>>,
    joueurs: QueryState<(&'static Player, &'static FixedTransform3D, &'static Collider), With<Rollback>>,
    murs: QueryState<(&'static FixedTransform3D, &'static Collider), With<Wall>>,
}

impl InvariantQueries {
    pub fn new(world: &mut World) -> Self {
        Self {
            santes: world.query_filtered(),
            net_ids: world.query_filtered(),
            joueurs: world.query_filtered(),
            murs: world.query_filtered(),
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
                .map(|(player, transform, collider)| (player.handle, transform.clone(), collider.clone()))
                .collect();
            'joueurs: for (handle, transform, collider) in &joueurs {
                for (mur_transform, mur_collider) in self.murs.iter(world) {
                    if is_colliding(&transform.translation, collider, &mur_transform.translation, mur_collider) {
                        failures.push(format!(
                            "invariant joueur_hors_mur : frame {frame} : joueur {handle} chevauche un mur en ({}, {})",
                            transform.translation.x, transform.translation.y
                        ));
                        break 'joueurs;
                    }
                }
            }
        }

        failures
    }
}
