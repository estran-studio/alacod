//! Aperçu d'une carte assemblée sans lancer le jeu (`alacod-map`, `docs/conventions.md` §1) :
//! génération par graine, rendu PNG (tuiles des calques LDtk quand les tilesets sont trouvés,
//! sinon plan abstrait) avec le plan par-dessus (index des pièces, prises reliées en vert et
//! murées en rouge, portes, apparitions, fenêtres, départ bordé d'or), et le `.ldtk` assemblé
//! relié à ses tilesets pour l'éditeur. Récompenses : pistolet jaune pour les armes murales,
//! croix blanche pour les sodas ; ids, prix et positions locales en cases dans le bilan.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Component, Path, PathBuf};

use bevy_ecs_ldtk::ldtk::{FieldValue, LayerInstance, LdtkJson, Level, TileInstance, Type};
use map::generation::{
    config::{MapGenerationConfig, MapGenerationMode},
    context::Side,
    map_generation,
};

use crate::generation::{from_map, GeneratedMap};
use crate::loader::file::load_ldtk_json_file;
use crate::map_const;

/// Pixels par case du plan abstrait (sans tuiles).
const PLAIN_SCALE: usize = 6;
const MARGIN_CELLS: usize = 2;

const FLOOR: [u8; 4] = [200, 200, 200, 255];
const WALL: [u8; 4] = [20, 20, 20, 255];
const BACKGROUND: [u8; 4] = [60, 60, 70, 255];
const OPEN_LINKED: [u8; 4] = [60, 200, 60, 255];
const OPEN_CLOSED: [u8; 4] = [220, 50, 50, 255];
const DOOR: [u8; 4] = [240, 150, 30, 255];
const PLAYER_SPAWN: [u8; 4] = [60, 110, 240, 255];
const ZOMBIE_SPAWN: [u8; 4] = [230, 60, 200, 255];
const CHARACTER_SPAWN: [u8; 4] = [140, 60, 220, 255];
const WINDOW: [u8; 4] = [60, 210, 230, 255];
const WEAPON_REWARD: [u8; 4] = [255, 240, 130, 255];
const SODA_REWARD: [u8; 4] = [255, 255, 255, 255];
const SPAWN_BORDER: [u8; 4] = [240, 200, 40, 255];
const LABEL: [u8; 4] = [255, 255, 255, 255];
const LABEL_SHADOW: [u8; 4] = [0, 0, 0, 255];

/// Une carte générée, prête à rendre ou à écrire.
pub struct GeneratedPreview {
    pub ldtk: LdtkJson,
    /// Gabarit d'origine de chaque niveau, dans l'ordre des niveaux.
    pub templates: Vec<String>,
}

/// Génère la carte de `library` avec `config` (`seed`, réglages, mode).
pub fn generate(library: &Path, config: MapGenerationConfig) -> GeneratedPreview {
    let data: LdtkJson = load_ldtk_json_file(library.to_str().expect("chemin UTF-8"))
        .unwrap_or_else(|e| panic!("{} : {e:?}", library.display()));
    if let MapGenerationMode::Cave(cave) = &config.mode {
        let id = config.map_path.strip_prefix("cave:").unwrap_or("preview");
        let ldtk = crate::generation::cave::build_cave_ldtk(&data, id, config.seed, cave);
        return GeneratedPreview {
            templates: vec![data.levels[0].identifier.clone()],
            ldtk,
        };
    }
    let context = from_map(&data, config);
    let mut generator = GeneratedMap::create(data);
    map_generation(context, &mut generator).expect("génération");
    let templates = generator
        .generated_rooms
        .iter()
        .map(|room| room.template.clone())
        .collect();
    GeneratedPreview {
        ldtk: generator.get_generated_map(),
        templates,
    }
}

/// Écrit le `.ldtk` assemblé dans `out`, tilesets reliés relativement à son dossier (les
/// `relPath` de la bibliothèque sont relatifs à son propre fichier).
pub fn write_ldtk(preview: &GeneratedPreview, library: &Path, out: &Path) {
    let mut ldtk = preview.ldtk.clone();
    let library_dir = absolute(library.parent().unwrap_or(Path::new(".")));
    let out_dir = absolute(out.parent().unwrap_or(Path::new(".")));
    for tileset in &mut ldtk.defs.tilesets {
        if let Some(rel) = tileset.rel_path.as_mut() {
            let target = normalize(&library_dir.join(rel.as_str()));
            *rel = relative_to(&out_dir, &target)
                .to_string_lossy()
                .replace('\\', "/");
        }
    }
    for level in &mut ldtk.levels {
        for layer in level.layer_instances.iter_mut().flatten() {
            if let Some(def) = ldtk
                .defs
                .tilesets
                .iter()
                .find(|def| Some(def.uid) == layer.tileset_def_uid)
            {
                layer.tileset_rel_path = def.rel_path.clone();
            }
        }
    }
    std::fs::write(
        out,
        serde_json::to_string_pretty(&ldtk).expect("sérialisation"),
    )
    .unwrap_or_else(|e| panic!("{} : {e}", out.display()));
}

/// Bilan d'un rendu, pour la console.
pub struct Report {
    /// Une ligne par pièce : gabarit, position en cases, taille, départ, prises murées,
    /// récompenses (arme jaune / soda à croix blanche, id, prix, position locale en cases).
    pub rooms: Vec<String>,
    pub loops: usize,
    pub walled: usize,
    /// Tilesets de la bibliothèque introuvables (rendu abstrait pour leurs calques).
    pub missing_tilesets: Vec<String>,
}

/// Rend la carte en PNG : tuiles des calques LDtk (`tiles`, si les tilesets sont trouvés à
/// côté de `library`) sinon plan abstrait, puis le plan par-dessus.
pub fn render(preview: &GeneratedPreview, library: &Path, tiles: bool, png_path: &Path) -> Report {
    let ldtk = &preview.ldtk;
    let tile = ldtk.default_grid_size.max(1) as usize;
    let mut tilesets = Tilesets::default();
    let mut missing = Vec::new();
    if tiles {
        let library_dir = absolute(library.parent().unwrap_or(Path::new(".")));
        for def in &ldtk.defs.tilesets {
            let Some(rel) = &def.rel_path else { continue };
            let path = normalize(&library_dir.join(rel));
            match load_png(&path) {
                Some(image) => {
                    tilesets.images.insert(def.uid, image);
                }
                None => missing.push(rel.clone()),
            }
        }
    }
    // Échelle : 1 px par pixel monde avec des tuiles, sinon PLAIN_SCALE px par case.
    let scale = if tilesets.images.is_empty() {
        PLAIN_SCALE
    } else {
        tile
    };
    let margin = MARGIN_CELLS * scale;

    let min_x = ldtk.levels.iter().map(|l| l.world_x).min().unwrap_or(0);
    let min_y = ldtk.levels.iter().map(|l| l.world_y).min().unwrap_or(0);
    let max_x = ldtk
        .levels
        .iter()
        .map(|l| l.world_x + l.px_wid)
        .max()
        .unwrap_or(0);
    let max_y = ldtk
        .levels
        .iter()
        .map(|l| l.world_y + l.px_hei)
        .max()
        .unwrap_or(0);
    let width = ((max_x - min_x) as usize / tile) * scale + 2 * margin;
    let height = ((max_y - min_y) as usize / tile) * scale + 2 * margin;
    let mut canvas = Canvas::new(width, height);
    let mut report = Report {
        rooms: Vec::new(),
        loops: 0,
        walled: 0,
        missing_tilesets: missing,
    };

    let all_openings: Vec<_> = ldtk.levels.iter().map(openings).collect();
    for level in &ldtk.levels {
        let ox = ((level.world_x - min_x) as usize / tile) * scale + margin;
        let oy = ((level.world_y - min_y) as usize / tile) * scale + margin;
        let (cw, ch) = (level.px_wid as usize / tile, level.px_hei as usize / tile);
        canvas.fill(ox, oy, cw * scale, ch * scale, FLOOR);

        // Calques LDtk, du fond vers le dessus (l'éditeur liste le dessus en premier).
        for layer in level.layer_instances.iter().flatten().rev() {
            if !layer.visible {
                continue;
            }
            let has_tiles = layer
                .tileset_def_uid
                .is_some_and(|uid| tilesets.images.contains_key(&uid));
            match layer.layer_instance_type {
                Type::IntGrid if layer.identifier == "Walls" => {
                    // Les cases murées par le générateur n'ont pas de tuile d'auto-règle
                    // (LDtk les recalcule à l'ouverture) : fond sombre sous les tuiles.
                    for (i, cell) in layer.int_grid_csv.iter().enumerate() {
                        if *cell != 0 {
                            let (x, y) = (i % layer.c_wid as usize, i / layer.c_wid as usize);
                            canvas.fill(ox + x * scale, oy + y * scale, scale, scale, WALL);
                        }
                    }
                    if has_tiles {
                        draw_tiles(&mut canvas, &tilesets, layer, ox, oy);
                    }
                }
                Type::IntGrid | Type::Tiles | Type::AutoLayer if has_tiles => {
                    draw_tiles(&mut canvas, &tilesets, layer, ox, oy);
                }
                _ => {}
            }
        }
    }

    // Les auto-tuiles peuvent dépasser le rectangle d'une salle : les annotations passent
    // après toutes les salles pour qu'une voisine ne masque ni une prise ni un marqueur.
    for (index, level) in ldtk.levels.iter().enumerate() {
        let ox = ((level.world_x - min_x) as usize / tile) * scale + margin;
        let oy = ((level.world_y - min_y) as usize / tile) * scale + margin;
        let (cw, ch) = (level.px_wid as usize / tile, level.px_hei as usize / tile);

        let mut rewards = Vec::new();
        if let Some(entities) = layer_named(level, map_const::LAYER_ENTITY) {
            for entity in &entities.entity_instances {
                let reward = match entity.identifier.as_str() {
                    map_const::ENTITY_WEAPON_LOCATION => {
                        Some((map_const::FIELD_WEAPON_NAME, "arme jaune", true))
                    }
                    map_const::ENTITY_SODA_LOCATION => {
                        Some((map_const::FIELD_PERK_NAME, "soda croix blanche", false))
                    }
                    _ => None,
                };
                if let Some((field_name, kind, weapon)) = reward {
                    let id = entity
                        .field_instances
                        .iter()
                        .find_map(|field| match &field.value {
                            FieldValue::String(Some(value)) if field.identifier == field_name => {
                                Some(value.as_str())
                            }
                            _ => None,
                        })
                        .unwrap_or("?");
                    let price = entity
                        .field_instances
                        .iter()
                        .find_map(|field| match &field.value {
                            FieldValue::Int(Some(value))
                                if field.identifier == map_const::FIELD_PRICE_NAME =>
                            {
                                Some(value.to_string())
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| if weapon { "absent" } else { "registre" }.to_owned());
                    rewards.push(format!(
                        "{kind} {field_name}={id} prix={price} local=({},{})",
                        entity.px.x / tile as i32,
                        entity.px.y / tile as i32,
                    ));
                    let x = ox + (entity.px.x as usize * scale) / tile;
                    let y = oy + (entity.px.y as usize * scale) / tile;
                    let w = (entity.width as usize * scale / tile).max(scale);
                    let h = (entity.height as usize * scale / tile).max(scale);
                    canvas.reward_marker(x + w / 2, y + h / 2, scale, weapon);
                    continue;
                }
                let color = match entity.identifier.as_str() {
                    "DoorHorizontal" | "DoorVertical" => DOOR,
                    "WindowHorizontal" | "WindowVertical" => WINDOW,
                    "PlayerSpawn" => PLAYER_SPAWN,
                    "ZombieSpawn" => ZOMBIE_SPAWN,
                    "CharacterSpawn" => CHARACTER_SPAWN,
                    _ => continue,
                };
                let x = ox + (entity.px.x as usize * scale) / tile;
                let y = oy + (entity.px.y as usize * scale) / tile;
                let w = (entity.width as usize * scale / tile).max(scale);
                let h = (entity.height as usize * scale / tile).max(scale);
                canvas.fill(x, y, w, h, color);
            }
        }

        let mut closed_here = Vec::new();
        for opening in &all_openings[index] {
            let linked = opening_is_linked(level, opening, ldtk, &all_openings);
            let walled = opening_is_walled(level, opening);
            let color = if linked { OPEN_LINKED } else { OPEN_CLOSED };
            if !linked {
                closed_here.push(format!(
                    "{}@{}+{} ({})",
                    opening.side.to_dir_str(),
                    opening.start,
                    opening.size,
                    if walled { "murée" } else { "ouverte" },
                ));
                report.walled += usize::from(walled);
            }
            let grid = layer_named(level, map_const::LAYER_CONNECTION)
                .expect("couche des prises")
                .grid_size as usize;
            let strip = (grid * scale / tile / 2).max(1);
            for k in opening.start..opening.start + opening.size {
                let (x, y) = opening.cell(level, k);
                let x = ox + x * grid * scale / tile;
                let y = oy + y * grid * scale / tile;
                let cell = grid * scale / tile;
                let (sx, sy, sw, sh) = match opening.side {
                    Side::N => (x, y, cell, strip),
                    Side::S => (x, y + cell - strip, cell, strip),
                    Side::W => (x, y, strip, cell),
                    Side::E => (x + cell - strip, y, strip, cell),
                };
                canvas.fill(sx, sy, sw, sh, color);
            }
        }

        let spawn = is_spawn(level);
        if spawn {
            canvas.outline(ox, oy, cw * scale, ch * scale, SPAWN_BORDER);
            canvas.outline(ox + 1, oy + 1, cw * scale - 2, ch * scale - 2, SPAWN_BORDER);
        }
        let glyph = (scale / 3).max(1);
        canvas.digits(
            ox + scale + 2 + glyph,
            oy + scale + 2 + glyph,
            index,
            glyph,
            LABEL_SHADOW,
        );
        canvas.digits(ox + scale + 2, oy + scale + 2, index, glyph, LABEL);

        let template = preview
            .templates
            .get(index)
            .map(String::as_str)
            .unwrap_or("?");
        let mut line = format!(
            "{template:<24} cases ({:>4},{:>4}) {cw}x{ch}",
            level.world_x / tile as i32,
            level.world_y / tile as i32
        );
        if spawn {
            line.push_str("  départ");
        }
        if !closed_here.is_empty() {
            line.push_str(&format!("  sans voisine : {}", closed_here.join(", ")));
        }
        if !rewards.is_empty() {
            line.push_str(&format!("  récompenses : {}", rewards.join(" ; ")));
        }
        report.rooms.push(line);
    }

    // Arêtes = voisinages / 2 ; un arbre en a `salles − 1`, le reste ferme des boucles.
    let edges: usize = ldtk
        .levels
        .iter()
        .map(|l| l.neighbours.len())
        .sum::<usize>()
        / 2;
    report.loops = (edges + 1).saturating_sub(ldtk.levels.len());

    canvas.write(png_path);
    report
}

/// Une prise entière, pas seulement son côté : un bord peut en porter plusieurs.
#[derive(Clone, Copy)]
struct Opening {
    side: Side,
    start: usize,
    size: usize,
}

impl Opening {
    fn cell(&self, level: &Level, k: usize) -> (usize, usize) {
        let layer = layer_named(level, map_const::LAYER_CONNECTION).expect("couche des prises");
        match self.side {
            Side::N => (k, 0),
            Side::S => (k, layer.c_hei as usize - 1),
            Side::W => (0, k),
            Side::E => (layer.c_wid as usize - 1, k),
        }
    }

    fn span(&self, level: &Level) -> (i32, i32, i32) {
        let grid = layer_named(level, map_const::LAYER_CONNECTION)
            .expect("couche des prises")
            .grid_size;
        let offset = self.start as i32 * grid;
        let size = self.size as i32 * grid;
        match self.side {
            Side::N => (
                level.world_y,
                level.world_x + offset,
                level.world_x + offset + size,
            ),
            Side::S => (
                level.world_y + level.px_hei,
                level.world_x + offset,
                level.world_x + offset + size,
            ),
            Side::W => (
                level.world_x,
                level.world_y + offset,
                level.world_y + offset + size,
            ),
            Side::E => (
                level.world_x + level.px_wid,
                level.world_y + offset,
                level.world_y + offset + size,
            ),
        }
    }
}

fn openings(level: &Level) -> Vec<Opening> {
    let Some(layer) = layer_named(level, map_const::LAYER_CONNECTION) else {
        return Vec::new();
    };
    let (w, h) = (layer.c_wid as usize, layer.c_hei as usize);
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    for side in [Side::N, Side::S, Side::W, Side::E] {
        let mut k = 0;
        let len = match side {
            Side::N | Side::S => w,
            _ => h,
        };
        while k < len {
            let cell = |k| match side {
                Side::N => k,
                Side::S => (h - 1) * w + k,
                Side::W => k * w,
                Side::E => k * w + w - 1,
            };
            if layer.int_grid_csv[cell(k)] == 0 {
                k += 1;
                continue;
            }
            let start = k;
            while k < len && layer.int_grid_csv[cell(k)] != 0 {
                k += 1;
            }
            result.push(Opening {
                side,
                start,
                size: k - start,
            });
        }
    }
    result
}

fn opening_is_linked(
    level: &Level,
    opening: &Opening,
    ldtk: &LdtkJson,
    all: &[Vec<Opening>],
) -> bool {
    level
        .neighbours
        .iter()
        .filter(|n| n.dir == opening.side.to_dir_str())
        .any(|neighbour| {
            ldtk.levels.iter().enumerate().any(|(index, other)| {
                other.iid == neighbour.level_iid
                    && all[index].iter().any(|candidate| {
                        candidate.side == opening.side.get_opposite()
                            && candidate.span(other) == opening.span(level)
                    })
            })
        })
}

fn opening_is_walled(level: &Level, opening: &Opening) -> bool {
    let Some(walls) = layer_named(level, "Walls") else {
        return false;
    };
    let grid = layer_named(level, map_const::LAYER_CONNECTION)
        .expect("couche des prises")
        .grid_size as usize;
    (opening.start..opening.start + opening.size).all(|k| {
        let (x, y) = opening.cell(level, k);
        let x = x * grid / walls.grid_size as usize;
        let y = y * grid / walls.grid_size as usize;
        walls
            .int_grid_csv
            .get(y * walls.c_wid as usize + x)
            .is_some_and(|cell| *cell != 0)
    })
}

fn draw_tiles(
    canvas: &mut Canvas,
    tilesets: &Tilesets,
    layer: &LayerInstance,
    ox: usize,
    oy: usize,
) {
    let Some(image) = layer
        .tileset_def_uid
        .and_then(|uid| tilesets.images.get(&uid))
    else {
        return;
    };
    let tiles: &[TileInstance] = if matches!(layer.layer_instance_type, Type::Tiles) {
        &layer.grid_tiles
    } else {
        &layer.auto_layer_tiles
    };
    let size = layer.grid_size.max(1) as usize;
    for t in tiles {
        let x = ox as i64 + layer.px_total_offset_x as i64 + t.px.x as i64;
        let y = oy as i64 + layer.px_total_offset_y as i64 + t.px.y as i64;
        canvas.blit(
            image,
            t.src.x as usize,
            t.src.y as usize,
            size,
            x,
            y,
            t.f & 1 != 0,
            t.f & 2 != 0,
            (t.a.clamp(0.0, 1.0) * layer.opacity.clamp(0.0, 1.0) * 255.0) as u8,
        );
    }
}

fn layer_named<'a>(level: &'a Level, identifier: &str) -> Option<&'a LayerInstance> {
    level
        .layer_instances
        .iter()
        .flatten()
        .find(|l| l.identifier == identifier)
}

fn is_spawn(level: &Level) -> bool {
    level.field_instances.iter().any(|f| {
        f.identifier == map_const::LEVEL_FIELD_SPAWN && matches!(f.value, FieldValue::Bool(true))
    })
}

#[derive(Default)]
struct Tilesets {
    images: BTreeMap<i32, Image>,
}

struct Image {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

fn load_png(path: &Path) -> Option<Image> {
    let file = File::open(path).ok()?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let (width, height) = (info.width as usize, info.height as usize);
    let channels = info.color_type.samples();
    let mut rgba = Vec::with_capacity(width * height * 4);
    for px in buf[..info.buffer_size()].chunks(channels) {
        match channels {
            4 => rgba.extend_from_slice(px),
            3 => rgba.extend_from_slice(&[px[0], px[1], px[2], 255]),
            2 => rgba.extend_from_slice(&[px[0], px[0], px[0], px[1]]),
            _ => rgba.extend_from_slice(&[px[0], px[0], px[0], 255]),
        }
    }
    Some(Image {
        width,
        height,
        rgba,
    })
}

struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        let mut pixels = Vec::with_capacity(width * height * 4);
        for _ in 0..width * height {
            pixels.extend_from_slice(&BACKGROUND);
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, color: [u8; 4]) {
        for py in y..(y + h).min(self.height) {
            for px in x..(x + w).min(self.width) {
                let i = (py * self.width + px) * 4;
                self.pixels[i..i + 4].copy_from_slice(&color);
            }
        }
    }

    fn outline(&mut self, x: usize, y: usize, w: usize, h: usize, color: [u8; 4]) {
        self.fill(x, y, w, 1, color);
        self.fill(x, y + h - 1, w, 1, color);
        self.fill(x, y, 1, h, color);
        self.fill(x + w - 1, y, 1, h, color);
    }

    /// Pictogrammes 7×5 sur fond noir : lisibles même à PLAIN_SCALE, sans tileset.
    fn reward_marker(&mut self, cx: usize, cy: usize, scale: usize, weapon: bool) {
        let dot = (scale / 8).max(1);
        let x = cx.saturating_sub(9 * dot / 2);
        let y = cy.saturating_sub(7 * dot / 2);
        self.fill(x, y, 9 * dot, 7 * dot, LABEL_SHADOW);
        let (glyph, color) = if weapon {
            (
                [0b0000000, 0b1111111, 0b1111110, 0b0110100, 0b0110000],
                WEAPON_REWARD,
            )
        } else {
            (
                [0b0001000, 0b0001000, 0b0111110, 0b0001000, 0b0001000],
                SODA_REWARD,
            )
        };
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..7 {
                if bits & (0b1000000 >> col) != 0 {
                    self.fill(x + (col + 1) * dot, y + (row + 1) * dot, dot, dot, color);
                }
            }
        }
    }

    /// Copie un carré `size × size` de `image` (coin `sx, sy`) en `dx, dy`, avec retournement
    /// et fondu alpha (les tuiles LDtk ont des pixels transparents).
    #[allow(clippy::too_many_arguments)]
    fn blit(
        &mut self,
        image: &Image,
        sx: usize,
        sy: usize,
        size: usize,
        dx: i64,
        dy: i64,
        flip_x: bool,
        flip_y: bool,
        alpha: u8,
    ) {
        for j in 0..size {
            for i in 0..size {
                let (si, sj) = (
                    if flip_x { size - 1 - i } else { i },
                    if flip_y { size - 1 - j } else { j },
                );
                let (px, py) = (sx + si, sy + sj);
                let (qx, qy) = (dx + i as i64, dy + j as i64);
                if px >= image.width
                    || py >= image.height
                    || qx < 0
                    || qy < 0
                    || qx >= self.width as i64
                    || qy >= self.height as i64
                {
                    continue;
                }
                let s = (py * image.width + px) * 4;
                let a = image.rgba[s + 3] as u32 * alpha as u32 / 255;
                if a == 0 {
                    continue;
                }
                let d = (qy as usize * self.width + qx as usize) * 4;
                for c in 0..3 {
                    let src = image.rgba[s + c] as u32;
                    let dst = self.pixels[d + c] as u32;
                    self.pixels[d + c] = ((src * a + dst * (255 - a)) / 255) as u8;
                }
                self.pixels[d + 3] = 255;
            }
        }
    }

    /// Nombre en chiffres 3×5 (`dot` px par point), pour relier la carte à la légende.
    fn digits(&mut self, x: usize, y: usize, value: usize, dot: usize, color: [u8; 4]) {
        const GLYPHS: [[u8; 5]; 10] = [
            [0b111, 0b101, 0b101, 0b101, 0b111],
            [0b010, 0b110, 0b010, 0b010, 0b111],
            [0b111, 0b001, 0b111, 0b100, 0b111],
            [0b111, 0b001, 0b111, 0b001, 0b111],
            [0b101, 0b101, 0b111, 0b001, 0b001],
            [0b111, 0b100, 0b111, 0b001, 0b111],
            [0b111, 0b100, 0b111, 0b101, 0b111],
            [0b111, 0b001, 0b001, 0b001, 0b001],
            [0b111, 0b101, 0b111, 0b101, 0b111],
            [0b111, 0b101, 0b111, 0b001, 0b111],
        ];
        let text = value.to_string();
        for (k, c) in text.bytes().enumerate() {
            let glyph = GLYPHS[(c - b'0') as usize];
            let gx = x + k * 4 * dot;
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..3 {
                    if bits & (0b100 >> col) != 0 {
                        self.fill(gx + col * dot, y + row * dot, dot, dot, color);
                    }
                }
            }
        }
    }

    fn write(&self, path: &Path) {
        let file = File::create(path).unwrap_or_else(|e| panic!("{} : {e}", path.display()));
        let mut encoder =
            png::Encoder::new(BufWriter::new(file), self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("entête png");
        writer.write_image_data(&self.pixels).expect("données png");
    }
}

fn absolute(path: &Path) -> PathBuf {
    // Les chemins relatifs exportés doivent partir du dossier réel : sur macOS, /tmp est
    // un lien vers /private/tmp ; un simple nettoyage lexical produit des chemins invalides.
    let path = if path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        path
    };
    path.canonicalize()
        .unwrap_or_else(|e| panic!("{} : {e}", path.display()))
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn relative_to(from_dir: &Path, target: &Path) -> PathBuf {
    let from: Vec<_> = from_dir.components().collect();
    let to: Vec<_> = target.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in common..from.len() {
        out.push("..");
    }
    for component in &to[common..] {
        out.push(component.as_os_str());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs_ldtk::ldtk::NeighbourLevel;

    fn room(iid: &str, x: i32, side: Side, starts: &[usize]) -> Level {
        let mut layer = LayerInstance {
            identifier: map_const::LAYER_CONNECTION.into(),
            grid_size: 16,
            c_wid: 8,
            c_hei: 8,
            int_grid_csv: vec![0; 64],
            ..Default::default()
        };
        for &start in starts {
            for k in start..start + 2 {
                let index = match side {
                    Side::E => k * 8 + 7,
                    Side::W => k * 8,
                    _ => unreachable!(),
                };
                layer.int_grid_csv[index] = 1;
            }
        }
        Level {
            iid: iid.into(),
            world_x: x,
            world_y: -128,
            px_wid: 128,
            px_hei: 128,
            layer_instances: Some(vec![layer]),
            ..Default::default()
        }
    }

    #[test]
    fn two_openings_on_one_edge_do_not_share_connection_status() {
        let mut left = room("left", -128, Side::E, &[1, 5]);
        let right = room("right", 0, Side::W, &[1]);
        left.neighbours.push(NeighbourLevel {
            dir: "e".into(),
            level_iid: right.iid.clone(),
            ..Default::default()
        });
        let walls = LayerInstance {
            identifier: "Walls".into(),
            grid_size: 16,
            c_wid: 8,
            c_hei: 8,
            int_grid_csv: (0..64).map(|i| i32::from(i == 47 || i == 55)).collect(),
            ..Default::default()
        };
        left.layer_instances.as_mut().unwrap().push(walls);
        let data = LdtkJson {
            levels: vec![left, right],
            ..Default::default()
        };
        let all: Vec<_> = data.levels.iter().map(openings).collect();
        assert_eq!(all[0].len(), 2);
        assert!(opening_is_linked(&data.levels[0], &all[0][0], &data, &all));
        assert!(!opening_is_linked(&data.levels[0], &all[0][1], &data, &all));
        assert!(!opening_is_walled(&data.levels[0], &all[0][0]));
        assert!(opening_is_walled(&data.levels[0], &all[0][1]));
    }

    #[test]
    fn tiles_flip_blend_and_clip_at_canvas_edge() {
        let image = Image {
            width: 2,
            height: 2,
            rgba: vec![
                200, 0, 0, 255, 0, 200, 0, 255, 0, 0, 200, 255, 200, 200, 200, 128,
            ],
        };
        let mut canvas = Canvas::new(2, 2);
        canvas.fill(0, 0, 2, 2, [0, 0, 0, 255]);
        canvas.blit(&image, 0, 0, 2, -1, 0, true, false, 255);
        assert_eq!(&canvas.pixels[..4], &[200, 0, 0, 255]);
        assert_eq!(&canvas.pixels[8..12], &[0, 0, 200, 255]);
        canvas.blit(&image, 0, 0, 2, 0, 0, false, true, 255);
        assert_eq!(&canvas.pixels[..4], &[0, 0, 200, 255]);
        assert_eq!(&canvas.pixels[4..8], &[100, 100, 100, 255]);
        assert_eq!(&canvas.pixels[8..12], &[200, 0, 0, 255]);
    }
}

#[cfg(all(test, unix))]
mod export_tests {
    use super::*;
    use bevy_ecs_ldtk::ldtk::TilesetDefinition;
    use std::os::unix::fs::symlink;

    #[test]
    fn exported_tilesets_resolve_through_symlinked_output_directory() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let root = std::env::temp_dir().join(format!("alacod-map-export-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let _cleanup = Scratch(root.clone());
        let library_dir = root.join("library");
        std::fs::create_dir(&library_dir).unwrap();
        std::fs::create_dir_all(root.join("real/deep")).unwrap();
        symlink(root.join("real/deep"), root.join("alias")).unwrap();
        let atlas = library_dir.join("atlas.png");
        std::fs::write(&atlas, []).unwrap();
        let mut data = LdtkJson::default();
        data.defs.tilesets.push(TilesetDefinition {
            uid: 1,
            rel_path: Some("atlas.png".into()),
            ..Default::default()
        });
        data.levels.push(Level {
            layer_instances: Some(vec![LayerInstance {
                tileset_def_uid: Some(1),
                tileset_rel_path: Some("atlas.png".into()),
                ..Default::default()
            }]),
            ..Default::default()
        });
        let out = root.join("alias/map.ldtk");
        write_ldtk(
            &GeneratedPreview {
                ldtk: data,
                templates: Vec::new(),
            },
            &library_dir.join("library.ldtk"),
            &out,
        );
        let exported: LdtkJson = serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        let layer = &exported.levels[0].layer_instances.as_ref().unwrap()[0];
        for rel in [&exported.defs.tilesets[0].rel_path, &layer.tileset_rel_path] {
            let resolved = out
                .parent()
                .unwrap()
                .join(rel.as_ref().unwrap())
                .canonicalize()
                .unwrap();
            assert_eq!(resolved, atlas.canonicalize().unwrap());
        }
    }
}
