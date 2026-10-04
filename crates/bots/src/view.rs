//! Vue en fixed-point de l'état lisible par un joueur, construite par
//! [`crate::input::read_bot_inputs`] et consommée par [`crate::decide::decide`]. Pas de
//! dépendance à Bevy ECS : uniquement des données, en `Fixed` (jamais `f32`).

use bevy_fixed::fixed_math::{Fixed, FixedVec2};

/// Ennemi le plus proche du joueur, vu par un bot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnemyView {
    pub position: FixedVec2,
    /// Distance au joueur (`FixedVec2::distance`, jamais recalculée dans `decide`).
    pub distance: Fixed,
}

/// Fenêtre la plus proche du joueur, vue par un bot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowView {
    pub position: FixedVec2,
    /// Santé courante (`WindowHealth::current`, 0..=max).
    pub health: u8,
    /// Distance au joueur.
    pub distance: Fixed,
    /// `true` si la fenêtre a perdu de la santé (`current < max`) : ça vaut la peine de la
    /// réparer. Ne dépend pas du cooldown de réparation (`WindowHealth::can_repair_after_frame`) :
    /// un bot qui maintient l'interaction pendant le cooldown ne fait juste rien ce temps-là.
    pub repairable: bool,
}

/// Vue d'un joueur sur son propre état et son entourage immédiat, en fixed-point. Construite
/// une fois par joueur local piloté par un bot, à chaque frame de `ReadInputs`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BotView {
    pub position: FixedVec2,
    pub health: Fixed,
    pub health_max: Fixed,
    /// Munitions dans le chargeur de l'arme active (0 si aucune arme).
    pub ammo: u32,
    /// Vague courante (`WaveState::current_wave`).
    pub wave: u32,
    pub nearest_enemy: Option<EnemyView>,
    pub nearest_window: Option<WindowView>,
    /// Navigation et interactions des profils v1 ; absent pour les profils v0.
    pub hunter: Option<crate::hunter::HunterView>,
    /// Portail ouvert du mode `Floors` (T1.8, `run::FloorState`), `None` sinon (portail fermé,
    /// ou autre mode) : un bot sans ennemi s'y rend pour passer au niveau suivant.
    pub portal: Option<FixedVec2>,
}

/// Choisit l'élément le plus proche parmi `candidates` (distance, puis `GgrsNetId.0` comme
/// tie-break déterministe, CLAUDE.md règle 4 : jamais le premier trouvé dans un ordre
/// arbitraire). Le résultat ne dépend pas de l'ordre d'itération de `candidates` : ce runner
/// trie quand même ses requêtes par `GgrsNetId` avant d'appeler cette fonction (convention du
/// projet, `order_iter!`), mais cette fonction elle-même n'en a pas besoin pour être correcte.
pub fn nearest_by_net_id<T: Copy>(
    candidates: impl Iterator<Item = (usize, Fixed, T)>,
) -> Option<T> {
    let mut best: Option<(usize, Fixed, T)> = None;
    for (net_id, distance, value) in candidates {
        let better = match &best {
            None => true,
            Some((best_net_id, best_distance, _)) => {
                distance < *best_distance || (distance == *best_distance && net_id < *best_net_id)
            }
        };
        if better {
            best = Some((net_id, distance, value));
        }
    }
    best.map(|(_, _, value)| value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    #[test]
    fn nearest_by_net_id_picks_closest() {
        let candidates = vec![(5usize, fx(100.0), "far"), (2usize, fx(10.0), "near")];
        assert_eq!(
            nearest_by_net_id(candidates.into_iter()),
            Some("near"),
            "l'élément à distance 10 doit gagner sur celui à distance 100"
        );
    }

    #[test]
    fn nearest_by_net_id_breaks_ties_by_net_id() {
        // Même distance : le plus petit GgrsNetId gagne, quel que soit l'ordre d'itération.
        let a = vec![(9usize, fx(50.0), "a"), (3usize, fx(50.0), "b")];
        let b = vec![(3usize, fx(50.0), "b"), (9usize, fx(50.0), "a")];
        assert_eq!(nearest_by_net_id(a.into_iter()), Some("b"));
        assert_eq!(nearest_by_net_id(b.into_iter()), Some("b"));
    }

    #[test]
    fn nearest_by_net_id_empty_is_none() {
        let empty: Vec<(usize, Fixed, ())> = vec![];
        assert_eq!(nearest_by_net_id(empty.into_iter()), None);
    }
}
