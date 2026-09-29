//! Branchement Bevy/rollback des contrats de `sim_core::stats`/`sim_core::modifier`
//! (T1.2, chantier B2, `docs/taches.md`).
//!
//! `sim_core` définit les types (`StatId`, `Stats`, `Modifier`, `Modifiers`, `resolve`)
//! sans les poser sur aucune entité ni les enregistrer en rollback (voir la doc de ce
//! crate). Ce crate fait le travail restant :
//!
//! - [`StatsPlugin`] : enregistre `Stats`/`Modifiers` en rollback (`RollbackTraceApp`, donc
//!   dans le `Checksum` GGRS et les traces de référence) et ajoute
//!   [`expire_modifiers_system`] (`RollbackSystemSet::Status`), qui retire les
//!   modificateurs expirés de chaque entité à la frame courante.
//! - [`StatReader`] : `SystemParam` unique pour lire une stat résolue (base + modificateurs
//!   actifs, `sim_core::modifier::resolve`) à la frame courante. Tous les systèmes de
//!   `crates/game` qui lisaient une constante d'équilibrage directement dans une
//!   `CharacterConfig`/`PathfindingConfig`/`WeaponConfig` passent par lui : pas de
//!   résolution ad hoc éparpillée dans le code.
//!
//! `crates/game` dépend de ce crate pour enregistrer [`StatsPlugin`]
//! (`character::mod::BaseCharacterGamePlugin`) et lire les stats depuis ses systèmes de
//! mouvement, d'armes, de santé et d'IA ennemie.

mod plugin;
mod reader;

pub use plugin::{expire_modifiers_system, StatsPlugin};
pub use reader::StatReader;

// Réexports de confort : la plupart des appelants n'ont besoin que de ces types-ci, sans
// dépendre directement de `sim_core`.
pub use sim_core::modifier::{Modifier, ModifierOp, ModifierSource, Modifiers};
pub use sim_core::stats::{StatId, Stats};
