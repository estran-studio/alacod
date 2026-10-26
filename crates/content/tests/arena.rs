//! `alacod new-arena` (`docs/conventions.md` §89) : l'arène vide du bac à sable, générée à partir des
//! cartes du jeu lui-même. Murs et `PlayerSpawn`, rien d'autre ; lint vert ; sortie déterministe.

use content::arena::auto_layer_tiles;
use content::load_and_lint;
use content::manifest::GameManifest;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn engine_dir(rel: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(rel)
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} : {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{} : {e}", path.display()))
}

fn layer<'a>(level: &'a Value, name: &str) -> &'a Value {
    level["layerInstances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["__identifier"] == name)
        .unwrap_or_else(|| panic!("couche {name}"))
}

fn entity_kinds(project: &Value) -> BTreeSet<String> {
    project["levels"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|level| level["layerInstances"].as_array().unwrap())
        .flat_map(|layer| layer["entityInstances"].as_array().unwrap())
        .map(|entity| entity["__identifier"].as_str().unwrap().to_string())
        .collect()
}

/// Une arène vide : un niveau, une couronne de murs pleine et un intérieur libre, une tuile d'auto-règle par
/// case, seulement des `PlayerSpawn` (un à quatre, tous dans la salle), et des tilesets qui existent.
fn assert_empty_arena(file: &Path, label: &str) {
    let project = read_json(file);
    let levels = project["levels"].as_array().unwrap();
    assert_eq!(levels.len(), 1, "{label} : un seul niveau");
    let level = &levels[0];
    assert_eq!(
        level["fieldInstances"][0]["__identifier"], "spawn",
        "{label}"
    );
    assert_eq!(level["fieldInstances"][0]["__value"], true, "{label}");
    assert_eq!(
        entity_kinds(&project),
        BTreeSet::from(["PlayerSpawn".to_string()]),
        "{label} : rien d'autre que des points de départ"
    );
    let (w, h) = (
        level["pxWid"].as_i64().unwrap() as usize / 16,
        level["pxHei"].as_i64().unwrap() as usize / 16,
    );
    let walls = layer(level, "Walls");
    let cells = walls["intGridCsv"].as_array().unwrap();
    assert_eq!(cells.len(), w * h, "{label}");
    for y in 0..h {
        for x in 0..w {
            let edge = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            assert_eq!(
                cells[y * w + x].as_i64().unwrap(),
                i64::from(edge),
                "{label} : case ({x}, {y})"
            );
        }
    }
    assert_eq!(
        walls["autoLayerTiles"].as_array().unwrap().len(),
        w * h,
        "{label} : une tuile par case (sinon les murs ne se voient pas)"
    );
    assert!(
        layer(level, "LevelConnection")["intGridCsv"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c == 0),
        "{label} : aucune prise"
    );
    let spawns = layer(level, "Entities")["entityInstances"]
        .as_array()
        .unwrap();
    assert!((1..=4).contains(&spawns.len()), "{label}");
    let indexes: Vec<i64> = spawns
        .iter()
        .map(|s| s["fieldInstances"][0]["__value"].as_i64().unwrap())
        .collect();
    assert_eq!(
        indexes,
        (0..spawns.len() as i64).collect::<Vec<_>>(),
        "{label}"
    );
    for spawn in spawns {
        let (x, y) = (
            spawn["px"][0].as_i64().unwrap(),
            spawn["px"][1].as_i64().unwrap(),
        );
        assert!(
            x >= 32 && y >= 32 && x <= (w as i64 - 3) * 16 && y <= (h as i64 - 3) * 16,
            "{label} : point de départ ({x}, {y}) trop près d'un mur"
        );
    }
    let dir = file.parent().unwrap();
    for tileset in project["defs"]["tilesets"].as_array().unwrap() {
        let rel = tileset["relPath"].as_str().unwrap();
        assert!(
            dir.join(rel).is_file(),
            "{label} : tileset {rel} introuvable"
        );
    }
    assert_eq!(
        walls["__tilesetRelPath"], project["defs"]["tilesets"][0]["relPath"],
        "{label}"
    );
}

/// Les cinq jeux livrés et le gabarit ouvrent leur bac à sable sur une arène vide, déclarée dans le registre.
#[test]
fn les_arenes_livrees_sont_vides() {
    for game in [
        "games/zombies",
        "games/gungeon",
        "games/isaac",
        "games/throne",
        "games/testbed",
        "templates/starter",
    ] {
        let dir = engine_dir(game);
        let manifest = GameManifest::load(&dir).unwrap();
        let arena = manifest
            .sandbox
            .as_ref()
            .expect("sandbox.arena")
            .arena
            .clone();
        assert_empty_arena(&GameManifest::assets_dir(&dir).join(&arena), game);
        let (registry, _, errors) = load_and_lint(&dir).unwrap();
        assert!(errors.is_empty(), "{game} : {errors:#?}");
        let id = content::registry::map_id_from_path(&arena);
        assert!(
            registry.maps.contains_key(&id),
            "{game} : carte {id} non déclarée"
        );
    }
}

/// Les auto-règles rejouées par `auto_layer_tiles` redonnent exactement les tuiles que LDtk a écrites (même
/// ordre) pour les trois niveaux d'une carte de l'engine : c'est la preuve que les murs de l'arène sont ceux
/// que LDtk dessinerait.
#[test]
fn les_tuiles_d_auto_regles_sont_celles_de_ldtk() {
    let project = read_json(&engine_dir("games/zombies/assets/exemples/test_map.ldtk"));
    let defs = project["defs"]["layers"].as_array().unwrap();
    let walls_def = defs.iter().find(|l| l["identifier"] == "Walls").unwrap();
    let tilesets = project["defs"]["tilesets"].as_array().unwrap();
    for level in project["levels"].as_array().unwrap() {
        let walls = layer(level, "Walls");
        let cells: Vec<i64> = walls["intGridCsv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_i64().unwrap())
            .collect();
        let (w, h) = (
            walls["__cWid"].as_u64().unwrap() as usize,
            walls["__cHei"].as_u64().unwrap() as usize,
        );
        let got = auto_layer_tiles(walls_def, tilesets, &cells, w, h).unwrap();
        let expected = walls["autoLayerTiles"].as_array().unwrap();
        assert_eq!(&got, expected, "niveau {}", level["identifier"]);
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn alacod(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_alacod"))
        .arg("new-arena")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// Un jeu copié : `alacod new-arena` écrit l'arène à partir de ses cartes, la sortie est déterministe (une
/// deuxième exécution n'écrit rien), `--use` pose `sandbox.arena`, `--size` et `--players` sont respectés,
/// le lint reste vert.
#[test]
fn new_arena_ecrit_une_arene_vide_deterministe_et_lint_vert() {
    let dir = std::env::temp_dir().join(format!("alacod-arena-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&engine_dir("templates/starter"), &dir);
    // Le gabarit livre déjà une arène vide : repartir de la carte d'origine.
    std::fs::remove_file(dir.join("assets/maps/arene_vide.ldtk")).unwrap();
    let manifest_text = std::fs::read_to_string(dir.join("assets/game.ron")).unwrap();
    std::fs::write(
        dir.join("assets/game.ron"),
        manifest_text.replace("sandbox: (arena: \"maps/arene_vide.ldtk\"),", ""),
    )
    .unwrap();

    let (ok, output) = alacod(&dir, &["--use"]);
    assert!(ok, "{output}");
    let file = dir.join("assets/maps/arene_vide.ldtk");
    assert_empty_arena(&file, "arene_vide");
    let first = std::fs::read(&file).unwrap();
    assert!(
        std::fs::read_to_string(dir.join("assets/game.ron"))
            .unwrap()
            .contains("sandbox: (arena: \"maps/arene_vide.ldtk\")"),
        "--use pose l'arène du bac à sable"
    );
    // Le joueur 0 apparaît sur le point de départ des gabarits de l'engine (424, 696 en jeu).
    let project = read_json(&file);
    assert_eq!(
        layer(&project["levels"][0], "Entities")["entityInstances"][0]["px"],
        serde_json::json!([160, 80])
    );

    let (ok, output) = alacod(&dir, &["--use"]);
    assert!(ok, "{output}");
    assert!(output.contains("inchangée"), "{output}");
    assert_eq!(std::fs::read(&file).unwrap(), first, "sortie déterministe");

    let (ok, output) = alacod(
        &dir,
        &["--name", "petite", "--size", "30x20", "--players", "2"],
    );
    assert!(ok, "{output}");
    let small = dir.join("assets/maps/petite.ldtk");
    assert_empty_arena(&small, "petite");
    let project = read_json(&small);
    assert_eq!(project["levels"][0]["pxWid"], 480);
    assert_eq!(project["levels"][0]["pxHei"], 320);
    assert_eq!(
        layer(&project["levels"][0], "Entities")["entityInstances"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // Refus clairs.
    let (ok, output) = alacod(&dir, &["--size", "4x4"]);
    assert!(!ok && output.contains("hors bornes"), "{output}");
    let (ok, output) = alacod(&dir, &["--name", "a b"]);
    assert!(!ok && output.contains("invalide"), "{output}");
    let (ok, output) = alacod(&dir, &["--from", "maps/n_existe_pas.ldtk"]);
    assert!(!ok && output.contains("introuvable"), "{output}");

    let (_, _, errors) = load_and_lint(&dir).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    std::fs::remove_dir_all(&dir).unwrap();
}
