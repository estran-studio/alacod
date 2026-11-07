//! `alacod build --web <jeu>` : la version web (Chrome) d'un jeu **hors de `games/`**, à partir de son seul
//! dossier (`docs/conventions.md` § 108). Les jeux de `games/` gardent `scripts/build-web-games.mjs`.
//!
//! Pourquoi une sous-commande d'`alacod` plutôt qu'un script : un jeu créé par `alacod new` n'a ni
//! `scripts/` ni `node` ; `alacod` est déjà l'outil unique de l'engine (§ 90) et sait lire le manifeste
//! (nom du jeu), lancer le lint et trouver le `Cargo.lock` du jeu, qui épingle `wasm-bindgen`.
//!
//! Chaîne : lint du contenu → `cargo build --target wasm32-unknown-unknown --no-default-features --features
//! render` (`APP_VERSION` = version du build, `--cfg=web_sys_unstable_apis`) → `wasm-bindgen --target web` (version
//! lue dans le `Cargo.lock` du jeu, installée dans `<target>/alacod-tools` si elle manque) → `wasm-opt` (release,
//! s'il est présent) → copie de `assets/` → `index.html` + `build.json`.
//!
//! Disposition de la sortie (la même que celle des builds du site : `/builds/<version>/<jeu>/`, que le `main.rs`
//! du gabarit pose comme racine d'assets) :
//!
//! ```text
//! <out>/index.html
//! <out>/builds/<version>/<jeu>/{wasm.js, wasm_bg.wasm, build.json, assets/…}
//! ```

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// Cible compilée par cargo.
const WASM_TARGET: &str = "wasm32-unknown-unknown";
/// Features de rendu d'un jeu (`render` du gabarit : tilemaps et écrans), sans `lighting` ni `native`.
pub const DEFAULT_FEATURES: &[&str] = &["render"];

#[derive(Debug, Clone)]
pub struct WebBuild {
    /// Dossier du jeu (contient `Cargo.toml` et `assets/game.ron`).
    pub game_dir: PathBuf,
    /// Dossier de sortie ; défaut `<target cargo du jeu>/web`.
    pub out: Option<PathBuf>,
    pub release: bool,
    /// Features cargo du jeu (`--no-default-features --features …`) ; vide : `DEFAULT_FEATURES`.
    pub features: Vec<String>,
    /// `APP_VERSION` ; défaut : `APP_VERSION` de l'environnement, sinon le SHA git du jeu.
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebOutput {
    pub game: String,
    pub version: String,
    pub out_dir: PathBuf,
    pub build_dir: PathBuf,
    pub wasm: PathBuf,
    pub wasm_bytes: u64,
    /// `wasm-opt` a tourné (release avec `wasm-opt` présent).
    pub optimized: bool,
    pub asset_files: usize,
}

struct CargoInfo {
    target_dir: PathBuf,
    workspace_root: PathBuf,
    bin: String,
}

/// Version valide pour un chemin d'URL et pour `APP_VERSION`.
pub fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.starts_with(|c: char| c.is_ascii_alphanumeric())
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn command_output(cmd: &mut Command) -> Option<String> {
    let out = cmd.output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// `APP_VERSION` de l'environnement, sinon le SHA git court du jeu (`-dirty` si des fichiers suivis ont
/// changé), sinon `local`. La version reste **stable** d'un build à l'autre : elle entre dans le wasm par
/// `APP_VERSION`, et une version qui change à chaque appel recompilerait `game` et `map_ldtk` à chaque fois.
pub fn default_version(game_dir: &Path) -> String {
    if let Ok(version) = std::env::var("APP_VERSION") {
        if !version.is_empty() {
            return version;
        }
    }
    let git =
        |args: &[&str]| command_output(Command::new("git").arg("-C").arg(game_dir).args(args));
    match git(&["rev-parse", "--short=12", "HEAD"]) {
        Some(sha) => match git(&["status", "--porcelain", "--untracked-files=no"]) {
            Some(status) if status.is_empty() => sha,
            _ => format!("{sha}-dirty"),
        },
        None => "local".into(),
    }
}

/// Supprime `builds/<autre version>/<jeu>` (et la version si elle devient vide) : un seul build par jeu dans `<out>`.
fn prune_other_versions(builds: &Path, keep: &str, game: &str) {
    let Ok(entries) = std::fs::read_dir(builds) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name() == keep || !entry.path().is_dir() {
            continue;
        }
        let _ = std::fs::remove_dir_all(entry.path().join(game));
        // `remove_dir` échoue (sans effet) si un autre jeu occupe encore cette version.
        let _ = std::fs::remove_dir(entry.path());
    }
}

fn cargo_info(game_dir: &Path) -> Result<CargoInfo, String> {
    let manifest = game_dir.join("Cargo.toml");
    let out = Command::new("cargo")
        .current_dir(game_dir)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .map_err(|e| format!("cargo metadata : {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata a échoué : {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata : {e}"))?;
    let canonical = manifest.canonicalize().map_err(|e| e.to_string())?;
    let package = json["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|p| {
            p["manifest_path"]
                .as_str()
                .and_then(|m| Path::new(m).canonicalize().ok())
                .is_some_and(|m| m == canonical)
        })
        .ok_or("cargo metadata : paquet du jeu introuvable")?;
    let package_name = package["name"].as_str().unwrap_or_default();
    let bins: Vec<&str> = package["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|t| {
            t["kind"]
                .as_array()
                .is_some_and(|k| k.iter().any(|k| k == "bin"))
        })
        .filter_map(|t| t["name"].as_str())
        .collect();
    let bin = bins
        .iter()
        .find(|b| **b == package_name)
        .or(bins.first())
        .ok_or_else(|| format!("le paquet {package_name} n'a pas de binaire (src/main.rs)"))?;
    let path = |key: &str| {
        json[key]
            .as_str()
            .map(PathBuf::from)
            .ok_or_else(|| format!("cargo metadata : `{key}` absent"))
    };
    Ok(CargoInfo {
        target_dir: path("target_directory")?,
        workspace_root: path("workspace_root")?,
        bin: bin.to_string(),
    })
}

/// Version de `wasm-bindgen` épinglée par un `Cargo.lock`.
pub fn locked_bindgen_version(lock: &str) -> Option<String> {
    let rest = lock.split("name = \"wasm-bindgen\"\nversion = \"").nth(1)?;
    rest.split('"').next().map(str::to_string)
}

fn bindgen_version(binary: &Path) -> Option<String> {
    let out = command_output(&mut Command::new(binary).arg("--version"))?;
    out.strip_prefix("wasm-bindgen ")
        .map(|v| v.trim().to_string())
}

/// Un `wasm-bindgen` de la version attendue : `WASM_BINDGEN`, le `PATH`, puis `<target>/alacod-tools/bin` ;
/// à défaut, `cargo install wasm-bindgen-cli --version <v>` dans ce dernier dossier.
fn resolve_bindgen(expected: &str, tools: &Path, game_dir: &Path) -> Result<PathBuf, String> {
    if let Ok(explicit) = std::env::var("WASM_BINDGEN") {
        let found = bindgen_version(Path::new(&explicit));
        return if found.as_deref() == Some(expected) {
            Ok(PathBuf::from(explicit))
        } else {
            Err(format!(
                "WASM_BINDGEN={explicit} est en version {} ; le Cargo.lock du jeu épingle {expected}",
                found.unwrap_or_else(|| "inconnue".into())
            ))
        };
    }
    let cached = tools.join("bin").join("wasm-bindgen");
    for candidate in [PathBuf::from("wasm-bindgen"), cached.clone()] {
        if bindgen_version(&candidate).as_deref() == Some(expected) {
            return Ok(candidate);
        }
    }
    eprintln!(
        "wasm-bindgen {expected} absent : cargo install wasm-bindgen-cli (une fois, dans {})",
        tools.display()
    );
    let status = Command::new("cargo")
        .current_dir(game_dir)
        .args([
            "install",
            "--locked",
            "wasm-bindgen-cli",
            "--version",
            expected,
            "--root",
        ])
        .arg(tools)
        .status()
        .map_err(|e| format!("cargo install : {e}"))?;
    if !status.success() || bindgen_version(&cached).as_deref() != Some(expected) {
        return Err(format!(
            "l'installation de wasm-bindgen-cli {expected} a échoué"
        ));
    }
    Ok(cached)
}

fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{what} : {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{what} a échoué ({status})"))
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Copie récursive de `assets/` ; retourne le nombre de fichiers.
fn copy_assets(from: &Path, to: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(to)?;
    let mut count = 0;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        if entry.file_name() == ".DS_Store" {
            continue;
        }
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            count += copy_assets(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst)?;
            count += 1;
        }
    }
    Ok(count)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    let Some((&first, _)) = needle.split_first() else {
        return true;
    };
    let mut from = 0;
    while let Some(offset) = haystack[from..].iter().position(|b| *b == first) {
        let at = from + offset;
        if haystack[at..].starts_with(needle) {
            return true;
        }
        from = at + 1;
    }
    false
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// La page qui lance le jeu : le canvas `#bevy-canvas` que cherche `CoreAppPlugin`, le module wasm-bindgen,
/// `download_log_file_js` attendu par le wasm, et un message visible si le jeu ne démarre pas.
pub fn index_html(game: &str, version: &str) -> String {
    let title = html_escape(game);
    format!(
        r##"<!doctype html>
<html lang="fr">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>{title}</title>
<style>
  html, body {{ margin: 0; width: 100%; height: 100%; overflow: hidden; background: #1a1a2e; color: #ccc; font-family: monospace; }}
  #bevy-canvas {{ width: 100%; height: 100%; display: block; outline: none; }}
  #erreur {{ display: none; position: fixed; inset: 0; padding: 2em; white-space: pre-wrap; background: #1a1a2e; }}
</style>
</head>
<body>
<canvas id="bevy-canvas" tabindex="0"></canvas>
<div id="erreur"></div>
<script type="module">
  const canvas = document.getElementById('bevy-canvas');
  const params = new URLSearchParams(location.search);
  const fail = (message) => {{
    const box = document.getElementById('erreur');
    box.style.display = 'block';
    box.textContent = `Le jeu ne démarre pas.\n\n${{message}}`;
    console.error(message);
  }};
  window.download_log_file_js = (filename, content) => {{
    const url = URL.createObjectURL(new Blob([content], {{ type: 'text/plain;charset=utf-8' }}));
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }};
  canvas.setAttribute('data-number-player', '1');
  canvas.setAttribute('data-telemetry', 'false');
  if (params.get('perf')) canvas.setAttribute('data-perf', params.get('perf'));
  window.addEventListener('unhandledrejection', (event) => fail(String(event.reason)));
  try {{
    const module = await import('./builds/{version}/{game}/wasm.js');
    await module.default();
    canvas.focus();
    canvas.addEventListener('pointerdown', () => canvas.focus());
  }} catch (error) {{
    fail(String(error));
  }}
</script>
</body>
</html>
"##
    )
}

/// Construit la version web du jeu. Écrit sur stderr la sortie de cargo et de `wasm-bindgen`.
pub fn build(options: &WebBuild) -> Result<WebOutput, String> {
    let game_dir = options
        .game_dir
        .canonicalize()
        .map_err(|e| format!("{} : {e}", options.game_dir.display()))?;
    if !game_dir.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} n'a pas de Cargo.toml (dossier d'un jeu)",
            game_dir.display()
        ));
    }

    // Le même contrôle que le démarrage du jeu : un contenu invalide ne part pas en ligne.
    let (_, manifest, errors) = crate::load_and_lint(&game_dir).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{error}");
        }
        return Err(format!(
            "{} erreur(s) de contenu (voir `alacod lint`)",
            errors.len()
        ));
    }
    let game = manifest.name;
    if !game
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        || game.is_empty()
    {
        return Err(format!(
            "nom de jeu « {game} » inutilisable dans une URL (lettres, chiffres, `-`, `_`)"
        ));
    }
    let version = options
        .version
        .clone()
        .unwrap_or_else(|| default_version(&game_dir));
    if !valid_version(&version) {
        return Err(format!(
            "version « {version} » invalide (lettres, chiffres, `.`, `_`, `-`)"
        ));
    }

    let info = cargo_info(&game_dir)?;
    let profile_dir = if options.release { "release" } else { "debug" };
    let tools = info.target_dir.join("alacod-tools");
    let lock_path = info.workspace_root.join("Cargo.lock");
    let lock_version = |path: &Path| {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|l| locked_bindgen_version(&l))
    };
    // Avant la compilation (longue) quand le verrou existe déjà : un outil manquant échoue tout de suite.
    let mut bindgen = match lock_version(&lock_path) {
        Some(expected) => Some(resolve_bindgen(&expected, &tools, &game_dir)?),
        None => None,
    };

    if which("rustup").is_some() {
        // Idempotent ; l'échec (hors ligne) n'est pas fatal : cargo dira si la cible manque.
        let _ = Command::new("rustup")
            .current_dir(&game_dir)
            .args(["target", "add", WASM_TARGET])
            .status();
    }
    let features = if options.features.is_empty() {
        DEFAULT_FEATURES.iter().map(|f| f.to_string()).collect()
    } else {
        options.features.clone()
    };
    let rustflags = match std::env::var("RUSTFLAGS") {
        Ok(flags) if !flags.is_empty() => format!("{flags} --cfg=web_sys_unstable_apis"),
        _ => "--cfg=web_sys_unstable_apis".to_string(),
    };
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(&game_dir)
        .args(["build", "--manifest-path"])
        .arg(game_dir.join("Cargo.toml"))
        .args([
            "--target",
            WASM_TARGET,
            "--no-default-features",
            "--features",
        ])
        .arg(features.join(","))
        .env("APP_VERSION", &version)
        .env("RUSTFLAGS", rustflags)
        // Prioritaire sur `RUSTFLAGS` pour cargo : sans cela le `--cfg` ci-dessus serait perdu.
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    if options.release {
        cargo.arg("--release");
    }
    run(&mut cargo, "cargo build (wasm32)")?;

    if bindgen.is_none() {
        let expected = lock_version(&lock_path)
            .ok_or_else(|| format!("wasm-bindgen absent de {}", lock_path.display()))?;
        bindgen = Some(resolve_bindgen(&expected, &tools, &game_dir)?);
    }
    let bindgen = bindgen.expect("résolu ci-dessus");
    let built = info
        .target_dir
        .join(WASM_TARGET)
        .join(profile_dir)
        .join(format!("{}.wasm", info.bin));
    if !built.is_file() {
        return Err(format!(
            "{} introuvable après la compilation",
            built.display()
        ));
    }

    let out_dir = options
        .out
        .clone()
        .unwrap_or_else(|| info.target_dir.join("web"));
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let out_dir = out_dir.canonicalize().map_err(|e| e.to_string())?;
    let versions = out_dir.join("builds").join(&version);
    let build_dir = versions.join(&game);
    let staging = versions.join(format!("{game}.partial-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(bool, usize), String> {
        run(
            Command::new(&bindgen)
                .arg("--out-dir")
                .arg(&staging)
                .args(["--out-name", "wasm", "--target", "web"])
                .arg(&built),
            "wasm-bindgen",
        )?;
        let wasm = staging.join("wasm_bg.wasm");
        let optimizer = std::env::var_os("WASM_OPT")
            .map(PathBuf::from)
            .or_else(|| which("wasm-opt"));
        let mut optimized = false;
        if options.release {
            match optimizer {
                Some(optimizer) => {
                    let tmp = staging.join("wasm_bg.optimized.wasm");
                    run(
                        Command::new(optimizer)
                            .arg(&wasm)
                            .args(["--strip-debug", "--strip-dwarf", "-O2", "-o"])
                            .arg(&tmp),
                        "wasm-opt",
                    )?;
                    std::fs::rename(&tmp, &wasm).map_err(|e| e.to_string())?;
                    optimized = true;
                }
                None => eprintln!(
                    "wasm-opt absent : wasm non optimisé (binaryen, ou WASM_OPT=<chemin>)"
                ),
            }
        }
        // `main.rs` du jeu doit chercher ses assets où on les pose ; sinon la page s'ouvre sur du noir et 404.
        let asset_root = format!("/builds/{version}/{game}/assets");
        let bytes = std::fs::read(&wasm).map_err(|e| e.to_string())?;
        if !contains(&bytes, asset_root.as_bytes()) {
            return Err(format!(
                "le wasm ne contient pas la racine d'assets `{asset_root}` : le `main.rs` du jeu doit poser \
                 `game_config.asset_root = Some(concat!(\"/builds/\", env!(\"APP_VERSION\"), \"/{game}/assets\").to_string())` \
                 sous `cfg(target_arch = \"wasm32\")` (gabarit `templates/starter/src/main.rs`)"
            ));
        }
        let files = copy_assets(&game_dir.join("assets"), &staging.join("assets"))
            .map_err(|e| e.to_string())?;
        let base = format!("/builds/{version}/{game}");
        let manifest = serde_json::json!({
            "schemaVersion": 1, "gameId": game, "buildId": version,
            "module": format!("{base}/wasm.js"), "wasm": format!("{base}/wasm_bg.wasm"),
            "assets": format!("{base}/assets"),
        });
        std::fs::write(
            staging.join("build.json"),
            serde_json::to_string_pretty(&manifest).unwrap() + "\n",
        )
        .map_err(|e| e.to_string())?;
        Ok((optimized, files))
    })();
    let (optimized, asset_files) = match result {
        Ok(done) => done,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(e);
        }
    };
    // Les builds d'autres versions de ce jeu pèsent des centaines de Mo chacun et ne servent plus à rien :
    // la page ouvre toujours la version qu'on vient d'écrire. Les autres jeux du même `<out>` restent.
    prune_other_versions(&out_dir.join("builds"), &version, &game);
    let _ = std::fs::remove_dir_all(&build_dir);
    std::fs::rename(&staging, &build_dir).map_err(|e| e.to_string())?;
    std::fs::write(out_dir.join("index.html"), index_html(&game, &version))
        .map_err(|e| e.to_string())?;
    let wasm = build_dir.join("wasm_bg.wasm");
    let wasm_bytes = std::fs::metadata(&wasm)
        .map(|m| m.len())
        .map_err(|e| e.to_string())?;
    Ok(WebOutput {
        game,
        version,
        out_dir,
        build_dir,
        wasm,
        wasm_bytes,
        optimized,
        asset_files,
    })
}

// ---------------------------------------------------------------------------------------
// Serveur local
// ---------------------------------------------------------------------------------------

/// Serveur HTTP statique minimal (GET et HEAD, un fil par connexion, rien d'autre que des fichiers de `root`) :
/// sert `application/wasm` et `text/javascript`, ce que `wasm-bindgen --target web` exige du serveur.
pub struct Server {
    listener: TcpListener,
    root: PathBuf,
}

impl Server {
    /// Écoute sur `127.0.0.1:<port>` (`0` : un port libre).
    pub fn bind(root: &Path, port: u16) -> Result<Self, String> {
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
            .map_err(|e| format!("127.0.0.1:{port} : {e} (autre port : --port <n>)"))?;
        Ok(Self {
            listener,
            root: root.to_path_buf(),
        })
    }

    pub fn port(&self) -> u16 {
        self.listener.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    /// Sert sans fin.
    pub fn run(self) {
        for stream in self.listener.incoming().flatten() {
            let root = self.root.clone();
            std::thread::spawn(move || {
                let _ = handle(stream, &root);
            });
        }
    }
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "json" | "ldtk" => "application/json",
        "css" => "text/css; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "ttf" => "font/ttf",
        "ron" | "txt" | "md" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn percent_decode(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Fichier de `root` désigné par l'URL, jamais en dehors de `root`.
fn resolve(root: &Path, target: &str) -> Option<PathBuf> {
    let path = percent_decode(target.split(['?', '#']).next()?)?;
    let mut file = root.to_path_buf();
    for part in Path::new(path.trim_start_matches('/')).components() {
        match part {
            Component::Normal(name) if !name.to_string_lossy().contains(['\\', '\0']) => {
                file.push(name)
            }
            Component::CurDir => {}
            _ => return None,
        }
    }
    if file.is_dir() {
        file.push("index.html");
    }
    file.is_file().then_some(file)
}

fn handle(mut stream: TcpStream, root: &Path) -> std::io::Result<()> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
    let mut head = Vec::new();
    let mut chunk = [0u8; 1024];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") && head.len() < 16 * 1024 {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        head.extend_from_slice(&chunk[..n]);
    }
    let text = String::from_utf8_lossy(&head);
    let mut line = text.lines().next().unwrap_or_default().split_whitespace();
    let (method, target) = (line.next().unwrap_or_default(), line.next().unwrap_or("/"));
    let respond = |stream: &mut TcpStream,
                   status: &str,
                   kind: &str,
                   body: &[u8],
                   with_body: bool| {
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            body.len()
        )?;
        if with_body {
            stream.write_all(body)?;
        }
        stream.flush()
    };
    if method != "GET" && method != "HEAD" {
        return respond(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain",
            b"GET seulement",
            true,
        );
    }
    match resolve(root, target).and_then(|file| std::fs::read(&file).ok().map(|body| (file, body)))
    {
        Some((file, body)) => respond(
            &mut stream,
            "200 OK",
            content_type(&file),
            &body,
            method == "GET",
        ),
        None => respond(
            &mut stream,
            "404 Not Found",
            "text/plain",
            b"introuvable",
            method == "GET",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindgen_version_is_read_from_the_lock_not_from_a_sibling_crate() {
        let lock = "[[package]]\nname = \"wasm-bindgen-futures\"\nversion = \"0.4.1\"\n\n[[package]]\nname = \"wasm-bindgen\"\nversion = \"0.2.129\"\n";
        assert_eq!(locked_bindgen_version(lock).as_deref(), Some("0.2.129"));
        assert_eq!(
            locked_bindgen_version("[[package]]\nname = \"bevy\"\nversion = \"1\"\n"),
            None
        );
    }

    #[test]
    fn versions_are_url_safe() {
        assert!(valid_version("a1b2c3d4e5f6"));
        assert!(valid_version("a1b2c3d4e5f6-dirty1762400000"));
        assert!(!valid_version(""));
        assert!(!valid_version("-x"));
        assert!(!valid_version("a/b"));
        assert!(!valid_version("v 1"));
    }

    #[test]
    fn a_new_build_replaces_older_versions_of_its_game_only() {
        let dir = std::env::temp_dir().join(format!("alacod-web-prune-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for path in ["old/jeu", "old/autre", "alone/jeu", "new/jeu"] {
            std::fs::create_dir_all(dir.join(path)).unwrap();
        }
        prune_other_versions(&dir, "new", "jeu");
        assert!(
            !dir.join("old/jeu").exists(),
            "ancienne version du jeu supprimée"
        );
        assert!(
            dir.join("old/autre").is_dir(),
            "un autre jeu de la même version reste"
        );
        assert!(!dir.join("alone").exists(), "version vide supprimée");
        assert!(dir.join("new/jeu").is_dir(), "la version écrite reste");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn index_loads_the_module_of_this_build() {
        let html = index_html("mon-jeu", "abc123");
        assert!(html.contains("import('./builds/abc123/mon-jeu/wasm.js')"));
        assert!(html.contains("id=\"bevy-canvas\""));
    }

    fn get(port: u16, target: &str) -> (String, String, Vec<u8>) {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(stream, "GET {target} HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).unwrap();
        let split = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let head = String::from_utf8_lossy(&raw[..split]).into_owned();
        let status = head.lines().next().unwrap().to_string();
        (status, head, raw[split + 4..].to_vec())
    }

    #[test]
    fn server_serves_wasm_and_modules_with_their_mime_types_and_stays_inside_its_root() {
        let dir = std::env::temp_dir().join(format!("alacod-web-serve-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("builds/v/jeu")).unwrap();
        std::fs::write(dir.join("index.html"), "<html>page</html>").unwrap();
        std::fs::write(dir.join("builds/v/jeu/wasm_bg.wasm"), b"\0asm").unwrap();
        std::fs::write(dir.join("builds/v/jeu/wasm.js"), "export default 1;").unwrap();
        std::fs::write(dir.join("secret.txt"), "hors racine").unwrap();
        let root = dir.join("builds");
        std::fs::write(root.join("a b.ron"), "()").unwrap();

        let server = Server::bind(&dir, 0).unwrap();
        let port = server.port();
        std::thread::spawn(move || server.run());

        let (status, head, body) = get(port, "/");
        assert_eq!(status, "HTTP/1.1 200 OK");
        assert!(head.contains("text/html"));
        assert_eq!(body, b"<html>page</html>");
        let (_, head, body) = get(port, "/builds/v/jeu/wasm_bg.wasm?x=1");
        assert!(head.contains("Content-Type: application/wasm"), "{head}");
        assert_eq!(body, b"\0asm");
        let (_, head, _) = get(port, "/builds/v/jeu/wasm.js");
        assert!(head.contains("text/javascript"), "{head}");
        let (_, _, body) = get(port, "/builds/a%20b.ron");
        assert_eq!(body, b"()");
        assert_eq!(get(port, "/nope").0, "HTTP/1.1 404 Not Found");
        // Une racine plus profonde ne laisse pas sortir `..`, même encodé.
        let server = Server::bind(&root, 0).unwrap();
        let port = server.port();
        std::thread::spawn(move || server.run());
        assert_eq!(get(port, "/../secret.txt").0, "HTTP/1.1 404 Not Found");
        assert_eq!(get(port, "/%2e%2e/secret.txt").0, "HTTP/1.1 404 Not Found");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
