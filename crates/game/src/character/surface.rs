//! Surfaces sous les pieds (T1.7, chantier E4 v1, `docs/conventions.md` §26) : modificateurs de
//! vitesse posés par la case de `world::SurfaceGrid` que le personnage occupe.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedVec2};
use bevy_ggrs::Rollback;
use sim_core::modifier::{Modifier, Modifiers};
use utils::net_id::GgrsNetId;
use utils::order_mut_iter;
use world::surface::{
    surface_modifier_source, surface_modifiers, SurfaceGrid, SurfaceTable, Walker,
};

use crate::character::enemy::ai::state::{EnemyAiConfig, MovementType};
use crate::character::player::Player;
use crate::collider::Collider;

/// Point de contact au sol : centre du collider, offset compris.
pub fn feet_position(transform: &fixed_math::FixedTransform3D, collider: &Collider) -> FixedVec2 {
    (transform.translation + collider.offset).truncate()
}

/// Qui marche : joueur, ennemi au sol, ou personne (ennemi volant, personnage sans IA).
pub fn walker_of(is_player: bool, ai: Option<&EnemyAiConfig>) -> Option<Walker> {
    if is_player {
        return Some(Walker::Player);
    }
    match ai {
        Some(ai) if ai.movement_type != MovementType::Flying => Some(Walker::Enemy),
        _ => None,
    }
}

/// Remplace exactement les modificateurs de source `surface` par `wanted` si et seulement si
/// ils diffèrent (rien n'est écrit quand la surface ne change pas). Rend vrai si `modifiers`
/// a changé.
pub fn apply_surface(modifiers: &mut Modifiers, wanted: Vec<Modifier>) -> bool {
    let source = surface_modifier_source();
    let current: Vec<&Modifier> = modifiers.iter().filter(|m| m.source == source).collect();
    if current.len() == wanted.len() && current.iter().zip(&wanted).all(|(a, b)| *a == b) {
        return false;
    }
    modifiers.remove_by_source(&source);
    for modifier in wanted {
        modifiers.push(modifier);
    }
    true
}

/// Dans `RollbackSystemSet::Input`, avant `apply_inputs` : effet dans la frame. Sur une carte
/// sans surface, ne fait que retirer d'éventuels restes (joueur passé d'un niveau `Floors` à
/// surfaces vers un niveau sans) ; un personnage hors surface n'est jamais touché.
#[allow(clippy::type_complexity)]
pub fn surface_modifiers_system(
    grid: Res<SurfaceGrid>,
    table: Option<Res<SurfaceTable>>,
    mut characters: Query<
        (
            &GgrsNetId,
            &fixed_math::FixedTransform3D,
            &Collider,
            &mut Modifiers,
            Has<Player>,
            Option<&EnemyAiConfig>,
        ),
        With<Rollback>,
    >,
) {
    let source = surface_modifier_source();
    if grid.is_empty() {
        for (_, _, _, mut modifiers, _, _) in order_mut_iter!(characters) {
            if modifiers.iter().any(|m| m.source == source) {
                modifiers.remove_by_source(&source);
            }
        }
        return;
    }
    for (net_id, transform, collider, mut modifiers, is_player, ai) in order_mut_iter!(characters) {
        let wanted = walker_of(is_player, ai)
            .and_then(|walker| {
                let id = grid.at(feet_position(transform, collider))?;
                let def = table.as_ref()?.0.get(&id)?;
                Some(surface_modifiers(def, walker))
            })
            .unwrap_or_default();
        // `Mut` : ne marquer le composant changé que si on écrit vraiment
        if modifiers
            .bypass_change_detection()
            .iter()
            .any(|m| m.source == source)
            || !wanted.is_empty()
        {
            if apply_surface(&mut modifiers, wanted) {
                debug!("surface : modificateurs de {} remplacés", net_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::Fixed;
    use sim_core::modifier::{ModifierOp, ModifierSource};
    use sim_core::stats::StatId;
    use world::SurfaceDef;

    fn def(move_speed: f32) -> SurfaceDef {
        SurfaceDef {
            name: "s".into(),
            tags: vec!["s".into()],
            move_speed: Fixed::from_num(move_speed),
            acceleration: Fixed::ONE,
        }
    }

    #[test]
    fn remplacement_exact_et_rien_si_inchange() {
        let other = Modifier {
            stat: StatId::MoveSpeed,
            op: ModifierOp::Mul,
            value: Fixed::from_num(0.25),
            source: ModifierSource::Named("downed".into()),
            until: None,
        };
        let mut modifiers = Modifiers(vec![other.clone()]);
        let eau = surface_modifiers(&def(0.5), Walker::Player);
        assert!(apply_surface(&mut modifiers, eau.clone()));
        assert_eq!(modifiers.0, vec![other.clone(), eau[0].clone()]);
        // Même surface : rien n'est écrit
        assert!(!apply_surface(&mut modifiers, eau.clone()));
        // Autre surface : remplacée, l'autre source intacte
        let sable = surface_modifiers(&def(0.8), Walker::Player);
        assert!(apply_surface(&mut modifiers, sable.clone()));
        assert_eq!(modifiers.0, vec![other.clone(), sable[0].clone()]);
        // Hors surface : retirée
        assert!(apply_surface(&mut modifiers, vec![]));
        assert_eq!(modifiers.0, vec![other]);
        assert!(!apply_surface(&mut modifiers, vec![]));
    }

    #[test]
    fn volant_ignore_les_surfaces() {
        let mut ai = EnemyAiConfig::default();
        assert_eq!(walker_of(true, None), Some(Walker::Player));
        assert_eq!(walker_of(false, None), None);
        assert_eq!(walker_of(false, Some(&ai)), Some(Walker::Enemy));
        ai.movement_type = MovementType::Flying;
        assert_eq!(walker_of(false, Some(&ai)), None);
    }

    #[test]
    fn pieds_au_centre_du_collider() {
        let transform = fixed_math::FixedTransform3D::new(
            fixed_math::FixedVec3::new(Fixed::from_num(100), Fixed::from_num(100), Fixed::ZERO),
            fixed_math::FixedMat3::IDENTITY,
            fixed_math::FixedVec3::ONE,
        );
        let collider = Collider {
            shape: crate::collider::ColliderShape::Circle {
                radius: Fixed::from_num(5),
            },
            offset: fixed_math::FixedVec3::new(Fixed::ZERO, Fixed::from_num(-6), Fixed::ZERO),
        };
        assert_eq!(
            feet_position(&transform, &collider),
            FixedVec2::new(Fixed::from_num(100), Fixed::from_num(94))
        );
    }
}
