//! CLI `alacod` : `alacod lint <dossier-du-jeu>` valide le contenu d'un jeu (manifeste,
//! registre, références, plages, kinds) sans lancer le moteur (`docs/plan-engine.md` §4.2,
//! carte T1.5). Code de sortie 1 et messages sur stderr en cas d'erreur, 0 sinon.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use content::load_and_lint;

#[derive(Parser)]
#[command(name = "alacod", about = "Outils de contenu pour l'engine alacod")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Valide le contenu d'un jeu : `alacod lint games/zombies` (sans chemin : le jeu qui
    /// contient le dossier courant).
    Lint {
        /// Dossier du jeu (contient `assets/game.ron`), ex. `games/zombies`.
        game_dir: Option<PathBuf>,
        /// Sortie JSON sur stdout (`errors`, `warnings`) au lieu du texte sur stderr.
        #[arg(long)]
        json: bool,
    },
    /// Crée un nouveau jeu hors de l'engine : `alacod new mon-jeu [--engine ~/alacod]`.
    New {
        /// Nom du jeu (minuscules, chiffres, `-` et `_`) ; crée le dossier `./<nom>`.
        name: String,
        /// Racine de l'engine (contient `crates/` et `templates/starter`). Par défaut : celle
        /// d'où ce binaire a été compilé.
        #[arg(long)]
        engine: Option<PathBuf>,
        /// Dépendre de l'engine par git (URL de `origin` de l'engine, révision = `--rev` ou
        /// le HEAD de l'engine, qui doit être poussé) au lieu d'un chemin local.
        #[arg(long)]
        git: bool,
        /// Révision git de l'engine (implique `--git`).
        #[arg(long)]
        rev: Option<String>,
    },
    /// Joue les scénarios générés d'un jeu (`test:` des armes, personnages, objets, salles,
    /// boss) : `alacod test [jeu] [id…] [--list]`. Sans id, tous ; avec un id (`amulette` ou
    /// `item_amulette`), seulement celui-là. Les traces de référence sont ignorées sauf `--bless`.
    Test {
        /// `[dossier du jeu] [id…]` : le premier argument est pris pour un dossier s'il existe.
        args: Vec<String>,
        /// Liste les sujets testables au lieu de les jouer.
        #[arg(long)]
        list: bool,
        /// Écrit les traces de référence (changement de gameplay voulu).
        #[arg(long)]
        bless: bool,
        /// Sortie JSON (tableau de résultats) sur stdout.
        #[arg(long)]
        json: bool,
        /// Racine de l'engine ; sinon celle du jeu (dépendance `Cargo.toml`) ou de ce binaire.
        #[arg(long)]
        engine: Option<PathBuf>,
    },
    /// Crée une entrée de contenu à partir d'un modèle du jeu :
    /// `alacod new-content [jeu] <Kind> <id> [--from <modèle>]`.
    NewContent {
        /// `[dossier du jeu] <Kind> <id>`.
        args: Vec<String>,
        /// Entrée à copier (défaut : la première du kind).
        #[arg(long)]
        from: Option<String>,
    },
    /// Inventaire du vocabulaire d'un jeu : `alacod describe [games/gungeon] [Kind]`.
    Describe {
        /// `[dossier du jeu] [Kind]` : le premier argument est pris pour un dossier s'il
        /// existe, sinon pour un kind (`alacod describe Item` dans un dossier de jeu).
        args: Vec<String>,
        /// Sortie JSON sur stdout.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Lint { game_dir, json } => match resolve_game_dir(game_dir) {
            Ok(dir) if json => lint_json(&dir),
            Ok(dir) => lint(&dir),
            Err(e) => fail(&e),
        },
        Command::New {
            name,
            engine,
            git,
            rev,
        } => match new_game(&name, engine, git || rev.is_some(), rev) {
            Ok(dir) => {
                println!("{} créé.", dir.display());
                println!("  cd {name} && alacod lint && alacod describe");
                println!("  cargo run   # premier build long (bevy)");
                ExitCode::SUCCESS
            }
            Err(e) => fail(&e),
        },
        Command::NewContent { args, from } => {
            let (dir_arg, rest) = match args.split_first() {
                Some((first, rest)) if PathBuf::from(first).is_dir() && rest.len() == 2 => {
                    (Some(PathBuf::from(first)), rest.to_vec())
                }
                _ => (None, args.clone()),
            };
            let [kind, id] = rest.as_slice() else {
                return fail("usage : alacod new-content [jeu] <Kind> <id> [--from <modèle>]");
            };
            match resolve_game_dir(dir_arg) {
                Ok(dir) => new_content(&dir, kind, id, from.as_deref()),
                Err(e) => fail(&e),
            }
        }
        Command::Test {
            args,
            list,
            bless,
            json,
            engine,
        } => {
            let (dir_arg, ids) = match args.split_first() {
                Some((first, rest)) if PathBuf::from(first).is_dir() => {
                    (Some(PathBuf::from(first)), rest.to_vec())
                }
                _ => (None, args.clone()),
            };
            match resolve_game_dir(dir_arg) {
                Ok(dir) => run_tests(&dir, &ids, list, bless, json, engine),
                Err(e) => fail(&e),
            }
        }
        Command::Describe { args, json } => {
            let (dir_arg, kind) = match args.as_slice() {
                [] => (None, None),
                [one] if !PathBuf::from(one).is_dir() => (None, Some(one.clone())),
                [one] => (Some(PathBuf::from(one)), None),
                [dir, kind, ..] => (Some(PathBuf::from(dir)), Some(kind.clone())),
            };
            match resolve_game_dir(dir_arg) {
                Ok(dir) => describe(&dir, kind.as_deref(), json),
                Err(e) => fail(&e),
            }
        }
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

/// Dossier du jeu : l'argument s'il est donné, sinon le premier ancêtre du dossier courant
/// (lui compris) qui contient `assets/game.ron` ; depuis `assets/` ou un de ses sous-dossiers,
/// le parent de `assets/`. Comme `git` pour `.git`.
fn resolve_game_dir(arg: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(dir) = arg {
        return Ok(dir);
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("assets").join("game.ron").is_file() {
            return Ok(dir.to_path_buf());
        }
        if dir.file_name().is_some_and(|n| n == "assets") && dir.join("game.ron").is_file() {
            return Ok(dir.parent().unwrap_or(dir).to_path_buf());
        }
    }
    Err(format!(
        "aucun jeu alacod trouvé depuis {} (cherché assets/game.ron dans les dossiers parents) ; \
         passer le dossier du jeu en argument",
        cwd.display()
    ))
}

fn lint(game_dir: &PathBuf) -> ExitCode {
    let (registry, manifest, errors) = match load_and_lint(game_dir) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    // M2-T15 : avertissements (atteignabilité, stats sans lecteur) ; n'échouent pas.
    let warnings = content::warnings(&registry, &manifest);
    for warning in &warnings {
        eprintln!("avertissement : {warning}");
    }
    if !warnings.is_empty() {
        eprintln!(
            "{} : {} avertissement(s)",
            game_dir.display(),
            warnings.len()
        );
    }

    if errors.is_empty() {
        println!(
            "{} : aucune erreur ({} personnages, {} armes, {} armes de corps à corps, {} vagues, {} cartes)",
            game_dir.display(),
            registry.characters.len(),
            registry.weapons.len(),
            registry.melee_weapons.len(),
            registry.waves.len(),
            registry.maps.len(),
        );
        return ExitCode::SUCCESS;
    }

    for error in &errors {
        eprintln!("{error}");
    }
    eprintln!(
        "{} : {} erreur(s) de contenu",
        game_dir.display(),
        errors.len()
    );
    ExitCode::FAILURE
}

/// `lint --json` : un seul objet sur stdout, même code de sortie que `lint` (1 s'il y a des
/// erreurs ; les avertissements n'échouent pas).
fn lint_json(game_dir: &PathBuf) -> ExitCode {
    let (registry, manifest, errors) = match load_and_lint(game_dir) {
        Ok(result) => result,
        Err(e) => {
            println!("{}", serde_json::json!({ "fatal": e.to_string() }));
            return ExitCode::FAILURE;
        }
    };
    let warnings = content::warnings(&registry, &manifest);
    let report = serde_json::json!({
        "game": manifest.name,
        "errors": errors.iter().map(|e| serde_json::json!({
            "kind": format!("{:?}", e.kind),
            "file": e.file,
            "message": e.message,
        })).collect::<Vec<_>>(),
        "warnings": warnings.iter().map(|w| serde_json::json!({
            "kind": format!("{:?}", w.kind),
            "file": w.file,
            "message": w.message,
        })).collect::<Vec<_>>(),
    });
    println!("{report}");
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn describe(game_dir: &PathBuf, kind: Option<&str>, json: bool) -> ExitCode {
    let desc = match content::describe::describe(game_dir) {
        Ok(desc) => desc,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match kind {
        None if json => println!("{}", serde_json::to_string(&desc).unwrap()),
        None => print!("{}", content::describe::render_overview(&desc)),
        Some(name) => match content::describe::find_kind(&desc, name) {
            Some(info) if json => println!("{}", serde_json::to_string(info).unwrap()),
            Some(info) => print!("{}", content::describe::render_kind(info)),
            None => {
                let known: Vec<_> = desc.kinds.iter().map(|k| k.kind.as_str()).collect();
                eprintln!(
                    "kind « {name} » absent de {} (présents : {})",
                    desc.game,
                    known.join(", ")
                );
                return ExitCode::FAILURE;
            }
        },
    }
    ExitCode::SUCCESS
}

/// Racine de l'engine par défaut : deux niveaux au-dessus de `crates/content`, capturée à la
/// compilation de `alacod`.
const BUILT_FROM_ENGINE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// `alacod new` : copie `<engine>/templates/starter` dans `./<name>` en remplaçant `{{name}}`
/// et `{{engine}}` dans les fichiers texte, et reprend le `Cargo.lock` de l'engine (mêmes
/// versions de dépendances que l'engine, qui compile).
fn new_game(
    name: &str,
    engine: Option<PathBuf>,
    git: bool,
    rev: Option<String>,
) -> Result<PathBuf, String> {
    let valid = name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid {
        return Err(format!(
            "nom de jeu « {name} » invalide : minuscules, chiffres, `-` et `_`, première lettre minuscule"
        ));
    }
    let engine = engine.unwrap_or_else(|| PathBuf::from(BUILT_FROM_ENGINE));
    let engine = engine
        .canonicalize()
        .map_err(|e| format!("engine introuvable ({}) : {e}", engine.display()))?;
    let template = engine.join("templates").join("starter");
    if !template.join("assets").join("game.ron").is_file() {
        return Err(format!(
            "{} n'est pas un gabarit de jeu (engine trop ancien ? --engine <racine>)",
            template.display()
        ));
    }
    let target = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join(name);
    if target.exists() {
        return Err(format!("{} existe déjà", target.display()));
    }
    let engine_str = engine.to_string_lossy().into_owned();
    let source = if git {
        EngineSource::Git {
            url: git_output(&engine, &["remote", "get-url", "origin"])?,
            rev: match rev {
                Some(rev) => rev,
                None => git_output(&engine, &["rev-parse", "HEAD"])?,
            },
        }
    } else {
        EngineSource::Path(engine_str.clone())
    };
    copy_template(&template, &target, name, &engine_str, &source).map_err(|e| e.to_string())?;
    let lock = engine.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, target.join("Cargo.lock")).map_err(|e| e.to_string())?;
    }
    Ok(target)
}

/// Comment le jeu référence les crates de l'engine dans son `Cargo.toml`.
enum EngineSource {
    Path(String),
    Git { url: String, rev: String },
}

impl EngineSource {
    /// Spécification de dépendance (`{{engine:<crate>}}` du gabarit).
    fn dependency(&self, krate: &str) -> String {
        match self {
            Self::Path(root) => format!("path = \"{root}/crates/{krate}\""),
            Self::Git { url, rev } => format!("git = \"{url}\", rev = \"{rev}\""),
        }
    }
}

fn git_output(dir: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git : {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} a échoué dans {} : {}",
            args.join(" "),
            dir.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn copy_template(
    from: &std::path::Path,
    to: &std::path::Path,
    name: &str,
    engine: &str,
    source: &EngineSource,
) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if entry.file_name() == ".DS_Store" {
            continue;
        }
        if src.is_dir() {
            copy_template(&src, &dst, name, engine, source)?;
            continue;
        }
        match std::fs::read_to_string(&src) {
            Ok(mut text) => {
                text = text.replace("{{name}}", name).replace("{{engine}}", engine);
                while let Some(start) = text.find("{{engine:") {
                    let end = start + text[start..].find("}}").unwrap_or(0) + 2;
                    let krate = text[start + "{{engine:".len()..end - 2].to_string();
                    text.replace_range(start..end, &source.dependency(&krate));
                }
                std::fs::write(&dst, text)?
            }
            // Binaire (png, ogg, ttf) : copie telle quelle.
            Err(_) => {
                std::fs::copy(&src, &dst)?;
            }
        }
    }
    Ok(())
}

/// Racine de l'engine pour un jeu : `--engine`, sinon l'ancêtre du jeu qui contient
/// `crates/scenario` (jeu de `games/`), sinon la dépendance `crates/content` de son
/// `Cargo.toml` (jeu créé par `alacod new`), sinon celle d'où `alacod` a été compilé.
fn engine_root_for(
    game_dir: &std::path::Path,
    explicit: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let is_engine = |p: &std::path::Path| {
        p.join("crates")
            .join("scenario")
            .join("Cargo.toml")
            .is_file()
    };
    if let Some(root) = explicit {
        return root
            .canonicalize()
            .map_err(|e| format!("engine introuvable ({}) : {e}", root.display()));
    }
    let game = game_dir.canonicalize().map_err(|e| e.to_string())?;
    if let Some(root) = game.ancestors().skip(1).find(|p| is_engine(p)) {
        return Ok(root.to_path_buf());
    }
    if let Ok(cargo) = std::fs::read_to_string(game.join("Cargo.toml")) {
        for line in cargo.lines() {
            let Some(rest) = line.trim().strip_prefix("content") else {
                continue;
            };
            if let Some(start) = rest.find("path = \"") {
                let path = &rest[start + 8..];
                if let Some(end) = path.find('"') {
                    let root = std::path::Path::new(&path[..end]).join("..").join("..");
                    if let Ok(root) = root.canonicalize() {
                        if is_engine(&root) {
                            return Ok(root);
                        }
                    }
                }
            }
        }
    }
    let built = PathBuf::from(BUILT_FROM_ENGINE);
    built
        .canonicalize()
        .ok()
        .filter(|r| is_engine(r))
        .ok_or_else(|| "engine introuvable : passer --engine <racine>".to_string())
}

/// `alacod test` : délègue à `alacod-gen` de l'engine (le crate `scenario` dépend de `content`,
/// pas l'inverse), en ignorant les traces sauf `--bless`.
fn run_tests(
    game_dir: &std::path::Path,
    ids: &[String],
    list: bool,
    bless: bool,
    json: bool,
    engine: Option<PathBuf>,
) -> ExitCode {
    let engine = match engine_root_for(game_dir, engine) {
        Ok(root) => root,
        Err(e) => return fail(&e),
    };
    let game = match game_dir.canonicalize() {
        Ok(g) => g,
        Err(e) => return fail(&e.to_string()),
    };
    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "run",
        "-q",
        "-p",
        "scenario",
        "--bin",
        "alacod-gen",
        "--profile",
        "headless",
    ])
    .arg("--manifest-path")
    .arg(engine.join("Cargo.toml"))
    .arg("--")
    .arg(&game);
    if list {
        cmd.arg("--list");
    } else {
        cmd.arg("--play").arg("--json");
        if bless {
            cmd.arg("--bless");
        } else {
            cmd.arg("--no-trace");
        }
        if !ids.is_empty() {
            cmd.arg("--only").arg(ids.join(","));
        }
    }
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => return fail(&format!("cargo : {e}")),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    // stderr de cargo (avertissements de compilation de l'engine) : seulement si ça a échoué.
    let show_stderr = |output: &std::process::Output| {
        let text = String::from_utf8_lossy(&output.stderr);
        // Les messages d'`alacod-gen` d'abord ; sinon la fin de la sortie de cargo.
        let own: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("alacod-gen"))
            .collect();
        let lines: Vec<&str> = text.lines().collect();
        let start = lines.len().saturating_sub(25);
        eprintln!(
            "{}",
            if own.is_empty() {
                lines[start..].join("\n")
            } else {
                own.join("\n")
            }
        );
    };

    if list {
        // Les journaux du jeu partagent stdout : seules les lignes `scénario<TAB>sujet` comptent.
        for line in stdout.lines().filter(|l| l.contains('\t')) {
            println!("{}", line.replace('\t', "  "));
        }
        return if output.status.success() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    // Le tableau JSON est la dernière ligne qui commence par `[`.
    let Some(results) = stdout
        .lines()
        .rev()
        .find(|l| l.starts_with('['))
        .and_then(|l| serde_json::from_str::<Vec<serde_json::Value>>(l).ok())
    else {
        show_stderr(&output);
        return fail("alacod test : aucun résultat (voir ci-dessus)");
    };
    if json {
        println!("{}", serde_json::Value::Array(results.clone()));
    } else {
        for r in &results {
            let ok = r["expectations_ok"].as_bool().unwrap_or(false);
            println!(
                "{} {:<28} {:>5} frames, {:>3} coups",
                if ok { "ok    " } else { "ÉCHEC " },
                r["scenario"].as_str().unwrap_or("?"),
                r["frames"],
                r["hits"],
            );
            for f in r["failures"].as_array().into_iter().flatten() {
                println!("        {}", f.as_str().unwrap_or("?"));
            }
            if ok && r["expectations"] == 0 {
                println!("        (aucune attente : ce test ne prouve rien ; ajouter `test:` à l'entrée, voir `alacod describe`)");
            }
        }
    }
    let failed = results
        .iter()
        .filter(|r| r["expectations_ok"] != true)
        .count();
    if !json {
        println!("{} scénario(s), {} échec(s)", results.len(), failed);
    }
    if failed == 0 && output.status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn new_content(game_dir: &std::path::Path, kind: &str, id: &str, from: Option<&str>) -> ExitCode {
    let created = match content::scaffold::new_content(game_dir, kind, id, from) {
        Ok(c) => c,
        Err(e) => return fail(&e),
    };
    println!(
        "{} {} « {id} » (modèle : {})",
        if created.in_table {
            "entrée ajoutée à"
        } else {
            "créé :"
        },
        created.file.display(),
        created.model
    );
    // Le modèle passe le lint : toute erreur vient du changement (référence à renommer, id…).
    match load_and_lint(game_dir) {
        Ok((_, _, errors)) if errors.is_empty() => {
            println!("lint : aucune erreur");
            println!("à adapter : alacod describe {kind} ; puis alacod test {id}");
            ExitCode::SUCCESS
        }
        Ok((_, _, errors)) => {
            for e in &errors {
                eprintln!("{e}");
            }
            eprintln!(
                "lint : {} erreur(s) à corriger dans {}",
                errors.len(),
                created.file.display()
            );
            ExitCode::FAILURE
        }
        Err(e) => fail(&e.to_string()),
    }
}
