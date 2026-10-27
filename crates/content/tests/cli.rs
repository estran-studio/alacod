//! Tests de bout en bout de la CLI `alacod` (lint --json, describe, new, new-content, test) et du
//! gabarit `templates/starter`. Chaque test travaille dans un dossier temporaire : jamais dans
//! l'engine.
//!
//! Rapides (joués par `make test_crates`) : tout sauf `alacod test`, qui compile et lance
//! l'engine (`#[ignore]`, `make test_alacod_cli`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

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

// ---------------------------------------------------------------------------------------
// bac à sable d'un jeu créé par `alacod new` (SB-4) — `make test_alacod_cli`
// ---------------------------------------------------------------------------------------

/// Dossier `target` de la construction du jeu créé : `CARGO_TARGET_DIR` (rendu absolu : cargo
/// tourne dans le dossier du jeu) ou le `target/` du jeu. Partager celui de l'engine évite de
/// recompiler bevy et les crates de l'engine pour chaque jeu d'essai.
fn game_target_dir(game: &Path) -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) => std::env::current_dir().unwrap().join(dir),
        None => game.join("target"),
    }
}

/// Un port local libre (la fenêtre entre la libération et la réutilisation est négligeable ici).
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// `scripts/alacod-remote <args>` sur `port`, tuée si elle dépasse `deadline` (échéance monotone :
/// le script a lui-même un délai de requête de 10 s, qui ne doit pas dépasser le temps restant).
/// `Err` : python absent, ou échéance dépassée.
fn remote(port: u16, args: &[&str], deadline: Instant) -> Result<Output, String> {
    let mut child = Command::new("python3")
        .arg(engine_root().join("scripts/alacod-remote"))
        .args(args)
        .env("ALACOD_REMOTE_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("python3 ne se lance pas : {e}"))?;
    // Lecteurs dédiés : un tube plein ne doit pas bloquer le script jusqu'à l'échéance.
    fn drain<R: std::io::Read + Send + 'static>(mut r: R) -> std::thread::JoinHandle<Vec<u8>> {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = r.read_to_end(&mut buf);
            buf
        })
    }
    let out = drain(child.stdout.take().unwrap());
    let err = drain(child.stderr.take().unwrap());
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("alacod-remote {args:?} : échéance dépassée"));
            }
            Err(e) => return Err(e.to_string()),
        }
    };
    Ok(Output {
        status,
        stdout: out.join().unwrap_or_default(),
        stderr: err.join().unwrap_or_default(),
    })
}

/// Le jeu lancé : tué et attendu au `Drop`, sur tout chemin (assertion, JSON invalide, python absent).
struct Game(std::process::Child);

impl Drop for Game {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// `alacod new demo`, compilation, `--sandbox` headless : le profil `arene` du gabarit pose le
/// joueur et la cible **sans une ligne de code ajoutée** au jeu créé. Commande équivalente à la
/// main : voir `docs/conventions.md` §88.
#[test]
#[ignore = "compile un jeu créé hors de l'engine (≈ 10 min à froid) : make test_alacod_cli"]
fn new_game_opens_its_sandbox_without_any_added_code() {
    let tmp = Tmp::new("sandbox-e2e");
    let out = new_game(&tmp, "demo", &[]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let game = tmp.0.join("demo");

    // Le jeu créé ne contient que du gabarit : un seul fichier Rust, qui ne dit pas « sandbox ».
    let main = std::fs::read_to_string(game.join("src/main.rs")).unwrap();
    assert!(
        !main.to_lowercase().contains("sandbox"),
        "main.rs parle de sandbox"
    );
    let rust_files: Vec<_> = std::fs::read_dir(game.join("src"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(rust_files, ["main.rs"]);

    // Compilation headless : le profil `headless` de l'engine, redéclaré en ligne de commande
    // (le jeu est son propre workspace et ne le porte pas).
    let target = game_target_dir(&game);
    let build = Command::new("cargo")
        .current_dir(&game)
        .env("CARGO_TARGET_DIR", &target)
        .args([
            "build",
            "--no-default-features",
            "--features",
            "native",
            "--profile",
            "headless",
            "--config",
            "profile.headless.inherits=\"dev\"",
            "--config",
            "profile.headless.opt-level=1",
            "--config",
            "profile.headless.debug=\"line-tables-only\"",
            "--config",
            "profile.headless.package.\"*\".opt-level=3",
        ])
        .output()
        .expect("cargo se lance");
    assert!(build.status.success(), "{}", text(&build.stderr));

    // `--sandbox` **sans autre option** (`--local-port` et `--players` ont leurs valeurs par défaut) :
    // arène du gabarit, profil `arene`, frames comptées par la trace d'état (`ALACOD_EXIT_AT_FRAME`
    // exige `ALACOD_STATE_TRACE`), le jeu s'arrête de lui-même. `Game` le tue au `Drop`.
    let port = free_port();
    let trace = tmp.0.join("trace.txt");
    let mut game_proc = Game(
        Command::new(target.join("headless/demo"))
            .current_dir(&game)
            .arg("--sandbox")
            .env("ALACOD_HEADLESS", "1")
            .env("ALACOD_REMOTE_PORT", port.to_string())
            .env("ALACOD_STATE_TRACE", &trace)
            .env("ALACOD_EXIT_AT_FRAME", "1200")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("le jeu créé se lance"),
    );

    // Attend une frame jouée (échéance totale de 60 s), puis lit l'état.
    let ready_deadline = Instant::now() + Duration::from_secs(60);
    let state: serde_json::Value = loop {
        if let Some(status) = game_proc.0.try_wait().unwrap() {
            panic!("le jeu s'est arrêté trop tôt : {status}");
        }
        assert!(
            Instant::now() < ready_deadline,
            "aucune frame jouée en 60 s"
        );
        let out = remote(port, &["state"], ready_deadline).unwrap();
        if out.status.success() {
            let state: serde_json::Value = serde_json::from_slice(&out.stdout)
                .unwrap_or_else(|e| panic!("état illisible ({e}) : {}", text(&out.stdout)));
            if state["frame"].as_u64().unwrap_or(0) >= 300 {
                break state;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    };
    let query_deadline = Instant::now() + Duration::from_secs(30);
    let profiles = text(&remote(port, &["profile"], query_deadline).unwrap().stdout);

    assert_eq!(state["sandbox"]["profile"], "arene");
    assert_eq!(state["editor"], true);
    assert_eq!(state["online"], false);
    assert_eq!(state["check_distance"], 0);
    assert_eq!(state["game"], "demo");
    assert_eq!(state["map"], "maps/arena.ldtk");
    assert_eq!(state["wave"]["phase"], "NotStarted");
    assert_eq!(state["wave"]["number"], 0);
    assert!(profiles.contains("arene"), "{profiles}");
    // Un seul joueur par défaut, posé sur le `PlayerSpawn` de l'arène, avec l'arme du personnage.
    assert_eq!(
        state["players"].as_array().unwrap().len(),
        1,
        "{}",
        state["players"]
    );
    let player = &state["players"][0];
    assert_eq!(
        (player["x"].as_f64(), player["y"].as_f64()),
        (Some(424.0), Some(696.0))
    );
    assert!(player["weapon"]["name"].is_string(), "{player}");
    // La cible du profil (`characters` à (624, 696), 200 unités à droite du joueur).
    let target_posed = state["enemies"].as_array().unwrap().iter().any(|e| {
        e["config"] == "target" && e["x"].as_f64() == Some(624.0) && e["y"].as_f64() == Some(696.0)
    });
    assert!(target_posed, "{}", state["enemies"]);
    // Le réglage du profil `arene` (`currency: 1000`) : `alacod/state` n'expose pas le solde,
    // `give --currency 0` ne change rien et renvoie le solde.
    let given = remote(port, &["give", "--currency", "0"], query_deadline).unwrap();
    assert!(
        given.status.success(),
        "{}{}",
        text(&given.stdout),
        text(&given.stderr)
    );
    let given: serde_json::Value = serde_json::from_slice(&given.stdout)
        .unwrap_or_else(|e| panic!("réponse de give illisible ({e})"));
    assert_eq!(given["currency"], 1000, "{given}");

    // Le jeu quitte seul à la frame demandée : code 0 et trace écrite (échéance de 120 s).
    let exit_deadline = Instant::now() + Duration::from_secs(120);
    let status = loop {
        if let Some(status) = game_proc.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < exit_deadline,
            "le jeu ne s'est pas arrêté à la frame 1200"
        );
        std::thread::sleep(Duration::from_millis(200));
    };
    assert!(status.success(), "{status}");
    assert!(trace.exists());
}

// ---------------------------------------------------------------------------------------
// plugin Rust d'un jeu externe (X-1, docs/conventions.md §89)
// ---------------------------------------------------------------------------------------

/// Jeu de la preuve : `alacod new plugdemo`, puis le plugin, le contenu et le scénario de
/// `tests/fixtures/plugin_game` posés par-dessus (le gabarit fournit le reste : carte, armes, sprites).
fn plugin_game(tmp: &Tmp) -> PathBuf {
    let out = new_game(tmp, "plugdemo", &[]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let game = tmp.0.join("plugdemo");
    fn copy_over(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_over(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    copy_over(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/plugin_game"),
        &game,
    );
    game
}

fn error_kinds(json: &serde_json::Value) -> Vec<(String, String)> {
    json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["kind"].as_str().unwrap().to_string(),
                e["message"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// `alacod lint` (binaire sans le code du jeu) connaît le vocabulaire du plugin par `game.ron` : le jeu
/// qui l'utilise passe, tout `Custom` ou kind non déclaré est une erreur, et les règles Rust du plugin,
/// que seul son binaire exécute, sont signalées en avertissement.
#[test]
fn alacod_lint_knows_the_plugin_vocabulary_from_game_ron() {
    let tmp = Tmp::new("plugin-lint");
    let game = plugin_game(&tmp);

    let (ok, json) = lint_json(&game);
    assert!(ok, "{json}");
    assert_eq!(json["errors"], serde_json::json!([]));
    let warnings = json["warnings"].to_string();
    assert!(
        warnings.contains("PluginRulesNotRun") && warnings.contains("crate_weights"),
        "{warnings}"
    );

    // Un id retiré de la déclaration : chacun des trois verbes est refusé, nommé.
    let manifest = game.join("assets/game.ron");
    let original = std::fs::read_to_string(&manifest).unwrap();
    for (declared, name, what) in [
        ("behaviors: [\"rush\"]", "rush", "behavior Custom"),
        (
            "actions: [\"grant_bounty\"]",
            "grant_bounty",
            "action Custom",
        ),
        (
            "conditions: [\"after_frame\"]",
            "after_frame",
            "condition Custom",
        ),
    ] {
        let key = declared.split(':').next().unwrap();
        std::fs::write(&manifest, original.replace(declared, &format!("{key}: []"))).unwrap();
        let (ok, json) = lint_json(&game);
        assert!(!ok, "{what} non déclaré accepté");
        assert!(
            error_kinds(&json).iter().any(|(kind, message)| {
                kind == "UndeclaredCustom" && message.contains(name) && message.contains(what)
            }),
            "{what} : {json}"
        );
    }

    // Un kind de contenu que ni l'engine ni le plugin ne déclarent.
    std::fs::write(
        &manifest,
        original.replace("kinds: [\"Crate\"]", "kinds: []"),
    )
    .unwrap();
    let (ok, json) = lint_json(&game);
    assert!(!ok);
    assert!(
        error_kinds(&json)
            .iter()
            .any(|(kind, message)| kind == "UnknownKind" && message.contains("Crate")),
        "{json}"
    );

    // Un kind du plugin qui reprend le nom d'un kind de l'engine.
    std::fs::write(
        &manifest,
        original.replace("kinds: [\"Crate\"]", "kinds: [\"Crate\", \"Item\"]"),
    )
    .unwrap();
    let (ok, json) = lint_json(&game);
    assert!(!ok);
    assert!(
        error_kinds(&json)
            .iter()
            .any(|(kind, message)| kind == "PluginMismatch" && message.contains("Item")),
        "{json}"
    );
}

/// Un `Custom` hors d'un effet (power-up, frise de boss) est refusé : seuls les effets de personnage et
/// d'objet le portent (le plugin lit `EffectOut::Custom`).
#[test]
fn custom_action_is_refused_outside_effects() {
    let tmp = Tmp::new("plugin-power-up");
    let game = plugin_game(&tmp);
    let powerups = game.join("assets/economy/powerups.ron");
    std::fs::write(
        &powerups,
        r#"(
    drop_chance: "0.0",
    powerups: {
        "bounty": (
            name: "Prime",
            weight: 1,
            pickup_range: "30.0",
            lifetime_frames: 60,
            actions: [Custom(id: "grant_bounty", params: {"crate": "coffre"})],
        ),
    },
)
"#,
    )
    .unwrap();
    let manifest = game.join("assets/game.ron");
    let text_ron = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        &manifest,
        text_ron.replace(
            "(path: \"crates\", kind: \"Crate\"),",
            "(path: \"crates\", kind: \"Crate\"),\n        (path: \"economy/powerups.ron\", kind: \"PowerUp\"),",
        ),
    )
    .unwrap();
    let (ok, json) = lint_json(&game);
    assert!(!ok, "{json}");
    assert!(
        error_kinds(&json)
            .iter()
            .any(|(_, message)| message.contains("Custom réservée aux effets")),
        "{json}"
    );
}

/// Le jeu de la preuve de bout en bout : son binaire (plugin compris) lint le contenu avec la règle du
/// plugin, génère ses scénarios, joue son scénario écrit à la main avec sa trace, et refuse un contenu
/// que sa règle de lint rejette. Compile l'engine et le jeu (≈ 10 min à froid).
#[test]
#[ignore = "compile un jeu créé hors de l'engine (≈ 10 min à froid) : make test_alacod_cli"]
fn plugin_game_runs_its_scenario_with_its_own_binary() {
    let tmp = Tmp::new("plugin-e2e");
    let game = plugin_game(&tmp);

    let target = game_target_dir(&game);
    let build = Command::new("cargo")
        .current_dir(&game)
        .env("CARGO_TARGET_DIR", &target)
        .args([
            "build",
            "--no-default-features",
            "--features",
            "native",
            "--profile",
            "headless",
            "--config",
            "profile.headless.inherits=\"dev\"",
            "--config",
            "profile.headless.opt-level=1",
            "--config",
            "profile.headless.debug=\"line-tables-only\"",
            "--config",
            "profile.headless.package.\"*\".opt-level=3",
        ])
        .output()
        .expect("cargo se lance");
    assert!(build.status.success(), "{}", text(&build.stderr));
    let bin = target.join("headless/plugdemo");
    let play = |args: &[&str]| {
        Command::new(&bin)
            .current_dir(&game)
            .env("ALACOD_HEADLESS", "1")
            .args(args)
            .output()
            .expect("le jeu se lance")
    };

    // Lint par le binaire du jeu : la règle du plugin s'exécute (plus d'avertissement « non exécutées »).
    let out = play(&["lint", "--json"]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["errors"], serde_json::json!([]));
    assert!(
        !json["warnings"].to_string().contains("PluginRulesNotRun"),
        "{json}"
    );

    // Le scénario du jeu, joué par son binaire : attentes vertes (le plugin agit), trace écrite puis relue.
    let out = play(&["scenarios", "play", "--bless", "--json"]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let results: serde_json::Value = serde_json::from_slice(
        text(&out.stdout)
            .lines()
            .rev()
            .find(|l| l.starts_with('['))
            .expect("tableau JSON")
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(results[0]["scenario"], "plugin");
    assert_eq!(results[0]["expectations_ok"], true, "{results}");
    assert_eq!(results[0]["expectations"], 4);
    let trace = std::fs::read_to_string(game.join("tests/scenarios/plugin.trace")).unwrap();
    assert_eq!(
        trace.lines().count(),
        199,
        "une ligne par frame confirmée (frames - 1)"
    );
    let out = play(&["scenarios", "play", "--json"]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(text(&out.stdout).contains("\"trace\":\"ok\""));

    // Les scénarios générés de l'engine passent par le même binaire (`gen`), plugin compris.
    let out = play(&[
        "scenarios",
        "gen",
        "--play",
        "--no-trace",
        "--only",
        "pistol",
    ]);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains("weapon_pistol"),
        "{}",
        text(&out.stdout)
    );
    assert!(game.join("tests/generated/weapon_pistol.ron").exists());

    // `--sandbox` démarre pour de bon avec le plugin : le profil `arene` du jeu donne l'objet `prime` au joueur ; sa condition
    // `after_frame` (plugin) devient vraie puis l'action `grant_bounty` (plugin) s'exécute, ce que prouve sa ligne de trace
    // `ggrs{f=… grant_bounty carrier=… weight=3}` ; le jeu s'arrête de lui-même (`ALACOD_EXIT_AT_FRAME`) avec le code 0.
    let port = free_port();
    let sandbox_log = game.join("sandbox.log");
    let mut sandbox = Game(
        Command::new(&bin)
            .current_dir(&game)
            .arg("--sandbox")
            .env("ALACOD_HEADLESS", "1")
            .env("ALACOD_MUTE", "1")
            .env("ALACOD_REMOTE_PORT", port.to_string())
            .env("ALACOD_STATE_TRACE", game.join("sandbox-trace.txt"))
            .env("ALACOD_EXIT_AT_FRAME", "300")
            .stdout(std::fs::File::create(&sandbox_log).unwrap())
            .stderr(Stdio::from(
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&sandbox_log)
                    .unwrap(),
            ))
            .spawn()
            .expect("le jeu se lance en bac à sable"),
    );
    let deadline = Instant::now() + Duration::from_secs(180);
    let status = loop {
        if let Some(status) = sandbox.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "le bac à sable ne s'arrête pas en 180 s");
        std::thread::sleep(Duration::from_millis(500));
    };
    let log = std::fs::read_to_string(&sandbox_log).unwrap_or_default();
    assert!(
        status.success(),
        "bac à sable : {status}\n{}",
        log.lines().rev().take(20).collect::<Vec<_>>().join("\n")
    );
    assert!(
        log.lines()
            .any(|l| l.contains("grant_bounty carrier=") && l.contains("weight=3}")),
        "l'action Custom du plugin ne s'est pas exécutée en bac à sable"
    );

    // Un poids hors d'un `Fixed` (32768 > 32767) est refusé par la règle Rust (il ferait déborder `Fixed::from_num`).
    std::fs::write(game.join("assets/crates/coffre.ron"), "(weight: 32768)").unwrap();
    let out = play(&["lint"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("tenir dans un Fixed"),
        "{}",
        text(&out.stderr)
    );
    let out = play(&["--sandbox"]);
    assert!(!out.status.success(), "la partie démarre sur un poids hors Fixed");

    // Sa règle de lint (Rust) refuse un contenu que `alacod lint` accepte.
    std::fs::write(game.join("assets/crates/coffre.ron"), "(weight: 0)").unwrap();
    let (ok, _) = lint_json(&game);
    assert!(ok, "alacod lint ne connaît pas la règle Rust du plugin");
    let out = play(&["lint"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("weight doit être >= 1"),
        "{}",
        text(&out.stderr)
    );
    // Et la partie refuse de démarrer sur ce contenu.
    let out = play(&["--sandbox"]);
    assert!(!out.status.success());
}
