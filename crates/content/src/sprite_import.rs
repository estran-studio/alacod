//! Import d'un manifeste pipeline `*.sprite.json` dans l'arbre `assets` d'un jeu (X-4b, D-X4b-1).
//!
//! Un outil externe de production de sprites décrit un personnage par un manifeste JSON : une
//! taille de case (`cell`), des directions dessinées (`directions`), un `pivot` et un `ppm`
//! (pixels par mètre) et, par animation (« clip »), sa planche PNG, sa grille et son rythme
//! (`sheet`, `columns`, `rows`, `fps`, `loop`). Ce module convertit ce manifeste en contenu engine
//! générique ; aucun nom, réglage ou équilibrage d'un jeu particulier n'est écrit ici.
//!
//! # Un atlas réduit, une entrée `sprites.ron` par personnage
//!
//! Le schéma de présentation a un PNG par `SpriteSheetConfig` et un seul `asset_name_ref` par
//! personnage : des planches HD séparées (dix textures de 3008×2976 px ≈ 360 Mo de mémoire vidéo)
//! ne passeraient pas d'un clip à l'autre. L'import **réduit** donc chaque planche et compose les
//! clips en **blocs** (colonnes = images, lignes = directions) dans **un seul PNG** de côté
//! `≤ max_side` (4096 par défaut) ; les cases inutilisées sont transparentes.
//!
//! ```text
//! atlas : bx blocs de large × by blocs de haut, chaque bloc = colonnes × lignes de directions
//! ┌──────────┬──────────┬──────────┐
//! │ clip 0   │ clip 1   │ clip 2   │  ← ligne de blocs 0
//! ├──────────┼──────────┼──────────┤
//! │ clip 3   │ clip 4   │ (vide)   │  ← ligne de blocs 1
//! └──────────┴──────────┴──────────┘
//! ```
//!
//! - `SpriteSheetConfig` (calque `body`) : `tile_size` = cellule **réduite**, `columns`/`rows` = grille
//!   de l'atlas (lignes multiples du nombre de directions, comme le lint X-4a l'exige),
//!   `directions: rows: [ordre du manifeste]`.
//! - `AnimationMapConfig` : chaque clip en `Direct { start, end }` sur la **première ligne de son
//!   bloc** (`start = ligne_du_bloc × colonnes_atlas + colonne_du_bloc`). Le runtime ajoute
//!   `DirectionRow × colonnes_atlas` (la ligne de direction, 0 à `directions - 1`) : il retombe donc
//!   sur la ligne de la direction **dans le bloc**, même quand le bloc ne commence pas à la colonne 0.
//!   Les états demandés (`--state Idle=idle`) reçoivent les indices du clip sous leur nom ; **aucun
//!   clip n'est abandonné** (il reste sous son propre nom). `clips` porte la durée d'image et la
//!   boucle de chaque clip (`animation::AnimationClipMeta`, facultatif et rétrocompatible).
//! - Le facteur de réduction est le plus grand qui tient sous `max_side` (jamais d'agrandissement),
//!   sauf si `cell_height` l'impose. La réduction se fait cellule par cellule en **alpha prémultiplié**
//!   (pas de halo sombre), filtre Lanczos3, en arithmétique flottante à ordre fixe : relancer l'import
//!   donne le même PNG, à l'octet près.
//!
//! # Échelle et pivot : lus dans le manifeste, reportés explicitement
//!
//! Pas d'échelle implicite : `units_per_meter` (U) donne la taille affichée, `U / ppm` unités monde par
//! pixel source, donc `scale = (U / ppm) / facteur` ; `scale` direct est accepté à la place. Le
//! `pivot` (pixels de la cellule source, origine en haut à gauche) devient `anchor: Center` plus
//! `offset_x` / `offset_y` en unités monde (`ConfigurableAnchor` n'a pas de `Custom`) : le pivot
//! tombe sur l'origine du personnage, ou sur `pivot_at` (`--pivot-at X,Y`, unités locales de l'entité, `0,0` par défaut :
//! `offset = (centre − pivot) × échelle + pivot_at`). **Le pivot d'un manifeste est aux pieds (point de contact au sol)** : les
//! personnages du testbed ont les leurs au bas du collider, d'où `--pivot-at`. La hauteur réelle de la silhouette (boîte alpha des images d'`idle`)
//! est mesurée et reportée pour choisir U.
//!
//! # Échecs nets, jamais d'artefacts partiels
//!
//! Tout est validé et composé en mémoire **avant** d'écrire (manifeste, directions, dimensions des PNG,
//! atlas, conflits) ; une erreur ne crée ni ne modifie aucun fichier. Si une écriture échoue à mi-chemin,
//! les fichiers créés sont retirés (au mieux) et `sprites.ron` n'est touché qu'en dernier.
//!
//! Les ancres de mains (`*_mains_*.png`, `ancres.json`) ne sont pas lues : `design/sprites-composables.md`
//! n'est pas tranché.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgba, RgbaImage};

/// Taille affichée : une valeur explicite, jamais devinée.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleSpec {
    /// `U` unités monde pour un mètre du manifeste (`ppm` pixels source) : `scale = (U / ppm) / facteur`.
    UnitsPerMeter(f32),
    /// `scale` posé tel quel sur la feuille (affiche chaque pixel de l'atlas réduit sur `scale` unités).
    Scale(f32),
}

/// Options d'import : tout ce qui est propre au jeu est ici.
#[derive(Debug, Clone)]
pub struct SpriteImportOptions {
    /// Id du sprite (`None` : nom du manifeste assaini). Nom du dossier, des fichiers et de l'entrée.
    pub sprite_id: Option<String>,
    pub scale: ScaleSpec,
    /// États de présentation → clip (`Idle` → `idle`) : la clé d'état reçoit les indices du clip.
    pub states: Vec<(String, String)>,
    /// Côté maximal de l'atlas en pixels (4096 par défaut).
    pub max_side: u32,
    /// Impose la hauteur de la cellule réduite (sinon le plus grand facteur qui tient).
    pub cell_height: Option<u32>,
    /// `Some(b)` : valeur de `mirror` ; `None` : vrai sauf pour huit directions.
    pub mirror: Option<bool>,
    /// Où se pose le pivot (pieds, point de contact au sol), en unités monde locales à l'entité :
    /// `(0, 0)` = l'origine de l'entité (comportement par défaut) ; `offset = (centre − pivot) × échelle + pivot_at`.
    pub pivot_at: (f32, f32),
    /// Écraser les fichiers et l'entrée déjà présents (jamais les autres entrées de `sprites.ron`).
    pub overwrite: bool,
    /// Tests seulement : simule une panne d'écriture au pas de remplacement donné (0 = image,
    /// 3 = `sprites.ron`) pour prouver que l'état d'avant est rétabli octet pour octet.
    #[doc(hidden)]
    pub fail_at_commit_step: Option<usize>,
}

impl SpriteImportOptions {
    pub fn new(scale: ScaleSpec) -> Self {
        Self {
            sprite_id: None,
            scale,
            states: Vec::new(),
            max_side: 4096,
            cell_height: None,
            mirror: None,
            overwrite: false,
            pivot_at: (0.0, 0.0),
            fail_at_commit_step: None,
        }
    }
}

/// Un clip placé dans l'atlas.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedClip {
    /// Nom du clip dans le manifeste.
    pub clip: String,
    /// Clés d'état qui jouent ce clip (`--state`).
    pub states: Vec<String>,
    /// Image de départ et de fin (inclusive) sur la première ligne du bloc, en indices d'atlas.
    pub start: usize,
    pub end: usize,
    /// Position du bloc dans l'atlas (colonne et ligne de la première case).
    pub block_column: u32,
    pub block_row: u32,
    pub frame_duration_ms: u64,
    pub looping: bool,
}

/// Résultat d'un import réussi : tout est écrit et `sprites.ron` à jour.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteImportResult {
    pub sprite_id: String,
    /// Table `SpriteSheet` mise à jour, relative à `assets/` (ex. `sprites/sprites.ron`).
    pub table: String,
    /// Fichiers écrits, relatifs à `assets/`.
    pub image: String,
    pub sheet: String,
    pub animation: String,
    /// Cellule source et cellule de l'atlas réduit, en pixels.
    pub source_cell: (u32, u32),
    pub cell: (u32, u32),
    /// Facteur de réduction (`cellule réduite / cellule source`, hauteur, ≤ 1).
    pub factor: f64,
    /// Grille de l'atlas (cases) et taille du PNG (pixels).
    pub columns: u32,
    pub rows: u32,
    pub atlas_size: (u32, u32),
    /// Taille de la grille de blocs : (colonnes, lignes).
    pub blocks: (u32, u32),
    /// `scale` écrit, décalage de pivot écrit (unités monde).
    pub scale: f32,
    pub offset: (f32, f32),
    /// Où le pivot (pieds) est posé, en unités monde locales (`--pivot-at`).
    pub pivot_at: (f32, f32),
    /// Hauteur de la silhouette mesurée sur l'`idle` : pixels source, pixels de l'atlas, unités monde
    /// (affichée, pour choisir `units_per_meter`).
    pub silhouette_source_px: u32,
    pub silhouette_atlas_px: f64,
    pub silhouette_units: f64,
    /// Clip de l'`idle` mesuré.
    pub silhouette_clip: String,
    pub clips: Vec<ImportedClip>,
    /// Avertissements non bloquants (état `Idle` absent, pivot ignoré…).
    pub warnings: Vec<String>,
}

/// Échec d'import : rien n'a été écrit ni modifié (voir l'en-tête du module).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpriteImportError {
    #[error("manifeste illisible ({path}) : {message}")]
    ManifestUnreadable { path: String, message: String },
    #[error("manifeste invalide ({path}) : {message}")]
    ManifestInvalid { path: String, message: String },
    #[error("option invalide : {message}")]
    InvalidOption { message: String },
    #[error("jeu invalide ({game_dir}) : {message}")]
    GameInvalid { game_dir: String, message: String },
    #[error("image incompatible (clip « {clip} », {path}) : {message}")]
    ImageMismatch {
        clip: String,
        path: String,
        message: String,
    },
    #[error("atlas impossible : {message}")]
    AtlasImpossible { message: String },
    #[error("conflit (sprites.ron ou assets) : {message}")]
    Conflict { message: String },
    #[error("écriture impossible ({path}) : {message}")]
    WriteFailed { path: String, message: String },
}

/// Directions du compas reconnues, dans l'ordre des lignes : celles de
/// `sim_core::sprite_directions::Compass`.
const COMPASS: [&str; 8] = ["E", "NE", "N", "NW", "W", "SW", "S", "SE"];

/// Alpha minimal (sur 255) d'un pixel compté dans la silhouette mesurée.
const SILHOUETTE_ALPHA: u8 = 32;

#[derive(Debug, serde::Deserialize)]
struct PipelineManifest {
    name: String,
    cell: Option<[u32; 2]>,
    pivot: Option<[f64; 2]>,
    ppm: Option<f64>,
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

fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+'
}

/// Assainit un nom en id : tout caractère hors `[A-Za-z0-9_+-]` devient `_`. Vide : `None`.
fn sanitize_id(raw: &str) -> Option<String> {
    let out: String = raw
        .chars()
        .map(|c| if is_id_char(c) { c } else { '_' })
        .collect();
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn fmt_f32(v: f32) -> String {
    format!("{v:?}")
}

fn fps_to_ms(fps: f64) -> Option<u64> {
    if !fps.is_finite() || fps <= 0.0 {
        return None;
    }
    Some((1000.0 / fps).round().clamp(1.0, u64::MAX as f64) as u64)
}

// ---------------------------------------------------------------------------------------------
// Disposition de l'atlas
// ---------------------------------------------------------------------------------------------

/// Disposition retenue : grille de blocs, cellule réduite et facteur.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Layout {
    blocks_x: u32,
    blocks_y: u32,
    cell_w: u32,
    cell_h: u32,
    factor: f64,
}

/// Plus grand facteur (≤ 1) qui fait tenir `block_cols × block_rows` cases par bloc, `n` blocs, sous
/// `max_side` ; à facteur égal, le moins de blocs, puis l'atlas le plus carré. `cell_height` impose la
/// hauteur de la cellule (le facteur en découle).
fn choose_layout(
    n: u32,
    block_cols: u32,
    block_rows: u32,
    src_cell: (u32, u32),
    max_side: u32,
    cell_height: Option<u32>,
) -> Result<Layout, SpriteImportError> {
    let (cw, ch) = (src_cell.0 as f64, src_cell.1 as f64);
    let mut best: Option<(Layout, (u32, u64))> = None;
    for blocks_x in 1..=n {
        let blocks_y = n.div_ceil(blocks_x);
        // Évite les grilles à blocs vides en trop : une ligne de blocs vide ne sert à rien.
        if (blocks_y - 1) * blocks_x >= n {
            continue;
        }
        let (cols, rows) = (blocks_x * block_cols, blocks_y * block_rows);
        let (cell_w, cell_h, factor) = match cell_height {
            Some(h) => {
                let f = h as f64 / ch;
                let w = ((cw * f).round() as u32).max(1);
                (w, h, f)
            }
            None => {
                let f = (max_side as f64 / (cols as f64 * cw))
                    .min(max_side as f64 / (rows as f64 * ch))
                    .min(1.0);
                let mut w = ((cw * f).floor() as u32).max(1);
                let mut h = ((ch * f).floor() as u32).max(1);
                while (cols as u64 * w as u64 > max_side as u64 && w > 1)
                    || (rows as u64 * h as u64 > max_side as u64 && h > 1)
                {
                    if cols as u64 * w as u64 > max_side as u64 {
                        w -= 1;
                    }
                    if rows as u64 * h as u64 > max_side as u64 {
                        h -= 1;
                    }
                }
                (w, h, h as f64 / ch)
            }
        };
        if cell_h > src_cell.1 || cell_w > src_cell.0 {
            continue;
        }
        if cols as u64 * cell_w as u64 > max_side as u64
            || rows as u64 * cell_h as u64 > max_side as u64
        {
            continue;
        }
        let layout = Layout {
            blocks_x,
            blocks_y,
            cell_w,
            cell_h,
            factor,
        };
        let width = cols as u64 * cell_w as u64;
        let height = rows as u64 * cell_h as u64;
        // Clé de comparaison : (blocs, déséquilibre de l'atlas) ; le facteur prime (plus grand = mieux).
        let key = (blocks_x * blocks_y, width.abs_diff(height));
        let better = match &best {
            None => true,
            Some((b, bkey)) => {
                if (layout.cell_h, layout.cell_w) != (b.cell_h, b.cell_w) {
                    (layout.cell_h, layout.cell_w) > (b.cell_h, b.cell_w)
                } else {
                    key < *bkey
                }
            }
        };
        if better {
            best = Some((layout, key));
        }
    }
    best.map(|(l, _)| l)
        .ok_or_else(|| SpriteImportError::AtlasImpossible {
            message: match cell_height {
                Some(h) => format!(
                "{n} clip(s) de {block_cols}×{block_rows} cases à une hauteur de cellule de {h} px \
                 (source {}×{}) ne tiennent dans aucun atlas de {max_side} px de côté, ou \
                 agrandiraient la source : réduisez --cell-height ou augmentez --max-side",
                src_cell.0, src_cell.1
            ),
                None => format!(
                "{n} clip(s) de {block_cols}×{block_rows} cases ne tiennent dans aucun atlas de \
                 {max_side} px de côté : augmentez --max-side"
            ),
            },
        })
}

// ---------------------------------------------------------------------------------------------
// Réduction : alpha prémultiplié, Lanczos3 séparable, ordre de sommation fixe
// ---------------------------------------------------------------------------------------------

fn lanczos3(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    if x.abs() >= 3.0 {
        return 0.0;
    }
    let px = std::f64::consts::PI * x;
    3.0 * px.sin() * (px / 3.0).sin() / (px * px)
}

/// Poids de rééchantillonnage d'une dimension `src` vers `dst` : par pixel de sortie, premier pixel
/// source et poids normalisés (filtre élargi du rapport de réduction).
fn axis_weights(src: usize, dst: usize) -> Vec<(usize, Vec<f64>)> {
    let ratio = src as f64 / dst as f64;
    let scale = ratio.max(1.0);
    let support = 3.0 * scale;
    (0..dst)
        .map(|x| {
            let center = (x as f64 + 0.5) * ratio;
            let lo = ((center - support).floor().max(0.0)) as usize;
            let hi = (((center + support).ceil()) as usize).min(src);
            let mut w: Vec<f64> = (lo..hi)
                .map(|i| lanczos3((i as f64 + 0.5 - center) / scale))
                .collect();
            let sum: f64 = w.iter().sum();
            if sum != 0.0 {
                for v in &mut w {
                    *v /= sum;
                }
            }
            (lo, w)
        })
        .collect()
}

/// Réduit une cellule RGBA8 `sw×sh` en `dw×dh` RGBA8, en alpha prémultiplié.
fn resize_cell(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    if sw == dw && sh == dh {
        return src.to_vec();
    }
    // Prémultiplie : 4 flottants par pixel.
    let mut premul = vec![0f32; sw * sh * 4];
    for (i, px) in src.chunks_exact(4).enumerate() {
        let a = px[3] as f32 / 255.0;
        premul[i * 4] = px[0] as f32 / 255.0 * a;
        premul[i * 4 + 1] = px[1] as f32 / 255.0 * a;
        premul[i * 4 + 2] = px[2] as f32 / 255.0 * a;
        premul[i * 4 + 3] = a;
    }
    let wx = axis_weights(sw, dw);
    let wy = axis_weights(sh, dh);
    // Horizontale : sh lignes × dw colonnes.
    let mut mid = vec![0f32; dw * sh * 4];
    for y in 0..sh {
        for (x, (lo, weights)) in wx.iter().enumerate() {
            let mut acc = [0f64; 4];
            for (k, w) in weights.iter().enumerate() {
                let p = &premul[(y * sw + lo + k) * 4..][..4];
                for c in 0..4 {
                    acc[c] += w * p[c] as f64;
                }
            }
            for c in 0..4 {
                mid[(y * dw + x) * 4 + c] = acc[c] as f32;
            }
        }
    }
    // Verticale : dh lignes × dw colonnes, puis dé-prémultiplication.
    let mut out = vec![0u8; dw * dh * 4];
    for (y, (lo, weights)) in wy.iter().enumerate() {
        for x in 0..dw {
            let mut acc = [0f64; 4];
            for (k, w) in weights.iter().enumerate() {
                let p = &mid[((lo + k) * dw + x) * 4..][..4];
                for c in 0..4 {
                    acc[c] += w * p[c] as f64;
                }
            }
            let a = acc[3].clamp(0.0, 1.0);
            let o = (y * dw + x) * 4;
            if a <= 0.0 {
                continue;
            }
            for c in 0..3 {
                let v = (acc[c].clamp(0.0, a) / a * 255.0).round();
                out[o + c] = v as u8;
            }
            out[o + 3] = (a * 255.0).round() as u8;
        }
    }
    out
}

/// Hauteur de la boîte alpha d'une cellule RGBA8 (lignes contenant un pixel d'alpha ≥ seuil), 0 si vide.
fn silhouette_height(cell: &RgbaImage) -> u32 {
    let (w, h) = cell.dimensions();
    let (mut min, mut max) = (u32::MAX, 0u32);
    for y in 0..h {
        let hit = (0..w).any(|x| cell.get_pixel(x, y).0[3] >= SILHOUETTE_ALPHA);
        if hit {
            min = min.min(y);
            max = y;
        }
    }
    if min == u32::MAX {
        0
    } else {
        max - min + 1
    }
}

fn encode_png(image: &RgbaImage) -> Result<Vec<u8>, String> {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::ImageEncoder;
    let mut bytes = Vec::new();
    PngEncoder::new_with_quality(&mut bytes, CompressionType::Best, FilterType::Adaptive)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

// ---------------------------------------------------------------------------------------------
// Table `sprites.ron` : insertion ou remplacement ciblé, commentaires conservés
// ---------------------------------------------------------------------------------------------

/// Chemin de la table `SpriteSheet` d'un jeu : la première déclaration `kind: "SpriteSheet"` des
/// `content_folders` de `assets/game.ron`, lue par le parseur du manifeste (`GameManifest`), donc
/// indépendante de l'ordre des champs, des commentaires et de l'UTF-8.
fn sprites_table_rel(game_ron: &str, path: &Path) -> Result<Option<String>, String> {
    let manifest = crate::manifest::GameManifest::parse(game_ron, path.to_path_buf())
        .map_err(|e| e.to_string())?;
    Ok(manifest
        .content_folders
        .into_iter()
        .find(|decl| decl.kind == "SpriteSheet")
        .map(|decl| decl.path))
}

/// Clé RON d'une table ou d'une carte : chaîne entre guillemets, échappée par la sérialisation RON
/// (une barre oblique inverse ou un guillemet ne produisent ni échappée invalide ni autre clé).
fn ron_key(key: &str) -> String {
    ron::to_string(key).unwrap_or_else(|_| format!("{key:?}"))
}

/// Refuse un texte RON qui ne se relit pas (jamais d'écriture d'un fichier que l'engine ne lirait pas).
fn verify_ron(text: &str, what: &str) -> Result<(), String> {
    ron::from_str::<ron::Value>(text)
        .map(|_| ())
        .map_err(|e| format!("{what} produit illisible, rien n'est écrit : {e}"))
}

/// Table relue après modification : une table de chaînes vers des entrées, avec l'entrée `id`.
fn verify_table(text: &str, id: &str) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct Table(BTreeMap<String, serde::de::IgnoredAny>);
    let table: Table = ron::from_str(text)
        .map_err(|e| format!("table produite illisible, rien n'est écrit : {e}"))?;
    if table.0.contains_key(id) {
        Ok(())
    } else {
        Err(format!(
            "entrée « {id} » absente de la table produite, rien n'est écrit"
        ))
    }
}

/// Entrée de premier niveau d'une table : clé lue, étendue de `"clé": (valeur)` et position juste après
/// la virgule qui la suit (s'il y en a une).
#[derive(Debug)]
struct TableEntry {
    key: String,
    start: usize,
    end: usize,
    after_comma: Option<usize>,
}

#[derive(Debug)]
struct TableScan {
    entries: Vec<TableEntry>,
    /// Position juste après le `{` de la table.
    open: usize,
}

/// Saute espaces et commentaires (`//` et `/* */`).
fn skip_trivia(b: &[u8], mut i: usize) -> usize {
    loop {
        match b.get(i) {
            Some(c) if c.is_ascii_whitespace() => i += 1,
            Some(b'/') if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            Some(b'/') if b.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(b.len());
            }
            _ => return i,
        }
    }
}

/// Chaîne RON ouvrant en `i` (`"`) : contenu décodé (échappées simples) et position après le `"` final.
fn parse_string(b: &[u8], i: usize) -> Result<(String, usize), String> {
    let mut out = Vec::new();
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'"' => return Ok((String::from_utf8_lossy(&out).into_owned(), j + 1)),
            b'\\' => {
                j += 1;
                match b.get(j) {
                    Some(b'n') => out.push(b'\n'),
                    Some(b't') => out.push(b'\t'),
                    Some(b'r') => out.push(b'\r'),
                    Some(c) => out.push(*c),
                    None => break,
                }
                j += 1;
            }
            c => {
                out.push(c);
                j += 1;
            }
        }
    }
    Err("chaîne non fermée".to_string())
}

/// Fin d'une valeur : position du `,` ou du `}` de niveau 0 qui la termine, chaînes et commentaires sautés.
fn skip_value(b: &[u8], mut i: usize) -> Result<usize, String> {
    let mut depth = 0usize;
    while i < b.len() {
        match b[i] {
            b'"' => i = parse_string(b, i)?.1,
            b'/' if matches!(b.get(i + 1), Some(b'/') | Some(b'*')) => i = skip_trivia(b, i),
            b'(' | b'[' | b'{' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' | b'}' if depth == 0 => {
                return if b[i] == b'}' {
                    Ok(i)
                } else {
                    Err("parenthèse fermante inattendue".to_string())
                }
            }
            b')' | b']' | b'}' => {
                depth -= 1;
                i += 1;
            }
            b',' if depth == 0 => return Ok(i),
            _ => i += 1,
        }
    }
    Err("valeur non terminée".to_string())
}

/// Entrées de premier niveau de `({ "id": (…), … })`, sans jamais regarder dans un commentaire, une
/// chaîne ou une valeur imbriquée.
fn scan_table(text: &str) -> Result<TableScan, String> {
    let b = text.as_bytes();
    let mut i = skip_trivia(b, 0);
    if b.get(i) != Some(&b'(') {
        return Err(
            "table qui ne commence pas par `({` : refuse de modifier un fichier inattendu".into(),
        );
    }
    i = skip_trivia(b, i + 1);
    if b.get(i) != Some(&b'{') {
        return Err(
            "table qui ne commence pas par `({` : refuse de modifier un fichier inattendu".into(),
        );
    }
    let open = i + 1;
    i = open;
    let mut entries = Vec::new();
    loop {
        i = skip_trivia(b, i);
        match b.get(i) {
            Some(b'}') => break,
            Some(b'"') => {}
            _ => return Err("entrée de table inattendue (clé entre guillemets attendue)".into()),
        }
        let start = i;
        let (key, after_key) = parse_string(b, i)?;
        i = skip_trivia(b, after_key);
        if b.get(i) != Some(&b':') {
            return Err(format!("« {key} » : `:` attendu"));
        }
        i = skip_trivia(b, i + 1);
        let terminator = skip_value(b, i)?;
        let mut end = terminator;
        while end > i && b[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        let after_comma = (b[terminator] == b',').then_some(terminator + 1);
        entries.push(TableEntry {
            key,
            start,
            end,
            after_comma,
        });
        i = after_comma.unwrap_or(terminator);
    }
    Ok(TableScan { entries, open })
}

/// Insère ou remplace l'entrée `"id": body` d'une table RON, commentaires et autres entrées préservés.
/// `body` inclut ses parenthèses.
fn upsert_table_entry(table: &str, entry_id: &str, body: &str) -> Result<String, String> {
    let scan = scan_table(table)?;
    let new = format!("{}: {body}", ron_key(entry_id));
    let mut out = table.to_string();
    if let Some(e) = scan.entries.iter().find(|e| e.key == entry_id) {
        out.replace_range(e.start..e.end, &new);
    } else {
        match scan.entries.last() {
            None => out.insert_str(scan.open, &format!("\n    {new},")),
            Some(last) => match last.after_comma {
                Some(at) => out.insert_str(at, &format!("\n    {new},")),
                None => out.insert_str(last.end, &format!(",\n    {new},")),
            },
        }
    }
    Ok(out)
}

/// Écriture transactionnelle : tout est d'abord écrit dans des fichiers temporaires du même dossier,
/// puis renommé ; l'ancien fichier est mis de côté avant d'être remplacé. En cas d'échec à n'importe
/// quel point, les anciens fichiers reviennent octet pour octet, les nouveaux et les dossiers créés
/// disparaissent. `fail_at` (tests) simule une panne d'E/S au pas de renommage donné.
fn commit_files(
    targets: &[(PathBuf, Vec<u8>)],
    fail_at: Option<usize>,
) -> Result<(), SpriteImportError> {
    let write_failed = |path: &Path, message: String| SpriteImportError::WriteFailed {
        path: path.display().to_string(),
        message,
    };
    let sidecar = |path: &Path, suffix: &str| {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("fichier");
        path.with_file_name(format!(".{name}.{}.{suffix}", std::process::id()))
    };
    let mut new_dirs: Vec<PathBuf> = Vec::new();
    let mut temps: Vec<PathBuf> = Vec::new();
    let abort = |new_dirs: &[PathBuf], temps: &[PathBuf]| {
        for t in temps {
            let _ = std::fs::remove_file(t);
        }
        for d in new_dirs {
            let _ = std::fs::remove_dir_all(d);
        }
    };

    for (path, _) in targets {
        if let Some(parent) = path.parent() {
            // Premier ancêtre absent : le seul dossier à retirer si l'import échoue.
            let missing = parent
                .ancestors()
                .take_while(|a| !a.as_os_str().is_empty() && !a.exists())
                .last()
                .map(Path::to_path_buf);
            if let Err(e) = std::fs::create_dir_all(parent) {
                abort(&new_dirs, &temps);
                return Err(write_failed(path, format!("dossier impossible : {e}")));
            }
            if let Some(m) = missing {
                new_dirs.push(m);
            }
        }
    }
    let mut staged: Vec<PathBuf> = Vec::new();
    for (path, bytes) in targets {
        let tmp = sidecar(path, "x4b-tmp");
        temps.push(tmp.clone());
        if let Err(e) = std::fs::write(&tmp, bytes) {
            abort(&new_dirs, &temps);
            return Err(write_failed(path, format!("écriture impossible : {e}")));
        }
        staged.push(tmp);
    }

    // Remplacement : (destination, sauvegarde de l'ancien fichier s'il existait).
    let mut done: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
    let mut failure: Option<(PathBuf, String)> = None;
    for (step, ((path, _), tmp)) in targets.iter().zip(&staged).enumerate() {
        if fail_at == Some(step) {
            failure = Some((path.clone(), "panne simulée".to_string()));
            break;
        }
        let backup = path.exists().then(|| sidecar(path, "x4b-bak"));
        if let Some(bak) = &backup {
            if let Err(e) = std::fs::rename(path, bak) {
                failure = Some((
                    path.clone(),
                    format!("sauvegarde de l'ancien fichier : {e}"),
                ));
                break;
            }
        }
        done.push((path.clone(), backup));
        if let Err(e) = std::fs::rename(tmp, path) {
            failure = Some((path.clone(), format!("remplacement : {e}")));
            break;
        }
    }
    if let Some((path, message)) = failure {
        for (dest, backup) in done.iter().rev() {
            let _ = std::fs::remove_file(dest);
            if let Some(bak) = backup {
                let _ = std::fs::rename(bak, dest);
            }
        }
        abort(&new_dirs, &temps);
        return Err(write_failed(
            &path,
            format!("{message} ; état d'avant rétabli"),
        ));
    }
    for (_, backup) in &done {
        if let Some(bak) = backup {
            let _ = std::fs::remove_file(bak);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------------------------

struct ValidClip {
    name: String,
    columns: u32,
    rows: u32,
    ms: u64,
    looping: bool,
    path: PathBuf,
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
    let bad_option = |message: String| SpriteImportError::InvalidOption { message };
    let game_invalid = |message: String| SpriteImportError::GameInvalid {
        game_dir: game_display.clone(),
        message,
    };
    let mut warnings: Vec<String> = Vec::new();

    match options.scale {
        ScaleSpec::UnitsPerMeter(u) if !u.is_finite() || u <= 0.0 => {
            return Err(bad_option(format!(
                "--units-per-meter = {u} : nombre fini > 0"
            )))
        }
        ScaleSpec::Scale(s) if !s.is_finite() || s <= 0.0 => {
            return Err(bad_option(format!("--scale = {s} : nombre fini > 0")))
        }
        _ => {}
    }
    if !options.pivot_at.0.is_finite() || !options.pivot_at.1.is_finite() {
        return Err(bad_option(format!(
            "--pivot-at = {:?} : deux nombres finis",
            options.pivot_at
        )));
    }
    if options.max_side == 0 {
        return Err(bad_option("--max-side = 0 : nombre > 0".to_string()));
    }
    if options.cell_height == Some(0) {
        return Err(bad_option("--cell-height = 0 : nombre > 0".to_string()));
    }

    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        unreadable("chemin sans dossier parent : donnez un fichier `*.sprite.json`".to_string())
    })?;
    let raw = std::fs::read_to_string(manifest_path)
        .map_err(|e| unreadable(format!("lecture impossible : {e}")))?;
    let manifest: PipelineManifest =
        serde_json::from_str(&raw).map_err(|e| invalid(format!("JSON illisible : {e}")))?;

    if manifest.name.trim().is_empty() {
        return Err(invalid(
            "champ `name` vide : un sprite sans nom".to_string(),
        ));
    }
    let sprite_id = match &options.sprite_id {
        Some(id) => sanitize_id(id).ok_or_else(|| {
            bad_option(format!(
                "--id « {id} » vide après assainissement ([A-Za-z0-9_+-], le reste devient `_`)"
            ))
        })?,
        None => sanitize_id(&manifest.name).ok_or_else(|| {
            invalid(format!(
                "nom « {} » vide après assainissement : donnez --id ([A-Za-z0-9_+-])",
                manifest.name
            ))
        })?,
    };
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
    let animations = manifest.animations.ok_or_else(|| {
        invalid("champ `animations` absent : rien à importer (au moins un clip)".to_string())
    })?;
    if animations.is_empty() {
        return Err(invalid(
            "`animations` est vide : rien à importer (au moins un clip)".to_string(),
        ));
    }

    // Clips : grille, rythme, taille réelle du PNG (en-tête seul) avant toute écriture.
    let mut clips: Vec<ValidClip> = Vec::new();
    for (clip_name, clip) in &animations {
        if clip_name.trim().is_empty() || clip_name.contains('"') || clip_name.contains('\n') {
            return Err(invalid(format!(
                "clip « {clip_name} » : nom vide ou avec guillemet/retour : renommez-le"
            )));
        }
        if clip.columns == 0 || clip.rows == 0 {
            return Err(invalid(format!(
                "clip « {clip_name} » : columns = {}, rows = {} : strictement positifs",
                clip.columns, clip.rows
            )));
        }
        if let Some(dirs) = &directions {
            if clip.rows as usize != dirs.len() {
                return Err(invalid(format!(
                    "clip « {clip_name} » : rows = {} pour {} direction(s) : une ligne par direction \
                     (les planches multi-blocs ne s'importent pas, découpez-les)",
                    clip.rows,
                    dirs.len()
                )));
            }
        }
        let ms = fps_to_ms(clip.fps).ok_or_else(|| {
            invalid(format!(
                "clip « {clip_name} » : fps = {} : nombre fini > 0",
                clip.fps
            ))
        })?;
        let path = manifest_dir.join(&clip.sheet);
        let mismatch = |message: String| SpriteImportError::ImageMismatch {
            clip: clip_name.clone(),
            path: path.display().to_string(),
            message,
        };
        let (w, h) = image::ImageReader::open(&path)
            .map_err(|e| mismatch(format!("PNG illisible à côté du manifeste : {e}")))?
            .with_guessed_format()
            .map_err(|e| mismatch(format!("PNG illisible : {e}")))?
            .into_dimensions()
            .map_err(|e| mismatch(format!("pas un PNG lisible : {e}")))?;
        if w != clip.columns * cell_w || h != clip.rows * cell_h {
            return Err(mismatch(format!(
                "PNG {w}×{h} px, grille {}×{} cases de {cell_w}×{cell_h} px = {}×{} px attendus",
                clip.columns,
                clip.rows,
                clip.columns * cell_w,
                clip.rows * cell_h,
            )));
        }
        clips.push(ValidClip {
            name: clip_name.clone(),
            columns: clip.columns,
            rows: clip.rows,
            ms,
            looping: clip.loop_clip,
            path,
        });
    }
    // `animations` est un BTreeMap : ordre alphabétique, donc blocs et sorties déterministes.

    let block_rows = clips[0].rows;
    if clips.iter().any(|c| c.rows != block_rows) {
        return Err(invalid(
            "les clips n'ont pas tous le même nombre de lignes : un bloc a une hauteur unique"
                .to_string(),
        ));
    }
    let block_cols = clips.iter().map(|c| c.columns).max().unwrap_or(1);

    // États demandés : clé → clip.
    let clip_names: BTreeSet<&str> = clips.iter().map(|c| c.name.as_str()).collect();
    let mut state_map: BTreeMap<String, String> = BTreeMap::new();
    for (state, clip) in &options.states {
        if state.trim().is_empty() || state.contains('"') || state.contains('\n') {
            return Err(bad_option(format!(
                "--state « {state}={clip} » : nom d'état vide ou avec guillemet"
            )));
        }
        if !clip_names.contains(clip.as_str()) {
            return Err(bad_option(format!(
                "--state {state}={clip} : clip « {clip} » absent du manifeste (clips : {})",
                clips
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        if state_map.insert(state.clone(), clip.clone()).is_some() {
            return Err(bad_option(format!("--state {state} donné deux fois")));
        }
        if clip_names.contains(state.as_str()) && state != clip {
            return Err(bad_option(format!(
                "--state {state}={clip} : « {state} » est aussi le nom d'un clip, qui jouerait deux \
                 animations sous la même clé"
            )));
        }
    }
    if !state_map.contains_key("Idle") {
        warnings.push(
            "aucun état `Idle` (--state Idle=<clip>) : le runtime retombe sur la première image de \
             la carte, sans animation, pour tout état non défini"
                .to_string(),
        );
    }

    // Atlas : disposition, puis composition en mémoire.
    let layout = choose_layout(
        clips.len() as u32,
        block_cols,
        block_rows,
        (cell_w, cell_h),
        options.max_side,
        options.cell_height,
    )?;
    let atlas_columns = layout.blocks_x * block_cols;
    let atlas_rows = layout.blocks_y * block_rows;
    let atlas_w = atlas_columns * layout.cell_w;
    let atlas_h = atlas_rows * layout.cell_h;
    let mut atlas: RgbaImage = ImageBuffer::from_pixel(atlas_w, atlas_h, Rgba([0, 0, 0, 0]));

    // Clip de l'`idle` mesuré : l'état `Idle`, sinon le clip nommé `idle`, sinon le premier.
    let idle_clip = state_map
        .get("Idle")
        .cloned()
        .or_else(|| clip_names.contains("idle").then(|| "idle".to_string()))
        .unwrap_or_else(|| clips[0].name.clone());
    let mut silhouette_source = 0u32;
    let mut placed: Vec<ImportedClip> = Vec::new();
    for (index, clip) in clips.iter().enumerate() {
        let block_column = (index as u32 % layout.blocks_x) * block_cols;
        let block_row = (index as u32 / layout.blocks_x) * block_rows;
        let mismatch = |message: String| SpriteImportError::ImageMismatch {
            clip: clip.name.clone(),
            path: clip.path.display().to_string(),
            message,
        };
        let sheet = image::open(&clip.path)
            .map_err(|e| mismatch(format!("décodage impossible : {e}")))?
            .to_rgba8();
        if sheet.dimensions() != (clip.columns * cell_w, clip.rows * cell_h) {
            return Err(mismatch("dimensions changées pendant l'import".to_string()));
        }
        for row in 0..clip.rows {
            for col in 0..clip.columns {
                let mut cell = Vec::with_capacity((cell_w * cell_h * 4) as usize);
                for y in 0..cell_h {
                    let from = (((row * cell_h + y) * sheet.width()) + col * cell_w) as usize * 4;
                    cell.extend_from_slice(&sheet.as_raw()[from..from + cell_w as usize * 4]);
                }
                if clip.name == idle_clip {
                    if let Some(img) = RgbaImage::from_raw(cell_w, cell_h, cell.clone()) {
                        silhouette_source = silhouette_source.max(silhouette_height(&img));
                    }
                }
                let reduced = resize_cell(
                    &cell,
                    cell_w as usize,
                    cell_h as usize,
                    layout.cell_w as usize,
                    layout.cell_h as usize,
                );
                let x0 = (block_column + col) * layout.cell_w;
                let y0 = (block_row + row) * layout.cell_h;
                for y in 0..layout.cell_h {
                    for x in 0..layout.cell_w {
                        let o = ((y * layout.cell_w + x) * 4) as usize;
                        atlas.put_pixel(
                            x0 + x,
                            y0 + y,
                            Rgba([reduced[o], reduced[o + 1], reduced[o + 2], reduced[o + 3]]),
                        );
                    }
                }
            }
        }
        let start = (block_row * atlas_columns + block_column) as usize;
        placed.push(ImportedClip {
            clip: clip.name.clone(),
            states: state_map
                .iter()
                .filter(|(_, c)| **c == clip.name)
                .map(|(s, _)| s.clone())
                .collect(),
            start,
            end: start + clip.columns as usize - 1,
            block_column,
            block_row,
            frame_duration_ms: clip.ms,
            looping: clip.looping,
        });
    }
    let png = encode_png(&atlas).map_err(|message| SpriteImportError::WriteFailed {
        path: "<atlas>".to_string(),
        message: format!("encodage PNG impossible : {message}"),
    })?;

    // Échelle et pivot.
    let units_per_source_px: f64 = match options.scale {
        ScaleSpec::UnitsPerMeter(u) => {
            let ppm = manifest
                .ppm
                .filter(|p| p.is_finite() && *p > 0.0)
                .ok_or_else(|| {
                    invalid(
                    "champ `ppm` absent : --units-per-meter en a besoin (donnez --scale à la place)"
                        .to_string(),
                )
                })?;
            u as f64 / ppm
        }
        ScaleSpec::Scale(s) => s as f64 * layout.factor,
    };
    let scale = match options.scale {
        ScaleSpec::Scale(s) => s,
        ScaleSpec::UnitsPerMeter(_) => (units_per_source_px / layout.factor) as f32,
    };
    // Pivot en pixels source (origine en haut à gauche) → décalage depuis le centre de la case,
    // en unités monde (y vers le haut) : le pivot tombe sur `pivot_at` (l'origine du personnage par défaut).
    let (pivot_x, pivot_y) = options.pivot_at;
    let offset = match manifest.pivot {
        Some([px, py]) if px.is_finite() && py.is_finite() => {
            let dx = px - cell_w as f64 / 2.0;
            let dy_down = py - cell_h as f64 / 2.0;
            (
                (-dx * units_per_source_px) as f32 + pivot_x,
                (dy_down * units_per_source_px) as f32 + pivot_y,
            )
        }
        _ => {
            warnings.push(
                "pas de `pivot` dans le manifeste : sprite centré (offset = --pivot-at)".into(),
            );
            (pivot_x, pivot_y)
        }
    };
    let mirror = options.mirror.unwrap_or(match &directions {
        Some(dirs) => dirs.len() != 8,
        None => true,
    });

    // Jeu : `assets/game.ron` et sa table `SpriteSheet`.
    let assets_dir = game_dir.join("assets");
    let game_ron_path = assets_dir.join("game.ron");
    let game_ron = std::fs::read_to_string(&game_ron_path).map_err(|e| {
        game_invalid(format!(
            "assets/game.ron illisible (dossier du jeu = celui qui contient assets/) : {e}"
        ))
    })?;
    let table_rel = sprites_table_rel(&game_ron, &game_ron_path)
        .map_err(|message| game_invalid(format!("assets/game.ron illisible : {message}")))?
        .ok_or_else(|| {
            game_invalid(
                "aucun `(path: …, kind: \"SpriteSheet\")` dans content_folders : déclarez la table \
                 des sprites avant d'importer"
                    .to_string(),
            )
        })?;
    let table_path = assets_dir.join(&table_rel);
    let table_exists = table_path.is_file();
    let table_text = match std::fs::read_to_string(&table_path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "({\n})\n".to_string(),
        Err(e) => {
            return Err(SpriteImportError::WriteFailed {
                path: table_path.display().to_string(),
                message: format!("table illisible : {e}"),
            })
        }
    };

    let dir_rel = format!("sprites/{sprite_id}");
    let image_rel = format!("{dir_rel}/{sprite_id}.png");
    let sheet_rel = format!("{dir_rel}/{sprite_id}_sheet.ron");
    let anim_rel = format!("{dir_rel}/{sprite_id}_animation.ron");
    let files = [&image_rel, &sheet_rel, &anim_rel];
    // Conflits d'abord : aucune écriture n'a encore eu lieu.
    if !options.overwrite {
        let scan = scan_table(&table_text)
            .map_err(|message| game_invalid(format!("{table_rel} : {message}")))?;
        if scan.entries.iter().any(|e| e.key == sprite_id) {
            return Err(SpriteImportError::Conflict {
                message: format!(
                    "entrée « {sprite_id} » déjà dans {table_rel} : --overwrite pour la remplacer"
                ),
            });
        }
        for rel in files {
            if assets_dir.join(rel).exists() {
                return Err(SpriteImportError::Conflict {
                    message: format!("assets/{rel} existe déjà : --overwrite pour le remplacer"),
                });
            }
        }
    }

    // Textes RON.
    let dirs_ron = match &directions {
        None => String::new(),
        Some(dirs) => format!(
            "\n    directions: Some((\n        rows: [{}],\n        mirror: {mirror},\n    )),",
            dirs.join(", ")
        ),
    };
    let sheet_ron = format!(
        "// Généré par `alacod import-sprite` (X-4b, docs/conventions.md §91) : atlas réduit de {atlas_w}×{atlas_h} px,\n\
         // {blocks_x}×{blocks_y} blocs de {block_cols}×{block_rows} cases de {cw}×{ch} px (facteur {factor:.4}).\n\
         // Le pivot du manifeste est aux pieds (point de contact au sol) ; il est posé à ({px}, {py}) unités locales de l'entité (--pivot-at) : voir offset_y.\n\
         (\n    path: \"{image_rel}\",\n    tile_size: ({cw}, {ch}),\n    columns: {atlas_columns},\n    rows: {atlas_rows},\n    animated: true,\n    name: \"body\",\n    scale: {scale},\n    anchor: Center,\n    offset_x: {ox},\n    offset_y: {oy},\n    offset_z: 1.0,{dirs_ron}\n)\n",
        blocks_x = layout.blocks_x,
        blocks_y = layout.blocks_y,
        cw = layout.cell_w,
        ch = layout.cell_h,
        factor = layout.factor,
        scale = fmt_f32(scale),
        ox = fmt_f32(offset.0),
        oy = fmt_f32(offset.1),
        px = fmt_f32(options.pivot_at.0),
        py = fmt_f32(options.pivot_at.1),
    );
    // Animations : états puis clips, ordre alphabétique (BTreeMap).
    let mut indices: BTreeMap<String, (usize, usize, u64, bool)> = BTreeMap::new();
    for c in &placed {
        indices.insert(
            c.clip.clone(),
            (c.start, c.end, c.frame_duration_ms, c.looping),
        );
        for state in &c.states {
            indices.insert(
                state.clone(),
                (c.start, c.end, c.frame_duration_ms, c.looping),
            );
        }
    }
    let global_ms = state_map
        .get("Idle")
        .and_then(|clip| placed.iter().find(|c| &c.clip == clip))
        .map_or(placed[0].frame_duration_ms, |c| c.frame_duration_ms);
    let mut anim_ron = format!(
        "// Généré par `alacod import-sprite` (X-4b) : indices d'atlas de la première ligne de direction de chaque bloc ;\n\
         // le rendu y ajoute `ligne de direction × columns`. `clips` : durée d'image (ms) et boucle de chaque clip.\n\
         (\n    frame_duration: {global_ms},\n    columns: Some({atlas_columns}),\n    animations: {{\n"
    );
    for (key, (start, end, _, _)) in &indices {
        anim_ron.push_str(&format!(
            "        {}: (start: {start}, end: {end}),\n",
            ron_key(key)
        ));
    }
    anim_ron.push_str("    },\n    clips: {\n");
    for (key, (_, _, ms, looping)) in &indices {
        anim_ron.push_str(&format!(
            "        {}: (frame_duration: Some({ms}), looping: Some({looping})),\n",
            ron_key(key)
        ));
    }
    anim_ron.push_str("    },\n)\n");

    let entry_body = format!(
        "(\n        animation: \"{anim_rel}\",\n        layers: {{ \"body\": \"{sheet_rel}\" }},\n    )"
    );
    let new_table = upsert_table_entry(&table_text, &sprite_id, &entry_body)
        .map_err(|message| game_invalid(format!("{table_rel} : {message}")))?;
    let table_text_out = if table_exists {
        new_table
    } else {
        format!(
            "// Table des feuilles de sprites : une entrée par sprite importé ou écrit à la main.\n{new_table}"
        )
    };
    // Tout se relit avant la moindre écriture : un texte que l'engine ne lirait pas est refusé.
    let verify = |r: Result<(), String>| r.map_err(|message| game_invalid(message));
    verify(verify_ron(&sheet_ron, "feuille"))?;
    verify(verify_ron(&anim_ron, "animation"))?;
    verify(verify_table(&table_text_out, &sprite_id).map_err(|m| format!("{table_rel} : {m}")))?;

    // Écriture transactionnelle : `sprites.ron` en dernier, tout ou rien.
    commit_files(
        &[
            (assets_dir.join(&image_rel), png),
            (assets_dir.join(&sheet_rel), sheet_ron.into_bytes()),
            (assets_dir.join(&anim_rel), anim_ron.into_bytes()),
            (table_path.clone(), table_text_out.into_bytes()),
        ],
        options.fail_at_commit_step,
    )?;

    let silhouette_atlas = silhouette_source as f64 * layout.factor;
    Ok(SpriteImportResult {
        sprite_id,
        table: table_rel,
        image: image_rel,
        sheet: sheet_rel,
        animation: anim_rel,
        source_cell: (cell_w, cell_h),
        cell: (layout.cell_w, layout.cell_h),
        factor: layout.factor,
        columns: atlas_columns,
        rows: atlas_rows,
        atlas_size: (atlas_w, atlas_h),
        blocks: (layout.blocks_x, layout.blocks_y),
        scale,
        offset,
        pivot_at: options.pivot_at,
        silhouette_source_px: silhouette_source,
        silhouette_atlas_px: silhouette_atlas,
        silhouette_units: silhouette_source as f64 * units_per_source_px,
        silhouette_clip: idle_clip,
        clips: placed,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_to_ms_rounds_and_rejects() {
        assert_eq!(fps_to_ms(12.0), Some(83));
        assert_eq!(fps_to_ms(4.0), Some(250));
        assert_eq!(fps_to_ms(0.0), None);
        assert_eq!(fps_to_ms(-3.0), None);
        assert_eq!(fps_to_ms(f64::NAN), None);
    }

    #[test]
    fn sanitize_id_replaces_and_trims() {
        assert_eq!(
            sanitize_id("marche avant!").as_deref(),
            Some("marche_avant")
        );
        assert_eq!(sanitize_id("???"), None);
        assert_eq!(sanitize_id("idle2").as_deref(), Some("idle2"));
    }

    #[test]
    fn upsert_inserts_into_empty_and_keeps_comments() {
        let out = upsert_table_entry("// tête\n({\n})\n", "hero", "(animation: \"a\")").unwrap();
        assert!(out.contains("// tête"), "{out}");
        assert!(out.contains("\"hero\": (animation: \"a\")"), "{out}");
    }

    #[test]
    fn upsert_appends_after_a_trailing_comma_and_replaces_only_its_entry() {
        let table =
            "({\n    \"keep\": (animation: \"k\"),\n    \"hero\": (animation: \"old\"),\n})";
        let added = upsert_table_entry(table, "new", "(animation: \"a\")").unwrap();
        assert!(!added.contains(",,"), "{added}");
        assert!(
            added.contains("\"keep\"") && added.contains("\"new\""),
            "{added}"
        );
        let replaced = upsert_table_entry(table, "hero", "(animation: \"new\")").unwrap();
        assert!(
            replaced.contains("\"keep\": (animation: \"k\")"),
            "{replaced}"
        );
        assert!(!replaced.contains("old"), "{replaced}");
    }

    #[test]
    fn layout_takes_the_largest_factor_and_never_enlarges() {
        // Dix blocs de 8×8 cases de 376×372 sous 4096 : environ 128×126.
        let l = choose_layout(10, 8, 8, (376, 372), 4096, None).unwrap();
        assert!(l.blocks_x * l.blocks_y >= 10);
        assert!(l.blocks_x * 8 * l.cell_w <= 4096 && l.blocks_y * 8 * l.cell_h <= 4096);
        assert!(
            (127..=130).contains(&l.cell_w) && (125..=128).contains(&l.cell_h),
            "{l:?}"
        );
        // Petite source : aucune réduction.
        let small = choose_layout(2, 4, 8, (8, 8), 4096, None).unwrap();
        assert_eq!((small.cell_w, small.cell_h), (8, 8));
        assert_eq!(small.factor, 1.0);
        // Impossible : trop de clips pour 16 px.
        assert!(choose_layout(10, 8, 8, (376, 372), 16, None).is_err());
        // Hauteur imposée plus grande que la source : refusée.
        assert!(choose_layout(1, 8, 8, (16, 16), 4096, Some(32)).is_err());
    }

    #[test]
    fn resize_keeps_color_at_transparent_edges() {
        // Moitié gauche rouge opaque, moitié droite transparente (RGB noir) : aucun halo sombre.
        let (w, h) = (8usize, 8usize);
        let mut src = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w / 2 {
                src[(y * w + x) * 4..][..4].copy_from_slice(&[255, 0, 0, 255]);
            }
        }
        let out = resize_cell(&src, w, h, 4, 4);
        for px in out.chunks_exact(4) {
            if px[3] > 0 {
                assert!(px[0] >= 250 && px[1] <= 5, "halo : {px:?}");
            }
        }
        assert_eq!(resize_cell(&src, w, h, w, h), src, "même taille : identité");
    }
}
