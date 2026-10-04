//! Statuts (T1.3, kind de contenu `Status`, `docs/conventions.md` §19) : la
//! [`StatusLibrary`] de `combat` est construite depuis le registre de contenu
//! (`content::registry::Registry::statuses`, fichiers `statuses/<id>.ron`), une fois au
//! lancement de la partie, comme la bibliothèque de patterns (`crate::patterns`). Ressource
//! hors rollback, identique sur tous les clients (même contenu).

use bevy::prelude::*;
use combat::status::{StatusDef, StatusLibrary, StatusSpec};
use content::registry::{Registry, StatusKindEntry};

/// Bibliothèque des statuts du registre.
pub fn build_status_library(registry: &Registry) -> StatusLibrary {
    StatusLibrary {
        statuses: registry
            .statuses
            .iter()
            .map(|(id, entry)| {
                let kind = match entry.kind {
                    StatusKindEntry::Burn => StatusDef::Burn,
                    StatusKindEntry::Slow => StatusDef::Slow,
                    StatusKindEntry::Stun => StatusDef::Stun,
                    StatusKindEntry::Freeze => StatusDef::Freeze,
                };
                (
                    id.as_str().to_string(),
                    StatusSpec {
                        kind,
                        frames: entry.frames,
                        damage: entry.damage,
                        period: entry.period,
                        factor: entry.factor,
                    },
                )
            })
            .collect(),
    }
}

/// `OnEnter(AppState::GameLoading)` : (re)construit la bibliothèque depuis le registre courant.
/// Sans registre, bibliothèque vide.
pub fn resolve_status_library_system(mut commands: Commands, registry: Option<Res<Registry>>) {
    let library = registry
        .map(|registry| build_status_library(&registry))
        .unwrap_or_default();
    commands.insert_resource(library);
}
