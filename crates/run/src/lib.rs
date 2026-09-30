//! État de run minimal (T2.3, chantier C5 v1, `docs/plan-engine.md` §5 C5) : monnaie et
//! perks par joueur. Crate volontairement minimal (dépend seulement de `sim_core`,
//! `bevy_fixed`, `utils`, `bevy` sans rendu) : les systèmes qui *produisent* de la monnaie
//! (points de kill/coup/réparation) ou qui la *dépensent* (portes, armes murales, perks)
//! vivent dans `game`, qui a accès aux types de gameplay (`Death`, `DamageEvent`,
//! `Interactable`...) que ce crate ne connaît pas. Voir [`currency::RunPlugin`].

pub mod currency;
pub mod perks;

pub use currency::{Currency, CurrencyEvent, RunPlugin};
pub use perks::Perks;
