//! `alacod new-arena` : l'arène vide du bac à sable (`docs/conventions.md` §90).
//!
//! Une carte LDtk à **un seul niveau** : une couronne de murs (couche IntGrid `Walls`, avec ses tuiles
//! d'auto-règles pour qu'ils se voient), un à quatre `PlayerSpawn`, **rien d'autre** (ni fenêtre, ni porte,
//! ni arme murale, ni soda, ni `ZombieSpawn`, ni `CharacterSpawn`).
//!
//! # D'où viennent les définitions
//!
//! Jamais d'un gabarit de l'engine : `defs` (tilesets, couches, définitions d'entités) est **recopié d'une
//! carte du jeu** (le « donneur »), sinon les images ne se résolvent pas (tileset, chemins) et les entités
//! ne correspondent pas aux définitions du jeu. Le donneur est, dans l'ordre : `--from`, l'arène du bac à
//! sable du manifeste (`sandbox.arena`, donc l'arène vide elle-même à la deuxième exécution), la carte de
//! `generate_template`, la première carte du registre. Les chemins de tileset (`relPath`, relatifs au
//! fichier `.ldtk`) sont recalculés pour le dossier d'écriture.
//!
//! # Tuiles des murs
//!
//! Les cartes de l'engine portent leurs `autoLayerTiles` (LDtk les recalcule à l'enregistrement) ; un
//! fichier écrit sans elles n'afficherait aucun mur. [`auto_layer_tiles`] rejoue les règles de la couche
//! (groupes et règles dans l'ordre, `breakOnMatch`, retournements X/Y, hors-limites sans correspondance
//! quand `outOfBoundsValue` est nul) : la preuve est un test qui recalcule les tuiles des niveaux de
//! `games/zombies/assets/exemples/test_map.ldtk` et retrouve celles de LDtk, dans le même ordre.
//! Une règle active qu'on ne sait pas jouer (`tileMode` autre que `Single`, damier, bruit de Perlin) est
//! une **erreur**, jamais une tuile fausse.
//!
//! Le fichier est déterministe (identifiants dérivés du nom, clés triées) : relancer la commande ne
//! change pas un octet.

use serde_json::{json, Map, Value};
use std::path::{Component, Path, PathBuf};

use crate::manifest::GameManifest;
use crate::registry::{map_id_from_path, Registry};

/// Nom par défaut (id de carte = nom de fichier sans extension, unique dans le jeu).
pub const DEFAULT_ARENA_NAME: &str = "arene_vide";
/// Taille par défaut en cases : 416 × 304 px, celle des gabarits de l'engine (le joueur 0 apparaît alors en
/// (424, 696), `docs/conventions.md` §86).
pub const DEFAULT_WIDTH_TILES: u32 = 26;
pub const DEFAULT_HEIGHT_TILES: u32 = 19;
/// Bornes de `--size` : assez grand pour les quatre points de départ, assez petit pour un niveau LDtk sain.
pub const MIN_WIDTH_TILES: u32 = 16;
pub const MIN_HEIGHT_TILES: u32 = 12;
pub const MAX_TILES: u32 = 200;

/// Points de départ des joueurs 0 à 3 dans la salle par défaut (pixels du niveau) : ceux des gabarits.
const DEFAULT_SPAWNS: [(i64, i64); 4] = [(160, 80), (224, 80), (160, 112), (224, 112)];

/// Identifiant LDtk de la couche des murs (`map_ldtk::game::collider`, `map::generation`).
const WALLS_LAYER: &str = "Walls";
const ENTITIES_LAYER: &str = "Entities";
const PLAYER_SPAWN: &str = "PlayerSpawn";
const LEVEL_IDENTIFIER: &str = "Arena";
/// Valeur LDtk d'un motif « n'importe quelle valeur non nulle ».
const PATTERN_ANY: i64 = 1_000_001;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArenaSpec {
    /// Id de la carte : nom du fichier sans `.ldtk`.
    pub name: String,
    pub width_tiles: u32,
    pub height_tiles: u32,
    /// Nombre de `PlayerSpawn` (1 à 4, index 0 à `players - 1`).
    pub players: u32,
}

impl Default for ArenaSpec {
    fn default() -> Self {
        ArenaSpec {
            name: DEFAULT_ARENA_NAME.to_string(),
            width_tiles: DEFAULT_WIDTH_TILES,
            height_tiles: DEFAULT_HEIGHT_TILES,
            players: 4,
        }
    }
}

impl ArenaSpec {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty()
            || !self
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(format!(
                "nom d'arène « {} » invalide (lettres, chiffres, `_` et `-`)",
                self.name
            ));
        }
        if !(MIN_WIDTH_TILES..=MAX_TILES).contains(&self.width_tiles)
            || !(MIN_HEIGHT_TILES..=MAX_TILES).contains(&self.height_tiles)
        {
            return Err(format!(
                "taille {}x{} hors bornes ({MIN_WIDTH_TILES}x{MIN_HEIGHT_TILES} à {MAX_TILES}x{MAX_TILES} cases)",
                self.width_tiles, self.height_tiles
            ));
        }
        if !(1..=4).contains(&self.players) {
            return Err(format!(
                "{} joueurs : entre 1 et 4 points de départ",
                self.players
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------
// Identifiants déterministes
// ---------------------------------------------------------------------------------------

/// UUID (forme LDtk) dérivé d'un texte : FNV-1a 64 bits, deux graines.
fn derived_iid(text: &str) -> String {
    fn fnv(seed: u64, text: &str) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64 ^ seed;
        for byte in text.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }
    let a = fnv(0, text);
    let b = fnv(0x9e37_79b9_7f4a_7c15, text);
    format!(
        "{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}",
        (a >> 32) as u32,
        (a >> 16) as u16,
        a as u16 & 0x0fff,
        (b >> 48) as u16 & 0x0fff,
        b & 0x0000_ffff_ffff_ffff
    )
}

// ---------------------------------------------------------------------------------------
// Chemins
// ---------------------------------------------------------------------------------------

/// Chemin sans `.` ni `..` (lexical, sans toucher au disque).
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Chemin de `target` relatif à `base` (deux chemins de même racine, normalisés), séparateurs `/`.
fn relative_path(base: &Path, target: &Path) -> String {
    let (base, target) = (normalize(base), normalize(target));
    let base: Vec<_> = base.components().collect();
    let target: Vec<_> = target.components().collect();
    let common = base
        .iter()
        .zip(target.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts: Vec<String> = Vec::new();
    for _ in common..base.len() {
        parts.push("..".to_string());
    }
    for component in &target[common..] {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    parts.join("/")
}

// ---------------------------------------------------------------------------------------
// Auto-règles
// ---------------------------------------------------------------------------------------

fn int(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or_default()
}

/// Tuiles d'auto-règles (`autoLayerTiles`) d'une couche IntGrid ou AutoLayer, comme LDtk les écrit :
/// une tuile par règle qui s'applique, **dans l'ordre inverse de la liste des règles** (la dernière est
/// dessinée la première), puis par case ; `d = [uid de la règle, index de la case]`.
///
/// `cells` : valeurs IntGrid de la couche source, ligne par ligne, `width × height`.
pub fn auto_layer_tiles(
    layer_def: &Value,
    tilesets: &[Value],
    cells: &[i64],
    width: usize,
    height: usize,
) -> Result<Vec<Value>, String> {
    let groups = layer_def
        .get("autoRuleGroups")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let rules: Vec<&Value> = groups
        .iter()
        .filter(|g| g["active"].as_bool().unwrap_or(true))
        .flat_map(|g| g["rules"].as_array().into_iter().flatten())
        .filter(|r| r["active"].as_bool().unwrap_or(true))
        .collect();
    if rules.is_empty() {
        return Ok(Vec::new());
    }
    let tileset_uid = layer_def.get("tilesetDefUid").and_then(Value::as_i64);
    let tileset = tileset_uid
        .and_then(|uid| tilesets.iter().find(|t| t["uid"].as_i64() == Some(uid)))
        .ok_or_else(|| {
            format!(
                "couche « {} » : des auto-règles mais aucun tileset (tilesetDefUid)",
                layer_def["identifier"].as_str().unwrap_or("?")
            )
        })?;
    let columns = tileset["__cWid"].as_i64().unwrap_or(1).max(1);
    let grid = int(tileset, "tileGridSize").max(1);
    let (padding, spacing) = (int(tileset, "padding"), int(tileset, "spacing"));
    let layer_grid = int(layer_def, "gridSize").max(1);

    for rule in &rules {
        let id = int(rule, "uid");
        if rule["tileMode"].as_str().unwrap_or("Single") != "Single" {
            return Err(format!(
                "règle {id} : `tileMode` {} non pris en charge (seulement Single)",
                rule["tileMode"]
            ));
        }
        if rule["checker"].as_str().unwrap_or("None") != "None"
            || rule["perlinActive"].as_bool().unwrap_or(false)
        {
            return Err(format!(
                "règle {id} : damier et bruit de Perlin non pris en charge"
            ));
        }
        let size = int(rule, "size") as usize;
        let pattern_len = rule["pattern"].as_array().map_or(0, Vec::len);
        if size % 2 == 0 || pattern_len != size * size {
            return Err(format!("règle {id} : motif de taille incohérente"));
        }
        let has_tile = rule["tileRectsIds"]
            .as_array()
            .and_then(|r| r.first())
            .and_then(Value::as_array)
            .is_some_and(|ids| !ids.is_empty());
        if !has_tile {
            return Err(format!("règle {id} : aucune tuile"));
        }
    }

    let mut found: Vec<(usize, usize, Value)> = Vec::new();
    for cy in 0..height {
        for cx in 0..width {
            for (order, rule) in rules.iter().enumerate() {
                let Some((flip_x, flip_y)) = rule_matches(rule, cells, width, height, cx, cy)
                else {
                    continue;
                };
                let tile_id = rule["tileRectsIds"][0][0].as_i64().unwrap_or_default();
                let src_x = padding + (tile_id % columns) * (grid + spacing);
                let src_y = padding + (tile_id / columns) * (grid + spacing);
                let flags = i64::from(flip_x) | (i64::from(flip_y) << 1);
                found.push((
                    order,
                    cy * width + cx,
                    json!({
                        "px": [cx as i64 * layer_grid, cy as i64 * layer_grid],
                        "src": [src_x, src_y],
                        "f": flags,
                        "t": tile_id,
                        "d": [int(rule, "uid"), (cy * width + cx) as i64],
                        "a": rule.get("alpha").cloned().unwrap_or(json!(1)),
                    }),
                ));
                if rule["breakOnMatch"].as_bool().unwrap_or(true) {
                    break;
                }
            }
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    Ok(found.into_iter().map(|(_, _, tile)| tile).collect())
}

/// `Some((flip_x, flip_y))` si la règle s'applique à la case (la première variante retournée qui colle).
fn rule_matches(
    rule: &Value,
    cells: &[i64],
    width: usize,
    height: usize,
    cx: usize,
    cy: usize,
) -> Option<(bool, bool)> {
    let (x_mod, y_mod) = (int(rule, "xModulo").max(1), int(rule, "yModulo").max(1));
    if (cx as i64 - int(rule, "xOffset")).rem_euclid(x_mod) != 0
        || (cy as i64 - int(rule, "yOffset")).rem_euclid(y_mod) != 0
    {
        return None;
    }
    let size = int(rule, "size") as usize;
    let radius = (size / 2) as i64;
    let pattern: Vec<i64> = rule["pattern"]
        .as_array()?
        .iter()
        .map(|v| v.as_i64().unwrap_or_default())
        .collect();
    let out_of_bounds = rule.get("outOfBoundsValue").and_then(Value::as_i64);
    let mut variants = vec![(false, false)];
    let flip_x = rule["flipX"].as_bool().unwrap_or(false);
    let flip_y = rule["flipY"].as_bool().unwrap_or(false);
    if flip_x {
        variants.push((true, false));
    }
    if flip_y {
        variants.push((false, true));
    }
    if flip_x && flip_y {
        variants.push((true, true));
    }
    'variant: for (fx, fy) in variants {
        for py in 0..size {
            for px in 0..size {
                let wanted = pattern[py * size + px];
                if wanted == 0 {
                    continue;
                }
                let mut dx = px as i64 - radius;
                let mut dy = py as i64 - radius;
                if fx {
                    dx = -dx;
                }
                if fy {
                    dy = -dy;
                }
                let (x, y) = (cx as i64 + dx, cy as i64 + dy);
                let value = if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                    cells[y as usize * width + x as usize]
                } else if let Some(v) = out_of_bounds {
                    v
                } else {
                    continue 'variant;
                };
                let good = if wanted == PATTERN_ANY {
                    value != 0
                } else if wanted == -PATTERN_ANY {
                    value == 0
                } else if wanted > 0 {
                    value == wanted
                } else {
                    value != -wanted
                };
                if !good {
                    continue 'variant;
                }
            }
        }
        return Some((fx, fy));
    }
    None
}

// ---------------------------------------------------------------------------------------
// Construction de la carte
// ---------------------------------------------------------------------------------------

fn array<'a>(value: &'a Value, key: &str, what: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{what} : champ « {key} » absent"))
}

/// Une carte LDtk à un niveau, à partir du JSON du donneur. `donor_dir` et `out_dir` : dossiers du fichier
/// du donneur et du fichier à écrire (pour recalculer les `relPath` des tilesets).
pub fn build_arena(
    donor: &Value,
    donor_dir: &Path,
    out_dir: &Path,
    spec: &ArenaSpec,
) -> Result<Value, String> {
    spec.validate()?;
    let layer_defs = array(&donor["defs"], "layers", "donneur")?;
    let walls_def = layer_defs
        .iter()
        .find(|l| l["identifier"] == WALLS_LAYER && l["__type"] == "IntGrid")
        .ok_or(
            "le donneur n'a pas de couche IntGrid « Walls » : choisir un autre donneur (--from)",
        )?;
    let wall_value = walls_def["intGridValues"]
        .as_array()
        .and_then(|values| values.first())
        .map(|v| int(v, "value"))
        .filter(|v| *v > 0)
        .ok_or("la couche « Walls » du donneur n'a aucune valeur IntGrid")?;
    let levels = array(donor, "levels", "donneur")?;
    if donor
        .get("worlds")
        .and_then(Value::as_array)
        .is_some_and(|w| !w.is_empty())
    {
        return Err("donneur à plusieurs mondes : format non pris en charge".to_string());
    }
    let template_level = levels
        .iter()
        .find(|level| {
            level["fieldInstances"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|f| f["__identifier"] == "spawn" && f["__value"] == true)
        })
        .or_else(|| levels.first())
        .ok_or("le donneur n'a aucun niveau")?;
    let template_layers = template_level
        .get("layerInstances")
        .and_then(Value::as_array)
        .ok_or("niveaux externes (layerInstances absent) : format non pris en charge")?;
    let spawn_template = levels
        .iter()
        .flat_map(|level| level["layerInstances"].as_array().into_iter().flatten())
        .filter(|layer| layer["__identifier"] == ENTITIES_LAYER)
        .flat_map(|layer| layer["entityInstances"].as_array().into_iter().flatten())
        .find(|entity| entity["__identifier"] == PLAYER_SPAWN)
        .ok_or("le donneur n'a aucun PlayerSpawn à copier : choisir un autre donneur (--from)")?;

    let grid = int(walls_def, "gridSize").max(1);
    let (width, height) = (spec.width_tiles as usize, spec.height_tiles as usize);
    let (px_wid, px_hei) = (width as i64 * grid, height as i64 * grid);

    // Tilesets : `relPath` relatif au nouveau fichier.
    let mut project = donor.clone();
    let mut tileset_paths: Vec<(i64, Option<String>)> = Vec::new();
    if let Some(tilesets) = project["defs"]["tilesets"].as_array_mut() {
        for tileset in tilesets {
            let relative = tileset["relPath"]
                .as_str()
                .map(|rel| relative_path(out_dir, &normalize(&donor_dir.join(rel))));
            if let Some(relative) = &relative {
                tileset["relPath"] = json!(relative);
            }
            tileset_paths.push((int(tileset, "uid"), relative));
        }
    }
    let tilesets: Vec<Value> = project["defs"]["tilesets"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // Niveau.
    let level_uid = int(template_level, "uid");
    let mut level = template_level.clone();
    level["identifier"] = json!(LEVEL_IDENTIFIER);
    level["iid"] = json!(derived_iid(&format!("{}:level", spec.name)));
    level["worldX"] = json!(0);
    level["worldY"] = json!(0);
    level["worldDepth"] = json!(0);
    level["pxWid"] = json!(px_wid);
    level["pxHei"] = json!(px_hei);
    level["__neighbours"] = json!([]);
    if let Some(fields) = level["fieldInstances"].as_array_mut() {
        for field in fields {
            if field["__identifier"] == "spawn" {
                field["__value"] = json!(true);
                field["realEditorValues"] = json!([{"id": "V_Bool", "params": [true]}]);
            }
        }
    }

    // Points de départ : ceux des gabarits, décalés pour rester dans la salle d'une autre taille.
    let shift_x = (spec.width_tiles as i64 - i64::from(DEFAULT_WIDTH_TILES)) / 2 * grid;
    let shift_y = (spec.height_tiles as i64 - i64::from(DEFAULT_HEIGHT_TILES)) / 2 * grid;
    let spawns: Vec<Value> = DEFAULT_SPAWNS
        .iter()
        .take(spec.players as usize)
        .enumerate()
        .map(|(index, (x, y))| {
            let (x, y) = (x + shift_x, y + shift_y);
            let mut entity = spawn_template.clone();
            entity["iid"] = json!(derived_iid(&format!("{}:spawn:{index}", spec.name)));
            entity["px"] = json!([x, y]);
            entity["__grid"] = json!([x / grid, y / grid]);
            entity["__worldX"] = json!(x);
            entity["__worldY"] = json!(y);
            if let Some(fields) = entity["fieldInstances"].as_array_mut() {
                for field in fields {
                    if field["__identifier"] == "index" {
                        field["__value"] = json!(index);
                        field["realEditorValues"] = if index == 0 {
                            json!([])
                        } else {
                            json!([{"id": "V_Int", "params": [index]}])
                        };
                    }
                }
            }
            entity
        })
        .collect();

    // Couches : mêmes définitions que le niveau modèle, contenu remplacé. Les IntGrid d'abord (les couches
    // automatiques lisent leur valeur), dans l'ordre du niveau modèle.
    let mut cells_by_layer: Vec<(i64, Vec<i64>)> = Vec::new();
    let mut layers: Vec<Value> = Vec::new();
    for template_layer in template_layers {
        let def_uid = int(template_layer, "layerDefUid");
        let def = layer_defs
            .iter()
            .find(|d| int(d, "uid") == def_uid)
            .ok_or_else(|| format!("couche {def_uid} : définition absente du donneur"))?;
        let mut layer = template_layer.clone();
        layer["__cWid"] = json!(width);
        layer["__cHei"] = json!(height);
        layer["iid"] = json!(derived_iid(&format!(
            "{}:layer:{}",
            spec.name,
            template_layer["__identifier"].as_str().unwrap_or("?")
        )));
        layer["levelId"] = json!(level_uid);
        layer["gridTiles"] = json!([]);
        layer["autoLayerTiles"] = json!([]);
        layer["entityInstances"] = json!([]);
        if let Some(tileset_uid) = layer.get("__tilesetDefUid").and_then(Value::as_i64) {
            if let Some((_, Some(rel))) = tileset_paths.iter().find(|(uid, _)| *uid == tileset_uid)
            {
                layer["__tilesetRelPath"] = json!(rel);
            }
        }
        match template_layer["__type"].as_str() {
            Some("IntGrid") => {
                let mut cells = vec![0i64; width * height];
                if template_layer["__identifier"] == WALLS_LAYER {
                    for y in 0..height {
                        for x in 0..width {
                            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                                cells[y * width + x] = wall_value;
                            }
                        }
                    }
                }
                layer["intGridCsv"] = json!(cells);
                layer["autoLayerTiles"] =
                    json!(auto_layer_tiles(def, &tilesets, &cells, width, height)?);
                cells_by_layer.push((def_uid, cells));
            }
            Some("Entities") => {
                if template_layer["__identifier"] == ENTITIES_LAYER {
                    layer["entityInstances"] = json!(spawns);
                }
            }
            _ => {}
        }
        layers.push(layer);
    }
    // Couches automatiques (`AutoLayer`) pilotées par une IntGrid.
    for layer in &mut layers {
        if layer["__type"] != "AutoLayer" {
            continue;
        }
        let def_uid = int(layer, "layerDefUid");
        let def = layer_defs.iter().find(|d| int(d, "uid") == def_uid);
        let source = def.and_then(|d| d.get("autoSourceLayerDefUid").and_then(Value::as_i64));
        if let (Some(def), Some(source)) = (def, source) {
            if let Some((_, cells)) = cells_by_layer.iter().find(|(uid, _)| *uid == source) {
                layer["autoLayerTiles"] =
                    json!(auto_layer_tiles(def, &tilesets, cells, width, height)?);
            }
        }
    }
    level["layerInstances"] = json!(layers);

    project["iid"] = json!(derived_iid(&format!("{}:project", spec.name)));
    project["dummyWorldIid"] = json!(derived_iid(&format!("{}:world", spec.name)));
    project["toc"] = json!([]);
    project["levels"] = json!([level]);
    Ok(project)
}

/// Texte du fichier : clés triées, un espace d'indentation (comme LDtk), fin de ligne.
pub fn to_ldtk_text(project: &Value) -> Result<String, String> {
    let mut out = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut serializer = serde_json::Serializer::with_formatter(&mut out, formatter);
    serde::Serialize::serialize(&sorted(project), &mut serializer).map_err(|e| e.to_string())?;
    let mut text = String::from_utf8(out).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text)
}

/// Copie à clés triées (serde_json sans `preserve_order` trie déjà, mais pas si une autre crate l'active :
/// la sortie ne doit pas dépendre de l'unification des features).
fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            let mut out = Map::new();
            for (key, value) in entries {
                out.insert(key.clone(), sorted(value));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------------------
// Sur le disque
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ArenaRequest {
    pub spec: ArenaSpec,
    /// Carte donneuse : chemin relatif à `assets/` du jeu, ou chemin du système de fichiers.
    pub from: Option<String>,
    /// Fichier à écrire : relatif à `assets/` du jeu (défaut : `maps/<nom>.ldtk` si le jeu a un dossier
    /// `maps/`, sinon le dossier du donneur).
    pub out: Option<String>,
    /// Pose `sandbox: (arena: "<carte>")` dans `game.ron`.
    pub set_default: bool,
}

#[derive(Debug, Clone)]
pub struct ArenaReport {
    /// Fichier écrit, relatif à `assets/` (séparateurs `/`) : la valeur de `sandbox.arena`.
    pub map: String,
    pub file: PathBuf,
    pub donor: PathBuf,
    /// Le fichier était déjà identique : rien d'écrit.
    pub unchanged: bool,
    /// Le registre du jeu connaît cette carte (dossier ou fichier déclaré `Map`).
    pub declared: bool,
    /// `game.ron` a été modifié (`set_default`).
    pub manifest_updated: bool,
    /// `sandbox.arena` du manifeste désigne déjà cette carte.
    pub is_default: bool,
}

fn resolve_existing(assets: &Path, designation: &str) -> Option<PathBuf> {
    let in_assets = assets.join(designation);
    if in_assets.is_file() {
        return Some(in_assets);
    }
    let direct = PathBuf::from(designation);
    direct.is_file().then_some(direct)
}

/// `alacod new-arena` : écrit l'arène vide du jeu `game_dir`.
pub fn new_arena(game_dir: &Path, request: &ArenaRequest) -> Result<ArenaReport, String> {
    request.spec.validate()?;
    let manifest = GameManifest::load(game_dir).map_err(|e| e.to_string())?;
    let assets = GameManifest::assets_dir(game_dir);
    let (registry, _) = Registry::build(game_dir, &manifest);

    let donor_path = match &request.from {
        Some(from) => resolve_existing(&assets, from).ok_or_else(|| {
            format!(
                "donneur « {from} » introuvable (relatif à {})",
                assets.display()
            )
        })?,
        None => {
            let mut candidates: Vec<String> = Vec::new();
            if let Some(sandbox) = &manifest.sandbox {
                candidates.push(sandbox.arena.clone());
            }
            if let Some(template) = &manifest.generate_template {
                candidates.push(template.map.clone());
            }
            let from_registry = registry
                .maps
                .values()
                .map(|entry| entry.file.to_string_lossy().into_owned());
            candidates
                .into_iter()
                .chain(from_registry)
                .filter(|c| c.ends_with(".ldtk"))
                .find_map(|c| resolve_existing(&assets, &c))
                .ok_or("aucune carte LDtk du jeu à prendre pour donneur (--from <carte.ldtk>)")?
        }
    };
    let donor_dir = donor_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    let out_rel = match &request.out {
        Some(out) => out.clone(),
        None if assets.join("maps").is_dir() => format!("maps/{}.ldtk", request.spec.name),
        None => relative_path(
            &assets,
            &donor_dir.join(format!("{}.ldtk", request.spec.name)),
        ),
    };
    if !out_rel.ends_with(".ldtk") || Path::new(&out_rel).is_absolute() || out_rel.contains("..") {
        return Err(format!(
            "sortie « {out_rel} » : un chemin relatif à assets/, terminé par .ldtk, sans « .. »"
        ));
    }
    let out_file = assets.join(&out_rel);
    let out_dir = out_file.parent().unwrap_or(&assets).to_path_buf();
    if let Some(stem) = Path::new(&out_rel).file_stem().and_then(|s| s.to_str()) {
        if stem != request.spec.name {
            return Err(format!(
                "sortie « {out_rel} » : l'id de carte est le nom du fichier (« {stem} »), pas « {} » ; \
                 donner --name {stem} ou une sortie qui se termine par {}.ldtk",
                request.spec.name, request.spec.name
            ));
        }
    }

    let donor_text = std::fs::read_to_string(&donor_path)
        .map_err(|e| format!("{} : {e}", donor_path.display()))?;
    let donor: Value = serde_json::from_str(&donor_text)
        .map_err(|e| format!("{} : JSON LDtk invalide : {e}", donor_path.display()))?;
    let project = build_arena(&donor, &donor_dir, &out_dir, &request.spec)
        .map_err(|e| format!("{} : {e}", donor_path.display()))?;
    let text = to_ldtk_text(&project)?;

    let unchanged = std::fs::read_to_string(&out_file).is_ok_and(|existing| existing == text);
    if !unchanged {
        std::fs::create_dir_all(&out_dir).map_err(|e| format!("{} : {e}", out_dir.display()))?;
        std::fs::write(&out_file, &text).map_err(|e| format!("{} : {e}", out_file.display()))?;
    }

    let manifest_updated = if request.set_default {
        set_sandbox_arena(&assets.join("game.ron"), &out_rel)?
    } else {
        false
    };

    // Déclarée ? Registre relu : le fichier doit y être sous son id.
    let manifest = GameManifest::load(game_dir).map_err(|e| e.to_string())?;
    let (registry, _) = Registry::build(game_dir, &manifest);
    let declared = registry
        .maps
        .get(&map_id_from_path(&out_rel))
        .is_some_and(|entry| entry.file == Path::new(&out_rel));
    let is_default = manifest
        .sandbox
        .as_ref()
        .is_some_and(|sandbox| sandbox.arena == out_rel);

    Ok(ArenaReport {
        map: out_rel,
        file: out_file,
        donor: donor_path,
        unchanged,
        declared,
        manifest_updated,
        is_default,
    })
}

/// Pose `sandbox: (arena: "<map>")` dans le texte de `game.ron` : remplace l'arène d'un champ `sandbox`
/// existant, sinon l'ajoute avant la parenthèse finale. `true` si le fichier a changé.
fn set_sandbox_arena(manifest_file: &Path, map: &str) -> Result<bool, String> {
    let text = std::fs::read_to_string(manifest_file)
        .map_err(|e| format!("{} : {e}", manifest_file.display()))?;
    let new_text = with_sandbox_arena(&text, map).ok_or_else(|| {
        format!(
            "{} : champ `sandbox` illisible, à écrire à la main",
            manifest_file.display()
        )
    })?;
    if new_text == text {
        return Ok(false);
    }
    std::fs::write(manifest_file, new_text)
        .map_err(|e| format!("{} : {e}", manifest_file.display()))?;
    Ok(true)
}

/// Voir [`set_sandbox_arena`] ; `None` si la forme du champ existant n'est pas `sandbox: (arena: "…")`.
fn with_sandbox_arena(text: &str, map: &str) -> Option<String> {
    // Champ existant : une ligne `sandbox: (arena: "…")` hors commentaire.
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("//") && trimmed.starts_with("sandbox:") {
            let open = line.find('"')?;
            let close = open + 1 + line[open + 1..].find('"')?;
            if !line[..open].contains("arena:") {
                return None;
            }
            let mut out = String::with_capacity(text.len() + map.len());
            out.push_str(&text[..offset + open + 1]);
            out.push_str(map);
            out.push_str(&text[offset + close..]);
            return Some(out);
        }
        offset += line.len();
    }
    // Absent : avant la dernière parenthèse fermante.
    let end = text.rfind(')')?;
    let head = text[..end].trim_end_matches([' ', '\t']);
    let head = if head.ends_with('\n') {
        head.to_string()
    } else {
        format!("{head}\n")
    };
    Some(format!(
        "{head}    // SB-1 (D2) : l'arène du bac à sable (`--sandbox`) : une salle vide (murs et points de départ, rien d'autre),\n    // écrite par `alacod new-arena` (`docs/conventions.md` §90) ; un profil la remplace par son champ `map`.\n    sandbox: (arena: \"{map}\"),\n{}",
        &text[end..]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iid_stable_et_distinct() {
        assert_eq!(derived_iid("a"), derived_iid("a"));
        assert_ne!(derived_iid("a"), derived_iid("b"));
        assert_eq!(derived_iid("a").len(), 36);
    }

    #[test]
    fn chemin_relatif_entre_dossiers() {
        let assets = Path::new("g/assets");
        assert_eq!(
            relative_path(&assets.join("maps"), &assets.join("maps/../atlas/x.png")),
            "../atlas/x.png"
        );
        assert_eq!(
            relative_path(
                &assets.join("exemples"),
                &assets.join("exemples/atlas/x.png")
            ),
            "atlas/x.png"
        );
        assert_eq!(
            relative_path(assets, &assets.join("maps/a.ldtk")),
            "maps/a.ldtk"
        );
    }

    #[test]
    fn la_pose_du_champ_sandbox_remplace_ou_ajoute() {
        let with = "(\n    name: \"x\",\n    sandbox: (arena: \"a/b.ldtk\"),\n)\n";
        assert_eq!(
            with_sandbox_arena(with, "maps/c.ldtk").unwrap(),
            "(\n    name: \"x\",\n    sandbox: (arena: \"maps/c.ldtk\"),\n)\n"
        );
        let without = "(\n    name: \"x\",\n    // sandbox: (arena: \"no\")\n)\n";
        let out = with_sandbox_arena(without, "maps/c.ldtk").unwrap();
        assert!(
            out.contains("    sandbox: (arena: \"maps/c.ldtk\"),\n)\n"),
            "{out}"
        );
        // Deuxième pose : identique.
        assert_eq!(with_sandbox_arena(&out, "maps/c.ldtk").unwrap(), out);
    }
}
