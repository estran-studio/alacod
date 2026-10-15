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
                eprintln!("kind « {name} » absent de {} (présents : {})", desc.game, known.join(", "));
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
    let valid = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase())
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
