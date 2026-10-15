//! `alacod describe` : inventaire du vocabulaire d'un jeu, pour un agent ou un humain qui
//! doit écrire du contenu sans lire `docs/conventions.md` en entier.
//!
//! Prototype : le « schéma » est **déduit du contenu réel** (champs de premier niveau des
//! fichiers RON de chaque kind, fréquence, exemple de valeur), pas des types Rust. Il ne dit
//! donc pas ce qui est *possible* mais ce qui est *utilisé* dans ce jeu. Un schéma exhaustif
//! (variantes d'enum, plages) viendra des types eux-mêmes (voir la note de fin de module).
//!
//! Le scan est lexical (commentaires et chaînes ignorés, profondeur des `(`/`{`/`[`) : il ne
//! dépend d'aucun type de `content`, donc il ne casse pas quand un vocabulaire change.

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::files::ContentFiles;
use crate::manifest::GameManifest;

/// Un champ de premier niveau rencontré dans les fichiers d'un kind.
#[derive(Debug, Clone, Serialize)]
pub struct FieldInfo {
    pub name: String,
    /// Nombre d'entrées (fichiers, ou clés d'une table) qui portent le champ.
    pub used_in: usize,
    /// Première valeur rencontrée, tronquée.
    pub example: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KindInfo {
    pub kind: String,
    /// Chemins déclarés dans `game.ron` pour ce kind.
    pub declared: Vec<String>,
    /// Fichiers effectivement lus (relatifs à `assets/`).
    pub files: Vec<String>,
    /// Ids : clés d'une table racine `{ "id": (...) }`, sinon nom de fichier sans extension.
    pub ids: Vec<String>,
    /// Nombre d'entrées (`ids.len()`, sauf kind sans ids lisibles comme `Map`).
    pub entries: usize,
    pub fields: Vec<FieldInfo>,
    /// Premier fichier du kind, à ouvrir comme modèle.
    pub example_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameDescription {
    pub game: String,
    pub kinds: Vec<KindInfo>,
}

pub fn describe(game_dir: &Path) -> Result<GameDescription, String> {
    let manifest = GameManifest::load(game_dir).map_err(|e| e.to_string())?;
    let files = ContentFiles::native(GameManifest::assets_dir(game_dir));

    let mut by_kind: BTreeMap<String, KindInfo> = BTreeMap::new();
    let mut field_acc: BTreeMap<String, BTreeMap<String, FieldInfo>> = BTreeMap::new();

    for decl in &manifest.content_folders {
        let ext = if decl.kind == "Map" { "ldtk" } else { "ron" };
        let info = by_kind.entry(decl.kind.clone()).or_insert_with(|| KindInfo {
            kind: decl.kind.clone(),
            declared: Vec::new(),
            files: Vec::new(),
            ids: Vec::new(),
            entries: 0,
            fields: Vec::new(),
            example_file: None,
        });
        info.declared.push(decl.path.clone());

        let Ok(paths) = files.discover(Path::new(&decl.path), ext) else {
            continue;
        };
        for path in paths {
            let rel = path.to_string_lossy().replace('\\', "/");
            info.files.push(rel.clone());
            if ext == "ldtk" {
                info.ids.push(stem(&path));
                continue;
            }
            let Ok(text) = files.read(&path) else {
                continue;
            };
            let scan = scan_ron(&text);
            let acc = field_acc.entry(decl.kind.clone()).or_default();
            if scan.table_root {
                for (id, fields) in scan.entries {
                    info.ids.push(id);
                    add_fields(acc, fields);
                }
            } else {
                info.ids.push(stem(&path));
                for (_, fields) in scan.entries {
                    add_fields(acc, fields);
                }
            }
        }
    }

    for (kind, info) in by_kind.iter_mut() {
        info.entries = info.ids.len();
        info.ids.sort();
        info.example_file = info.files.first().cloned();
        if let Some(acc) = field_acc.remove(kind) {
            let mut fields: Vec<FieldInfo> = acc.into_values().collect();
            fields.sort_by(|a, b| b.used_in.cmp(&a.used_in).then(a.name.cmp(&b.name)));
            info.fields = fields;
        }
    }

    Ok(GameDescription {
        game: manifest.name,
        kinds: by_kind.into_values().collect(),
    })
}

fn stem(path: &Path) -> String {
    PathBuf::from(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn add_fields(acc: &mut BTreeMap<String, FieldInfo>, fields: Vec<(String, String)>) {
    for (name, example) in fields {
        let slot = acc.entry(name.clone()).or_insert_with(|| FieldInfo {
            name,
            used_in: 0,
            example: truncate(&example, 60),
        });
        slot.used_in += 1;
    }
}

fn truncate(s: &str, max: usize) -> String {
    let flat: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        flat
    } else {
        let cut: String = flat.chars().take(max).collect();
        format!("{cut}…")
    }
}

// ---------------------------------------------------------------------------------------
// Scan lexical
// ---------------------------------------------------------------------------------------

struct Scan {
    /// La racine est une table `{ "id": (...) }`.
    table_root: bool,
    /// (id, champs de premier niveau avec leur valeur) ; une seule entrée anonyme pour un
    /// fichier à racine struct.
    entries: Vec<(String, Vec<(String, String)>)>,
}

/// Retire commentaires `//` et `/* */` en gardant les chaînes intactes.
fn strip_comments(text: &str) -> String {
    let b: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            '"' => {
                out.push('"');
                i += 1;
                while i < b.len() {
                    out.push(b[i]);
                    if b[i] == '\\' && i + 1 < b.len() {
                        out.push(b[i + 1]);
                        i += 2;
                        continue;
                    }
                    if b[i] == '"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            '/' if b.get(i + 1) == Some(&'/') => {
                while i < b.len() && b[i] != '\n' {
                    i += 1;
                }
            }
            '/' if b.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Découpe `body` (contenu d'une paire de délimiteurs) en éléments séparés par les virgules
/// de profondeur 0.
fn split_top(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut in_str = false;
    let mut esc = false;
    for c in body.chars() {
        if in_str {
            cur.push(c);
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
                cur.push(c);
            }
            '(' | '{' | '[' => {
                depth += 1;
                cur.push(c);
            }
            ')' | '}' | ']' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                if !cur.trim().is_empty() {
                    parts.push(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }
    parts
}

/// Contenu entre le premier délimiteur ouvrant et son fermant (en sautant un éventuel nom
/// de struct devant `(`). Retourne (ouvrant, contenu).
fn outer_body(text: &str) -> Option<(char, String)> {
    let start = text.find(['(', '{'])?;
    let open = text[start..].chars().next()?;
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for (off, c) in text[start..].char_indices() {
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
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open, text[start + 1..start + off].to_string()));
                }
            }
            _ => {}
        }
    }
    None
}

/// `name: value` au niveau 0 d'un élément ; `None` si l'élément n'a pas la forme.
fn split_field(item: &str) -> Option<(String, String)> {
    let item = item.trim();
    let colon = item.find(':')?;
    let key = item[..colon].trim();
    if key.is_empty() || key.contains(['(', ')', '{', '}', '[', ']', ' ']) {
        return None;
    }
    Some((
        key.trim_matches('"').to_string(),
        item[colon + 1..].trim().to_string(),
    ))
}

fn struct_fields(body: &str) -> Vec<(String, String)> {
    split_top(body)
        .iter()
        .filter_map(|item| split_field(item))
        .collect()
}

fn scan_ron(text: &str) -> Scan {
    let clean = strip_comments(text);
    let Some((open, body)) = outer_body(&clean) else {
        return Scan {
            table_root: false,
            entries: Vec::new(),
        };
    };
    // Newtype `({ "id": (...) })` : la table est dans une paire de parenthèses.
    let (open, body) = match body.trim() {
        inner if open == '(' && inner.starts_with('{') => {
            outer_body(inner).unwrap_or((open, body.clone()))
        }
        _ => (open, body),
    };
    if open == '{' {
        let entries = split_top(&body)
            .iter()
            .filter_map(|item| split_field(item))
            .map(|(id, value)| {
                let fields = outer_body(&value)
                    .map(|(_, inner)| struct_fields(&inner))
                    .unwrap_or_default();
                (id, fields)
            })
            .collect();
        Scan {
            table_root: true,
            entries,
        }
    } else {
        Scan {
            table_root: false,
            entries: vec![(String::new(), struct_fields(&body))],
        }
    }
}

// ---------------------------------------------------------------------------------------
// Rendu texte
// ---------------------------------------------------------------------------------------

pub fn render_overview(desc: &GameDescription) -> String {
    let mut out = format!("{} : {} kinds\n\n", desc.game, desc.kinds.len());
    for k in &desc.kinds {
        out.push_str(&format!(
            "{:<13} {:>3} entrée(s)  {}\n",
            k.kind,
            k.entries,
            k.declared.join(", ")
        ));
    }
    out.push_str("\nDétail d'un kind : alacod describe <jeu> <Kind>\n");
    out
}

pub fn render_kind(info: &KindInfo) -> String {
    let mut out = format!("{} ({})\n", info.kind, info.declared.join(", "));
    out.push_str(&format!("  {} entrée(s) : {}\n", info.entries, info.ids.join(", ")));
    if let Some(example) = &info.example_file {
        out.push_str(&format!("  modèle à ouvrir : {example}\n"));
    }
    if info.fields.is_empty() {
        out.push_str("  (aucun champ lisible)\n");
    } else {
        out.push_str("  champs de premier niveau (utilisé dans / exemple) :\n");
        let total = info.entries.max(1);
        for f in &info.fields {
            out.push_str(&format!(
                "    {:<22} {:>3}/{:<3} {}\n",
                f.name, f.used_in, total, f.example
            ));
        }
    }
    out
}

pub fn find_kind<'a>(desc: &'a GameDescription, kind: &str) -> Option<&'a KindInfo> {
    desc.kinds
        .iter()
        .find(|k| k.kind.eq_ignore_ascii_case(kind))
}

// Note — vers un schéma exhaustif : dériver `schemars::JsonSchema` sur les types de contenu
// (`schemars` 1.x est déjà dans le graphe de dépendances) donnerait variantes d'enum et types
// de champs ; les champs `Fixed` (chaînes RON) demandent un `#[schemars(with = "String")]`.
// À faire kind par kind, en commençant par `Item` et `Room`.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_root_fields_ignore_comments_and_nesting() {
        let scan = scan_ron(
            "// c\n(\n name: \"A, B\", // x\n kind: Passive,\n effects: [(on: OnPickup, do: [Heal(\"1\")])],\n)",
        );
        assert!(!scan.table_root);
        let names: Vec<_> = scan.entries[0].1.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["name", "kind", "effects"]);
        assert_eq!(scan.entries[0].1[0].1, "\"A, B\"");
    }

    #[test]
    fn table_root_gives_ids_and_entry_fields() {
        let scan = scan_ron("{ \"a\": (damage: \"1\", cfg: (x: 1)), \"b\": (damage: \"2\") }");
        assert!(scan.table_root);
        let ids: Vec<_> = scan.entries.iter().map(|(i, _)| i.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
        let names: Vec<_> = scan.entries[0].1.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["damage", "cfg"]);
    }
}
