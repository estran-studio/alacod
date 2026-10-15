//! `alacod new-content <Kind> <id>` : crée une entrée de contenu en copiant un modèle du jeu.
//!
//! - Kind par fichier (dossier déclaré, ex. `objets/`, `rooms/`) : `<dossier>/<id>.ron`, copie
//!   d'un fichier existant.
//! - Kind en table (`weapons.ron`, `powerups.ron`… : `{ "id": (...) }`) : l'entrée modèle est
//!   dupliquée sous la nouvelle clé, à la fin de la table.
//!
//! Le modèle (`--from <id>`, sinon le premier du kind) garantit une entrée qui passe le lint ;
//! seuls l'id et le nom sont réécrits, tout le reste est à adapter (l'agent lit
//! `alacod describe <Kind>` pour savoir quoi). Rien n'est inventé : pas de gabarit propre,
//! donc rien qui puisse dériver du format réel.

use std::path::{Path, PathBuf};

use crate::describe::{self, KindInfo};
use crate::manifest::GameManifest;

/// Résultat d'une création.
#[derive(Debug)]
pub struct Created {
    /// Fichier écrit (relatif au dossier du jeu).
    pub file: PathBuf,
    /// L'entrée est ajoutée à une table existante (`true`) ou c'est un nouveau fichier.
    pub in_table: bool,
    /// Id de l'entrée modèle.
    pub model: String,
}

pub fn new_content(
    game_dir: &Path,
    kind: &str,
    id: &str,
    from: Option<&str>,
) -> Result<Created, String> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        || id.starts_with(|c: char| c.is_ascii_digit())
    {
        return Err(format!(
            "id « {id} » invalide : minuscules, chiffres et `_`, première lettre non numérique"
        ));
    }
    let description = describe::describe(game_dir)?;
    let info = describe::find_kind(&description, kind).ok_or_else(|| {
        let known: Vec<_> = description.kinds.iter().map(|k| k.kind.as_str()).collect();
        format!(
            "kind « {kind} » absent de {} (présents : {})",
            description.game,
            known.join(", ")
        )
    })?;
    if matches!(info.kind.as_str(), "Map" | "Ui" | "Camera" | "Economy" | "Difficulty") {
        return Err(format!(
            "{} : pas de création par modèle ({}). Éditer le fichier existant, ou les cartes dans LDtk.",
            info.kind,
            match info.kind.as_str() {
                "Map" => "cartes LDtk",
                "Ui" | "Camera" => "fichiers de présentation, un par jeu",
                _ => "un seul fichier par jeu",
            }
        ));
    }
    if info.ids.iter().any(|existing| existing == id) {
        return Err(format!("{} « {id} » existe déjà", info.kind));
    }
    let model_id = match from {
        Some(m) if info.ids.iter().any(|x| x == m) => m.to_string(),
        Some(m) => {
            return Err(format!(
                "modèle « {m} » inconnu pour {} (existants : {})",
                info.kind,
                info.ids.join(", ")
            ))
        }
        None => info
            .ids
            .first()
            .cloned()
            .ok_or_else(|| format!("{} : aucun modèle existant à copier", info.kind))?,
    };

    let assets = GameManifest::assets_dir(game_dir);
    let dir_decl = info.declared.iter().find(|d| !d.ends_with(".ron"));
    match dir_decl {
        Some(dir) => per_file(&assets, info, dir, &model_id, id),
        None => in_table(&assets, info, &model_id, id),
    }
}

fn per_file(
    assets: &Path,
    info: &KindInfo,
    dir: &str,
    model_id: &str,
    id: &str,
) -> Result<Created, String> {
    // Le fichier modèle : celui du dossier dont le nom est l'id (ids par fichier), sinon le premier.
    let model_file = info
        .files
        .iter()
        .find(|f| f.starts_with(dir) && Path::new(f).file_stem().is_some_and(|s| s == model_id))
        .or_else(|| info.files.iter().find(|f| f.starts_with(dir)))
        .ok_or_else(|| format!("{} : aucun fichier modèle sous {dir}", info.kind))?;
    let text = std::fs::read_to_string(assets.join(model_file)).map_err(|e| e.to_string())?;
    let rel = PathBuf::from(dir).join(format!("{id}.ron"));
    let target = assets.join(&rel);
    if target.exists() {
        return Err(format!("{} existe déjà", target.display()));
    }

    let body = strip_leading_comments(&text);
    let body = rename_fields(&body, model_id, id);
    let out = format!(
        "// {} « {id} » : créé par `alacod new-content` à partir de « {model_id} » ; à adapter\n// (`alacod describe {}` liste les champs possibles).\n{body}",
        info.kind, info.kind
    );
    std::fs::write(&target, out).map_err(|e| format!("{} : {e}", target.display()))?;
    Ok(Created {
        file: Path::new("assets").join(rel),
        in_table: false,
        model: model_id.to_string(),
    })
}

fn in_table(
    assets: &Path,
    info: &KindInfo,
    model_id: &str,
    id: &str,
) -> Result<Created, String> {
    let file = info
        .files
        .iter()
        .find(|f| {
            std::fs::read_to_string(assets.join(f))
                .ok()
                .and_then(|t| find_table(&t))
                .is_some_and(|table| table.entries.iter().any(|e| e.key == model_id))
        })
        .ok_or_else(|| {
            format!(
                "{} : « {model_id} » n'est dans aucune table ({} fichier(s)) ; --from <id> d'une table",
                info.kind,
                info.files.len()
            )
        })?;
    let path = assets.join(file);
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let table = find_table(&text).ok_or("table introuvable")?;
    let model = table
        .entries
        .iter()
        .find(|e| e.key == model_id)
        .ok_or("entrée modèle introuvable")?;
    let entry_text = &text[model.value_start..model.end];
    let renamed = rename_fields(entry_text, model_id, id);
    let indent = line_indent(&text, model.start);

    // Insère après la dernière entrée, virgule finale comprise (ajoutée si elle manque).
    let last = table.entries.last().ok_or("table vide")?;
    let block = format!(
        "\n{indent}// « {id} » : créé par `alacod new-content` à partir de « {model_id} » ; à adapter.\n{indent}\"{id}\": {renamed},"
    );
    let rest = &text[last.end..];
    let out = if rest.trim_start().starts_with(',') {
        let comma = last.end + rest.find(',').unwrap_or(0) + 1;
        format!("{}{block}{}", &text[..comma], &text[comma..])
    } else {
        format!("{},{block}{}", &text[..last.end], &text[last.end..])
    };
    std::fs::write(&path, out).map_err(|e| format!("{} : {e}", path.display()))?;
    Ok(Created {
        file: Path::new("assets").join(file),
        in_table: true,
        model: model_id.to_string(),
    })
}

fn line_indent(text: &str, pos: usize) -> String {
    let line_start = text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    text[line_start..pos]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

/// Retire les lignes de commentaire `//` en tête (elles décrivent le modèle, pas la copie).
fn strip_leading_comments(text: &str) -> String {
    let mut lines = text.lines().peekable();
    while lines.peek().is_some_and(|l| l.trim_start().starts_with("//") || l.trim().is_empty()) {
        lines.next();
    }
    let mut out: String = lines.collect::<Vec<_>>().join("\n");
    out.push('\n');
    out
}

/// Réécrit l'identité : `asset_name_ref: "<modèle>"` (personnages), puis le premier
/// `name: "…"` (objets, armes, salles…). Le reste n'est pas touché : `sprite_config.name` et
/// les références croisées restent celles du modèle, que le lint vérifie.
fn rename_fields(text: &str, model_id: &str, id: &str) -> String {
    let mut out = text.to_string();
    let asset_ref = format!("asset_name_ref: \"{model_id}\"");
    if out.contains(&asset_ref) {
        out = out.replacen(&asset_ref, &format!("asset_name_ref: \"{id}\""), 1);
        return out;
    }
    let masked = mask_comments(&out);
    // ne pas toucher `asset_name_ref`, `sprite_name`… : `name` doit commencer un champ.
    let pos = masked
        .match_indices("name: \"")
        .map(|(pos, _)| pos)
        .find(|pos| {
            masked[..*pos]
                .chars()
                .last()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
        });
    if let Some(pos) = pos {
        let value_start = pos + "name: \"".len();
        if let Some(len) = out[value_start..].find('"') {
            out.replace_range(value_start..value_start + len, id);
        }
    }
    out
}

// ---------------------------------------------------------------------------------------
// Table RON `{ "id": (...), ... }`, avec offsets
// ---------------------------------------------------------------------------------------

struct TableEntry {
    key: String,
    /// Début de la clé (`"id"`), fin de l'entrée (avant la virgule).
    start: usize,
    value_start: usize,
    end: usize,
}

struct Table {
    entries: Vec<TableEntry>,
}

/// Texte de même longueur sans les commentaires `//` (remplacés par des espaces) : les
/// offsets restent ceux du fichier.
fn mask_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    let mut in_str = false;
    let mut esc = false;
    while let Some((_, c)) = chars.next() {
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push(c);
            }
            '/' if chars.peek().is_some_and(|(_, n)| *n == '/') => {
                out.push(' ');
                for (_, n) in chars.by_ref() {
                    if n == '\n' {
                        out.push('\n');
                        break;
                    }
                    for _ in 0..n.len_utf8() {
                        out.push(' ');
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Cherche la table du fichier : racine `{…}`, newtype `({…})`, ou champ `{…}` unique d'une
/// racine struct (`(drop_chance: …, powerups: {…})`).
fn find_table(text: &str) -> Option<Table> {
    let masked = mask_comments(text);
    let bytes = masked.as_bytes();
    // premier délimiteur ouvrant significatif
    let open = masked.find(['(', '{'])?;
    let brace = if bytes[open] == b'{' {
        open
    } else {
        let inner = masked[open + 1..].trim_start();
        if inner.starts_with('{') {
            open + 1 + (masked[open + 1..].len() - inner.len())
        } else {
            // champ `{` unique de profondeur 1
            let mut depth = 0i32;
            let mut found = Vec::new();
            let mut in_str = false;
            for (i, c) in masked.char_indices().skip_while(|(i, _)| *i < open) {
                if in_str {
                    if c == '"' && masked.as_bytes()[i - 1] != b'\\' {
                        in_str = false;
                    }
                    continue;
                }
                match c {
                    '"' => in_str = true,
                    '(' | '[' => depth += 1,
                    '{' => {
                        if depth == 1 {
                            found.push(i);
                        }
                        depth += 1;
                    }
                    ')' | ']' | '}' => depth -= 1,
                    _ => {}
                }
            }
            if found.len() != 1 {
                return None;
            }
            found[0]
        }
    };

    // éléments de profondeur 1 dans `{…}`
    let mut entries = Vec::new();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    let mut elem_start = brace + 1;
    let mut end_brace = None;
    for (i, c) in masked.char_indices().skip_while(|(i, _)| *i < brace) {
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    push_entry(&masked, &mut entries, elem_start, i);
                    end_brace = Some(i);
                    break;
                }
            }
            ',' if depth == 1 => {
                push_entry(&masked, &mut entries, elem_start, i);
                elem_start = i + 1;
            }
            _ => {}
        }
    }
    end_brace?;
    Some(Table { entries })
}

fn push_entry(masked: &str, entries: &mut Vec<TableEntry>, from: usize, to: usize) {
    let slice = &masked[from..to];
    let trimmed = slice.trim();
    if trimmed.is_empty() {
        return;
    }
    let lead = slice.len() - slice.trim_start().len();
    let start = from + lead;
    let Some(colon) = trimmed.find(':') else {
        return;
    };
    let key = trimmed[..colon].trim().trim_matches('"').to_string();
    let after_colon = &trimmed[colon + 1..];
    let value_lead = after_colon.len() - after_colon.trim_start().len();
    entries.push(TableEntry {
        key,
        start,
        value_start: start + colon + 1 + value_lead,
        end: start + trimmed.len(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_entries_of_a_table_with_offsets() {
        let text = "// c\n({\n    // un\n    \"a\": (x: 1),\n    \"b\": (y: \"2,3\"),\n})";
        let table = find_table(text).unwrap();
        let keys: Vec<_> = table.entries.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["a", "b"]);
        let b = &table.entries[1];
        assert_eq!(&text[b.value_start..b.end], "(y: \"2,3\")");
    }

    #[test]
    fn finds_the_single_table_field_of_a_struct_root() {
        let text = "(\n    drop_chance: \"0.0\",\n    powerups: {\n        \"m\": (weight: 1),\n    },\n)";
        let table = find_table(text).unwrap();
        assert_eq!(table.entries.len(), 1);
        assert_eq!(table.entries[0].key, "m");
    }

    #[test]
    fn rename_sets_asset_ref_or_first_name() {
        assert_eq!(
            rename_fields("(asset_name_ref: \"old\", name: \"x\")", "old", "neuf"),
            "(asset_name_ref: \"neuf\", name: \"x\")"
        );
        assert_eq!(
            rename_fields("(sprite_name: \"s\", name: \"Old\")", "old", "neuf"),
            "(sprite_name: \"s\", name: \"neuf\")"
        );
    }
}
