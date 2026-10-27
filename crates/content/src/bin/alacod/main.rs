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
    /// Joue les scénarios d'un jeu **par son propre binaire** (plugin Rust compris) : générés
    /// (`scenarios gen --play`, écrits dans `<jeu>/tests/generated`) **et** écrits à la main
    /// (`scenarios play`, `<jeu>/tests/scenarios`), avec leurs traces (`<scénario>.trace` comparée,
    /// `--bless` pour l'écrire). Rien n'est écrit hors du jeu.
    ///
    /// `alacod test [jeu] [id…] [--list]`. Sans id, tous ; avec un id (`amulette`, `item_amulette`
    /// ou le nom d'un scénario écrit à la main), seulement ceux-là.
    Test {
        /// `[dossier du jeu] [id…]` : le premier argument est pris pour un dossier s'il existe.
        args: Vec<String>,
        /// Liste les scénarios testables (générés puis écrits à la main) au lieu de les jouer.
        #[arg(long)]
        list: bool,
        /// Écrit les traces de référence (changement de gameplay voulu).
        #[arg(long)]
        bless: bool,
        /// Sortie JSON (tableau de résultats) sur stdout.
        #[arg(long)]
        json: bool,
        /// Compatibilité X-2a, sans effet : le jeu compile l'engine qu'il épingle
        /// (`alacod engine status`), jamais celui d'`--engine`.
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
    /// Rend un scénario du jeu en vidéo **par son propre binaire** (plugin compris) : capture
    /// (`scenarios capture`, images + `events.json`) puis encodage ffmpeg, le tout côté jeu
    /// (défaut : `<jeu>/target/videos/<scénario>.mp4`). Demande un GPU (Mac : Metal ; Linux :
    /// Vulkan) et `ffmpeg` ; sans eux, échec expliqué, rien d'inventé.
    ///
    /// `alacod video [jeu] <scénario> [--out <dossier>] [--every 2] [--follow <handle>] [--size 960x540]`.
    /// `<scénario>` : nom d'un scénario écrit à la main (`tests/scenarios`) ou généré
    /// (`tests/generated`), ou chemin d'un `.ron` du jeu.
    Video {
        /// `[dossier du jeu] <scénario>`.
        args: Vec<String>,
        /// Dossier des vidéos (défaut : `<jeu>/target/videos`).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Une image toutes les N frames de simulation (défaut 2 → 30 images/s).
        #[arg(long, default_value_t = 2)]
        every: u32,
        /// Force la caméra à suivre le joueur avec ce handle GGRS.
        #[arg(long)]
        follow: Option<usize>,
        /// Taille des images `<largeur>x<hauteur>` (défaut 960x540).
        #[arg(long)]
        size: Option<String>,
    },
    /// Construit le jeu (`cargo build` dans son dossier) avec sa version (`APP_VERSION`) : binaire
    /// natif, `--release` pour l'optimisé, `--web` pour le wasm (`wasm32-unknown-unknown`,
    /// `wasm-bindgen`, assets copiés, `build.json`, comme `scripts/build-web-games.mjs` mais côté
    /// jeu : `<jeu>/target/web/<version>/<jeu>/`). Artefacts seulement dans le jeu.
    Build {
        /// Dossier du jeu (défaut : le jeu qui contient le dossier courant).
        game_dir: Option<PathBuf>,
        /// Profil `release` (natif et web).
        #[arg(long)]
        release: bool,
        /// Build web au lieu du binaire natif.
        #[arg(long)]
        web: bool,
    },
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
        Command::Video {
            args,
            out,
            every,
            follow,
            size,
        } => {
            let (dir_arg, scenario) = match args.as_slice() {
                [one] => (None, one.clone()),
                [dir, name] if PathBuf::from(dir).is_dir() => {
                    (Some(PathBuf::from(dir)), name.clone())
                }
                _ => return fail("usage : alacod video [jeu] <scénario> [--out <dossier>]"),
            };
            match resolve_game_dir(dir_arg) {
                Ok(dir) => run_video(&dir, &scenario, out, every, follow, size.as_deref()),
                Err(e) => fail(&e),
            }
        }
        Command::Build {
            game_dir,
            release,
            web,
        } => match resolve_game_dir(game_dir) {
            Ok(dir) => run_build(&dir, release, web),
            Err(e) => fail(&e),
        },
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

/// `alacod test` (X-2b) : joue les scénarios du jeu **par son propre binaire** (plugin Rust compris),
/// générés (`scenarios gen --play`, écrits dans `<jeu>/tests/generated`) **et** écrits à la main
/// (`scenarios play`, `<jeu>/tests/scenarios`), avec leurs traces (`<scénario>.trace` comparée,
/// `--bless` pour l'écrire). Tout s'écrit dans le jeu : jamais dans l'engine.
fn run_tests(
    game_dir: &std::path::Path,
    ids: &[String],
    list: bool,
    bless: bool,
    json: bool,
    engine: Option<PathBuf>,
) -> ExitCode {
    if let Some(root) = engine {
        if let Err(e) = root.canonicalize() {
            return fail(&format!("--engine inutilisé ({}) : {e}", root.display()));
        }
        eprintln!(
            "avertissement : --engine est sans effet, le jeu compile l'engine qu'il épingle (`alacod engine status`)"
        );
    }
    let game = match game_dir.canonicalize() {
        Ok(g) => g,
        Err(e) => return fail(&e.to_string()),
    };
    let version = app_version(&game, false);
    if list {
        return list_tests(&game, &version);
    }
    let only = (!ids.is_empty()).then(|| ids.join(","));
    let mut gen_args = vec!["scenarios", "gen", "--play", "--json"];
    if bless {
        gen_args.push("--bless");
    }
    if let Some(only) = &only {
        gen_args.extend(["--only", only]);
    }
    let gen = match game_tool(&game, &version, &gen_args, true) {
        Ok(out) => out,
        Err(e) => return fail(&e),
    };
    let mut play_args = vec!["scenarios", "play", "--json"];
    if bless {
        play_args.push("--bless");
    }
    if let Some(only) = &only {
        play_args.extend(["--only", only]);
    }
    let play = match game_tool(&game, &version, &play_args, true) {
        Ok(out) => out,
        Err(e) => return fail(&e),
    };
    // Un côté qui ne connaît pas ces ids (code 2) laisse l'autre trancher.
    let mut results = Vec::new();
    let mut tool_failed = false;
    for (out, unknown) in [(&gen, "aucun sujet"), (&play, "aucun scénario")] {
        if !out.ok && out.code == Some(2) && out.stderr.contains(unknown) {
            continue;
        }
        match tool_results(out) {
            Some(mut r) => results.append(&mut r),
            None => {
                show_tool_stderr(out);
                return fail("alacod test : aucun résultat (voir ci-dessus)");
            }
        }
        tool_failed |= !out.ok;
    }
    if results.is_empty() {
        show_tool_stderr(&gen);
        show_tool_stderr(&play);
        return fail(&if ids.is_empty() {
            "alacod test : aucun scénario dans ce jeu (ni généré ni tests/scenarios)".to_string()
        } else {
            format!("alacod test : aucun scénario « {} »", ids.join(", "))
        });
    }
    if json {
        println!("{}", serde_json::Value::Array(results.clone()));
    } else {
        for r in &results {
            let ok = !result_failed(r);
            let hits = r
                .get("hits")
                .and_then(|h| h.as_u64())
                .map(|h| format!(", {h:>3} coups"))
                .unwrap_or_default();
            println!(
                "{} {:<28} {:>5} frames{hits}, trace {}",
                if ok { "ok    " } else { "ÉCHEC " },
                r["scenario"].as_str().unwrap_or("?"),
                r["frames"],
                r.get("trace").and_then(|t| t.as_str()).unwrap_or("?"),
            );
            for f in r["failures"].as_array().into_iter().flatten() {
                println!("        {}", f.as_str().unwrap_or("?"));
            }
            if ok && r["expectations"] == 0 {
                println!("        (aucune attente : ce test ne prouve rien ; ajouter `test:` à l'entrée, voir `alacod describe`)");
            }
        }
    }
    let failed = results.iter().filter(|r| result_failed(r)).count();
    if !json {
        println!("{} scénario(s), {} échec(s)", results.len(), failed);
    }
    if failed == 0 && !tool_failed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Un résultat est-il un échec ? Attentes rouges, ou trace ni `ok` ni `blessée` (les traces se
/// comparent par défaut, `--bless` les écrit ; `ignorée` : `--no-trace` passé au binaire à la main).
fn result_failed(r: &serde_json::Value) -> bool {
    if r["expectations_ok"] != true {
        return true;
    }
    !matches!(
        r.get("trace").and_then(|t| t.as_str()),
        Some("ok" | "blessée" | "ignorée")
    )
}

/// `alacod test --list` : sujets générés puis scénarios écrits à la main, sans rien jouer.
fn list_tests(game_dir: &std::path::Path, app_version: &str) -> ExitCode {
    let gen = match game_tool(game_dir, app_version, &["scenarios", "gen", "--list"], true) {
        Ok(out) => out,
        Err(e) => return fail(&e),
    };
    let play = match game_tool(
        game_dir,
        app_version,
        &["scenarios", "play", "--list"],
        true,
    ) {
        Ok(out) => out,
        Err(e) => return fail(&e),
    };
    if !gen.ok {
        show_tool_stderr(&gen);
        return ExitCode::FAILURE;
    }
    if !play.ok {
        show_tool_stderr(&play);
        return ExitCode::FAILURE;
    }
    // Les journaux du jeu partagent stdout : seules les lignes `nom<TAB>…` comptent.
    for line in gen
        .stdout
        .lines()
        .chain(play.stdout.lines())
        .filter(|l| l.contains('\t'))
    {
        println!("{}", line.replace('\t', "  "));
    }
    ExitCode::SUCCESS
}

/// Sortie d'un outillage du jeu (`cargo run -q -- …` dans son dossier).
struct ToolOut {
    ok: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// `cargo run -q -- <args…>` dans le dossier du jeu, avec sa version (`APP_VERSION`).
/// `headless` : sans rendu (`--no-default-features`, profil `headless` redéclaré : le jeu est son
/// propre workspace, voir `run::HEADLESS_PROFILE_CONFIG`). Le `target/` est celui du jeu (défaut de
/// cargo) : rien n'est compilé ni écrit dans l'engine.
fn game_tool(
    game_dir: &std::path::Path,
    app_version: &str,
    tool_args: &[&str],
    headless: bool,
) -> Result<ToolOut, String> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(game_dir)
        .env("APP_VERSION", app_version)
        .arg("run")
        .arg("-q");
    if headless {
        cmd.arg("--no-default-features");
        cmd.args(run::HEADLESS_PROFILE_CONFIG.iter());
        cmd.args(run::HEADLESS_DEPS_CONFIG.iter());
    }
    cmd.arg("--").args(tool_args);
    let output = cmd.output().map_err(|e| format!("cargo : {e}"))?;
    Ok(ToolOut {
        ok: output.status.success(),
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// Fin de la stderr d'un outillage (les lignes du jeu d'abord, sinon celles de cargo).
fn show_tool_stderr(out: &ToolOut) {
    let own: Vec<&str> = out
        .stderr
        .lines()
        .filter(|l| {
            l.starts_with("alacod-gen")
                || l.starts_with("scenarios play")
                || l.starts_with("scenarios capture")
        })
        .collect();
    let lines: Vec<&str> = out.stderr.lines().collect();
    let start = lines.len().saturating_sub(25);
    eprintln!(
        "{}",
        if own.is_empty() {
            lines[start..].join("\n")
        } else {
            own.join("\n")
        }
    );
}

/// Le tableau JSON d'un outillage : la dernière ligne de stdout qui commence par `[`.
fn tool_results(out: &ToolOut) -> Option<Vec<serde_json::Value>> {
    out.stdout
        .lines()
        .rev()
        .find(|l| l.starts_with('['))
        .and_then(|l| serde_json::from_str(l).ok())
}

/// `.ron` d'un scénario du jeu : chemin existant (relatif au jeu ou absolu), sinon
/// `tests/scenarios/<nom>.ron`, sinon `tests/generated/<nom>.ron`. Ambigu (les deux existent) :
/// erreur qui liste les deux chemins.
fn resolve_scenario_file(game_dir: &std::path::Path, scenario: &str) -> Result<PathBuf, String> {
    let direct = game_dir.join(scenario);
    if direct.is_file() {
        return Ok(direct);
    }
    let direct = PathBuf::from(scenario);
    if direct.is_file() {
        return Ok(direct);
    }
    let name = scenario.trim_end_matches(".ron");
    let hand = game_dir
        .join("tests")
        .join("scenarios")
        .join(format!("{name}.ron"));
    let gen = game_dir
        .join("tests")
        .join("generated")
        .join(format!("{name}.ron"));
    match (hand.is_file(), gen.is_file()) {
        (true, true) => Err(format!(
            "scénario ambigu : « {name} » existe en écrit à la main ({}) et en généré ({}) : donner le chemin",
            hand.display(),
            gen.display()
        )),
        (true, false) => Ok(hand),
        (false, true) => Ok(gen),
        (false, false) => Err(format!(
            "scénario inconnu : « {scenario} » (ni {}, ni {})",
            hand.display(),
            gen.display()
        )),
    }
}

/// `alacod video [jeu] <scénario>` (X-2b) : capture par le binaire du jeu (avec son plugin et le
/// rendu : `scenarios capture`), puis encodage ffmpeg, le tout écrit côté jeu (défaut :
/// `<jeu>/target/videos/<scénario>.mp4` + `.events.json`).
fn run_video(
    game_dir: &std::path::Path,
    scenario: &str,
    out: Option<PathBuf>,
    every: u32,
    follow: Option<usize>,
    size: Option<&str>,
) -> ExitCode {
    let game = match game_dir.canonicalize() {
        Ok(g) => g,
        Err(e) => return fail(&e.to_string()),
    };
    let ron = match resolve_scenario_file(&game, scenario) {
        Ok(p) => p,
        Err(e) => return fail(&e),
    };
    let stem = ron
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("scenario")
        .to_string();
    if every == 0 {
        return fail("alacod video : --every : au moins 1");
    }
    if let Some(size) = size {
        let ok = size.split_once('x').is_some_and(|(w, h)| {
            w.parse::<u32>().is_ok_and(|w| w > 0) && h.parse::<u32>().is_ok_and(|h| h > 0)
        });
        if !ok {
            return fail(&format!(
                "alacod video : --size : <largeur>x<hauteur> attendu, pas « {size} »"
            ));
        }
    }
    let version = app_version(&game, false);
    let frames = std::env::temp_dir().join(format!("alacod-video-{}-{stem}", std::process::id()));
    let _ = std::fs::remove_dir_all(&frames);
    let mut args = vec![
        "scenarios".to_string(),
        "capture".to_string(),
        ron.to_string_lossy().into_owned(),
        "--dir".to_string(),
        frames.to_string_lossy().into_owned(),
        "--every".to_string(),
        every.to_string(),
    ];
    if let Some(f) = follow {
        args.push("--follow".to_string());
        args.push(f.to_string());
    }
    if let Some(s) = size {
        args.push("--size".to_string());
        args.push(s.to_string());
    }
    // Avec le rendu : features par défaut du jeu (le binaire `scenarios capture` dessine).
    let capture = match game_tool(
        &game,
        &version,
        &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        false,
    ) {
        Ok(out) => out,
        Err(e) => return fail(&e),
    };
    if !capture.ok {
        show_tool_stderr(&capture);
        let log = capture.stderr.to_lowercase();
        if ["vulkan", "wgpu", "adapter", "no suitable", "metal", "glx"]
            .iter()
            .any(|h| log.contains(h))
        {
            eprintln!(
                "cause probable : GPU absent ou inutilisable ; rendre sur un poste avec GPU (Mac : Metal)."
            );
        }
        eprintln!("images partielles conservées dans {}", frames.display());
        return fail("alacod video : la capture a échoué (voir ci-dessus)");
    }
    let out_dir = match out {
        Some(d) => d,
        None => game.join("target").join("videos"),
    };
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        return fail(&format!("{} : {e}", out_dir.display()));
    }
    let mp4 = out_dir.join(format!("{stem}.mp4"));
    if let Err(e) = encode_video(&frames, &mp4, &format!("{stem} {version}"), every) {
        eprintln!("images conservées dans {}", frames.display());
        return fail(&e);
    }
    let events = out_dir.join(format!("{stem}.events.json"));
    if std::fs::copy(frames.join("events.json"), &events).is_ok() {
        println!("moments clés : {}", events.display());
    }
    let _ = std::fs::remove_dir_all(&frames);
    println!("vidéo : {}", mp4.display());
    ExitCode::SUCCESS
}

/// Assemble les `frame_*.png` en mp4 (`scripts/scenario-video`, mêmes réglages : 60/`every`
/// images/s, x264) ; étiquette `drawtext` si ffmpeg la connaît et qu'une police monospace existe.
fn encode_video(
    frames: &std::path::Path,
    mp4: &std::path::Path,
    label: &str,
    every: u32,
) -> Result<(), String> {
    let ffmpeg = |args: &[&str]| {
        std::process::Command::new("ffmpeg")
            .args(args)
            .output()
            .map_err(|_| "ffmpeg introuvable (https://ffmpeg.org) : installer ffmpeg puis relancer `alacod video`".to_string())
    };
    let version = ffmpeg(&["-hide_banner", "-version"])?;
    if !version.status.success() {
        return Err("ffmpeg ne répond pas".to_string());
    }
    let filters = ffmpeg(&["-hide_banner", "-filters"])
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-framerate"])
        .arg((60 / every.max(1)).to_string())
        .args(["-pattern_type", "glob", "-i"])
        .arg(frames.join("frame_*.png").to_string_lossy().into_owned());
    if filters.contains(" drawtext ") {
        if let Some(font) = monospace_font() {
            let label = label.replace([':', '\''], " ");
            cmd.arg("-vf").arg(format!(
                "drawtext=fontfile={font}:fontsize=20:fontcolor=white:box=1:boxcolor=black@0.6:boxborderw=6:x=10:y=10:text='{label}  frame %{{eif\\:n*{every}\\:d}}'"
            ));
        } else {
            eprintln!("avertissement : aucune police monospace (fc-match), vidéo sans étiquette");
        }
    } else {
        eprintln!("avertissement : ffmpeg sans drawtext, vidéo sans étiquette");
    }
    cmd.args([
        "-pix_fmt", "yuv420p", "-c:v", "libx264", "-preset", "medium", "-crf", "20",
    ])
    .arg(mp4);
    let out = cmd
        .output()
        .map_err(|_| "ffmpeg ne se lance pas".to_string())?;
    if !out.status.success() {
        return Err(format!(
            "ffmpeg a échoué : {}",
            String::from_utf8_lossy(&out.stderr)
                .lines()
                .last()
                .unwrap_or("voir la sortie")
        ));
    }
    Ok(())
}

/// Police monospace pour l'étiquette ffmpeg (`scripts/scenario-video` : `fc-match`) ; `None` sans
/// fontconfig (Mac sans `fc-match`) : la vidéo est alors encodée sans étiquette.
fn monospace_font() -> Option<String> {
    std::process::Command::new("fc-match")
        .args(["-f", "%{file}", "monospace"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// `alacod build` : `cargo build` dans le dossier du jeu avec sa version (`APP_VERSION`, lue par
/// `game::build.rs`), natif ou `--web`. Artefacts seulement dans le jeu (`target/…`).
fn run_build(game_dir: &std::path::Path, release: bool, web: bool) -> ExitCode {
    let game = match game_dir.canonicalize() {
        Ok(g) => g,
        Err(e) => return fail(&e.to_string()),
    };
    let (name, version) = match package_id(&game) {
        Ok(id) => id,
        Err(e) => return fail(&e),
    };
    let build_id = app_version(&game, true);
    if web {
        return build_web(&game, &name, &build_id, release, &version);
    }
    let _ = version;
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(&game)
        .env("APP_VERSION", &build_id)
        .arg("build");
    if release {
        cmd.arg("--release");
    }
    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(_) => return fail("alacod build : cargo build a échoué (voir ci-dessus)"),
        Err(e) => return fail(&format!("cargo : {e}")),
    }
    let bin = target_dir_for(&game)
        .join(if release { "release" } else { "debug" })
        .join(bin_name(&name));
    println!("construit : {} (APP_VERSION={build_id})", bin.display());
    ExitCode::SUCCESS
}

/// Nom du binaire d'un paquet (`name`, `.exe` sur Windows).
fn bin_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// Nom et version du paquet du jeu (son `Cargo.toml`), pour nommer le binaire et `APP_VERSION`.
fn package_id(game_dir: &std::path::Path) -> Result<(String, String), String> {
    let manifest = game_dir.join("Cargo.toml");
    let text =
        std::fs::read_to_string(&manifest).map_err(|e| format!("{} : {e}", manifest.display()))?;
    let doc: toml_edit::DocumentMut = text
        .parse()
        .map_err(|e| format!("{} : TOML invalide : {e}", manifest.display()))?;
    let name = doc
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{} : [package] name manque", manifest.display()))?;
    let version = doc
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{} : [package] version manque", manifest.display()))?;
    Ok((name.to_string(), version.to_string()))
}

/// Version compilée dans le jeu (`APP_VERSION`, lue par `game::build.rs` et servie par `entry::run`
/// sur le web sous `/builds/<version>/<jeu>/assets`) : `APP_VERSION` (ou `WEB_BUILD_ID`) si posée,
/// sinon `v<version du Cargo.toml du jeu>`, stable d'un build à l'autre. `unique` (`alacod build`)
/// ajoute la révision git courte du jeu (`-local` hors git) : chaque version est un dossier immuable
/// (`<jeu>/target/web/<version>/`), comme `scripts/build-web-games.mjs`.
fn app_version(game_dir: &std::path::Path, unique: bool) -> String {
    for key in ["APP_VERSION", "WEB_BUILD_ID"] {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                return v.trim().to_string();
            }
        }
    }
    let base = package_id(game_dir)
        .map(|(_, v)| format!("v{v}"))
        .unwrap_or_else(|_| "v0.0.0".into());
    if !unique {
        return base;
    }
    match git_short_sha(game_dir) {
        Some(sha) => format!("{base}-{sha}"),
        None => format!("{base}-local"),
    }
}

/// Révision git courte du dossier (git local, hors ligne) ; `None` hors dépôt ou sans git.
fn git_short_sha(dir: &std::path::Path) -> Option<String> {
    std::process::Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Dossier `target` d'un build lancé dans le jeu : `CARGO_TARGET_DIR` (absolu, ou relatif au dossier
/// courant comme cargo) sinon `<jeu>/target`.
fn target_dir_for(game_dir: &std::path::Path) -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) => {
            let dir = PathBuf::from(dir);
            if dir.is_absolute() {
                dir
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(dir)
            }
        }
        None => game_dir.join("target"),
    }
}

/// `alacod build --web` : wasm du jeu (`wasm32-unknown-unknown`, rendu seul, `APP_VERSION` posée),
/// `wasm-bindgen` (même version que le `Cargo.lock` du jeu), assets copiés et `build.json`, dans
/// `<jeu>/target/web/<version>/<jeu>/` (immuable : refuse si le dossier existe déjà).
fn build_web(
    game_dir: &std::path::Path,
    name: &str,
    build_id: &str,
    release: bool,
    _cargo_version: &str,
) -> ExitCode {
    if build_id.is_empty()
        || !build_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return fail(&format!(
            "alacod build --web : version « {build_id} » invalide ([A-Za-z0-9._-], APP_VERSION/WEB_BUILD_ID pour la forcer)"
        ));
    }
    let web_dir = target_dir_for(game_dir).join("web");
    let out = web_dir.join(build_id).join(name);
    if out.exists() {
        return fail(&format!(
            "alacod build --web : {} existe déjà (build immuable : changer de version)",
            out.display()
        ));
    }
    let staging = web_dir.join(format!(".partial-{build_id}-{name}-{}", std::process::id()));
    let profile = if release { "release" } else { "dev" };
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(game_dir)
        .env("APP_VERSION", build_id)
        .env("RUSTFLAGS", "--cfg=web_sys_unstable_apis")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--no-default-features",
            "--features",
            "render",
        ]);
    if release {
        cmd.arg("--release");
    }
    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(_) => {
            return fail(
                "alacod build --web : cargo build a échoué (cible installée ? `rustup target add wasm32-unknown-unknown`)",
            );
        }
        Err(e) => return fail(&format!("cargo : {e}")),
    }
    let wasm = target_dir_for(game_dir)
        .join("wasm32-unknown-unknown")
        .join(profile)
        .join(format!("{name}.wasm"));
    if !wasm.is_file() {
        return fail(&format!(
            "alacod build --web : {} absent après le build",
            wasm.display()
        ));
    }
    let bindgen_bin = std::env::var("WASM_BINDGEN").unwrap_or_else(|_| "wasm-bindgen".to_string());
    let found = std::process::Command::new(&bindgen_bin)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    match (
        found.strip_prefix("wasm-bindgen "),
        wasm_bindgen_crate_version(game_dir),
    ) {
        (Some(have), Some(want)) if have == want => {}
        (Some(have), Some(want)) => {
            return fail(&format!(
                "alacod build --web : {bindgen_bin} {have}, le jeu demande {want} (`cargo install wasm-bindgen-cli --version {want}`)"
            ));
        }
        _ => {
            return fail(
                "alacod build --web : wasm-bindgen introuvable (même version que le Cargo.lock du jeu)",
            );
        }
    }
    if let Err(e) = std::fs::create_dir_all(&staging) {
        return fail(&format!("{} : {e}", staging.display()));
    }
    let built = (|| -> Result<(), String> {
        let status = std::process::Command::new(&bindgen_bin)
            .arg("--out-dir")
            .arg(&staging)
            .args(["--out-name", "wasm", "--target", "web"])
            .arg(&wasm)
            .status()
            .map_err(|e| format!("{bindgen_bin} : {e}"))?;
        if !status.success() {
            return Err(format!("{bindgen_bin} a échoué"));
        }
        if std::env::var("WEB_OPTIMIZE").as_deref() != Ok("0") {
            let opt = std::env::var("WASM_OPT").unwrap_or_else(|_| "wasm-opt".to_string());
            match std::process::Command::new(&opt)
                .arg(staging.join("wasm_bg.wasm"))
                .args(["--strip-debug", "--strip-dwarf", "-O2", "-o"])
                .arg(staging.join("wasm_bg.optimized.wasm"))
                .status()
            {
                Ok(s) if s.success() => {
                    std::fs::rename(
                        staging.join("wasm_bg.optimized.wasm"),
                        staging.join("wasm_bg.wasm"),
                    )
                    .map_err(|e| e.to_string())?;
                }
                _ => eprintln!("avertissement : wasm-opt absent ou en échec, wasm non optimisé"),
            }
        }
        copy_dir(&game_dir.join("assets"), &staging.join("assets"))?;
        let generated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let manifest = serde_json::json!({
            "schemaVersion": 1,
            "gameId": name,
            "buildId": build_id,
            "module": format!("/builds/{build_id}/{name}/wasm.js"),
            "wasm": format!("/builds/{build_id}/{name}/wasm_bg.wasm"),
            "assets": format!("/builds/{build_id}/{name}/assets"),
            "generatedAt": generated_at,
        });
        std::fs::write(
            staging.join("build.json"),
            serde_json::to_string_pretty(&manifest).unwrap() + "\n",
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(e) = built {
        let _ = std::fs::remove_dir_all(&staging);
        return fail(&format!("alacod build --web : {e}"));
    }
    if let Some(parent) = out.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            let _ = std::fs::remove_dir_all(&staging);
            return fail(&format!("{} : {e}", parent.display()));
        }
    }
    if std::fs::rename(&staging, &out).is_err() {
        let _ = std::fs::remove_dir_all(&staging);
        return fail(&format!(
            "alacod build --web : {} existe déjà (build immuable)",
            out.display()
        ));
    }
    println!("web : {} (APP_VERSION={build_id})", out.display());
    ExitCode::SUCCESS
}

/// Version de la crate `wasm-bindgen` dans le `Cargo.lock` du jeu (le binaire `wasm-bindgen` doit
/// être la même, voir `scripts/build-web-games.mjs`).
fn wasm_bindgen_crate_version(game_dir: &std::path::Path) -> Option<String> {
    let lock = std::fs::read_to_string(game_dir.join("Cargo.lock")).ok()?;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"wasm-bindgen\"" {
            for next in lines.by_ref() {
                let next = next.trim();
                if let Some(v) = next
                    .strip_prefix("version = \"")
                    .and_then(|v| v.strip_suffix('"'))
                {
                    return Some(v.to_string());
                }
                if next.is_empty() || next == "[[package]]" {
                    break;
                }
            }
        }
    }
    None
}

/// Copie récursive (assets du jeu vers un build web).
fn copy_dir(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{} : {e}", to.display()))?;
    for entry in std::fs::read_dir(from).map_err(|e| format!("{} : {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)
                .map_err(|e| format!("{} : {e}", target.display()))?;
        }
    }
    Ok(())
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
