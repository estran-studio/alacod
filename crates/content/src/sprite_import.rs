//! Import d'un manifeste pipeline `*.sprite.json` dans l'arbre `assets` d'un jeu (X-4b).
//!
//! Un outil externe de production de sprites décrit un personnage par un manifeste JSON :
//! une taille de case (`cell`), des directions dessinées (`directions`) et, par animation,
//! son PNG, sa grille et son rythme (`sheet`, `columns`, `rows`, `fps`, `loop`). Ce module
//! convertit ce manifeste en contenu engine générique, sans rien supposer du jeu : aucun nom,
//! réglage ou équilibrage d'un jeu particulier n'est écrit en dur ici.
//!
//! # Ce qui est généré (aucune animation abandonnée)
//!
//! Le schéma de présentation a **un PNG par `SpriteSheetConfig`** : chaque clip garde donc sa
//! feuille. Pour un sprite `id` avec les clips `c1`, `c2` :
//!
//! - `assets/sprites/<id>/<id>_<clip>_sheet.ron` : une `SpriteSheetConfig` par clip (case,
//!   grille, directions du manifeste, `scale`/`anchor`/`offset` des options) ;
//! - `assets/sprites/<id>/<id>_<clip>_animation.ron` : une `AnimationMapConfig` par clip, avec
//!   l'animation du clip (toute la ligne : `(row: 0, end: columns - 1)`, la ligne de direction
//!   étant ajoutée au rendu par `DirectionRow`) sous son nom **et** sous `"Idle"` (le repli du
//!   runtime quand l'état courant manque), `frame_duration` déduit du FPS du clip, et une table
//!   `clips` qui garde le FPS et la boucle par animation
//!   (`animation::AnimationClipMeta`, extension générique et facultative) ;
//! - les PNG copiés dans `assets/sprites/<id>/` ;
//! - une entrée `<id>_<clip>` par clip dans la table `SpriteSheet` du jeu (`sprites.ron`
//!   existant conservé, commentaires compris : ajout ou remplacement ciblé).
//!
//! # Échelle et ancrage
//!
//! Le manifeste porte un `pivot`/`ppm` propres à son pipeline : ils sont **ignorés** (documentés
//! comme tels, jamais lus en douce). L'échelle et l'ancrage du jeu sont des options explicites
//! ([`SpriteImportOptions`]) : rien n'est inféré d'un jeu particulier.
//!
//! # Échecs nets, jamais d'artefacts partiels
//!
//! Tout est validé **avant** d'écrire (manifeste, directions, grilles, dimensions PNG réelles) ;
//! une erreur ne crée ni ne modifie aucun fichier. Si une écriture échoue à mi-chemin, les
//! fichiers déjà créés sont retirés (au mieux) et `sprites.ron` n'est touché qu'en dernier.
//!
//! # Câblage CLI (X-2b)
//!
//! `alacod import-sprite <manifeste> [jeu] [--id …] [--scale …] [--anchor …] [--offset …]
//! [--mirror/--no-mirror] [--overwrite]` appelle
//! [`import_sprite`] avec [`SpriteImportOptions`] et affiche [`SpriteImportResult`] ou
//! [`SpriteImportError`]. Ce module ne connaît pas `clap` : la CLI reste dans
//! `crates/content/src/bin/alacod/main.rs`, propriété de X-2b.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Ancre d'une feuille générée : les neuf variantes de `animation::ConfigurableAnchor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpriteAnchor {
    #[default]
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight,
}

impl SpriteAnchor {
    fn as_ron(&self) -> &'static str {
        match self {
            SpriteAnchor::Center => "Center",
            SpriteAnchor::BottomLeft => "BottomLeft",
            SpriteAnchor::BottomCenter => "BottomCenter",
            SpriteAnchor::BottomRight => "BottomRight",
            SpriteAnchor::CenterLeft => "CenterLeft",
            SpriteAnchor::CenterRight => "CenterRight",
            SpriteAnchor::TopLeft => "TopLeft",
            SpriteAnchor::TopCenter => "TopCenter",
            SpriteAnchor::TopRight => "TopRight",
        }
    }
}

/// Options d'import : tout ce qui est propre au jeu est ici, jamais deviné.
#[derive(Debug, Clone)]
pub struct SpriteImportOptions {
    /// Id du sprite (`None` : nom du manifeste assaini). Préfixe des fichiers et des entrées.
    pub sprite_id: Option<String>,
    /// Échelle de rendu des feuilles (`1.0` par défaut).
    pub scale: f32,
    /// Ancre des feuilles ([`SpriteAnchor::Center`] par défaut).
    pub anchor: SpriteAnchor,
    /// Décalage de rendu `(x, y, z)` (`(0.0, 0.0, 1.0)` par défaut, comme les feuilles du dépôt).
    pub offset: (f32, f32, f32),
    /// `Some(true)` / `Some(false)` : le côté non dessiné est / n'est pas la ligne retournée.
    /// `None` (défaut) : `true`, le seul choix sans perte pour 1 et 5 directions et sans effet
    /// sur 8 directions complètes (les candidats miroir y perdent au tie-break).
    pub mirror: Option<bool>,
    /// Écraser les feuilles, animations et PNG déjà présents (jamais les entrées des autres
    /// sprites de `sprites.ron`). `false` par défaut : un conflit est une erreur.
    pub overwrite: bool,
}

impl Default for SpriteImportOptions {
    fn default() -> Self {
        Self {
            sprite_id: None,
            scale: 1.0,
            anchor: SpriteAnchor::Center,
            offset: (0.0, 0.0, 1.0),
            mirror: None,
            overwrite: false,
        }
    }
}

/// Un clip importé : une entrée `sprites.ron` jouable telle quelle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedClip {
    /// Nom du clip dans le manifeste.
    pub clip: String,
    /// Id dans la table `SpriteSheet` (`<sprite>_<clip>`).
    pub entry_id: String,
    /// Chemins relatifs à `assets/` : feuille, animation, image.
    pub sheet: String,
    pub animation: String,
    pub image: String,
    /// Grille du PNG : colonnes, lignes, images par ligne.
    pub columns: u32,
    pub rows: u32,
    pub frames_per_row: u32,
    /// Rythme : millisecondes par image (de `fps`) et boucle (de `loop`).
    pub frame_duration_ms: u64,
    pub looping: bool,
}

/// Résultat d'un import réussi : tout est écrit et `sprites.ron` à jour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteImportResult {
    pub sprite_id: String,
    /// Table `SpriteSheet` mise à jour, relative à `assets/` (ex. `sprites/sprites.ron`).
    pub table: String,
    pub clips: Vec<ImportedClip>,
}

/// Échec d'import : rien n'a été écrit ni modifié (voir l'en-tête du module).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpriteImportError {
    #[error("manifeste illisible ({path}) : {message}")]
    ManifestUnreadable { path: String, message: String },
    #[error("manifeste invalide ({path}) : {message}")]
    ManifestInvalid { path: String, message: String },
    #[error("jeu invalide ({game_dir}) : {message}")]
    GameInvalid { game_dir: String, message: String },
    #[error("image incompatible (clip « {clip} », {path}) : {message}")]
    ImageMismatch {
        clip: String,
        path: String,
        message: String,
    },
    #[error("conflit (sprites.ron ou assets) : {message}")]
    Conflict { message: String },
    #[error("écriture impossible ({path}) : {message}")]
    WriteFailed { path: String, message: String },
}

/// Directions du compas reconnues, dans l'ordre des lignes : celles de
/// `sim_core::sprite_directions::Compass`, sans dépendre de la présentation.
const COMPASS: [&str; 8] = ["E", "NE", "N", "NW", "W", "SW", "S", "SE"];

/// Index d'atlas d'une image : ligne de direction `dir_row`, image `frame` dans la ligne.
/// Même arithmétique que le runtime (`DirectionRow.row × columns + image`) : un test la fige.
pub fn frame_index(columns: u32, dir_row: usize, frame: usize) -> usize {
    dir_row * columns as usize + frame
}

#[derive(Debug, serde::Deserialize)]
struct PipelineManifest {
    name: String,
    cell: Option<[u32; 2]>,
    directions: Option<Vec<String>>,
    animations: Option<BTreeMap<String, ClipDecl>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ClipDecl {
    sheet: String,
    columns: u32,
    rows: u32,
    fps: f64,
    #[serde(default = "default_loop", rename = "loop")]
    loop_clip: bool,
}

fn default_loop() -> bool {
    true
}

struct ValidClip {
    clip: String,
    entry_id: String,
    sheet_name: String,
    columns: u32,
    rows: u32,
    frame_duration_ms: u64,
    looping: bool,
    png_bytes: Vec<u8>,
}

fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+'
}

/// Assainit un nom de manifeste ou de clip en id : tout caractère hors `[A-Za-z0-9_+-]`
/// devient `_`. Vide après assainissement : `None`.
fn sanitize_id(raw: &str) -> Option<String> {
    let out: String = raw.chars().map(|c| if is_id_char(c) { c } else { '_' }).collect();
    let out = out.trim_matches('_').to_string();
    if out.is_empty() { None } else { Some(out) }
}

fn fmt_f32(v: f32) -> String {
    format!("{v:?}")
}

fn png_dimensions(bytes: &[u8], path: &str, clip: &str) -> Result<(u32, u32), SpriteImportError> {
    const SIG: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    let mismatch = |message: String| SpriteImportError::ImageMismatch {
        clip: clip.to_string(),
        path: path.to_string(),
        message,
    };
    if bytes.len() < 33 {
        return Err(mismatch(format!(
            "fichier trop court ({} octets) : PNG tronqué ou absent",
            bytes.len()
        )));
    }
    if bytes[0..8] != SIG {
        return Err(mismatch(
            "signature absente : ce n'est pas un PNG (l'engine ne charge que du PNG ici)".into(),
        ));
    }
    let len = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if &bytes[12..16] != b"IHDR" || len != 13 {
        return Err(mismatch(
            "premier chunk inattendu : PNG non standard, réexportez-le".into(),
        ));
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if w == 0 || h == 0 {
        return Err(mismatch("dimensions nulles dans l'en-tête IHDR".into()));
    }
    Ok((w, h))
}

fn fps_to_ms(fps: f64) -> Option<u64> {
    if !fps.is_finite() || fps <= 0.0 {
        return None;
    }
    Some((1000.0 / fps).round().clamp(1.0, u64::MAX as f64) as u64)
}

/// Lit le chemin de la table `SpriteSheet` dans `assets/game.ron` (`content_folders`).
fn sprites_table_rel(game_ron: &str) -> Option<String> {
    let mut search = 0;
    while let Some(found) = game_ron[search..].find("\"SpriteSheet\"") {
        let pos = search + found;
        let window_start = pos.saturating_sub(400);
        let window = &game_ron[window_start..pos];
        if let Some(p) = window.rfind("path:") {
            let after = &window[p + 5..];
            let after = after.trim_start();
            if let Some(rest) = after.strip_prefix('"') {
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
        search = pos + 1;
    }
    None
}

/// Insère ou remplace une entrée `"id": (...)` dans une table RON, commentaires et autres
/// entrées préservés. `body` inclut ses parenthèses.
fn upsert_table_entry(table: &str, entry_id: &str, body: &str) -> Result<String, String> {
    let key = format!("\"{entry_id}\"");
    if let Some(span) = entry_span(table, &key) {
        let mut out = table.to_string();
        out.replace_range(span, body);
        return Ok(out);
    }
    insert_table_entry(table, &key, body)
}

fn entry_span(table: &str, key: &str) -> Option<std::ops::Range<usize>> {
    let bytes = table.as_bytes();
    let mut i = 0;
    while let Some(found) = table[i..].find(key) {
        let k = i + found;
        // Hors chaîne ou commentaire : le scan naïf suffit, vérifié par le `:` qui suit.
        let mut j = k + key.len();
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if bytes.get(j) != Some(&b':') {
            i = k + key.len();
            continue;
        }
        j += 1;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if bytes.get(j) != Some(&b'(') {
            i = k + key.len();
            continue;
        }
        let end = match_paren(table, j).ok()?;
        return Some(k..end);
    }
    None
}

/// Fin (exclue) de la parenthèse équilibrée ouvrant en `open`, chaînes et commentaires sautés.
fn match_paren(text: &str, open: usize) -> Result<usize, String> {
    let b = text.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
            }
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => i += 1,
        }
    }
    Err("parenthèse non fermée".to_string())
}

fn insert_table_entry(table: &str, key: &str, body: &str) -> Result<String, String> {
    let trimmed_end = table.trim_end();
    if !trimmed_end.ends_with('}') {
        return Err("table sans `}` final : refuse de modifier un fichier inattendu".to_string());
    }
    let without_brace = trimmed_end[..trimmed_end.len() - 1].trim_end().to_string();
    if !without_brace.ends_with(')') {
        return Err("table sans `)` final : refuse de modifier un fichier inattendu".to_string());
    }
    let inner = without_brace[..without_brace.len() - 1].to_string();
    let trailing_nl = if table.ends_with('\n') { "\n" } else { "" };
    let needs_comma = !inner.trim_end().ends_with('(') && !inner.trim().is_empty();
    // Garde l'indentation du fichier : 4 espaces comme les tables du dépôt.
    let entry = format!("{key}: {body}");
    let mut out = inner;
    if needs_comma {
        out.push(',');
    }
    // Table vide `({})` : pas de virgule, entrée sur sa ligne.
    out.push('\n');
    out.push_str("    ");
    out.push_str(&entry);
    out.push_str(",\n})");
    out.push_str(trailing_nl);
    Ok(out)
}

/// Importe `manifest_path` (`*.sprite.json`) dans `game_dir` (dossier avec `assets/game.ron`).
pub fn import_sprite(
    manifest_path: &Path,
    game_dir: &Path,
    options: SpriteImportOptions,
) -> Result<SpriteImportResult, SpriteImportError> {
    let manifest_display = manifest_path.display().to_string();
    let game_display = game_dir.display().to_string();
    let unreadable = |message: String| SpriteImportError::ManifestUnreadable {
        path: manifest_display.clone(),
        message,
    };
    let invalid = |message: String| SpriteImportError::ManifestInvalid {
        path: manifest_display.clone(),
        message,
    };
    let game_invalid = |message: String| SpriteImportError::GameInvalid {
        game_dir: game_display.clone(),
        message,
    };

    if !options.scale.is_finite() || options.scale <= 0.0 {
        return Err(invalid(format!(
            "scale = {} : doit être un nombre fini > 0 (option --scale)",
            options.scale
        )));
    }
    for (name, v) in [
        ("offset_x", options.offset.0),
        ("offset_y", options.offset.1),
        ("offset_z", options.offset.2),
    ] {
        if !v.is_finite() {
            return Err(invalid(format!(
                "{name} = {v} : doit être un nombre fini (option --offset)"
            )));
        }
    }

    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        unreadable("chemin sans dossier parent : donnez un fichier `*.sprite.json`".to_string())
    })?;
    let raw = std::fs::read_to_string(manifest_path)
        .map_err(|e| unreadable(format!("lecture impossible : {e}")))?;
    let manifest: PipelineManifest =
        serde_json::from_str(&raw).map_err(|e| invalid(format!("JSON illisible : {e}")))?;

    let sprite_id = match &options.sprite_id {
        Some(id) => sanitize_id(id).ok_or_else(|| {
            invalid(format!(
                "id « {id} » vide après assainissement ([A-Za-z0-9_+-], le reste devient `_`)"
            ))
        })?,
        None => sanitize_id(&manifest.name).ok_or_else(|| {
            invalid(format!(
                "nom « {} » vide après assainissement : donnez --id ([A-Za-z0-9_+-])",
                manifest.name
            ))
        })?,
    };
    if manifest.name.trim().is_empty() {
        return Err(invalid("champ `name` vide : un sprite sans nom".to_string()));
    }
    let Some([cell_w, cell_h]) = manifest.cell else {
        return Err(invalid(
            "champ `cell: [largeur, hauteur]` absent : taille de case inconnue".to_string(),
        ));
    };
    if cell_w == 0 || cell_h == 0 {
        return Err(invalid(format!(
            "cell = [{cell_w}, {cell_h}] : dimensions strictement positives"
        )));
    }
    let directions: Option<Vec<String>> = match manifest.directions {
        None => None,
        Some(dirs) => {
            if dirs.is_empty() {
                return Err(invalid(
                    "`directions` est vide : au moins une direction, ou champ absent".to_string(),
                ));
            }
            for d in &dirs {
                if !COMPASS.contains(&d.as_str()) {
                    return Err(invalid(format!(
                        "direction « {d} » inconnue : E, NE, N, NW, W, SW, S, SE exactement"
                    )));
                }
            }
            for (n, d) in dirs.iter().enumerate() {
                if dirs[..n].contains(d) {
                    return Err(invalid(format!(
                        "direction « {d} » répétée (ligne {n}) : une ligne par direction"
                    )));
                }
            }
            Some(dirs)
        }
    };
    let mirror = options.mirror.unwrap_or(true);
    let animations = manifest.animations.ok_or_else(|| {
        invalid("champ `animations` absent : rien à importer (au moins un clip)".to_string())
    })?;
    if animations.is_empty() {
        return Err(invalid(
            "`animations` est vide : rien à importer (au moins un clip)".to_string(),
        ));
    }

    // Valide chaque clip (grille, rythme, PNG réel) avant la moindre écriture.
    let mut clips: Vec<ValidClip> = Vec::new();
    let mut seen_entries: BTreeMap<String, String> = BTreeMap::new();
    for (clip_name, clip) in &animations {
        let clip_here = || clip_name.clone();
        if clip_name.trim().is_empty() || clip_name.contains('"') || clip_name.contains('\n') {
            return Err(invalid(format!(
                "clip « {} » : nom vide ou avec guillemet/retour : renommez-le",
                clip_name
            )));
        }
        if clip.columns == 0 || clip.rows == 0 {
            return Err(invalid(format!(
                "clip « {} » : columns = {}, rows = {} : strictement positifs",
                clip_name, clip.columns, clip.rows
            )));
        }
        if let Some(dirs) = &directions {
            if clip.rows as usize != dirs.len() {
                return Err(invalid(format!(
                    "clip « {} » : rows = {} pour {} direction(s) : une ligne par direction \
                     (les planches multi-blocs ne s'importent pas, découpez-les)",
                    clip_name,
                    clip.rows,
                    dirs.len()
                )));
            }
        }
        let frame_duration_ms = fps_to_ms(clip.fps).ok_or_else(|| {
            invalid(format!(
                "clip « {} » : fps = {} : nombre fini > 0",
                clip_name, clip.fps
            ))
        })?;
        let sheet_src = manifest_dir.join(&clip.sheet);
        let png_bytes = std::fs::read(&sheet_src).map_err(|e| SpriteImportError::ImageMismatch {
            clip: clip_here(),
            path: sheet_src.display().to_string(),
            message: format!("PNG illisible à côté du manifeste : {e}"),
        })?;
        let (w, h) = png_dimensions(
            &png_bytes,
            &sheet_src.display().to_string(),
            clip_name,
        )?;
        if w != clip.columns * cell_w || h != clip.rows * cell_h {
            return Err(SpriteImportError::ImageMismatch {
                clip: clip_here(),
                path: sheet_src.display().to_string(),
                message: format!(
                    "PNG {w}×{h} px, grille {}×{} cases de {cell_w}×{cell_h} px = {}×{} px attendus",
                    clip.columns,
                    clip.rows,
                    clip.columns * cell_w,
                    clip.rows * cell_h,
                ),
            });
        }
        let sanitized = sanitize_id(clip_name).ok_or_else(|| {
            invalid(format!(
                "clip « {} » vide après assainissement : renommez-le",
                clip_name
            ))
        })?;
        let entry_id = format!("{sprite_id}_{sanitized}");
        if let Some(first) = seen_entries.insert(entry_id.clone(), clip_name.clone()) {
            return Err(invalid(format!(
                "clips « {first} » et « {clip_name} » : même entrée « {entry_id} » après \
                 assainissement : renommez-en un"
            )));
        }
        clips.push(ValidClip {
            clip: clip_name.clone(),
            entry_id,
            sheet_name: Path::new(&clip.sheet)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&clip.sheet)
                .to_string(),
            columns: clip.columns,
            rows: clip.rows,
            frame_duration_ms,
            looping: clip.loop_clip,
            png_bytes,
        });
    }
    clips.sort_by(|a, b| a.clip.cmp(&b.clip));

    // Jeu : `assets/game.ron` et sa table `SpriteSheet`.
    let assets_dir = game_dir.join("assets");
    let game_ron_path = assets_dir.join("game.ron");
    let game_ron = std::fs::read_to_string(&game_ron_path).map_err(|e| {
        game_invalid(format!(
            "assets/game.ron illisible (dossier du jeu = celui qui contient assets/) : {e}"
        ))
    })?;
    let table_rel = sprites_table_rel(&game_ron).ok_or_else(|| {
        game_invalid(
            "aucun `(path: …, kind: \"SpriteSheet\")` dans content_folders : déclarez la table \
             des sprites avant d'importer"
                .to_string(),
        )
    })?;
    let table_path = assets_dir.join(&table_rel);
    let table_text = match std::fs::read_to_string(&table_path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            "({\n})".to_string() + "\n"
        }
        Err(e) => {
            return Err(SpriteImportError::WriteFailed {
                path: table_path.display().to_string(),
                message: format!("table illisible : {e}"),
            });
        }
    };
    // Conflits d'abord : aucune écriture n'a encore eu lieu.
    for clip in &clips {
        if entry_span(&table_text, &format!("\"{}\"", clip.entry_id)).is_some() && !options.overwrite
        {
            return Err(SpriteImportError::Conflict {
                message: format!(
                    "entrée « {} » déjà dans {} : --overwrite pour la remplacer",
                    clip.entry_id, table_rel
                ),
            });
        }
    }

    let sprite_assets = assets_dir.join("sprites").join(&sprite_id);
    let mut created: Vec<PathBuf> = Vec::new();
    let mut cleanup = |created: &[PathBuf]| {
        for p in created {
            let _ = std::fs::remove_file(p);
        }
    };

    let sheet_ron = |clip: &ValidClip| -> String {
        let dirs_ron = match &directions {
            None => String::new(),
            Some(dirs) => {
                let rows = dirs.join(", ");
                let mirror_ron = if mirror { ", mirror: true" } else { "" };
                format!("    directions: Some((rows: [{rows}]{mirror_ron})),\n")
            }
        };
        format!(
            "(\n    path: \"sprites/{sprite_id}/{png}\",\n    tile_size: ({cell_w}, {cell_h}),\n    columns: {cols},\n    rows: {rows},\n    animated: {animated},\n    name: \"body\",\n    scale: {scale},\n    anchor: {anchor},\n    offset_x: {ox},\n    offset_y: {oy},\n    offset_z: {oz},\n{dirs})",
            sprite_id = sprite_id,
            png = clip.sheet_name,
            cols = clip.columns,
            rows = clip.rows,
            animated = clip.columns * clip.rows > 1,
            scale = fmt_f32(options.scale),
            anchor = options.anchor.as_ron(),
            ox = fmt_f32(options.offset.0),
            oy = fmt_f32(options.offset.1),
            oz = fmt_f32(options.offset.2),
            dirs = dirs_ron,
        )
    };
    let anim_ron = |clip: &ValidClip| -> String {
        let end = clip.columns - 1;
        let idle_alias = if clip.clip == "Idle" {
            String::new()
        } else {
            format!("        \"Idle\": (row: 0, end: {end}),\n")
        };
        let bool_ron = |b: bool| if b { "true" } else { "false" };
        format!(
            "(\n    frame_duration: {ms},\n    columns: Some({cols}),\n    animations: {{\n        \"{clip}\": (row: 0, end: {end}),\n{alias}    }},\n    clips: {{\n        \"{clip}\": (frame_duration: Some({ms}), looping: Some({looping})),\n    }},\n)",
            ms = clip.frame_duration_ms,
            cols = clip.columns,
            clip = clip.clip,
            end = end,
            alias = idle_alias,
            looping = bool_ron(clip.looping),
        )
    };

    let write_file = |path: &Path, bytes: &[u8], created: &mut Vec<PathBuf>| -> Result<(), SpriteImportError> {
        if path.exists() && !options.overwrite {
            return Err(SpriteImportError::Conflict {
                message: format!(
                    "{} existe déjà : --overwrite pour le remplacer",
                    path.display()
                ),
            });
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SpriteImportError::WriteFailed {
                path: path.display().to_string(),
                message: format!("dossier impossible : {e}"),
            })?;
        }
        std::fs::write(path, bytes).map_err(|e| SpriteImportError::WriteFailed {
            path: path.display().to_string(),
            message: format!("écriture impossible : {e}"),
        })?;
        created.push(path.to_path_buf());
        Ok(())
    };

    let mut result_clips = Vec::new();
    let mut writes: Result<(), SpriteImportError> = Ok(());
    for clip in &clips {
        let sheet_file = format!("{}_{}_sheet.ron", sprite_id, sanitize_id(&clip.clip).unwrap_or_else(|| "clip".into()));
        let anim_file = format!("{}_{}_animation.ron", sprite_id, sanitize_id(&clip.clip).unwrap_or_else(|| "clip".into()));
        // Noms stables : entrée `<sprite>_<clip>` ; fichiers assortis.
        let sheet_rel = format!("sprites/{sprite_id}/{sheet_file}");
        let anim_rel = format!("sprites/{sprite_id}/{anim_file}");
        let image_rel = format!("sprites/{sprite_id}/{}", clip.sheet_name);
        if writes.is_ok() {
            writes = write_file(&sprite_assets.join(&sheet_file), sheet_ron(clip).as_bytes(), &mut created)
                .and_then(|_| write_file(&sprite_assets.join(&anim_file), anim_ron(clip).as_bytes(), &mut created))
                .and_then(|_| write_file(&sprite_assets.join(&clip.sheet_name), &clip.png_bytes, &mut created));
        }
        result_clips.push(ImportedClip {
            clip: clip.clip.clone(),
            entry_id: clip.entry_id.clone(),
            sheet: sheet_rel,
            animation: anim_rel,
            image: image_rel,
            columns: clip.columns,
            rows: clip.rows,
            frames_per_row: clip.columns,
            frame_duration_ms: clip.frame_duration_ms,
            looping: clip.looping,
        });
    }
    if let Err(e) = writes {
        cleanup(&created);
        return Err(e);
    }

    // `sprites.ron` en dernier : une fois écrit, l'import est complet.
    let mut new_table = table_text.clone();
    for imported in result_clips.iter() {
        let sheet_ron_name = Path::new(&imported.sheet).file_name().unwrap().to_str().unwrap();
        let anim_ron_name = Path::new(&imported.animation).file_name().unwrap().to_str().unwrap();
        let body = format!(
            "(\n        animation: \"sprites/{sprite_id}/{anim}\",\n        layers: {{ \"body\": \"sprites/{sprite_id}/{sheet}\" }},\n    )",
            sprite_id = sprite_id,
            anim = anim_ron_name,
            sheet = sheet_ron_name,
        );
        new_table = upsert_table_entry(&new_table, &imported.entry_id, &body).map_err(|message| {
            cleanup(&created);
            SpriteImportError::GameInvalid {
                game_dir: game_display.clone(),
                message: format!("{table_rel} : {message}"),
            }
        })?;
    }
    if table_path.exists() {
        std::fs::write(&table_path, new_table.as_bytes()).map_err(|e| {
            cleanup(&created);
            SpriteImportError::WriteFailed {
                path: table_path.display().to_string(),
                message: format!("table non mise à jour, fichiers créés retirés : {e}"),
            }
        })?;
    } else {
        if let Some(parent) = table_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                cleanup(&created);
                SpriteImportError::WriteFailed {
                    path: table_path.display().to_string(),
                    message: format!("dossier de la table impossible : {e}"),
                }
            })?;
        }
        let header = "// Table des feuilles de sprites : une entrée par sprite importé ou écrit à la main.\n";
        std::fs::write(&table_path, format!("{header}{new_table}")).map_err(|e| {
            cleanup(&created);
            SpriteImportError::WriteFailed {
                path: table_path.display().to_string(),
                message: format!("table non créée, fichiers créés retirés : {e}"),
            }
        })?;
    }

    Ok(SpriteImportResult {
        sprite_id,
        table: table_rel,
        clips: result_clips,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_names_cover_eight_rows() {
        assert_eq!(COMPASS.len(), 8);
        assert!(COMPASS.contains(&"SE"));
    }

    #[test]
    fn fps_to_ms_rounds_and_rejects() {
        assert_eq!(fps_to_ms(12.0), Some(83));
        assert_eq!(fps_to_ms(6.0), Some(167));
        assert_eq!(fps_to_ms(0.0), None);
        assert_eq!(fps_to_ms(-3.0), None);
        assert_eq!(fps_to_ms(f64::NAN), None);
        assert_eq!(fps_to_ms(f64::INFINITY), None);
    }

    #[test]
    fn frame_index_matches_runtime_direction_offset() {
        assert_eq!(frame_index(12, 0, 0), 0);
        assert_eq!(frame_index(12, 3, 5), 41);
        assert_eq!(frame_index(4, 1, 3), 7);
    }

    #[test]
    fn sanitize_id_replaces_and_trims() {
        assert_eq!(sanitize_id("marche avant!").as_deref(), Some("marche_avant"));
        assert_eq!(sanitize_id("???"), None);
        assert_eq!(sanitize_id("idle2").as_deref(), Some("idle2"));
    }

    #[test]
    fn upsert_inserts_into_empty_and_keeps_comments() {
        let table = "// tête\n({\n})\n";
        let out = upsert_table_entry(table, "hero_idle", "(animation: \"a\")").unwrap();
        assert!(out.contains("// tête"), "{out}");
        assert!(out.contains("\"hero_idle\": (animation: \"a\")"), "{out}");
    }

    #[test]
    fn upsert_appends_beside_existing_entry() {
        let table = "({\n    \"keep\": (animation: \"k\"),\n})";
        let out = upsert_table_entry(table, "new_idle", "(animation: \"a\")").unwrap();
        assert!(out.contains("\"keep\": (animation: \"k\")"), "{out}");
        assert!(out.contains("\"new_idle\": (animation: \"a\")"), "{out}");
    }

    #[test]
    fn upsert_replaces_matching_entry_only() {
        let table = "({\n    \"hero_idle\": (animation: \"old\"),\n    \"other\": (animation: \"o\"),\n})";
        let out = upsert_table_entry(table, "hero_idle", "(animation: \"new\")").unwrap();
        assert!(out.contains("\"hero_idle\": (animation: \"new\")"), "{out}");
        assert!(out.contains("\"other\": (animation: \"o\")"), "{out}");
        assert!(!out.contains("old"), "{out}");
    }

    #[test]
    fn png_dimensions_reads_ihdr() {
        // 4×2 RGBA : signature + IHDR (longueur 13) + opaque.
        let mut bytes = vec![137, 80, 78, 71, 13, 10, 26, 10];
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&4u32.to_be_bytes());
        bytes.extend_from_slice(&2u32.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes.extend_from_slice(&[0; 4]);
        assert_eq!(png_dimensions(&bytes, "t.png", "c"), Ok((4, 2)));
    }

    #[test]
    fn png_dimensions_rejects_non_png() {
        let err = png_dimensions(b"GIF89a........", "t.gif", "c").unwrap_err();
        assert!(err.to_string().contains("pas un PNG"), "{err}");
    }
}
