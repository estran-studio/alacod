//! M3-T0e (`docs/conventions.md` §64) : fixtures de lint des curses, du champ `curses` d'une
//! séquence `Floors` et des accomplissements.

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

fn assert_no_error(errors: &[LintError], needle: &str) {
    assert!(
        !errors.iter().any(|e| e.message.contains(needle)),
        "aucune erreur ne devrait contenir {needle:?}, obtenu :\n{errors:#?}"
    );
}

#[test]
fn curse_values_are_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("curse_invalid_values")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« sans_poids » : weight = 0",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« inerte » : effects vide",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« salles_nulles » : effects[0] (FloorRooms) : mult = 0",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« mul_nul » : effects[0] (Modifier MoveSpeed, op Mul) : value = 0",
    );
}

#[test]
fn floors_curses_field_is_checked() {
    let (_, _, errors) = load_and_lint(&fixture_dir("floors_curses_invalid")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "curses.chance = 1.5");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "curses.chance = -0.5");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "curses.pool vide");
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "curse « fantome » inconnue",
    );
    // La curse connue du même pool n'est pas rapportée.
    assert_no_error(&errors, "curse « connue » inconnue");
}

#[test]
fn achievements_are_checked_against_the_game() {
    let (_, _, errors) = load_and_lint(&fixture_dir("achievement_invalid")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« sans_condition » : when vide",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« sans_deblocage » : unlocks vide",
    );
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "déblocage « fantome » inconnu",
    );
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "(ItemHeld) : objet « fantome » inconnu",
    );
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "(BossDefeated) : « fantome » n'est pas un personnage à `boss:`",
    );
    assert_no_error(&errors, "déblocage « connu » inconnu");
}

#[test]
fn testbed_curses_and_achievement_lint_clean() {
    let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../games/testbed"));
    let (registry, _, errors) = load_and_lint(&dir).unwrap();
    let blocking: Vec<_> = errors
        .iter()
        .filter(|e| {
            ["curse", "accomplissement", "epuisable", "maudit"]
                .iter()
                .any(|needle| e.message.contains(needle) || e.file.contains(needle))
        })
        .collect();
    assert!(blocking.is_empty(), "{blocking:#?}");
    assert_eq!(registry.curses.len(), 5);
    assert!(registry.achievements.contains_key("deux_etages"));
    let floors = registry
        .floors
        .values()
        .find(|floors| floors.id.to_string() == "maudit")
        .expect("séquence maudit");
    assert_eq!(floors.curses.as_ref().map(|c| c.pool.len()), Some(5));
    assert!(registry.loot_tables["epuisable"].def.exhaust);
    assert!(!registry.loot_tables["commun"].def.exhaust);
}

#[test]
fn curse_modifier_on_an_unread_stat_is_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("curse_unread_stat")).unwrap();
    assert_has_error(&errors, LintErrorKind::UnreadStat, "curse « muette »");
}
