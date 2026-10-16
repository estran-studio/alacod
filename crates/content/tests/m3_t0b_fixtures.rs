//! M3-T0b (`docs/conventions.md` §61) : fixtures de lint des contrats objets et santé.

use content::{load_and_lint, LintError, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

fn assert_has_error(errors: &[LintError], kind: LintErrorKind, needle: &str) {
    assert!(
        errors
            .iter()
            .any(|e| e.kind == kind && e.message.contains(needle)),
        "attendu une erreur {kind:?} contenant {needle:?}, obtenu :\n{errors:#?}"
    );
}

#[test]
fn scaled_is_refused_in_a_powerup() {
    let (_, _, errors) = load_and_lint(&fixture_dir("scaled_in_powerup")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Scaled réservée");
}

#[test]
fn scaled_is_refused_in_a_boss_timeline() {
    let (_, _, errors) = load_and_lint(&fixture_dir("scaled_in_boss")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "Scaled : réservée");
}

#[test]
fn scaled_wraps_an_action_that_is_still_validated() {
    let (_, _, errors) = load_and_lint(&fixture_dir("scaled_inner_invalid")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Heal = -1");
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "pattern « absent » inconnu",
    );
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "arme « absente » inconnue",
    );
    // OnUse d'un consommable : même règle.
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("ration") && e.message.contains("Heal = -1")),
        "{errors:#?}"
    );
}

#[test]
fn scaled_is_accepted_in_an_item_effect() {
    let (_, _, errors) = load_and_lint(&fixture_dir("scaled_in_item_ok")).unwrap();
    // Seule l'erreur d'entrée (carte absente de la fixture) est attendue.
    let others: Vec<_> = errors
        .iter()
        .filter(|e| !e.message.contains("entry.start_map"))
        .collect();
    assert!(others.is_empty(), "{others:#?}");
}

#[test]
fn max_stack_zero_is_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_max_stack_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "max_stack = 0");
}

#[test]
fn hit_iframes_zero_is_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("character_hit_iframes_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "hit_iframes = 0");
}
