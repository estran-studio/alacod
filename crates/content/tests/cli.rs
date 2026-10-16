//! Tests de bout en bout de la CLI `alacod` (lint --json, describe, new, new-content, test) et du
//! gabarit `templates/starter`. Chaque test travaille dans un dossier temporaire : jamais dans
//! l'engine.
//!
//! Rapides (joués par `make test_crates`) : tout sauf `alacod test`, qui compile et lance
//! l'engine (`#[ignore]`, `make test_alacod_cli`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

fn alacod() -> Command {
    Command::new(env!("CARGO_BIN_EXE_alacod"))
}

fn engine_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Dossier temporaire unique, supprimé à la fin du test.
struct Tmp(PathBuf);

impl Tmp {
    fn new(label: &str) -> Self {
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "alacod-cli-{label}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir.canonicalize().unwrap())
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn run(mut cmd: Command) -> Output {
    cmd.output().expect("alacod se lance")
}

/// `alacod new <nom> --engine <engine>` dans `tmp`.
fn new_game(tmp: &Tmp, name: &str, extra: &[&str]) -> Output {
    let mut cmd = alacod();
    cmd.current_dir(&tmp.0)
        .args(["new", name, "--engine"])
        .arg(engine_root())
        .args(extra);
    run(cmd)
}

fn lint_json(dir: &Path) -> (bool, serde_json::Value) {
    let mut cmd = alacod();
    cmd.args(["lint", "--json"]).arg(dir);
    let out = run(cmd);
    let json = serde_json::from_str(&text(&out.stdout)).expect("lint --json donne du JSON");
    (out.status.success(), json)
}

// ---------------------------------------------------------------------------------------
// Gabarit
// ---------------------------------------------------------------------------------------

#[test]
fn starter_template_lints_clean() {
    // Garde-fou : si le format du contenu change, le gabarit doit suivre (docs/conventions.md §3).
    let (ok, json) = lint_json(&engine_root().join("templates/starter"));
    assert!(ok, "templates/starter ne passe plus le lint : {json}");
    assert_eq!(json["errors"], serde_json::json!([]));
}

#[test]
fn starter_template_has_no_license_registry_and_declares_a_test_arena() {
    let starter = engine_root().join("templates/starter");
    assert!(!starter.join("assets/assets.yaml").exists());
    let manifest = std::fs::read_to_string(starter.join("assets/game.ron")).unwrap();
    assert!(
        manifest.contains("generate_template"),
        "armes jouées dans le testbed sinon"
    );
}

// ---------------------------------------------------------------------------------------
// lint --json et découverte du jeu
// ---------------------------------------------------------------------------------------

#[test]
fn lint_json_reports_errors_with_file_and_exit_code() {
    let tmp = Tmp::new("lint");
    assert!(new_game(&tmp, "demo", &[]).status.success());
    let game = tmp.0.join("demo");
    let (ok, json) = lint_json(&game);
    assert!(ok);
    assert_eq!(json["game"], "demo");

    // Casse une valeur : l'erreur nomme le fichier et fait échouer la commande.
    let player = game.join("assets/ZombieShooter/Sprites/Character/player_config.ron");
    let broken =
        std::fs::read_to_string(&player)
            .unwrap()
            .replacen("max_speed: \"", "max_speed: \"-1", 1);
    std::fs::write(&player, broken).unwrap();
    let (ok, json) = lint_json(&game);
    assert!(
        !ok,
        "une erreur de contenu doit donner un code de sortie non nul"
    );
    let errors = json["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{json}");
    assert_eq!(errors[0]["kind"], "OutOfRange");
    assert!(errors[0]["file"]
        .as_str()
        .unwrap()
        .contains("player_config.ron"));
}

#[test]
fn lint_finds_the_game_from_a_subfolder_and_fails_outside_a_game() {
    let tmp = Tmp::new("discover");
    assert!(new_game(&tmp, "demo", &[]).status.success());

    for sub in ["demo", "demo/assets", "demo/assets/characters"] {
        let mut cmd = alacod();
        cmd.current_dir(tmp.0.join(sub)).arg("lint");
        let out = run(cmd);
        assert!(out.status.success(), "depuis {sub} : {}", text(&out.stderr));
    }

    let mut cmd = alacod();
    cmd.current_dir(&tmp.0).arg("lint");
    let out = run(cmd);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("aucun jeu alacod trouvé"));
}

// ---------------------------------------------------------------------------------------
// describe
// ---------------------------------------------------------------------------------------

#[test]
fn describe_lists_kinds_and_gives_the_exhaustive_schema() {
    let tmp = Tmp::new("describe");
    assert!(new_game(&tmp, "demo", &[]).status.success());
    let game = tmp.0.join("demo");

    let mut cmd = alacod();
    cmd.arg("describe").arg(&game);
    let overview = text(&run(cmd).stdout);
    for kind in ["Character", "Weapon", "Map"] {
        assert!(overview.contains(kind), "{overview}");
    }

    let mut cmd = alacod();
    cmd.args(["describe"]).arg(&game).arg("Weapon");
    let weapon = text(&run(cmd).stdout);
    assert!(
        weapon.contains("schéma (tous les champs possibles)"),
        "{weapon}"
    );
    assert!(weapon.contains("firing_rate"), "{weapon}");
    assert!(
        weapon.contains("lint : > 0"),
        "les plages du lint doivent apparaître :\n{weapon}"
    );
    assert!(weapon.contains("Fixed"), "{weapon}");

    // Sans chemin : `Item` n'est pas un dossier, donc un kind.
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["describe", "Character"]);
    assert!(text(&run(cmd).stdout).contains("asset_name_ref"));

    let mut cmd = alacod();
    cmd.arg("describe").arg(&game).args(["Weapon", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&text(&run(cmd).stdout)).unwrap();
    assert!(json["schema"]["$defs"].is_object() || json["schema"]["properties"].is_object());

    let mut cmd = alacod();
    cmd.arg("describe").arg(&game).arg("Inexistant");
    let out = run(cmd);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("absent de demo"));
}

// ---------------------------------------------------------------------------------------
// new
// ---------------------------------------------------------------------------------------

#[test]
fn new_creates_a_lint_clean_game_with_agent_config() {
    let tmp = Tmp::new("new");
    let out = new_game(&tmp, "mon-jeu", &[]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let game = tmp.0.join("mon-jeu");

    for file in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "src/main.rs",
        "CLAUDE.md",
        ".claude/settings.json",
        "assets/game.ron",
        "assets/maps/arena.ldtk",
    ] {
        assert!(game.join(file).exists(), "{file} manquant");
    }
    assert!(!game.join("assets/assets.yaml").exists());

    let cargo = std::fs::read_to_string(game.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("name = \"mon-jeu\""));
    assert!(cargo.contains(&format!(
        "path = \"{}/crates/game\"",
        engine_root().display()
    )));
    assert!(cargo.contains("[workspace]") && cargo.contains("[patch.crates-io]"));
    assert!(!cargo.contains("{{"), "placeholder non remplacé :\n{cargo}");

    for file in ["src/main.rs", "CLAUDE.md", "assets/game.ron"] {
        let text = std::fs::read_to_string(game.join(file)).unwrap();
        assert!(!text.contains("{{"), "{file} : placeholder non remplacé");
        assert!(
            !text.contains("testbed") || file == "assets/game.ron",
            "{file} cite le testbed"
        );
    }
    let manifest = std::fs::read_to_string(game.join("assets/game.ron")).unwrap();
    assert!(manifest.contains("name: \"mon-jeu\""));

    let (ok, json) = lint_json(&game);
    assert!(ok, "{json}");
}

#[test]
fn new_rejects_bad_names_and_existing_folders() {
    let tmp = Tmp::new("new-bad");
    for bad in ["Mon_Jeu", "1jeu", "a b", ""] {
        let out = new_game(&tmp, bad, &[]);
        assert!(!out.status.success(), "« {bad} » aurait dû être refusé");
    }
    assert!(new_game(&tmp, "demo", &[]).status.success());
    let again = new_game(&tmp, "demo", &[]);
    assert!(!again.status.success());
    assert!(text(&again.stderr).contains("existe déjà"));
}

#[test]
fn new_with_git_writes_git_dependencies_pinned_to_a_revision() {
    let tmp = Tmp::new("new-git");
    let out = new_game(&tmp, "demo", &["--git", "--rev", "abc1234"]);
    // L'engine de test est un dépôt git avec `origin` ; sans lui, la commande échoue proprement.
    if !out.status.success() {
        assert!(text(&out.stderr).contains("git"), "{}", text(&out.stderr));
        return;
    }
    let cargo = std::fs::read_to_string(tmp.0.join("demo/Cargo.toml")).unwrap();
    assert!(cargo.contains("git = \""), "{cargo}");
    assert!(cargo.contains("rev = \"abc1234\""), "{cargo}");
    assert!(
        !cargo.contains("path = \""),
        "aucune dépendance par chemin avec --git :\n{cargo}"
    );
}

// ---------------------------------------------------------------------------------------
// new-content
// ---------------------------------------------------------------------------------------

fn game_with_content(tmp: &Tmp) -> PathBuf {
    assert!(new_game(tmp, "demo", &[]).status.success());
    tmp.0.join("demo")
}

#[test]
fn new_content_copies_a_file_for_per_file_kinds() {
    let tmp = Tmp::new("nc-file");
    let game = game_with_content(&tmp);
    let mut cmd = alacod();
    cmd.current_dir(&game)
        .args(["new-content", "Character", "costaud", "--from", "dummy"]);
    let out = run(cmd);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );

    let created = std::fs::read_to_string(game.join("assets/characters/costaud.ron")).unwrap();
    assert!(
        created.contains("asset_name_ref: \"costaud\""),
        "l'id doit être réécrit"
    );
    assert!(created.starts_with("// Character « costaud »"));
    let (ok, json) = lint_json(&game);
    assert!(ok, "{json}");
}

#[test]
fn new_content_adds_an_entry_to_a_table_kind() {
    let tmp = Tmp::new("nc-table");
    let game = game_with_content(&tmp);
    let weapons = game.join("assets/ZombieShooter/Sprites/Character/weapons.ron");
    let before = std::fs::read_to_string(&weapons).unwrap();

    let mut cmd = alacod();
    cmd.current_dir(&game)
        .args(["new-content", "Weapon", "super_gun", "--from", "pistol"]);
    let out = run(cmd);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );

    let after = std::fs::read_to_string(&weapons).unwrap();
    assert!(after.contains("\"super_gun\": ("));
    assert!(after.starts_with(before.lines().next().unwrap()));
    assert_eq!(
        after.matches("\"pistol\": (").count(),
        1,
        "le modèle ne doit pas être touché"
    );
    assert!(after.contains("name: \"super_gun\""));
    let (ok, json) = lint_json(&game);
    assert!(ok, "{json}");

    // Le nouveau contenu est connu de `describe`.
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["describe", "Weapon"]);
    assert!(text(&run(cmd).stdout).contains("super_gun"));
}

#[test]
fn new_content_refuses_bad_requests_without_touching_files() {
    let tmp = Tmp::new("nc-bad");
    let game = game_with_content(&tmp);
    let snapshot = |dir: &Path| {
        let mut files = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    files.push((p.clone(), std::fs::metadata(&p).unwrap().len()));
                }
            }
        }
        files.sort();
        files
    };
    let before = snapshot(&game);

    for (args, needle) in [
        (vec!["Weapon", "pistol"], "existe déjà"),
        (vec!["Weapon", "Bad-Id"], "invalide"),
        (vec!["Item", "x"], "absent de demo"),
        (vec!["Map", "x"], "pas de création par modèle"),
        (vec!["Weapon", "x", "--from", "inconnu"], "inconnu"),
    ] {
        let mut cmd = alacod();
        cmd.current_dir(&game).arg("new-content").args(&args);
        let out = run(cmd);
        assert!(!out.status.success(), "{args:?} aurait dû échouer");
        assert!(
            text(&out.stderr).contains(needle),
            "{args:?} : {}",
            text(&out.stderr)
        );
    }
    assert_eq!(snapshot(&game), before, "un refus ne doit rien écrire");
}

// ---------------------------------------------------------------------------------------
// test (lent : compile et lance l'engine) — `make test_alacod_cli`
// ---------------------------------------------------------------------------------------

#[test]
#[ignore = "compile l'engine en profil headless : make test_alacod_cli"]
fn test_command_plays_generated_scenarios_of_a_game_outside_the_engine() {
    let tmp = Tmp::new("test-cmd");
    let game = game_with_content(&tmp);

    let mut cmd = alacod();
    cmd.current_dir(&game)
        .args(["new-content", "Weapon", "super_gun", "--from", "pistol"]);
    assert!(run(cmd).status.success());

    // Liste : l'arme nouvelle est testable.
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["test", "--list"]);
    let listed = text(&run(cmd).stdout);
    assert!(listed.contains("weapon_super_gun"), "{listed}");

    // Verdict vert, en JSON : une attente, jouée sur la carte du jeu, rien écrit dans l'engine.
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["test", "super_gun", "--json"]);
    let out = run(cmd);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let line = text(&out.stdout)
        .lines()
        .rev()
        .find(|l| l.starts_with('['))
        .unwrap()
        .to_string();
    let results: Vec<serde_json::Value> = serde_json::from_str(&line).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["expectations_ok"], true);
    assert_eq!(results[0]["expectations"], 1);
    assert!(results[0]["hits"].as_u64().unwrap() >= 1);
    assert!(game.join("tests/generated/weapon_super_gun.ron").exists());

    // Une arme qui ne fait aucun dégât échoue, avec un code de sortie non nul.
    let weapons = game.join("assets/ZombieShooter/Sprites/Character/weapons.ron");
    let text_ron = std::fs::read_to_string(&weapons).unwrap();
    let i = text_ron.find("\"super_gun\": (").unwrap();
    let (head, tail) = text_ron.split_at(i);
    let broken = format!("{head}{}", regex_free_replace_damage(tail));
    std::fs::write(&weapons, broken).unwrap();
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["test", "super_gun"]);
    let out = run(cmd);
    assert!(!out.status.success(), "{}", text(&out.stdout));
    assert!(text(&out.stdout).contains("ÉCHEC"));

    // Sujet inconnu : message qui liste les sujets, code de sortie non nul.
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["test", "n_existe_pas"]);
    let out = run(cmd);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("aucun sujet"));
}

/// Remplace la première valeur de `damage: "…"` de `text` par `"0.0"`.
fn regex_free_replace_damage(text: &str) -> String {
    let key = "damage: \"";
    let start = text.find(key).unwrap() + key.len();
    let end = start + text[start..].find('"').unwrap();
    format!("{}0.0{}", &text[..start], &text[end..])
}
