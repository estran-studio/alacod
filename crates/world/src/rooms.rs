//! Salles typées et verrouillées (M2-E1, chantier E, `docs/conventions.md` §35) : contrats
//! sans dépendance au rendu ni au chargement LDtk.
//!
//! - [`RoomKind`] : type d'une salle (`depart`, `combat`, `recompense`, `boss`, `boutique`…),
//!   composant **statique** posé sur l'entité du niveau LDtk (champ de niveau `room_kind`).
//!   La liste est **ouverte** : les types sont les fichiers du dossier de contenu `Room`
//!   ([`RoomKindDef`], table [`RoomKindTable`]) ; un type inconnu est une erreur de lint.
//! - [`RoomStates`] : état rollback de chaque salle verrouillante ([`RoomState`]), ressource
//!   **neutre** (vide hors carte typée : contribution 0 au checksum, traces existantes
//!   inchangées).
//! - [`RoomDormant`] : marqueur (rollback, neutre) d'un ennemi qui vit dans une salle
//!   dormante ; l'IA saute alors sélection de cible, déplacement et attaque.
//! - [`RoomChanged`] : `FrameEvents`, un changement d'état de salle (récompenses
//!   `OnRoomClear` et moment clé `room`).
//!
//! Les systèmes (verrouillage, portes, téléportation des absents) vivent dans `map_ldtk`
//! (`map_ldtk::game::rooms`) : ils ont besoin des portes et des bornes de salle du chargement.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Type d'une salle (statique, hors rollback : posé au chargement du niveau).
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RoomKind(pub String);

/// Définition d'un type de salle (`rooms/<id>.ron`, kind de contenu `Room`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomKindDef {
    /// La salle se verrouille quand un joueur debout y entre et qu'un ennemi y vit, et ses
    /// ennemis dorment tant qu'elle n'est pas activée. Faux : la salle est un lieu comme un
    /// autre (départ, récompense, boutique).
    #[serde(default)]
    pub locks: bool,
}

/// Types de salles du jeu par identifiant (contenu, hors rollback, posée au chargement de la
/// carte comme `SurfaceTable`). Vide hors jeu avec dossier `Room`.
#[derive(Resource, Clone, Debug, Default)]
pub struct RoomKindTable(pub BTreeMap<String, RoomKindDef>);

impl RoomKindTable {
    /// Le type verrouille-t-il ses salles ? (Type inconnu : non.)
    pub fn locks(&self, kind: &str) -> bool {
        self.0.get(kind).is_some_and(|def| def.locks)
    }
}

/// État d'une salle verrouillante.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum RoomState {
    /// Pas encore activée : ses ennemis dorment, ses portes sont ouvertes.
    #[default]
    Dormant,
    /// Un joueur est entré, des ennemis vivent : portes fermées.
    Locked,
    /// Plus aucun ennemi : portes rouvertes pour de bon.
    Cleared,
}

/// État des salles par identifiant de niveau (`LevelId`). Rollback + checksum + trace en
/// variante **neutre** : vide (défaut) sur une carte sans `room_kind`. Une salle absente de
/// `rooms` est [`RoomState::Dormant`].
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoomStates {
    /// Les portes de salle ont été ouvertes au premier tick (une porte de salle est ouverte
    /// par défaut).
    pub doors_opened: bool,
    pub rooms: BTreeMap<String, RoomState>,
}

impl RoomStates {
    pub fn get(&self, room: &str) -> RoomState {
        self.rooms.get(room).copied().unwrap_or_default()
    }
}

/// Posé par la simulation sur un ennemi qui vit dans une salle dormante, retiré à
/// l'activation. Rollback + trace en variante **neutre** (type nouveau, absent de tout
/// ennemi hors carte typée).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoomDormant;

/// Un changement d'état de salle, dans la frame où il a lieu.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RoomChanged {
    pub frame: u32,
    pub room: String,
    pub kind: String,
    pub from: RoomState,
    pub to: RoomState,
}

/// Point d'entrée d'une salle à la porte `door` : à `depth` unités à l'intérieur de la
/// salle, sur la normale du bord le plus proche de la porte, avec la tangente unitaire de ce
/// bord (pour écarter plusieurs joueurs). Boîte `(position, size)` en coordonnées monde (+y
/// vers le haut), comme `RoomBounds`.
pub fn entrance_point(
    position: FixedVec2,
    size: FixedVec2,
    door: FixedVec2,
    depth: Fixed,
) -> (FixedVec2, FixedVec2) {
    let to_left = (door.x - position.x).abs();
    let to_right = (position.x + size.x - door.x).abs();
    let to_bottom = (door.y - position.y).abs();
    let to_top = (position.y + size.y - door.y).abs();
    let nearest = to_left.min(to_right).min(to_bottom).min(to_top);
    let up = FixedVec2::new(Fixed::ZERO, Fixed::ONE);
    let right = FixedVec2::new(Fixed::ONE, Fixed::ZERO);
    // Égalité : gauche, droite, bas, haut (ordre fixe, déterministe).
    if nearest == to_left {
        (FixedVec2::new(position.x + depth, door.y), up)
    } else if nearest == to_right {
        (FixedVec2::new(position.x + size.x - depth, door.y), up)
    } else if nearest == to_bottom {
        (FixedVec2::new(door.x, position.y + depth), right)
    } else {
        (FixedVec2::new(door.x, position.y + size.y - depth), right)
    }
}

/// Le point `p` est dans la boîte `(position, size)` réduite de `inset` de chaque côté
/// (`inset` négatif : agrandie).
pub fn inside_box(position: FixedVec2, size: FixedVec2, p: FixedVec2, inset: Fixed) -> bool {
    p.x >= position.x + inset
        && p.y >= position.y + inset
        && p.x <= position.x + size.x - inset
        && p.y <= position.y + size.y - inset
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: i32, y: i32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    #[test]
    fn entrance_est_a_l_interieur_du_bord_le_plus_proche() {
        let (pos, size) = (v(100, 0), v(160, 120));
        let depth = Fixed::from_num(40);
        // porte sur le bord gauche
        assert_eq!(entrance_point(pos, size, v(100, 60), depth).0, v(140, 60));
        // porte sur le bord droit
        assert_eq!(entrance_point(pos, size, v(260, 50), depth).0, v(220, 50));
        // porte sur le bord bas / haut
        assert_eq!(entrance_point(pos, size, v(180, 0), depth).0, v(180, 40));
        assert_eq!(entrance_point(pos, size, v(180, 120), depth).0, v(180, 80));
        // porte de la salle voisine, posée juste à l'extérieur du bord gauche
        assert_eq!(entrance_point(pos, size, v(92, 60), depth).0, v(140, 60));
    }

    #[test]
    fn inside_box_marge() {
        let (pos, size) = (v(0, 0), v(100, 100));
        assert!(inside_box(pos, size, v(10, 10), Fixed::ZERO));
        assert!(!inside_box(pos, size, v(10, 10), Fixed::from_num(24)));
        assert!(inside_box(pos, size, v(-10, 50), Fixed::from_num(-24)));
    }

    #[test]
    fn etat_par_defaut_neutre() {
        assert_eq!(RoomStates::default().get("x"), RoomState::Dormant);
        assert!(!RoomKindTable::default().locks("combat"));
    }
}
