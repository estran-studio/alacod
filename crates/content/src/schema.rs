//! Schémas JSON exhaustifs des fichiers de contenu, dérivés des types Rust eux-mêmes
//! (`schemars`, feature `schema` des crates de contenu) : champs, types, valeurs par défaut,
//! variantes d'enum et commentaires de doc. `alacod describe <Kind>` les joint à l'inventaire
//! du contenu existant (`describe.rs`).
//!
//! Couverture : tous les kinds de `registry::KNOWN_KIND_NAMES` sauf [`UNCOVERED_KINDS`] (test
//! `every_kind_has_a_schema`). Un kind sans schéma garde l'inventaire déduit du contenu. Ajouter un kind = dériver
//! `JsonSchema` sur ses types (`tools/derive_schema.py` fait le plus gros) puis une ligne
//! dans la table.
//!
//! Les valeurs `Fixed` apparaissent comme le type `Fixed` : une **chaîne** RON (`"1.5"`).

use schemars::Schema;
use serde_json::Value;
use std::collections::BTreeSet;

/// Schéma racine d'un fichier du kind (`None` : kind non couvert). La table est dans
/// `registry.rs`, qui voit les types de fichier privés.
pub fn schema_for_kind(kind: &str) -> Option<Schema> {
    crate::registry::schema_for_kind(kind)
}

/// Kinds sans schéma, et pourquoi : `Map` est du LDtk (JSON de l'éditeur), `Ui` et `Camera`
/// sont des fichiers de présentation validés à part.
pub const UNCOVERED_KINDS: &[&str] = &["Map", "Ui", "Camera"];

/// Rendu texte compact : le type racine puis chaque type référencé, une seule fois.
pub fn render(schema: &Schema) -> String {
    let root = schema.as_value();
    let defs = root.get("$defs").and_then(Value::as_object);
    let mut out = String::new();
    let mut seen = BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();

    // Racine qui n'est qu'une référence (newtype d'une table : `weapons.ron`) : la décrire.
    if let Some(name) = ref_name(root) {
        queue.push(name);
    } else {
        out.push_str(&render_type_body(root, &mut queue));
    }
    while let Some(name) = queue.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(def) = defs.and_then(|d| d.get(&name)) else {
            continue;
        };
        out.push_str(&format!("\n{name}\n"));
        out.push_str(&render_type_body(def, &mut queue));
    }
    out
}

fn doc(v: &Value) -> String {
    v.get("description")
        .and_then(Value::as_str)
        .map(|d| {
            let flat = d.split_whitespace().collect::<Vec<_>>().join(" ");
            format!("  — {flat}")
        })
        .unwrap_or_default()
}

fn ref_name(v: &Value) -> Option<String> {
    // schemars encode `<`, `>` et `,` des noms génériques dans le `$ref`.
    v.get("$ref")?
        .as_str()?
        .rsplit('/')
        .next()
        .map(|n| n.replace("%3C", "<").replace("%3E", ">").replace("%2C", ","))
}

/// Nom court d'un type ; empile les types nommés à décrire ensuite.
fn type_name(v: &Value, queue: &mut Vec<String>) -> String {
    if let Some(name) = ref_name(v) {
        queue.push(name.clone());
        return name;
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(arr) = v.get(key).and_then(Value::as_array) {
            let non_null: Vec<&Value> = arr
                .iter()
                .filter(|a| a.get("type").and_then(Value::as_str) != Some("null"))
                .collect();
            if non_null.len() == 1 && arr.len() == 2 {
                return format!("{}?", type_name(non_null[0], queue));
            }
            return non_null
                .iter()
                .map(|a| type_name(a, queue))
                .collect::<Vec<_>>()
                .join(" | ");
        }
    }
    match v.get("type") {
        Some(Value::Array(types)) => {
            let names: Vec<&str> = types.iter().filter_map(Value::as_str).collect();
            let non_null: Vec<&&str> = names.iter().filter(|t| **t != "null").collect();
            let base = if names.contains(&"array") {
                array_name(v, queue)
            } else {
                non_null
                    .first()
                    .map(|t| primitive(t, v))
                    .unwrap_or_default()
            };
            if names.contains(&"null") {
                format!("{base}?")
            } else {
                base
            }
        }
        Some(Value::String(t)) => match t.as_str() {
            "array" => array_name(v, queue),
            "object" => match v.get("additionalProperties") {
                Some(extra) if extra.is_object() => {
                    format!("{{texte: {}}}", type_name(extra, queue))
                }
                _ => "objet".into(),
            },
            other => primitive(other, v),
        },
        _ => "?".into(),
    }
}

fn array_name(v: &Value, queue: &mut Vec<String>) -> String {
    if let Some(prefix) = v.get("prefixItems").and_then(Value::as_array) {
        let parts: Vec<String> = prefix
            .iter()
            .map(|p| {
                let rule = lint_rule(p).map(|r| format!(" [{r}]")).unwrap_or_default();
                format!("{}{rule}", type_name(p, queue))
            })
            .collect();
        format!("({})", parts.join(", "))
    } else if let Some(items) = v.get("items") {
        format!("[{}]", type_name(items, queue))
    } else {
        "[?]".into()
    }
}

fn primitive(t: &str, v: &Value) -> String {
    match t {
        "integer" => match v.get("minimum").and_then(Value::as_i64) {
            Some(0) => "entier ≥ 0".into(),
            _ => "entier".into(),
        },
        "number" => "nombre".into(),
        "boolean" => "booléen".into(),
        "string" => "texte".into(),
        "object" => "objet".into(),
        other => other.into(),
    }
}

fn render_type_body(v: &Value, queue: &mut Vec<String>) -> String {
    let mut out = String::new();
    if let Some(d) = v.get("description").and_then(Value::as_str) {
        let flat = d.split_whitespace().collect::<Vec<_>>().join(" ");
        out.push_str(&format!("  {flat}\n"));
    }
    if let Some(props) = v.get("properties").and_then(Value::as_object) {
        let required: BTreeSet<&str> = v
            .get("required")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        for (name, prop) in props {
            let ty = type_name(prop, queue);
            let req = if required.contains(name.as_str()) {
                "requis"
            } else {
                "optionnel"
            };
            let default = prop
                .get("default")
                .map(|d| format!(", défaut {d}"))
                .unwrap_or_default();
            let lint = lint_rule(prop)
                .map(|r| format!(", lint : {r}"))
                .unwrap_or_default();
            out.push_str(&format!(
                "    {name}: {ty}  [{req}{default}{lint}]{}\n",
                doc(prop)
            ));
        }
        return out;
    }
    if let Some(extra) = v.get("additionalProperties").filter(|e| e.is_object()) {
        out.push_str(&format!("    \"<id>\": {}\n", type_name(extra, queue)));
        return out;
    }
    if let Some(values) = v.get("enum").and_then(Value::as_array) {
        for value in values {
            out.push_str(&format!("    {}\n", value.as_str().unwrap_or("?")));
        }
        return out;
    }
    if let Some(variants) = v.get("oneOf").or_else(|| v.get("anyOf")).and_then(Value::as_array) {
        for variant in variants {
            out.push_str(&render_variant(variant, queue));
        }
        return out;
    }
    if v.get("type").is_some() && v.get("description").is_none() {
        out.push_str(&format!("    {}\n", type_name(v, queue)));
    }
    out
}

fn render_variant(variant: &Value, queue: &mut Vec<String>) -> String {
    // Variante unitaire : `{"const": "OnHit"}`.
    if let Some(name) = variant.get("const").and_then(Value::as_str) {
        return format!("    {name}{}\n", doc(variant));
    }
    if let Some(names) = variant.get("enum").and_then(Value::as_array) {
        return names
            .iter()
            .filter_map(Value::as_str)
            .map(|n| format!("    {n}{}\n", doc(variant)))
            .collect();
    }
    // Variante à données : `{"properties": {"Nom": payload}}` (enum externe).
    if let Some(props) = variant.get("properties").and_then(Value::as_object) {
        if props.len() == 1 {
            let (name, payload) = props.iter().next().unwrap();
            return format!("    {name}{}{}\n", payload_shape(payload, queue), doc(variant));
        }
    }
    format!("    {}\n", type_name(variant, queue))
}

/// Règle que le lint ajoute au type (`x-lint`, posée par `#[schemars(extend(...))]` sur le champ).
fn lint_rule(v: &Value) -> Option<&str> {
    v.get("x-lint").and_then(Value::as_str)
}

/// `(A, B)` pour un tuple, `(A)` pour un newtype, `{ champ: T, … }` pour un struct.
fn payload_shape(payload: &Value, queue: &mut Vec<String>) -> String {
    if let Some(props) = payload.get("properties").and_then(Value::as_object) {
        let required: BTreeSet<&str> = payload
            .get("required")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let fields: Vec<String> = props
            .iter()
            .map(|(n, p)| {
                let opt = if required.contains(n.as_str()) { "" } else { "?" };
                let rule = lint_rule(p).map(|r| format!(" [{r}]")).unwrap_or_default();
                format!("{n}{opt}: {}{rule}", type_name(p, queue))
            })
            .collect();
        return format!(" {{ {} }}", fields.join(", "));
    }
    let ty = type_name(payload, queue);
    if ty.starts_with('(') {
        ty
    } else {
        let rule = lint_rule(payload)
            .map(|r| format!(" [{r}]"))
            .unwrap_or_default();
        format!("({ty}{rule})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::KNOWN_KIND_NAMES;

    #[test]
    fn every_kind_has_a_schema() {
        for kind in KNOWN_KIND_NAMES {
            let covered = schema_for_kind(kind).is_some();
            let excluded = UNCOVERED_KINDS.contains(kind);
            assert!(
                covered != excluded,
                "kind {kind} : {} (ajouter le kind à registry::schema_for_kind, ou à UNCOVERED_KINDS avec la raison)",
                if covered { "a un schéma mais est listé non couvert" } else { "sans schéma" }
            );
        }
    }

    #[test]
    fn every_schema_renders_fields() {
        // Un schéma dont le rendu est vide (racine en simple référence, par exemple) ne dit rien.
        for kind in KNOWN_KIND_NAMES {
            let Some(schema) = schema_for_kind(kind) else {
                continue;
            };
            let text = render(&schema);
            let fields = text.lines().filter(|l| l.contains(": ")).count();
            assert!(fields >= 1, "kind {kind} : schéma sans aucun champ rendu :\n{text}");
        }
    }

    #[test]
    fn fixed_fields_are_strings_in_the_schema() {
        let text = render(&schema_for_kind("Item").unwrap());
        assert!(text.contains("pickup_range: Fixed"), "{text}");
        assert!(text.contains("Nombre décimal écrit en chaîne"), "{text}");
    }

    #[test]
    fn enum_variants_are_listed() {
        let text = render(&schema_for_kind("Item").unwrap());
        for variant in ["Passive", "Active(ActiveCharge)", "OnPickup", "Heal(Fixed [> 0])", "Legendary"] {
            assert!(text.contains(variant), "variante {variant} absente :\n{text}");
        }
    }
}
