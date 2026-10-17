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
//! [`Profiles`]. La simulation ne le lit jamais : un déblocage entre dans une run par
//! `items::RunUnlocks`, figé au lancement et identique chez tous les pairs (M2-T12, §51).

pub mod achievement;
pub use achievement::*;
pub mod hub;
pub use hub::*;

use bevy::prelude::Resource;
use run::{RunEnd, RunSummary};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Version courante du format. `0` : fichier sans champ `version` (avant versionnement).
///
/// - `1` : monnaies, déblocages, statistiques (M2-T0c).
/// - `2` : + [`Profile::achievements`] (M3-T10, `docs/conventions.md` §75) ; un fichier v0 ou v1
///   se lit tel quel (champ absent = aucun accomplissement), seul le numéro change.
pub const PROFILE_VERSION: u32 = 2;

/// Jalons que [`Profile::apply_run`] ajoute aux déblocages (hors `Unlock` du jeu) : un service du
/// hub peut en dépendre (M2-T13).
pub const MILESTONES: [&str; 2] = ["first_run", "first_victory"];

/// Monnaie méta créditée par [`Profile::apply_run`].
pub const ESSENCE: &str = "essence";

/// Essence gagnée par run (M2-T12, `meta/essence.ron` du jeu, kind `MetaEconomy`) ; défaut =
/// les règles provisoires de M2-T0c (1 par run, 1 par kill, 10 par victoire).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EssenceRules {
    #[serde(default = "one")]
    pub per_run: u64,
    #[serde(default = "one")]
    pub per_kill: u64,
    #[serde(default = "ten")]
    pub per_victory: u64,
}

fn one() -> u64 {
    1
}

fn ten() -> u64 {
    10
}

impl Default for EssenceRules {
    fn default() -> Self {
        Self {
            per_run: 1,
            per_kill: 1,
            per_victory: 10,
        }
    }
}

/// Règles du jeu (hors rollback), posées par le chargeur de contenu ; défaut sans fichier.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MetaRules(pub EssenceRules);

/// Condition d'achat d'un déblocage (copie des champs de `items::UnlockDef`, sans dépendre d'`items`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurchaseTerms<'a> {
    /// Coût en [`ESSENCE`] ; `None` : gratuit.
    pub cost: Option<i64>,
    /// Déblocages préalables.
    pub requires: &'a [String],
    /// Statistique minimale `(nom, valeur)`.
    pub stat: Option<(&'a str, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurchaseError {
    AlreadyOwned,
    MissingPrerequisite(String),
    StatTooLow { stat: String, have: u64, need: u64 },
    NotEnoughEssence { have: u64, need: u64 },
}

impl std::fmt::Display for PurchaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyOwned => write!(f, "déjà débloqué"),
            Self::MissingPrerequisite(id) => write!(f, "prérequis manquant : {id}"),
            Self::StatTooLow { stat, have, need } => {
                write!(f, "statistique « {stat} » {have} < {need}")
            }
            Self::NotEnoughEssence { have, need } => write!(f, "essence {have} < {need}"),
        }
    }
}

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
    /// Accomplissements obtenus (ids de `achievements/<id>.ron`, M3-T10) : ce qui a déjà accordé
    /// ses déblocages, pour ne les annoncer qu'une fois. Les déblocages, eux, sont dans
    /// [`Profile::unlocks`] (la seule donnée que la run suivante lit, par `RunUnlocks`).
    #[serde(default)]
    pub achievements: BTreeSet<String>,
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

    /// Lit un profil et le migre vers [`PROFILE_VERSION`] (v0 → v1 → v2 : seul le numéro change,
    /// les champs absents prennent leur valeur par défaut).
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
        self.apply_run_with(summary, &EssenceRules::default());
    }

    /// Accorde `def` si `id` n'est pas encore obtenu : ajoute `id` à [`Profile::achievements`] et
    /// ses `unlocks` à [`Profile::unlocks`] (M3-T10). Renvoie vrai pour un accomplissement nouveau ;
    /// un accomplissement déjà obtenu ne rajoute rien (un déblocage retiré à la main ne revient pas).
    pub fn grant_achievement(&mut self, id: &str, def: &AchievementDef) -> bool {
        if !self.achievements.insert(id.to_string()) {
            return false;
        }
        self.unlocks.extend(def.unlocks.iter().cloned());
        true
    }

    /// Achète un déblocage hors simulation : vérifie les conditions puis débite l'essence et
    /// l'ajoute à [`Profile::unlocks`]. Rien ne change en cas d'erreur.
    pub fn purchase(&mut self, id: &str, terms: &PurchaseTerms) -> Result<(), PurchaseError> {
        if self.has_unlock(id) {
            return Err(PurchaseError::AlreadyOwned);
        }
        if let Some(missing) = terms.requires.iter().find(|r| !self.has_unlock(r)) {
            return Err(PurchaseError::MissingPrerequisite(missing.clone()));
        }
        if let Some((stat, need)) = terms.stat {
            let have = self.stat(stat);
            if have < need {
                return Err(PurchaseError::StatTooLow {
                    stat: stat.to_string(),
                    have,
                    need,
                });
            }
        }
        let cost = terms.cost.unwrap_or(0).max(0) as u64;
        let have = self.currency(ESSENCE);
        if have < cost {
            return Err(PurchaseError::NotEnoughEssence { have, need: cost });
        }
        if cost > 0 {
            self.currencies.insert(ESSENCE.to_string(), have - cost);
        }
        self.unlocks.insert(id.to_string());
        Ok(())
    }

    /// [`Profile::apply_run`] avec les règles d'essence du jeu.
    pub fn apply_run_with(&mut self, summary: &RunSummary, rules: &EssenceRules) {
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
            rules.per_run
                + rules.per_kill * u64::from(summary.kills)
                + if victory { rules.per_victory } else { 0 },
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
        assert!(text.contains("version: 2"), "{text}");
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
    fn migration_v1_vers_v2_ajoute_les_accomplissements_sans_rien_perdre() {
        // Un profil écrit avant M3-T10 : version 1, pas de champ `achievements`.
        let v1 = r#"(version: 1, id: "local-0", currencies: {"essence": 11}, unlocks: ["first_run", "tresor_trefle"], stats: {"runs": 3})"#;
        let migrated = Profile::from_ron(v1).unwrap();
        assert_eq!(migrated.version, 2);
        assert!(migrated.achievements.is_empty());
        assert_eq!(migrated.currency(ESSENCE), 11);
        assert!(migrated.has_unlock("tresor_trefle") && migrated.has_unlock("first_run"));
        assert_eq!(migrated.stat("runs"), 3);
        // Réécrit en v2, relu à l'identique.
        let rewritten = migrated.to_ron();
        assert!(rewritten.contains("version: 2"), "{rewritten}");
        assert_eq!(Profile::from_ron(&rewritten).unwrap(), migrated);
        // Un fichier v2 garde ses accomplissements.
        let mut earned = migrated.clone();
        earned.achievements.insert("gardien_vaincu".into());
        assert_eq!(Profile::from_ron(&earned.to_ron()).unwrap(), earned);
    }

    #[test]
    fn un_accomplissement_accorde_ses_deblocages_une_seule_fois() {
        let def = AchievementDef {
            name: String::new(),
            when: vec![AchievementCondition::Victory],
            unlocks: vec!["a".into(), "b".into()],
        };
        let mut profile = Profile::new("local-0");
        assert!(profile.grant_achievement("fini", &def));
        assert!(profile.has_unlock("a") && profile.has_unlock("b"));
        assert!(profile.achievements.contains("fini"));
        profile.unlocks.remove("a");
        assert!(!profile.grant_achievement("fini", &def));
        assert!(!profile.has_unlock("a"));
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
    fn achat_debite_et_verifie_les_conditions() {
        let mut profile = Profile::new("local-0");
        profile.apply_run(&summary(9, RunEnd::Defeat)); // 10 essence, kills 9
        let req = ["first_run".to_string()];
        let terms = PurchaseTerms {
            cost: Some(4),
            requires: &req,
            stat: Some(("kills", 5)),
        };
        profile.purchase("a", &terms).unwrap();
        assert_eq!(profile.currency(ESSENCE), 6);
        assert!(profile.has_unlock("a"));
        assert_eq!(
            profile.purchase("a", &terms),
            Err(PurchaseError::AlreadyOwned)
        );
        let dear = PurchaseTerms {
            cost: Some(7),
            requires: &[],
            stat: None,
        };
        assert!(matches!(
            profile.purchase("b", &dear),
            Err(PurchaseError::NotEnoughEssence { .. })
        ));
        let missing = ["zzz".to_string()];
        let needs = PurchaseTerms {
            cost: None,
            requires: &missing,
            stat: None,
        };
        assert!(matches!(
            profile.purchase("c", &needs),
            Err(PurchaseError::MissingPrerequisite(_))
        ));
        let stat = PurchaseTerms {
            cost: None,
            requires: &[],
            stat: Some(("kills", 50)),
        };
        assert!(matches!(
            profile.purchase("d", &stat),
            Err(PurchaseError::StatTooLow { .. })
        ));
        assert_eq!(profile.currency(ESSENCE), 6);
        assert!(!profile.has_unlock("b") && !profile.has_unlock("c") && !profile.has_unlock("d"));
    }

    #[test]
    fn regles_d_essence_en_donnees() {
        let rules = EssenceRules {
            per_run: 2,
            per_kill: 3,
            per_victory: 20,
        };
        let mut profile = Profile::new("x");
        profile.apply_run_with(&summary(4, RunEnd::Victory), &rules);
        assert_eq!(profile.currency(ESSENCE), 2 + 12 + 20);
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
