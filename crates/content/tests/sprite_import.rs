//! Import `*.sprite.json` (X-4b, D-X4b-1) : atlas réduit composé à l'import, une entrée `sprites.ron`
//! par personnage. Manifestes à 1/4/5/8 directions, correspondance atlas/ligne (y compris un bloc hors de
//! la colonne 0), refus sans artefact, déterminisme du PNG, lint du jeu généré, fusion de la table et
//! réimport.
//!
//! Les PNG sont synthétiques et neutres : chaque case est un aplat de couleur qui dit d'où elle vient
//! (clip, ligne, colonne). Aucune image d'un jeu réel n'entre dans le dépôt.

use content::sprite_import::{import_sprite, ScaleSpec, SpriteImportError, SpriteImportOptions};
use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};

const EIGHT: [&str; 8] = ["E", "NE", "N", "NW", "W", "SW", "S", "SE"];

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alacod-sprite-import-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Couleur de la case (clip `k`, ligne `r`, colonne `c`) d'une planche source.
fn cell_color(k: u32, r: u32, c: u32) -> [u8; 4] {
    [10 + 20 * k as u8, 10 + 25 * r as u8, 10 + 15 * c as u8, 255]
}

/// Planche `columns × rows` de cases `cell × cell`, une couleur par case.
fn write_sheet(path: &Path, k: u32, columns: u32, rows: u32, cell: u32) {
    let mut img = RgbaImage::new(columns * cell, rows * cell);
    for (x, y, px) in img.enumerate_pixels_mut() {
        *px = Rgba(cell_color(k, y / cell, x / cell));
    }
    img.save(path).unwrap();
}

fn clip_json(name: &str, file: &str, cols: u32, rows: u32, fps: &str, looping: bool) -> String {
    format!(
        "\"{name}\": {{\"sheet\": \"{file}\", \"columns\": {cols}, \"rows\": {rows}, \"fps\": {fps}, \"loop\": {looping}}}"
    )
}

fn manifest_json(name: &str, cell: u32, dirs: Option<&[&str]>, clips: &[String]) -> String {
    let dirs = match dirs {
        None => "null".to_string(),
        Some(d) => format!(
            "[{}]",
            d.iter()
                .map(|d| format!("\"{d}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    format!(
        "{{\"name\": \"{name}\", \"cell\": [{cell}, {cell}], \"pivot\": [{}, {}], \"ppm\": 8.0, \"directions\": {dirs}, \"animations\": {{{}}}}}",
        cell as f64 / 2.0,
        cell as f64 * 0.75,
        clips.join(", ")
    )
}

fn make_game(dir: &Path, table: &str) {
    let assets = dir.join("assets");
    std::fs::create_dir_all(assets.join("sprites")).unwrap();
    std::fs::write(
        assets.join("game.ron"),
        "(\n    name: \"sprite-import-test\",\n    content_folders: [\n        (path: \"sprites/sprites.ron\", kind: \"SpriteSheet\"),\n    ],\n    entry: (\n        start_map: \"unused\",\n        default_seed: 1,\n    ),\n)\n",
    )
    .unwrap();
    std::fs::write(assets.join("sprites/sprites.ron"), table).unwrap();
}

fn options() -> SpriteImportOptions {
    SpriteImportOptions::new(ScaleSpec::UnitsPerMeter(8.0))
}

fn non_start_map_errors(errors: &[content::LintError]) -> Vec<String> {
    errors
        .iter()
        .filter(|e| !e.message.contains("start_map"))
        .map(|e| format!("{:?}: {}", e.kind, e.message))
        .collect()
}

/// Deux clips à huit directions, 4 colonnes, cellules de 8 px : le second bloc ne démarre pas à la
/// colonne 0 (atlas 2×1 blocs).
fn eight_direction_hero(dir: &Path) -> PathBuf {
    write_sheet(&dir.join("hero_idle.png"), 0, 4, 8, 8);
    write_sheet(&dir.join("hero_marche.png"), 1, 4, 8, 8);
    let manifest = manifest_json(
        "hero",
        8,
        Some(&EIGHT),
        &[
            clip_json("idle", "hero_idle.png", 4, 8, "4.0", true),
            clip_json("marche", "hero_marche.png", 4, 8, "8.0", false),
        ],
    );
    let path = dir.join("hero.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    path
}

#[test]
fn import_eight_directions_composes_one_atlas_and_one_entry() {
    let dir = scratch("eight");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");

    let mut opts = options();
    opts.states = vec![
        ("Idle".into(), "idle".into()),
        ("Run".into(), "marche".into()),
    ];
    let r = import_sprite(&manifest, &game, opts).unwrap();
    assert_eq!(r.sprite_id, "hero");
    assert_eq!(r.table, "sprites/sprites.ron");
    assert_eq!(r.factor, 1.0, "petite source : aucune réduction");
    assert_eq!(r.blocks, (2, 1));
    assert_eq!((r.columns, r.rows), (8, 8));
    assert_eq!(r.atlas_size, (64, 64));
    assert_eq!(r.cell, (8, 8));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);

    for rel in [&r.image, &r.sheet, &r.animation] {
        assert!(game.join("assets").join(rel).exists(), "manquant : {rel}");
    }
    let sheet = std::fs::read_to_string(game.join("assets").join(&r.sheet)).unwrap();
    assert!(sheet.contains("path: \"sprites/hero/hero.png\""), "{sheet}");
    assert!(sheet.contains("tile_size: (8, 8)"), "{sheet}");
    assert!(sheet.contains("columns: 8"), "{sheet}");
    assert!(sheet.contains("rows: 8"), "{sheet}");
    assert!(
        sheet.contains("rows: [E, NE, N, NW, W, SW, S, SE]"),
        "{sheet}"
    );
    assert!(sheet.contains("mirror: false"), "{sheet}");
    assert!(sheet.contains("anchor: Center"), "{sheet}");
    assert!(sheet.contains("name: \"body\""), "{sheet}");

    // Le premier clip (ordre alphabétique : idle) est au bloc 0, le second au bloc suivant (colonne 4).
    let anim = std::fs::read_to_string(game.join("assets").join(&r.animation)).unwrap();
    assert!(anim.contains("columns: Some(8)"), "{anim}");
    assert!(anim.contains("\"idle\": (start: 0, end: 3)"), "{anim}");
    assert!(anim.contains("\"Idle\": (start: 0, end: 3)"), "{anim}");
    assert!(anim.contains("\"marche\": (start: 4, end: 7)"), "{anim}");
    assert!(anim.contains("\"Run\": (start: 4, end: 7)"), "{anim}");
    // Durée et boucle par clip, tirées de `fps` et de `loop` (4 fps = 250 ms, 8 fps = 125 ms).
    assert!(
        anim.contains("\"idle\": (frame_duration: Some(250), looping: Some(true))"),
        "{anim}"
    );
    assert!(
        anim.contains("\"Run\": (frame_duration: Some(125), looping: Some(false))"),
        "{anim}"
    );
    assert!(
        anim.contains("frame_duration: 250,"),
        "global = `Idle` : {anim}"
    );

    let table = std::fs::read_to_string(game.join("assets/sprites/sprites.ron")).unwrap();
    assert!(table.contains("\"hero\""), "{table}");
    assert!(table.contains("sprites/hero/hero_animation.ron"), "{table}");
    assert_eq!(
        table.matches("hero_sheet.ron").count(),
        1,
        "une seule entrée : {table}"
    );
    // Aucun clip abandonné : chaque clip est joué sous son nom.
    let names: Vec<_> = r.clips.iter().map(|c| c.clip.as_str()).collect();
    assert_eq!(names, ["idle", "marche"]);
}

#[test]
fn import_supports_one_four_and_five_direction_manifests() {
    let dir = scratch("dirs");
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    for (id, dirs, cols) in [
        ("solo", vec!["E"], 4u32),
        ("quatre", vec!["E", "N", "W", "S"], 4u32),
        ("cinq", vec!["S", "SE", "E", "NE", "N"], 6u32),
    ] {
        let rows = dirs.len() as u32;
        write_sheet(&dir.join(format!("{id}.png")), 0, cols, rows, 8);
        let manifest = manifest_json(
            id,
            8,
            Some(&dirs),
            &[clip_json(
                "idle",
                &format!("{id}.png"),
                cols,
                rows,
                "10.0",
                true,
            )],
        );
        let path = dir.join(format!("{id}.sprite.json"));
        std::fs::write(&path, manifest).unwrap();
        let mut opts = options();
        opts.states = vec![("Idle".into(), "idle".into())];
        let r = import_sprite(&path, &game, opts).unwrap();
        assert_eq!(r.rows, rows, "{id} : un bloc de {rows} ligne(s)");
        let sheet = std::fs::read_to_string(game.join("assets").join(&r.sheet)).unwrap();
        assert!(
            sheet.contains(&format!("rows: [{}]", dirs.join(", "))),
            "{id} : {sheet}"
        );
        assert!(sheet.contains("mirror: true"), "{id} : {sheet}");
    }
    let (_, _, errors) = content::load_and_lint(&game).unwrap();
    assert_eq!(non_start_map_errors(&errors), Vec::<String>::new());
}

#[test]
fn atlas_cells_land_in_their_block_even_off_column_zero() {
    let dir = scratch("mapping");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let r = import_sprite(&manifest, &game, options()).unwrap();
    let atlas = image::open(game.join("assets").join(&r.image))
        .unwrap()
        .to_rgba8();
    assert_eq!(atlas.dimensions(), r.atlas_size);
    let off_zero = r
        .clips
        .iter()
        .find(|c| c.block_column > 0)
        .expect("un bloc hors colonne 0");
    assert_eq!(off_zero.clip, "marche");
    let (cw, ch) = r.cell;
    for (k, clip) in r.clips.iter().enumerate() {
        for row in 0..8u32 {
            for col in 0..4u32 {
                // Même arithmétique que le runtime : image d'animation + `ligne × columns`.
                let index = clip.start as u32 + col + row * r.columns;
                let (x, y) = ((index % r.columns) * cw, (index / r.columns) * ch);
                assert_eq!(
                    atlas.get_pixel(x + cw / 2, y + ch / 2).0,
                    cell_color(k as u32, row, col),
                    "clip {} ligne {row} image {col} (index {index})",
                    clip.clip
                );
            }
        }
    }
}

#[test]
fn reduction_keeps_colors_and_fits_under_max_side() {
    let dir = scratch("reduce");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let mut opts = options();
    opts.max_side = 32; // 2 blocs de 4×8 cases : 64 px de large à pleine taille, 32 réduits de moitié.
    let r = import_sprite(&manifest, &game, opts).unwrap();
    assert!(r.factor < 1.0 && r.factor > 0.0, "{}", r.factor);
    assert!(
        r.atlas_size.0 <= 32 && r.atlas_size.1 <= 32,
        "{:?}",
        r.atlas_size
    );
    let atlas = image::open(game.join("assets").join(&r.image))
        .unwrap()
        .to_rgba8();
    let (cw, ch) = r.cell;
    let clip = &r.clips[1];
    let index = clip.start as u32 + 2 + 3 * r.columns;
    let (x, y) = ((index % r.columns) * cw, (index / r.columns) * ch);
    let got = atlas.get_pixel(x + cw / 2, y + ch / 2).0;
    let want = cell_color(1, 3, 2);
    for c in 0..4 {
        assert!(got[c].abs_diff(want[c]) <= 2, "{got:?} != {want:?}");
    }
    // `scale` rend à l'écran la taille voulue : (U / ppm) / facteur.
    assert!(
        (r.scale as f64 - 1.0 / r.factor).abs() < 1e-4,
        "{} / {}",
        r.scale,
        r.factor
    );
}

#[test]
fn png_is_identical_byte_for_byte_when_reimported() {
    let dir = scratch("determinisme");
    let manifest = eight_direction_hero(&dir);
    let mut opts = options();
    opts.max_side = 40;
    let mut atlases = Vec::new();
    for name in ["a", "b"] {
        let game = dir.join(name);
        make_game(&game, "({\n})\n");
        let r = import_sprite(&manifest, &game, opts.clone()).unwrap();
        atlases.push(std::fs::read(game.join("assets").join(&r.image)).unwrap());
    }
    assert_eq!(
        atlases[0], atlases[1],
        "même import, même PNG, à l'octet près"
    );
}

#[test]
fn scale_and_pivot_are_read_from_the_manifest_and_reported() {
    let dir = scratch("scale");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    // ppm 8, U = 16 → 2 unités par pixel source ; pivot (4, 6) sur une case de 8 : dx = 0, dy = +2 px bas.
    let mut opts = SpriteImportOptions::new(ScaleSpec::UnitsPerMeter(16.0));
    opts.states = vec![("Idle".into(), "idle".into())];
    let r = import_sprite(&manifest, &game, opts).unwrap();
    assert_eq!(r.scale, 2.0);
    assert_eq!(
        r.offset,
        (0.0, 4.0),
        "pivot sous le centre : le sprite monte"
    );
    // Silhouette : cases pleines de 8 px de haut, 2 unités par pixel.
    assert_eq!(r.silhouette_source_px, 8);
    assert_eq!(r.silhouette_units, 16.0);
    let sheet = std::fs::read_to_string(game.join("assets").join(&r.sheet)).unwrap();
    assert!(sheet.contains("scale: 2.0"), "{sheet}");
    assert!(sheet.contains("offset_y: 4.0"), "{sheet}");
    // `--scale` direct : posé tel quel.
    let game2 = dir.join("jeu2");
    make_game(&game2, "({\n})\n");
    let direct = import_sprite(
        &manifest,
        &game2,
        SpriteImportOptions::new(ScaleSpec::Scale(0.5)),
    )
    .unwrap();
    assert_eq!(direct.scale, 0.5);
}

#[test]
fn state_mapping_rejects_unknown_clips_and_warns_without_idle() {
    let dir = scratch("states");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let mut opts = options();
    opts.states = vec![("Run".into(), "absent".into())];
    let err = import_sprite(&manifest, &game, opts).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::InvalidOption { .. }),
        "{err:?}"
    );
    assert!(!game.join("assets/sprites/hero").exists());
    let r = import_sprite(&manifest, &game, options()).unwrap();
    assert!(
        r.warnings.iter().any(|w| w.contains("Idle")),
        "{:?}",
        r.warnings
    );
}

#[test]
fn mismatched_rows_fail_without_artifacts() {
    let dir = scratch("rows-mismatch");
    write_sheet(&dir.join("x.png"), 0, 4, 4, 8);
    let manifest = manifest_json(
        "x",
        8,
        Some(&EIGHT),
        &[clip_json("idle", "x.png", 4, 4, "6.0", true)],
    );
    let path = dir.join("x.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let table_before = std::fs::read_to_string(game.join("assets/sprites/sprites.ron")).unwrap();
    let err = import_sprite(&path, &game, options()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ManifestInvalid { .. }),
        "{err:?}"
    );
    assert!(err.to_string().contains("une ligne par direction"), "{err}");
    assert!(!game.join("assets/sprites/x").exists());
    assert_eq!(
        std::fs::read_to_string(game.join("assets/sprites/sprites.ron")).unwrap(),
        table_before
    );
}

#[test]
fn unknown_direction_fails_without_artifacts() {
    let dir = scratch("unknown-dir");
    write_sheet(&dir.join("x.png"), 0, 1, 2, 8);
    let manifest = manifest_json(
        "x",
        8,
        Some(&["E", "East"]),
        &[clip_json("idle", "x.png", 1, 2, "6.0", true)],
    );
    let path = dir.join("x.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let err = import_sprite(&path, &game, options()).unwrap_err();
    assert!(err.to_string().contains("East"), "{err}");
    assert!(!game.join("assets/sprites/x").exists());
}

#[test]
fn png_dimension_mismatch_fails_without_artifacts() {
    let dir = scratch("png-mismatch");
    write_sheet(&dir.join("x.png"), 0, 2, 8, 8);
    let manifest = manifest_json(
        "x",
        8,
        Some(&EIGHT),
        &[clip_json("idle", "x.png", 4, 8, "6.0", true)],
    );
    let path = dir.join("x.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let err = import_sprite(&path, &game, options()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ImageMismatch { .. }),
        "{err:?}"
    );
    assert!(err.to_string().contains("16×64"), "{err}");
    assert!(!game.join("assets/sprites/x").exists());
}

#[test]
fn malformed_manifest_and_missing_png_fail_cleanly() {
    let dir = scratch("malformed");
    let bad = dir.join("bad.sprite.json");
    std::fs::write(&bad, "{ pas du json").unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let err = import_sprite(&bad, &game, options()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ManifestInvalid { .. }),
        "{err:?}"
    );
    let missing = dir.join("missing.sprite.json");
    std::fs::write(
        &missing,
        manifest_json(
            "m",
            8,
            None,
            &[clip_json("idle", "absent.png", 2, 1, "6.0", true)],
        ),
    )
    .unwrap();
    let err = import_sprite(&missing, &game, options()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ImageMismatch { .. }),
        "{err:?}"
    );
    assert!(!game.join("assets/sprites/m").exists());
}

#[test]
fn impossible_atlas_fails_without_artifacts() {
    let dir = scratch("atlas-impossible");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let mut opts = options();
    opts.max_side = 4; // 8 lignes de directions dans 4 px : une case de 0 px.
    let err = import_sprite(&manifest, &game, opts).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::AtlasImpossible { .. }),
        "{err:?}"
    );
    assert!(!game.join("assets/sprites/hero").exists());
    // Hauteur de cellule imposée plus grande que la source : refus, pas d'agrandissement.
    let mut opts = options();
    opts.cell_height = Some(64);
    let err = import_sprite(&manifest, &game, opts).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::AtlasImpossible { .. }),
        "{err:?}"
    );
    assert!(!game.join("assets/sprites/hero").exists());
}

#[test]
fn lint_of_generated_game_is_clean() {
    let dir = scratch("lint-clean");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let mut opts = options();
    opts.states = vec![("Idle".into(), "idle".into())];
    import_sprite(&manifest, &game, opts).unwrap();
    let (registry, _, errors) = content::load_and_lint(&game).unwrap();
    assert_eq!(non_start_map_errors(&errors), Vec::<String>::new());
    let entry = &registry.sprite_sheets[&"hero".into()];
    assert!(entry.directions.contains_key("body"));
    assert_eq!(entry.images["body"], "sprites/hero/hero.png");
}

#[test]
fn merge_preserves_existing_table_entries_and_comments() {
    let dir = scratch("merge");
    let game = dir.join("jeu");
    make_game(
        &game,
        "// gardé\n({\n    \"keep\": (\n        animation: \"sprites/keep_anim.ron\",\n        layers: { \"body\": \"sprites/keep_sheet.ron\" },\n    ),\n})\n",
    );
    let assets = game.join("assets");
    write_sheet(&assets.join("sprites/keep.png"), 0, 1, 1, 8);
    std::fs::write(
        assets.join("sprites/keep_sheet.ron"),
        "(path: \"sprites/keep.png\", tile_size: (8, 8), columns: 1, rows: 1, animated: false, name: \"body\", scale: 1.0, anchor: Center, offset_x: 0.0, offset_y: 0.0, offset_z: 0.0)",
    )
    .unwrap();
    std::fs::write(
        assets.join("sprites/keep_anim.ron"),
        "(frame_duration: 100, animations: {\"Idle\": (row: 0, end: 0)})",
    )
    .unwrap();
    let manifest = eight_direction_hero(&dir);
    import_sprite(&manifest, &game, options()).unwrap();
    let table = std::fs::read_to_string(assets.join("sprites/sprites.ron")).unwrap();
    assert!(table.contains("// gardé"), "{table}");
    assert!(table.contains("\"keep\""), "{table}");
    assert!(table.contains("\"hero\""), "{table}");
    let (_, _, errors) = content::load_and_lint(&game).unwrap();
    assert_eq!(non_start_map_errors(&errors), Vec::<String>::new());
}

#[test]
fn second_import_conflicts_unless_overwrite() {
    let dir = scratch("conflict");
    let manifest = eight_direction_hero(&dir);
    let game = dir.join("jeu");
    make_game(&game, "// gardé\n({\n})\n");
    import_sprite(&manifest, &game, options()).unwrap();
    let table = std::fs::read_to_string(game.join("assets/sprites/sprites.ron")).unwrap();
    let err = import_sprite(&manifest, &game, options()).unwrap_err();
    assert!(matches!(err, SpriteImportError::Conflict { .. }), "{err:?}");
    assert_eq!(
        std::fs::read_to_string(game.join("assets/sprites/sprites.ron")).unwrap(),
        table,
        "le refus ne touche à rien"
    );
    let mut over = options();
    over.overwrite = true;
    let ok = import_sprite(&manifest, &game, over).unwrap();
    assert_eq!(ok.clips.len(), 2);
    let after = std::fs::read_to_string(game.join("assets/sprites/sprites.ron")).unwrap();
    assert_eq!(
        after.matches("\"hero\"").count(),
        1,
        "remplacée, pas dupliquée : {after}"
    );
    assert!(after.contains("// gardé"), "{after}");
}
