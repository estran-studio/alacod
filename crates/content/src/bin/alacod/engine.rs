//! Dépendance d'un jeu envers l'engine (`docs/conventions.md` §92, `docs/taches/plan-1837-cli.md` D-b) :
//! lecture et réécriture des dépendances `git`/`path` du `Cargo.toml` du jeu, surcharge locale hors du
//! `Cargo.toml` versionné (`.cargo/config.toml`, `[patch."<url>"]`), et **racine de l'engine résolue par
//! `cargo metadata`** (la source réelle de la dépendance : checkout git, surcharge locale ou chemin).

use std::path::{Path, PathBuf};
use std::process::Command;

use toml_edit::{value, DocumentMut, Item, TableLike, Value};

const BLOCK_BEGIN: &str =
    "# >>> alacod engine local (écrit par `alacod engine local`, ne pas versionner) >>>";
const BLOCK_END: &str = "# <<< alacod engine local <<<";

/// Révision git d'une dépendance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRef {
    Tag(String),
    Rev(String),
    Branch(String),
}

impl GitRef {
    pub fn key(&self) -> &'static str {
        match self {
            Self::Tag(_) => "tag",
            Self::Rev(_) => "rev",
            Self::Branch(_) => "branch",
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Tag(v) | Self::Rev(v) | Self::Branch(v) => v,
        }
    }
}

/// D'où vient l'engine d'un jeu, d'après son `Cargo.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Git {
        url: String,
        reference: GitRef,
    },
    /// Dépendance par chemin : la racine de l'engine (`crates/content` est au-dessous).
    Path(String),
}

impl Source {
    /// Spécification de dépendance d'une crate de l'engine (`{{engine:<crate>}}` du gabarit).
    pub fn dependency(&self, krate: &str) -> String {
        match self {
            Self::Path(root) => format!("path = \"{root}/crates/{krate}\""),
            Self::Git { url, reference } => {
                format!(
                    "git = \"{url}\", {} = \"{}\"",
                    reference.key(),
                    reference.value()
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Lecture et réécriture des dépendances de `Cargo.toml` (`toml_edit`)
// ---------------------------------------------------------------------------------------
//
// Toute forme valide de dépendance est comprise : table en ligne (une ligne ou répartie sur plusieurs),
// table `[dependencies.content]`, sections `dependencies`, `dev-dependencies`, `build-dependencies` et leurs
// variantes `[target.'cfg(…)'.…]`. Les commentaires et la mise en page du reste du fichier sont conservés.

const SOURCE_KEYS: [&str; 5] = ["git", "tag", "rev", "branch", "path"];

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn parse_manifest(text: &str, label: &Path) -> Result<DocumentMut, String> {
    text.parse::<DocumentMut>()
        .map_err(|e| format!("{} : TOML invalide : {e}", label.display()))
}

/// Appelle `f(nom, dépendance)` pour chaque dépendance déclarée, dans toutes les sections de dépendances.
fn visit_dependencies(doc: &mut DocumentMut, f: &mut dyn FnMut(&str, &mut dyn TableLike)) {
    fn sections(table: &mut dyn TableLike, f: &mut dyn FnMut(&str, &mut dyn TableLike)) {
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(deps) = table.get_mut(section).and_then(Item::as_table_like_mut) {
                for (name, item) in deps.iter_mut() {
                    if let Some(dep) = item.as_table_like_mut() {
                        f(name.get(), dep);
                    }
                }
            }
        }
    }
    sections(doc.as_table_mut(), f);
    if let Some(targets) = doc.get_mut("target").and_then(Item::as_table_like_mut) {
        for (_, target) in targets.iter_mut() {
            if let Some(target) = target.as_table_like_mut() {
                sections(target, f);
            }
        }
    }
}

fn source_of(table: &dyn TableLike) -> Option<Source> {
    let get = |key: &str| table.get(key).and_then(Item::as_str).map(str::to_string);
    if let Some(url) = get("git") {
        let reference = if let Some(tag) = get("tag") {
            GitRef::Tag(tag)
        } else if let Some(rev) = get("rev") {
            GitRef::Rev(rev)
        } else if let Some(branch) = get("branch") {
            GitRef::Branch(branch)
        } else {
            return None;
        };
        return Some(Source::Git { url, reference });
    }
    // `path = "<racine>/crates/<crate>"` : la racine est deux niveaux plus haut.
    let path = PathBuf::from(get("path")?);
    let root = path.parent()?.parent()?;
    Some(Source::Path(root.to_string_lossy().into_owned()))
}

/// Source de l'engine du jeu : la dépendance `content` de son `Cargo.toml`.
pub fn read_source(game_dir: &Path) -> Result<Source, String> {
    let manifest = game_dir.join("Cargo.toml");
    let cargo = std::fs::read_to_string(&manifest)
        .map_err(|e| format!("{} illisible : {e}", manifest.display()))?;
    source_in_manifest(&cargo, &manifest)
}

fn source_in_manifest(cargo: &str, label: &Path) -> Result<Source, String> {
    let mut doc = parse_manifest(cargo, label)?;
    let mut found: Option<Option<Source>> = None;
    visit_dependencies(&mut doc, &mut |name, dep| {
        if found.is_none() && name == "content" {
            found = Some(source_of(dep));
        }
    });
    match found {
        Some(Some(source)) => Ok(source),
        Some(None) => Err(format!(
            "{} : la dépendance `content` n'a ni `git` + `tag`/`rev`/`branch`, ni `path`",
            label.display()
        )),
        None => Err(format!(
            "{} : aucune dépendance `content` (ce dossier n'est pas un jeu de l'engine)",
            label.display()
        )),
    }
}

/// Remplace la valeur de `key` en gardant sa mise en forme (commentaire de fin, espaces).
fn set_in_place(dep: &mut dyn TableLike, key: &str, new: &str) {
    if let Some(old) = dep.get_mut(key).and_then(Item::as_value_mut) {
        let decor = old.decor().clone();
        *old = Value::from(new);
        *old.decor_mut() = decor;
    }
}

/// Réécrit une dépendance de l'engine pour qu'elle pointe sur `to` : la source d'abord, puis les autres clés
/// (`features`, `default-features`, `package`…) inchangées et dans leur ordre.
fn point_dependency_at(dep: &mut dyn TableLike, name: &str, to: &Source) {
    // Git → git avec les mêmes clés (`tag` → `tag`, `rev` → `rev`) : les valeurs changent sur place, les
    // commentaires et l'ordre des clés restent.
    if let Source::Git { url, reference } = to {
        if dep.contains_key("git") && dep.contains_key(reference.key()) {
            set_in_place(dep, "git", url);
            set_in_place(dep, reference.key(), reference.value());
            for key in SOURCE_KEYS {
                if key != "git" && key != reference.key() {
                    dep.remove(key);
                }
            }
            return;
        }
    }
    // Le chemin de la crate (`…/crates/<crate>`) vient de la dépendance elle-même, pas de son nom : une
    // dépendance renommée (`package = "x"`) garde sa crate.
    let krate = dep
        .get("path")
        .and_then(Item::as_str)
        .and_then(|p| {
            Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| name.to_string());
    let kept: Vec<(String, Item)> = dep
        .iter()
        .filter(|(key, _)| !SOURCE_KEYS.contains(key))
        .map(|(key, item)| (key.to_string(), item.clone()))
        .collect();
    for key in SOURCE_KEYS {
        dep.remove(key);
    }
    for (key, _) in &kept {
        dep.remove(key);
    }
    match to {
        Source::Git { url, reference } => {
            dep.insert("git", value(url.as_str()));
            dep.insert(reference.key(), value(reference.value()));
        }
        Source::Path(root) => {
            dep.insert("path", value(format!("{root}/crates/{krate}")));
        }
    }
    for (key, item) in kept {
        dep.insert(&key, item);
    }
}

/// Réécrit les dépendances du `Cargo.toml` qui visent l'engine (même dépôt git, ou chemin dans la racine
/// courante) pour qu'elles pointent sur `to`. Rend le nouveau texte et le nombre de dépendances réécrites.
fn rewrite_dependencies(
    cargo: &str,
    label: &Path,
    from: &Source,
    to: &Source,
) -> Result<(String, usize), String> {
    let mut doc = parse_manifest(cargo, label)?;
    let mut count = 0;
    visit_dependencies(&mut doc, &mut |name, dep| {
        let Some(current) = source_of(dep) else {
            return;
        };
        let same_engine = match (&current, from) {
            (Source::Git { url: a, .. }, Source::Git { url: b, .. }) => a == b,
            (Source::Path(a), Source::Path(b)) => a == b,
            _ => false,
        };
        if same_engine {
            point_dependency_at(dep, name, to);
            count += 1;
        }
    });
    Ok((doc.to_string(), count))
}

// ---------------------------------------------------------------------------------------
// Racine de l'engine (cargo metadata)
// ---------------------------------------------------------------------------------------

fn cargo_metadata(manifest: &Path, extra: &[&str]) -> Result<serde_json::Value, String> {
    // Cargo cherche ses fichiers de configuration depuis le dossier courant, pas depuis
    // nécessairement celui de `--manifest-path`. Se placer dans le jeu est indispensable pour
    // que `alacod engine local` (patch `.cargo/config.toml`) soit pris en compte quand le jeu
    // est passé en argument depuis un autre dossier.
    let game_dir = manifest.parent().unwrap_or_else(|| Path::new("."));
    let run = |offline: bool| {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(game_dir)
            .args(["metadata", "--format-version", "1", "--manifest-path"])
            .arg(manifest)
            .args(extra);
        if offline {
            cmd.arg("--offline");
        }
        cmd.output()
    };
    // Hors ligne d'abord (sources déjà téléchargées : pas de réseau, plus rapide), puis en ligne
    // (premier appel d'un jeu dont la dépendance git n'est pas encore dans le cache de cargo).
    let mut output = run(true).map_err(|e| format!("cargo : {e}"))?;
    if !output.status.success() {
        output = run(false).map_err(|e| format!("cargo : {e}"))?;
    }
    if !output.status.success() {
        return Err(format!(
            "cargo metadata a échoué pour {} : {}",
            manifest.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("cargo metadata : JSON illisible : {e}"))
}

/// Racine de l'engine que **ce jeu compile réellement** : le dossier qui contient la crate `content`
/// résolue par `cargo metadata` (checkout git du tag épinglé, surcharge locale `engine local`, ou chemin).
pub fn resolve_root(game_dir: &Path) -> Result<PathBuf, String> {
    let manifest = game_dir.join("Cargo.toml").canonicalize().map_err(|e| {
        format!(
            "{} introuvable : {e}",
            game_dir.join("Cargo.toml").display()
        )
    })?;
    let metadata = cargo_metadata(&manifest, &[])?;
    root_from_metadata(&metadata).ok_or_else(|| {
        format!(
            "{} : la crate `content` de l'engine n'est pas dans les dépendances (`alacod engine status`)",
            manifest.display()
        )
    })
}

/// `crates/content/Cargo.toml` d'un dépôt de l'engine (qui a aussi `crates/scenario`) → la racine.
pub fn root_from_metadata(metadata: &serde_json::Value) -> Option<PathBuf> {
    metadata["packages"].as_array()?.iter().find_map(|package| {
        if package["name"] != "content" {
            return None;
        }
        let manifest = PathBuf::from(package["manifest_path"].as_str()?);
        let root = manifest.parent()?.parent()?.parent()?.to_path_buf();
        (manifest.ends_with("crates/content/Cargo.toml")
            && root.join("crates/scenario/Cargo.toml").is_file())
        .then_some(root)
    })
}

/// Une racine est-elle dans le cache git de cargo ? On n'y compile pas : les cibles iraient dans le cache.
pub fn is_cargo_checkout(root: &Path) -> bool {
    root.components().any(|c| c.as_os_str() == "checkouts")
        && root.components().any(|c| c.as_os_str() == "git")
}

// ---------------------------------------------------------------------------------------
// Dépôt git de l'engine : url, tags
// ---------------------------------------------------------------------------------------

pub fn git_output(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
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

/// Dernier tag de l'engine par ordre de version (`v0.10.0` après `v0.9.0`).
pub fn latest_tag(engine: &Path) -> Result<String, String> {
    git_output(engine, &["tag", "--list", "--sort=-v:refname"])?
        .lines()
        .next()
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "aucun tag dans {} : `--tag <tag>`, `--rev <révision>` ou `--path` (dépendance par chemin)",
                engine.display()
            )
        })
}

/// `tag` ou `rev` ? Un tag existe sous `refs/tags/` du dépôt `url` (`git ls-remote`, qui marche aussi sur un
/// chemin local) ; sinon une révision hexadécimale (7 à 40 chiffres).
pub fn classify_ref(url: &str, value: &str) -> Result<GitRef, String> {
    let status = Command::new("git")
        .args(["ls-remote", "--exit-code", "--tags", url])
        .arg(format!("refs/tags/{value}"))
        .output()
        .map_err(|e| format!("git : {e}"))?;
    if status.status.success() {
        return Ok(GitRef::Tag(value.to_string()));
    }
    if (7..=40).contains(&value.len()) && value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(GitRef::Rev(value.to_string()));
    }
    Err(format!(
        "« {value} » n'est ni un tag de {url} ni une révision git (7 à 40 chiffres hexadécimaux){}",
        match status.status.code() {
            Some(2) => String::new(),
            _ => format!(" : {}", String::from_utf8_lossy(&status.stderr).trim()),
        }
    ))
}

// ---------------------------------------------------------------------------------------
// Surcharge locale : `.cargo/config.toml`
// ---------------------------------------------------------------------------------------

fn config_path(game_dir: &Path) -> PathBuf {
    game_dir.join(".cargo").join("config.toml")
}

/// Le chemin de la surcharge locale posée par `alacod engine local`, s'il y en a une.
pub fn local_override(game_dir: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(config_path(game_dir)).ok()?;
    let block = text.split(BLOCK_BEGIN).nth(1)?.split(BLOCK_END).next()?;
    let doc: DocumentMut = block.parse().ok()?;
    let path = doc
        .get("patch")?
        .as_table_like()?
        .iter()
        .find_map(|(_, patched)| {
            patched
                .as_table_like()?
                .get("content")?
                .as_table_like()?
                .get("path")?
                .as_str()
                .map(PathBuf::from)
        })?;
    path.parent()?.parent().map(Path::to_path_buf)
}

/// Noms des crates de `<root>/crates/*` (lus dans leur `[package]`, sans cargo).
fn engine_crates(root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir(root.join("crates"))
        .map_err(|e| format!("{}/crates illisible : {e}", root.display()))?;
    for entry in entries.flatten() {
        let dir = entry.path();
        let Ok(text) = std::fs::read_to_string(dir.join("Cargo.toml")) else {
            continue;
        };
        let mut in_package = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_package = line == "[package]";
            } else if in_package {
                if let Some(value) = line.strip_prefix("name") {
                    if let Some((_, name)) = value.split_once('=') {
                        found.push((unquote(name), dir.clone()));
                        break;
                    }
                }
            }
        }
    }
    found.sort();
    Ok(found)
}

fn strip_block(text: &str) -> String {
    let Some((before, rest)) = text.split_once(BLOCK_BEGIN) else {
        return text.to_string();
    };
    let after = rest.split_once(BLOCK_END).map_or("", |(_, after)| after);
    let joined = format!("{}{}", before.trim_end(), after);
    let joined = joined.trim().to_string();
    if joined.is_empty() {
        joined
    } else {
        joined + "\n"
    }
}

fn remove_local_override(game_dir: &Path) -> Result<bool, String> {
    let path = config_path(game_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(false);
    };
    if !text.contains(BLOCK_BEGIN) {
        return Ok(false);
    }
    let rest = strip_block(&text);
    if rest.is_empty() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        // Le dossier `.cargo` créé par `engine local` part avec lui s'il est vide.
        let _ = std::fs::remove_dir(game_dir.join(".cargo"));
    } else {
        std::fs::write(&path, rest).map_err(|e| e.to_string())?;
    }
    Ok(true)
}

/// Ajoute `.cargo/config.toml` au `.gitignore` du jeu (la surcharge est locale à la machine).
fn ignore_config(game_dir: &Path) -> Result<(), String> {
    let path = game_dir.join(".gitignore");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    if text.lines().any(|l| l.trim() == "/.cargo/config.toml") {
        return Ok(());
    }
    let mut text = text;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("/.cargo/config.toml\n");
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------------------
// Commandes `alacod engine …`
// ---------------------------------------------------------------------------------------

/// `alacod engine local <chemin>` : le jeu compile l'engine de `<chemin>` sans toucher à son
/// `Cargo.toml` versionné (`[patch."<url>"]` dans `.cargo/config.toml`, ignoré par git).
pub fn local(game_dir: &Path, path: &Path) -> Result<String, String> {
    let root = path
        .canonicalize()
        .map_err(|e| format!("{} introuvable : {e}", path.display()))?;
    if !root.join("crates/content/Cargo.toml").is_file()
        || !root.join("templates/starter/assets/game.ron").is_file()
    {
        return Err(format!(
            "{} n'est pas la racine d'un engine (crates/content, templates/starter)",
            root.display()
        ));
    }
    let url = match read_source(game_dir)? {
        Source::Git { url, .. } => url,
        Source::Path(_) => {
            return Err(
                "ce jeu dépend déjà de l'engine par chemin : `alacod engine pin <tag|rev>` d'abord (la surcharge locale remplace une dépendance git)"
                    .into(),
            )
        }
    };
    let mut block = format!("{BLOCK_BEGIN}\n[patch.\"{url}\"]\n");
    let crates = engine_crates(&root)?;
    for (name, dir) in &crates {
        block.push_str(&format!("{name} = {{ path = \"{}\" }}\n", dir.display()));
    }
    block.push_str(BLOCK_END);
    block.push('\n');

    let config = config_path(game_dir);
    let existing = std::fs::read_to_string(&config).unwrap_or_default();
    let mut text = strip_block(&existing);
    if !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&block);
    std::fs::create_dir_all(game_dir.join(".cargo")).map_err(|e| e.to_string())?;
    std::fs::write(&config, text).map_err(|e| e.to_string())?;
    ignore_config(game_dir)?;
    Ok(format!(
        "{} crates de {} substituées ({})",
        crates.len(),
        root.display(),
        config.display()
    ))
}

/// `alacod engine pin <tag|rev>` : retire la surcharge locale et épingle la dépendance de l'engine sur un
/// tag ou une révision git. `url` : `origin` de l'engine ; sans lui, celui de la dépendance actuelle (ou de
/// la racine de l'engine quand la dépendance est un chemin).
pub fn pin(game_dir: &Path, reference: &str, url: Option<String>) -> Result<String, String> {
    let manifest = game_dir.join("Cargo.toml");
    let current = read_source(game_dir)?;
    let url = match (&url, &current) {
        (Some(url), _) => url.clone(),
        (None, Source::Git { url, .. }) => url.clone(),
        (None, Source::Path(root)) => {
            git_output(Path::new(root), &["remote", "get-url", "origin"])?
        }
    };
    let reference = classify_ref(&url, reference)?;
    let target = Source::Git {
        url,
        reference: reference.clone(),
    };
    let cargo = std::fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
    let (rewritten, count) = rewrite_dependencies(&cargo, &manifest, &current, &target)?;
    if count == 0 {
        return Err(format!(
            "{} : aucune dépendance de l'engine à réécrire",
            manifest.display()
        ));
    }
    std::fs::write(&manifest, rewritten).map_err(|e| e.to_string())?;
    let removed = remove_local_override(game_dir)?;
    Ok(format!(
        "{count} dépendances épinglées sur {} {}{}",
        reference.key(),
        reference.value(),
        if removed {
            " ; surcharge locale retirée"
        } else {
            ""
        }
    ))
}

/// `alacod engine status` : source déclarée, surcharge locale, racine réellement compilée.
pub fn status(game_dir: &Path) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    match read_source(game_dir)? {
        Source::Git { url, reference } => lines.push(format!(
            "dépendance : git {url} {} {}",
            reference.key(),
            reference.value()
        )),
        Source::Path(root) => lines.push(format!("dépendance : chemin {root}")),
    }
    match local_override(game_dir) {
        Some(root) => lines.push(format!(
            "surcharge locale : {} (.cargo/config.toml)",
            root.display()
        )),
        None => lines.push("surcharge locale : aucune".to_string()),
    }
    match resolve_root(game_dir) {
        Ok(root) => lines.push(format!("racine compilée : {}", root.display())),
        Err(e) => lines.push(format!("racine compilée : indéterminée ({e})")),
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARGO: &str = r#"[dependencies]
bevy = { version = "0.19.1", default-features = false, features = ["a", "b"] }
content = { git = "https://x/engine.git", tag = "v0.1.0" }
game = { git = "https://x/engine.git", tag = "v0.1.0", features = ["lighting", "debug_ui"] }
other = { git = "https://y/other.git", tag = "v9" }
"#;

    fn git(url: &str, tag: &str) -> Source {
        Source::Git {
            url: url.into(),
            reference: GitRef::Tag(tag.into()),
        }
    }

    fn rewrite(cargo: &str, from: &Source, to: &Source) -> (String, usize) {
        rewrite_dependencies(cargo, Path::new("Cargo.toml"), from, to).unwrap()
    }

    fn source(cargo: &str) -> Result<Source, String> {
        source_in_manifest(cargo, Path::new("Cargo.toml"))
    }

    #[test]
    fn pin_rewrites_only_the_engine_and_keeps_other_keys() {
        let (out, count) = rewrite(
            CARGO,
            &git("https://x/engine.git", "v0.1.0"),
            &git("https://x/engine.git", "v0.2.0"),
        );
        assert_eq!(count, 2);
        assert!(
            out.contains(r#"content = { git = "https://x/engine.git", tag = "v0.2.0" }"#),
            "{out}"
        );
        assert!(
            out.contains(r#"features = ["lighting", "debug_ui"]"#),
            "{out}"
        );
        assert!(
            out.contains(r#"other = { git = "https://y/other.git", tag = "v9" }"#),
            "{out}"
        );
        assert!(out.contains(r#"bevy = { version"#), "{out}");
    }

    #[test]
    fn a_path_dependency_becomes_git_with_the_crate_directory() {
        let cargo = "[dependencies]\ncontent = { path = \"/e/crates/content\" }\nmap_ldtk = { path = \"/e/crates/map_ldtk\", features = [\"render\"] }\n";
        let (out, count) = rewrite(
            cargo,
            &Source::Path("/e".into()),
            &Source::Git {
                url: "u".into(),
                reference: GitRef::Rev("abc1234".into()),
            },
        );
        assert_eq!(count, 2);
        assert!(
            out.contains("content = { git = \"u\", rev = \"abc1234\" }"),
            "{out}"
        );
        assert!(
            out.contains("map_ldtk = { git = \"u\", rev = \"abc1234\", features = [\"render\"] }"),
            "{out}"
        );
    }

    /// Toute forme TOML valide de la dépendance est lue et réécrite : table `[dependencies.content]`, table en
    /// ligne répartie sur plusieurs lignes (tableau `features` sur plusieurs lignes), sections `dev-dependencies`
    /// et `target`, commentaires conservés.
    #[test]
    fn every_valid_toml_form_of_the_dependency_is_understood() {
        let from = git("https://x/engine.git", "v0.1.0");
        let to = git("https://x/engine.git", "v0.2.0");
        let cases = [
            // Table dédiée.
            "[dependencies.content]\ngit = \"https://x/engine.git\"\ntag = \"v0.1.0\"\n",
            // Table dédiée, clés dans un autre ordre, commentaire.
            "[dependencies.content]\n# l'engine\ntag = \"v0.1.0\"\ngit = \"https://x/engine.git\"\n",
            // Inline avec tableau sur plusieurs lignes.
            "[dependencies]\ncontent = { git = \"https://x/engine.git\", tag = \"v0.1.0\", features = [\n  \"a\",\n  \"b\",\n] }\n",
            // Inline multiligne (TOML 1.1).
            "[dependencies]\ncontent = {\n  git = \"https://x/engine.git\",\n  tag = \"v0.1.0\",\n}\n",
            // Dépendance de développement, de compilation, ou propre à une cible.
            "[dev-dependencies]\ncontent = { git = \"https://x/engine.git\", tag = \"v0.1.0\" }\n",
            "[build-dependencies]\ncontent = { git = \"https://x/engine.git\", tag = \"v0.1.0\" }\n",
            "[target.'cfg(unix)'.dependencies]\ncontent = { git = \"https://x/engine.git\", tag = \"v0.1.0\" }\n",
        ];
        for cargo in cases {
            assert_eq!(source(cargo).unwrap(), from, "lecture : {cargo}");
            let (out, count) = rewrite(cargo, &from, &to);
            assert_eq!(count, 1, "{cargo}");
            assert_eq!(source(&out).unwrap(), to, "réécriture : {out}");
            assert!(!out.contains("v0.1.0"), "{out}");
        }
        // Rien d'autre ne bouge : commentaire et `features` multiligne conservés.
        let (out, _) = rewrite(cases[1], &from, &to);
        assert!(out.contains("# l'engine"), "{out}");
        let (out, _) = rewrite(cases[2], &from, &to);
        assert!(out.contains("features = ["), "{out}");
        assert!(out.contains("\"a\"") && out.contains("\"b\""), "{out}");
    }

    #[test]
    fn a_table_form_dependency_can_switch_from_tag_to_revision() {
        let cargo = "[dependencies.content]\ngit = \"u\"\ntag = \"v1\"\nfeatures = [\"x\"]\n";
        let (out, count) = rewrite(
            cargo,
            &git("u", "v1"),
            &Source::Git {
                url: "u".into(),
                reference: GitRef::Rev("abc1234".into()),
            },
        );
        assert_eq!(count, 1);
        assert!(
            out.contains("rev = \"abc1234\"") && !out.contains("tag"),
            "{out}"
        );
        assert!(out.contains("features = [\"x\"]"), "{out}");
    }

    #[test]
    fn a_missing_or_malformed_dependency_is_reported() {
        assert!(source("[dependencies]\nbevy = \"0.19\"\n")
            .unwrap_err()
            .contains("aucune dépendance `content`"));
        assert!(source("[dependencies]\ncontent = { version = \"1\" }\n")
            .unwrap_err()
            .contains("ni `git`"));
        assert!(source("[dependencies\ncontent = 1\n")
            .unwrap_err()
            .contains("TOML invalide"));
    }

    #[test]
    fn the_local_override_path_is_read_back_from_the_config_block() {
        let dir = std::env::temp_dir().join(format!("alacod-override-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".cargo")).unwrap();
        std::fs::write(
            dir.join(".cargo/config.toml"),
            format!("[build]\njobs = 4\n\n{BLOCK_BEGIN}\n[patch.\"u\"]\ncontent = {{ path = \"/e/crates/content\" }}\n{BLOCK_END}\n"),
        )
        .unwrap();
        assert_eq!(local_override(&dir), Some(PathBuf::from("/e")));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_engine_root_is_the_folder_above_crates_content() {
        let dir = std::env::temp_dir().join(format!("alacod-root-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("crates/scenario")).unwrap();
        std::fs::write(dir.join("crates/scenario/Cargo.toml"), "").unwrap();
        let metadata = serde_json::json!({"packages": [
            {"name": "bevy", "manifest_path": "/x/bevy/Cargo.toml"},
            {"name": "content", "manifest_path": dir.join("crates/content/Cargo.toml")},
        ]});
        assert_eq!(root_from_metadata(&metadata), Some(dir.clone()));
        assert_eq!(
            root_from_metadata(&serde_json::json!({"packages": []})),
            None
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn stripping_the_block_keeps_the_rest_of_the_config() {
        let text =
            format!("[build]\njobs = 4\n\n{BLOCK_BEGIN}\n[patch.\"u\"]\na = 1\n{BLOCK_END}\n");
        assert_eq!(strip_block(&text), "[build]\njobs = 4\n");
        assert_eq!(strip_block(&format!("{BLOCK_BEGIN}\nx\n{BLOCK_END}\n")), "");
    }
}
