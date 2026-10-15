//! `test:` des objets, des types de salle et des boss (M2-T17, `docs/conventions.md` §54) :
//! opt-in comme celui des armes et des personnages (§28), lu par le générateur de scénarios
//! (`scenario::generate`) et compté par le lint (`content`, qui n'a pas `Expectation`).
//!
//! Les fichiers `objets/<id>.ron` et `rooms/<id>.ron` sont lus par leurs crates de contrat
//! (`items::ItemDef`, `world::RoomKindDef`), qui ignorent le champ `test` : [`ItemFile`] et
//! [`RoomFile`] ne lisent que lui.

use combat::weapons::expectations::Expectation;
use serde::Deserialize;

/// Graine de carte d'un gabarit sans `seed` (celle de `Scenario::default`).
pub const DEFAULT_SEED: i32 = 123456;

fn default_seed() -> i32 {
    DEFAULT_SEED
}

/// `test:` d'un objet : `test: Some((frames: 60, expect: [...]))`. Gabarit `ItemHeld` : le joueur
/// commence avec l'objet, sans input (hors `UseActive` d'un actif).
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ItemTest {
    /// Durée du scénario généré (> 0).
    pub frames: u32,
    /// Attentes en plus des attentes par défaut.
    #[serde(default)]
    pub expect: Vec<Expectation>,
}

/// `test:` d'un type de salle : `test: Some((frames: 1200, floors: "etages_court"))`.
/// Gabarit `RoomClear` : un bot `explorateur` joue un étage qui contient une salle de ce type.
/// Exactement un de `floors` (séquence `Floors` du jeu) ou `map` (carte LDtk du jeu).
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RoomTest {
    pub frames: u32,
    #[serde(default)]
    pub floors: Option<String>,
    #[serde(default)]
    pub map: Option<String>,
    #[serde(default = "default_seed")]
    pub seed: i32,
    #[serde(default)]
    pub expect: Vec<Expectation>,
}

/// `boss:` du `test:` d'un personnage qui a `boss:` : gabarit `BossFight`, un bot `fonceur` joue
/// l'étage ou la carte qui contient ce boss. Sans `floors` ni `map`, la partie du manifeste
/// (`entry.start_map`, comme `gungeon_boss_bot`).
#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct BossFightTest {
    #[serde(default)]
    pub floors: Option<String>,
    #[serde(default)]
    pub map: Option<String>,
    #[serde(default = "default_seed")]
    pub seed: i32,
    #[serde(default)]
    pub expect: Vec<Expectation>,
}

/// Un fichier d'objet, vu par son seul champ `test`.
#[derive(Debug, Deserialize, Default)]
pub struct ItemFile {
    #[serde(default)]
    pub test: Option<ItemTest>,
}

/// Un fichier de type de salle, vu par son seul champ `test`.
#[derive(Debug, Deserialize, Default)]
pub struct RoomFile {
    #[serde(default)]
    pub test: Option<RoomTest>,
}
