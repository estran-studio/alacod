//! Lecture résolue des stats : [`StatReader`].

use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Query, Res};
use bevy_fixed::fixed_math::Fixed;
use sim_core::modifier::{resolve, Modifiers};
use sim_core::stats::{StatId, Stats};
use utils::frame::FrameCount;

/// `SystemParam` unique pour lire une stat résolue à la frame courante
/// (`sim_core::modifier::resolve` : base de [`Stats`] + modificateurs actifs de
/// [`Modifiers`]). Voir la doc du crate : tout système qui lisait jusqu'ici une constante
/// d'équilibrage directement dans une config passe par ce type plutôt que de rappeler
/// `resolve` à la main.
///
/// Requêtes en lecture seule (`Option<&Stats>`, `Option<&Modifiers>`) : coexiste sans
/// conflit avec n'importe quelle autre query du système appelant, y compris une query
/// mutable sur un autre composant de la même entité (`Health`, `Velocity`...).
#[derive(SystemParam)]
pub struct StatReader<'w, 's> {
    frame: Res<'w, FrameCount>,
    query: Query<'w, 's, (Option<&'static Stats>, Option<&'static Modifiers>)>,
}

impl StatReader<'_, '_> {
    /// Valeur résolue de `id` pour `entity`, ou `None` si l'entité n'a pas de composant
    /// [`Stats`] ou que `id` n'y est pas défini (pas de base sur laquelle appliquer des
    /// modificateurs). À utiliser quand l'absence de la stat doit se traduire par « ne rien
    /// faire » plutôt que par une valeur par défaut arbitraire (voir
    /// `character::health::sync_health_from_stats`, qui ne touche pas `Health.max` si
    /// `MaxHealth` est absente plutôt que de réutiliser la valeur déjà en place comme base —
    /// ça la ferait dériver à chaque frame sous un modificateur multiplicatif).
    pub fn try_get(&self, entity: Entity, id: &StatId) -> Option<Fixed> {
        let (stats, modifiers) = self.query.get(entity).ok()?;
        let base = stats?.get(id)?;
        Some(self.resolve(base, id, modifiers))
    }

    /// Comme [`Self::try_get`], avec `default` comme base si l'entité n'a pas de [`Stats`]
    /// ou que `id` n'y est pas défini (les modificateurs actifs s'appliquent quand même
    /// sur ce défaut). En pratique T1.2 pose explicitement toutes les stats utilisées par
    /// l'engine à la création du personnage (`character::create::create_character`), donc
    /// ce chemin ne sert qu'aux entités sans `Stats` du tout (garde défensive).
    pub fn get(&self, entity: Entity, id: &StatId, default: Fixed) -> Fixed {
        let Ok((stats, modifiers)) = self.query.get(entity) else {
            return default;
        };
        let base = stats.and_then(|s| s.get(id)).unwrap_or(default);
        self.resolve(base, id, modifiers)
    }

    fn resolve(&self, base: Fixed, id: &StatId, modifiers: Option<&Modifiers>) -> Fixed {
        match modifiers {
            Some(m) => resolve(
                base,
                m.iter().filter(|modifier| &modifier.stat == id),
                self.frame.frame,
            ),
            None => base,
        }
    }
}
