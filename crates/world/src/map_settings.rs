//! Réglages de l'assemblage de pièces (`BasicMapGeneration`, `docs/conventions.md` §1), par jeu :
//! champ `entry.map` de `game.ron`, recopiés dans `MapGenerationConfig` au lancement. Donnée pure.

use serde::{Deserialize, Serialize};

fn default_max_rooms() -> usize {
    10
}
fn default_bound() -> i32 {
    1000
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct MapSettings {
    /// Nombre de pièces au plus (départ comprise) ; `0` = sans plafond.
    #[serde(default = "default_max_rooms")]
    pub max_rooms: usize,
    /// Demi-largeur et demi-hauteur de la carte en pixels autour de l'origine : une pièce dont
    /// le coin haut-gauche sort de ±`max_width` × ±`max_height` n'est pas posée.
    #[serde(default = "default_bound")]
    pub max_width: i32,
    #[serde(default = "default_bound")]
    pub max_height: i32,
    /// `true` : deux prises s'emboîtent seulement au même décalage le long du bord (règle
    /// d'origine : pièces alignées coin haut-gauche). `false` : côté opposé et même longueur
    /// suffisent, la pièce est décalée pour faire coïncider les prises.
    #[serde(default = "default_true")]
    pub aligned_openings: bool,
    /// Après le placement, deux pièces posées bord à bord dont des prises libres se font face
    /// (même emprise dans le monde) sont reliées : portes appariées, boucles dans la carte.
    #[serde(default = "default_true")]
    pub close_loops: bool,
    /// Une prise restée sans voisine est murée (cases `Walls`) et sa porte n'est pas posée ;
    /// `false` : la porte est posée non interactive, l'ouverture reste dessinée.
    #[serde(default = "default_true")]
    pub wall_unused_openings: bool,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            max_rooms: default_max_rooms(),
            max_width: default_bound(),
            max_height: default_bound(),
            aligned_openings: true,
            close_loops: true,
            wall_unused_openings: true,
        }
    }
}
