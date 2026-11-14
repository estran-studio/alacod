//! Les gestes d'un ennemi à distance lus dans l'état de la simulation (P-1d, `docs/conventions.md` § 122).
//!
//! Un tireur n'a ni détente ni chargeur : sa séquence est un [`Emitter`] (`Wait` avant le télégraphe =
//! l'épaulé, `Telegraph` = en joue, étapes de tir, `Wait` après = le geste du tir), puis son
//! `cooldown_frames` (`RangedAttackState::ready_at`, `RangedCycle::idle_from`) = le rechargement. Cette
//! lecture donne à [`super::pose::choose`] les mêmes entrées que le joueur (détente tenue, temps depuis le
//! tir, rechargement) : le clip joué, sa position et sa durée suivent l'état de la simulation, et un rollback
//! qui le corrige corrige la pose. Pur : testé seul, ne lit que de l'état (aucune écriture, aucun `f32` de
//! simulation).

use combat::emitter::Emitter;
use combat::projectile::Pattern;

use crate::character::enemy::ai::cycle::{first_shot_step, RangedCycle};
use crate::character::enemy::ai::state::RangedAttackState;

/// Ce que la présentation lit du tir d'un ennemi, en frames simulées.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShooterRead {
    /// Séquence en cours avant le tir (épaulé puis en joue) : frames depuis son début.
    pub raising_for: Option<u32>,
    /// Frames depuis la première salve de la séquence en cours (le geste du tir).
    pub since_shot: Option<u32>,
    /// Rechargement : (frames écoulées, durée totale) ; la durée est le `cooldown_frames` du tireur.
    pub reload: Option<(u32, u32)>,
}

/// Durée bloquante d'une étape (`Wait`, `Telegraph`) ; 0 pour une salve.
fn blocking_frames(step: &Pattern) -> u32 {
    match step {
        Pattern::Wait(n) | Pattern::Telegraph(n) => *n,
        _ => 0,
    }
}

/// Lecture des gestes d'un tireur à la frame `frame`. `cooldown_frames` : le rechargement de sa règle
/// `Shoot` ; `cycle` : son cycle posté (`ranged_cycle`) s'il en a un.
pub fn read(
    frame: u32,
    emitter: Option<&Emitter>,
    ranged: &RangedAttackState,
    cooldown_frames: u32,
    cycle: Option<&RangedCycle>,
) -> ShooterRead {
    let mut read = ShooterRead::default();
    if let Some(emitter) = emitter {
        let elapsed = frame.saturating_sub(emitter.started_at);
        match first_shot_step(emitter) {
            // La première étape joue la frame qui suit la pose de l'émetteur.
            Some(shot) if emitter.next_step > shot => {
                let before: u32 = emitter.steps[..shot as usize]
                    .iter()
                    .map(blocking_frames)
                    .sum();
                read.since_shot = Some(elapsed.saturating_sub(before + 1));
            }
            _ => read.raising_for = Some(elapsed),
        }
        return read;
    }
    // Sonné (coup reçu) : ni geste ni rechargement, la recharge due reprend après.
    if cycle.is_some_and(|c| frame < c.staggered_until) {
        return read;
    }
    // Fin du rechargement : la première frame où le tireur est de nouveau libre (`idle_from` du cycle,
    // sinon la fin du refroidissement).
    let end = cycle.map_or(ranged.ready_at, |c| c.idle_from);
    if cooldown_frames > 0 && ranged.target.is_none() && frame < end && end - frame <= cooldown_frames
    {
        read.reload = Some((cooldown_frames - (end - frame), cooldown_frames));
    }
    read
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::{Fixed, FixedVec2};
    use sim_core::damage::FriendlyFire;
    use std::sync::Arc;

    /// La salve du fantassin : épauler (23), en joue (60), une balle, le geste du tir (35).
    fn salvo(started_at: u32, next_step: u32) -> Emitter {
        let pattern = Pattern::Sequence(vec![
            Pattern::Wait(23),
            Pattern::Telegraph(60),
            Pattern::Aimed {
                count: 1,
                spread: Fixed::ZERO,
                projectile: "balle".into(),
            },
            Pattern::Wait(35),
        ]);
        let mut emitter = Emitter::new(
            "salve",
            &pattern,
            "arme",
            Arc::new(Default::default()),
            Fixed::ONE,
            FriendlyFire::Never,
            FixedVec2::new(Fixed::ONE, Fixed::ZERO),
            started_at,
        );
        emitter.next_step = next_step;
        emitter
    }

    #[test]
    fn l_epaule_puis_en_joue_comptent_depuis_le_debut_de_la_sequence() {
        let ranged = RangedAttackState::default();
        let early = read(110, Some(&salvo(100, 0)), &ranged, 180, None);
        assert_eq!(early.raising_for, Some(10));
        assert_eq!((early.since_shot, early.reload), (None, None));
        let aiming = read(190, Some(&salvo(100, 1)), &ranged, 180, None);
        assert_eq!(aiming.raising_for, Some(90));
    }

    #[test]
    fn le_geste_du_tir_compte_depuis_la_salve() {
        let ranged = RangedAttackState::default();
        // Salve à la frame 100 + 1 + 23 + 60 = 184 ; l'étape suivante (`Wait`) est l'index 3.
        let fired = read(194, Some(&salvo(100, 3)), &ranged, 180, None);
        assert_eq!(fired.since_shot, Some(10));
        assert_eq!(fired.raising_for, None);
    }

    /// Le rechargement complet : 180 frames de 0 à 180, une seule fois, puis plus rien (l'attente
    /// tirée suit sans clip de recharge).
    #[test]
    fn le_rechargement_court_sur_tout_le_refroidissement() {
        let ranged = RangedAttackState {
            target: None,
            ready_at: 400,
        };
        // Un cycle posté : la recharge finit à `idle_from`, l'attente tirée va jusqu'à `ready_at`.
        let cycle = RangedCycle {
            idle_from: 380,
            ..Default::default()
        };
        let ranged = RangedAttackState {
            ready_at: 400,
            ..ranged
        };
        assert_eq!(
            read(200, None, &ranged, 180, Some(&cycle)).reload,
            Some((0, 180))
        );
        assert_eq!(
            read(290, None, &ranged, 180, Some(&cycle)).reload,
            Some((90, 180))
        );
        assert_eq!(
            read(379, None, &ranged, 180, Some(&cycle)).reload,
            Some((179, 180))
        );
        assert_eq!(read(380, None, &ranged, 180, Some(&cycle)).reload, None);
        assert_eq!(read(399, None, &ranged, 180, Some(&cycle)).reload, None);
    }

    #[test]
    fn sans_cycle_le_refroidissement_est_le_rechargement() {
        let ranged = RangedAttackState {
            target: None,
            ready_at: 500,
        };
        assert_eq!(read(320, None, &ranged, 180, None).reload, Some((0, 180)));
        assert_eq!(read(100, None, &ranged, 180, None).reload, None, "trop tôt");
    }

    #[test]
    fn un_coup_recu_ne_joue_pas_le_rechargement() {
        let cycle = RangedCycle {
            idle_from: 237,
            staggered_until: 237,
            ..Default::default()
        };
        let ranged = RangedAttackState {
            target: None,
            ready_at: 260,
        };
        assert_eq!(read(210, None, &ranged, 180, Some(&cycle)).reload, None);
    }
}
