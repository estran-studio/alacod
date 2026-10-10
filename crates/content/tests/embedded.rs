//! Browser content must parse and lint identically to the on-disk source. Run this on
//! the actual clones so future content kinds cannot silently omit browser loading.
use std::path::Path;

fn inventory(root: &Path, dir: &Path, result: &mut Vec<(&'static str, Option<&'static str>)>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            inventory(root, &path, result);
            continue;
        }
        let name: &'static str = Box::leak(
            path.strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/")
                .into_boxed_str(),
        );
        let text = match path.extension().and_then(|ext| ext.to_str()) {
            Some("ron" | "ldtk" | "json") => Some(Box::leak(
                std::fs::read_to_string(&path).unwrap().into_boxed_str(),
            ) as &'static str),
            _ => None,
        };
        result.push((name, text));
    }
}

#[test]
fn real_clones_have_identical_native_and_browser_registries() {
    for game in ["zombies", "throne", "gungeon"] {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../games")
            .join(game);
        let (native, native_manifest, native_errors) = content::load_and_lint(&dir).unwrap();
        assert!(native_errors.is_empty(), "{game}: {native_errors:?}");
        let root = dir.join("assets");
        let mut files = Vec::new();
        inventory(&root, &root, &mut files);
        files.sort_by_key(|(name, _)| *name);
        let (mut browser, browser_manifest, browser_errors) =
            content::load_embedded(Box::leak(files.into_boxed_slice())).unwrap();
        assert!(browser_errors.is_empty(), "{game}: {browser_errors:?}");
        assert_eq!(native_manifest, browser_manifest);
        browser.game_dir = native.game_dir.clone();
        browser.embedded_files = None;
        assert_eq!(
            format!("{native:#?}"),
            format!("{browser:#?}"),
            "all parsed fields of {game}"
        );
    }
}

#[test]
fn embedded_inventory_checks_missing_assets_and_direct_children() {
    use content::files::ContentFiles;
    let files = ContentFiles::embedded(&[
        ("weapons/b.ron", Some("b")),
        ("weapons/a.ron", Some("a")),
        ("weapons/nested/c.ron", Some("c")),
        ("image.png", None),
    ]);
    assert_eq!(
        files.discover(Path::new("weapons"), "ron").unwrap(),
        vec![Path::new("weapons/a.ron"), Path::new("weapons/b.ron")]
    );
    assert!(files.contains("image.png"));
    assert!(!files.contains("missing.png"));
    assert!(files.read(Path::new("image.png")).is_err());
    assert_eq!(files.read(Path::new("weapons/a.ron")).unwrap(), "a");
    assert!(files.discover(Path::new("absent"), "ron").is_err());
}
