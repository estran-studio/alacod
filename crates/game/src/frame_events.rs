//! Réexport : voir `sim_core::frame_events`. Le type déménage dans `sim_core` en T0.2
//! (`docs/taches.md`) pour que les futurs crates de vocabulaire (`combat`, `effects`,
//! `behaviors`...) puissent l'utiliser sans dépendre de `game`. Réexporté ici pour que les
//! sites existants gardent `use crate::frame_events::...` (même idiome que
//! `crate::rollback`, qui réexporte `utils::rollback`).

pub use sim_core::frame_events::*;
