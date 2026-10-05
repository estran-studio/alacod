//! Mises en page RON du kind `Ui` qui ont un schéma typé partagé entre le lint et le jeu.
//!
//! T1.16 (`docs/conventions.md` §30) : `ui/mutation_screen.ron`, l'écran de mutation. Le jeu
//! (`game::ui::mutation_screen`) le charge tel quel ; le lint vérifie que la police existe sous
//! `assets/`, qu'il y a exactement trois emplacements de carte (bits `ChoiceA/B/C`) et que les
//! tailles sont positives.

use serde::{Deserialize, Serialize};

/// Nom du fichier de l'écran de mutation dans un dossier `Ui`.
pub const MUTATION_SCREEN_FILE_NAME: &str = "mutation_screen.ron";

/// Nombre de cartes de l'écran : une par bit `ChoiceA/B/C`.
pub const MUTATION_SCREEN_SLOTS: usize = 3;

/// Écran de mutation (`ui/mutation_screen.ron`). Positions en pixels depuis le centre de
/// l'écran (`x` vers la droite, `y` vers le bas) ; couleurs `#rrggbb` ou `#rrggbbaa`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationScreenLayout {
    /// Police de tous les textes, relative à `assets/` (doit exister : lint).
    pub font: String,
    pub title: String,
    pub title_offset: (f32, f32),
    pub title_size: f32,
    /// Taille d'une carte (largeur, hauteur).
    pub card_size: (f32, f32),
    /// Centre de chaque carte : exactement trois (lint).
    pub slots: Vec<(f32, f32)>,
    pub name_size: f32,
    pub text_size: f32,
    pub card_color: String,
    /// Bordure de la carte surlignée.
    pub highlight_color: String,
    pub text_color: String,
    /// Barre de temps : centre, taille à plein, couleur.
    pub bar_offset: (f32, f32),
    pub bar_size: (f32, f32),
    pub bar_color: String,
    /// Aide sous les cartes (« ← → choisir · A / Entrée valider · 1 2 3 »).
    pub hint: String,
    pub hint_offset: (f32, f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_screen_des_jeux() {
        for text in [
            include_str!("../../../games/testbed/assets/ui/mutation_screen.ron"),
            include_str!("../../../games/throne/assets/ui/mutation_screen.ron"),
        ] {
            let layout: MutationScreenLayout = ron::from_str(text).expect("mutation_screen.ron");
            assert_eq!(layout.slots.len(), MUTATION_SCREEN_SLOTS);
        }
    }
}
