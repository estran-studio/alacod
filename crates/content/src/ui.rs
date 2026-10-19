//! Mises en page RON du kind `Ui` qui ont un schéma typé partagé entre le lint et le jeu.
//!
//! `ui/hud.ron` (conventions §15, §32) : le lint lit la source de chaque widget
//! ([`HudFileSchema`], les autres champs sont ceux de `game::ui::hud::HudConfig`) et refuse
//! une source absente de [`HUD_SOURCES`] (`UnknownKind`).
//!
//! L'écran de mutation (T1.16) n'a plus de schéma RON : c'est un gabarit `bevy_hui`
//! (`ui/mutation_screen.html`, D71, `docs/conventions.md` §56), lu comme les autres `.html`.

use serde::{Deserialize, Serialize};

/// Nom du fichier du HUD dans un dossier `Ui`.
pub const HUD_FILE_NAME: &str = "hud.ron";

/// Sources que le HUD sait lire (liste fermée). T2.12 : `perks`, `downed`, `powerups`,
/// `prompt` ; T1.18 (§32) : `rads`, `level`, `ammo_by_type`, `statuses`, `floor` ; M3-T16 (§81) :
/// `hearts`, `coins`, `item_banner`.
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
    "blanks",
    "keys",
    "bombs",
    "shells",
    "passives",
    "active_item",
    "active_charge",
    "minimap",
    "curses",
    "achievements",
    "legend",
    "hearts",
    "coins",
    "item_banner",
];

/// Ce que le lint lit de `ui/hud.ron` : la source de chaque widget.
#[derive(Debug, Clone, Deserialize)]
pub struct HudFileSchema {
    pub widgets: Vec<HudWidgetSchema>,
    /// Légende (source `legend`, `docs/conventions.md` §83) : le lint vérifie que chaque `sprite`
    /// cité existe dans la table `SpriteSheet` du jeu.
    #[serde(default)]
    pub legend: Vec<LegendGroupSchema>,
    /// Dessin des murs (`game::ui::hud::WallArt`, §83) : le lint vérifie que la feuille existe.
    #[serde(default)]
    pub wall_art: Option<WallArtSchema>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WallArtSchema {
    pub layer: String,
    pub atlas: String,
    #[serde(default)]
    pub tile: Option<u32>,
    #[serde(default)]
    pub columns: Option<u32>,
    /// Cases d'autres couches IntGrid dessinées par une tuile fixe (m3-salles-murs-isaac) : le lint vérifie leur couche.
    #[serde(default)]
    pub cells: Vec<CellArtSchema>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CellArtSchema {
    pub layer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegendGroupSchema {
    pub entries: Vec<LegendEntrySchema>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegendEntrySchema {
    #[serde(default)]
    pub sprite: Option<String>,
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
    Map {
        source: String,
    },
    Legend {
        source: String,
    },
    Hearts {
        source: String,
    },
    Counter {
        source: String,
    },
}

impl HudWidgetKindSchema {
    pub fn source(&self) -> &str {
        match self {
            Self::Bar { source }
            | Self::Text { source, .. }
            | Self::Icons { source }
            | Self::Map { source }
            | Self::Legend { source }
            | Self::Hearts { source }
            | Self::Counter { source } => source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_des_jeux_sources_connues() {
        for text in [
            include_str!("../../../games/isaac/assets/ui/hud.ron"),
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
