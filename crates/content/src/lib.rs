pub mod expr;
pub mod feedback;
pub mod lint;
pub mod manifest;
pub mod registry;
pub mod ui;
pub mod value;

pub use expr::{BoolExpr, Context, Expr, Kind, NumExpr, ParseError, Value};
pub use lint::{LintError, LintErrorKind};
pub use manifest::{ContentFolderDecl, EntryMode, EntryPoint, GameManifest, ManifestError};
pub use registry::{
    known_content_kinds, CharacterId, EnemyId, MapId, MeleeWeaponId, PowerUpId, Registry,
    WaveConfigId, WeaponId,
};

/// Charge le manifeste, construit le registre et lint le contenu d'un jeu en un seul
/// appel (utilisé par `bin/alacod.rs`, `crates/game` au démarrage et le rechargement à
/// chaud). Retourne toutes les erreurs accumulées (chargement puis lint sémantique),
/// jamais seulement la première : voir `docs/taches.md` T1.5.
pub fn load_and_lint(
    game_dir: &std::path::Path,
) -> Result<(Registry, GameManifest, Vec<LintError>), ManifestError> {
    let manifest = GameManifest::load(game_dir)?;
    let (registry, mut errors) = Registry::build(game_dir, &manifest);
    errors.extend(lint::run(&registry, &manifest));
    Ok((registry, manifest, errors))
}
