//! Vocabulaire minimal d'actions de ramassage (T2.5, chantier C1 v0, `docs/taches.md`).
//!
//! C'est la graine du futur système de déclencheurs/conditions/actions décrit par
//! `docs/plan-engine.md` §5 (« C1. Effets ») : ce crate ne porte **que** les actions elles-
//! mêmes ([`actions::Action`]), pas de déclencheur ni de condition. Crate volontairement
//! minimal (comme `run`, voir sa doc) : il ne dépend que de `sim_core`/`bevy_fixed`, pas de
//! `bevy` — `Action` est une donnée pure, intégrée dans la définition d'un power-up
//! (`game::powerups::PowerUpDef::actions`), jamais un composant en soi. L'application d'une
//! action à l'état ECS (joueurs, fenêtres, ennemis) vit dans `crates/game/src/powerups.rs`,
//! qui a accès à ces types de gameplay — exactement comme `run::currency::Currency` (donnée)
//! et `game::economy::award_points_system` (application) se partagent le travail.

pub mod actions;

pub use actions::{Action, CURRENCY_MULTIPLIER_STAT};
