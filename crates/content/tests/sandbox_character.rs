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

/// Copie de la fixture dont `game.ron` déclare `entry.player` (D131).
fn fixture_with_player(player: &str) -> (PathBuf, content::GameManifest, Vec<content::LintError>) {
    let from = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sandbox_character"
    ));
    let to = std::env::temp_dir().join(format!(
        "alacod-entry-player-{}-{player}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&to);
    fn copy(from: &std::path::Path, to: &std::path::Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.path().is_dir() {
                copy(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    copy(&from, &to);
    let manifest = to.join("assets/game.ron");
    let text = std::fs::read_to_string(&manifest).unwrap().replace(
        "default_seed: 1)",
        &format!("default_seed: 1, player: Some(\"{player}\"))"),
    );
    std::fs::write(&manifest, text).unwrap();
    let (_, manifest, errors) = load_and_lint(&to).unwrap();
    (to, manifest, errors)
}

/// D131 : `entry.player` désigne le personnage par défaut du joueur ; le lint D78 (skins `1` et `2`) le
/// vise, et un id inconnu est refusé.
#[test]
fn entry_player_designe_le_personnage_par_defaut_du_joueur() {
    let (dir, manifest, errors) = fixture_with_player("heros");
    assert_eq!(manifest.player_character(), "heros");
    // Les deux erreurs d'avant (profils `fantome` et `sans_skin`), aucune de plus : `heros` a ses skins.
    assert_eq!(errors.len(), 2, "{errors:#?}");
    let _ = std::fs::remove_dir_all(dir);

    let (dir, _, errors) = fixture_with_player("grunt");
    assert!(
        errors.iter().any(|e| e
            .message
            .contains("personnage « grunt » : skin « 2 » absent de skins")),
        "le joueur par défaut sans skin « 2 » : {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(dir);

    let (dir, _, errors) = fixture_with_player("personne");
    assert!(
        errors.iter().any(|e| e.file == "game.ron"
            && e.message
                .contains("entry.player = « personne » : aucun personnage")),
        "{errors:#?}"
    );
    let _ = std::fs::remove_dir_all(dir);

    // Sans `entry.player`, le défaut du moteur : `player`.
    let (_, manifest, _) = load_and_lint(&PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sandbox_character"
    )))
    .unwrap();
    assert_eq!(manifest.player_character(), "player");
}
