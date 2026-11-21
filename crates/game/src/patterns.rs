//! Patterns nommés (T1.2, kind de contenu `Pattern`, `docs/conventions.md` §20) : la
//! [`PatternLibrary`] de `combat` est construite depuis le registre de contenu
//! (`content::registry::Registry::patterns`, fichiers `patterns/<nom>.ron`), une fois au
//! lancement de la partie, comme l'équilibrage (`crate::balance`). Ressource hors rollback,
//! identique sur tous les clients (même contenu) : elle résout `Pattern::Named` au départ
//! d'un émetteur ennemi et à l'expiration d'un projectile.

use std::sync::Arc;

use bevy::prelude::*;
use combat::projectile::{Pattern, PatternLibrary};
use content::registry::{PatternEntry, Registry};

/// Convertit le mirroir de lint (`content`) en pattern de simulation (`combat`).
pub fn pattern_from_entry(entry: &PatternEntry) -> Pattern {
    match entry {
        PatternEntry::Aimed {
            count,
            spread,
            projectile,
        } => Pattern::Aimed {
            count: *count,
            spread: spread.get(),
            projectile: projectile.clone(),
        },
        PatternEntry::Spread {
            count,
            spread,
            projectile,
        } => Pattern::Spread {
            count: *count,
            spread: spread.get(),
            projectile: projectile.clone(),
        },
        PatternEntry::Ring {
            count,
            speed,
            projectile,
            every,
        } => Pattern::Ring {
            count: *count,
            speed: speed.get(),
            projectile: projectile.clone(),
            every: *every,
        },
        PatternEntry::Sequence(children) => {
            Pattern::Sequence(children.iter().map(pattern_from_entry).collect())
        }
        PatternEntry::Telegraph(frames) => Pattern::Telegraph(*frames),
        PatternEntry::Wait(frames) => Pattern::Wait(*frames),
        PatternEntry::Scatter {
            count,
            spread,
            projectile,
        } => Pattern::Scatter {
            count: *count,
            spread: spread.get(),
            projectile: projectile.clone(),
        },
        PatternEntry::Named(name) => Pattern::Named(name.clone()),
        PatternEntry::Fan {
            count,
            spread,
            speed,
            projectile,
        } => Pattern::Fan {
            count: *count,
            spread: spread.get(),
            speed: speed.get(),
            projectile: projectile.clone(),
        },
        PatternEntry::Spiral {
            count,
            step,
            shots,
            every,
            speed,
            projectile,
        } => Pattern::Spiral {
            count: *count,
            step: step.get(),
            shots: *shots,
            every: *every,
            speed: speed.get(),
            projectile: projectile.clone(),
        },
        PatternEntry::Burst {
            count,
            per_burst,
            interval,
            spread,
            projectile,
        } => Pattern::Burst {
            count: *count,
            per_burst: *per_burst,
            interval: *interval,
            spread: spread.get(),
            projectile: projectile.clone(),
        },
    }
}

/// Étapes d'un pattern, `Sequence` imbriquées dépliées dans l'ordre (comme l'émetteur).
fn flatten(pattern: &Pattern, out: &mut Vec<Pattern>) {
    match pattern {
        Pattern::Sequence(children) => children.iter().for_each(|child| flatten(child, out)),
        other => out.push(other.clone()),
    }
}

/// X-6e (`docs/conventions.md` § 127) : décharge calée sur les départs d'un clip. Un pattern lié à un clip par un modèle
/// (`pattern_clips`) dont le manifeste donne `moments.<clip>.departs` (déjà en images de simulation entières, depuis le début
/// du clip) voit ses étapes de tir datées : la `k`-ième part `departs[k]` frames après la première étape de l'émetteur
/// (`Wait` intercalés). **Le contenu prime** : un pattern qui porte déjà un `Wait` ou un `Telegraph` n'est jamais touché, ni
/// un pattern dont le nombre d'étapes diffère du nombre de départs (le lint le refuse). `None` : rien à dériver.
pub fn timed_on_departs(pattern: &Pattern, departs: &[u32]) -> Option<Pattern> {
    let mut steps = Vec::new();
    flatten(pattern, &mut steps);
    if departs.is_empty()
        || steps.len() != departs.len()
        || steps
            .iter()
            .any(|step| matches!(step, Pattern::Wait(_) | Pattern::Telegraph(_)))
    {
        return None;
    }
    let mut timed = Vec::with_capacity(steps.len() * 2);
    let mut at = 0;
    for (step, depart) in steps.into_iter().zip(departs) {
        let wait = depart.saturating_sub(at);
        if wait > 0 {
            timed.push(Pattern::Wait(wait));
        }
        timed.push(step);
        at = at.max(*depart);
    }
    Some(Pattern::Sequence(timed))
}

/// Bibliothèque des patterns nommés du registre. X-6e : les patterns liés à un clip à départs sont datés
/// ([`timed_on_departs`]).
pub fn build_pattern_library(registry: &Registry) -> PatternLibrary {
    let departs = registry
        .models
        .as_ref()
        .map(|table| table.pattern_departs())
        .unwrap_or_default();
    PatternLibrary {
        patterns: registry
            .patterns
            .iter()
            .map(|(id, entry)| {
                let pattern = pattern_from_entry(&entry.pattern);
                let pattern = departs
                    .get(id.as_str())
                    .and_then(|frames| timed_on_departs(&pattern, frames))
                    .unwrap_or(pattern);
                (id.as_str().to_string(), Arc::new(pattern))
            })
            .collect(),
    }
}

/// `OnEnter(AppState::GameLoading)` : (re)construit la bibliothèque depuis le registre
/// courant (un rechargement à chaud du contenu entre deux parties est donc pris en compte).
/// Sans registre (aucun cas aujourd'hui), bibliothèque vide.
pub fn resolve_pattern_library_system(mut commands: Commands, registry: Option<Res<Registry>>) {
    let library = registry
        .map(|registry| build_pattern_library(&registry))
        .unwrap_or_default();
    commands.insert_resource(library);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fixed::fixed_math::Fixed;

    fn shot() -> Pattern {
        Pattern::Spread {
            count: 3,
            spread: Fixed::ZERO,
            projectile: "plomb".into(),
        }
    }

    /// X-6e : deux tirs datés sur les départs (3, 15) du clip : `Wait(3)`, tir, `Wait(12)`, tir.
    #[test]
    fn decharge_datee_sur_les_departs() {
        let pattern = Pattern::Sequence(vec![shot(), shot()]);
        assert_eq!(
            timed_on_departs(&pattern, &[3, 15]),
            Some(Pattern::Sequence(vec![
                Pattern::Wait(3),
                shot(),
                Pattern::Wait(12),
                shot(),
            ]))
        );
        // Départ à 0 : pas de `Wait` devant ; deux départs égaux : le second tir part la même frame.
        assert_eq!(
            timed_on_departs(&pattern, &[0, 0]),
            Some(Pattern::Sequence(vec![shot(), shot()]))
        );
    }

    /// Le contenu prime : un `Wait`/`Telegraph` écrit, ou un nombre de tirs différent, n'est pas daté.
    #[test]
    fn le_contenu_prime() {
        let waited = Pattern::Sequence(vec![shot(), Pattern::Wait(5), shot()]);
        assert_eq!(timed_on_departs(&waited, &[3, 15]), None);
        let pattern = Pattern::Sequence(vec![shot(), shot()]);
        assert_eq!(timed_on_departs(&pattern, &[3]), None);
        assert_eq!(timed_on_departs(&pattern, &[]), None);
    }
}
