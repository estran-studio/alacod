//! X-6h (`docs/conventions.md` §130) : avertissements sur les réglages de salle et de caméra par pièce sans effet :
//! `no_rally` sans `locks` ni `arena`, `room_fit` ou `room_slide_seconds` sans `room_frame`, `room_slide_seconds`
//! négatif ou non fini. Un contenu cohérent n'en donne aucun ; ce ne sont jamais des erreurs.

use content::{load_and_lint, LintWarningKind};
use std::path::PathBuf;

fn warnings(name: &str) -> Vec<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let (registry, manifest, errors) = load_and_lint(&dir).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    content::warnings(&registry, &manifest)
        .into_iter()
        .filter(|w| w.kind == LintWarningKind::InertRoomSetting)
        .map(|w| w.message)
        .collect()
}

#[test]
fn les_reglages_sans_effet_sont_signales() {
    let found = warnings("room_setting_inert");
    assert_eq!(found.len(), 4, "{found:#?}");
    assert!(
        found
            .iter()
            .any(|m| m.contains("« seule »") && m.contains("no_rally sans locks ni arena")),
        "{found:#?}"
    );
    assert!(
        found.iter().any(|m| m.contains("room_fit sans room_frame")),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|m| m.contains("room_slide_seconds = -0.2")),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|m| m.contains("room_slide_seconds = -0.2 sans room_frame")),
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|m| m.contains("verrouillee")),
        "no_rally avec locks est cohérent : {found:#?}"
    );
}

#[test]
fn un_contenu_coherent_n_est_pas_signale() {
    let found = warnings("room_setting_ok");
    assert!(found.is_empty(), "{found:#?}");
}
