//! Scénarios de jeu scriptés et déterministes.
//!
//! Un scénario décrit une partie (map, seed, un script d'inputs par joueur, nombre de
//! frames) et des attentes sur l'état du jeu. Il se joue en headless, sans fenêtre ni
//! GPU, et produit une trace d'état comparée à une trace de référence
//! (`tests/scenarios/<nom>.trace`) pour détecter toute régression.

pub mod format;
pub mod runner;

pub use format::Scenario;
pub use runner::{run, ScenarioOutcome};
