//! SB-3 (D126, `docs/conventions.md` §87) : le personnage d'un joueur de profil de bac à sable est vérifié
//! au lint (connu, skins `1` et `2` : D78), pas à l'essai.

use content::{load_and_lint, LintErrorKind};
use std::path::PathBuf;

#[test]
fn le_personnage_d_un_joueur_de_profil_est_verifie() {
    let dir = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sandbox_character"
    ));
    let (registry, _, errors) = load_and_lint(&dir).unwrap();
    assert_eq!(
        registry.sandbox_profiles["arene"].profile.players[0]
            .character
            .as_deref(),
        Some("heros")
    );
    let mut messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
    messages.sort();
    assert_eq!(messages.len(), 2, "{errors:#?}");
    assert!(errors
        .iter()
        .all(|e| e.kind == LintErrorKind::BrokenReference));
    assert!(
        messages[0].contains(
            "profil « fantome » : players[0].character = « fantome » : personnage inconnu"
        ),
        "{messages:?}"
    );
    assert!(
        messages[1].contains(
            "profil « sans_skin » : players[0].character = « grunt » : skin « 2 » absent"
        ),
        "{messages:?}"
    );
}
