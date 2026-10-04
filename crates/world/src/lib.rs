//! Monde à cellules (T1.0b + T1.6, chantier E3, `docs/conventions.md` §21) : grille de terrain
//! rollback, cavernes générées par automate cellulaire et destruction par explosion.
//!
//! Une caverne passe par le chemin LDtk ordinaire (`map::generation`, mode `Cave`) : colliders,
//! navigation et points de départ en sortent comme pour toute carte. Ce crate ne porte que la
//! grille, le générateur pur et la règle de destruction ; le branchement ECS (remplissage au
//! chargement, reconstruction des murs) vit dans `map_ldtk`.

pub mod cave;
pub mod destroy;
pub mod grid;

pub use cave::{generate, points_of_interest, CaveConfig, CavePoints};
pub use destroy::destroy_terrain;
pub use grid::{CellGrid, CellKind, Destructible, CELL_SIZE};
