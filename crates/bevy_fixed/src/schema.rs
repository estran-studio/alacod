//! Schéma JSON des valeurs `Fixed` du contenu : elles s'écrivent en **chaîne** RON (`"1.5"`),
//! jamais en nombre (CLAUDE.md, règle 1). Les champs `Fixed` des types de contenu portent
//! `#[schemars(with = "bevy_fixed::schema::FixedString")]`.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use std::borrow::Cow;

/// Marqueur de schéma : un nombre décimal écrit en chaîne.
pub struct FixedString;

impl JsonSchema for FixedString {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Fixed")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^-?[0-9]+(\\.[0-9]+)?$",
            "description": "Nombre décimal écrit en chaîne RON, ex. \"1.5\" (jamais un littéral nu).",
            "examples": ["1.5", "-10", "0.25"]
        })
    }
}
