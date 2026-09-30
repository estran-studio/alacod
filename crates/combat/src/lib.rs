//! Vocabulaire de combat de l'engine : équipes, dégâts, statuts, projectiles, grille spatiale.
//!
//! - [`grid`] : grille spatiale pour retrouver vite les entités proches d'une zone.
//! - [`team`] : règles d'équipe (qui peut viser qui), au-dessus du type `Team` de `sim_core`.
//! - [`damage`] : résolution des dégâts (`resolve_damage`, `Defenses`), au-dessus de
//!   `DamageEvent`/`FriendlyFire` de `sim_core` (T1.1, chantier B1 « Équipes et dégâts »).
//! - [`downed`] : contrats « à terre » et réanimation (`Downed`, `Reviving`), T1.3,
//!   chantier B6.
//! - [`inventory`] : réserves de munitions par type (`AmmoReserves`), T2.2, chantier B7.

pub mod damage;
pub mod downed;
pub mod grid;
pub mod inventory;
pub mod team;
