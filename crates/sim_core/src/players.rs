//! Nombre de joueurs de la session en cours.

use bevy::prelude::Resource;

/// Nombre de joueurs de la session, renseigné une fois au démarrage
/// (`crates/game/src/args/mod.rs`, `GameArgsPlugin::build`, là où ce nombre est
/// résolu depuis `GameArgs`).
///
/// Ressource **ordinaire, hors rollback** : fixée avant le début de la session et
/// constante pour toute sa durée, elle n'a pas besoin d'être dans l'instantané GGRS ni au
/// checksum. Ne pas l'enregistrer avec `RollbackTraceApp` : ça changerait
/// `tests/scenarios/*.trace` sans raison de gameplay.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PlayersCount(pub usize);
