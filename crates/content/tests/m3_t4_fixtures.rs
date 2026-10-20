//! M3-T4 (`docs/conventions.md` §69) : `Explode` est exécutée, le lint ne la refuse plus dans un effet.

use content::{load_and_lint, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

#[test]
fn explode_in_an_effect_is_no_longer_unsupported() {
    let (_, _, errors) = load_and_lint(&fixture_dir("explode_effect_ok")).unwrap();
    assert!(
        !errors
            .iter()
            .any(|e| e.kind == LintErrorKind::Unsupported && e.message.contains("Explode")),
        "{errors:#?}"
    );
    // Seule l'erreur d'entrée (carte absente de la fixture) est attendue.
    let others: Vec<_> = errors
        .iter()
        .filter(|e| !e.message.contains("entry.start_map"))
        .collect();
    assert!(others.is_empty(), "{others:#?}");
}

/// D80 : `Explode` est exécutée dans l'`OnUse` d'un objet (même exécution que les effets de personnage),
/// le lint ne la refuse plus ; ses bornes (rayon > 0) restent vérifiées, comme dans un effet.
#[test]
fn explode_in_onuse_of_an_item_is_accepted() {
    let (_, _, errors) = load_and_lint(&fixture_dir("explode_onuse_ok")).unwrap();
    let others: Vec<_> = errors
        .iter()
        .filter(|e| !e.message.contains("entry.start_map"))
        .collect();
    assert!(others.is_empty(), "{others:#?}");
}

#[test]
fn explode_in_onuse_still_checks_its_radius() {
    let (_, _, errors) = load_and_lint(&fixture_dir("explode_onuse_radius_zero")).unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.kind == LintErrorKind::OutOfRange
                && e.message.contains("Explode : rayon = 0")),
        "{errors:#?}"
    );
    assert!(
        !errors
            .iter()
            .any(|e| e.message.contains("pas exécutée par OnUse")),
        "{errors:#?}"
    );
}
