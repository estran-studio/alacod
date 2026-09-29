//! Manifeste d'un jeu (`game.ron`, voir `docs/conventions.md` §3 et `docs/plan-engine.md`
//! §4.2). Décrit le nom du jeu, les dossiers de contenu à charger (typés) et le point
//! d'entrée (carte de départ, graine par défaut).

use serde::Deserialize;
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
}

/// Point d'entrée d'une partie : carte de départ et graine par défaut.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct EntryPoint {
    /// Chemin de la carte LDtk de départ, relatif à `<jeu>/assets/`.
    pub start_map: String,
    pub default_seed: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GameManifest {
    pub name: String,
    pub content_folders: Vec<ContentFolderDecl>,
    pub entry: EntryPoint,
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
        ron::from_str(&text).map_err(|e| ManifestError::Parse {
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
    }

    #[test]
    fn missing_manifest_is_not_found() {
        let err = GameManifest::load(Path::new("/does/not/exist")).unwrap_err();
        assert!(matches!(err, ManifestError::NotFound { .. }));
    }
}
