//! Manifeste d'un jeu (`game.ron`, voir `docs/conventions.md` §3 et `docs/plan-engine.md`
//! §4.2). Décrit le nom du jeu, les dossiers de contenu à charger (typés) et le point
//! d'entrée (carte de départ, graine par défaut).

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Nom du fichier manifeste, toujours dans `<jeu>/assets/`.
pub const MANIFEST_FILE_NAME: &str = "game.ron";

/// Un dossier (ou fichier) de contenu déclaré par le manifeste, avec son "kind".
///
/// `kind` est une chaîne (pas un enum Rust fermé) : une valeur non reconnue doit produire
/// une erreur de lint claire (« kind inconnu », voir `lint.rs`) plutôt qu'un échec de
/// désérialisation RON générique.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ContentFolderDecl {
    /// Chemin relatif à `<jeu>/assets/` : un fichier unique, ou un dossier scanné pour les
    /// fichiers de l'extension attendue par `kind` (non récursif, voir `registry.rs`).
    pub path: String,
    pub kind: String,
    /// m1-d26-doublons-generes (D26) : `false` : `alacod-gen` ne génère aucun scénario pour les
    /// définitions de ce dossier (copie de celles d'un autre jeu, déjà couvertes par ses
    /// scénarios générés) ; elles restent dans le registre et le jeu. Défaut `true`. Hors
    /// simulation.
    #[serde(default = "generate_by_default")]
    pub generate: bool,
}

fn generate_by_default() -> bool {
    true
}

/// Mode de run déclaré par le manifeste (T2.4, chantier F1, `docs/plan-engine.md` §5 F1).
/// Enum fermé (comme `sim_core::damage::FriendlyFire`) : une valeur inconnue échoue au
/// chargement RON, rapportée comme n'importe quelle autre erreur de parse — pas de lint
/// dédié nécessaire. Converti en `run::run::RunMode` par `game::jjrs` (ce crate ne dépend
/// pas de `run`, voir `docs/conventions.md` §13).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryMode {
    /// Le système de vagues actuel (`game::waves`). Exige un dossier de contenu `Wave`
    /// (validé par `lint::lint_entry_point`).
    Waves,
    /// Aucune condition de fin hors défaite.
    Sandbox,
    /// Séquence de niveaux (T1.8) : la séquence désignée par `entry.floors`, sinon (D59) la
    /// première (ordre des ids) d'un dossier de contenu `Floors` (validé par
    /// `lint::lint_entry_point`). Voir `docs/conventions.md` §17.
    Floors,
}

/// Mécanique qu'une **partie ordinaire** du jeu doit rencontrer (M3-T12, `docs/conventions.md` §77) :
/// une ligne du tableau « Atteignabilité » de `docs/taches.md` §8.1. Déclarée par `mechanics` du
/// manifeste ; le lint (`content::warnings`) avertit, statiquement, de chaque mécanique déclarée que
/// le contenu ne porte pas (kind `MechanicUnmet`), en nommant la salle ou le pool qui devrait la
/// porter. Hors simulation : aucun effet en jeu.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mechanic {
    /// Larmes modifiées et empilement : objets à `projectile` ou à `ShotSpeed`/`ShotSize` dans le pool
    /// d'une salle exigée par la grammaire (trésor) et dans une boutique, tenables en deux exemplaires.
    Tears,
    /// Effets v3 et transformations : objets à effets dans les pools trésor, boutique et boss ;
    /// synergies dont tous les objets sont atteignables.
    Effects,
    /// Familiers et orbitaux : au moins deux dans le pool du trésor, un dans celui de la boutique.
    Familiars,
    /// Bombes, rochers et piques : bombe de départ ou de récompense, rochers et piques dans les
    /// gabarits `combat` d'une grammaire.
    Bombs,
    /// Salle secrète : exigée par la grammaire en grille, murée (`door_lock: Secret`), bombe atteignable.
    SecretRoom,
    /// Clés et trésor fermé : clés en récompense de salle et en boutique, porte `Key` exigée par la grammaire.
    Keys,
    /// Cœurs, sous, boutique : cœur et sous dans la récompense de salle, boutique exigée par la grammaire.
    Hearts,
    /// Pools à épuisement : un pool `exhaust` tiré au moins deux fois par partie.
    ExhaustPools,
    /// Marché du diable : une salle post-boss à chance non nulle dans une grammaire de la partie ordinaire
    /// (son contenu, coffre ou boutique, relève de la règle « salle spéciale vide »).
    DevilMarket,
    /// Champions : variante à chance non nulle sur un ennemi des salles atteintes.
    Champions,
    /// Curses : chance par étage non nulle dans la séquence ordinaire.
    Curses,
    /// Accomplissements : un accomplissement atteignable dont le déblocage entre dans un pool tiré.
    Achievements,
}

/// Point d'entrée d'une partie : carte de départ, graine par défaut, mode de run.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct EntryPoint {
    /// Chemin de la carte LDtk de départ, relatif à `<jeu>/assets/`.
    pub start_map: String,
    pub default_seed: i32,
    /// Mode de run (T2.4). Absent (défaut) : `Waves` si le jeu déclare un dossier de
    /// contenu `Wave`, sinon `Sandbox` — résolu par `game::jjrs` à partir du registre, pas
    /// ici (ce module ne construit pas de registre).
    #[serde(default)]
    pub mode: Option<EntryMode>,
    /// D59 : id de la séquence du dossier `Floors` jouée par défaut en mode `Floors` (nom de
    /// fichier sans extension). Absent : la première par ordre d'id (comportement d'avant D59,
    /// gardé pour les jeux qui n'ont qu'une séquence). Un scénario garde la main par son champ
    /// `floors`. Exige `mode: Floors` et un id de séquence existant (lint).
    #[serde(default)]
    pub floors: Option<String>,
    /// M2-T13 (§57) : le hub du jeu. Absent : la partie démarre directement dans la run.
    #[serde(default)]
    pub hub: Option<HubEntry>,
    /// Progression active (T1.10, `docs/conventions.md` §27) : id d'un fichier du kind
    /// `Progression`. Absent (défaut) : aucune progression, même si le jeu en déclare une
    /// (le testbed la garde pour ses scénarios, qui l'imposent par `Scenario::progression`).
    #[serde(default)]
    pub progression: Option<String>,
    /// T1.9 : horloges actives dans une partie jouée (ids du kind `Clock`). Absent : aucune
    /// (un scénario les demande par son champ `clocks`).
    #[serde(default)]
    pub clocks: Option<Vec<String>>,
    /// T1.9 : difficulté (kind `Difficulty`) active dans une partie jouée. Absent : non.
    #[serde(default)]
    pub difficulty: Option<bool>,
    /// Assemblage des pièces (2026-10-17, `docs/conventions.md` §1) : nombre de pièces,
    /// bornes, alignement des prises, boucles, murage des prises libres. Absent : défauts de
    /// `world::MapSettings` (10 pièces, ±1000 px, aligné, boucles et murage actifs).
    #[serde(default)]
    pub map: world::MapSettings,
}

#[derive(Resource, Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GameManifest {
    pub name: String,
    pub content_folders: Vec<ContentFolderDecl>,
    pub entry: EntryPoint,
    /// T1.13 : carte et cible des scénarios générés de ce jeu (`scenario::generate`,
    /// `docs/conventions.md` §28). Absent : les gabarits jouent dans le testbed.
    #[serde(default)]
    pub generate_template: Option<GenerateTemplate>,
    /// M3-T12 (§77) : mécaniques que la partie ordinaire doit rencontrer ; chacune non portée par le
    /// contenu est un avertissement. Absent (tous les jeux avant `isaac`) : aucune vérification.
    #[serde(default)]
    pub mechanics: Vec<Mechanic>,
    /// SB-1 (`docs/taches/plan-bac-a-sable.md` D2) : réglages du bac à sable (`--sandbox`). Absent :
    /// l'arène est `generate_template.map`. Un jeu qui a un dossier `Sandbox` doit avoir une arène.
    #[serde(default)]
    pub sandbox: Option<SandboxEntry>,
    /// M-7 (§121) : la musique du jeu (feature `harmonium` de `game`, présentation seule). Absent : pas de musique.
    #[serde(default)]
    pub music: Option<MusicEntry>,
}

/// `hub: Some((map: "maps/hub.ldtk"))` (M2-T13, §57) : la carte LDtk du hub, relative à
/// `<jeu>/assets/` ; les services viennent du dossier de kind `HubService`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct HubEntry {
    pub map: String,
}

/// Banque principale par défaut de `music` : GeneralUser-GS compressée (SF3, lue en natif comme sur le web), relative à
/// `<jeu>/assets/` ; téléchargée par `scripts/harmonium-banks.sh` (jamais commitée).
pub const DEFAULT_MUSIC_MAIN_BANK: &str = "harmonium/GeneralUser-GS.sf3";
/// Banque de percussion par défaut de `music`, même dossier et même script.
pub const DEFAULT_MUSIC_PERCUSSION_BANK: &str = "harmonium/harmonium_trad.sf2";

fn default_main_bank() -> String {
    DEFAULT_MUSIC_MAIN_BANK.to_string()
}

fn default_percussion_bank() -> String {
    DEFAULT_MUSIC_PERCUSSION_BANK.to_string()
}

/// Durée d'une pulsation dansée en frames de simulation, sous forme de fraction (`num / den`, `3600 / 230` permis).
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct PulseFrames {
    pub num: u32,
    pub den: u32,
}

/// `music: (profile: "…", pulse_frames: (num: 3600, den: 230))` (M-7, `docs/conventions.md` §121) : le profil
/// harmonium joué et la période de sa **pulsation dansée** (blanche au reel 2/2, noire pointée en 6/8, noire en
/// 4/4) en frames de simulation. L'horloge du jeu (`FrameCount`) est la source du temps, la musique la suit
/// (option a de William, 2026-11-06) ; l'ancrage est la frame d'entrée en partie. Ce crate ne dépend pas
/// d'harmonium : l'existence du profil et le tempo (40 à 300 BPM à la noire) sont jugés au lancement par `game`,
/// qui refuse (journal) et ne joue rien. Hors simulation : aucun système de `GgrsSchedule` ne lit ce champ.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct MusicEntry {
    /// Id d'un profil distribué par harmonium (`reel_quebecois`, `gmd_pop`…).
    pub profile: String,
    pub pulse_frames: PulseFrames,
    /// Banque principale, relative à `<jeu>/assets/` (défaut [`DEFAULT_MUSIC_MAIN_BANK`]).
    #[serde(default = "default_main_bank")]
    pub main_bank: String,
    /// Banque de percussion (défaut [`DEFAULT_MUSIC_PERCUSSION_BANK`]).
    #[serde(default = "default_percussion_bank")]
    pub percussion_bank: String,
}

/// `sandbox: (arena: "maps/arena.ldtk")` (SB-1, D2) : la carte de l'arène du bac à sable, relative à
/// `<jeu>/assets/` (carte LDtk ; `cave:<id>` et `floor:<id>` comme `entry.start_map`). Un profil la
/// remplace par son champ `map`. Validée par `lint::lint_sandbox`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SandboxEntry {
    pub arena: String,
}

/// `generate_template: (map: "maps/salle.ldtk", target: "cible")` (T1.13) : les gabarits
/// d'armes et d'ennemis du jeu jouent dans ce jeu, sur `map` ; `target` est l'id du personnage
/// (`counts_hits: true`) que vise le gabarit d'arme. Validé par `lint::lint_generate_template`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GenerateTemplate {
    pub map: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    NotFound { path: PathBuf },
    Io { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestError::NotFound { path } => {
                write!(f, "{} : manifeste introuvable", path.display())
            }
            ManifestError::Io { path, message } => {
                write!(f, "{} : erreur de lecture ({message})", path.display())
            }
            ManifestError::Parse { path, message } => {
                write!(f, "{} : RON invalide : {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ManifestError {}

impl GameManifest {
    /// Dossier `assets/` d'un jeu, à partir de son dossier racine (ex. `games/zombies`).
    /// Tous les chemins de `content_folders` et `entry.start_map` sont relatifs à ce
    /// dossier (convention `docs/conventions.md` §3).
    pub fn assets_dir(game_dir: &Path) -> PathBuf {
        game_dir.join("assets")
    }

    /// Chemin du manifeste d'un jeu (`<game_dir>/assets/game.ron`).
    pub fn manifest_path(game_dir: &Path) -> PathBuf {
        Self::assets_dir(game_dir).join(MANIFEST_FILE_NAME)
    }

    /// Charge et parse `<game_dir>/assets/game.ron`.
    pub fn load(game_dir: &Path) -> Result<Self, ManifestError> {
        let path = Self::manifest_path(game_dir);
        if !path.is_file() {
            return Err(ManifestError::NotFound { path });
        }
        let text = std::fs::read_to_string(&path).map_err(|e| ManifestError::Io {
            path: path.clone(),
            message: e.to_string(),
        })?;
        Self::parse(&text, path)
    }

    pub fn parse(text: &str, path: PathBuf) -> Result<Self, ManifestError> {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(text)
            .map_err(|e| ManifestError::Parse {
                path,
                message: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_manifest() {
        let ron_text = r#"
        (
            name: "demo",
            content_folders: [
                (path: "characters/player.ron", kind: "Character"),
                (path: "weapons.ron", kind: "Weapon"),
            ],
            entry: (
                start_map: "exemples/test_map.ldtk",
                default_seed: 123,
            ),
        )
        "#;
        let manifest: GameManifest = ron::from_str(ron_text).unwrap();
        assert_eq!(manifest.name, "demo");
        assert_eq!(manifest.content_folders.len(), 2);
        assert_eq!(manifest.entry.default_seed, 123);
        assert_eq!(manifest.entry.mode, None);
    }

    /// D26 : `generate` vaut `true` par défaut ; `generate: false` se lit tel quel.
    #[test]
    fn parses_generate_flag() {
        let ron_text = r#"
        (
            name: "demo",
            content_folders: [
                (path: "weapons.ron", kind: "Weapon"),
                (path: "melee.ron", kind: "MeleeWeapon", generate: false),
            ],
            entry: (start_map: "carte.ldtk", default_seed: 1),
        )
        "#;
        let manifest: GameManifest = ron::from_str(ron_text).unwrap();
        assert!(manifest.content_folders[0].generate);
        assert!(!manifest.content_folders[1].generate);
    }

    /// T2.4 : `entry.mode` absent reste `None` (résolu par `game::jjrs`, pas ici) ;
    /// présent, il parse comme un enum fermé (`Waves`/`Sandbox`).
    #[test]
    fn parses_explicit_entry_mode() {
        let ron_text = r#"
        (
            name: "demo",
            content_folders: [],
            entry: (
                start_map: "exemples/test_map.ldtk",
                default_seed: 123,
                mode: Some(Sandbox),
            ),
        )
        "#;
        let manifest: GameManifest = ron::from_str(ron_text).unwrap();
        assert_eq!(manifest.entry.mode, Some(EntryMode::Sandbox));
    }

    #[test]
    fn loads_implicit_entry_mode() {
        let game_dir = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/entry_mode_waves_without_waves"
        ));
        let manifest = GameManifest::load(game_dir).unwrap();
        assert_eq!(manifest.entry.mode, Some(EntryMode::Waves));
    }

    #[test]
    fn missing_manifest_is_not_found() {
        let err = GameManifest::load(Path::new("/does/not/exist")).unwrap_err();
        assert!(matches!(err, ManifestError::NotFound { .. }));
    }

    /// M-7 : `music` absent = pas de musique ; présent, les banques ont un défaut (SF3 + percussion).
    #[test]
    fn parses_music_with_default_banks() {
        let base = r#"(name: "demo", content_folders: [], entry: (start_map: "carte.ldtk", default_seed: 1)"#;
        let manifest: GameManifest = ron::from_str(&format!("{base})")).unwrap();
        assert_eq!(manifest.music, None);
        let manifest: GameManifest = ron::from_str(&format!(
            r#"{base}, music: Some((profile: "gmd_pop", pulse_frames: (num: 3600, den: 230))))"#
        ))
        .unwrap();
        let music = manifest.music.expect("music lu");
        assert_eq!(music.profile, "gmd_pop");
        assert_eq!(
            music.pulse_frames,
            PulseFrames {
                num: 3600,
                den: 230
            }
        );
        assert_eq!(music.main_bank, DEFAULT_MUSIC_MAIN_BANK);
        assert_eq!(music.percussion_bank, DEFAULT_MUSIC_PERCUSSION_BANK);
    }
}
