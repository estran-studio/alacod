//! Mises en page RON du kind `Ui` qui ont un schéma typé partagé entre le lint et le jeu.
//!
//! `ui/hud.ron` (conventions §15, §32) : le lint lit la source de chaque widget
//! ([`HudFileSchema`], les autres champs sont ceux de `game::ui::hud::HudConfig`) et refuse
//! une source absente de [`HUD_SOURCES`] (`UnknownKind`).
//!
//! T1.16 (`docs/conventions.md` §30) : `ui/mutation_screen.ron`, l'écran de mutation. Le jeu
//! (`game::ui::mutation_screen`) le charge tel quel ; le lint vérifie que la police existe sous
//! `assets/`, qu'il y a exactement trois emplacements de carte (bits `ChoiceA/B/C`) et que les
//! tailles sont positives.

use serde::{Deserialize, Serialize};

/// Nom du fichier du HUD dans un dossier `Ui`.
pub const HUD_FILE_NAME: &str = "hud.ron";

/// Sources que le HUD sait lire (liste fermée). T2.12 : `perks`, `downed`, `powerups`,
/// `prompt` ; T1.18 (§32) : `rads`, `level`, `ammo_by_type`, `statuses`, `floor`.
pub const HUD_SOURCES: &[&str] = &[
    "health",
    "wave",
    "ammo",
    "weapon",
    "enemies",
    "players",
    "currency",
    "perks",
    "downed",
    "powerups",
    "prompt",
    "rads",
    "level",
    "ammo_by_type",
    "statuses",
    "floor",
];

/// Ce que le lint lit de `ui/hud.ron` : la source de chaque widget.
#[derive(Debug, Clone, Deserialize)]
pub struct HudFileSchema {
    pub widgets: Vec<HudWidgetSchema>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HudWidgetSchema {
    pub kind: HudWidgetKindSchema,
}

#[derive(Debug, Clone, Deserialize)]
pub enum HudWidgetKindSchema {
    Bar {
        source: String,
    },
    Text {
        source: String,
        #[serde(default)]
        #[allow(dead_code)]
        prefix: Option<String>,
    },
    Icons {
        source: String,
    },
}

impl HudWidgetKindSchema {
    pub fn source(&self) -> &str {
        match self {
            Self::Bar { source } | Self::Text { source, .. } | Self::Icons { source } => source,
        }
    }
}

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

    #[test]
    fn hud_des_jeux_sources_connues() {
        for text in [
            include_str!("../../../games/testbed/assets/ui/hud.ron"),
            include_str!("../../../games/throne/assets/ui/hud.ron"),
            include_str!("../../../games/zombies/assets/ui/hud.ron"),
        ] {
            let hud: HudFileSchema = ron::from_str(text).expect("hud.ron");
            for widget in &hud.widgets {
                assert!(
                    HUD_SOURCES.contains(&widget.kind.source()),
                    "{:?}",
                    widget.kind
                );
            }
        }
    }
}
