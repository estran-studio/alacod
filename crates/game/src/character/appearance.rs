//! Apparence d'un personnage : donnée posée par la simulation à la création, dessinée par la
//! présentation (`character::visuals`). Composant rollback : `Hash` manuel, voir ci-dessous.

use bevy::prelude::*;

/// Apparence d'un personnage : donnée par la simulation, dessinée par la présentation.
#[derive(Component, Clone, Debug)]
pub struct CharacterAppearance {
    /// Clé de `CharacterConfigs` (le `CharacterId` du registre, l'`asset_name_ref`).
    pub config_name: String,
    /// Skin de la config ; `None` pour le skin de départ.
    pub skin: Option<String>,
    pub health_bar_color: Color,
}

/// Hash manuel : exclut `health_bar_color` (présentation, `Color` est `f32` et
/// n'implémente pas `Hash`) ; seuls `config_name`/`skin` (le choix de skin, qui affecte
/// bien la simulation via le spawn des layers) contribuent au checksum.
impl std::hash::Hash for CharacterAppearance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.config_name.hash(state);
        self.skin.hash(state);
    }
}
