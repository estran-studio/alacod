//! [`StatsPlugin`] : enregistrement rollback de `Stats`/`Modifiers` et expiration des
//! modificateurs.

use bevy::prelude::*;
use bevy_ggrs::{GgrsSchedule, Rollback};
use sim_core::modifier::Modifiers;
use sim_core::stats::Stats;
use sim_core::system_set::RollbackSystemSet;
use utils::rollback::RollbackTraceApp;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_mut_iter};

/// Enregistre [`Stats`]/[`Modifiers`] en rollback (`RollbackTraceApp` : rollback bevy_ggrs,
/// `Checksum` GGRS, trace d'état — voir sa doc dans `utils::rollback`) et ajoute
/// [`expire_modifiers_system`] à `GgrsSchedule`, dans `RollbackSystemSet::Status`.
///
/// N'insère aucun `Stats`/`Modifiers` sur aucune entité : ça reste le travail de
/// `character::create::create_character` (T1.2). Ce plugin ne fait que rendre l'état
/// existant visible au rollback/checksum et le tenir à jour (expiration).
pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.rollback_and_trace::<Stats>()
            .rollback_and_trace::<Modifiers>()
            .add_systems(
                GgrsSchedule,
                expire_modifiers_system.in_set(RollbackSystemSet::Status),
            );
    }
}

/// Retire les modificateurs expirés (`Modifiers::retain_active`) de chaque entité, à la
/// frame courante.
///
/// Indépendant par entité (aucune interaction croisée, aucune consommation de ressource
/// partagée dans l'ordre) : l'ordre de parcours n'affecte pas le résultat final. La query
/// trie quand même par `GgrsNetId` (`order_mut_iter!`), par convention du projet pour
/// toute itération mutable d'entités rollback dans `GgrsSchedule` (CLAUDE.md, checklist
/// « Nouveau système GGRS »), et parce que `resolve()` filtre de toute façon les
/// modificateurs expirés lui-même : cette purge est une optimisation mémoire (les
/// `Modifiers` ne grossissent pas sans borne), pas une condition de correction.
///
/// Public pour que `crates/game` (`character::health::sync_health_from_stats`, qui lit
/// aussi `Modifiers` via [`crate::StatReader`]) s'ordonne explicitement après avec
/// `.after(expire_modifiers_system)` : les deux systèmes touchent `Modifiers` dans le même
/// `RollbackSystemSet::Status`, et `ambiguity_detection: Error` du `GgrsSchedule` refuse un
/// ordre implicite entre eux.
pub fn expire_modifiers_system(
    frame: Res<FrameCount>,
    mut query: Query<(&GgrsNetId, &mut Modifiers), With<Rollback>>,
) {
    for (_net_id, mut modifiers) in order_mut_iter!(query) {
        modifiers.retain_active(frame.frame);
    }
}
