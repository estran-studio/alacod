//! Vocabulaire libre et léger pour marquer des entités ou des configs de contenu
//! (ex. `tags: ["froid", "arbres"]` sur une salle, `docs/plan-engine.md` §4.3). Ensemble
//! ordonné (`BTreeSet`) : jamais de `HashSet` (CLAUDE.md, règle 5 — déterminisme).

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// Une étiquette. Type newtype transparent en (dé)sérialisation : un champ RON
/// `Vec<String>` ou une liste `["froid", "arbres"]` désérialise directement en `Tag`
/// pour chaque élément, sans wrapper explicite dans le RON.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag(pub String);

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Tag(value.to_string())
    }
}

impl From<String> for Tag {
    fn from(value: String) -> Self {
        Tag(value)
    }
}

/// Ensemble ordonné de [`Tag`]. Se désérialise depuis une liste RON de chaînes
/// (`["froid", "arbres"]`) grâce à la désérialisation « newtype » transparente de `Tag`
/// et de `Tags` elle-même (voir le test `deserializes_from_ron_string_list`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tags(BTreeSet<Tag>);

impl Tags {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn has(&self, tag: &Tag) -> bool {
        self.0.contains(tag)
    }

    /// Renvoie `true` si le tag n'était pas déjà présent (comme `BTreeSet::insert`).
    pub fn insert(&mut self, tag: Tag) -> bool {
        self.0.insert(tag)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tag> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Construit un ensemble de tags depuis une liste de noms, typiquement le
    /// `Vec<String>` d'un champ RON de contenu déjà désérialisé ailleurs.
    pub fn parse<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self(names.into_iter().map(|s| Tag(s.into())).collect())
    }
}

impl FromIterator<Tag> for Tags {
    fn from_iter<I: IntoIterator<Item = Tag>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a Tags {
    type Item = &'a Tag;
    type IntoIter = std::collections::btree_set::Iter<'a, Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_has() {
        let mut tags = Tags::new();
        assert!(!tags.has(&Tag::new("froid")));
        assert!(tags.insert(Tag::new("froid")));
        assert!(tags.has(&Tag::new("froid")));
        assert!(
            !tags.insert(Tag::new("froid")),
            "déjà présent, doit renvoyer false"
        );
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn parse_from_names() {
        let tags = Tags::parse(["froid", "arbres"]);
        assert!(tags.has(&Tag::new("arbres")));
        assert!(tags.has(&Tag::new("froid")));
        assert_eq!(tags.len(), 2);
        assert!(!tags.is_empty());
    }

    #[test]
    fn deserializes_from_ron_string_list() {
        let tags: Tags = ron::from_str(r#"["froid", "arbres"]"#).expect("RON valide");
        assert!(tags.has(&Tag::new("froid")));
        assert!(tags.has(&Tag::new("arbres")));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn iteration_is_alphabetically_ordered() {
        let tags = Tags::parse(["zebre", "arbre", "mouton"]);
        let names: Vec<&str> = tags.iter().map(|t| t.0.as_str()).collect();
        assert_eq!(names, vec!["arbre", "mouton", "zebre"]);
    }
}
