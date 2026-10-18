//! Statut `Charm` (suivi Q12, `docs/conventions.md` §19) : règles pures de l'ennemi charmé.
//!
//! Un ennemi sous `combat::status::StatusDef::Charm` sert le camp des joueurs pendant la durée du
//! statut : il cible l'**ennemi hostile le plus proche** ([`nearest_target`], égalité : le plus petit
//! `GgrsNetId`), marche droit sur lui ([`charm_motion`]), le frappe (`behavior::enemy_attack_system`,
//! dégât d'équipe `Players`) et ne touche aucun joueur ; le statut expiré, il redevient hostile (sa cible
//! `TargetType::Enemy` est effacée par `behavior::enemy_target_selection`).
//!
//! Un ennemi « hostile » pour un charmé est un ennemi vivant, non charmé, hors salle dormante, autre que
//! lui-même. Le mouvement est une ligne droite (glissement sur les murs de `pathing::move_enemies`), pas
//! le champ de flux, qui ne mène qu'aux joueurs.

use bevy_fixed::fixed_math::{self, Fixed, FixedVec2};

use super::rules::BehaviorMotion;

/// Un ennemi que le charmé peut viser : `GgrsNetId.0` et position.
pub type CharmCandidate = (usize, FixedVec2);

/// L'ennemi le plus proche de `from` parmi `candidates` : distance au carré en `FixedWide` ; à distance
/// égale, le plus petit `GgrsNetId` (ordre des candidats indifférent). `None` sans candidat. Pure.
pub fn nearest_target(
    from: FixedVec2,
    candidates: impl IntoIterator<Item = CharmCandidate>,
) -> Option<CharmCandidate> {
    candidates
        .into_iter()
        .map(|(id, position)| ((from.distance_squared(&position), id), (id, position)))
        .min_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, candidate)| candidate)
}

/// Déplacement d'un charmé vers `target` : droit sur lui à pleine vitesse, à l'arrêt à moins de
/// `stop_distance` (portée de corps à corps) ou sans cible. Pure.
pub fn charm_motion(
    position: FixedVec2,
    target: Option<FixedVec2>,
    stop_distance: Fixed,
) -> BehaviorMotion {
    match target {
        Some(target) if position.distance(&target) >= stop_distance => BehaviorMotion {
            direction: (target - position).normalize_or_zero(),
            speed_mult: fixed_math::FIXED_ONE,
        },
        _ => BehaviorMotion {
            direction: FixedVec2::ZERO,
            speed_mult: fixed_math::FIXED_ZERO,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, y: f32) -> FixedVec2 {
        FixedVec2::new(fixed_math::new(x), fixed_math::new(y))
    }

    #[test]
    fn cible_l_ennemi_le_plus_proche() {
        let candidates = [
            (30, at(100.0, 0.0)),
            (12, at(0.0, -40.0)),
            (7, at(200.0, 0.0)),
        ];
        assert_eq!(
            nearest_target(at(0.0, 0.0), candidates),
            Some((12, at(0.0, -40.0)))
        );
        assert_eq!(nearest_target(at(0.0, 0.0), []), None);
    }

    /// Distances exactement égales : le plus petit net id, quel que soit l'ordre des candidats.
    #[test]
    fn egalite_va_au_plus_petit_net_id() {
        let forward = [(9, at(50.0, 0.0)), (4, at(-50.0, 0.0)), (6, at(0.0, 50.0))];
        let backward = [(6, at(0.0, 50.0)), (4, at(-50.0, 0.0)), (9, at(50.0, 0.0))];
        assert_eq!(nearest_target(at(0.0, 0.0), forward).map(|c| c.0), Some(4));
        assert_eq!(nearest_target(at(0.0, 0.0), backward).map(|c| c.0), Some(4));
    }

    /// Droit sur la cible, arrêt à portée, immobile sans cible.
    #[test]
    fn marche_droit_puis_s_arrete_a_portee() {
        let stop = fixed_math::new(30.0);
        let walking = charm_motion(at(0.0, 0.0), Some(at(100.0, 0.0)), stop);
        assert_eq!(walking.direction, at(1.0, 0.0));
        assert_eq!(walking.speed_mult, fixed_math::FIXED_ONE);
        let close = charm_motion(at(0.0, 0.0), Some(at(20.0, 0.0)), stop);
        assert_eq!(close.speed_mult, fixed_math::FIXED_ZERO);
        let lost = charm_motion(at(0.0, 0.0), None, stop);
        assert_eq!(lost.speed_mult, fixed_math::FIXED_ZERO);
        assert_eq!(lost.direction, FixedVec2::ZERO);
    }
}
