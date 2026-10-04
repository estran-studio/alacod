//! Vocabulaire de combat de l'engine : équipes, dégâts, statuts, projectiles, grille spatiale.
//!
//! - [`grid`] : grille spatiale pour retrouver vite les entités proches d'une zone.
//! - [`team`] : règles d'équipe (qui peut viser qui), au-dessus du type `Team` de `sim_core`.
//! - [`damage`] : résolution des dégâts (`resolve_damage`, `Defenses`), au-dessus de
//!   `DamageEvent`/`FriendlyFire` de `sim_core` (T1.1, chantier B1 « Équipes et dégâts »).
//! - [`downed`] : contrats « à terre » et réanimation (`Downed`, `Reviving`), T1.3,
//!   chantier B6.
//! - [`weapons`] : tir, munitions, inventaire et mêlée, déplacés de game en T1.0a.
//! - [`actors`] / [`collider`] / [`collision_grid`] : données partagées requises par les armes.
//! - [`projectile`] : projectiles composables (T1.1, B5 v1) — modificateurs, `on_hit`,
//!   `on_expire` ; systèmes dans `RollbackSystemSet::Projectiles`, montés par
//!   `weapons::BaseWeaponGamePlugin`.
//! - [`status`] : squelette M1 (T1.3 l'exécutera dans `RollbackSystemSet::Status`).
//!
//! Kinds : catégories snake_case, noms de variantes Rust exacts (PascalCase).
//!
//! - [`inventory`] : réserves de munitions par type (`AmmoReserves`), T2.2, chantier B7.

pub mod damage;
pub mod downed;
pub mod grid;
pub mod inventory;
pub mod team;

pub mod actors;
pub mod collider;
pub mod collision_grid;
pub mod weapons;

/// Émetteurs : patterns joués dans le temps et tir ennemi (T1.2).
pub mod emitter;
/// Contrats de M1 (B3/B5). `projectile` est exécuté depuis T1.1 ; `status` attend T1.3.
/// Kinds : catégories snake_case, noms de variantes Rust exacts (PascalCase).
pub mod projectile;
pub mod status;
pub use projectile::{
    ExpireAction, Pattern, Projectile, ProjectileDef, ProjectileModifier, ProjectileSpec,
};
pub use status::{StatusDef, StatusEntry, StatusLibrary, StatusSpec, Statuses};

use bevy::prelude::{App, Plugin};
use sim_core::kinds::{KindDecl, KindRegistry};
use utils::rollback::RollbackTraceApp;

/// Monte le vocabulaire et enregistre l'état rollback, sans poser de composant.
pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        for (category, names) in [
            (
                "projectile_modifier",
                &["Bounce", "Pierce", "Size", "Lifetime", "Homing", "Gravity"][..],
            ),
            (
                "pattern",
                &[
                    "Aimed",
                    "Spread",
                    "Ring",
                    "Sequence",
                    "Telegraph",
                    "Wait",
                    "Scatter",
                    "Named",
                ][..],
            ),
            ("status", &["Burn", "Slow", "Stun", "Freeze"][..]),
        ] {
            app.register_kinds(names.iter().map(|name| KindDecl::new(category, *name)));
        }
        // T1.3 (§19) : enregistrement de T1.0a gardé tel quel (rollback + checksum) : les traces
        // de référence l'incluent déjà, sans porteur ; le passer en variante neutre changerait
        // le checksum de toutes les frames de tous les scénarios.
        app.rollback_and_trace::<Statuses>();
        app.init_resource::<StatusLibrary>();
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use sim_core::kinds::Kinds;
    use utils::rollback::{StateTracers, TracedTypes};
    #[test]
    fn plugin_registers_contract_kinds_and_rollback_state() {
        let mut app = App::new();
        app.add_plugins(CombatPlugin);
        let kinds = app.world().resource::<Kinds>();
        for name in ["Bounce", "Pierce", "Size", "Lifetime", "Homing", "Gravity"] {
            assert!(kinds.has("projectile_modifier", name));
        }
        for name in [
            "Aimed",
            "Spread",
            "Ring",
            "Sequence",
            "Telegraph",
            "Wait",
            "Scatter",
            "Named",
        ] {
            assert!(kinds.has("pattern", name));
        }
        for name in ["Burn", "Slow", "Stun", "Freeze"] {
            assert!(kinds.has("status", name));
        }
        assert_eq!(kinds.names("projectile_modifier").count(), 6);
        assert_eq!(kinds.names("pattern").count(), 8);
        assert_eq!(kinds.names("status").count(), 4);
        let name = std::any::type_name::<Statuses>();
        assert!(app.world().resource::<TracedTypes>().0.contains(&name));
        assert!(app
            .world()
            .resource::<StateTracers>()
            .components
            .iter()
            .any(|(n, _)| *n == name));
    }
}
