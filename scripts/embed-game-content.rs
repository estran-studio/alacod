// Shared build script for browser game binaries. Definitions are embedded, while binary
// assets are inventoried for lint and copied to the web build for HTTP loading.
use std::{env, fs, path::Path};

fn scan(root: &Path, dir: &Path, entries: &mut Vec<(String, Option<String>)>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    for entry in fs::read_dir(dir).expect("read assets") {
        let path = entry.expect("asset entry").path();
        if path.is_dir() {
            scan(root, &path, entries);
        } else {
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            let text = match path.extension().and_then(|ext| ext.to_str()) {
                Some("ron" | "ldtk" | "json" | "html") => {
                    Some(fs::read_to_string(&path).expect("UTF-8 content"))
                }
                _ => None,
            };
            entries.push((name, text));
        }
    }
}

/// D161 : la surcharge privée (`scripts/assets-prives.patterns`, docs/conventions.md § 99) n'est inventoriée
/// (et ses `*.local.ron` embarqués) que si `ALACOD_WEB_PRIVATE_ASSETS=1` ; sinon un poste qui l'a installée
/// compile quand même le repli public.
fn is_private(name: &str) -> bool {
    include_str!("assets-prives.patterns")
        .lines()
        .map(str::trim)
        .filter(|rule| !rule.is_empty() && !rule.starts_with('#'))
        .any(|rule| match rule.strip_prefix('*') {
            Some(suffix) => name.ends_with(suffix),
            None => name.starts_with(rule),
        })
}

fn main() {
    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }
    println!("cargo:rerun-if-env-changed=ALACOD_WEB_PRIVATE_ASSETS");
    let include_private = env::var("ALACOD_WEB_PRIVATE_ASSETS").as_deref() == Ok("1");
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
    let mut entries = Vec::new();
    scan(&root, &root, &mut entries);
    entries.retain(|(name, _)| include_private || !is_private(name));
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut source =
        String::from("pub const EMBEDDED_CONTENT: content::files::EmbeddedFiles = &[\n");
    for (name, text) in entries {
        let value = text
            .map(|text| format!("Some({text:?})"))
            .unwrap_or_else(|| "None".into());
        source.push_str(&format!("({name:?}, {value}),\n"));
    }
    source.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("embedded_content.rs"),
        source,
    )
    .unwrap();
}
