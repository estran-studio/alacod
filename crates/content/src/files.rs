//! Content readers shared by native tools and browser builds. Only text definitions and
//! an inventory of asset paths are embedded; images/audio still load over HTTP in Bevy.
use std::path::{Path, PathBuf};

pub type EmbeddedFiles = &'static [(&'static str, Option<&'static str>)];

#[derive(Debug, Clone, Default)]
pub struct ContentFiles {
    root: PathBuf,
    embedded: Option<EmbeddedFiles>,
}

impl ContentFiles {
    pub fn native(root: PathBuf) -> Self {
        Self {
            root,
            embedded: None,
        }
    }

    pub fn embedded(files: EmbeddedFiles) -> Self {
        Self {
            root: PathBuf::new(),
            embedded: Some(files),
        }
    }

    pub fn contains(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        match self.embedded {
            Some(files) => files.iter().any(|(name, _)| Path::new(name) == path),
            None => self.root.join(path).is_file(),
        }
    }

    pub fn read(&self, path: &Path) -> Result<String, std::io::Error> {
        match self.embedded {
            Some(files) => files
                .iter()
                .find(|(name, _)| Path::new(name) == path)
                .and_then(|(_, text)| *text)
                .map(str::to_owned)
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, path.display().to_string())
                }),
            None => std::fs::read_to_string(self.root.join(path)),
        }
    }

    /// Discover direct children, preserving the native registry's sorted ordering.
    pub fn discover(&self, path: &Path, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
        if self.contains(path) {
            return Ok(vec![path.to_path_buf()]);
        }
        let mut paths = match self.embedded {
            Some(files) => {
                if !files
                    .iter()
                    .any(|(name, _)| Path::new(name).starts_with(path))
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        path.display().to_string(),
                    ));
                }
                files
                    .iter()
                    .map(|(name, _)| PathBuf::from(name))
                    .filter(|name| name.parent() == Some(path))
                    .filter(|name| name.extension().and_then(|ext| ext.to_str()) == Some(extension))
                    .collect::<Vec<_>>()
            }
            None => std::fs::read_dir(self.root.join(path))?
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_file())
                .map(|entry| path.join(entry.file_name()))
                .filter(|name| name.extension().and_then(|ext| ext.to_str()) == Some(extension))
                .collect(),
        };
        paths.sort();
        Ok(paths)
    }
}
