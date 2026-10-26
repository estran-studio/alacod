//! `alacod get` / `alacod set` / `alacod undo` : lire et régler **une** valeur d'un fichier RON de contenu
//! (`docs/editeur-vocal.md` §2 et §4, `docs/conventions.md` §89).
//!
//! L'édition est **textuelle** : le fichier est lu par un parseur RON à positions (`Node`, un parcours lexical
//! qui garde l'étendue de chaque valeur), la valeur visée est remplacée octet pour octet et le reste du fichier
//! (commentaires, retours à la ligne, indentation) ne bouge pas. Rien n'est resérialisé. La sémantique (la valeur
//! est-elle acceptée par le jeu ?) n'est pas jugée ici : c'est le lint (`load_and_lint`, comme `alacod lint`) qui
//! tranche après l'écriture, et le fichier est restauré s'il apparaît une erreur qui n'y était pas avant.
//!
//! Chemins : `champ.sous_champ`, à travers les structs, les tables (`firing_modes.default`, clé sans guillemets),
//! les listes (`modifiers.0.value`, ou le nom d'une variante si elle est unique : `behaviors.Shoot.cooldown_frames`),
//! les tuples (`rooms.0`) ; `Some(..)` et les variantes à un seul champ (`Standard(damage: ..)`, `Heal("20")`) se
//! traversent sans être nommés. Un chemin inconnu ou ambigu est refusé avec la liste des chemins possibles.

use std::collections::BTreeSet;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::files::ContentFiles;
use crate::manifest::GameManifest;

type Span = Range<usize>;

// ---------------------------------------------------------------------------------------
// Arbre RON à positions
// ---------------------------------------------------------------------------------------

/// Une valeur RON et sa position (octets) dans le texte du fichier.
#[derive(Debug, Clone)]
pub enum Node {
    /// Nombre, chaîne, booléen, caractère, variante sans champ (`None`, `Common`).
    Scalar(Span),
    /// `(a, b)` ou `Nom(a, b)` ; `Some(x)` en est un (nom `Some`, un élément).
    Tuple {
        name: Option<String>,
        items: Vec<Node>,
        span: Span,
    },
    /// `(champ: v, …)` ou `Nom(champ: v, …)`.
    Struct {
        name: Option<String>,
        fields: Vec<(String, Node)>,
        span: Span,
    },
    List {
        items: Vec<Node>,
        span: Span,
    },
    Map {
        entries: Vec<(Node, Node)>,
        span: Span,
    },
}

impl Node {
    pub fn span(&self) -> &Span {
        match self {
            Node::Scalar(span)
            | Node::Tuple { span, .. }
            | Node::Struct { span, .. }
            | Node::List { span, .. }
            | Node::Map { span, .. } => span,
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            Node::Tuple { name, .. } | Node::Struct { name, .. } => name.as_deref(),
            _ => None,
        }
    }

    fn text<'s>(&self, src: &'s str) -> &'s str {
        &src[self.span().clone()]
    }

    fn describe(&self) -> &'static str {
        match self {
            Node::Scalar(_) => "une valeur",
            Node::Tuple { .. } => "un tuple",
            Node::Struct { .. } => "un struct",
            Node::List { .. } => "une liste",
            Node::Map { .. } => "une table",
        }
    }
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

struct Parser<'a> {
    src: &'a str,
    b: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn error(&self, message: &str) -> String {
        let before = self.src.get(..self.pos.min(self.src.len())).unwrap_or("");
        let line = before.matches('\n').count() + 1;
        let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
        format!("RON illisible, ligne {line}, colonne {column} : {message}")
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.pos).copied()
    }

    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn starts_with(&self, s: &[u8]) -> bool {
        self.b[self.pos.min(self.b.len())..].starts_with(s)
    }

    /// Espaces et commentaires (`//`, `/* */` imbriqués).
    fn skip_trivia(&mut self) -> Result<(), String> {
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_whitespace() => self.pos += 1,
                Some(b'/') if self.b.get(self.pos + 1) == Some(&b'/') => {
                    while self.peek().is_some_and(|c| c != b'\n') {
                        self.pos += 1;
                    }
                }
                Some(b'/') if self.b.get(self.pos + 1) == Some(&b'*') => {
                    let mut depth = 0usize;
                    loop {
                        if self.starts_with(b"/*") {
                            depth += 1;
                            self.pos += 2;
                        } else if self.starts_with(b"*/") {
                            depth -= 1;
                            self.pos += 2;
                            if depth == 0 {
                                break;
                            }
                        } else if self.pos >= self.b.len() {
                            return Err(self.error("commentaire /* non fermé"));
                        } else {
                            self.pos += 1;
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn ident(&mut self) -> &'a str {
        let start = self.pos;
        while self.peek().is_some_and(is_ident) {
            self.pos += 1;
        }
        &self.src[start..self.pos]
    }

    /// Chaîne ou caractère entre `q` (le curseur est sur le guillemet ouvrant).
    fn quoted(&mut self, q: u8) -> Result<(), String> {
        self.pos += 1;
        loop {
            match self.peek() {
                None => return Err(self.error("chaîne non fermée")),
                Some(b'\\') => self.pos += 2,
                Some(c) if c == q => {
                    self.pos += 1;
                    return Ok(());
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    /// Chaîne brute `r#"…"#` (le curseur est après le `r`, sur les `#` ou le guillemet) ; `false` si ce n'en est
    /// pas une (`r#ident`).
    fn raw_string(&mut self) -> Result<bool, String> {
        let mut hashes = 0usize;
        while self.b.get(self.pos + hashes) == Some(&b'#') {
            hashes += 1;
        }
        if self.b.get(self.pos + hashes) != Some(&b'"') {
            return Ok(false);
        }
        self.pos += hashes + 1;
        loop {
            match self.peek() {
                None => return Err(self.error("chaîne brute non fermée")),
                Some(b'"')
                    if (0..hashes).all(|i| self.b.get(self.pos + 1 + i) == Some(&b'#')) =>
                {
                    self.pos += 1 + hashes;
                    return Ok(true);
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    fn number(&mut self) {
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.pos += 1;
        }
        while let Some(c) = self.peek() {
            let exponent_sign = matches!(c, b'+' | b'-')
                && self.pos > 0
                && matches!(self.b[self.pos - 1], b'e' | b'E');
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || exponent_sign {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn value(&mut self) -> Result<Node, String> {
        self.skip_trivia()?;
        let start = self.pos;
        let Some(c) = self.peek() else {
            return Err(self.error("fin de fichier inattendue"));
        };
        match c {
            b'"' => {
                self.quoted(b'"')?;
                Ok(Node::Scalar(start..self.pos))
            }
            b'\'' => {
                self.quoted(b'\'')?;
                Ok(Node::Scalar(start..self.pos))
            }
            b'(' => self.paren(None, start),
            b'[' => self.list(start),
            b'{' => self.map(start),
            b'-' | b'+' | b'0'..=b'9' => {
                self.number();
                Ok(Node::Scalar(start..self.pos))
            }
            c if is_ident_start(c) => {
                let word = self.ident();
                match (word, self.peek()) {
                    ("b", Some(b'"')) => {
                        self.quoted(b'"')?;
                        return Ok(Node::Scalar(start..self.pos));
                    }
                    ("b", Some(b'\'')) => {
                        self.quoted(b'\'')?;
                        return Ok(Node::Scalar(start..self.pos));
                    }
                    ("r" | "br", Some(b'"' | b'#')) => {
                        if self.raw_string()? {
                            return Ok(Node::Scalar(start..self.pos));
                        }
                        // `r#ident` : identifiant brut.
                        self.pos += 1;
                        self.ident();
                        return Ok(Node::Scalar(start..self.pos));
                    }
                    _ => {}
                }
                let word = word.to_string();
                let after_ident = self.pos;
                self.skip_trivia()?;
                if self.peek() == Some(b'(') {
                    self.paren(Some(word), start)
                } else {
                    self.pos = after_ident;
                    Ok(Node::Scalar(start..after_ident))
                }
            }
            other => Err(self.error(&format!("caractère inattendu « {} »", other as char))),
        }
    }

    /// Le curseur est sur `(` ; struct si le premier élément est `ident:`, tuple sinon.
    fn paren(&mut self, name: Option<String>, start: usize) -> Result<Node, String> {
        self.pos += 1;
        let save = self.pos;
        self.skip_trivia()?;
        let struct_like = self.peek().is_some_and(is_ident_start) && {
            self.ident();
            self.skip_trivia()?;
            self.peek() == Some(b':')
        };
        self.pos = save;
        if struct_like {
            let mut fields = Vec::new();
            loop {
                self.skip_trivia()?;
                if self.eat(b')') {
                    break;
                }
                if !self.peek().is_some_and(is_ident_start) {
                    return Err(self.error("nom de champ attendu"));
                }
                let key = self.ident().to_string();
                self.skip_trivia()?;
                if !self.eat(b':') {
                    return Err(self.error("`:` attendu après le nom de champ"));
                }
                fields.push((key, self.value()?));
                self.separator(b')')?;
            }
            Ok(Node::Struct {
                name,
                fields,
                span: start..self.pos,
            })
        } else {
            let mut items = Vec::new();
            loop {
                self.skip_trivia()?;
                if self.eat(b')') {
                    break;
                }
                items.push(self.value()?);
                self.separator(b')')?;
            }
            Ok(Node::Tuple {
                name,
                items,
                span: start..self.pos,
            })
        }
    }

    /// Après un élément : `,` (puis la boucle voit la fermeture ou l'élément suivant) ou la fermeture.
    fn separator(&mut self, close: u8) -> Result<(), String> {
        self.skip_trivia()?;
        if self.eat(b',') || self.peek() == Some(close) {
            Ok(())
        } else {
            Err(self.error(&format!("`,` ou `{}` attendu", close as char)))
        }
    }

    fn list(&mut self, start: usize) -> Result<Node, String> {
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.eat(b']') {
                break;
            }
            items.push(self.value()?);
            self.separator(b']')?;
        }
        Ok(Node::List {
            items,
            span: start..self.pos,
        })
    }

    fn map(&mut self, start: usize) -> Result<Node, String> {
        self.pos += 1;
        let mut entries = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.eat(b'}') {
                break;
            }
            let key = self.value()?;
            self.skip_trivia()?;
            if !self.eat(b':') {
                return Err(self.error("`:` attendu après la clé"));
            }
            entries.push((key, self.value()?));
            self.separator(b'}')?;
        }
        Ok(Node::Map {
            entries,
            span: start..self.pos,
        })
    }
}

/// Parse un fichier RON complet (attributs `#![enable(..)]` et commentaires compris).
pub fn parse(src: &str) -> Result<Node, String> {
    let mut p = Parser {
        src,
        b: src.as_bytes(),
        pos: 0,
    };
    p.skip_trivia()?;
    while p.starts_with(b"#!") {
        let mut depth = 0usize;
        loop {
            match p.peek() {
                None => return Err(p.error("attribut `#![…]` non fermé")),
                Some(b'[') => depth += 1,
                Some(b']') => {
                    depth -= 1;
                    if depth == 0 {
                        p.pos += 1;
                        break;
                    }
                }
                Some(_) => {}
            }
            p.pos += 1;
        }
        p.skip_trivia()?;
    }
    let node = p.value()?;
    p.skip_trivia()?;
    if p.pos < p.b.len() {
        return Err(p.error("contenu inattendu après la valeur racine"));
    }
    Ok(node)
}

// ---------------------------------------------------------------------------------------
// Chemins
// ---------------------------------------------------------------------------------------

/// Texte d'une clé de table : une chaîne sans ses guillemets, sinon le texte tel quel.
fn key_text<'s>(node: &Node, src: &'s str) -> &'s str {
    let text = node.text(src);
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

/// `Some(x)` se traverse sans être nommé.
fn unwrap_some(node: &Node) -> &Node {
    let mut n = node;
    while let Node::Tuple {
        name: Some(name),
        items,
        ..
    } = n
    {
        if name == "Some" && items.len() == 1 {
            n = &items[0];
        } else {
            break;
        }
    }
    n
}

enum Miss {
    NotFound(Vec<String>),
    Ambiguous(Vec<String>),
    Scalar,
}

fn step<'n>(node: &'n Node, seg: &str, src: &str) -> Result<&'n Node, Miss> {
    let n = unwrap_some(node);
    match n {
        Node::Struct { name, fields, .. } => {
            if let Some((_, v)) = fields.iter().find(|(k, _)| k == seg) {
                return Ok(v);
            }
            if name.as_deref() == Some(seg) {
                return Ok(n);
            }
            Err(Miss::NotFound(fields.iter().map(|(k, _)| k.clone()).collect()))
        }
        Node::Map { entries, .. } => entries
            .iter()
            .find(|(k, _)| key_text(k, src) == seg)
            .map(|(_, v)| v)
            .ok_or_else(|| {
                Miss::NotFound(entries.iter().map(|(k, _)| key_text(k, src).to_string()).collect())
            }),
        Node::List { items, .. } => {
            if let Ok(i) = seg.parse::<usize>() {
                return items
                    .get(i)
                    .ok_or_else(|| Miss::NotFound((0..items.len()).map(|i| i.to_string()).collect()));
            }
            let hits: Vec<usize> = items
                .iter()
                .enumerate()
                .filter(|(_, it)| unwrap_some(it).name() == Some(seg))
                .map(|(i, _)| i)
                .collect();
            match hits.as_slice() {
                [one] => Ok(&items[*one]),
                [] => Err(Miss::NotFound(
                    items
                        .iter()
                        .enumerate()
                        .map(|(i, it)| match unwrap_some(it).name() {
                            Some(name) => format!("{name} (ou {i})"),
                            None => i.to_string(),
                        })
                        .collect(),
                )),
                many => Err(Miss::Ambiguous(many.iter().map(|i| i.to_string()).collect())),
            }
        }
        Node::Tuple { name, items, .. } => {
            if let Ok(i) = seg.parse::<usize>() {
                return items
                    .get(i)
                    .ok_or_else(|| Miss::NotFound((0..items.len()).map(|i| i.to_string()).collect()));
            }
            if name.as_deref() == Some(seg) {
                return Ok(if items.len() == 1 { &items[0] } else { n });
            }
            if items.len() == 1 {
                return step(&items[0], seg, src);
            }
            Err(Miss::NotFound((0..items.len()).map(|i| i.to_string()).collect()))
        }
        Node::Scalar(_) => Err(Miss::Scalar),
    }
}

fn join(prefix: &str, seg: &str) -> String {
    if prefix.is_empty() {
        seg.to_string()
    } else {
        format!("{prefix}.{seg}")
    }
}

/// Au plus douze éléments, puis « … (+N) ».
fn list_some(items: &[String]) -> String {
    const MAX: usize = 12;
    let shown = items.iter().take(MAX).cloned().collect::<Vec<_>>().join(", ");
    if items.len() > MAX {
        format!("{shown}, … (+{})", items.len() - MAX)
    } else {
        shown
    }
}

/// Descend `path` depuis `root`. Le nœud rendu est celui que désigne le chemin, sans le réduire à une valeur
/// simple (voir `as_scalar`).
pub fn resolve<'n>(root: &'n Node, path: &str, src: &str) -> Result<&'n Node, String> {
    if path.is_empty() {
        return Err("chemin vide".to_string());
    }
    let mut cur = root;
    let mut trail = String::new();
    for seg in path.split('.') {
        if seg.is_empty() {
            return Err(format!("chemin « {path} » : segment vide"));
        }
        match step(cur, seg, src) {
            Ok(next) => cur = next,
            Err(Miss::NotFound(candidates)) => {
                let full: Vec<String> = candidates.iter().map(|c| join(&trail, c)).collect();
                let at = if trail.is_empty() { "la racine" } else { &trail };
                return Err(format!(
                    "chemin « {path} » : « {seg} » introuvable sous {at} ; chemins possibles : {}",
                    list_some(&full)
                ));
            }
            Err(Miss::Ambiguous(indices)) => {
                let full: Vec<String> = indices.iter().map(|i| join(&trail, i)).collect();
                return Err(format!(
                    "chemin « {path} » : « {seg} » est ambigu ({} éléments de ce nom) ; préciser par l'indice : {}",
                    indices.len(),
                    list_some(&full)
                ));
            }
            Err(Miss::Scalar) => {
                return Err(format!(
                    "chemin « {path} » : « {trail} » est une valeur simple, elle n'a pas de champ « {seg} »"
                ));
            }
        }
        trail = join(&trail, seg);
    }
    Ok(cur)
}

/// La valeur simple qu'un nœud désigne : un scalaire, ou le scalaire seul d'un tuple à un élément
/// (`Some("500.0")`, `Heal("20")`).
pub fn as_scalar(node: &Node) -> Option<&Node> {
    let mut n = node;
    loop {
        match n {
            Node::Scalar(_) => return Some(n),
            Node::Tuple { items, .. } if items.len() == 1 => n = &items[0],
            _ => return None,
        }
    }
}

/// Les chemins des valeurs simples sous `node`.
fn leaves(node: &Node, src: &str, prefix: &str, out: &mut Vec<String>) {
    match node {
        Node::Scalar(_) => out.push(prefix.to_string()),
        Node::Struct { fields, .. } => {
            for (k, v) in fields {
                leaves(v, src, &join(prefix, k), out);
            }
        }
        Node::Map { entries, .. } => {
            for (k, v) in entries {
                leaves(v, src, &join(prefix, key_text(k, src)), out);
            }
        }
        Node::List { items, .. } => {
            for (i, v) in items.iter().enumerate() {
                leaves(v, src, &join(prefix, &i.to_string()), out);
            }
        }
        Node::Tuple { items, .. } if items.len() == 1 => leaves(&items[0], src, prefix, out),
        Node::Tuple { items, .. } => {
            for (i, v) in items.iter().enumerate() {
                leaves(v, src, &join(prefix, &i.to_string()), out);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Valeurs
// ---------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Lit<'s> {
    Str(&'s str),
    Int,
    Float,
    Bool,
    Ident,
    Other,
}

fn classify(raw: &str) -> Lit<'_> {
    let first = raw.as_bytes().first().copied().unwrap_or(b' ');
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        Lit::Str(&raw[1..raw.len() - 1])
    } else if raw == "true" || raw == "false" {
        Lit::Bool
    } else if first.is_ascii_digit() || first == b'-' || first == b'+' {
        if raw.parse::<i64>().is_ok() {
            Lit::Int
        } else {
            Lit::Float
        }
    } else if is_ident_start(first) && raw.bytes().all(is_ident) {
        Lit::Ident
    } else {
        Lit::Other
    }
}

fn looks_numeric(s: &str) -> bool {
    !s.is_empty()
        && s.parse::<f64>().is_ok()
        && s.chars().all(|c| c.is_ascii_digit() || "+-.eE".contains(c))
}

/// Ajoute `.0` à un entier quand l'ancienne valeur s'écrivait avec une virgule (`"150.0"` → `"120.0"`).
fn like_old_style(old: &str, new: &str) -> String {
    let integer = new.trim_start_matches(['+', '-']).bytes().all(|c| c.is_ascii_digit());
    if old.contains('.') && integer {
        format!("{new}.0")
    } else {
        new.to_string()
    }
}

/// Le texte RON de la nouvelle valeur, dans le type de l'ancienne : un `Fixed` reste une chaîne
/// (`"120.0"`), un entier reste un entier, un booléen un booléen, une variante un identifiant.
pub fn format_value(old_raw: &str, new: &str) -> Result<String, String> {
    let new = new.trim();
    // `+5` est la valeur absolue 5 (les valeurs relatives sont `x0.9`, `+10%`, `+=5`) : écrite sans son signe.
    let new = match new.strip_prefix('+') {
        Some(rest) if looks_numeric(rest) && !rest.starts_with(['+', '-']) => rest,
        _ => new,
    };
    match classify(old_raw) {
        Lit::Str(inner) => {
            if new.len() >= 2 && new.starts_with('"') && new.ends_with('"') {
                Ok(new.to_string())
            } else if looks_numeric(inner) && looks_numeric(new) {
                Ok(format!("\"{}\"", like_old_style(inner, new)))
            } else {
                Ok(format!("\"{}\"", new.replace('\\', "\\\\").replace('"', "\\\"")))
            }
        }
        Lit::Int => {
            if new.parse::<i64>().is_ok() {
                Ok(new.to_string())
            } else {
                Err(format!(
                    "« {new} » : entier attendu (la valeur actuelle est l'entier {old_raw})"
                ))
            }
        }
        Lit::Float => {
            if looks_numeric(new) {
                Ok(like_old_style(old_raw, new))
            } else {
                Err(format!(
                    "« {new} » : nombre attendu (la valeur actuelle est le nombre {old_raw})"
                ))
            }
        }
        Lit::Bool => match new {
            "true" | "false" => Ok(new.to_string()),
            _ => Err(format!("« {new} » : `true` ou `false` attendu")),
        },
        Lit::Ident if old_raw == "None" => Err(
            "la valeur actuelle est `None` : donner la valeur complète avec --raw, ex. --raw 'Some(\"500.0\")'"
                .to_string(),
        ),
        Lit::Ident => {
            let first = new.as_bytes().first().copied().unwrap_or(b' ');
            if is_ident_start(first) && new.bytes().all(is_ident) {
                Ok(new.to_string())
            } else {
                Err(format!(
                    "« {new} » : identifiant de variante attendu (la valeur actuelle est {old_raw}) ; \
                     pour autre chose : --raw"
                ))
            }
        }
        Lit::Other => Err(format!(
            "la valeur actuelle ({old_raw}) n'est ni un nombre, ni une chaîne, ni une variante : \
             l'écrire en entier avec --raw"
        )),
    }
}

/// La valeur pour l'affichage : une chaîne sans ses guillemets.
pub fn display_value(raw: &str) -> String {
    match classify(raw) {
        Lit::Str(inner) => inner.to_string(),
        _ => raw.split_whitespace().collect::<Vec<_>>().join(" "),
    }
}

/// Une valeur relative d'`alacod set` : `x0.9` / `*0.9` (facteur), `+10%` / `-10%` (pourcentage, signe
/// obligatoire), `+=5` / `-=5` (ajout). `+5` et `-5` restent des valeurs **absolues** (5, −5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Relative {
    Mul(f64),
    Percent(f64),
    Add(f64),
}

fn finite(s: &str) -> Option<f64> {
    looks_numeric(s)
        .then(|| s.parse::<f64>().ok())
        .flatten()
        .filter(|v| v.is_finite())
}

impl Relative {
    /// Lit une valeur relative ; `None` pour une valeur absolue (`0.9`, `-5`, `"texte"`, `Common`…).
    pub fn parse(value: &str) -> Option<Relative> {
        let value = value.trim();
        if let Some(rest) = value.strip_prefix(['x', '*']) {
            return finite(rest).map(Relative::Mul);
        }
        if let Some(rest) = value.strip_prefix("+=") {
            return finite(rest).map(Relative::Add);
        }
        if let Some(rest) = value.strip_prefix("-=") {
            return finite(rest).map(|v| Relative::Add(-v));
        }
        let pct = value.strip_suffix('%')?;
        if !pct.starts_with(['+', '-']) {
            return None;
        }
        finite(pct).map(Relative::Percent)
    }

    fn apply(self, old: f64) -> f64 {
        match self {
            Relative::Mul(f) => old * f,
            Relative::Percent(p) => old * (1.0 + p / 100.0),
            Relative::Add(a) => old + a,
        }
    }
}

/// Nombre de chiffres après la virgule d'un nombre écrit (`0.15` → 2, `100` → 0).
fn decimals(s: &str) -> usize {
    let mantissa = s.split(['e', 'E']).next().unwrap_or(s);
    mantissa.split_once('.').map_or(0, |(_, d)| d.len())
}

/// Écrit `v` avec au plus `max(décimales de l'ancienne, 1) + 2` décimales, zéros de fin retirés (`110.000` → `110`,
/// mis ensuite à `110.0` par [`format_value`] si l'ancienne valeur avait une virgule) : `0.15 x0.9` → `0.135`.
fn decimal_text(v: f64, old: &str) -> String {
    let places = decimals(old).max(1) + 2;
    let text = format!("{v:.places$}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_string()
    } else {
        text.to_string()
    }
}

/// Le texte (absolu, avant [`format_value`]) d'une valeur relative appliquée à `old_raw` : un nombre ou une chaîne
/// `Fixed` seulement ; un entier reste un entier (résultat fractionnaire refusé, avec les entiers voisins proposés).
pub fn apply_relative(old_raw: &str, value: &str, relative: Relative) -> Result<String, String> {
    match classify(old_raw) {
        Lit::Int => {
            let old: f64 = old_raw
                .parse()
                .map_err(|_| format!("entier illisible : {old_raw}"))?;
            let new = relative.apply(old);
            let rounded = new.round();
            if (new - rounded).abs() > 1e-9 {
                return Err(format!(
                    "« {value} » sur l'entier {old_raw} donne {} : pas un entier ; donner {} ou {}",
                    decimal_text(new, "0.0"),
                    new.floor() as i64,
                    new.ceil() as i64
                ));
            }
            Ok(format!("{}", rounded as i64))
        }
        Lit::Float => {
            let old: f64 = old_raw
                .parse()
                .map_err(|_| format!("nombre illisible : {old_raw}"))?;
            Ok(decimal_text(relative.apply(old), old_raw))
        }
        Lit::Str(inner) if looks_numeric(inner) => {
            let old: f64 = inner
                .parse()
                .map_err(|_| format!("nombre illisible : {inner}"))?;
            Ok(decimal_text(relative.apply(old), inner))
        }
        _ => Err(format!(
            "« {value} » est une valeur relative : seulement sur un nombre ou un `Fixed` (la valeur actuelle est \
             {old_raw}) ; pour écrire ce texte tel quel : --raw"
        )),
    }
}

// ---------------------------------------------------------------------------------------
// Localiser l'entrée d'un kind
// ---------------------------------------------------------------------------------------

/// Une entrée de contenu : son fichier, le texte lu et le nœud de l'entrée.
pub struct Entry {
    /// Fichier relatif à `assets/`.
    pub rel: String,
    pub abs: PathBuf,
    pub text: String,
    pub node: Node,
}

/// Racine d'une table `{ "id": (…) }`, nue ou dans une paire de parenthèses `({ … })`.
fn table_root(root: &Node) -> Option<&Node> {
    match root {
        Node::Map { .. } => Some(root),
        Node::Tuple {
            name: None, items, ..
        } if items.len() == 1 && matches!(items[0], Node::Map { .. }) => Some(&items[0]),
        _ => None,
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut diag = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let up = row[j + 1];
            row[j + 1] = (diag + usize::from(ca != cb)).min(up + 1).min(row[j] + 1);
            diag = up;
        }
    }
    row[b.len()]
}

/// Les ids les plus proches de `wanted` d'abord (la dictée déforme les identifiants).
fn closest(wanted: &str, ids: &BTreeSet<String>) -> Vec<String> {
    let wanted = wanted.to_lowercase();
    let mut scored: Vec<(usize, &String)> = ids
        .iter()
        .map(|id| {
            let lower = id.to_lowercase();
            let near = lower.contains(&wanted) || wanted.contains(&lower);
            (if near { 0 } else { edit_distance(&wanted, &lower) }, id)
        })
        .collect();
    scored.sort();
    scored.into_iter().map(|(_, id)| id.clone()).collect()
}

/// Cherche `id` parmi les fichiers du kind déclarés dans `game.ron` : clé d'une table racine, sinon nom de fichier
/// sans extension (comme `alacod describe`) ou, pour un personnage, son `asset_name_ref` (l'id du registre).
pub fn locate(game_dir: &Path, kind: &str, id: &str) -> Result<Entry, String> {
    let manifest = GameManifest::load(game_dir).map_err(|e| e.to_string())?;
    let declared: BTreeSet<&str> = manifest
        .content_folders
        .iter()
        .map(|d| d.kind.as_str())
        .collect();
    if !declared.contains(kind) {
        return Err(format!(
            "kind « {kind} » absent de game.ron ; kinds du jeu : {}",
            declared.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }
    if kind == "Map" {
        return Err(
            "les cartes (kind Map, fichiers .ldtk) s'éditent dans LDtk, pas par `alacod set`".to_string(),
        );
    }
    let assets = GameManifest::assets_dir(game_dir);
    let files = ContentFiles::native(assets.clone());
    let mut ids = BTreeSet::new();
    let mut found: Vec<Entry> = Vec::new();
    let mut skipped = Vec::new();
    let mut seen = BTreeSet::new();
    for decl in manifest.content_folders.iter().filter(|d| d.kind == kind) {
        let Ok(paths) = files.discover(Path::new(&decl.path), "ron") else {
            continue;
        };
        for path in paths {
            let rel = path.to_string_lossy().replace('\\', "/");
            if !seen.insert(rel.clone()) {
                continue;
            }
            let Ok(text) = files.read(&path) else {
                continue;
            };
            let root = match parse(&text) {
                Ok(root) => root,
                Err(e) => {
                    skipped.push(format!("{rel} ({e})"));
                    continue;
                }
            };
            let abs = assets.join(&path);
            if let Some(Node::Map { entries, .. }) = table_root(&root) {
                for (k, v) in entries {
                    let key = key_text(k, &text);
                    ids.insert(key.to_string());
                    if key == id {
                        found.push(Entry {
                            rel: rel.clone(),
                            abs: abs.clone(),
                            text: text.clone(),
                            node: v.clone(),
                        });
                    }
                }
            } else {
                let stem = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let mut names = vec![stem];
                if let Node::Struct { fields, .. } = &root {
                    if let Some((_, v)) = fields.iter().find(|(k, _)| k == "asset_name_ref") {
                        names.push(key_text(v, &text).to_string());
                    }
                }
                ids.extend(names.iter().cloned());
                if names.iter().any(|n| n == id) {
                    found.push(Entry {
                        rel,
                        abs,
                        text,
                        node: root,
                    });
                }
            }
        }
    }
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => {
            let near = closest(id, &ids);
            let mut message = format!(
                "id « {id} » inconnu pour le kind {kind} ; le plus proche : {} ; ids : {}",
                near.first().map_or("aucun", String::as_str),
                list_some(&near)
            );
            if !skipped.is_empty() {
                message.push_str(&format!(" ; fichiers illisibles ignorés : {}", skipped.join(" ; ")));
            }
            Err(message)
        }
        _ => Err(format!(
            "id « {id} » ambigu pour le kind {kind} : défini dans {}",
            found.iter().map(|e| e.rel.as_str()).collect::<Vec<_>>().join(", ")
        )),
    }
}

// ---------------------------------------------------------------------------------------
// get
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Reading {
    pub kind: String,
    pub id: String,
    pub path: String,
    /// Fichier relatif à `assets/`.
    pub file: String,
    /// Le texte RON tel qu'il est dans le fichier (`"150.0"`, `Some(2)`, …).
    pub raw: String,
    /// La valeur pour l'affichage (une chaîne sans ses guillemets).
    pub value: String,
    /// La valeur est un réglage simple (sinon `raw` est un bloc : sous-chemins dans `leaves`).
    pub scalar: bool,
    pub leaves: Vec<String>,
}

/// `alacod get` : la valeur actuelle d'un chemin ; un bloc (struct, liste…) rend son texte et les chemins de ses
/// valeurs simples.
pub fn get(game_dir: &Path, kind: &str, id: &str, path: &str) -> Result<Reading, String> {
    let entry = locate(game_dir, kind, id)?;
    let node = resolve(&entry.node, path, &entry.text)?;
    let target = as_scalar(node);
    let shown = target.unwrap_or(node);
    let raw = shown.text(&entry.text).to_string();
    let mut leaf_paths = Vec::new();
    if target.is_none() {
        leaves(node, &entry.text, path, &mut leaf_paths);
    }
    Ok(Reading {
        kind: kind.to_string(),
        id: id.to_string(),
        path: path.to_string(),
        file: entry.rel,
        value: if target.is_some() {
            display_value(&raw)
        } else {
            raw.clone()
        },
        scalar: target.is_some(),
        raw,
        leaves: leaf_paths,
    })
}

// ---------------------------------------------------------------------------------------
// set
// ---------------------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct SetOptions {
    /// Écrit `value` tel quel à la place du nœud désigné (un bloc, `Some(..)`, `None`…).
    pub raw: bool,
    /// Commit `editor: <kind> <id> <chemin> <ancien> -> <nouveau>` du seul fichier touché.
    pub commit: bool,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub kind: String,
    pub id: String,
    pub path: String,
    /// Fichier relatif à `assets/`.
    pub file: String,
    pub old: String,
    pub new: String,
    /// Faux : la valeur était déjà celle-là, rien n'est écrit.
    pub changed: bool,
    /// Quand l'effet se voit en jeu (`docs/editeur-vocal.md` §5).
    pub effect: &'static str,
    /// Abrégé du commit `editor:` si `--commit`.
    pub commit: Option<String>,
    /// Erreurs de lint qui existaient avant le changement et subsistent (il n'en apparaît aucune de nouvelle).
    pub preexisting_lint_errors: usize,
}

impl Change {
    /// La ligne que la commande affiche.
    pub fn line(&self) -> String {
        if !self.changed {
            return format!(
                "{} {} {} : déjà {} (rien changé)",
                self.kind, self.id, self.path, self.new
            );
        }
        let mut line = format!(
            "{} {} {} : {} -> {} ({}) ; effet : {}",
            self.kind, self.id, self.path, self.old, self.new, self.file, self.effect
        );
        if let Some(sha) = &self.commit {
            line.push_str(&format!(" ; commit {sha}"));
        }
        line
    }
}

/// Quand un changement de ce kind se voit dans la partie en cours (`docs/editeur-vocal.md` §5).
pub fn effect_for(kind: &str) -> &'static str {
    match kind {
        "Character" => {
            "au prochain spawn (`alacod remote restart` pour les entités déjà là, joueur compris)"
        }
        "Weapon" | "Camera" | "Ui" | "SpriteSheet" => "immédiat (rechargement à chaud)",
        _ => "après `alacod remote restart` (le registre se relit seul, les tables dérivées au prochain démarrage de partie)",
    }
}

fn lint_errors(game_dir: &Path) -> Result<Vec<String>, String> {
    let (_, _, errors) = crate::load_and_lint(game_dir).map_err(|e| e.to_string())?;
    Ok(errors.iter().map(ToString::to_string).collect())
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git introuvable : {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        Err(format!(
            "git {} : {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// `alacod set` : remplace la valeur de `path`, lint, restaure le fichier si le lint s'aggrave, commit si demandé.
pub fn set(
    game_dir: &Path,
    kind: &str,
    id: &str,
    path: &str,
    value: &str,
    options: &SetOptions,
) -> Result<Change, String> {
    let entry = locate(game_dir, kind, id)?;
    let node = resolve(&entry.node, path, &entry.text)?;
    let target = if options.raw {
        node
    } else {
        as_scalar(node).ok_or_else(|| {
            let mut found = Vec::new();
            leaves(node, &entry.text, path, &mut found);
            format!(
                "chemin « {path} » désigne {}, pas une valeur simple ; réglages possibles : {} \
                 (ou --raw pour remplacer le bloc)",
                node.describe(),
                list_some(&found)
            )
        })?
    };
    let old_raw = target.text(&entry.text).to_string();
    let new_raw = if options.raw {
        parse(value).map_err(|e| format!("--raw « {value} » : {e}"))?;
        value.trim().to_string()
    } else {
        let value = match Relative::parse(value) {
            Some(relative) => apply_relative(&old_raw, value, relative)
                .map_err(|e| format!("chemin « {path} » : {e}"))?,
            None => value.to_string(),
        };
        format_value(&old_raw, &value).map_err(|e| format!("chemin « {path} » : {e}"))?
    };
    let mut change = Change {
        kind: kind.to_string(),
        id: id.to_string(),
        path: path.to_string(),
        file: entry.rel.clone(),
        old: display_value(&old_raw),
        new: display_value(&new_raw),
        changed: new_raw != old_raw,
        effect: effect_for(kind),
        commit: None,
        preexisting_lint_errors: 0,
    };
    if !change.changed {
        return Ok(change);
    }

    let dir = entry
        .abs
        .parent()
        .ok_or_else(|| format!("{} : pas de dossier parent", entry.abs.display()))?
        .to_path_buf();
    let name = entry
        .abs
        .file_name()
        .ok_or_else(|| format!("{} : pas de nom de fichier", entry.abs.display()))?
        .to_string_lossy()
        .into_owned();
    if options.commit {
        git(&dir, &["rev-parse", "--is-inside-work-tree"])
            .map_err(|e| format!("--commit demande un dépôt git : {e}"))?;
        let status = git(&dir, &["status", "--porcelain", "--", &name])?;
        if !status.is_empty() {
            return Err(format!(
                "--commit refusé : {} a déjà des changements non commités ({status}) que le commit \
                 emporterait ; les commiter ou les restaurer d'abord",
                entry.rel
            ));
        }
    }

    let before = lint_errors(game_dir)?;
    let span = target.span().clone();
    let mut edited = entry.text.clone();
    edited.replace_range(span, &new_raw);
    // Garde-fou : le fichier édité se relit encore (le jugement sur la valeur est celui du lint, plus bas).
    parse(&edited).map_err(|e| format!("l'édition produirait un RON illisible : {e}"))?;
    std::fs::write(&entry.abs, &edited).map_err(|e| format!("{} : {e}", entry.abs.display()))?;
    let restore = |why: String| -> String {
        match std::fs::write(&entry.abs, &entry.text) {
            Ok(()) => format!("{why} ; {} restauré", entry.rel),
            Err(e) => format!("{why} ; ATTENTION, {} n'a pas pu être restauré : {e}", entry.rel),
        }
    };
    let after = match lint_errors(game_dir) {
        Ok(after) => after,
        Err(e) => return Err(restore(format!("lint impossible après l'édition : {e}"))),
    };
    let known: BTreeSet<&String> = before.iter().collect();
    let fresh: Vec<&String> = after.iter().filter(|e| !known.contains(e)).collect();
    if !fresh.is_empty() {
        return Err(restore(format!(
            "lint rouge après {kind} {id} {path} = {} : {}",
            change.new,
            fresh.iter().map(|e| e.as_str()).collect::<Vec<_>>().join(" | ")
        )));
    }
    change.preexisting_lint_errors = after.len();

    if options.commit {
        let message = format!(
            "editor: {kind} {id} {path} {} -> {}",
            change.old, change.new
        );
        let committed = git(&dir, &["add", "--", &name])
            .and_then(|_| git(&dir, &["commit", "--only", "-m", &message, "--", &name]))
            .and_then(|_| git(&dir, &["rev-parse", "--short", "HEAD"]));
        match committed {
            Ok(sha) => change.commit = Some(sha),
            Err(e) => {
                let _ = git(&dir, &["reset", "-q", "--", &name]);
                return Err(restore(format!("commit impossible : {e}")));
            }
        }
    }
    Ok(change)
}

// ---------------------------------------------------------------------------------------
// undo
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Undone {
    /// Sujet du commit annulé.
    pub subject: String,
    /// Abrégé du commit d'annulation.
    pub commit: String,
    pub effect: &'static str,
}

/// `alacod undo` : annule le dernier commit **si et seulement si** son sujet commence par `editor:`.
///
/// Comme `git revert`, mais seuls les fichiers du commit sont touchés : le travail en cours ailleurs (fichiers
/// modifiés ou indexés) n'empêche rien et n'est pas emporté. Refusé si l'un de ces fichiers a des changements non
/// commités. Le commit d'annulation s'appelle `Revert "editor: …"` : un second `undo` ne l'annule donc pas.
pub fn undo(dir: &Path) -> Result<Undone, String> {
    let subject = git(dir, &["log", "-1", "--format=%s"])?;
    if !subject.starts_with("editor:") {
        return Err(format!(
            "le dernier commit est « {subject} » : ce n'est pas un commit `editor:`, rien n'est annulé"
        ));
    }
    let root = PathBuf::from(git(dir, &["rev-parse", "--show-toplevel"])?);
    let parents = git(&root, &["rev-list", "--parents", "-n", "1", "HEAD"])?;
    if parents.split_whitespace().count() != 2 {
        return Err(format!(
            "« {subject} » n'a pas exactement un parent (commit racine ou fusion) : rien n'est annulé"
        ));
    }
    let sha = git(&root, &["rev-parse", "HEAD"])?;
    let listed = git(&root, &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])?;
    let files: Vec<&str> = listed.lines().collect();
    let pathspec = |args: &mut Vec<String>| {
        args.push("--".to_string());
        args.extend(files.iter().map(|f| f.to_string()));
    };
    let run = |mut args: Vec<String>| -> Result<String, String> {
        pathspec(&mut args);
        git(&root, &args.iter().map(String::as_str).collect::<Vec<_>>())
    };
    let status = run(vec!["status".into(), "--porcelain".into()])?;
    if !status.is_empty() {
        return Err(format!(
            "annulation refusée : des fichiers du commit ont des changements non commités ({}) ; rien n'a changé",
            status.replace('\n', " | ")
        ));
    }
    let undo = run(vec!["checkout".into(), "HEAD~1".into()]).and_then(|_| {
        let revert = format!("Revert \"{subject}\"");
        let body = format!("This reverts commit {sha}.");
        run(vec![
            "commit".into(),
            "--only".into(),
            "-m".into(),
            revert,
            "-m".into(),
            body,
        ])
    });
    if let Err(e) = undo {
        let _ = run(vec!["checkout".into(), "HEAD".into()]);
        return Err(format!("annulation impossible (rien n'a changé) : {e}"));
    }
    let commit = git(dir, &["rev-parse", "--short", "HEAD"])?;
    // `editor: <Kind> <id> <chemin> <ancien> -> <nouveau>` : le kind dit quand l'effet se voit.
    let effect = subject
        .strip_prefix("editor:")
        .and_then(|rest| rest.split_whitespace().next())
        .map_or(effect_for(""), effect_for);
    Ok(Undone {
        subject,
        commit,
        effect,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEAPONS: &str = r#"// en-tête
({
    "pistolet": (
        config: (
            name: "pistolet", // commentaire
            ammo_type: Custom("balles"),
            firing_modes: {
                "default": (
                    firing_rate: "3.0",
                    firing_mode: Shotgun(pellet_count: 8, spread_angle: "0.4"),
                    bullet_type: Standard(
                        damage: "12.0",
                        speed: "380.0",
                    ),
                    mag: Mag(mag_size: 12, mag_limit: 9999),
                    on_expire: [Spawn(Ring(count: 8)), Spawn(Ring(count: 4))],
                )
            },
            test: Some((frames: 200, min_hits: 1)),
            aggro: Some("500.0"),
            rooms: (10, 14),
            list: [(stat: MoveSpeed, op: Pct, value: "0.25")],
        ),
    ),
})"#;

    fn at(path: &str) -> Result<String, String> {
        let root = parse(WEAPONS).unwrap();
        let Node::Tuple { items, .. } = &root else {
            panic!("racine");
        };
        let table = &items[0];
        let node = resolve(table, path, WEAPONS)?;
        Ok(as_scalar(node).unwrap_or(node).text(WEAPONS).to_string())
    }

    #[test]
    fn le_chemin_traverse_tables_variantes_some_et_listes() {
        assert_eq!(at("pistolet.config.name").unwrap(), "\"pistolet\"");
        let base = "pistolet.config.firing_modes.default";
        assert_eq!(at(&format!("{base}.firing_rate")).unwrap(), "\"3.0\"");
        assert_eq!(at(&format!("{base}.bullet_type.Standard.damage")).unwrap(), "\"12.0\"");
        assert_eq!(at(&format!("{base}.bullet_type.speed")).unwrap(), "\"380.0\"");
        assert_eq!(at(&format!("{base}.firing_mode.pellet_count")).unwrap(), "8");
        assert_eq!(at(&format!("{base}.mag.mag_size")).unwrap(), "12");
        assert_eq!(at("pistolet.config.test.frames").unwrap(), "200");
        assert_eq!(at("pistolet.config.aggro").unwrap(), "\"500.0\"");
        assert_eq!(at("pistolet.config.rooms.1").unwrap(), "14");
        assert_eq!(at("pistolet.config.list.0.value").unwrap(), "\"0.25\"");
        assert_eq!(at("pistolet.config.ammo_type").unwrap(), "\"balles\"");
    }

    #[test]
    fn un_chemin_inconnu_liste_les_candidats() {
        let e = at("pistolet.config.nom").unwrap_err();
        assert!(e.contains("pistolet.config.name") && e.contains("introuvable"), "{e}");
        let e = at("pistolet.config.firing_modes.default.bullet_type.Standard.dmg").unwrap_err();
        assert!(e.contains("Standard.damage"), "{e}");
        let e = at("pistolet.config.name.x").unwrap_err();
        assert!(e.contains("valeur simple"), "{e}");
    }

    #[test]
    fn un_nom_de_variante_en_double_est_ambigu_et_l_indice_tranche() {
        let base = "pistolet.config.firing_modes.default.on_expire";
        let e = at(&format!("{base}.Spawn.count")).unwrap_err();
        assert!(e.contains("ambigu") && e.contains(&format!("{base}.0")), "{e}");
        assert_eq!(at(&format!("{base}.1.count")).unwrap(), "4");
    }

    #[test]
    fn les_valeurs_gardent_leur_type() {
        assert_eq!(format_value("\"150.0\"", "120").unwrap(), "\"120.0\"");
        assert_eq!(format_value("\"150.0\"", "120.5").unwrap(), "\"120.5\"");
        assert_eq!(format_value("\"1\"", "2").unwrap(), "\"2\"");
        assert_eq!(format_value("\"150.0\"", "\"1 + floor\"").unwrap(), "\"1 + floor\"");
        assert_eq!(format_value("\"1 + floor\"", "2 + floor").unwrap(), "\"2 + floor\"");
        assert_eq!(format_value("12", "20").unwrap(), "20");
        assert!(format_value("12", "20.5").unwrap_err().contains("entier"));
        assert_eq!(format_value("3.0", "4").unwrap(), "4.0");
        assert!(format_value("3.0", "vite").unwrap_err().contains("nombre"));
        assert_eq!(format_value("true", "false").unwrap(), "false");
        assert!(format_value("true", "oui").is_err());
        assert_eq!(format_value("Common", "Rare").unwrap(), "Rare");
        assert!(format_value("None", "3").unwrap_err().contains("--raw"));
        assert_eq!(display_value("\"150.0\""), "150.0");
    }

    #[test]
    fn le_parseur_garde_les_positions_et_refuse_le_rest() {
        let src = "#![enable(implicit_some)]\n/* a /* b */ c */ (x: r#\"a\"b\"#, y: 'c', z: -1.5e-3, t: Name (1, 2))";
        let node = parse(src).unwrap();
        let Node::Struct { fields, .. } = &node else {
            panic!("struct attendu");
        };
        assert_eq!(fields[0].1.text(src), "r#\"a\"b\"#");
        assert_eq!(fields[2].1.text(src), "-1.5e-3");
        assert_eq!(fields[3].1.text(src), "Name (1, 2)");
        assert!(parse("(a: 1").is_err());
        assert!(parse("(a: 1) (b: 2)").unwrap_err().contains("ligne 1"));
    }
}
