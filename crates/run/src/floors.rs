//! Mode `Floors` (T1.8, chantier F1, `docs/plan-engine.md` §5 F1) : une séquence ordonnée de
//! niveaux (cartes LDtk, contenu `Floors` du manifeste), un portail qui s'ouvre quand le niveau
//! courant n'a plus d'ennemi, et le passage au niveau suivant **dans la simulation** (voir
//! `map_ldtk::game::floors`, qui détruit les entités du niveau quitté et crée celles du
//! suivant dans `GgrsSchedule`). Voir `docs/conventions.md` §17.
//!
//! Ce module ne porte que l'état rollback ([`FloorState`]) et les règles pures, testables
//! sans Bevy : les systèmes qui lisent les ennemis, les joueurs et les cartes vivent dans
//! `map_ldtk` (seul crate qui connaît à la fois `game` et `bevy_ecs_ldtk`).

use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};

/// Rayon (unités monde) dans lequel un joueur franchit un portail ouvert. Un peu plus grand
/// que le collider d'un joueur (20×20) : il suffit de marcher sur le portail.
pub const PORTAL_RADIUS: Fixed = Fixed::from_bits(24 << 16);

/// État du mode `Floors` : ressource rollback (checksum + trace, enregistrée par
/// [`crate::RunPlugin`] avec un checksum **neutre** : la valeur par défaut — toute partie
/// d'un autre mode — contribue `0` au checksum GGRS, voir
/// `utils::rollback::RollbackTraceApp::rollback_and_trace_resource_neutral`). Hors du mode
/// `Floors`, elle reste à sa valeur par défaut et aucun système n'y écrit.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FloorState {
    /// Index du niveau courant (attente de scénario `FloorIndex`), `0` au départ ; ne fait
    /// que croître (boucle infinie au dernier niveau, voir [`level_for_floor`]).
    pub index: u32,
    /// Position du portail du niveau courant (`x`, `y`, unités monde) : barycentre des
    /// `PlayerSpawn` du niveau, posé au chargement du niveau. `None` hors mode `Floors`.
    pub anchor: Option<(Fixed, Fixed)>,
    /// Portail ouvert : posé quand le niveau n'a plus d'ennemi ([`portal_should_open`]),
    /// remis à `false` au passage au niveau suivant.
    pub portal_open: bool,
    /// Ennemis placés par les niveaux chargés jusqu'ici (cumulé) : `kills` du résumé =
    /// ce total moins les ennemis encore en vie.
    pub enemies_placed: u32,
}

impl FloorState {
    /// Position du portail en vecteur fixe, si le niveau en a une.
    pub fn anchor_vec(&self) -> Option<FixedVec2> {
        self.anchor.map(|(x, y)| FixedVec2::new(x, y))
    }
}

/// Niveau de la séquence à jouer pour l'index `index` : la séquence se joue dans l'ordre,
/// puis **boucle sur le dernier niveau** (décision T1.8, `docs/conventions.md` §17 : pas de
/// victoire en `Floors`, la partie ne finit que par la défaite). `len == 0` n'arrive pas
/// (le lint refuse une séquence vide) : `0` par sécurité.
pub fn level_for_floor(index: u32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (index as usize).min(len - 1)
}

/// Le portail s'ouvre quand le niveau courant a une position de portail, que le portail est
/// encore fermé et qu'il ne reste aucun ennemi (`EntityCount(enemy) == 0`).
pub fn portal_should_open(state: &FloorState, enemies_alive: usize) -> bool {
    state.anchor.is_some() && !state.portal_open && enemies_alive == 0
}

/// Un joueur à `position` franchit le portail ouvert de `state` (distance au carré, en
/// `Fixed`, comparée au carré de [`PORTAL_RADIUS`] : pas de racine).
pub fn crosses_portal(state: &FloorState, position: FixedVec2) -> bool {
    if !state.portal_open {
        return false;
    }
    let Some(anchor) = state.anchor_vec() else {
        return false;
    };
    let delta = position - anchor;
    // Hors du carré englobant : pas de produit (évite un débordement 16.16 pour un joueur loin).
    if delta.x.abs() > PORTAL_RADIUS || delta.y.abs() > PORTAL_RADIUS {
        return false;
    }
    delta.x * delta.x + delta.y * delta.y <= PORTAL_RADIUS * PORTAL_RADIUS
}

/// Barycentre de positions (portail d'un niveau : centre des `PlayerSpawn`, donc de la salle
/// de départ). `None` si la liste est vide. Somme en `Fixed`, ordre indifférent.
pub fn barycenter(points: &[FixedVec2]) -> Option<(Fixed, Fixed)> {
    if points.is_empty() {
        return None;
    }
    let n = Fixed::from_num(points.len() as i32);
    let (sx, sy) = points
        .iter()
        .fold((Fixed::ZERO, Fixed::ZERO), |(sx, sy), p| {
            (sx + p.x / n, sy + p.y / n)
        });
    Some((sx, sy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::new as fx;

    fn open_at(x: f32, y: f32) -> FloorState {
        FloorState {
            anchor: Some((fx(x), fx(y))),
            portal_open: true,
            ..Default::default()
        }
    }

    #[test]
    fn la_sequence_boucle_sur_le_dernier_niveau() {
        assert_eq!(level_for_floor(0, 2), 0);
        assert_eq!(level_for_floor(1, 2), 1);
        assert_eq!(level_for_floor(2, 2), 1);
        assert_eq!(level_for_floor(57, 3), 2);
        assert_eq!(level_for_floor(4, 0), 0);
    }

    #[test]
    fn portail_ouvert_seulement_sans_ennemi_et_avec_position() {
        let state = FloorState {
            anchor: Some((fx(0.0), fx(0.0))),
            ..Default::default()
        };
        assert!(!portal_should_open(&state, 1));
        assert!(portal_should_open(&state, 0));
        // Déjà ouvert : rien à faire.
        assert!(!portal_should_open(&open_at(0.0, 0.0), 0));
        // Hors mode Floors (valeur par défaut) : jamais.
        assert!(!portal_should_open(&FloorState::default(), 0));
    }

    #[test]
    fn franchissement_dans_le_rayon() {
        let state = open_at(100.0, 50.0);
        assert!(crosses_portal(&state, FixedVec2::new(fx(100.0), fx(50.0))));
        assert!(crosses_portal(&state, FixedVec2::new(fx(120.0), fx(50.0))));
        assert!(!crosses_portal(&state, FixedVec2::new(fx(125.0), fx(50.0))));
        // Diagonale : 20² + 20² = 800 > 576.
        assert!(!crosses_portal(&state, FixedVec2::new(fx(120.0), fx(70.0))));
        // Loin : pas de débordement.
        assert!(!crosses_portal(
            &state,
            FixedVec2::new(fx(30000.0), fx(-30000.0))
        ));
        // Fermé : jamais.
        let closed = FloorState {
            portal_open: false,
            ..state
        };
        assert!(!crosses_portal(
            &closed,
            FixedVec2::new(fx(100.0), fx(50.0))
        ));
    }

    #[test]
    fn barycentre_des_points_de_depart() {
        assert_eq!(barycenter(&[]), None);
        let points = [
            FixedVec2::new(fx(0.0), fx(0.0)),
            FixedVec2::new(fx(32.0), fx(0.0)),
            FixedVec2::new(fx(0.0), fx(16.0)),
            FixedVec2::new(fx(32.0), fx(16.0)),
        ];
        assert_eq!(barycenter(&points), Some((fx(16.0), fx(8.0))));
    }
}
