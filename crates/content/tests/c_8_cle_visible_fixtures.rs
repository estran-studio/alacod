//! C-8 (`docs/conventions.md` §134) : le visuel d'une clé d'équipe au sol est une entrée `key/<id>` (puis `key`) de la table
//! `SpriteSheet`, avec un `display_scale` optionnel. Une clé portée (`carries_key`) sans entrée est signalée (`KeyWithoutVisual`,
//! avertissement : marqueur par défaut) ; un `display_scale` qui n'est pas > 0 est une erreur.

use content::{load_and_lint, LintErrorKind, LintWarningKind};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn key_warnings(name: &str) -> Vec<String> {
    let (registry, manifest, errors) = load_and_lint(&fixture(name)).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    content::warnings(&registry, &manifest)
        .into_iter()
        .filter(|w| w.kind == LintWarningKind::KeyWithoutVisual)
        .map(|w| w.message)
        .collect()
}

#[test]
fn a_carried_key_without_a_visual_is_warned_once() {
    let warnings = key_warnings("key_visual_missing");
    assert_eq!(warnings.len(), 1, "{warnings:#?}");
    assert!(
        warnings[0].contains("« cle_x »") && warnings[0].contains("key/cle_x"),
        "{warnings:?}"
    );
}

#[test]
fn a_specific_or_a_general_entry_is_a_visual() {
    assert!(key_warnings("key_visual_given").is_empty());
    assert!(key_warnings("key_visual_general").is_empty());
}

#[test]
fn the_display_scale_is_read_and_defaults_to_none() {
    let (given, _, errors) = load_and_lint(&fixture("key_visual_given")).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    let scale = |registry: &content::Registry, id: &str| {
        registry
            .sprite_sheets
            .get(&content::registry::SpriteSheetId::from(id))
            .map(|entry| entry.display_scale)
    };
    assert_eq!(scale(&given, "key/cle_x"), Some(Some(2.0)));
    let (general, _, _) = load_and_lint(&fixture("key_visual_general")).unwrap();
    assert_eq!(scale(&general, "key"), Some(None));
}

#[test]
fn a_display_scale_that_is_not_positive_is_an_error() {
    let (_, _, errors) = load_and_lint(&fixture("key_visual_bad_scale")).unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.kind == LintErrorKind::OutOfRange && e.message.contains("display_scale")),
        "{errors:#?}"
    );
}
