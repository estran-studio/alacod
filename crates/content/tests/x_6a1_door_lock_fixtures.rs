//! X-6a1 (`docs/conventions.md` §117) : une porte de carte à `lock_key` dont aucun personnage ne porte la clé
//! (`carries_key`) est signalée (`UnobtainableKey`, avertissement : la porte ne s'ouvrirait jamais) ; une clé portée, non.
//! Une porte sans `lock_key` (champ vide) n'est jamais signalée ; un `carries_key` vide est une erreur de lint.

use content::{load_and_lint, LintErrorKind, LintWarningKind};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn unobtainable(name: &str) -> Vec<String> {
    unobtainable_in(&fixture(name))
}

fn unobtainable_in(dir: &PathBuf) -> Vec<String> {
    let (registry, manifest, errors) = load_and_lint(dir).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    content::warnings(&registry, &manifest)
        .into_iter()
        .filter(|w| w.kind == LintWarningKind::UnobtainableKey)
        .map(|w| w.message)
        .collect()
}

#[test]
fn une_serrure_sans_porteur_de_cle_est_signalee_une_fois() {
    let warnings = unobtainable("door_lock_unobtainable");
    assert_eq!(warnings.len(), 1, "{warnings:#?}");
    assert!(
        warnings[0].contains("« cle_x »") && warnings[0].contains("carries_key"),
        "{warnings:?}"
    );
}

#[test]
fn une_serrure_dont_un_personnage_porte_la_cle_n_est_pas_signalee() {
    assert!(unobtainable("door_lock_carried").is_empty());
}

#[test]
fn le_testbed_n_a_aucune_serrure_sans_cle() {
    assert!(unobtainable_in(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games/testbed")
    )
    .is_empty());
}

#[test]
fn une_cle_portee_vide_est_une_erreur_de_lint() {
    let (_, _, errors) = load_and_lint(&fixture("carries_key_empty")).unwrap();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].kind, LintErrorKind::OutOfRange);
    assert!(
        errors[0].message.contains("carries_key") && errors[0].message.contains("non vide"),
        "{errors:#?}"
    );
}
