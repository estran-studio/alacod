//! Profil de méta-progression (M2-T0c, chantier G1, `docs/conventions.md` §38).
//!
//! Un [`Profile`] par joueur local (identifiant : pubkey allumette si connue, sinon
//! `local-<handle>`) : monnaies méta, déblocages, statistiques cumulées. Sérialisé en RON
//! **versionné** ([`PROFILE_VERSION`], migration par `version`), un fichier par profil dans un
//! dossier configurable ([`profile_dir`] : `ALACOD_PROFILE_DIR`, sinon le dossier de données de
//! l'utilisateur).
//!
//! **Hors simulation** : écrit à la fin d'une run depuis son [`RunSummary`] par
//! `game::profile` (`Update`, jamais `GgrsSchedule`) et lu au lancement dans la ressource
//! [`Profiles`]. La simulation ne le lit pas dans cette tâche : M2-T12 décidera comment un
//! déblocage entre dans une run de façon déterministe et partagée entre pairs.

use bevy::prelude::Resource;
use run::{RunEnd, RunSummary};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Version courante du format. `0` : fichier sans champ `version` (avant versionnement).
pub const PROFILE_VERSION: u32 = 1;

/// Monnaie méta créditée par [`Profile::apply_run`].
pub const ESSENCE: &str = "essence";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    #[serde(default)]
    pub currencies: BTreeMap<String, u64>,
    #[serde(default)]
    pub unlocks: BTreeSet<String>,
    #[serde(default)]
    pub stats: BTreeMap<String, u64>,
}

#[derive(Debug)]
pub enum ProfileError {
    Parse(String),
    /// Écrit par une version plus récente du jeu : on ne le réécrit pas.
    TooNew(u32),
    Io(String),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "profil illisible : {e}"),
            Self::TooNew(v) => write!(
                f,
                "profil de version {v} > {PROFILE_VERSION} (jeu plus récent) : non touché"
            ),
            Self::Io(e) => write!(f, "profil : {e}"),
        }
    }
}

impl Profile {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            version: PROFILE_VERSION,
            id: id.into(),
            ..Default::default()
        }
    }

    pub fn to_ron(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .expect("un Profile se sérialise toujours")
    }

    /// Lit un profil et le migre vers [`PROFILE_VERSION`] (v0 → v1 : seul le numéro change).
    pub fn from_ron(text: &str) -> Result<Self, ProfileError> {
        let mut profile: Profile =
            ron::from_str(text).map_err(|e| ProfileError::Parse(e.to_string()))?;
        if profile.version > PROFILE_VERSION {
            return Err(ProfileError::TooNew(profile.version));
        }
        profile.version = PROFILE_VERSION;
        Ok(profile)
    }

    pub fn has_unlock(&self, unlock: &str) -> bool {
        self.unlocks.contains(unlock)
    }

    pub fn currency(&self, id: &str) -> u64 {
        self.currencies.get(id).copied().unwrap_or(0)
    }

    pub fn stat(&self, id: &str) -> u64 {
        self.stats.get(id).copied().unwrap_or(0)
    }

    /// Crédite une run terminée : statistiques cumulées (`runs`, `victories`, `kills`, `frames`,
    /// meilleures `wave_best` et `floor_best`), monnaie [`ESSENCE`] (+1 par run, +1 par kill,
    /// +10 par victoire) et déblocages `first_run` / `first_victory`. Règles provisoires (M2-T12).
    pub fn apply_run(&mut self, summary: &RunSummary) {
        let add = |map: &mut BTreeMap<String, u64>, key: &str, n: u64| {
            *map.entry(key.to_string()).or_default() += n;
        };
        let best = |map: &mut BTreeMap<String, u64>, key: &str, n: u64| {
            let slot = map.entry(key.to_string()).or_default();
            *slot = (*slot).max(n);
        };
        add(&mut self.stats, "runs", 1);
        add(&mut self.stats, "kills", u64::from(summary.kills));
        add(&mut self.stats, "frames", u64::from(summary.frames));
        best(
            &mut self.stats,
            "wave_best",
            u64::from(summary.wave_reached),
        );
        best(
            &mut self.stats,
            "floor_best",
            u64::from(summary.floor_reached),
        );
        let victory = summary.outcome == RunEnd::Victory;
        if victory {
            add(&mut self.stats, "victories", 1);
            self.unlocks.insert("first_victory".to_string());
        }
        add(
            &mut self.currencies,
            ESSENCE,
            1 + u64::from(summary.kills) + if victory { 10 } else { 0 },
        );
        self.unlocks.insert("first_run".to_string());
    }
}

/// Dossier des profils : `ALACOD_PROFILE_DIR`, sinon `$XDG_DATA_HOME/alacod/profiles`, sinon
/// `$HOME/.local/share/alacod/profiles` ; `None` si rien n'est déterminable.
pub fn profile_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("ALACOD_PROFILE_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|d| !d.is_empty())
                .map(|home| PathBuf::from(home).join(".local/share"))
        })?;
    Some(base.join("alacod/profiles"))
}

/// Identifiant d'un joueur local sans pubkey connue.
pub fn local_id(handle: usize) -> String {
    format!("local-{handle}")
}

/// Fichier d'un profil : l'identifiant est nettoyé (`[A-Za-z0-9_-]`, le reste devient `_`).
pub fn profile_path(dir: &Path, id: &str) -> PathBuf {
    let clean: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    dir.join(format!("{clean}.ron"))
}

/// Écriture atomique (fichier temporaire puis renommage) : un crash ne laisse jamais un profil
/// à moitié écrit.
pub fn save(dir: &Path, profile: &Profile) -> Result<PathBuf, ProfileError> {
    std::fs::create_dir_all(dir).map_err(|e| ProfileError::Io(e.to_string()))?;
    let path = profile_path(dir, &profile.id);
    let tmp = path.with_extension("ron.tmp");
    std::fs::write(&tmp, profile.to_ron()).map_err(|e| ProfileError::Io(e.to_string()))?;
    std::fs::rename(&tmp, &path).map_err(|e| ProfileError::Io(e.to_string()))?;
    Ok(path)
}

pub fn load(dir: &Path, id: &str) -> Result<Option<Profile>, ProfileError> {
    let path = profile_path(dir, id);
    match std::fs::read_to_string(&path) {
        Ok(text) => Profile::from_ron(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ProfileError::Io(e.to_string())),
    }
}

/// Profils chargés (hors rollback, hors simulation).
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Profiles(pub BTreeMap<String, Profile>);

impl Profiles {
    /// Charge tous les `*.ron` du dossier ; un fichier illisible est ignoré (retourné dans les
    /// erreurs, jamais écrasé).
    pub fn load_dir(dir: &Path) -> (Self, Vec<(PathBuf, ProfileError)>) {
        let mut profiles = BTreeMap::new();
        let mut errors = Vec::new();
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "ron"))
            .collect();
        files.sort();
        for path in files {
            match std::fs::read_to_string(&path)
                .map_err(|e| ProfileError::Io(e.to_string()))
                .and_then(|text| Profile::from_ron(&text))
            {
                Ok(profile) => {
                    profiles.insert(profile.id.clone(), profile);
                }
                Err(e) => errors.push((path, e)),
            }
        }
        (Self(profiles), errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(kills: u32, outcome: RunEnd) -> RunSummary {
        RunSummary {
            wave_reached: 3,
            kills,
            points_total: 100,
            frames: 600,
            outcome,
            floor_reached: 1,
        }
    }

    #[test]
    fn ron_aller_retour_et_version() {
        let mut profile = Profile::new("local-0");
        profile.apply_run(&summary(4, RunEnd::Victory));
        let text = profile.to_ron();
        assert!(text.contains("version: 1"), "{text}");
        assert_eq!(Profile::from_ron(&text).unwrap(), profile);
    }

    #[test]
    fn migration_v0_et_version_future() {
        // v0 : pas de champ `version`, pas de stats.
        let v0 = r#"(id: "local-1", currencies: {"essence": 7})"#;
        let migrated = Profile::from_ron(v0).unwrap();
        assert_eq!(migrated.version, PROFILE_VERSION);
        assert_eq!(migrated.currency(ESSENCE), 7);
        assert!(matches!(
            Profile::from_ron(r#"(version: 99, id: "x")"#),
            Err(ProfileError::TooNew(99))
        ));
        assert!(matches!(
            Profile::from_ron("pas du ron"),
            Err(ProfileError::Parse(_))
        ));
    }

    #[test]
    fn run_credite_stats_monnaie_et_deblocages() {
        let mut profile = Profile::new("local-0");
        profile.apply_run(&summary(4, RunEnd::Defeat));
        assert_eq!(profile.currency(ESSENCE), 5);
        assert!(profile.has_unlock("first_run") && !profile.has_unlock("first_victory"));
        profile.apply_run(&summary(2, RunEnd::Victory));
        assert_eq!(profile.currency(ESSENCE), 5 + 13);
        assert_eq!(profile.stat("runs"), 2);
        assert_eq!(profile.stat("victories"), 1);
        assert_eq!(profile.stat("kills"), 6);
        assert_eq!(profile.stat("wave_best"), 3);
        assert!(profile.has_unlock("first_victory"));
    }

    #[test]
    fn ecriture_atomique_et_relecture() {
        let dir = std::env::temp_dir().join(format!("alacod-meta-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut profile = Profile::new("local-0");
        profile.apply_run(&summary(1, RunEnd::Defeat));
        let path = save(&dir, &profile).unwrap();
        assert!(path.ends_with("local-0.ron"));
        assert!(!path.with_extension("ron.tmp").exists());
        assert_eq!(load(&dir, "local-0").unwrap(), Some(profile.clone()));
        assert_eq!(load(&dir, "autre").unwrap(), None);
        // fichier illisible : signalé, jamais perdu
        std::fs::write(dir.join("casse.ron"), "???").unwrap();
        let (profiles, errors) = Profiles::load_dir(&dir);
        assert_eq!(profiles.0.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            profile_path(&dir, "a/b c").file_name().unwrap(),
            "a_b_c.ron"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
