//! Suivi Q10 (`docs/conventions.md` §44) : lint des transformations (`SynergyDef::requires_tag`).

use std::path::PathBuf;

use content::{load_and_lint, LintError, LintErrorKind};

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn assert_has_error(errors: &[LintError], kind: LintErrorKind, needle: &str) {
    assert!(
        errors
            .iter()
            .any(|e| e.kind == kind && e.message.contains(needle)),
        "attendu une erreur {kind:?} contenant {needle:?}, obtenu :\n{errors:#?}"
    );
}

/// Une transformation sans `requires`, et une qui cumule `requires` et tag, ne sont refusées par rien.
#[test]
fn transformation_valide_n_est_pas_signalee() {
    let (_, _, errors) = load_and_lint(&fixture_dir("transformation_ok")).unwrap();
    let synergy_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.message.contains("synergie"))
        .collect();
    assert!(synergy_errors.is_empty(), "{synergy_errors:#?}");
}

#[test]
fn transformation_refusee_tag_inconnu_count_nul_trop_haut_teinte_doublon() {
    let (_, _, errors) = load_and_lint(&fixture_dir("transformation_bad")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "tag « nope » inconnu",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "count = 0 : doit être > 0",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "count = 3 : seuls 2 objet(s) portent le tag « x »",
    );
    assert_has_error(&errors, LintErrorKind::OutOfRange, "tint = \"rouge\"");
    assert_has_error(
        &errors,
        LintErrorKind::DuplicateId,
        "même ensemble d'objets",
    );
}
