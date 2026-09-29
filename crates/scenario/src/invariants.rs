//! Invariants vérifiés à chaque frame simulée, sans rien écrire dans le scénario.
//! Une violation produit une failure « invariant <nom> : frame N : détail ».

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use game::character::health::Health;
use game::collider::{Collider, Wall};
use utils::net_id::GgrsNetId;

/// Configuration des invariants pour un scénario.
#[derive(Component, Clone, Debug)]
pub struct InvariantConfig {
    /// Si faux, `sante_bornee` n'est pas vérifié.
    pub sante_bornee: bool,
    /// Si faux, `net_ids_uniques` n'est pas vérifié.
    pub net_ids_uniques: bool,
    /// Si faux, `joueur_hors_mur` n'est pas vérifié.
    pub joueur_hors_mur: bool,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            sante_bornee: true,
            net_ids_uniques: true,
            joueur_hors_mur: true,
        }
    }
}

/// Résultat d'une vérification d'invariant.
pub type InvariantResult = Result<(), String>;

/// Vérifie tous les invariants actifs.
pub fn check_invariants(
    world: &mut World,
    config: &InvariantConfig,
    frame: u32,
) -> Vec<String> {
    let mut failures = Vec::new();

    if config.sante_bornee {
        if let Err(msg) = check_sante_bornee(world) {
            failures.push(format!("invariant sante_bornee : frame {frame} : {msg}"));
        }
    }

    if config.net_ids_uniques {
        if let Err(msg) = check_net_ids_uniques(world) {
            failures.push(format!("invariant net_ids_uniques : frame {frame} : {msg}"));
        }
    }

    if config.joueur_hors_mur {
        if let Err(msg) = check_joueur_hors_mur(world) {
            failures.push(format!("invariant joueur_hors_mur : frame {frame} : {msg}"));
        }
    }

    failures
}

/// Pour toute entité rollback avec `Health`, `0 ≤ current ≤ max`.
fn check_sante_bornee(world: &mut World) -> InvariantResult {
    for (net_id, health) in world.query_filtered::<(&GgrsNetId, &Health), With<bevy_ggrs::Rollback>>().iter(world) {
        if health.current < fixed_math::Fixed::ZERO {
            return Err(format!("net_id {} : santé négative {}", net_id.0, health.current));
        }
        if health.current > health.max {
            return Err(format!("net_id {} : santé {} > max {}", net_id.0, health.current, health.max));
        }
    }
    Ok(())
}

/// Deux entités rollback n'ont jamais le même `GgrsNetId`.
fn check_net_ids_uniques(world: &mut World) -> InvariantResult {
    use std::collections::BTreeSet;
    let mut seen = BTreeSet::new();
    for net_id in world.query_filtered::<&GgrsNetId, With<bevy_ggrs::Rollback>>().iter(world) {
        if !seen.insert(net_id.0) {
            return Err(format!("net_id {} apparaît deux fois", net_id.0));
        }
    }
    Ok(())
}

/// Aucun joueur ne chevauche un collider `Wall`.
/// Utilise la même logique que `move_characters` dans `crates/game/src/character/player/input.rs`.
fn check_joueur_hors_mur(world: &mut World) -> InvariantResult {
    use game::collider::is_colliding;

    let players: Vec<_> = world
        .query_filtered::<(&game::character::player::Player, &bevy_fixed::fixed_math::FixedTransform3D, &Collider), With<bevy_ggrs::Rollback>>()
        .iter(world)
        .map(|(player, transform, collider)| (player.clone(), transform.translation.clone(), collider.clone()))
        .collect();

    let walls: Vec<_> = world
        .query_filtered::<(&bevy_fixed::fixed_math::FixedTransform3D, &Collider), With<Wall>>()
        .iter(world)
        .map(|(t, c)| (t.translation.clone(), c.clone()))
        .collect();

    for (player, player_pos, player_collider) in &players {
        for (wall_pos, wall_collider) in &walls {
            if is_colliding(player_pos, player_collider, wall_pos, wall_collider) {
                return Err(format!(
                    "joueur {} chevauche un mur à ({}, {})",
                    player.handle,
                    player_pos.x.to_num::<f32>(),
                    player_pos.y.to_num::<f32>()
                ));
            }
        }
    }
    Ok(())
}
