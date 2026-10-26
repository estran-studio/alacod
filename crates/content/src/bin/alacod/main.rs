//! CLI `alacod` : `alacod lint <dossier-du-jeu>` valide le contenu d'un jeu (manifeste,
//! registre, références, plages, kinds) sans lancer le moteur (`docs/plan-engine.md` §4.2,
//! carte T1.5). Code de sortie 1 et messages sur stderr en cas d'erreur, 0 sinon.

mod engine;
mod remote;
mod run;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use content::load_and_lint;
use engine::{GitRef, Source};

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
    /// Crée un nouveau jeu hors de l'engine : `alacod new mon-jeu`. Le jeu dépend de l'engine par un **tag
    /// git** (le dernier de l'engine, `--tag`/`--rev` pour un autre) ; `alacod engine local <chemin>`
    /// développe ensuite engine et jeu ensemble.
    New {
        /// Nom du jeu (minuscules, chiffres, `-` et `_`) ; crée le dossier `./<nom>`.
        name: String,
        /// Dépôt de l'engine qui fournit le gabarit, les tags et le `Cargo.lock` (contient `crates/` et
        /// `templates/starter`). Par défaut : celui d'où ce binaire a été compilé.
        #[arg(long)]
        engine: Option<PathBuf>,
        /// Tag de l'engine à épingler (défaut : le dernier tag du dépôt de l'engine).
        #[arg(long, conflicts_with_all = ["rev", "path"])]
        tag: Option<String>,
        /// Révision git de l'engine à épingler (à la place d'un tag).
        #[arg(long, conflicts_with = "path")]
        rev: Option<String>,
        /// URL git de l'engine écrite dans le jeu (défaut : `origin` du dépôt de l'engine).
        #[arg(long, conflicts_with = "path")]
        url: Option<String>,
        /// Dépendance par **chemin** vers le dépôt de l'engine au lieu d'un tag git (hors ligne : tests,
        /// poste sans réseau). Le chemin absolu est écrit dans le `Cargo.toml` versionné.
        #[arg(long)]
        path: bool,
    },
    /// Version d'engine d'un jeu : `alacod engine status | pin <tag|rev> | local <chemin>`.
    Engine {
        #[command(subcommand)]
        action: EngineAction,
        /// Dossier du jeu (défaut : le jeu qui contient le dossier courant).
        #[arg(long, global = true)]
        game: Option<PathBuf>,
    },
    /// Lance le jeu : `alacod run [dossier] [--sandbox [profil]] [--headless] [--seed N] [--players N] [-- args…]`.
    /// `cargo run` dans le dossier du jeu ; sans fenêtre avec `--headless` (`ALACOD_HEADLESS=1`, sans rendu).
    Run {
        /// Dossier du jeu (défaut : le jeu qui contient le dossier courant). À donner **avant** les options.
        game_dir: Option<PathBuf>,
        /// Bac à sable : l'arène du jeu et le profil `sandbox/<profil>.ron` (`arene` sans profil), mode éditeur.
        #[arg(long, value_name = "PROFIL", num_args = 0..=1, default_missing_value = content::sandbox::DEFAULT_PROFILE)]
        sandbox: Option<String>,
        /// Sans fenêtre ni GPU, compilé sans le rendu (une frame par update).
        #[arg(long)]
        headless: bool,
        /// Graine de la carte.
        #[arg(long, allow_hyphen_values = true)]
        seed: Option<i32>,
        /// Nombre de joueurs locaux (un par défaut).
        #[arg(long)]
        players: Option<u32>,
        /// Arguments passés tels quels au jeu (`ALACOD_*` : variables d'environnement, héritées).
        #[arg(last = true)]
        game_args: Vec<String>,
    },
    /// Pilote un jeu lancé (`alacod run --sandbox`) par ses méthodes BRP `alacod/*` : `alacod remote brief`,
    /// `step 30`, `give --weapon x`, `spawn <perso> --ahead`, `profile`, `save <scénario>`…
    Remote {
        /// Port du remote (défaut : `ALACOD_REMOTE_PORT`, sinon 15702).
        #[arg(long, global = true)]
        port: Option<u16>,
        #[command(subcommand)]
        command: remote::RemoteCommand,
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
        /// Racine de l'engine ; sinon celle que le jeu compile réellement (`cargo metadata`).
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

#[derive(Subcommand)]
enum EngineAction {
    /// Dépendance déclarée, surcharge locale éventuelle, racine réellement compilée.
    Status,
    /// Épingle l'engine sur un tag ou une révision git et retire la surcharge locale.
    Pin {
        /// Tag (`v0.1.0`) ou révision (7 à 40 chiffres hexadécimaux).
        reference: String,
        /// URL git de l'engine (défaut : celle de la dépendance actuelle).
        #[arg(long)]
        url: Option<String>,
    },
    /// Compile l'engine d'un dépôt local (surcharge hors du `Cargo.toml` versionné : `.cargo/config.toml`).
    Local {
        /// Racine de l'engine (contient `crates/` et `templates/starter`).
        path: PathBuf,
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
            tag,
            rev,
            url,
            path,
        } => match new_game(&name, engine, NewSource { tag, rev, url, path }) {
            Ok((dir, description)) => {
                println!("{} créé ({description}).", dir.display());
                println!("  cd {name} && alacod lint && alacod describe");
                println!("  alacod run --sandbox   # premier build long (bevy)");
                ExitCode::SUCCESS
            }
            Err(e) => fail(&e),
        },
        Command::Engine { action, game } => match resolve_game_dir(game) {
            Ok(dir) => engine_command(&dir, action),
            Err(e) => fail(&e),
        },
        Command::Run {
            game_dir,
            sandbox,
            headless,
            seed,
            players,
            game_args,
        } => match resolve_game_dir(game_dir) {
            Ok(dir) => match run::run(
                &dir,
                &run::RunOptions {
                    sandbox,
                    headless,
                    seed,
                    players,
                    passthrough: game_args,
                },
            ) {
                Ok(0) => ExitCode::SUCCESS,
                Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
                Err(e) => fail(&e),
            },
            Err(e) => fail(&e),
        },
        Command::Remote { port, command } => {
            let result = remote::port_from(port)
                .and_then(|port| remote::run(&remote::Client::new(port), command));
            match result {
                Ok(text) => {
                    if !text.is_empty() {
                        println!("{text}");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => fail(&e),
            }
        }
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

/// Dépôt de l'engine d'où `alacod` a été compilé : source du gabarit de `alacod new` quand `--engine` n'est
/// pas donné. Les outils d'un jeu existant (`test`, `run`…) ne s'en servent **pas** : leur racine est celle
/// que le jeu compile (`engine::resolve_root`).
const TEMPLATE_ENGINE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// Comment `alacod new` écrit la dépendance : tag git (défaut), révision, ou chemin.
struct NewSource {
    tag: Option<String>,
    rev: Option<String>,
    url: Option<String>,
    path: bool,
}

/// `alacod new` : copie `<engine>/templates/starter` dans `./<name>` en remplaçant `{{name}}`
/// et `{{engine}}` dans les fichiers texte, et reprend le `Cargo.lock` de l'engine (mêmes
/// versions de dépendances que l'engine, qui compile). Rend le dossier et la dépendance écrite.
fn new_game(
    name: &str,
    engine: Option<PathBuf>,
    how: NewSource,
) -> Result<(PathBuf, String), String> {
    let valid = name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid {
        return Err(format!(
            "nom de jeu « {name} » invalide : minuscules, chiffres, `-` et `_`, première lettre minuscule"
        ));
    }
    let engine = engine.unwrap_or_else(|| PathBuf::from(TEMPLATE_ENGINE));
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
    let source = if how.path {
        Source::Path(engine_str.clone())
    } else {
        let url = match how.url {
            Some(url) => url,
            None => engine::git_output(&engine, &["remote", "get-url", "origin"])?,
        };
        let reference = match (how.rev, how.tag) {
            (Some(rev), _) => GitRef::Rev(rev),
            (None, Some(tag)) => GitRef::Tag(tag),
            (None, None) => GitRef::Tag(engine::latest_tag(&engine)?),
        };
        // Le gabarit vient du dépôt de l'engine tel qu'il est : s'il a avancé depuis le tag, le jeu créé
        // peut demander à l'engine épinglé des choses qu'il n'a pas encore.
        if let GitRef::Tag(tag) = &reference {
            let tagged = engine::git_output(&engine, &["rev-list", "-n", "1", tag]).ok();
            let head = engine::git_output(&engine, &["rev-parse", "HEAD"]).ok();
            if tagged.is_some() && tagged != head {
                eprintln!(
                    "attention : le gabarit vient de {} (HEAD), plus récent que le tag {tag} épinglé ; \
                     `alacod engine local {}` pour développer sur cette version",
                    engine.display(),
                    engine.display()
                );
            }
        }
        Source::Git { url, reference }
    };
    let description = match &source {
        Source::Path(root) => format!("dépendance par chemin {root}"),
        Source::Git { url, reference } => {
            format!("engine {url} {} {}", reference.key(), reference.value())
        }
    };
    copy_template(&template, &target, name, &engine_str, &source).map_err(|e| e.to_string())?;
    let lock = engine.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, target.join("Cargo.lock")).map_err(|e| e.to_string())?;
    }
    Ok((target, description))
}

fn copy_template(
    from: &std::path::Path,
    to: &std::path::Path,
    name: &str,
    engine: &str,
    source: &Source,
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

/// `alacod engine …` : affiche le résultat, code de sortie 1 sur erreur.
fn engine_command(game_dir: &std::path::Path, action: EngineAction) -> ExitCode {
    let result = match action {
        EngineAction::Status => engine::status(game_dir).map(|lines| lines.join("\n")),
        EngineAction::Pin { reference, url } => engine::pin(game_dir, &reference, url),
        EngineAction::Local { path } => engine::local(game_dir, &path),
    };
    match result {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => fail(&e),
    }
}

/// Racine de l'engine pour un jeu : `--engine`, sinon celle que le jeu compile réellement, lue par
/// `cargo metadata` (checkout git du tag épinglé, surcharge `alacod engine local`, ou dépendance par chemin).
fn engine_root_for(
    game_dir: &std::path::Path,
    explicit: Option<PathBuf>,
) -> Result<PathBuf, String> {
    match explicit {
        Some(root) => root
            .canonicalize()
            .map_err(|e| format!("engine introuvable ({}) : {e}", root.display())),
        None => engine::resolve_root(game_dir),
    }
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
    .arg(engine.join("Cargo.toml"));
    // Engine épinglé par un tag : son checkout est dans le cache git de cargo, on n'y écrit pas de `target/`.
    if engine::is_cargo_checkout(&engine) {
        cmd.arg("--target-dir")
            .arg(game.join("target").join("alacod-engine"));
    }
    cmd.arg("--").arg(&game);
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
