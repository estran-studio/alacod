//! Import `*.sprite.json` (X-4b) : manifests 1/4/5/8 directions, mapping des images,
//! lignes incohérentes, PNG incompatibles, fusion de `sprites.ron` et lint du jeu généré.
//!
//! Les PNG sont synthétiques et neutres (damier gris), écrits par un mini-encodeur local
//! (aucune dépendance) : aucune image d'un jeu réel n'entre dans le dépôt.

use content::sprite_import::{
    frame_index, import_sprite, SpriteImportError, SpriteImportOptions,
};
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alacod-sprite-import-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let mut a = 1u32;
    let mut b = 0u32;
    for &x in data {
        a = (a + x as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut mac = kind.to_vec();
    mac.extend_from_slice(data);
    out.extend_from_slice(&crc32(&mac).to_be_bytes());
}

/// PNG RGBA neutre `w×h` (damier gris, sans métadonnée) : assez pour la validation des
/// dimensions et le chargement par l'engine.
fn write_png(path: &Path, w: u32, h: u32) {
    let mut raw = Vec::new();
    for y in 0..h {
        raw.push(0);
        for x in 0..w {
            let v = if (x + y) % 2 == 0 { 200 } else { 80 };
            raw.extend_from_slice(&[v as u8, v as u8, v as u8, 255]);
        }
    }
    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
    let n = blocks.len();
    for (i, block) in blocks.iter().enumerate() {
        zlib.push(if i + 1 == n { 1 } else { 0 });
        zlib.extend_from_slice(&(block.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!block.len() as u16).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &zlib);
    chunk(&mut png, b"IEND", &[]);
    std::fs::write(path, png).unwrap();
}

fn clip_json(name: &str, file: &str, cols: u32, rows: u32, fps: &str, looping: bool) -> String {
    format!(
        "\"{name}\": ({{\"sheet\": \"{file}\", \"columns\": {cols}, \"rows\": {rows}, \"fps\": {fps}, \"loop\": {looping}}})"
    )
}

fn manifest_json(name: &str, cell: u32, dirs: Option<&[&str]>, clips: &[String]) -> String {
    let dirs = match dirs {
        None => "null".to_string(),
        Some(d) => format!(
            "[{}]",
            d.iter().map(|d| format!("\"{d}\"")).collect::<Vec<_>>().join(", ")
        ),
    };
    format!(
        "{{\"name\": \"{name}\", \"cell\": [{cell}, {cell}], \"directions\": {dirs}, \"animations\": {{{}}}}}",
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

fn non_start_map_errors(errors: &[content::LintError]) -> Vec<String> {
    errors
        .iter()
        .filter(|e| !e.message.contains("start_map"))
        .map(|e| format!("{:?}: {}", e.kind, e.message))
        .collect()
}

#[test]
fn import_eight_directions_writes_sheets_and_table() {
    let dir = scratch("eight");
    write_png(&dir.join("hero_idle.png"), 12 * 16, 8 * 16);
    write_png(&dir.join("hero_marche.png"), 12 * 16, 8 * 16);
    let manifest = manifest_json(
        "hero",
        16,
        Some(&["E", "NE", "N", "NW", "W", "SW", "S", "SE"]),
        &[
            clip_json("idle", "hero_idle.png", 12, 8, "6.0", true),
            clip_json("marche", "hero_marche.png", 12, 8, "12.0", false),
        ],
    );
    let manifest_path = dir.join("hero.sprite.json");
    std::fs::write(&manifest_path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");

    let result = import_sprite(&manifest_path, &game, SpriteImportOptions::default()).unwrap();
    assert_eq!(result.sprite_id, "hero");
    assert_eq!(result.table, "sprites/sprites.ron");
    assert_eq!(result.clips.len(), 2);

    let idle = result.clips.iter().find(|c| c.clip == "idle").unwrap();
    assert_eq!(idle.entry_id, "hero_idle");
    assert_eq!(idle.frames_per_row, 12);
    assert_eq!(idle.frame_duration_ms, 167);
    assert!(idle.looping);
    let marche = result.clips.iter().find(|c| c.clip == "marche").unwrap();
    assert_eq!(marche.frame_duration_ms, 83);
    assert!(!marche.looping);

    for clip in &result.clips {
        for rel in [&clip.sheet, &clip.animation, &clip.image] {
            assert!(
                game.join("assets").join(rel).exists(),
                "manquant : {rel}"
            );
        }
    }
    let sheet: String = std::fs::read_to_string(
        game.join("assets").join(&idle.sheet),
    )
    .unwrap();
    assert!(sheet.contains("tile_size: (16, 16)"), "{sheet}");
    assert!(sheet.contains("columns: 12"), "{sheet}");
    assert!(sheet.contains("rows: 8"), "{sheet}");
    assert!(
        sheet.contains("directions: Some((rows: [E, NE, N, NW, W, SW, S, SE]))"),
        "{sheet}"
    );
    assert!(sheet.contains("scale: 1.0"), "{sheet}");
    assert!(sheet.contains("anchor: Center"), "{sheet}");
    let anim: String = std::fs::read_to_string(
        game.join("assets").join(&idle.animation),
    )
    .unwrap();
    // L'animation couvre toute la ligne (fin = colonnes - 1) sous le nom du clip et `"Idle"`.
    assert!(anim.contains("\"idle\": (row: 0, end: 11)"), "{anim}");
    assert!(anim.contains("\"Idle\": (row: 0, end: 11)"), "{anim}");
    assert!(anim.contains("frame_duration: 167"), "{anim}");
    let marche_anim: String = std::fs::read_to_string(
        game.join("assets").join(&marche.animation),
    )
    .unwrap();
    assert!(marche_anim.contains("looping: Some(false)"), "{marche_anim}");

    let table: String =
        std::fs::read_to_string(game.join("assets").join("sprites/sprites.ron")).unwrap();
    assert!(table.contains("\"hero_idle\""), "{table}");
    assert!(table.contains("\"hero_marche\""), "{table}");
}

#[test]
fn import_supports_one_four_and_five_direction_manifests() {
    let dir = scratch("dirs");
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    // 1 direction : une ligne, miroir (comportement historique) ; 4 et 5 : une ligne chacune.
    for (id, dirs, cols) in [
        ("solo", vec!["E"], 4u32),
        ("quatre", vec!["E", "N", "W", "S"], 4u32),
        ("cinq", vec!["S", "SE", "E", "NE", "N"], 6u32),
    ] {
        let rows = dirs.len() as u32;
        write_png(&dir.join(format!("{id}.png")), cols * 8, rows * 8);
        let manifest = manifest_json(
            id,
            8,
            Some(&dirs),
            &[clip_json("idle", &format!("{id}.png"), cols, rows, "10.0", true)],
        );
        let path = dir.join(format!("{id}.sprite.json"));
        std::fs::write(&path, manifest).unwrap();
        let result = import_sprite(&path, &game, SpriteImportOptions::default()).unwrap();
        assert_eq!(result.clips.len(), 1);
        let sheet: String =
            std::fs::read_to_string(game.join("assets").join(&result.clips[0].sheet)).unwrap();
        let rows_ron = dirs.join(", ");
        assert!(
            sheet.contains(&format!("rows: [{}]", rows_ron)),
            "{id} : {sheet}"
        );
        assert!(sheet.contains("mirror: true"), "{id} : {sheet}");
    }
    let (_, _, errors) = content::load_and_lint(&game).unwrap();
    assert_eq!(non_start_map_errors(&errors), Vec::<String>::new());
}

#[test]
fn atlas_mapping_matches_direction_row_offsets() {
    // Ligne de direction × colonnes + image dans la ligne (cf. `DirectionRow`).
    assert_eq!(frame_index(12, 0, 0), 0);
    assert_eq!(frame_index(12, 3, 5), 41);
    assert_eq!(frame_index(12, 7, 11), 95);
}

#[test]
fn mismatched_rows_fail_without_artifacts() {
    let dir = scratch("rows-mismatch");
    write_png(&dir.join("x.png"), 12 * 16, 4 * 16);
    let manifest = manifest_json(
        "x",
        16,
        Some(&["E", "NE", "N", "NW", "W", "SW", "S", "SE"]),
        &[clip_json("idle", "x.png", 12, 4, "6.0", true)],
    );
    let path = dir.join("x.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let err = import_sprite(&path, &game, SpriteImportOptions::default()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ManifestInvalid { .. }),
        "{err:?}"
    );
    assert!(err.to_string().contains("une ligne par direction"), "{err}");
    assert!(!game.join("assets/sprites/x").exists());
}

#[test]
fn unknown_direction_fails_without_artifacts() {
    let dir = scratch("unknown-dir");
    write_png(&dir.join("x.png"), 16, 16);
    let manifest = manifest_json(
        "x",
        16,
        Some(&["E", "East"]),
        &[clip_json("idle", "x.png", 1, 2, "6.0", true)],
    );
    let path = dir.join("x.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let err = import_sprite(&path, &game, SpriteImportOptions::default()).unwrap_err();
    assert!(err.to_string().contains("East"), "{err}");
    assert!(!game.join("assets/sprites/x").exists());
}

#[test]
fn png_dimension_mismatch_fails_without_artifacts() {
    let dir = scratch("png-mismatch");
    write_png(&dir.join("x.png"), 6 * 16, 8 * 16);
    let manifest = manifest_json(
        "x",
        16,
        Some(&["E", "NE", "N", "NW", "W", "SW", "S", "SE"]),
        &[clip_json("idle", "x.png", 12, 8, "6.0", true)],
    );
    let path = dir.join("x.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let err = import_sprite(&path, &game, SpriteImportOptions::default()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ImageMismatch { .. }),
        "{err:?}"
    );
    assert!(err.to_string().contains("192×128"), "{err}");
    assert!(!game.join("assets/sprites/x").exists());
}

#[test]
fn malformed_manifest_and_missing_png_fail_cleanly() {
    let dir = scratch("malformed");
    let bad = dir.join("bad.sprite.json");
    std::fs::write(&bad, "{ pas du json").unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    let err = import_sprite(&bad, &game, SpriteImportOptions::default()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ManifestInvalid { .. }),
        "{err:?}"
    );
    let missing = dir.join("missing.sprite.json");
    std::fs::write(
        &missing,
        manifest_json("m", 16, None, &[clip_json("idle", "absent.png", 2, 1, "6.0", true)]),
    )
    .unwrap();
    let err = import_sprite(&missing, &game, SpriteImportOptions::default()).unwrap_err();
    assert!(
        matches!(err, SpriteImportError::ImageMismatch { .. }),
        "{err:?}"
    );
    assert!(!game.join("assets/sprites/m").exists());
}

#[test]
fn lint_of_generated_game_is_clean() {
    let dir = scratch("lint-clean");
    write_png(&dir.join("hero_idle.png"), 12 * 16, 8 * 16);
    let manifest = manifest_json(
        "hero",
        16,
        Some(&["E", "NE", "N", "NW", "W", "SW", "S", "SE"]),
        &[clip_json("idle", "hero_idle.png", 12, 8, "6.0", true)],
    );
    let path = dir.join("hero.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    import_sprite(&path, &game, SpriteImportOptions::default()).unwrap();
    let (registry, _, errors) = content::load_and_lint(&game).unwrap();
    assert_eq!(non_start_map_errors(&errors), Vec::<String>::new());
    let entry = &registry.sprite_sheets[&"hero_idle".into()];
    assert!(entry.directions.contains_key("body"));
    assert_eq!(entry.images["body"], "sprites/hero/hero_idle.png");
}

#[test]
fn merge_preserves_existing_table_entries_and_comments() {
    let dir = scratch("merge");
    let game = dir.join("jeu");
    // Entrée préexistante avec ses fichiers réels et un commentaire à garder.
    make_game(
        &game,
        "// gardé\n({\n    \"keep\": (\n        animation: \"sprites/keep_anim.ron\",\n        layers: { \"body\": \"sprites/keep_sheet.ron\" },\n    ),\n})\n",
    );
    let assets = game.join("assets");
    write_png(&assets.join("sprites/keep.png"), 8, 8);
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
    let manifest = manifest_json(
        "nouveau",
        16,
        Some(&["E", "NE", "N", "NW", "W", "SW", "S", "SE"]),
        // 8 directions exigent 8 lignes : 2 colonnes × 8 lignes = 32×128.
        &[clip_json("idle", "n.png", 2, 8, "6.0", true)],
    );
    write_png(&dir.join("n.png"), 2 * 16, 8 * 16);
    let path = dir.join("n.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    import_sprite(&path, &game, SpriteImportOptions::default()).unwrap();
    let table: String =
        std::fs::read_to_string(assets.join("sprites/sprites.ron")).unwrap();
    assert!(table.contains("// gardé"), "{table}");
    assert!(table.contains("\"keep\""), "{table}");
    assert!(table.contains("\"nouveau_idle\""), "{table}");
    let (_, _, errors) = content::load_and_lint(&game).unwrap();
    assert_eq!(non_start_map_errors(&errors), Vec::<String>::new());
}

#[test]
fn second_import_conflicts_unless_overwrite() {
    let dir = scratch("conflict");
    write_png(&dir.join("x.png"), 16, 16);
    let manifest = manifest_json("x", 16, Some(&["E"]), &[clip_json("idle", "x.png", 1, 1, "6.0", true)]);
    let path = dir.join("x.sprite.json");
    std::fs::write(&path, manifest).unwrap();
    let game = dir.join("jeu");
    make_game(&game, "({\n})\n");
    import_sprite(&path, &game, SpriteImportOptions::default()).unwrap();
    let err = import_sprite(&path, &game, SpriteImportOptions::default()).unwrap_err();
    assert!(matches!(err, SpriteImportError::Conflict { .. }), "{err:?}");
    let ok = import_sprite(
        &path,
        &game,
        SpriteImportOptions {
            overwrite: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(ok.clips.len(), 1);
}
