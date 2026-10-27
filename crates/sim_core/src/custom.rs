//! Paramètres d'un verbe `Custom` (X-1, `docs/conventions.md` §91) : `Behavior::Custom`,
//! `Action::Custom` et `Condition::Custom` portent un `id` (déclaré par le plugin du jeu dans
//! `game.ron`, section `plugin`) et ces paramètres **bruts**, décodés par le plugin à l'exécution.
//!
//! Format : une table texte → texte. Les valeurs `Fixed` s'écrivent en chaîne comme partout
//! (`"30.0"`, règle 8 de `CLAUDE.md`), les entiers aussi (`"4"`) : aucun flottant, aucun type
//! RON que le moteur devrait connaître. `BTreeMap` : ordre d'itération, `Hash` et `Ord`
//! déterministes (ces types sont sous checksum quand ils vivent dans un composant rollback).

use bevy_fixed::fixed_math::Fixed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Paramètres bruts d'un verbe `Custom`. Transparent en RON : `params: {"range": "30.0"}`.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CustomParams(pub BTreeMap<String, String>);

impl CustomParams {
    /// Valeur brute de `key`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// `key` lue comme un `Fixed` décimal (`"30.0"`) ; `None` si absente ou illisible.
    pub fn fixed(&self, key: &str) -> Option<Fixed> {
        self.get(key)?.trim().parse::<Fixed>().ok()
    }

    /// `key` lue comme un entier positif (`"4"`) ; `None` si absente ou illisible.
    pub fn uint(&self, key: &str) -> Option<u32> {
        self.get(key)?.trim().parse::<u32>().ok()
    }
}

impl<const N: usize> From<[(&str, &str); N]> for CustomParams {
    fn from(pairs: [(&str, &str); N]) -> Self {
        Self(
            pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ron_round_trip_and_typed_reads() {
        let params: CustomParams =
            ron::from_str(r#"{"range": "30.5", "count": "4", "name": "x"}"#).unwrap();
        assert_eq!(params.fixed("range"), Some(Fixed::from_num(30.5)));
        assert_eq!(params.uint("count"), Some(4));
        assert_eq!(params.get("name"), Some("x"));
        assert_eq!(params.fixed("name"), None);
        assert_eq!(params.uint("absent"), None);
        let back: CustomParams = ron::from_str(&ron::to_string(&params).unwrap()).unwrap();
        assert_eq!(back, params);
    }
}
