//! CLI `alacod` : le point d'entrée unique de l'engine pour un humain, un script ou un agent IA
//! (`alacod --help` liste tout ; guide : `docs/editeur-vocal.md`, `docs/conventions.md` §90).
//! `alacod lint <dossier-du-jeu>` valide le contenu d'un jeu (manifeste, registre, références, plages, kinds)
//! sans lancer le moteur (`docs/plan-engine.md` §4.2, carte T1.5) ; `get`/`set`/`undo` lisent et règlent le RON ;
//! `sandbox`/`editor`/`remote` lancent et pilotent la partie. Code de sortie 1 et messages sur stderr en cas
//! d'erreur, 0 sinon (`remote` : 2 usage, 3 jeu injoignable, 4 assertion fausse).

mod engine;
mod run;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use content::load_and_lint;
use engine::{GitRef, Source};

#[derive(Parser)]
#[command(
    name = "alacod",
    version,
    about = "Outils de contenu et pilotage du bac à sable de l'engine alacod",
    after_help = "\
Exemples :
  alacod new mon-jeu && cd mon-jeu     crée un jeu ; `alacod lint` valide son contenu
  alacod sandbox games/zombies --detach  lance le bac à sable ; `alacod remote brief` le lit
  alacod set games/zombies Character zombie_1 movement.max_speed 100 --commit
                                       règle UNE valeur (lint, restauration si rouge, commit `editor:`)
  alacod undo                          annule le dernier commit `editor:`
  alacod remote run games/zombies/scripts/monde_test.alacod
                                       joue un fichier de commandes (spawn, give, wait, expect…)

Codes de sortie : 0 réussi, 1 erreur, 2 usage, 3 jeu injoignable (remote), 4 assertion fausse (remote run).
Guide complet : docs/editeur-vocal.md"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Valide le contenu d'un jeu (manifeste, registre, références, plages).
    ///
    /// `alacod lint games/zombies` ; sans chemin : le jeu qui contient le dossier courant. Code de sortie 1 s'il y
    /// a des erreurs.
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
    /// Joue les scénarios générés d'un jeu (armes, personnages, objets, salles, boss).
    ///
    /// `alacod test [jeu] [id…] [--list]`. Sans id, tous ; avec un id (`amulette` ou `item_amulette`), seulement
    /// celui-là. Les traces de référence sont ignorées sauf `--bless`.
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
    /// Crée une entrée de contenu à partir d'un modèle du jeu.
    ///
    /// `alacod new-content [jeu] <Kind> <id> [--from <modèle>]`.
    NewContent {
        /// `[dossier du jeu] <Kind> <id>`.
        args: Vec<String>,
        /// Entrée à copier (défaut : la première du kind).
        #[arg(long)]
        from: Option<String>,
    },
    /// Écrit l'arène vide du bac à sable (murs + joueurs, rien d'autre).
    ///
    /// `alacod new-arena [jeu] [--name arene_vide] [--size 26x19] [--players 4] [--use]`, à partir des cartes du jeu.
    NewArena {
        /// Dossier du jeu (contient `assets/game.ron`) ; sans chemin : le jeu du dossier courant.
        game_dir: Option<PathBuf>,
        /// Id de la carte (nom du fichier sans `.ldtk`).
        #[arg(long, default_value = content::arena::DEFAULT_ARENA_NAME)]
        name: String,
        /// Taille en cases `LxH` (26x19 = 416x304 px : le joueur 0 apparaît en (424, 696)).
        #[arg(long)]
        size: Option<String>,
        /// Nombre de points de départ de joueurs (1 à 4).
        #[arg(long, default_value_t = 4)]
        players: u32,
        /// Carte donneuse des définitions LDtk (tilesets, couches, entités) ; défaut : l'arène du bac à sable
        /// du jeu, sa carte de gabarit, sa première carte.
        #[arg(long)]
        from: Option<String>,
        /// Fichier écrit, relatif à `assets/` (défaut : `maps/<nom>.ldtk`, ou le dossier du donneur).
        #[arg(long)]
        out: Option<String>,
        /// Pose `sandbox: (arena: "<carte>")` dans `game.ron` : le bac à sable du jeu s'ouvre sur cette arène.
        #[arg(long = "use")]
        use_arena: bool,
    },
    /// Construit et lance le jeu en bac à sable (arène vide + profil).
    ///
    /// `alacod sandbox [jeu] [--profile lourd] [--seed N] [--headless] [--detach]`. Le pilotage se fait ensuite par
    /// `alacod remote …`.
    Sandbox {
        /// Dossier du jeu (a un `Cargo.toml`) ; sans chemin : le jeu du dossier courant.
        game_dir: Option<PathBuf>,
        /// Profil `sandbox/<profil>.ron` (défaut du jeu : `arene`).
        #[arg(long)]
        profile: Option<String>,
        /// Graine de carte imposée (`--seed`).
        #[arg(long)]
        seed: Option<i64>,
        /// Sans fenêtre ni GPU (VPS, CI).
        #[arg(long)]
        headless: bool,
        /// Lance le jeu en arrière-plan (groupe de processus à part, journal dans le dossier temporaire) et
        /// rend la main quand le remote répond.
        #[arg(long)]
        detach: bool,
        /// Avec `--detach` : secondes d'attente du remote (compilation à froid comprise).
        #[arg(long, default_value_t = 900)]
        wait: u64,
        /// Arguments passés au jeu après `--sandbox` (`-- --lobby x`).
        #[arg(last = true)]
        game_args: Vec<String>,
    },
    /// Construit et lance le jeu complet (cartes, vagues) en mode éditeur.
    ///
    /// `alacod editor [jeu] [--seed N] [--headless] [--detach]` : partie en marche, clavier actif. Pilotage par
    /// `alacod remote …`.
    Editor {
        /// Dossier du jeu (a un `Cargo.toml`) ; sans chemin : le jeu du dossier courant.
        game_dir: Option<PathBuf>,
        /// Graine de carte imposée (`--seed`).
        #[arg(long)]
        seed: Option<i64>,
        /// Sans fenêtre ni GPU (VPS, CI).
        #[arg(long)]
        headless: bool,
        /// Lance le jeu en arrière-plan et rend la main quand le remote répond.
        #[arg(long)]
        detach: bool,
        /// Avec `--detach` : secondes d'attente du remote.
        #[arg(long, default_value_t = 900)]
        wait: u64,
        /// Arguments passés au jeu (`-- --lobby x`).
        #[arg(last = true)]
        game_args: Vec<String>,
    },
    /// Pilote le jeu en cours (bac à sable ou éditeur) : état, spawn, give, step, restart, script.
    ///
    /// Le remote BRP écoute sur 127.0.0.1:15702 (`ALACOD_REMOTE_PORT`) : `alacod remote
    /// brief|give|spawn|restart|reload|modifier|screenshot|run|…`. `--json` donne une enveloppe JSON ;
    /// codes de sortie : 0 réussi, 1 erreur, 3 jeu injoignable, 4 assertion fausse.
    Remote {
        /// Sortie JSON sur stdout : `{"ok": true, "result": …}` ou `{"ok": false, "error": …, "code": N}`.
        #[arg(long, global = true)]
        json: bool,
        /// Port du jeu (défaut : `ALACOD_REMOTE_PORT`, sinon 15702).
        #[arg(long, global = true)]
        port: Option<u16>,
        #[command(subcommand)]
        command: content::remote::RemoteCommand,
    },
    /// Inventaire du vocabulaire d'un jeu : kinds, ids, champs.
    ///
    /// `alacod describe [games/gungeon] [Kind]`.
    Describe {
        /// `[dossier du jeu] [Kind]` : le premier argument est pris pour un dossier s'il
        /// existe, sinon pour un kind (`alacod describe Item` dans un dossier de jeu).
        args: Vec<String>,
        /// Sortie JSON sur stdout.
        #[arg(long)]
        json: bool,
    },
    /// Lit la valeur d'un champ d'un fichier RON de contenu.
    ///
    /// `alacod get [jeu] <Kind> <id> <chemin.du.champ>`, ex. `alacod get games/zombies Character zombie_1
    /// movement.max_speed`. Le chemin traverse structs, tables, listes
    /// (`modifiers.0.value`) et variantes ; un bloc rend son texte et la liste des réglages qu'il contient.
    Get {
        /// `[dossier du jeu] <Kind> <id> <chemin.du.champ>`.
        args: Vec<String>,
        /// Sortie JSON sur stdout.
        #[arg(long)]
        json: bool,
    },
    /// Règle UNE valeur d'un fichier RON de contenu (lint, restauration si rouge, commit possible).
    ///
    /// `alacod set [jeu] <Kind> <id> <chemin.du.champ> <valeur>`, ex. `alacod set games/zombies Character zombie_1
    /// movement.max_speed 100 --commit`. Remplacement textuel (commentaires et mise en
    /// forme du fichier intacts), un `Fixed` reste une chaîne (`"100.0"`). Le lint de `alacod lint` tourne
    /// ensuite : s'il apparaît une erreur nouvelle, le fichier est restauré. Un chemin inconnu ou ambigu est
    /// refusé avec les chemins possibles. Dit quand l'effet se voit (immédiat, ou `alacod remote restart`).
    ///
    /// Valeurs relatives (nombre ou `Fixed` seulement) : `x0.9` ou `*0.9` (facteur), `+10%` / `-10%` (pourcentage,
    /// signe obligatoire), `+=5` / `-=5` (ajout). `+5` et `-5` restent des valeurs absolues. Un `Fixed` reste une
    /// chaîne (`"0.15"` x0.9 → `"0.135"`), un entier reste un entier : un résultat fractionnaire est refusé avec les
    /// deux entiers voisins. La sortie montre l'ancienne et la nouvelle valeur.
    Set {
        /// `[dossier du jeu] <Kind> <id> <chemin.du.champ> <valeur>` (valeur absolue ou relative : `x0.9`, `-10%`, `+=5`).
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
        /// Commit `editor: <Kind> <id> <chemin> <ancien> -> <nouveau>` du seul fichier touché (refusé si ce fichier
        /// a déjà des changements non commités).
        #[arg(long)]
        commit: bool,
        /// Écrit `<valeur>` telle quelle à la place du nœud désigné (un bloc, `Some("500.0")`, `None`…).
        #[arg(long)]
        raw: bool,
        /// Sortie JSON sur stdout.
        #[arg(long)]
        json: bool,
    },
    /// Annule le dernier commit `editor:` (et lui seul) : `alacod undo [jeu]`.
    ///
    /// `git revert` du dernier commit si son sujet commence par `editor:`, sinon refus sans rien toucher ; relance
    /// ensuite le lint du jeu.
    Undo {
        /// Dossier du jeu ou du dépôt ; sans chemin : le dossier courant.
        game_dir: Option<PathBuf>,
        /// Sortie JSON sur stdout.
        #[arg(long)]
        json: bool,
    },
    /// Dit en une ligne si le binaire `alacod` installé est plus ancien que le fichier le plus récent de
    /// `crates/content/src` et `crates/game/src` de l'engine dont il a été compilé (code 1 : périmé, à réinstaller).
    SelfCheck,
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
        } => match new_game(
            &name,
            engine,
            NewSource {
                tag,
                rev,
                url,
                path,
            },
        ) {
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
        Command::Get { args, json } => get_cmd(args, json),
        Command::Set {
            args,
            raw,
            commit,
            json,
        } => set_cmd(args, raw, commit, json),
        Command::Undo { game_dir, json } => undo_cmd(game_dir, json),
        Command::NewArena {
            game_dir,
            name,
            size,
            players,
            from,
            out,
            use_arena,
        } => match resolve_game_dir(game_dir) {
            Ok(dir) => new_arena(&dir, name, size.as_deref(), players, from, out, use_arena),
            Err(e) => fail(&e),
        },
        Command::Sandbox {
            game_dir,
            profile,
            seed,
            headless,
            detach,
            wait,
            game_args,
        } => launch_game(content::launch::GameLaunch {
            kind: content::launch::LaunchKind::Sandbox,
            game_dir: game_dir.unwrap_or_default(),
            profile,
            seed,
            headless,
            detach,
            wait_secs: wait,
            game_args,
        }),
        Command::Editor {
            game_dir,
            seed,
            headless,
            detach,
            wait,
            game_args,
        } => launch_game(content::launch::GameLaunch {
            kind: content::launch::LaunchKind::Editor,
            game_dir: game_dir.unwrap_or_default(),
            profile: None,
            seed,
            headless,
            detach,
            wait_secs: wait,
            game_args,
        }),
        Command::Remote {
            json,
            port,
            command,
        } => ExitCode::from(content::remote::run_cli(&command, json, port)),
        Command::SelfCheck => self_check(),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

/// `alacod sandbox` et `alacod editor` : le dossier du jeu est celui d'`assets/game.ron` (argument vide : le
/// jeu du dossier courant).
fn launch_game(mut launch: content::launch::GameLaunch) -> ExitCode {
    let arg = (!launch.game_dir.as_os_str().is_empty()).then(|| launch.game_dir.clone());
    launch.game_dir = match resolve_game_dir(arg) {
        Ok(dir) => dir,
        Err(e) => return fail(&e),
    };
    match content::launch::run(&launch) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(e) => fail(&e),
    }
}

/// `alacod new-arena` : écrit l'arène vide et dit où elle est, d'où elle vient et ce qui reste à faire.
fn new_arena(
    game_dir: &std::path::Path,
    name: String,
    size: Option<&str>,
    players: u32,
    from: Option<String>,
    out: Option<String>,
    use_arena: bool,
) -> ExitCode {
    let mut spec = content::arena::ArenaSpec {
        name,
        players,
        ..Default::default()
    };
    if let Some(size) = size {
        let parsed = size.split_once(['x', 'X']).and_then(|(w, h)| {
            Some((w.trim().parse::<u32>().ok()?, h.trim().parse::<u32>().ok()?))
        });
        let Some((w, h)) = parsed else {
            return fail(&format!(
                "--size « {size} » : attendu LxH en cases (ex. 26x19)"
            ));
        };
        spec.width_tiles = w;
        spec.height_tiles = h;
    }
    let request = content::arena::ArenaRequest {
        spec,
        from,
        out,
        set_default: use_arena,
    };
    let report = match content::arena::new_arena(game_dir, &request) {
        Ok(report) => report,
        Err(e) => return fail(&e),
    };
    println!(
        "{} : {} (définitions de {})",
        report.map,
        if report.unchanged {
            "inchangée"
        } else {
            "écrite"
        },
        report.donor.display()
    );
    if report.manifest_updated {
        println!("game.ron : sandbox: (arena: \"{}\")", report.map);
    } else if !report.is_default {
        println!(
            "pour l'ouvrir au lancement du bac à sable : `sandbox: (arena: \"{}\")` dans game.ron (ou --use)",
            report.map
        );
    }
    if !report.declared {
        println!(
            "attention : le registre du jeu ne connaît pas cette carte ; ajouter \
             `(path: \"{}\", kind: \"Map\")` à `content_folders` de game.ron",
            report.map
        );
    }
    ExitCode::SUCCESS
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

/// Le fichier le plus récent sous `dir` (récursif) : sa date de modification et son chemin.
fn newest_file(dir: &std::path::Path) -> Option<(std::time::SystemTime, PathBuf)> {
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let candidate = if path.is_dir() {
            newest_file(&path)
        } else {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .map(|t| (t, path))
        };
        if let Some(candidate) = candidate {
            if newest.as_ref().map_or(true, |(t, _)| candidate.0 > *t) {
                newest = Some(candidate);
            }
        }
    }
    newest
}

/// `alacod self-check` : binaire installé (date de `current_exe`) contre les sources de `crates/content/src` et
/// `crates/game/src` du dépôt qui a fourni le gabarit (`TEMPLATE_ENGINE`) ; code 1 si le binaire est plus ancien.
fn self_check() -> ExitCode {
    let reinstall = "env -u CI cargo install --path crates/content --bin alacod --locked";
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => return fail(&format!("self-check : binaire introuvable : {e}")),
    };
    let built = match std::fs::metadata(&exe).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(e) => return fail(&format!("self-check : {} : {e}", exe.display())),
    };
    let root =
        std::fs::canonicalize(TEMPLATE_ENGINE).unwrap_or_else(|_| PathBuf::from(TEMPLATE_ENGINE));
    let newest = ["crates/content/src", "crates/game/src"]
        .iter()
        .filter_map(|rel| newest_file(&root.join(rel)))
        .max_by_key(|(t, _)| *t);
    let Some((source_time, source)) = newest else {
        println!(
            "alacod self-check : sources de l'engine introuvables sous {} : rien à comparer",
            root.display()
        );
        return ExitCode::SUCCESS;
    };
    let source_rel = source
        .strip_prefix(&root)
        .unwrap_or(&source)
        .display()
        .to_string();
    match source_time.duration_since(built) {
        Ok(late) if !late.is_zero() => {
            let minutes = late.as_secs() / 60;
            println!(
                "alacod PÉRIMÉ : {} est plus ancien de {} h {:02} min que {source_rel} ; réinstaller depuis {} : {reinstall}",
                exe.display(),
                minutes / 60,
                minutes % 60,
                root.display()
            );
            ExitCode::FAILURE
        }
        _ => {
            println!(
                "alacod à jour : {} est plus récent que crates/content/src et crates/game/src ({}, dernier : {source_rel})",
                exe.display(),
                root.display()
            );
            ExitCode::SUCCESS
        }
    }
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

/// `[dossier du jeu] <n mots>` : le dossier est celui qui précède les `n` mots, s'il y en a un de plus.
fn split_game_args(
    args: Vec<String>,
    n: usize,
    usage: &str,
) -> Result<(Option<PathBuf>, Vec<String>), String> {
    if args.len() == n {
        Ok((None, args))
    } else if args.len() == n + 1 {
        let mut words = args.into_iter();
        let dir = words.next().map(PathBuf::from);
        Ok((dir, words.collect()))
    } else {
        Err(format!("usage : {usage}"))
    }
}

/// Erreur d'une commande : texte sur stderr, ou `{"ok": false, "error": …}` sur stdout avec `--json` ; code 1.
fn fail_as(message: &str, json: bool) -> ExitCode {
    if json {
        println!("{}", serde_json::json!({"ok": false, "error": message}));
        ExitCode::FAILURE
    } else {
        fail(message)
    }
}

/// `alacod get` : la valeur actuelle d'un champ d'un fichier RON de contenu.
fn get_cmd(args: Vec<String>, json: bool) -> ExitCode {
    let usage = "alacod get [jeu] <Kind> <id> <chemin.du.champ>";
    let (dir_arg, words) = match split_game_args(args, 3, usage) {
        Ok(split) => split,
        Err(e) => return fail_as(&e, json),
    };
    let dir = match resolve_game_dir(dir_arg) {
        Ok(dir) => dir,
        Err(e) => return fail_as(&e, json),
    };
    match content::edit::get(&dir, &words[0], &words[1], &words[2]) {
        Ok(reading) if json => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": true, "kind": reading.kind, "id": reading.id, "path": reading.path,
                    "file": reading.file, "value": reading.value, "raw": reading.raw,
                    "scalar": reading.scalar, "leaves": reading.leaves,
                })
            );
            ExitCode::SUCCESS
        }
        Ok(reading) => {
            println!("{}", reading.raw);
            if !reading.scalar {
                println!("réglages : {}", reading.leaves.join(", "));
            }
            ExitCode::SUCCESS
        }
        Err(e) => fail_as(&e, json),
    }
}

/// `alacod set` : règle une valeur, lint, restaure si le lint s'aggrave, commit si demandé.
fn set_cmd(args: Vec<String>, raw: bool, commit: bool, json: bool) -> ExitCode {
    // Les mots acceptent un `-` en tête (`-10%`, `-=5`, `-5`) : clap range alors dans `args` les drapeaux écrits
    // après la valeur ; on les reprend ici.
    let (mut raw, mut commit, mut json) = (raw, commit, json);
    let mut words = Vec::with_capacity(args.len());
    for arg in args {
        match arg.as_str() {
            "--raw" => raw = true,
            "--commit" => commit = true,
            "--json" => json = true,
            _ => words.push(arg),
        }
    }
    let args = words;
    let usage = "alacod set [jeu] <Kind> <id> <chemin.du.champ> <valeur> [--commit] [--raw]";
    let (dir_arg, words) = match split_game_args(args, 4, usage) {
        Ok(split) => split,
        Err(e) => return fail_as(&e, json),
    };
    let dir = match resolve_game_dir(dir_arg) {
        Ok(dir) => dir,
        Err(e) => return fail_as(&e, json),
    };
    let options = content::edit::SetOptions { raw, commit };
    match content::edit::set(&dir, &words[0], &words[1], &words[2], &words[3], &options) {
        Ok(change) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true, "kind": change.kind, "id": change.id, "path": change.path,
                        "file": change.file, "old": change.old, "new": change.new,
                        "changed": change.changed, "effect": change.effect, "commit": change.commit,
                        "preexisting_lint_errors": change.preexisting_lint_errors,
                    })
                );
            } else {
                println!("{}", change.line());
                if change.preexisting_lint_errors > 0 {
                    eprintln!(
                        "lint : {} erreur(s) déjà présente(s) avant ce changement, aucune nouvelle (alacod lint)",
                        change.preexisting_lint_errors
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => fail_as(&e, json),
    }
}

/// `alacod undo` : annule le dernier commit `editor:` (et lui seul), puis relance le lint du jeu s'il y en a un.
fn undo_cmd(game_dir: Option<PathBuf>, json: bool) -> ExitCode {
    let repo_dir = game_dir.clone().unwrap_or_else(|| PathBuf::from("."));
    let undone = match content::edit::undo(&repo_dir) {
        Ok(undone) => undone,
        Err(e) => return fail_as(&e, json),
    };
    let lint = resolve_game_dir(game_dir)
        .ok()
        .map(|dir| load_and_lint(&dir).map(|(_, _, errors)| errors));
    let errors: Vec<String> = match &lint {
        Some(Ok(errors)) => errors.iter().map(ToString::to_string).collect(),
        Some(Err(e)) => vec![e.to_string()],
        None => Vec::new(),
    };
    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": errors.is_empty(), "undone": undone.subject, "commit": undone.commit,
                "effect": undone.effect, "linted": lint.is_some(), "lint_errors": errors,
            })
        );
    } else {
        println!(
            "annulé : {} (commit {}) ; effet : {}",
            undone.subject, undone.commit, undone.effect
        );
        match (&lint, errors.is_empty()) {
            (None, _) => {}
            (Some(_), true) => println!("lint : aucune erreur"),
            (Some(_), false) => {
                for e in &errors {
                    eprintln!("{e}");
                }
                eprintln!("lint : {} erreur(s) après l'annulation", errors.len());
            }
        }
    }
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
