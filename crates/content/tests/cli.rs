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

/// `alacod new <nom> --engine <engine> --path` dans `tmp` : dépendance par chemin, hors ligne (le défaut,
/// un tag git, demande le réseau au premier build).
fn new_game(tmp: &Tmp, name: &str, extra: &[&str]) -> Output {
    let mut cmd = alacod();
    cmd.current_dir(&tmp.0)
        .args(["new", name, "--engine"])
        .arg(engine_root())
        .arg("--path")
        .args(extra);
    run(cmd)
}

/// Un faux dépôt d'engine : le gabarit de l'engine (lien symbolique) dans un dépôt git à `origin` donné,
/// avec les tags `tags` sur son seul commit. `alacod new --engine <lui>` n'a besoin que de cela.
fn fake_engine(tmp: &Tmp, tags: &[&str]) -> PathBuf {
    let dir = tmp.0.join("fake-engine");
    std::fs::create_dir_all(&dir).unwrap();
    std::os::unix::fs::symlink(engine_root().join("templates"), dir.join("templates")).unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .current_dir(&dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?} : {}", text(&out.stderr));
    };
    git(&["init", "-q"]);
    git(&["remote", "add", "origin", "https://example.org/engine.git"]);
    git(&["commit", "-q", "--allow-empty", "-m", "engine"]);
    for tag in tags {
        git(&["tag", tag]);
    }
    dir
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

/// Par défaut le jeu dépend de l'engine par le **dernier tag** de son dépôt (ordre de version : `v0.10.0`
/// après `v0.9.0`), jamais par un chemin absolu.
#[test]
fn new_pins_the_latest_engine_tag_by_default() {
    let tmp = Tmp::new("new-tag");
    let engine = fake_engine(&tmp, &["v0.9.0", "v0.10.0", "v0.2.1"]);
    let mut cmd = alacod();
    cmd.current_dir(&tmp.0)
        .args(["new", "demo", "--engine"])
        .arg(&engine);
    let out = run(cmd);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let cargo = std::fs::read_to_string(tmp.0.join("demo/Cargo.toml")).unwrap();
    assert!(
        cargo.contains("content = { git = \"https://example.org/engine.git\", tag = \"v0.10.0\" }"),
        "{cargo}"
    );
    assert!(
        !cargo.contains("path = \""),
        "aucun chemin par défaut :\n{cargo}"
    );
    assert!(!cargo.contains("{{"), "{cargo}");
    assert!(
        text(&out.stdout).contains("tag v0.10.0"),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn new_accepts_an_explicit_tag_rev_and_url_and_refuses_an_engine_without_tags() {
    let tmp = Tmp::new("new-ref");
    let engine = fake_engine(&tmp, &["v1.0.0"]);
    let new = |name: &str, extra: &[&str]| {
        let mut cmd = alacod();
        cmd.current_dir(&tmp.0)
            .args(["new", name, "--engine"])
            .arg(&engine)
            .args(extra);
        run(cmd)
    };
    let out = new("a", &["--rev", "abc1234", "--url", "file:///x/engine"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let cargo = std::fs::read_to_string(tmp.0.join("a/Cargo.toml")).unwrap();
    assert!(
        cargo.contains("git = \"file:///x/engine\", rev = \"abc1234\""),
        "{cargo}"
    );
    let out = new("b", &["--tag", "v7"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let cargo = std::fs::read_to_string(tmp.0.join("b/Cargo.toml")).unwrap();
    assert!(cargo.contains("tag = \"v7\""), "{cargo}");
    // Une dépendance par chemin et un tag s'excluent.
    assert!(!new("c", &["--path", "--tag", "v1.0.0"]).status.success());

    let bare = Tmp::new("new-no-tag");
    let engine = fake_engine(&bare, &[]);
    let mut cmd = alacod();
    cmd.current_dir(&bare.0)
        .args(["new", "demo", "--engine"])
        .arg(&engine);
    let out = run(cmd);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("aucun tag"),
        "{}",
        text(&out.stderr)
    );
    assert!(!bare.0.join("demo").exists());
}

// ---------------------------------------------------------------------------------------
// engine local / pin / status
// ---------------------------------------------------------------------------------------

/// Un jeu qui dépend d'un dépôt git `file://` (avec un tag), comme celui d'un `alacod new` publié.
fn game_on_git_tag(tmp: &Tmp) -> (PathBuf, String) {
    let engine = fake_engine(tmp, &["v1.0.0", "v1.1.0"]);
    let url = format!("file://{}", engine.display());
    let mut cmd = alacod();
    cmd.current_dir(&tmp.0)
        .args(["new", "demo", "--tag", "v1.0.0", "--url", &url, "--engine"])
        .arg(&engine);
    assert!(run(cmd).status.success());
    (tmp.0.join("demo"), url)
}

#[test]
fn engine_pin_switches_tag_or_rev_and_keeps_other_dependency_keys() {
    let tmp = Tmp::new("engine-pin");
    let (game, url) = game_on_git_tag(&tmp);
    let cargo_path = game.join("Cargo.toml");
    let before = std::fs::read_to_string(&cargo_path).unwrap();
    assert!(
        before.contains(&format!("git = \"{url}\", tag = \"v1.0.0\"")),
        "{before}"
    );

    let pin = |reference: &str| {
        let mut cmd = alacod();
        cmd.current_dir(&game).args(["engine", "pin", reference]);
        run(cmd)
    };
    let out = pin("v1.1.0");
    assert!(out.status.success(), "{}", text(&out.stderr));
    let after = std::fs::read_to_string(&cargo_path).unwrap();
    assert_eq!(after.matches("tag = \"v1.1.0\"").count(), 7, "{after}");
    assert!(!after.contains("v1.0.0"), "{after}");
    // Seules les dépendances de l'engine bougent.
    assert_eq!(before.replace("v1.0.0", "v1.1.0"), after);

    // Une révision (pas un tag du dépôt) : `rev`.
    let head = text(
        &Command::new("git")
            .current_dir(tmp.0.join("fake-engine"))
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    );
    let out = pin(head.trim());
    assert!(out.status.success(), "{}", text(&out.stderr));
    let after = std::fs::read_to_string(&cargo_path).unwrap();
    assert!(
        after.contains(&format!("rev = \"{}\"", head.trim())),
        "{after}"
    );
    assert!(!after.contains("tag = "), "{after}");

    // Ni tag ni révision : refus, rien d'écrit.
    let snapshot = std::fs::read_to_string(&cargo_path).unwrap();
    let out = pin("v9.9.9");
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("ni un tag"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(std::fs::read_to_string(&cargo_path).unwrap(), snapshot);
}

/// `engine local` ne touche pas au `Cargo.toml` versionné : la surcharge est dans `.cargo/config.toml`,
/// ignoré par git ; `pin` la retire.
#[test]
fn engine_local_patches_outside_the_versioned_manifest_and_pin_removes_it() {
    let tmp = Tmp::new("engine-local");
    let (game, url) = game_on_git_tag(&tmp);
    let manifest = std::fs::read(game.join("Cargo.toml")).unwrap();

    let mut cmd = alacod();
    cmd.current_dir(&game)
        .args(["engine", "local"])
        .arg(engine_root());
    let out = run(cmd);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read(game.join("Cargo.toml")).unwrap(),
        manifest,
        "Cargo.toml versionné inchangé"
    );
    let config = std::fs::read_to_string(game.join(".cargo/config.toml")).unwrap();
    assert!(config.contains(&format!("[patch.\"{url}\"]")), "{config}");
    for krate in ["content", "game", "map_ldtk", "combat", "sim_core"] {
        assert!(
            config.contains(&format!(
                "{krate} = {{ path = \"{}/crates/{krate}\" }}",
                engine_root().display()
            )),
            "{krate} manquant :\n{config}"
        );
    }
    let ignore = std::fs::read_to_string(game.join(".gitignore")).unwrap();
    assert!(
        ignore.lines().any(|l| l == "/.cargo/config.toml"),
        "{ignore}"
    );

    // `status` voit la surcharge ; la racine compilée n'est pas demandée à un réseau absent.
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["engine", "status"]);
    let status = text(&run(cmd).stdout);
    assert!(
        status.contains(&format!("git {url} tag v1.0.0")),
        "{status}"
    );
    assert!(
        status.contains(&format!("surcharge locale : {}", engine_root().display())),
        "{status}"
    );

    // Un dossier qui n'est pas un engine est refusé.
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["engine", "local"]).arg(&tmp.0);
    assert!(!run(cmd).status.success());

    let mut cmd = alacod();
    cmd.current_dir(&game).args(["engine", "pin", "v1.1.0"]);
    let out = run(cmd);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        !game.join(".cargo").exists(),
        "la surcharge est retirée avec son dossier"
    );
    assert!(
        text(&out.stdout).contains("surcharge locale retirée"),
        "{}",
        text(&out.stdout)
    );
}

/// La racine de l'engine d'un jeu se lit dans ce que **cargo résout** (ici une dépendance par chemin) :
/// ni `BUILT_FROM_ENGINE` ni heuristique.
#[test]
fn engine_status_resolves_the_root_the_game_really_compiles() {
    let tmp = Tmp::new("engine-root");
    assert!(new_game(&tmp, "demo", &[]).status.success());
    let mut cmd = alacod();
    cmd.current_dir(tmp.0.join("demo"))
        .args(["engine", "status"]);
    let out = run(cmd);
    let status = text(&out.stdout);
    assert!(out.status.success(), "{}{}", status, text(&out.stderr));
    assert!(
        status.contains(&format!("dépendance : chemin {}", engine_root().display())),
        "{status}"
    );
    assert!(status.contains("surcharge locale : aucune"), "{status}");
    assert!(
        status.contains(&format!("racine compilée : {}", engine_root().display())),
        "{status}"
    );
}

// ---------------------------------------------------------------------------------------
// run (cargo simulé : seuls les arguments et l'environnement comptent)
// ---------------------------------------------------------------------------------------

/// Un `cargo` qui affiche son dossier courant, ses arguments et `ALACOD_HEADLESS`, placé en tête du PATH.
fn fake_cargo(tmp: &Tmp) -> std::ffi::OsString {
    use std::os::unix::fs::PermissionsExt;
    let bin = tmp.0.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let script = bin.join("cargo");
    std::fs::write(
        &script,
        "#!/bin/sh\necho \"cwd=$(basename \"$PWD\")\"\necho \"args=$*\"\necho \"headless=$ALACOD_HEADLESS\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut path = std::ffi::OsString::from(&bin);
    path.push(":");
    path.push(std::env::var_os("PATH").unwrap_or_default());
    path
}

#[test]
fn run_launches_cargo_in_the_game_folder_with_the_game_arguments() {
    let tmp = Tmp::new("run");
    assert!(new_game(&tmp, "demo", &[]).status.success());
    let path = fake_cargo(&tmp);

    let mut cmd = alacod();
    cmd.env("PATH", &path)
        .env_remove("ALACOD_HEADLESS")
        .current_dir(tmp.0.join("demo/assets"))
        .args([
            "run",
            "--sandbox",
            "lourd",
            "--headless",
            "--seed",
            "-7",
            "--players",
            "2",
            "--",
            "--debug-ai",
        ]);
    let out = run(cmd);
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{stdout}{}", text(&out.stderr));
    assert!(stdout.contains("cwd=demo"), "{stdout}");
    assert!(stdout.contains("headless=1"), "{stdout}");
    let args = stdout
        .lines()
        .find_map(|l| l.strip_prefix("args="))
        .unwrap();
    assert!(
        args.starts_with("run --no-default-features --features native --profile headless"),
        "{args}"
    );
    assert!(
        args.ends_with("-- --sandbox lourd --seed=-7 --players localhost localhost --debug-ai"),
        "{args}"
    );

    // Sans option : le profil `arene` est celui du jeu (`--sandbox` seul), rien d'autre.
    let mut cmd = alacod();
    cmd.env("PATH", &path)
        .env_remove("ALACOD_HEADLESS")
        .current_dir(tmp.0.join("demo"))
        .args(["run", "--sandbox"]);
    let stdout = text(&run(cmd).stdout);
    assert!(
        stdout.contains("args=run --features native -- --sandbox arene"),
        "{stdout}"
    );
    assert!(stdout.contains("headless=\n"), "{stdout}");

    let mut cmd = alacod();
    cmd.env("PATH", &path)
        .env_remove("ALACOD_HEADLESS")
        .current_dir(tmp.0.join("demo"))
        .args(["run", "--players", "0"]);
    assert!(!run(cmd).status.success());
}

// ---------------------------------------------------------------------------------------
// remote (faux jeu : un serveur JSON-RPC local)
// ---------------------------------------------------------------------------------------

/// Un faux remote : répond `handler(méthode, paramètres)` à chaque requête et garde `(méthode, paramètres)`.
struct FakeRemote {
    port: u16,
    seen: std::sync::Arc<std::sync::Mutex<Vec<(String, serde_json::Value)>>>,
}

impl FakeRemote {
    fn start(
        handler: impl Fn(&str, &serde_json::Value, usize) -> serde_json::Value + Send + 'static,
    ) -> Self {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = seen.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
                let mut raw = Vec::new();
                let mut chunk = [0u8; 4096];
                // Requête complète : en-têtes, puis `Content-Length` octets de corps.
                let body = loop {
                    let n = stream.read(&mut chunk).unwrap_or(0);
                    if n == 0 {
                        break String::new();
                    }
                    raw.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&raw).into_owned();
                    if let Some((head, body)) = text.split_once("\r\n\r\n") {
                        let length: usize = head
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse().unwrap())
                            })
                            .unwrap_or(0);
                        if body.len() >= length {
                            break body.to_string();
                        }
                    }
                };
                let request: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                let method = request["method"].as_str().unwrap_or("").to_string();
                let calls = log.lock().unwrap().len();
                let result = handler(&method, &request["params"], calls);
                log.lock()
                    .unwrap()
                    .push((method, request["params"].clone()));
                let reply =
                    serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": result}).to_string();
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                    reply.len()
                );
            }
        });
        Self { port, seen }
    }

    fn remote(&self, args: &[&str]) -> Output {
        let mut cmd = alacod();
        cmd.arg("remote")
            .args(["--port", &self.port.to_string()])
            .args(args);
        run(cmd)
    }

    fn calls(&self) -> Vec<(String, serde_json::Value)> {
        self.seen.lock().unwrap().clone()
    }
}

#[test]
fn remote_speaks_the_alacod_methods_of_the_game() {
    let fake = FakeRemote::start(|method, _params, calls| match method {
        "alacod/give" => serde_json::json!({"handle": 0, "currency": 1000}),
        "alacod/step" => serde_json::json!({"target_frame": 3}),
        // La partie avance d'une frame par lecture d'état.
        "alacod/state" => serde_json::json!({
            "frame": calls, "clock": {"paused": false}, "wave": null, "enemies": [], "windows": [],
            "players": [{"handle": 0, "x": 10.0, "y": 20.0, "facing": "Up", "health": 4, "max_health": 5,
                         "weapon": {"name": "pistol", "ammo": 1, "reserve": 2, "reloading": false}}]
        }),
        "alacod/sandbox_profile" => serde_json::json!({
            "active": "arene",
            "profiles": [{"id": "arene", "map": "maps/arena.ldtk", "players": [{"currency": 1000, "items": []}], "characters": ["target"]}]
        }),
        "alacod/reload_content" => serde_json::json!({"errors": ["boum"]}),
        _ => serde_json::json!({"ok": true}),
    });

    let out = fake.remote(&[
        "give",
        "--weapon",
        "shotgun",
        "--currency",
        "-5",
        "--handle",
        "1",
    ]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("\"currency\":1000"),
        "{}",
        text(&out.stdout)
    );
    assert_eq!(
        fake.calls()[0],
        (
            "alacod/give".to_string(),
            serde_json::json!({"handle": 1, "weapon": "shotgun", "currency": -5})
        )
    );
    // Rien à donner : refus avant tout appel.
    let before = fake.calls().len();
    assert!(!fake.remote(&["give"]).status.success());
    assert_eq!(fake.calls().len(), before);

    // `step` avance, puis attend la frame cible en lisant l'état, et affiche le résumé.
    let out = fake.remote(&["step", "3"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("joueur 0 (10.0,20.0) pv 4/5 arme pistol 1x2"),
        "{stdout}"
    );
    assert!(fake
        .calls()
        .iter()
        .any(|(m, p)| m == "alacod/step" && p == &serde_json::json!({"frames": 3})));

    // `spawn --ahead` : devant le joueur, dans son `facing` (Up : +y).
    let out = fake.remote(&["spawn", "zombie", "--ahead", "30", "--team", "enemies"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let (method, params) = fake
        .calls()
        .into_iter()
        .rfind(|(m, _)| m == "alacod/spawn_character")
        .unwrap();
    assert_eq!(method, "alacod/spawn_character");
    assert_eq!(
        params,
        serde_json::json!({"x": 10.0, "y": 50.0, "character": "zombie", "team": "enemies"})
    );
    let out = fake.remote(&["spawn-item", "bomb", "--at", "1,2"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        fake.calls().last().unwrap().1,
        serde_json::json!({"x": 1.0, "y": 2.0, "id": "bomb"})
    );
    assert!(
        !fake.remote(&["spawn", "zombie"]).status.success(),
        "ni --at ni --ahead"
    );

    let out = fake.remote(&["profile"]);
    assert!(
        text(&out.stdout)
            .contains("* arene  carte=maps/arena.ldtk  joueurs=[currency=1000]  posés=[target]"),
        "{}",
        text(&out.stdout)
    );
    let out = fake.remote(&[
        "modifier",
        "MoveSpeed",
        "Mul",
        "0.9",
        "--handle",
        "0",
        "--until",
        "60",
    ]);
    assert!(out.status.success());
    assert_eq!(
        fake.calls().last().unwrap(),
        &(
            "alacod/set_modifier".to_string(),
            serde_json::json!({"stat": "MoveSpeed", "op": "Mul", "value": "0.9", "source": "editor", "handle": 0, "until_frames": 60})
        )
    );
    let out = fake.remote(&["input", "Up,Fire", "--pan", "100,0"]);
    assert!(out.status.success());
    assert_eq!(
        fake.calls().last().unwrap(),
        &(
            "alacod/input".to_string(),
            serde_json::json!({"handle": 0, "buttons": ["Up", "Fire"], "pan": [100, 0]})
        )
    );
    // Des erreurs de rechargement : le JSON du jeu reste sur stdout, code de sortie non nul
    // (comme `scripts/alacod-remote reload`).
    let out = fake.remote(&["reload"]);
    assert!(!out.status.success());
    assert_eq!(
        text(&out.stdout).trim(),
        r#"{"errors":["boum"]}"#,
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn remote_reload_succeeds_with_the_json_on_stdout_when_the_content_is_valid() {
    let fake = FakeRemote::start(|_, _, _| serde_json::json!({"errors": [], "reloaded": 3}));
    let out = fake.remote(&["reload"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(text(&out.stdout).trim(), r#"{"errors":[],"reloaded":3}"#);
}

/// `engine status` et `engine pin` lisent toute forme TOML valide de la dépendance : table dédiée
/// `[dependencies.content]` et table en ligne multiligne, avec les autres clés conservées.
#[test]
fn engine_commands_accept_table_form_and_multiline_dependencies() {
    let tmp = Tmp::new("engine-toml");
    let (game, url) = game_on_git_tag(&tmp);
    let cargo_path = game.join("Cargo.toml");
    let original = std::fs::read_to_string(&cargo_path).unwrap();
    let inline = format!("content = {{ git = \"{url}\", tag = \"v1.0.0\" }}");
    assert!(original.contains(&inline), "{original}");
    let forms = [
        // Table dédiée, après les autres tables du fichier (valide : `[dependencies]` existe déjà).
        format!(
            "{}\n[dependencies.content]\n# engine\ngit = \"{url}\"\ntag = \"v1.0.0\"\nfeatures = []\n",
            original.replace(&inline, "")
        ),
        // Table en ligne répartie sur plusieurs lignes.
        original.replace(
            &inline,
            &format!("content = {{\n  git = \"{url}\",\n  tag = \"v1.0.0\",\n}}"),
        ),
    ];
    for cargo in forms {
        std::fs::write(&cargo_path, cargo).unwrap();
        let mut cmd = alacod();
        cmd.current_dir(&game).args(["engine", "status"]);
        let status = text(&run(cmd).stdout);
        assert!(
            status.contains(&format!("dépendance : git {url} tag v1.0.0")),
            "{status}"
        );

        let mut cmd = alacod();
        cmd.current_dir(&game).args(["engine", "pin", "v1.1.0"]);
        let out = run(cmd);
        assert!(
            out.status.success(),
            "{}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        let after = std::fs::read_to_string(&cargo_path).unwrap();
        assert!(
            after.contains("v1.1.0") && !after.contains("v1.0.0"),
            "{after}"
        );
        assert!(after.contains(&format!("git = \"{url}\"")), "{after}");
        // Les six autres dépendances de l'engine (inline) ont suivi, `content` aussi.
        assert_eq!(after.matches("tag = \"v1.1.0\"").count(), 7, "{after}");
    }
}

#[test]
fn remote_says_what_to_do_when_no_game_listens() {
    let port = free_port();
    let mut cmd = alacod();
    cmd.args(["remote", "--port", &port.to_string(), "brief"]);
    let out = run(cmd);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("ne répond pas"),
        "{}",
        text(&out.stderr)
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

/// `alacod remote <args>` sur `port`, tuée si elle dépasse `deadline` (échéance monotone). `Err` : échéance
/// dépassée.
fn remote(port: u16, args: &[&str], deadline: Instant) -> Result<Output, String> {
    let mut child = alacod()
        .arg("remote")
        .args(["--port", &port.to_string()])
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("alacod remote ne se lance pas : {e}"))?;
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

/// Le jeu lancé par `alacod run` (donc `alacod` → `cargo` → jeu) : son **groupe de processus** est tué au
/// `Drop`, sur tout chemin (assertion, JSON invalide). Jamais de `pkill` par motif : on tue le groupe de
/// l'enfant que ce test a lancé.
struct Game(std::process::Child);

impl Drop for Game {
    fn drop(&mut self) {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", self.0.id())])
            .output();
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Un dépôt git de l'engine à `HEAD`, sans réseau : le jeu en dépend par `git = "file://…", rev = HEAD`.
fn engine_head() -> (String, String) {
    let root = engine_root();
    let head = Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git");
    assert!(head.status.success(), "{}", text(&head.stderr));
    (
        format!("file://{}", root.display()),
        text(&head.stdout).trim().to_string(),
    )
}

/// Un jeu créé par `alacod new` (dépendance git épinglée sur `HEAD`, sans réseau), mené **uniquement par
/// `alacod`** : `engine local` (le code de ce dépôt, y compris non commité), `run --sandbox --headless`,
/// `remote`, puis `engine pin` qui revient à la révision. Ni Makefile ni script de l'engine. Commandes
/// équivalentes à la main : voir `docs/conventions.md` §89.
#[test]
#[ignore = "compile un jeu créé hors de l'engine (≈ 10 min à froid) : make test_alacod_cli"]
fn new_game_runs_its_sandbox_and_answers_remote_through_alacod_alone() {
    let tmp = Tmp::new("sandbox-e2e");
    let (url, head) = engine_head();
    let mut cmd = alacod();
    cmd.current_dir(&tmp.0)
        .args(["new", "demo", "--rev", &head, "--url", &url, "--engine"])
        .arg(engine_root());
    let out = run(cmd);
    assert!(
        out.status.success(),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let game = tmp.0.join("demo");
    let manifest = std::fs::read(game.join("Cargo.toml")).unwrap();

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

    // Développer engine et jeu ensemble : l'engine local remplace le tag, `Cargo.toml` ne bouge pas.
    let mut cmd = alacod();
    cmd.current_dir(&game)
        .args(["engine", "local"])
        .arg(engine_root());
    let out = run(cmd);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(std::fs::read(game.join("Cargo.toml")).unwrap(), manifest);
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["engine", "status"]);
    let status = text(&run(cmd).stdout);
    assert!(
        status.contains(&format!("racine compilée : {}", engine_root().display())),
        "{status}"
    );

    // `alacod run --sandbox --headless` : compile (profil headless) puis joue ; `ALACOD_EXIT_AT_FRAME`
    // exige `ALACOD_STATE_TRACE`, le jeu s'arrête de lui-même.
    let port = free_port();
    let trace = tmp.0.join("trace.txt");
    let mut game_proc = {
        use std::os::unix::process::CommandExt;
        let mut cmd = alacod();
        cmd.current_dir(&game)
            .args(["run", "--sandbox", "--headless"])
            .env("ALACOD_REMOTE_PORT", port.to_string())
            .env("ALACOD_STATE_TRACE", &trace)
            .env("ALACOD_EXIT_AT_FRAME", "1200")
            .env("CARGO_TARGET_DIR", game_target_dir(&game))
            .process_group(0)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        Game(cmd.spawn().expect("alacod run se lance"))
    };

    // Attend une frame jouée (la compilation à froid compte dans l'échéance), puis lit l'état.
    let ready_deadline = Instant::now() + Duration::from_secs(1800);
    let state: serde_json::Value = loop {
        if let Some(status) = game_proc.0.try_wait().unwrap() {
            panic!("alacod run s'est arrêté trop tôt : {status}");
        }
        assert!(
            Instant::now() < ready_deadline,
            "aucune frame jouée en 30 min"
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
    let target_posed = state["enemies"].as_array().unwrap().iter().any(|e| {
        e["config"] == "target" && e["x"].as_f64() == Some(624.0) && e["y"].as_f64() == Some(696.0)
    });
    assert!(target_posed, "{}", state["enemies"]);
    // Le réglage du profil `arene` (`currency: 1000`) : `give --currency 0` ne change rien et renvoie le solde.
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

    // Le jeu quitte seul à la frame demandée : code 0 et trace écrite.
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

    // Retour au tag : la surcharge locale part, `Cargo.toml` est celui d'origine (même révision).
    let mut cmd = alacod();
    cmd.current_dir(&game).args(["engine", "pin", &head]);
    let out = run(cmd);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!game.join(".cargo").exists());
    assert_eq!(std::fs::read(game.join("Cargo.toml")).unwrap(), manifest);
}
