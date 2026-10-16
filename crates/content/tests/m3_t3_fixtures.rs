//! M3-T3 (`docs/conventions.md` §68) : fixtures de lint des cœurs, des plafonds et des sous au sol.

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
fn refuse_when_full_needs_a_heal() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_refuse_when_full_without_heal")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "refuse_when_full exige un consommable avec un effet Heal",
    );
}

#[test]
fn gain_soul_zero_is_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_gain_soul_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "GainSoul = 0");
}

#[test]
fn currency_zero_is_refused_in_a_pool() {
    let (_, _, errors) = load_and_lint(&fixture_dir("loot_currency_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Currency(0)");
}

#[test]
fn a_shop_pool_cannot_sell_coins() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shop_currency_pool")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "a une entrée Currency");
}

#[test]
fn starting_consumables_respect_max_stack() {
    let (_, _, errors) = load_and_lint(&fixture_dir("character_starting_over_max_stack")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "dépasse max_stack = 2");
}

#[test]
fn an_unlock_cannot_add_zero_coins_to_a_pool() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_currency_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Currency(0)");
}

#[test]
fn an_unlock_cannot_add_coins_to_a_shop_pool() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_currency_shop_pool")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "pool de boutique");
}

#[test]
fn scaled_is_refused_in_an_on_use_effect() {
    let (_, _, errors) = load_and_lint(&fixture_dir("scaled_in_onuse")).unwrap();
    // Consommable et actif : l'un comme l'autre, `apply_instant_effects` n'exécute pas l'enveloppe.
    for item in ["ration", "eclat"] {
        assert!(
            errors.iter().any(|e| e.kind == LintErrorKind::Unsupported
                && e.message.contains(item)
                && e.message.contains("Scaled : pas exécuté par OnUse")),
            "{item} : {errors:#?}"
        );
    }
}
