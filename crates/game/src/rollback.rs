//! Réexport de l'extension rollback : l'implémentation vit dans `utils` (voir sa doc)
//! parce qu'`animation` en a besoin sans dépendre de `game`. Réexporté ici pour que les
//! sites de `game` continuent d'écrire `use crate::rollback::RollbackTraceApp;`.

pub use utils::rollback::*;

/// Couverture de `TracedTypes`/`StateTracers` (plan §9.6, tâche T0.1c) : chaque type
/// enregistré par `RollbackTraceApp` doit avoir un tracer, qu'il soit passé par
/// `rollback_and_trace{,_debug}` (composant) ou `rollback_and_trace{,_debug}_resource` /
/// `rollback_and_trace_copy_resource{,_no_checksum}` (ressource). Vit dans `game` (et pas
/// `utils`, où vit le mécanisme) car T0.1c demande ce test précisément ici ; il exerce le
/// même code que le jeu utilise, via ce réexport.
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component, Clone, Hash, Debug)]
    struct CompHash;

    #[derive(Component, Clone, Debug)]
    struct CompDebugOnly;

    #[derive(Resource, Clone, Hash, Debug, Default)]
    struct ResHash;

    #[derive(Resource, Clone, Debug, Default)]
    struct ResDebugOnly;

    #[derive(Resource, Clone, Copy, Hash, Debug, Default)]
    struct ResCopy;

    #[derive(Resource, Clone, Copy, Debug, Default)]
    struct ResCopyNoChecksum;

    #[test]
    fn chaque_type_trace_a_un_tracer() {
        let mut app = App::new();
        app.rollback_and_trace::<CompHash>();
        app.rollback_and_trace_debug::<CompDebugOnly>();
        app.rollback_and_trace_resource::<ResHash>();
        app.rollback_and_trace_debug_resource::<ResDebugOnly>();
        app.rollback_and_trace_copy_resource::<ResCopy>();
        app.rollback_and_trace_copy_resource_no_checksum::<ResCopyNoChecksum>();

        let traced = app.world().resource::<TracedTypes>();
        let tracers = app.world().resource::<StateTracers>();

        assert_eq!(
            traced.0.len(),
            6,
            "les six méthodes de RollbackTraceApp doivent enregistrer un type chacune, trouvé {:?}",
            traced.0
        );
        assert_eq!(
            tracers.components.len() + tracers.resources.len(),
            traced.0.len(),
            "chaque type de TracedTypes doit avoir exactement un tracer (composant ou ressource)"
        );
        for name in &traced.0 {
            let has_tracer = tracers.components.iter().any(|(n, _)| n == name)
                || tracers.resources.iter().any(|(n, _)| n == name);
            assert!(
                has_tracer,
                "{name} enregistré dans TracedTypes mais sans tracer dans StateTracers"
            );
        }
    }
}
