//! Vocabulaire d'effets C1 : Action des power-ups (T2.5), puis contrats de M1
//! Effect / On / Condition (T1.0a). Données pures, sans système d'exécution ajouté.
//! L'application des power-ups reste dans game::powerups ; T1.10 exécutera les nouveaux
//! contrats dans RollbackSystemSet::Effects, sans nouveau set.
//! Kinds : catégories snake_case, noms de variantes Rust exacts (PascalCase),
//! références de contenu à valider par T1.12.

pub mod actions;

pub use actions::{Action, CURRENCY_MULTIPLIER_STAT};

pub mod contracts;
use bevy::prelude::{App, Plugin};
pub use contracts::{Condition, Effect, GaugeThreshold, On};
use sim_core::kinds::{KindDecl, KindRegistry};

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        for (category, names) in [
            (
                "effect_trigger",
                &[
                    "OnHit",
                    "OnKill",
                    "OnDamageTaken",
                    "OnDodge",
                    "OnReload",
                    "OnRoomClear",
                    "OnPickup",
                    "OnUse",
                    "OnGauge",
                    "Tick",
                    "OnEvent",
                ][..],
            ),
            (
                "effect_condition",
                &[
                    "TargetTag",
                    "Carrying",
                    "HpBelow",
                    "HasTag",
                    "SquadSize",
                    "TargetInRange",
                    "NotHitFor",
                ][..],
            ),
        ] {
            app.register_kinds(names.iter().map(|name| KindDecl::new(category, *name)));
        }
    }
}
#[cfg(test)]
mod contract_tests {
    use super::*;
    use sim_core::kinds::Kinds;
    #[test]
    fn plugin_registers_contract_kinds() {
        let mut app = App::new();
        app.add_plugins(EffectsPlugin);
        let kinds = app.world().resource::<Kinds>();
        for name in [
            "OnHit",
            "OnKill",
            "OnDamageTaken",
            "OnDodge",
            "OnReload",
            "OnRoomClear",
            "OnPickup",
            "OnUse",
            "OnGauge",
            "Tick",
            "OnEvent",
        ] {
            assert!(kinds.has("effect_trigger", name));
        }
        for name in [
            "TargetTag",
            "Carrying",
            "HpBelow",
            "HasTag",
            "SquadSize",
            "TargetInRange",
            "NotHitFor",
        ] {
            assert!(kinds.has("effect_condition", name));
        }
        assert_eq!(kinds.names("effect_trigger").count(), 11);
        assert_eq!(kinds.names("effect_condition").count(), 7);
    }
}
