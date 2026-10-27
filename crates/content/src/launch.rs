//! `alacod sandbox <jeu>` et `alacod editor <jeu>` : construisent et lancent le binaire d'un jeu en bac à sable
//! (arène vide) ou en mode éditeur (le jeu complet) (`docs/conventions.md` §90). Remplacent les recettes
//! `make sandbox` et `make editor` (qui n'en sont plus que des alias).
//!
//! Même commande cargo que la recette d'origine : `cargo run [-p <paquet>] --features native
//! [--features game/dev-dylib] -- --sandbox [profil] [--seed N] --local-port 7000 --players localhost`,
//! `ALACOD_LABELS=1` (étiquettes F6) et, hors rendu, `ALACOD_HEADLESS=1` en profil `headless`. Mêmes
//! features et même `APP_VERSION` que `make` : un binaire construit par l'un n'est pas reconstruit par l'autre.
//!
//! - Un jeu de l'engine (`games/<jeu>`, membre du workspace) se lance depuis la racine de l'engine par
//!   `-p <paquet>` ; un jeu créé par `alacod new` est son propre workspace : `cargo run` dans son dossier.
//! - `--detach` : le jeu tourne dans son propre groupe de processus, sortie dans un journal, et la commande
//!   rend la main quand le remote répond (ou échoue si le jeu meurt avant) : c'est le mode des agents.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Ce qu'on lance : le bac à sable (`--sandbox [profil]`) ou le jeu complet en mode éditeur
/// (`ALACOD_EDITOR=1 ALACOD_REMOTE=1`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LaunchKind {
    #[default]
    Sandbox,
    Editor,
}

#[derive(Debug, Clone, Default)]
pub struct GameLaunch {
    pub kind: LaunchKind,
    pub game_dir: PathBuf,
    pub profile: Option<String>,
    pub seed: Option<i64>,
    /// Sans fenêtre ni GPU (`ALACOD_HEADLESS=1`, profil `headless`).
    pub headless: bool,
    pub detach: bool,
    /// Secondes d'attente du remote en `--detach` (compilation à froid comprise).
    pub wait_secs: u64,
    /// Arguments passés tels quels au jeu, après `--sandbox`.
    pub game_args: Vec<String>,
}

/// La commande cargo à lancer, sans l'avoir lancée (testable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub package: String,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
    /// Le jeu est un membre du workspace de l'engine (lancé par `-p` depuis sa racine).
    pub in_engine: bool,
}

/// Nom du paquet du `Cargo.toml` d'un jeu (`[package] name = "…"`).
pub fn package_name(cargo_toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in cargo_toml.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = line.strip_prefix("name") {
                let rest = rest.trim_start().strip_prefix('=')?.trim();
                return rest
                    .strip_prefix('"')
                    .and_then(|r| r.split('"').next())
                    .map(str::to_string);
            }
        }
    }
    None
}

fn is_engine_root(path: &Path) -> bool {
    path.join("crates/scenario/Cargo.toml").is_file()
}

/// Édition de liens dynamique de Bevy : la règle de `scripts/dev-features.sh` (désactivée par
/// `ALACOD_DYLIB=0`, `CI`, `ALACOD_PROFILE=prod`, `ALACOD_TARGET=web`).
fn dev_dylib_enabled() -> bool {
    let env = |key: &str| std::env::var(key).unwrap_or_default();
    env("ALACOD_DYLIB") != "0"
        && env("CI").is_empty()
        && env("ALACOD_PROFILE") != "prod"
        && env("ALACOD_TARGET") != "web"
}

/// `APP_VERSION` de `make` (`git describe`) : le même que la recette d'origine pour ne pas recompiler
/// `game` quand on passe de l'une à l'autre. `APP_VERSION` de l'environnement l'emporte.
fn app_version(engine: &Path) -> Option<String> {
    if let Ok(version) = std::env::var("APP_VERSION") {
        if !version.is_empty() {
            return Some(version);
        }
    }
    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(engine)
            .stderr(Stdio::null())
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    };
    if let Some(tag) = git(&["describe", "--tags", "--exact-match", "HEAD"]) {
        return Some(tag);
    }
    let latest = git(&["describe", "--tags", "--abbrev=0"]).unwrap_or_else(|| "v0.0.0".into());
    let sha = git(&["rev-parse", "--short", "HEAD"])?;
    Some(format!("{latest}-{sha}"))
}

pub fn plan(launch: &GameLaunch) -> Result<LaunchPlan, String> {
    let game = launch
        .game_dir
        .canonicalize()
        .map_err(|e| format!("{} : {e}", launch.game_dir.display()))?;
    let cargo_toml = std::fs::read_to_string(game.join("Cargo.toml")).map_err(|e| {
        format!(
            "{} : pas de Cargo.toml ({e}) : `alacod sandbox` et `alacod editor` lancent un jeu qui a son paquet cargo",
            game.display()
        )
    })?;
    let package = package_name(&cargo_toml).ok_or_else(|| {
        format!(
            "{} : nom de paquet introuvable",
            game.join("Cargo.toml").display()
        )
    })?;
    let standalone = cargo_toml.lines().any(|l| l.trim() == "[workspace]");
    let engine = (!standalone)
        .then(|| game.ancestors().skip(1).find(|p| is_engine_root(p)))
        .flatten()
        .map(Path::to_path_buf);

    let mut args: Vec<String> = vec!["run".into()];
    let mut envs: Vec<(String, String)> = vec![("ALACOD_LABELS".into(), "1".into())];
    if launch.kind == LaunchKind::Editor {
        envs.push(("ALACOD_EDITOR".into(), "1".into()));
        envs.push(("ALACOD_REMOTE".into(), "1".into()));
    }
    if launch.headless {
        envs.push(("ALACOD_HEADLESS".into(), "1".into()));
    }
    let cwd = match &engine {
        Some(root) => {
            if launch.headless {
                args.extend(["--profile".into(), "headless".into()]);
            }
            args.extend(["-p".into(), package.clone()]);
            if let Some(version) = app_version(root) {
                envs.push(("APP_VERSION".into(), version));
            }
            root.clone()
        }
        None => game.clone(),
    };
    if launch.headless {
        args.push("--no-default-features".into());
    }
    args.extend(["--features".into(), "native".into()]);
    if engine.is_some() && dev_dylib_enabled() {
        args.extend(["--features".into(), "game/dev-dylib".into()]);
    }
    args.push("--".into());
    if launch.kind == LaunchKind::Sandbox {
        args.push("--sandbox".into());
        if let Some(profile) = &launch.profile {
            args.push(profile.clone());
        }
    }
    if let Some(seed) = launch.seed {
        args.extend(["--seed".into(), seed.to_string()]);
    }
    args.extend(launch.game_args.iter().cloned());
    args.extend([
        "--local-port".into(),
        "7000".into(),
        "--players".into(),
        "localhost".into(),
    ]);
    Ok(LaunchPlan {
        package,
        cwd,
        args,
        envs,
        in_engine: engine.is_some(),
    })
}

/// Lance le jeu. Premier plan : code de sortie de cargo. `--detach` : 0 quand le remote répond.
pub fn run(launch: &GameLaunch) -> Result<i32, String> {
    let plan = plan(launch)?;
    let port = crate::remote::port();
    if crate::remote::call_on(port, "rpc.discover", None).is_ok() {
        return Err(format!(
            "un jeu répond déjà sur 127.0.0.1:{port} : l'arrêter avant, ou lancer celui-ci avec \
             ALACOD_REMOTE_PORT=<autre port>"
        ));
    }
    let mut cmd = Command::new("cargo");
    cmd.args(&plan.args)
        .current_dir(&plan.cwd)
        .stdin(Stdio::null());
    for (key, value) in &plan.envs {
        cmd.env(key, value);
    }
    if !launch.detach {
        let status = cmd.status().map_err(|e| format!("cargo : {e}"))?;
        return Ok(status.code().unwrap_or(1));
    }

    let kind = match launch.kind {
        LaunchKind::Sandbox => "sandbox",
        LaunchKind::Editor => "editor",
    };
    let log = std::env::temp_dir().join(format!("alacod-{kind}-{}.log", plan.package));
    let file = std::fs::File::create(&log).map_err(|e| format!("{} : {e}", log.display()))?;
    let err = file.try_clone().map_err(|e| e.to_string())?;
    cmd.stdout(Stdio::from(file)).stderr(Stdio::from(err));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Groupe à part : ni le terminal ni la fin de cette commande ne l'interrompent, et
        // `kill -- -<pid>` arrête cargo et le jeu d'un coup.
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(|e| format!("cargo : {e}"))?;
    let pid = child.id();
    eprintln!(
        "{} lancé (pid {pid}) ; journal : {} ; arrêt : kill -- -{pid}",
        plan.package,
        log.display()
    );
    let waited = crate::remote::wait_ready(port, Duration::from_secs(launch.wait_secs), || {
        matches!(child.try_wait(), Ok(None))
    });
    match waited {
        Ok(()) => {
            println!(
                "{} : remote prêt sur 127.0.0.1:{port} (pid {pid}, journal {})",
                plan.package,
                log.display()
            );
            Ok(0)
        }
        Err(e) => Err(format!("{e} (journal : {})", log.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nom_du_paquet() {
        let toml =
            "[package]\nname = \"demo\"\nversion = \"0.0.1\"\n\n[dependencies]\nname = \"x\"\n";
        assert_eq!(package_name(toml).as_deref(), Some("demo"));
        assert_eq!(package_name("[dependencies]\nname = \"x\"\n"), None);
    }
}

#[cfg(test)]
mod plan_tests {
    use super::*;

    fn engine(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel)
    }

    fn has(args: &[String], pair: &[&str]) -> bool {
        args.windows(pair.len()).any(|w| w == pair)
    }

    #[test]
    fn un_jeu_de_l_engine_se_lance_par_son_paquet_depuis_la_racine() {
        let plan = plan(&GameLaunch {
            game_dir: engine("games/zombies"),
            profile: Some("lourd".into()),
            seed: Some(7),
            headless: true,
            ..Default::default()
        })
        .unwrap();
        assert!(plan.in_engine);
        assert_eq!(plan.package, "zombies");
        assert!(is_engine_root(&plan.cwd));
        assert!(has(&plan.args, &["--profile", "headless", "-p", "zombies"]));
        assert!(has(
            &plan.args,
            &["--no-default-features", "--features", "native"]
        ));
        assert!(has(
            &plan.args,
            &["--", "--sandbox", "lourd", "--seed", "7"]
        ));
        assert!(has(
            &plan.args,
            &["--local-port", "7000", "--players", "localhost"]
        ));
        assert!(plan.envs.contains(&("ALACOD_HEADLESS".into(), "1".into())));
        assert!(plan.envs.contains(&("ALACOD_LABELS".into(), "1".into())));
        assert!(!plan.envs.iter().any(|(k, _)| k == "ALACOD_EDITOR"));
    }

    #[test]
    fn le_mode_editeur_allume_editeur_et_remote_sans_sandbox() {
        let plan = plan(&GameLaunch {
            kind: LaunchKind::Editor,
            game_dir: engine("games/zombies"),
            ..Default::default()
        })
        .unwrap();
        assert!(plan.envs.contains(&("ALACOD_EDITOR".into(), "1".into())));
        assert!(plan.envs.contains(&("ALACOD_REMOTE".into(), "1".into())));
        assert!(!plan.args.iter().any(|a| a == "--sandbox"));
        assert!(!plan.envs.iter().any(|(k, _)| k == "ALACOD_HEADLESS"));
        assert!(!plan.args.iter().any(|a| a == "--profile"));
    }

    #[test]
    fn un_jeu_cree_par_alacod_new_est_son_propre_workspace() {
        // Le gabarit a `[workspace]` : `cargo run` dans son dossier, pas de `-p`.
        let plan = plan(&GameLaunch {
            game_dir: engine("templates/starter"),
            ..Default::default()
        })
        .unwrap();
        assert!(!plan.in_engine);
        assert!(!plan.args.iter().any(|a| a == "-p"));
        assert!(!plan.args.iter().any(|a| a == "game/dev-dylib"));
        assert!(plan.cwd.ends_with("templates/starter"));
    }
}
