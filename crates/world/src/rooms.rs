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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomKindDef {
    /// La salle se verrouille quand un joueur debout y entre et qu'un ennemi y vit, et ses
    /// ennemis dorment tant qu'elle n'est pas activée. Faux : la salle est un lieu comme un
    /// autre (départ, récompense, boutique).
    #[serde(default)]
    pub locks: bool,
    /// M2-T5 : coffre posé au centre de chaque salle de ce type (`docs/conventions.md` §48).
    #[serde(default)]
    pub chest: Option<items::RoomChestDef>,
    /// M2-T6 : présentoirs posés au centre de chaque salle de ce type (`docs/conventions.md`
    /// §50). Remplace le coffre de M2-T5 dans `boutique`.
    #[serde(default)]
    pub shop: Option<items::RoomShopDef>,
    /// M2-T7 (`docs/conventions.md` §45) : arène à tenir. La salle se verrouille dès qu'un
    /// joueur debout y entre (même vide), fait apparaître ses vagues internes et ne se rouvre
    /// qu'à la fin de son horloge, si un joueur vit encore. Implique le verrouillage.
    #[serde(default)]
    pub arena: Option<ArenaDef>,
    /// M3-T0c (§62) : la porte des salles de ce type est fermée à clé ou murée. Absent (tout le
    /// contenu existant) : porte ordinaire, hors du hash de la définition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub door_lock: Option<DoorLock>,
    /// M3-T5 (§70) : étage (indice dans la séquence `Floors`, 0 = le premier) à partir duquel
    /// `door_lock` s'applique ; avant, la porte est ordinaire. 0 (défaut) : dès le premier étage.
    /// Le trésor d'Isaac est ouvert au premier étage et fermé à clé ensuite.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub lock_from_floor: u32,
    /// M3-T8 (§73) : butin du boss de cette salle (pool, rareté mini) : à sa mort un objet tombe au
    /// centre de la salle. Absent (tout le contenu existant) : rien ne tombe, hors du hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss_drop: Option<items::BossDropDef>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Verrou de la porte d'une salle (M3-T0c, `docs/conventions.md` §62).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DoorLock {
    /// Reste fermée jusqu'à ce qu'un joueur la touche par Interaction avec `n` clés
    /// (`game::loot::KEY_ITEM`, consommées à l'ouverture). `n > 0`.
    Key(#[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))] u32),
    /// Mur plein jusqu'à une explosion `destroys` qui le touche (M3-T4).
    Secret,
}

/// Salles dont la porte verrouillée a été ouverte, par identifiant de niveau. Rollback +
/// checksum + trace en variante **neutre** : vide (défaut) tant qu'aucune porte verrouillée n'a
/// été ouverte. Une salle à `door_lock` absente de l'ensemble est fermée.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct DoorLocks(pub std::collections::BTreeSet<String>);

impl DoorLocks {
    /// La porte de `room` est-elle ouverte ? (Une salle sans `door_lock` ne passe pas par ici.)
    pub fn is_open(&self, room: &str) -> bool {
        self.0.contains(room)
    }
}

impl DoorLock {
    /// Problème du verrou pour le lint ; `None` s'il est valide.
    pub fn problem(&self) -> Option<String> {
        match self {
            DoorLock::Key(0) => Some("door_lock : Key(0) : doit être > 0".to_string()),
            _ => None,
        }
    }
}

/// Arène à tenir (M2-T7) : horloge de `clock` frames comptée depuis le verrouillage, vagues
/// internes datées sur cette horloge.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArenaDef {
    /// Durée à tenir, en frames depuis le verrouillage (> 0).
    #[cfg_attr(feature = "schema", schemars(extend("x-lint" = "> 0")))]
    pub clock: u32,
    /// Vagues internes (au moins une), dans l'ordre de leurs `at`.
    pub waves: Vec<ArenaWave>,
}

/// Une vague d'arène : à `at` frames du verrouillage, les personnages listés apparaissent dans
/// la salle (positions tirées du flux RNG de la salle, `room:<niveau>`).
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArenaWave {
    pub at: u32,
    /// `(personnage, nombre)`, nombre > 0.
    pub enemies: Vec<(String, u32)>,
}

/// Frame de verrouillage de chaque arène en cours, par identifiant de niveau (M2-T7).
/// Rollback + checksum + trace en variante **neutre** : vide hors arène. Les vagues dues et la
/// fin de l'horloge se déduisent de `frame - début` (pas de curseur).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ArenaClocks(pub BTreeMap<String, u32>);

/// Nom du flux RNG d'une salle (`RngStreams`) : positions des renforts de boss et des vagues
/// d'arène.
pub fn room_rng_stream(level_id: &str) -> String {
    format!("room:{level_id}")
}

/// Types de salles du jeu par identifiant (contenu, hors rollback, posée au chargement de la
/// carte comme `SurfaceTable`). Vide hors jeu avec dossier `Room`.
#[derive(Resource, Clone, Debug, Default)]
pub struct RoomKindTable(pub BTreeMap<String, RoomKindDef>);

impl RoomKindTable {
    /// Le type verrouille-t-il ses salles ? (Type inconnu : non.)
    pub fn locks(&self, kind: &str) -> bool {
        self.0
            .get(kind)
            .is_some_and(|def| def.locks || def.arena.is_some())
    }

    /// L'arène du type, s'il en est une (M2-T7).
    pub fn arena(&self, kind: &str) -> Option<&ArenaDef> {
        self.0.get(kind).and_then(|def| def.arena.as_ref())
    }

    /// Le verrou de porte du type, s'il en a un (M3-T0c), tous étages confondus.
    pub fn door_lock(&self, kind: &str) -> Option<DoorLock> {
        self.0.get(kind).and_then(|def| def.door_lock)
    }

    /// Le verrou de porte du type **à l'étage `floor`** (M3-T5) : `door_lock` seulement à partir
    /// de `lock_from_floor`, sinon la porte est ordinaire.
    pub fn door_lock_at(&self, kind: &str, floor: u32) -> Option<DoorLock> {
        self.0
            .get(kind)
            .filter(|def| floor >= def.lock_from_floor)
            .and_then(|def| def.door_lock)
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

    fn table_with(kind: &str, door_lock: Option<DoorLock>, lock_from_floor: u32) -> RoomKindTable {
        let mut table = RoomKindTable::default();
        table.0.insert(
            kind.to_string(),
            RoomKindDef {
                door_lock,
                lock_from_floor,
                ..Default::default()
            },
        );
        table
    }

    #[test]
    fn verrou_des_le_premier_etage_par_defaut() {
        let table = table_with("tresor", Some(DoorLock::Key(1)), 0);
        assert_eq!(table.door_lock_at("tresor", 0), Some(DoorLock::Key(1)));
        assert_eq!(table.door_lock_at("tresor", 3), Some(DoorLock::Key(1)));
        assert_eq!(table.door_lock("tresor"), Some(DoorLock::Key(1)));
    }

    #[test]
    fn verrou_seulement_a_partir_de_lock_from_floor() {
        let table = table_with("tresor_ferme", Some(DoorLock::Key(2)), 1);
        assert_eq!(table.door_lock_at("tresor_ferme", 0), None);
        assert_eq!(
            table.door_lock_at("tresor_ferme", 1),
            Some(DoorLock::Key(2))
        );
        assert_eq!(
            table.door_lock_at("tresor_ferme", 2),
            Some(DoorLock::Key(2))
        );
        // `door_lock` ignore l'étage : tous étages confondus.
        assert_eq!(table.door_lock("tresor_ferme"), Some(DoorLock::Key(2)));
    }

    #[test]
    fn verrou_absent_ou_type_inconnu() {
        let table = table_with("combat", None, 1);
        assert_eq!(table.door_lock_at("combat", 5), None);
        assert_eq!(table.door_lock_at("inconnu", 5), None);
    }
}
