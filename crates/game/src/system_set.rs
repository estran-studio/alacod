//! Réexport : voir `sim_core::system_set`. Déménagé dans `sim_core` en T0.2
//! (`docs/taches.md`) et complété (`Projectiles`, `Effects`, `Status`, `Run`) pour que les
//! futurs crates de vocabulaire (`combat`, `effects`, `behaviors`...) puissent placer
//! leurs systèmes sans dépendre de `game`. Réexporté ici pour que les sites existants
//! gardent `use crate::system_set::RollbackSystemSet` (même idiome que `crate::rollback`,
//! qui réexporte `utils::rollback`). `crates/game/src/core.rs` configure la chaîne
//! d'ordre à partir de `RollbackSystemSet::ORDER`.

pub use sim_core::system_set::*;
