//! Fonctions pures des familiers (M3-T6, `docs/conventions.md` §71) : position d'un orbital, pas de
//! file d'un suiveur. `Fixed` seulement (aucun `f32`), sans état : une position d'orbite se déduit
//! du numéro de frame, un pas de file de la position courante et de celle de l'ancre.

use bevy::prelude::Component;
use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};

/// Vitesse de rattrapage d'un suiveur, en unités par frame (300 u/s : plus vite qu'un joueur).
pub const FOLLOW_SPEED: i32 = 5;

/// Phase de départ de l'orbital `index` parmi `count` orbitaux d'un même porteur :
/// `index × 2π / count` (répartis régulièrement sur le cercle). `count = 0` : 0.
pub fn orbit_phase(index: u32, count: u32) -> Fixed {
    if count == 0 {
        return Fixed::ZERO;
    }
    fixed_math::FIXED_TAU.saturating_mul(Fixed::from_num(index)) / Fixed::from_num(count)
}

/// Replie un angle dans `[-π, π]` : le CORDIC de `fixed_math` perd sa précision (et peut déborder)
/// au-delà de `3π/2`, et l'angle d'une orbite atteint `4π` (phase + tour).
fn wrap_angle(angle: Fixed) -> Fixed {
    let mut angle = angle;
    while angle > fixed_math::FIXED_PI {
        angle -= fixed_math::FIXED_TAU;
    }
    while angle < -fixed_math::FIXED_PI {
        angle += fixed_math::FIXED_TAU;
    }
    angle
}

/// Position de l'orbital `index` parmi `count` autour de `center` à la frame `frame` : angle
/// `index × 2π / count + 2π × (frame mod frames_per_turn) / frames_per_turn` (sens trigonométrique),
/// à `radius` unités. Pure : aucun état, donc juste après un rollback ou à l'apparition.
/// `frames_per_turn = 0` : traité comme 1 (le lint le refuse).
pub fn orbit_position(
    center: FixedVec2,
    radius: Fixed,
    frames_per_turn: u32,
    index: u32,
    count: u32,
    frame: u32,
) -> FixedVec2 {
    let per_turn = frames_per_turn.max(1);
    // Fraction de tour en entiers (`Fixed` est un I16F16 : `frames_per_turn` peut dépasser 32767).
    let fraction =
        Fixed::from_bits((u64::from(frame % per_turn) * 65_536 / u64::from(per_turn)) as i32);
    let turn = fixed_math::FIXED_TAU.saturating_mul(fraction);
    let angle = wrap_angle(orbit_phase(index, count).saturating_add(turn));
    FixedVec2::new(
        center
            .x
            .saturating_add(fixed_math::cos_fixed(angle).saturating_mul(radius)),
        center
            .y
            .saturating_add(fixed_math::sin_fixed(angle).saturating_mul(radius)),
    )
}

/// Pas d'un suiveur (`Follow`) : rejoint `anchor` (le porteur, ou le familier de rang précédent)
/// jusqu'à `gap` unités. À `gap` ou moins : immobile. Au-delà : avance de `FOLLOW_SPEED` unités,
/// ou du quart de l'excédent s'il est plus grand (un porteur qui file ne distance pas la file),
/// sans jamais dépasser l'excédent. Les familiers d'un même porteur forment ainsi une file :
/// chacun vise celui qui le précède.
pub fn follow_step(position: FixedVec2, anchor: FixedVec2, gap: Fixed) -> FixedVec2 {
    let delta = FixedVec2::new(
        anchor.x.saturating_sub(position.x),
        anchor.y.saturating_sub(position.y),
    );
    let distance = delta.length();
    if distance <= gap {
        return position;
    }
    let excess = distance - gap;
    let step = Fixed::from_num(FOLLOW_SPEED)
        .max(excess / Fixed::from_num(4))
        .min(excess);
    let direction = delta.normalize_or_zero();
    FixedVec2::new(
        position.x.saturating_add(direction.x.saturating_mul(step)),
        position.y.saturating_add(direction.y.saturating_mul(step)),
    )
}

/// Dernier tir d'un familier `MirrorShot` (frame), pour l'espacement de `every` frames. Composant
/// rollback **neutre** (posé seulement sur les familiers à `MirrorShot`, aucun avant M3-T6).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FamiliarGun {
    pub last_shot: Option<u32>,
}

impl FamiliarGun {
    /// Peut tirer à `frame` : jamais tiré, ou au moins `every` frames depuis le dernier tir.
    pub fn ready(&self, frame: u32, every: u32) -> bool {
        self.last_shot
            .is_none_or(|last| frame.saturating_sub(last) >= every)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    fn v(x: f32, y: f32) -> FixedVec2 {
        FixedVec2::new(fx(x), fx(y))
    }

    fn near(a: FixedVec2, b: FixedVec2, tolerance: f32) -> bool {
        a.distance(&b) <= fx(tolerance)
    }

    #[test]
    fn phases_reparties_sur_le_cercle() {
        assert_eq!(orbit_phase(0, 3), Fixed::ZERO);
        assert_eq!(orbit_phase(0, 0), Fixed::ZERO);
        let third = orbit_phase(1, 3);
        assert!((third - fixed_math::FIXED_TAU / fx(3.0)).abs() < fx(0.001));
        assert!((orbit_phase(2, 4) - fixed_math::FIXED_PI).abs() < fx(0.001));
    }

    #[test]
    fn orbite_a_rayon_constant_et_sens_trigonometrique() {
        let center = v(100.0, -40.0);
        // Un tour en 60 frames, un seul orbital : f0 sur +x, quart de tour sur +y.
        assert!(near(
            orbit_position(center, fx(30.0), 60, 0, 1, 0),
            v(130.0, -40.0),
            0.05
        ));
        assert!(near(
            orbit_position(center, fx(30.0), 60, 0, 1, 15),
            v(100.0, -10.0),
            0.05
        ));
        assert!(near(
            orbit_position(center, fx(30.0), 60, 0, 1, 30),
            v(70.0, -40.0),
            0.05
        ));
        // Le rayon ne varie jamais, y compris avec les phases qui dépassent 3π/2.
        for count in 1..=5 {
            for index in 0..count {
                for frame in 0..120 {
                    let p = orbit_position(center, fx(30.0), 60, index, count, frame);
                    assert!(
                        (p.distance(&center) - fx(30.0)).abs() < fx(0.1),
                        "index {index}/{count} frame {frame} : {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn orbite_periodique_et_orbitaux_decales() {
        let center = v(0.0, 0.0);
        let a = orbit_position(center, fx(20.0), 40, 0, 2, 7);
        assert!(near(
            a,
            orbit_position(center, fx(20.0), 40, 0, 2, 47),
            0.05
        ));
        // Deux orbitaux : diamétralement opposés à chaque frame.
        let b = orbit_position(center, fx(20.0), 40, 1, 2, 7);
        assert!(near(a, v(-b.x.to_num::<f32>(), -b.y.to_num::<f32>()), 0.1));
        // Pur : même entrée, même sortie, bit pour bit.
        assert_eq!(a, orbit_position(center, fx(20.0), 40, 0, 2, 7));
    }

    #[test]
    fn suiveur_immobile_dans_la_laisse_et_rattrape_au_dela() {
        let anchor = v(100.0, 0.0);
        let near_pos = v(80.0, 0.0);
        assert_eq!(
            follow_step(near_pos, anchor, fx(28.0)),
            near_pos,
            "à 20 < 28 : immobile"
        );
        // À 60 unités (excédent 32) : pas = max(5, 8) = 8.
        let stepped = follow_step(v(40.0, 0.0), anchor, fx(28.0));
        assert!(near(stepped, v(48.0, 0.0), 0.05), "{stepped:?}");
        // Excédent de 2 : on ne dépasse pas la laisse.
        let snug = follow_step(v(70.0, 0.0), anchor, fx(28.0));
        assert!(near(snug, v(72.0, 0.0), 0.05), "{snug:?}");
        // Converge à `gap` sans le dépasser.
        let mut pos = v(0.0, 0.0);
        for _ in 0..200 {
            pos = follow_step(pos, anchor, fx(28.0));
            assert!(pos.distance(&anchor) >= fx(27.9));
        }
        assert!(near(pos, v(72.0, 0.0), 0.2));
    }

    #[test]
    fn file_de_suiveurs_chacun_derriere_le_precedent() {
        // Porteur en (100, 0) ; trois suiveurs au même point : en quelques frames, ils s'étagent.
        let gap = fx(20.0);
        let owner = v(100.0, 0.0);
        let mut chain = [v(100.0, 0.0), v(100.0, 0.0), v(100.0, 0.0)];
        // Le porteur s'éloigne à 3 u/frame vers +x.
        for step in 0..120 {
            let owner = v(100.0 + 3.0 * step as f32, 0.0);
            let mut anchor = owner;
            for follower in chain.iter_mut() {
                *follower = follow_step(*follower, anchor, gap);
                anchor = *follower;
            }
        }
        let owner_end = v(100.0 + 3.0 * 119.0, 0.0);
        let _ = owner;
        assert!(chain[0].distance(&owner_end) <= gap + fx(0.5));
        assert!(chain[1].distance(&chain[0]) <= gap + fx(0.5));
        assert!(chain[2].distance(&chain[1]) <= gap + fx(0.5));
        assert!(
            chain[0].x > chain[1].x && chain[1].x > chain[2].x,
            "file ordonnée : {chain:?}"
        );
    }

    /// `Fixed` est un I16F16 : un tour en plus de 32767 frames ne doit ni paniquer ni se figer.
    #[test]
    fn orbite_tres_lente_ne_deborde_pas() {
        let center = v(0.0, 0.0);
        assert!(near(
            orbit_position(center, fx(24.0), 36_000, 0, 1, 0),
            v(24.0, 0.0),
            0.05
        ));
        // 9000 frames sur 36000 : quart de tour.
        assert!(near(
            orbit_position(center, fx(24.0), 36_000, 0, 1, 9_000),
            v(0.0, 24.0),
            0.1
        ));
        assert!(near(
            orbit_position(center, fx(24.0), u32::MAX, 0, 1, 5),
            v(24.0, 0.0),
            0.05
        ));
    }

    #[test]
    fn tir_espace_de_every_frames() {
        let mut gun = FamiliarGun::default();
        assert!(gun.ready(0, 12));
        gun.last_shot = Some(10);
        assert!(!gun.ready(21, 12));
        assert!(gun.ready(22, 12));
        assert!(
            gun.ready(10, 0),
            "every = 0 : jamais bloqué (le lint le refuse)"
        );
    }
}
