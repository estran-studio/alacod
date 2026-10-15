//! Rochers des salles (M3-T0c, `docs/conventions.md` §62) : obstacles destructibles posés par la
//! couche IntGrid optionnelle `Rocks` d'un gabarit LDtk.
//!
//! **Une entité par rocher**, pas un remplissage de [`crate::CellGrid`] : cette grille est dense,
//! ancrée à l'origine et non négative (une caverne, un niveau), et son système de reconstruction
//! détruit *tous* les murs pour les recréer depuis la grille. Sur un étage assemblé (salles à
//! des positions quelconques, murs IntGrid propres à chaque salle) elle effacerait les murs des
//! salles. Un rocher-entité porte son collider et son `Obstacle` (le champ de flux le voit sans
//! code de plus) et disparaît par `despawn_rollback` : c'est ce que M3-T4 (explosion
//! `destroys`) fera. Aucun rocher hors d'une carte avec la couche : le composant est neutre.

use bevy::prelude::*;

/// Nom de la couche IntGrid des rochers dans un niveau LDtk.
pub const LAYER_ROCKS: &str = "Rocks";
/// Valeur IntGrid d'un rocher ordinaire.
pub const ROCK_VALUE: i32 = 1;
/// Valeur IntGrid d'un rocher à butin (le butin vient de M3-T4).
pub const ROCK_LOOT_VALUE: i32 = 2;

/// Un rocher : une case de 16 unités. Rollback + trace en variante neutre.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rock {
    /// Case de grille monde `(x, y)` (+y vers le haut), pour le tri et les attentes.
    pub cell: (i32, i32),
    /// Rocher à butin (valeur IntGrid 2).
    pub loot: bool,
}

/// Rochers d'une couche IntGrid : `(col, row)` de la couche (rangées LDtk, de haut en bas) et
/// `loot`, dans l'ordre de lecture (rangée puis colonne). Toute autre valeur est ignorée.
pub fn rocks_of_layer(int_grid_csv: &[i32], width: usize) -> Vec<(usize, usize, bool)> {
    if width == 0 {
        return Vec::new();
    }
    int_grid_csv
        .iter()
        .enumerate()
        .filter_map(|(i, &value)| match value {
            ROCK_VALUE => Some((i % width, i / width, false)),
            ROCK_LOOT_VALUE => Some((i % width, i / width, true)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_de_la_couche() {
        // 3 colonnes, 2 rangées
        let csv = [0, 1, 0, 2, 0, 7];
        assert_eq!(rocks_of_layer(&csv, 3), vec![(1, 0, false), (0, 1, true)]);
        assert!(rocks_of_layer(&csv, 0).is_empty());
        assert!(rocks_of_layer(&[0, 0], 2).is_empty());
    }
}
