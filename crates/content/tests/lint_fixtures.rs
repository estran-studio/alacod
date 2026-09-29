//! Une fixture par erreur de lint (carte T1.5, `docs/taches.md`) : `tests/fixtures/<erreur>/`
//! est un mini-jeu (`assets/game.ron` + contenu minimal) avec un seul problème volontaire.
//! Plus un test qui linte `games/zombies` et `games/testbed` sans erreur.

use content::{load_and_lint, LintError, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

/// `games/<jeu>`, relatif à `CARGO_MANIFEST_DIR` de `content` (`crates/content`).
fn game_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../games")).join(name)
}

fn assert_has_error(errors: &[LintError], kind: LintErrorKind, needle: &str) {
    let found = errors
        .iter()
        .find(|e| e.kind == kind && e.message.contains(needle));
    assert!(
        found.is_some(),
        "attendu une erreur {kind:?} contenant {needle:?}, obtenu :\n{errors:#?}"
    );
}

#[test]
fn broken_reference_fixture_reports_unknown_weapon() {
    let (_, _, errors) = load_and_lint(&fixture_dir("broken_reference")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "plasma_rifle");
}

#[test]
fn duplicate_id_fixture_reports_repeated_asset_name_ref() {
    let (_, _, errors) = load_and_lint(&fixture_dir("duplicate_id")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "hero");
}

#[test]
fn out_of_range_fixture_reports_non_positive_health() {
    let (_, _, errors) = load_and_lint(&fixture_dir("out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "base_health.max");
}

#[test]
fn unknown_kind_fixture_reports_unsupported_content_kind() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unknown_kind")).unwrap();
    assert_has_error(&errors, LintErrorKind::UnknownKind, "Npc");
}

#[test]
fn float_literal_fixture_reports_bare_number_in_fixed_field() {
    let (_, _, errors) = load_and_lint(&fixture_dir("float_literal")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "littéral");
}

#[test]
fn zombies_and_testbed_lint_without_error() {
    for game in ["zombies", "testbed"] {
        let (registry, _, errors) = load_and_lint(&game_dir(game))
            .unwrap_or_else(|e| panic!("games/{game} : manifeste invalide : {e}"));
        assert!(
            errors.is_empty(),
            "games/{game} : erreurs de lint inattendues :\n{errors:#?}"
        );
        assert!(
            !registry.characters.is_empty(),
            "games/{game} : aucun personnage chargé (game.ron incomplet ?)"
        );
    }
}
