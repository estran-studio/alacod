//! Registre des « kinds » (types de contenu) déclarés par les plugins de l'engine
//! (armes, ennemis, items...). Lu par le lint de contenu (`crates/content`, T1.5) pour
//! vérifier qu'une référence RON pointe vers un kind connu.
//!
//! `Kinds` est une ressource de configuration, posée une fois au montage des plugins
//! (`Plugin::build`) : elle ne change pas en jeu et n'a pas besoin d'être enregistrée en
//! rollback.

use bevy::app::App;
use bevy::prelude::Resource;
use std::collections::{BTreeMap, BTreeSet};

/// Déclaration d'un kind : une catégorie (`"weapon"`, `"enemy"`, `"item"`...) et un nom.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KindDecl {
    pub category: &'static str,
    pub name: String,
}

impl KindDecl {
    pub fn new(category: &'static str, name: impl Into<String>) -> Self {
        Self {
            category,
            name: name.into(),
        }
    }
}

/// Tous les kinds déclarés, par catégorie. `BTreeMap`/`BTreeSet` pour un ordre
/// déterministe (CLAUDE.md, règle 5), même si `Kinds` n'est pas elle-même posée en
/// rollback : un futur lint ou diagnostic en dérive une sortie reproductible.
#[derive(Resource, Debug, Clone, Default)]
pub struct Kinds(BTreeMap<&'static str, BTreeSet<String>>);

impl Kinds {
    pub fn register(&mut self, decl: KindDecl) {
        self.0.entry(decl.category).or_default().insert(decl.name);
    }

    pub fn has(&self, category: &str, name: &str) -> bool {
        self.0
            .get(category)
            .is_some_and(|names| names.contains(name))
    }

    pub fn categories(&self) -> impl Iterator<Item = &&'static str> {
        self.0.keys()
    }

    pub fn names(&self, category: &str) -> impl Iterator<Item = &String> {
        self.0
            .get(category)
            .into_iter()
            .flat_map(|names| names.iter())
    }
}

/// Extension d'`App` pour qu'un plugin déclare ses kinds au montage (`Plugin::build`).
/// Voir `crates/game/src/weapons/mod.rs` (`BaseWeaponGamePlugin`) et
/// `crates/game/src/character/mod.rs` (`BaseCharacterGamePlugin`) pour des exemples
/// d'enregistrement des kinds existants (armes, ennemis).
pub trait KindRegistry {
    fn register_kinds(&mut self, decls: impl IntoIterator<Item = KindDecl>) -> &mut Self;
}

impl KindRegistry for App {
    fn register_kinds(&mut self, decls: impl IntoIterator<Item = KindDecl>) -> &mut Self {
        if !self.world().contains_resource::<Kinds>() {
            self.init_resource::<Kinds>();
        }
        {
            let mut kinds = self.world_mut().resource_mut::<Kinds>();
            for decl in decls {
                kinds.register(decl);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_query() {
        let mut app = App::new();
        app.register_kinds([
            KindDecl::new("weapon", "shotgun"),
            KindDecl::new("weapon", "pistol"),
            KindDecl::new("enemy", "zombie_1"),
        ]);

        let kinds = app.world().resource::<Kinds>();
        assert!(kinds.has("weapon", "pistol"));
        assert!(!kinds.has("weapon", "unknown"));
        assert!(kinds.has("enemy", "zombie_1"));

        let names: Vec<&String> = kinds.names("weapon").collect();
        assert_eq!(
            names,
            vec!["pistol", "shotgun"],
            "ordre BTreeSet, déterministe"
        );
    }

    #[test]
    fn register_kinds_called_twice_accumulates() {
        let mut app = App::new();
        app.register_kinds([KindDecl::new("weapon", "shotgun")]);
        app.register_kinds([KindDecl::new("weapon", "pistol")]);

        let kinds = app.world().resource::<Kinds>();
        assert_eq!(kinds.names("weapon").count(), 2);
    }

    #[test]
    fn unknown_category_has_no_names() {
        let mut app = App::new();
        app.register_kinds([KindDecl::new("weapon", "shotgun")]);

        let kinds = app.world().resource::<Kinds>();
        assert_eq!(kinds.names("unknown").count(), 0);
        assert!(!kinds.has("unknown", "shotgun"));
    }
}
