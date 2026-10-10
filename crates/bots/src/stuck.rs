//! m1-bots-apres-movement-feel : ennemis coincés, jugés sur leur déplacement réel.
//!
//! `Velocity::main` est la vitesse **voulue** par l'IA : un ennemi qui pousse contre un mur
//! la garde non nulle sans bouger (throne, 2 bots, graine 19 : un rat collé à un coin de roche
//! de f2450 à f3050, position immobile à 0,15 px près, vitesse voulue non nulle). Il
//! n'était pas vu comme immobile (`BotView::enemy_still`) : `prudent` avançait jusqu'à la
//! ligne de tir, s'y trouvait sous `PRUDENT_MIN_DISTANCE`, reculait, perdait la ligne et
//! recommençait, toutes les 18 frames, sans le tuer.
//!
//! Mémoire hors rollback, comme [`crate::arms::WeaponChoices`] : les bots produisent des inputs,
//! pas de l'état de simulation ; déterministe (frame `FrameCount`, ordre `GgrsNetId`), vidée à
//! chaque entrée en partie (`OnEnter(AppState::InGame)`).
use std::collections::BTreeMap;

use bevy::prelude::Resource;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};

/// Un ennemi qui reste à moins de ce rayon de son point d'ancrage n'a pas bougé.
pub const STUCK_RADIUS: Fixed = Fixed::from_bits(2 << 16);

/// Frames sans quitter [`STUCK_RADIUS`] pour qu'un ennemi soit vu comme coincé (1 s).
pub const STUCK_FRAMES: u32 = 60;

/// `GgrsNetId` d'ennemi → (point d'ancrage, frame où il y est arrivé).
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct EnemyMoves(pub BTreeMap<usize, (FixedVec2, u32)>);

impl EnemyMoves {
    /// Relève les positions de la frame `frame` (`enemies` : tous les ennemis présents) et
    /// oublie les absents.
    pub fn observe(&mut self, frame: u32, enemies: &[(usize, FixedVec2)]) {
        self.0
            .retain(|id, _| enemies.iter().any(|(present, _)| present == id));
        for (id, position) in enemies {
            let entry = self.0.entry(*id).or_insert((*position, frame));
            if entry.0.distance(position) > STUCK_RADIUS {
                *entry = (*position, frame);
            }
        }
    }

    /// L'ennemi `id` n'a pas quitté son point d'ancrage depuis [`STUCK_FRAMES`] frames.
    pub fn stuck(&self, id: usize, frame: u32) -> bool {
        self.0
            .get(&id)
            .is_some_and(|(_, since)| frame.saturating_sub(*since) >= STUCK_FRAMES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f32, y: f32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    #[test]
    fn coince_apres_une_seconde_sur_place() {
        let mut moves = EnemyMoves::default();
        // Graine 19 : le rat tremble de quelques centièmes de pixel contre la roche
        for frame in 0..=STUCK_FRAMES {
            let jitter = (frame % 3) as f32 * 0.07;
            moves.observe(frame, &[(247, p(165.99 - jitter, 600.88))]);
            assert_eq!(moves.stuck(247, frame), frame >= STUCK_FRAMES);
        }
    }

    #[test]
    fn un_ennemi_qui_marche_n_est_pas_coince() {
        let mut moves = EnemyMoves::default();
        for frame in 0..200 {
            moves.observe(frame, &[(7, p(frame as f32 * 0.5, 0.0))]);
            assert!(!moves.stuck(7, frame));
        }
    }

    #[test]
    fn repart_puis_oublie_les_absents() {
        let mut moves = EnemyMoves::default();
        for frame in 0..=STUCK_FRAMES {
            moves.observe(frame, &[(1, p(0.0, 0.0)), (2, p(50.0, 0.0))]);
        }
        assert!(moves.stuck(1, STUCK_FRAMES));
        // Il se dégage : plus coincé, nouvel ancrage
        moves.observe(STUCK_FRAMES + 1, &[(1, p(3.0, 0.0)), (2, p(50.0, 0.0))]);
        assert!(!moves.stuck(1, STUCK_FRAMES + 1));
        assert!(moves.stuck(2, STUCK_FRAMES + 1));
        // L'ennemi 2 meurt : oublié
        moves.observe(STUCK_FRAMES + 2, &[(1, p(3.0, 0.0))]);
        assert!(!moves.0.contains_key(&2));
    }
}
