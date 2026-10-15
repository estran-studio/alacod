//! Formations simples (M2-T8, chantier D1 v2, `docs/conventions.md` §49) : escouades en arc et
//! tir alterné.
//!
//! Une escouade est une **donnée dérivée de l'état** : les ennemis vivants (hors salle
//! dormante) qui déclarent le même `ai.squad` et listent une règle `Formation`, rangés par
//! `GgrsNetId`. [`formation_system`] la recalcule à chaque frame dans [`SquadSlots`] (ressource
//! **hors rollback**, vidée puis réécrite avant tout retour anticipé : aucun état caché) ;
//! `behavior_select_system` y lit le tour de tir, `move_enemies` la place à rejoindre.

use std::collections::BTreeMap;

use behaviors::{Behavior, BossState, FormationShape};
use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};
use combat::downed::Downed;
use utils::{net_id::GgrsNetId, order_iter};

use super::state::EnemyBehaviors;
use crate::character::{enemy::Enemy, health::Death, player::Player};

/// Distance à la place sous laquelle un membre arrête de la rejoindre et ne fait plus que
/// `Strafe` léger autour.
pub const FORMATION_TOLERANCE: i32 = 6;
/// Vitesse (fraction de la vitesse max) du `Strafe` léger d'un membre en place.
pub const FORMATION_STRAFE_MULT: f32 = 0.25;

/// Place d'un membre : rang et effectif de son escouade, point à rejoindre sur l'arc, cible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SquadSlot {
    pub rank: u32,
    pub count: u32,
    pub place: FixedVec2,
    pub target: FixedVec2,
    pub radius: Fixed,
    pub period: u32,
    pub volley: bool,
}

/// Places de la frame, par `GgrsNetId.0`. Dérivée, jamais rollbackée.
#[derive(Resource, Default, Debug)]
pub struct SquadSlots(pub BTreeMap<usize, SquadSlot>);

impl SquadSlot {
    /// Ce membre a-t-il le tour de tirer à la frame `frame` ?
    pub fn may_fire(&self, frame: u32) -> bool {
        self.volley || fire_turn(frame, self.period, self.count, self.rank)
    }
}

/// `slot = (frame / period) mod effectif == rang` : un seul membre à la fois, sans curseur.
pub fn fire_turn(frame: u32, period: u32, count: u32, rank: u32) -> bool {
    count <= 1 || period == 0 || (frame / period) % count == rank
}

/// Le plus proche de `from` parmi `candidates` (triés par `GgrsNetId` croissant) ; à distance
/// égale, le premier de la liste (plus petit `GgrsNetId`).
pub fn nearest(from: FixedVec2, candidates: &[(usize, FixedVec2)]) -> Option<FixedVec2> {
    let mut best: Option<(Fixed, FixedVec2)> = None;
    for (_, pos) in candidates {
        let distance = from.distance(pos);
        if best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, *pos));
        }
    }
    best.map(|(_, pos)| pos)
}

/// Places d'un arc de `count` points, centré sur `target`, d'ouverture `spread`, dont le
/// milieu pointe dans la direction unitaire `axis`. Le rang `k` prend le point `k`.
pub fn arc_places(
    count: u32,
    target: FixedVec2,
    axis: FixedVec2,
    radius: Fixed,
    spread: Fixed,
) -> Vec<FixedVec2> {
    (0..count)
        .map(|k| {
            let offset = if count > 1 {
                let t = Fixed::from_num(k) / Fixed::from_num(count - 1) - fixed_math::FIXED_HALF;
                spread.saturating_mul(t)
            } else {
                fixed_math::FIXED_ZERO
            };
            let (cos, sin) = (fixed_math::cos_fixed(offset), fixed_math::sin_fixed(offset));
            let rotated = FixedVec2::new(axis.x * cos - axis.y * sin, axis.x * sin + axis.y * cos);
            target + rotated * radius
        })
        .collect()
}

/// `RollbackSystemSet::EnemyAI`, après `enemy_target_selection` et avant
/// `behavior_select_system`.
#[allow(clippy::type_complexity)]
pub fn formation_system(
    mut slots: ResMut<SquadSlots>,
    enemies: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &EnemyBehaviors,
            Option<&BossState>,
        ),
        (With<Enemy>, Without<world::RoomDormant>),
    >,
    players: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            Has<Downed>,
            Has<Death>,
        ),
        (With<Player>, Without<Enemy>),
    >,
) {
    slots.0.clear();

    // Escouades : id -> membres (ordre `GgrsNetId`, donc rang), avec leur formation.
    #[allow(clippy::type_complexity)]
    let mut squads: BTreeMap<&str, Vec<(usize, FixedVec2, &FormationShape, u32, bool)>> =
        BTreeMap::new();
    for (net_id, transform, rules, boss) in order_iter!(enemies) {
        let Some(squad) = rules.squad.as_deref() else {
            continue;
        };
        let formation =
            rules
                .active(boss.map_or(0, |b| b.phase))
                .iter()
                .find_map(|rule| match rule {
                    Behavior::Formation {
                        shape,
                        period,
                        volley,
                    } => Some((shape, *period, *volley)),
                    _ => None,
                });
        if let Some((shape, period, volley)) = formation {
            squads.entry(squad).or_default().push((
                net_id.0,
                transform.translation.truncate(),
                shape,
                period,
                volley,
            ));
        }
    }
    if squads.is_empty() {
        return;
    }

    // Joueurs debout, par `GgrsNetId` croissant.
    let standing: Vec<(usize, FixedVec2)> = order_iter!(players)
        .into_iter()
        .filter(|(_, _, downed, dead)| !downed && !dead)
        .map(|(net_id, transform, ..)| (net_id.0, transform.translation.truncate()))
        .collect();

    for members in squads.values() {
        let count = members.len() as u32;
        // Centre de l'escouade (moyenne de positions déjà divisées : pas de dépassement).
        let share = Fixed::from_num(count);
        let centroid = members.iter().fold(FixedVec2::ZERO, |sum, member| {
            sum + FixedVec2::new(member.1.x / share, member.1.y / share)
        });
        // Cible : le joueur debout le plus proche du centre, ex æquo au plus petit `GgrsNetId`.
        let target = nearest(centroid, &standing);
        let Some(target) = target else {
            continue;
        };
        let axis = {
            let away = (centroid - target).normalize_or_zero();
            if away == FixedVec2::ZERO {
                FixedVec2::new(
                    fixed_math::FIXED_ZERO,
                    fixed_math::FIXED_ZERO - fixed_math::FIXED_ONE,
                )
            } else {
                away
            }
        };
        // La forme et le rythme sont ceux du premier membre (même contenu pour toute l'escouade).
        let (_, _, shape, period, volley) = members[0];
        let FormationShape::Arc { radius, spread } = shape;
        let places = arc_places(count, target, axis, *radius, *spread);
        for (rank, (id, ..)) in members.iter().enumerate() {
            slots.0.insert(
                *id,
                SquadSlot {
                    rank: rank as u32,
                    count,
                    place: places[rank],
                    target,
                    radius: *radius,
                    period,
                    volley,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_seul_tireur_par_periode() {
        for frame in 0..300 {
            let tireurs = (0..3).filter(|&rank| fire_turn(frame, 30, 3, rank)).count();
            assert_eq!(tireurs, 1, "frame {frame}");
        }
        // l'effectif change : les tours se redistribuent sans trou
        for frame in 0..200 {
            let tireurs = (0..2).filter(|&rank| fire_turn(frame, 30, 2, rank)).count();
            assert_eq!(tireurs, 1);
        }
        // seul : toujours son tour ; période 0 : pas d'alternance
        assert!(fire_turn(7, 30, 1, 0));
        assert!(fire_turn(7, 0, 3, 2));
    }

    #[test]
    fn cible_la_plus_proche_ex_aequo_au_plus_petit_id() {
        let v = |x: i32, y: i32| FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y));
        let candidates = [(3, v(-10, 0)), (5, v(10, 0)), (9, v(100, 0))];
        // centre à égale distance de 3 et 5 : le plus petit id gagne
        assert_eq!(nearest(v(0, 0), &candidates), Some(v(-10, 0)));
        assert_eq!(nearest(v(40, 0), &candidates), Some(v(10, 0)));
        assert_eq!(nearest(v(0, 0), &[]), None);
    }

    #[test]
    fn volley_garde_le_tir_groupe() {
        let slot = |volley| SquadSlot {
            rank: 1,
            count: 3,
            place: FixedVec2::ZERO,
            target: FixedVec2::ZERO,
            radius: Fixed::from_num(100),
            period: 40,
            volley,
        };
        assert!(!slot(false).may_fire(0));
        assert!(slot(false).may_fire(40));
        assert!(slot(true).may_fire(0));
    }

    #[test]
    fn arc_a_rayon_constant_et_milieu_vers_l_axe() {
        let target = FixedVec2::new(Fixed::from_num(100), Fixed::from_num(50));
        let axis = FixedVec2::new(fixed_math::FIXED_ONE, fixed_math::FIXED_ZERO);
        let radius = Fixed::from_num(80);
        let places = arc_places(3, target, axis, radius, Fixed::from_num(2));
        assert_eq!(places.len(), 3);
        for place in &places {
            let d = target.distance(place);
            assert!((d - radius).abs() < Fixed::from_num(0.5), "d = {d}");
        }
        // le point du milieu est exactement sur l'axe
        assert!((places[1].y - target.y).abs() < Fixed::from_num(0.1));
        assert!(places[1].x > target.x);
        // un seul membre : sur l'axe
        let seul = arc_places(1, target, axis, radius, Fixed::from_num(2));
        assert!((seul[0].x - (target.x + radius)).abs() < Fixed::from_num(0.1));
    }
}
