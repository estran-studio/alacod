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
    "blanks",
    "keys",
    "shells",
    "passives",
    "active_item",
    "active_charge",
    "minimap",
    "curses",
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
    Map {
        source: String,
    },
}

impl HudWidgetKindSchema {
    pub fn source(&self) -> &str {
        match self {
            Self::Bar { source }
            | Self::Text { source, .. }
            | Self::Icons { source }
            | Self::Map { source } => source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
