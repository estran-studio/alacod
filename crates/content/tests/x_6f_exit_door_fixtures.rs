//! X-6f (`docs/conventions.md` §128) : lint d'une séquence `Floors` qui sort par porte (`exit_by_door: true`) : chaque
//! niveau sauf le dernier doit porter une porte de sortie (`exit_next_level`), le dernier n'en porte aucune (il n'y a
//! pas de niveau suivant). Sans `exit_by_door` (défaut), aucune règle : les séquences existantes ne changent pas.

use content::{load_and_lint, LintErrorKind};
use std::path::PathBuf;

fn lint(name: &str) -> Vec<content::LintError> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    load_and_lint(&dir).unwrap().2
}

#[test]
fn une_sequence_a_sortie_par_porte_bien_formee_passe() {
    let errors = lint("exit_door_ok");
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn une_porte_de_sortie_sur_le_dernier_niveau_est_refusee() {
    let errors = lint("exit_door_on_last");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].kind, LintErrorKind::OutOfRange);
    assert!(
        errors[0].message.contains("maps/dernier.ldtk")
            && errors[0].message.contains("dernier niveau"),
        "{errors:#?}"
    );
}

#[test]
fn un_niveau_sans_porte_de_sortie_est_refuse() {
    let errors = lint("exit_door_missing");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].kind, LintErrorKind::OutOfRange);
    assert!(
        errors[0].message.contains("maps/premier.ldtk")
            && errors[0].message.contains("aucune porte de sortie"),
        "{errors:#?}"
    );
}

#[test]
fn sans_exit_by_door_aucune_regle_ne_s_applique() {
    let errors = lint("exit_door_default");
    assert!(errors.is_empty(), "{errors:#?}");
}
