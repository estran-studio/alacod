//! Types de champ partagés par les schémas de contenu (`registry.rs`).
//!
//! Le RON n'a qu'un seul type numérique ; le projet distingue les entiers nus (frames,
//! tailles de chargeur...) des valeurs `Fixed` (toujours écrites en chaîne, voir
//! `docs/conventions.md` §2 « Fixed-point »). [`FixedField`] applique cette règle au
//! chargement du contenu : un littéral RON nu (`1.5` ou `100`) dans un champ `Fixed`
//! produit un message clair au lieu d'un rejet générique de (dé)sérialisation.

use bevy_fixed::fixed_math::Fixed;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;

/// Une valeur `Fixed` lue depuis le contenu. N'accepte qu'une chaîne RON (`"1.5"`),
/// comme le type `Fixed` réel de la simulation (`fixed` crate, feature `serde-str`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedField(pub Fixed);

impl FixedField {
    pub fn get(self) -> Fixed {
        self.0
    }
}

impl<'de> Deserialize<'de> for FixedField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FixedVisitor;

        impl Visitor<'_> for FixedVisitor {
            type Value = FixedField;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "une valeur Fixed écrite en chaîne, ex. \"1.5\"")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<FixedField, E> {
                v.trim().parse::<Fixed>().map(FixedField).map_err(|_| {
                    E::custom(format!(
                        "valeur Fixed invalide \"{v}\" (attendu un nombre décimal en chaîne)"
                    ))
                })
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<FixedField, E> {
                self.visit_str(&v)
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<FixedField, E> {
                Err(E::custom(format!(
                    "flottant littéral {v} là où une valeur Fixed est attendue : écrire \"{v}\" (chaîne) ; voir CLAUDE.md §Déterminisme, règle 1"
                )))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<FixedField, E> {
                Err(E::custom(format!(
                    "nombre littéral {v} là où une valeur Fixed est attendue : écrire \"{v}\" (chaîne) ; voir CLAUDE.md §Déterminisme, règle 1"
                )))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<FixedField, E> {
                Err(E::custom(format!(
                    "nombre littéral {v} là où une valeur Fixed est attendue : écrire \"{v}\" (chaîne) ; voir CLAUDE.md §Déterminisme, règle 1"
                )))
            }
        }

        deserializer.deserialize_any(FixedVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_quoted_string() {
        let v: FixedField = ron::from_str("\"1.5\"").unwrap();
        assert_eq!(v.get(), Fixed::from_num(1.5));
    }

    #[test]
    fn rejects_bare_float() {
        let err = ron::from_str::<FixedField>("1.5").unwrap_err();
        assert!(
            err.to_string().contains("flottant littéral"),
            "message inattendu: {err}"
        );
    }

    #[test]
    fn rejects_bare_integer() {
        let err = ron::from_str::<FixedField>("100").unwrap_err();
        assert!(
            err.to_string().contains("littéral"),
            "message inattendu: {err}"
        );
    }
}
