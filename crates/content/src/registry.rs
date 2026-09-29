//! Registre typé : charge les dossiers de contenu déclarés par le manifeste
//! (`manifest.rs`) en tables indexées par des identifiants typés (newtypes sur chaîne,
//! `Ord`, `BTreeMap` partout : CLAUDE.md, règle de déterminisme 5). C'est la source de
//! vérité des ids de contenu (`docs/plan-engine.md` §4.2) ; les `Handle<...>` Bevy
//! (`crates/game/src/global_asset.rs`) restent le support de rendu/animation.
//!
//! Le chargement est "best effort" : une entrée invalide (RON cassé, id dupliqué) est
//! rapportée comme erreur et ignorée plutôt que d'interrompre tout le chargement, pour que
//! `alacod lint` et le rechargement à chaud (`crates/game/src/content_hot_reload.rs`)
//! rapportent plusieurs problèmes en un seul passage.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use sim_core::kinds::{KindDecl, Kinds};
use sim_core::stats::StatId;

use crate::lint::{LintError, LintErrorKind};
use crate::manifest::{ContentFolderDecl, GameManifest, MANIFEST_FILE_NAME};
use crate::value::FixedField;

// ---------------------------------------------------------------------------------------
// Identifiants typés
// ---------------------------------------------------------------------------------------

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
    };
}

string_id!(
    /// Identifiant d'un personnage (joueur ou ennemi : les deux sont un `CharacterConfig`,
    /// voir `crates/game/src/character/config.rs`). Dérivé du champ `asset_name_ref` de
    /// chaque fichier, pas du nom de fichier : les fichiers actuels sous
    /// `ZombieShooter/Sprites/**` ne sont pas nommés d'après leur id (voir
    /// `games/zombies/assets/game.ron`).
    CharacterId
);
string_id!(
    /// Identifiant d'une entrée de `weapons.ron` (arme à distance).
    WeaponId
);
string_id!(
    /// Identifiant d'une entrée de `melee_weapons.ron` (arme de corps à corps).
    MeleeWeaponId
);
string_id!(
    /// Référence à un `CharacterId` dans un rôle d'ennemi (ex. `wave_config.ron`,
    /// `enemy_probabilities`). Même espace de noms que `CharacterId` aujourd'hui : il
    /// n'existe pas encore de dossier `enemies/*.ron` séparé de `characters/*.ron` avec un
    /// schéma propre (voir la doc du module et le rapport de la tâche T1.5).
    EnemyId
);
string_id!(
    /// Identifiant d'une configuration de vagues (nom de fichier, sans extension).
    WaveConfigId
);
string_id!(
    /// Identifiant d'une carte LDtk (nom de fichier, sans extension).
    MapId
);

/// Dérive le même id qu'au chargement (`load_maps`) à partir d'un chemin quelconque
/// (utilisé pour valider `entry.start_map`, qui n'est pas forcément le même chemin exact
/// que celui listé par un dossier `Map`, mais désigne le même fichier).
pub fn map_id_from_path(path: &str) -> MapId {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    MapId::from(stem.to_string())
}

// ---------------------------------------------------------------------------------------
// Entrées
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CharacterEntry {
    pub id: CharacterId,
    /// Chemin relatif à `assets/` : message d'erreur et `AssetServer::load`.
    pub file: PathBuf,
    pub asset_name_ref: String,
    pub base_health_max: FixedField,
    pub max_speed: FixedField,
    pub starting_skin: String,
    pub skins: BTreeSet<String>,
    /// `WeaponId` donnés au spawn, **dans l'ordre déclaré** (`starting_weapons` du RON) :
    /// l'ordre compte (le premier est l'arme active), voir
    /// `crates/game/src/character/player/create.rs`.
    pub starting_weapons: Vec<WeaponId>,
    /// Surcharges de stats (T1.2, chantier B2), pour la règle « valeurs >= 0 ».
    pub stats: BTreeMap<StatId, FixedField>,
}

#[derive(Debug, Clone)]
pub struct WeaponEntry {
    pub id: WeaponId,
    pub file: PathBuf,
    /// Cadence de tir par mode (`firing_modes`), pour la règle « cadence > 0 ».
    pub firing_rates: BTreeMap<String, FixedField>,
}

#[derive(Debug, Clone)]
pub struct MeleeWeaponEntry {
    pub id: MeleeWeaponId,
    pub file: PathBuf,
    pub damage: FixedField,
}

#[derive(Debug, Clone)]
pub struct WaveConfigEntry {
    pub id: WaveConfigId,
    pub file: PathBuf,
    /// Noms référencés par `wave_tiers[].enemy_probabilities`, union de tous les paliers.
    pub enemy_refs: BTreeSet<EnemyId>,
}

#[derive(Debug, Clone)]
pub struct MapEntry {
    pub id: MapId,
    pub file: PathBuf,
}

/// Registre de contenu d'un jeu, chargé depuis son manifeste (`GameManifest`). Voir le
/// module pour les garanties (BTreeMap partout, chargement "best effort").
#[derive(Resource, Debug, Clone, Default)]
pub struct Registry {
    pub game_dir: PathBuf,
    pub characters: BTreeMap<CharacterId, CharacterEntry>,
    pub weapons: BTreeMap<WeaponId, WeaponEntry>,
    pub melee_weapons: BTreeMap<MeleeWeaponId, MeleeWeaponEntry>,
    pub waves: BTreeMap<WaveConfigId, WaveConfigEntry>,
    pub maps: BTreeMap<MapId, MapEntry>,
    /// Fichiers `Ui`/`Camera` validés (RON syntaxiquement correct). Pas de table typée par
    /// id : rien ne les référence par id aujourd'hui (décision T1.5, voir le rapport de la
    /// tâche).
    pub ui_files: Vec<PathBuf>,
    pub camera_files: Vec<PathBuf>,
}

/// Les "kinds" de dossier de contenu que cette version de `content` sait charger. Exposé
/// via `sim_core::kinds::Kinds` (constante ici plutôt que construite depuis une `App`
/// Bevy : voir le rapport de la tâche T1.5, décision « liste des kinds pour la CLI »).
pub const KNOWN_KIND_NAMES: &[&str] = &[
    "Character",
    "Weapon",
    "MeleeWeapon",
    "Wave",
    "Map",
    "Ui",
    "Camera",
];

pub fn known_content_kinds() -> Kinds {
    let mut kinds = Kinds::default();
    for name in KNOWN_KIND_NAMES {
        kinds.register(KindDecl::new("content_folder", *name));
    }
    kinds
}

impl Registry {
    /// Charge tous les dossiers déclarés par `manifest` (chemins relatifs à
    /// `game_dir/assets/`). Retourne un registre "best effort" (les entrées valides sont
    /// gardées même si d'autres échouent) et la liste des erreurs de chargement (RON
    /// cassé, id dupliqué, kind de dossier inconnu). Les erreurs *sémantiques*
    /// (références, plages de valeurs) sont du ressort de `lint::run`, appelé séparément
    /// sur le résultat.
    pub fn build(game_dir: &Path, manifest: &GameManifest) -> (Registry, Vec<LintError>) {
        let assets_dir = GameManifest::assets_dir(game_dir);
        let mut registry = Registry {
            game_dir: game_dir.to_path_buf(),
            ..Default::default()
        };
        let mut errors = Vec::new();
        let kinds = known_content_kinds();

        for decl in &manifest.content_folders {
            if !kinds.has("content_folder", &decl.kind) {
                errors.push(LintError {
                    kind: LintErrorKind::UnknownKind,
                    file: MANIFEST_FILE_NAME.to_string(),
                    message: format!(
                        "content_folders : kind « {} » inconnu pour le dossier « {} » (connus : {})",
                        decl.kind,
                        decl.path,
                        KNOWN_KIND_NAMES.join(", ")
                    ),
                });
                continue;
            }

            match decl.kind.as_str() {
                "Character" => load_characters(&assets_dir, decl, &mut registry, &mut errors),
                "Weapon" => load_weapons(&assets_dir, decl, &mut registry, &mut errors),
                "MeleeWeapon" => load_melee_weapons(&assets_dir, decl, &mut registry, &mut errors),
                "Wave" => load_waves(&assets_dir, decl, &mut registry, &mut errors),
                "Map" => load_maps(&assets_dir, decl, &mut registry, &mut errors),
                "Ui" => load_generic_ron(&assets_dir, decl, &mut registry.ui_files, &mut errors),
                "Camera" => {
                    load_generic_ron(&assets_dir, decl, &mut registry.camera_files, &mut errors)
                }
                _ => unreachable!("filtré par `kinds.has` ci-dessus"),
            }
        }

        (registry, errors)
    }
}

// ---------------------------------------------------------------------------------------
// Découverte de fichiers
// ---------------------------------------------------------------------------------------

/// Résout une déclaration de contenu (`decl.path`, relatif à `assets/`) en fichiers : un
/// fichier unique tel quel, ou un dossier scanné (non récursif, trié) pour l'extension
/// `ext`. Non récursif : certains dossiers de contenu actuels mélangent plusieurs kinds
/// (ex. `ZombieShooter/Sprites/Character/` contient aussi bien des personnages que des
/// feuilles de sprite) ; le manifeste déclare alors des fichiers individuels plutôt que le
/// dossier entier (voir `games/zombies/assets/game.ron`).
fn discover_files(
    assets_dir: &Path,
    decl: &ContentFolderDecl,
    ext: &str,
) -> Result<Vec<PathBuf>, LintError> {
    let abs = assets_dir.join(&decl.path);

    if abs.is_file() {
        return Ok(vec![PathBuf::from(&decl.path)]);
    }

    if abs.is_dir() {
        let read_dir = std::fs::read_dir(&abs).map_err(|e| LintError {
            kind: LintErrorKind::Parse,
            file: decl.path.clone(),
            message: format!("dossier illisible : {e}"),
        })?;

        let mut files: Vec<PathBuf> = read_dir
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some(ext))
            .filter_map(|p| p.file_name().map(|n| Path::new(&decl.path).join(n)))
            .collect();
        files.sort();
        return Ok(files);
    }

    Err(LintError {
        kind: LintErrorKind::Parse,
        file: decl.path.clone(),
        message: "fichier ou dossier introuvable (content_folders)".to_string(),
    })
}

fn read_file(assets_dir: &Path, rel: &Path) -> Result<String, LintError> {
    let abs = assets_dir.join(rel);
    std::fs::read_to_string(&abs).map_err(|e| LintError {
        kind: LintErrorKind::Parse,
        file: rel.display().to_string(),
        message: format!("lecture impossible ({}) : {e}", abs.display()),
    })
}

// ---------------------------------------------------------------------------------------
// Schémas de lint (mirroir minimal des types réels de `crates/game`, voir `value.rs`)
// ---------------------------------------------------------------------------------------
//
// Ces structs ne portent que les champs utiles au lint (id, références, plages). Le RON
// est self-describing (comme JSON) : un champ non déclaré ici est silencieusement ignoré
// par serde (pas de `deny_unknown_fields`), donc un fichier réel plus riche (collider,
// sprite_config, audio_config...) se charge sans erreur.

#[derive(Deserialize)]
struct CharacterFileSchema {
    movement: MovementSchema,
    asset_name_ref: String,
    base_health: HealthSchema,
    starting_skin: String,
    #[serde(default)]
    skins: BTreeMap<String, serde::de::IgnoredAny>,
    #[serde(default)]
    starting_weapons: Vec<String>,
    /// T1.2, chantier B2 : surcharges de stats. `StatId` vient directement de `sim_core`
    /// (pas de mirroir : c'est déjà le type réel, `content` en dépend déjà).
    #[serde(default)]
    stats: BTreeMap<StatId, FixedField>,
}

#[derive(Deserialize)]
struct MovementSchema {
    max_speed: FixedField,
}

#[derive(Deserialize)]
struct HealthSchema {
    max: FixedField,
}

#[derive(Deserialize)]
struct WeaponsFileSchema(BTreeMap<String, WeaponEntrySchema>);

#[derive(Deserialize)]
struct WeaponEntrySchema {
    config: WeaponConfigSchema,
}

#[derive(Deserialize)]
struct WeaponConfigSchema {
    #[serde(default)]
    firing_modes: BTreeMap<String, FiringModeSchema>,
}

#[derive(Deserialize)]
struct FiringModeSchema {
    firing_rate: FixedField,
}

#[derive(Deserialize)]
struct MeleeWeaponsFileSchema(BTreeMap<String, MeleeWeaponEntrySchema>);

#[derive(Deserialize)]
struct MeleeWeaponEntrySchema {
    config: MeleeWeaponConfigSchema,
}

#[derive(Deserialize)]
struct MeleeWeaponConfigSchema {
    damage: FixedField,
}

#[derive(Deserialize)]
struct WaveConfigFileSchema {
    #[serde(default)]
    wave_tiers: Vec<WaveTierSchema>,
}

#[derive(Deserialize)]
struct WaveTierSchema {
    #[serde(default)]
    enemy_probabilities: BTreeMap<String, u32>,
}

// ---------------------------------------------------------------------------------------
// Chargeurs par kind
// ---------------------------------------------------------------------------------------

fn load_characters(
    assets_dir: &Path,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: CharacterFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        let id = CharacterId::from(parsed.asset_name_ref.clone());
        if let Some(existing) = registry.characters.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de personnage « {id} » (champ asset_name_ref) déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }

        registry.characters.insert(
            id.clone(),
            CharacterEntry {
                id,
                file: rel,
                asset_name_ref: parsed.asset_name_ref,
                base_health_max: parsed.base_health.max,
                max_speed: parsed.movement.max_speed,
                starting_skin: parsed.starting_skin,
                skins: parsed.skins.into_keys().collect(),
                starting_weapons: parsed
                    .starting_weapons
                    .into_iter()
                    .map(WeaponId::from)
                    .collect(),
                stats: parsed.stats,
            },
        );
    }
}

fn load_weapons(
    assets_dir: &Path,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: WeaponsFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        for (name, entry) in parsed.0 {
            let id = WeaponId::from(name);
            if let Some(existing) = registry.weapons.get(&id) {
                errors.push(LintError {
                    kind: LintErrorKind::DuplicateId,
                    file: rel.display().to_string(),
                    message: format!(
                        "id d'arme « {id} » déjà défini dans {}",
                        existing.file.display()
                    ),
                });
                continue;
            }
            let firing_rates = entry
                .config
                .firing_modes
                .into_iter()
                .map(|(mode, cfg)| (mode, cfg.firing_rate))
                .collect();
            registry.weapons.insert(
                id.clone(),
                WeaponEntry {
                    id,
                    file: rel.clone(),
                    firing_rates,
                },
            );
        }
    }
}

fn load_melee_weapons(
    assets_dir: &Path,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: MeleeWeaponsFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        for (name, entry) in parsed.0 {
            let id = MeleeWeaponId::from(name);
            if let Some(existing) = registry.melee_weapons.get(&id) {
                errors.push(LintError {
                    kind: LintErrorKind::DuplicateId,
                    file: rel.display().to_string(),
                    message: format!(
                        "id d'arme de corps à corps « {id} » déjà défini dans {}",
                        existing.file.display()
                    ),
                });
                continue;
            }
            registry.melee_weapons.insert(
                id.clone(),
                MeleeWeaponEntry {
                    id,
                    file: rel.clone(),
                    damage: entry.config.damage,
                },
            );
        }
    }
}

fn load_waves(
    assets_dir: &Path,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let parsed: WaveConfigFileSchema = match ron::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                errors.push(LintError {
                    kind: LintErrorKind::Parse,
                    file: rel.display().to_string(),
                    message: format!("erreur RON : {e}"),
                });
                continue;
            }
        };

        let id = WaveConfigId::from(
            rel.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        if let Some(existing) = registry.waves.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de vagues « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }

        let mut enemy_refs = BTreeSet::new();
        for tier in &parsed.wave_tiers {
            for name in tier.enemy_probabilities.keys() {
                enemy_refs.insert(EnemyId::from(name.clone()));
            }
        }

        registry.waves.insert(
            id.clone(),
            WaveConfigEntry {
                id,
                file: rel,
                enemy_refs,
            },
        );
    }
}

fn load_maps(
    assets_dir: &Path,
    decl: &ContentFolderDecl,
    registry: &mut Registry,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ldtk") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let id = map_id_from_path(rel.to_str().unwrap_or_default());
        if let Some(existing) = registry.maps.get(&id) {
            errors.push(LintError {
                kind: LintErrorKind::DuplicateId,
                file: rel.display().to_string(),
                message: format!(
                    "id de carte « {id} » déjà défini dans {}",
                    existing.file.display()
                ),
            });
            continue;
        }
        registry.maps.insert(id.clone(), MapEntry { id, file: rel });
    }
}

/// `Ui`/`Camera` : seule la validité syntaxique RON est vérifiée (pas de schéma typé,
/// ces fichiers ne sont référencés par id par aucun autre contenu aujourd'hui).
fn load_generic_ron(
    assets_dir: &Path,
    decl: &ContentFolderDecl,
    into: &mut Vec<PathBuf>,
    errors: &mut Vec<LintError>,
) {
    let files = match discover_files(assets_dir, decl, "ron") {
        Ok(f) => f,
        Err(e) => {
            errors.push(e);
            return;
        }
    };

    for rel in files {
        let text = match read_file(assets_dir, &rel) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        match ron::from_str::<ron::Value>(&text) {
            Ok(_) => into.push(rel),
            Err(e) => errors.push(LintError {
                kind: LintErrorKind::Parse,
                file: rel.display().to_string(),
                message: format!("erreur RON : {e}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_kinds_cover_dispatch_table() {
        let kinds = known_content_kinds();
        for name in KNOWN_KIND_NAMES {
            assert!(kinds.has("content_folder", name));
        }
        assert!(!kinds.has("content_folder", "Npc"));
    }

    #[test]
    fn map_id_from_path_strips_dir_and_extension() {
        assert_eq!(
            map_id_from_path("exemples/test_map.ldtk").as_str(),
            "test_map"
        );
        assert_eq!(
            map_id_from_path("testbed/testbed_empty.ldtk").as_str(),
            "testbed_empty"
        );
    }
}
