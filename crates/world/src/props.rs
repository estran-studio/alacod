//! Objets de décor interactifs (M2-T11, chantier E4 v2, `docs/conventions.md` §46) : tables
//! renversables et barils explosifs. Données de contenu (kind `Prop`, `props/<id>.ron`), état
//! rollback des entités et règles pures (direction de renversement, collider renversé). Le
//! branchement ECS (interaction, dégâts, explosion) vit dans `game::props` ; la lecture des
//! entités LDtk `Table`/`Barrel` dans `map_ldtk`.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use serde::{Deserialize, Serialize};

/// Définition d'un objet de décor (`props/<id>.ron`) :
/// ```ron
/// Table(bullets: 6)
/// Barrel(health: "20.0", radius: "48.0", damage: "30.0")
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropDef {
    Table(TableDef),
    Barrel(BarrelDef),
}

impl PropDef {
    /// Nom de l'entité LDtk qui accepte cette définition (`Table`, `Barrel`).
    pub fn ldtk_entity(&self) -> &'static str {
        match self {
            PropDef::Table(_) => "Table",
            PropDef::Barrel(_) => "Barrel",
        }
    }
}

/// Table renversable : obstacle `Barricade` tant qu'elle est debout (bloque le passage, pas
/// les balles) ; renversée par une interaction, elle devient une couverture qui arrête les
/// balles et se brise après `bullets` balles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableDef {
    /// Balles encaissées une fois renversée avant d'être détruite (> 0, lint).
    pub bullets: u32,
    /// Épaisseur (unités) de la couverture une fois renversée, au bord opposé au joueur.
    #[serde(default = "default_flipped_thickness")]
    pub thickness: Fixed,
    /// Portée de l'interaction (unités, depuis le bord du collider).
    #[serde(default = "default_flip_range")]
    pub range: Fixed,
}

fn default_flipped_thickness() -> Fixed {
    Fixed::from_num(6)
}

fn default_flip_range() -> Fixed {
    Fixed::from_num(20)
}

/// Baril explosif : `health` PV ; à 0, explosion de rayon `radius` qui inflige `damage` à
/// tout ce qu'elle touche (joueurs, ennemis, autres barils : chaîne).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarrelDef {
    pub health: Fixed,
    /// Rayon de l'explosion en unités (> 0, lint).
    pub radius: Fixed,
    pub damage: Fixed,
}

/// Objets de décor du jeu par identifiant (contenu, hors rollback, posée au chargement de la
/// carte comme `SurfaceTable`). Vide hors jeu avec dossier `Prop`.
#[derive(Resource, Clone, Debug, Default)]
pub struct PropTable(pub BTreeMap<String, PropDef>);

/// Direction de renversement d'une table : à l'opposé du joueur qui la renverse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlipDirection {
    Right,
    Left,
    Up,
    Down,
}

/// Direction de renversement : de `player` vers `table`, sur l'axe dominant (égalité : axe x ;
/// écart nul : `Right`).
pub fn flip_direction(player: FixedVec2, table: FixedVec2) -> FlipDirection {
    let d = table - player;
    if d.x.abs() >= d.y.abs() {
        if d.x >= Fixed::ZERO {
            FlipDirection::Right
        } else {
            FlipDirection::Left
        }
    } else if d.y >= Fixed::ZERO {
        FlipDirection::Up
    } else {
        FlipDirection::Down
    }
}

/// Rectangle (largeur, hauteur, décalage x, décalage y) de la table renversée : une bande
/// d'épaisseur `thickness` (bornée par la table) au bord situé du côté `direction`, sur toute
/// la longueur de ce bord. Le reste de l'emprise de la table est libéré.
pub fn flipped_rect(
    width: Fixed,
    height: Fixed,
    direction: FlipDirection,
    thickness: Fixed,
) -> (Fixed, Fixed, Fixed, Fixed) {
    let two = Fixed::from_num(2);
    match direction {
        FlipDirection::Right | FlipDirection::Left => {
            let t = thickness.min(width);
            let offset = (width - t) / two;
            let x = if direction == FlipDirection::Right {
                offset
            } else {
                -offset
            };
            (t, height, x, Fixed::ZERO)
        }
        FlipDirection::Up | FlipDirection::Down => {
            let t = thickness.min(height);
            let offset = (height - t) / two;
            let y = if direction == FlipDirection::Up {
                offset
            } else {
                -offset
            };
            (width, t, Fixed::ZERO, y)
        }
    }
}

/// État rollback d'une table (composant **neutre** : porté seulement par les tables).
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TableState {
    /// Id de contenu (`props/<id>.ron`).
    pub prop: String,
    /// Dimensions de la table debout (entité LDtk).
    pub width: Fixed,
    pub height: Fixed,
    /// `None` : debout.
    pub flipped: Option<FlipDirection>,
}

/// État rollback d'un baril (composant **neutre** : porté seulement par les barils). Les PV
/// sont dans `Health`.
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BarrelState {
    pub prop: String,
    pub radius: Fixed,
    pub damage: Fixed,
}

/// Dernière position sûre d'un joueur (M2-T11, fosses) : translation d'une frame où ses pieds
/// étaient sur une case sans fosse dont les 8 voisines sont aussi sans fosse. Composant
/// rollback **neutre**, posé seulement sur une carte qui a des fosses. Champs séparés :
/// `FixedVec2` n'est pas `Hash`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PitSafePosition {
    pub x: Fixed,
    pub y: Fixed,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: i32, y: i32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    #[test]
    fn renversee_a_l_oppose_du_joueur() {
        assert_eq!(flip_direction(v(0, 0), v(10, 3)), FlipDirection::Right);
        assert_eq!(flip_direction(v(0, 0), v(-10, 3)), FlipDirection::Left);
        assert_eq!(flip_direction(v(0, 0), v(3, 10)), FlipDirection::Up);
        assert_eq!(flip_direction(v(0, 0), v(3, -10)), FlipDirection::Down);
        // Égalité : axe x
        assert_eq!(flip_direction(v(0, 0), v(5, 5)), FlipDirection::Right);
        assert_eq!(flip_direction(v(0, 0), v(0, 0)), FlipDirection::Right);
    }

    #[test]
    fn bande_au_bord_du_cote_renverse() {
        let (w, h, t) = (Fixed::from_num(32), Fixed::from_num(16), Fixed::from_num(6));
        assert_eq!(
            flipped_rect(w, h, FlipDirection::Up, t),
            (w, t, Fixed::ZERO, Fixed::from_num(5))
        );
        assert_eq!(
            flipped_rect(w, h, FlipDirection::Down, t),
            (w, t, Fixed::ZERO, Fixed::from_num(-5))
        );
        assert_eq!(
            flipped_rect(w, h, FlipDirection::Left, t),
            (t, h, Fixed::from_num(-13), Fixed::ZERO)
        );
        // Épaisseur bornée par la table
        assert_eq!(
            flipped_rect(w, h, FlipDirection::Up, Fixed::from_num(40)),
            (w, h, Fixed::ZERO, Fixed::ZERO)
        );
    }

    #[test]
    fn definitions_en_ron() {
        let table: PropDef = ron::from_str("Table(bullets: 6)").unwrap();
        assert_eq!(
            table,
            PropDef::Table(TableDef {
                bullets: 6,
                thickness: Fixed::from_num(6),
                range: Fixed::from_num(20),
            })
        );
        let barrel: PropDef =
            ron::from_str(r#"Barrel(health: "20.0", radius: "48.0", damage: "30.0")"#).unwrap();
        assert_eq!(barrel.ldtk_entity(), "Barrel");
    }
}
