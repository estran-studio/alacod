//! État de run minimal (T2.3 chantier C5 v1 + T2.4 chantier F1, `docs/plan-engine.md` §5
//! C5 et F1) : monnaie et perks par joueur ([`currency`], [`perks`]), et l'état de la
//! partie elle-même — mode, étape, résumé ([`run`], [`modes`]). Crate volontairement
//! minimal (dépend seulement de `sim_core`, `bevy_fixed`, `utils`, `bevy` sans rendu, pas
//! de `content` ni de `game`) : les systèmes qui *produisent* de la monnaie (points de
//! kill/coup/réparation), qui la *dépensent* (portes, armes murales, perks), qui résolvent
//! le mode du manifeste ou qui détectent la fin de partie (défaite, victoire) vivent dans
//! `game`, qui a accès aux types de gameplay (`Death`, `DamageEvent`, `WaveState`,
//! `Interactable`...) que ce crate ne connaît pas. Voir [`currency::RunPlugin`] et
//! `game::run` (le module qui branche [`Run`]/[`RunModeRules`] sur la simulation).

pub mod currency;
pub mod floors;
pub mod modes;
pub mod perks;
pub mod run;

pub use currency::{Currency, CurrencyEvent, RunPlugin};
pub use floors::FloorState;
pub use modes::{RunContext, RunModeRules};
pub use perks::Perks;
pub use run::{Run, RunEnd, RunMode, RunStep, RunSummary};
