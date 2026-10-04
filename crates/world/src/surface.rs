//! Surfaces (T1.7, chantier E4 v1, `docs/conventions.md` §26) : tags de cases qui modifient le
//! mouvement des personnages par des modificateurs de stats.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use serde::{Deserialize, Serialize};
use sim_core::modifier::{Modifier, ModifierOp, ModifierSource};
use sim_core::stats::StatId;

use crate::grid::CellGrid;

/// Identifiant d'une surface : la valeur IntGrid de la couche LDtk `Surfaces` (non nulle).
pub type SurfaceId = u8;

/// Couche IntGrid optionnelle des cartes LDtk qui porte les surfaces.
pub const LAYER_SURFACES: &str = "Surfaces";

/// Source des modificateurs posés par une surface (`Modifiers::remove_by_source`).
pub fn surface_modifier_source() -> ModifierSource {
    ModifierSource::Named("surface".into())
}

/// Surfaces de la carte courante, **creuse** : case de grille monde (16 unités, +y vers le
/// haut, même découpage que `CellGrid`, sans origine imposée) → surface. Ressource rollback,
/// checksum neutre : vide (défaut) sur toute carte sans couche `Surfaces`, traces inchangées.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SurfaceGrid {
    pub cells: BTreeMap<(i32, i32), SurfaceId>,
}

impl SurfaceGrid {
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn get(&self, x: i32, y: i32) -> Option<SurfaceId> {
        self.cells.get(&(x, y)).copied()
    }

    /// Surface de la case qui contient `position` (monde).
    pub fn at(&self, position: FixedVec2) -> Option<SurfaceId> {
        let (x, y) = CellGrid::cell_of(position);
        self.get(x, y)
    }
}

/// Effets d'une surface sur le mouvement : facteurs abstraits (`ModifierOp::Mul`), traduits
/// vers la stat du personnage (voir [`surface_modifiers`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceDef {
    pub name: String,
    pub tags: Vec<String>,
    /// Vitesse maximale (`MoveSpeed` d'un joueur, `EnemyMoveSpeed` d'un ennemi).
    pub move_speed: Fixed,
    /// Accélération du joueur (`Acceleration`) ; les ennemis n'en ont pas.
    pub acceleration: Fixed,
}

/// Surfaces du jeu par identifiant (contenu, kind `Surface`). Hors rollback (donnée de
/// contenu, comme `Assets`), posée au chargement de la carte.
#[derive(Resource, Clone, Debug, Default)]
pub struct SurfaceTable(pub BTreeMap<SurfaceId, SurfaceDef>);

/// Qui marche sur la surface : la stat de vitesse visée en dépend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Walker {
    Player,
    /// Ennemi au sol (un ennemi volant ignore les surfaces : pas de modificateur).
    Enemy,
}

/// Modificateurs voulus pour un personnage sur une surface (facteur `1` : rien), source
/// [`surface_modifier_source`], sans `until`. Ordre fixe : vitesse puis accélération.
pub fn surface_modifiers(def: &SurfaceDef, walker: Walker) -> Vec<Modifier> {
    let mut out = Vec::new();
    let mut push = |stat: StatId, value: Fixed| {
        if value != Fixed::ONE {
            out.push(Modifier {
                stat,
                op: ModifierOp::Mul,
                value,
                source: surface_modifier_source(),
                until: None,
            });
        }
    };
    match walker {
        Walker::Player => {
            push(StatId::MoveSpeed, def.move_speed);
            push(StatId::Acceleration, def.acceleration);
        }
        Walker::Enemy => push(StatId::EnemyMoveSpeed, def.move_speed),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eau() -> SurfaceDef {
        SurfaceDef {
            name: "eau".into(),
            tags: vec!["eau".into()],
            move_speed: Fixed::from_num(0.5),
            acceleration: Fixed::ONE,
        }
    }

    #[test]
    fn grille_creuse_origine_quelconque() {
        let mut grid = SurfaceGrid::default();
        assert!(grid.is_empty());
        grid.cells.insert((-3, 7), 2);
        assert_eq!(grid.get(-3, 7), Some(2));
        // Case (-3, 7) = monde [-48, -32) × [112, 128)
        assert_eq!(
            grid.at(FixedVec2::new(Fixed::from_num(-40), Fixed::from_num(120))),
            Some(2)
        );
        assert_eq!(
            grid.at(FixedVec2::new(Fixed::from_num(-32), Fixed::from_num(120))),
            None
        );
        assert_eq!(
            SurfaceGrid::default(),
            SurfaceGrid::default(),
            "vide = défaut (neutre)"
        );
    }

    #[test]
    fn traduction_vers_la_stat_du_personnage() {
        let player = surface_modifiers(&eau(), Walker::Player);
        assert_eq!(player.len(), 1, "accélération 1 : pas de modificateur");
        assert_eq!(player[0].stat, StatId::MoveSpeed);
        assert_eq!(player[0].op, ModifierOp::Mul);
        assert_eq!(player[0].until, None);
        let enemy = surface_modifiers(&eau(), Walker::Enemy);
        assert_eq!(enemy.len(), 1);
        assert_eq!(enemy[0].stat, StatId::EnemyMoveSpeed);
        let glace = SurfaceDef {
            name: "glace".into(),
            tags: vec!["glace".into()],
            move_speed: Fixed::ONE,
            acceleration: Fixed::from_num(0.2),
        };
        let player = surface_modifiers(&glace, Walker::Player);
        assert_eq!(player.len(), 1);
        assert_eq!(player[0].stat, StatId::Acceleration);
        assert!(surface_modifiers(&glace, Walker::Enemy).is_empty());
    }
}
