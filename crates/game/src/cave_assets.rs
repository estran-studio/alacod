//! Source d'assets `cave://` (m1-integration-scenarios, `docs/conventions.md` §21) : **une
//! caverne = un asset**.
//!
//! Une caverne `cave:<id>` est générée en mémoire à partir du gabarit LDtk de son dossier
//! (`<dossier>/gabarit.ldtk`, `content::registry::CAVE_TEMPLATE_FILE`). Chargées toutes depuis
//! ce même chemin, les cavernes d'une séquence `Floors` recevaient le même handle :
//! l'`AssetServer` dédoublonne par chemin et rend le premier chargement (la première caverne)
//! à tous les étages. Chaque caverne se charge donc depuis son propre chemin virtuel
//! `cave://<dossier>/<id>.ldtk`, que le lecteur de cette source sert avec le gabarit du dossier ;
//! tout autre fichier (tilesets référencés par le gabarit) est lu tel quel.

use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSource, AssetSourceBuilder, ErasedAssetReader, PathStream,
    Reader, VecReader,
};
use bevy::prelude::*;
use std::path::{Path, PathBuf};

/// Nom de la source d'assets des cavernes.
pub const CAVE_ASSET_SOURCE: &str = "cave";

/// Chemin d'asset de la caverne `id` dont le gabarit est `template` (ex.
/// `caves/gabarit.ldtk`, `niveau_2` → `cave://caves/niveau_2.ldtk`).
pub fn cave_asset_path(template: &str, id: &str) -> String {
    let dir = Path::new(template)
        .parent()
        .map(|dir| dir.to_string_lossy().into_owned())
        .unwrap_or_default();
    if dir.is_empty() {
        format!("{CAVE_ASSET_SOURCE}://{id}.ldtk")
    } else {
        format!("{CAVE_ASSET_SOURCE}://{dir}/{id}.ldtk")
    }
}

/// Id de caverne d'un chemin d'asset de [`cave_asset_path`] (`None` pour une autre carte).
pub fn cave_id_of_asset_path(path: &str) -> Option<String> {
    let rest = path.strip_prefix(CAVE_ASSET_SOURCE)?.strip_prefix("://")?;
    Path::new(rest)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
}

/// Fichier réellement lu pour `path` : le gabarit du dossier pour un `.ldtk`, `path` sinon.
fn physical_path(path: &Path) -> PathBuf {
    if path.extension().is_some_and(|ext| ext == "ldtk") {
        path.with_file_name(content::registry::CAVE_TEMPLATE_FILE)
    } else {
        path.to_path_buf()
    }
}

/// Lecteur de la source `cave://`, au-dessus du lecteur par défaut de la plateforme (fichiers,
/// ou HTTP en wasm) sur la même racine que les autres assets.
struct CaveTemplateReader(Box<dyn ErasedAssetReader>, fn(&Path) -> PathBuf);

impl AssetReader for CaveTemplateReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        // Le lecteur sous-jacent emprunte le chemin qu'on lui passe : on lit tout (le gabarit
        // pèse quelques dizaines de Ko) pour rendre un lecteur qui possède ses octets.
        let physical = (self.1)(path);
        let mut reader = self.0.read(&physical).await?;
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| AssetReaderError::Io(error.into()))?;
        Ok(VecReader::new(bytes))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        // `AssetMetaCheck::Never` : jamais appelé ; pas de `.meta` pour une caverne.
        Err::<VecReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

/// Enregistre la source `cave://` ; doit être construit **avant** `AssetPlugin` (voir
/// `CoreSetupPlugin::get_default_plugin`). `file_path` : la racine des assets, la même que
/// celle d'`AssetPlugin`.
pub struct CaveAssetSourcePlugin {
    pub file_path: String,
}

impl Plugin for CaveAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        let mut default_reader = AssetSource::get_default_reader(self.file_path.clone());
        app.register_asset_source(
            CAVE_ASSET_SOURCE,
            AssetSourceBuilder::new(move || {
                Box::new(CaveTemplateReader(default_reader(), physical_path)) as Box<dyn ErasedAssetReader>
            }),
        );
    }
}

/// Nom de la source d'assets des étages de grammaire (M2-T10, `docs/conventions.md` §40).
pub const FLOOR_ASSET_SOURCE: &str = "floor";

/// Chemin d'asset de l'étage d'emplacement `slot` dont la carte de gabarits est `templates`
/// (ex. `maps/salles.ldtk`, 1 → `floor://maps/salles@1.ldtk`). Comme les cavernes, **un étage = un
/// asset** : l'`AssetServer` dédoublonne par chemin, et deux étages de même grammaire recevraient
/// sinon le même monde.
pub fn floor_asset_path(templates: &str, slot: usize) -> String {
    let path = Path::new(templates);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let dir = path
        .parent()
        .map(|dir| dir.to_string_lossy().into_owned())
        .unwrap_or_default();
    if dir.is_empty() {
        format!("{FLOOR_ASSET_SOURCE}://{stem}@{slot}.ldtk")
    } else {
        format!("{FLOOR_ASSET_SOURCE}://{dir}/{stem}@{slot}.ldtk")
    }
}

/// Fichier réellement lu pour un chemin de [`floor_asset_path`] : la carte de gabarits (le
/// suffixe `@<emplacement>` retiré) ; tout autre fichier tel quel.
fn floor_physical_path(path: &Path) -> PathBuf {
    let is_ldtk = path.extension().is_some_and(|ext| ext == "ldtk");
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
    match (is_ldtk, stem.as_deref().and_then(|s| s.rsplit_once('@'))) {
        (true, Some((base, slot))) if slot.chars().all(|c| c.is_ascii_digit()) => {
            path.with_file_name(format!("{base}.ldtk"))
        }
        _ => path.to_path_buf(),
    }
}

/// Enregistre la source `floor://` (comme [`CaveAssetSourcePlugin`], avant `AssetPlugin`).
pub struct FloorAssetSourcePlugin {
    pub file_path: String,
}

impl Plugin for FloorAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        let mut default_reader = AssetSource::get_default_reader(self.file_path.clone());
        app.register_asset_source(
            FLOOR_ASSET_SOURCE,
            AssetSourceBuilder::new(move || {
                Box::new(CaveTemplateReader(default_reader(), floor_physical_path))
                    as Box<dyn ErasedAssetReader>
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chemin_virtuel_par_caverne() {
        assert_eq!(
            cave_asset_path("caves/gabarit.ldtk", "niveau_2"),
            "cave://caves/niveau_2.ldtk"
        );
        assert_eq!(
            cave_id_of_asset_path("cave://caves/niveau_2.ldtk").as_deref(),
            Some("niveau_2")
        );
        assert_eq!(cave_id_of_asset_path("caves/gabarit.ldtk"), None);
        assert_eq!(
            physical_path(Path::new("caves/niveau_2.ldtk")),
            Path::new("caves/gabarit.ldtk")
        );
        assert_eq!(
            physical_path(Path::new("atlas/roche.png")),
            Path::new("atlas/roche.png")
        );
    }

    #[test]
    fn chemin_virtuel_par_etage() {
        assert_eq!(
            floor_asset_path("maps/salles.ldtk", 1),
            "floor://maps/salles@1.ldtk"
        );
        assert_eq!(
            floor_physical_path(Path::new("maps/salles@1.ldtk")),
            Path::new("maps/salles.ldtk")
        );
        assert_eq!(
            floor_physical_path(Path::new("maps/salles.ldtk")),
            Path::new("maps/salles.ldtk")
        );
        assert_eq!(
            floor_physical_path(Path::new("atlas/roche.png")),
            Path::new("atlas/roche.png")
        );
    }
}
