//! M3-T8 (`docs/conventions.md` §73) : fixtures de lint des pools à épuisement, du prix propre d'un
//! présentoir (`shop.cost`) et du butin de boss (`boss_drop`).

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
fn an_exhausting_pool_cannot_exclude_held_items() {
    let (_, _, errors) = load_and_lint(&fixture_dir("loot_exhaust_exclude_held")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "« tresor » : exhaust et exclude_held sont incompatibles",
    );
}

#[test]
fn shelf_costs_are_checked() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shop_cost_invalid")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« diable_zero » : boutique : cost : prix MaxHealth(0)",
    );
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "« diable_fantome » : boutique : cost : « fantome » n'est pas un consommable",
    );
}

#[test]
fn a_boss_drop_needs_a_known_pool() {
    let (_, _, errors) = load_and_lint(&fixture_dir("boss_drop_unknown_pool")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "« boss » : boss_drop : pool « fantome » inconnu",
    );
}
