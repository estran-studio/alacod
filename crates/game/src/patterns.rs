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
    }
}

/// Bibliothèque des patterns nommés du registre.
pub fn build_pattern_library(registry: &Registry) -> PatternLibrary {
    PatternLibrary {
        patterns: registry
            .patterns
            .iter()
            .map(|(id, entry)| {
                (
                    id.as_str().to_string(),
                    Arc::new(pattern_from_entry(&entry.pattern)),
                )
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
