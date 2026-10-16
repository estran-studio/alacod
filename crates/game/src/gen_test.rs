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
    /// M3-T13 (§78) : gabarit `TearsOnTarget` en plus de `ItemHeld` (scénario `tears_<id>`).
    #[serde(default)]
    pub tears: Option<TearsTest>,
    /// M3-T13 : gabarit `FamiliarHeld` en plus de `ItemHeld` (scénario `familiar_<id>`), pour un
    /// objet qui a `familiar:`.
    #[serde(default)]
    pub familiar: Option<FamiliarTest>,
}

fn one_hit() -> u32 {
    1
}

/// `tears:` du `test:` d'un objet : le joueur tient l'objet, tire sur la cible de l'arène du jeu
/// (`target`, ou `generate_template.target`) et doit la toucher `min_hits` fois.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TearsTest {
    /// Durée du scénario `tears_<id>` ; absent : `test.frames` (le `ItemHeld` d'un objet déjà
    /// généré garde ainsi sa durée).
    #[serde(default)]
    pub frames: Option<u32>,
    /// Arme du joueur (`PlayerScript::weapon`) ; absent : ses armes de départ.
    #[serde(default)]
    pub weapon: Option<String>,
    /// Personnage qui compte ses coups (`counts_hits`), posé juste devant la cible de l'arène sur
    /// la ligne de tir : sa santé règle la cible. Absent : la cible de l'arène.
    #[serde(default)]
    pub target: Option<String>,
    /// Coups minimum sur la cible à la dernière frame (défaut 1).
    #[serde(default = "one_hit")]
    pub min_hits: u32,
    #[serde(default)]
    pub expect: Vec<Expectation>,
}

/// `familiar:` du `test:` d'un objet : le joueur tient l'objet ; `FamiliarCount` = `per_copy` ;
/// `min_hits` > 0 : le joueur tire aussi sur la cible et elle doit recevoir au moins ce nombre de
/// coups (plus que le joueur seul n'en donne : à fixer d'après la partie jouée).
#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct FamiliarTest {
    /// Durée du scénario `familiar_<id>` ; absent : `test.frames`.
    #[serde(default)]
    pub frames: Option<u32>,
    #[serde(default)]
    pub weapon: Option<String>,
    #[serde(default)]
    pub min_hits: u32,
    #[serde(default)]
    pub expect: Vec<Expectation>,
}

/// `champions:` du `test:` d'un personnage à `variants:` : gabarit `ChampionForced`, un scénario
/// par entrée (`champion_<id>_<variante>`). L'ennemi apparaît avec la variante imposée
/// (`CharacterPlacement::variant`) ; `kill` : le joueur le tire (dans l'axe, `weapon`) et les
/// effets de mort déclarés (`Explode` : moment clé `explosion` ; `SpawnCharacters` : au moins
/// `count` ennemis) sont attendus.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ChampionTest {
    pub variant: String,
    #[serde(default)]
    pub kill: bool,
    #[serde(default)]
    pub weapon: Option<String>,
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
