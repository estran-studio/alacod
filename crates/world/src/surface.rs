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
    /// M2-T11 (§46) : la surface est une fosse. `None` pour toute surface de mouvement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pit: Option<PitDef>,
}

/// Fosse (M2-T11, `docs/conventions.md` §46) : un joueur dont les pieds y entrent perd
/// `fall_damage` PV et revient à sa dernière position sûre ; les ennemis au sol ne la traversent
/// pas (champ de flux, déplacement), les volants la survolent.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitDef {
    #[cfg_attr(feature = "schema", schemars(with = "bevy_fixed::schema::FixedString"))]
    pub fall_damage: Fixed,
    /// Où revient le joueur tombé.
    #[serde(default)]
    pub respawn: PitRespawn,
    /// Un dash en cours traverse la fosse sans y tomber (il tombe s'il finit dessus).
    #[serde(default)]
    pub dash_crosses: bool,
}

/// Point de retour d'un joueur tombé dans une fosse.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PitRespawn {
    /// Dernière position dont la case et ses 8 voisines étaient sans fosse
    /// (`world::props::PitSafePosition`).
    #[default]
    LastSafeCell,
}

impl SurfaceTable {
    /// Définition de fosse de la surface `id`, s'il y en a une.
    pub fn pit(&self, id: SurfaceId) -> Option<&PitDef> {
        self.0.get(&id)?.pit.as_ref()
    }

    /// Le jeu a au moins une surface fosse.
    pub fn has_pits(&self) -> bool {
        self.0.values().any(|def| def.pit.is_some())
    }
}

impl SurfaceGrid {
    /// Fosse sous `position` (monde), s'il y en a une.
    pub fn pit_at<'a>(&self, table: &'a SurfaceTable, position: FixedVec2) -> Option<&'a PitDef> {
        table.pit(self.at(position)?)
    }

    /// Case `(x, y)` sûre : ni elle ni ses 8 voisines ne sont une fosse.
    pub fn is_safe_cell(&self, table: &SurfaceTable, x: i32, y: i32) -> bool {
        (-1..=1).all(|dx| {
            (-1..=1).all(|dy| {
                self.get(x + dx, y + dy)
                    .is_none_or(|id| table.pit(id).is_none())
            })
        })
    }

    /// Cases de fosse de la grille, dans l'ordre de la grille (déterministe).
    pub fn pit_cells<'a>(
        &'a self,
        table: &'a SurfaceTable,
    ) -> impl Iterator<Item = (i32, i32)> + 'a {
        self.cells
            .iter()
            .filter(|(_, id)| table.pit(**id).is_some())
            .map(|(cell, _)| *cell)
    }
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
            pit: None,
        }
    }

    #[test]
    fn fosse_et_cases_sures() {
        let mut table = SurfaceTable::default();
        table.0.insert(1, eau());
        let mut fosse = eau();
        fosse.name = "fosse".into();
        fosse.pit = Some(PitDef {
            fall_damage: Fixed::from_num(10),
            respawn: PitRespawn::LastSafeCell,
            dash_crosses: true,
        });
        table.0.insert(2, fosse);
        assert!(table.has_pits());
        let mut grid = SurfaceGrid::default();
        grid.cells.insert((5, 5), 2);
        grid.cells.insert((0, 0), 1);
        assert!(grid
            .pit_at(
                &table,
                FixedVec2::new(Fixed::from_num(88), Fixed::from_num(88))
            )
            .is_some());
        assert!(grid
            .pit_at(
                &table,
                FixedVec2::new(Fixed::from_num(8), Fixed::from_num(8))
            )
            .is_none());
        assert!(!grid.is_safe_cell(&table, 4, 4));
        assert!(!grid.is_safe_cell(&table, 6, 5));
        assert!(grid.is_safe_cell(&table, 7, 5));
        assert!(grid.is_safe_cell(&table, 0, 1), "l'eau n'est pas une fosse");
        assert_eq!(grid.pit_cells(&table).collect::<Vec<_>>(), vec![(5, 5)]);
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
            pit: None,
        };
        let player = surface_modifiers(&glace, Walker::Player);
        assert_eq!(player.len(), 1);
        assert_eq!(player[0].stat, StatId::Acceleration);
        assert!(surface_modifiers(&glace, Walker::Enemy).is_empty());
    }
}
