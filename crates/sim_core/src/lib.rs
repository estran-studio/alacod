//! `sim_core` : contrats partagés par les futurs crates de simulation (`combat`, `stats`,
//! `effects`, `behaviors`, `world`, `run`). Voir `docs/plan-engine.md` §4-5 et la tâche
//! T0.2 dans `docs/taches.md`.
//!
//! ## Rien ici n'est branché
//!
//! Ce crate ne pose aucun de ses types sur une entité, n'enregistre aucun composant ou
//! ressource en rollback (`utils::rollback::RollbackTraceApp`), et n'ajoute aucun système
//! à `GgrsSchedule` au-delà de ce qui existait déjà (le nettoyage de `FrameEvents` et
//! l'ordre des `RollbackSystemSet`, déménagés tels quels depuis `crates/game`). C'est
//! voulu : tout ce qui touche au checksum GGRS change les traces de référence
//! (`tests/scenarios/*.trace`), et T0.2 ne doit rien y changer. Les chantiers suivants
//! (T1.1 « Équipes et dégâts », T1.2 « Stats branchées », T1.3 « statuts »...) attachent
//! ces contrats à des entités et les enregistrent, avec le `BLESS=1` que ça implique.
//!
//! - [`team`] : équipe d'une entité (composant statique, non rollback — voir sa doc).
//! - [`tag`] : vocabulaire libre ordonné (`Tag`, `Tags`).
//! - [`damage`] : genres de dégâts, `FriendlyFire` et `DamageEvent` (branché depuis T1.1,
//!   voir la doc du module).
//! - [`stats`] : identifiants de stats et composant `Stats` (valeurs de base).
//! - [`modifier`] : modificateurs de stats et leur résolution déterministe.
//! - [`gauge`] : valeur bornée avec plancher et seuils (santé, jauges 1837...).
//! - [`players`] : `PlayersCount`, ressource simple hors rollback.
//! - [`kinds`] : registre des « kinds » de contenu déclarés par les plugins.
//! - [`frame_events`] : `FrameEvents<T>`, déménagé de `crates/game`.
//! - [`system_set`] : `RollbackSystemSet` et son ordre total, déménagé de `crates/game`.
//! - [`ammo`] : type de munition d'une arme à distance (T2.2, chantier B7).

pub mod ammo;
pub mod damage;
pub mod frame_events;
pub mod gauge;
pub mod kinds;
pub mod modifier;
pub mod players;
pub mod stats;
pub mod system_set;
pub mod tag;
pub mod team;
