//! Scénarios de jeu scriptés et déterministes.
//!
//! Un scénario décrit une partie (map, seed, un script d'inputs par joueur, nombre de
//! frames) et des attentes sur l'état du jeu. Il se joue en headless, sans fenêtre ni
//! GPU, et produit une trace d'état comparée à une trace de référence
//! (`tests/scenarios/<nom>.trace`) pour détecter toute régression.

pub mod runner;

/// Format des scénarios (défini dans `game`, qui écrit aussi les enregistrements).
pub use game::replay as format;
pub use game::replay::Scenario;
pub use runner::{run, ScenarioOutcome};
