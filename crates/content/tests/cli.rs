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

/// `alacod remote <args>` sur `port`, tuée si elle dépasse `deadline` (échéance monotone : la commande a
/// elle-même un délai de requête de 10 s, qui ne doit pas dépasser le temps restant).
/// `Err` : le binaire ne se lance pas, ou échéance dépassée.
fn remote(port: u16, args: &[&str], deadline: Instant) -> Result<Output, String> {
    let mut child = alacod()
        .arg("remote")
        .args(args)
        .env("ALACOD_REMOTE_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("alacod ne se lance pas : {e}"))?;
    // Lecteurs dédiés : un tube plein ne doit pas bloquer la commande jusqu'à l'échéance.
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
                return Err(format!("alacod remote {args:?} : échéance dépassée"));
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

/// Le jeu lancé : tué et attendu au `Drop`, sur tout chemin (assertion, JSON invalide).
struct Game(std::process::Child);

impl Drop for Game {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// `alacod new demo`, compilation, `--sandbox` headless : le profil `arene` du gabarit ouvre l'arène vide
/// (`maps/arene_vide.ldtk`, §90) et pose le joueur **sans une ligne de code ajoutée** au jeu créé ; `alacod
/// remote` le pilote. Commande équivalente à la main : voir `docs/conventions.md` §88.
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
    assert_eq!(state["map"], "maps/arene_vide.ldtk");
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
    // L'arène est vide : le profil `arene` ne pose aucun ennemi (ni la carte, qui n'a que des `PlayerSpawn`).
    assert_eq!(
        state["enemies"].as_array().unwrap().len(),
        0,
        "{}",
        state["enemies"]
    );
    // ... et la cible se fait apparaître à la demande, 60 px devant le joueur.
    let spawned = remote(
        port,
        &["spawn", "target", "--ahead"],
        Instant::now() + Duration::from_secs(30),
    )
    .unwrap();
    assert!(
        spawned.status.success(),
        "{}{}",
        text(&spawned.stdout),
        text(&spawned.stderr)
    );
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
