//! SB-1 (`docs/conventions.md` §86, `docs/taches/plan-bac-a-sable.md` D1/D2) : le kind `Sandbox`,
//! les profils livrés par les jeux et le gabarit, la résolution de l'arène et la création par modèle.

use content::sandbox::{manifest_arena, SandboxProfile, DEFAULT_PROFILE};
use content::{load_and_lint, LintError, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

fn engine_dir(rel: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(rel)
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
fn un_profil_valide_passe_le_lint_et_se_resout_sur_l_arene_du_manifeste() {
    let (registry, manifest, errors) = load_and_lint(&fixture_dir("sandbox_ok")).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    let profile = &registry.sandbox_profiles[DEFAULT_PROFILE].profile;
    assert_eq!(profile.players.len(), 1);
    assert_eq!(profile.characters[0].character, "grunt");
    assert_eq!(profile.arena(&manifest), Some("maps/salle.ldtk"));
}

#[test]
fn l_arene_du_profil_l_emporte_puis_le_manifeste_puis_generate_template() {
    let (_, manifest, _) = load_and_lint(&fixture_dir("sandbox_ok")).unwrap();
    let own = SandboxProfile::parse("(map: \"maps/autre.ldtk\")").unwrap();
    assert_eq!(own.arena(&manifest), Some("maps/autre.ldtk"));
    assert_eq!(manifest_arena(&manifest), Some("maps/salle.ldtk"));

    // Sans `sandbox.arena`, le repli est `generate_template.map` (un jeu qui n'a pas écrit son arène vide).
    let without = content::manifest::GameManifest::parse(
        "(name: \"x\", content_folders: [], entry: (start_map: \"m.ldtk\", default_seed: 1), \
         generate_template: (map: \"maps/gabarit_armes.ldtk\", target: \"cible\"))",
        PathBuf::from("game.ron"),
    )
    .unwrap();
    assert!(without.sandbox.is_none());
    assert_eq!(manifest_arena(&without), Some("maps/gabarit_armes.ldtk"));
    // Les jeux livrés déclarent leur arène vide (`alacod new-arena`, §90).
    for (game, arena) in [
        ("games/zombies", "maps/arene_vide.ldtk"),
        ("games/gungeon", "maps/arene_vide.ldtk"),
        ("games/isaac", "maps/arene_vide.ldtk"),
        ("games/throne", "arene_vide.ldtk"),
        ("games/testbed", "testbed/arene_vide.ldtk"),
        ("templates/starter", "maps/arene_vide.ldtk"),
    ] {
        let (_, manifest, _) = load_and_lint(&engine_dir(game)).unwrap();
        assert_eq!(manifest_arena(&manifest), Some(arena), "{game}");
    }
}

#[test]
fn toutes_les_references_d_un_profil_sont_verifiees() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sandbox_bad_references")).unwrap();
    for needle in [
        "maps/absente.ldtk",
        "weapon = « plasma » : arme inconnue",
        "objet « fantome »",
        "mutation « inconnue »",
        "personnage « dragon »",
        "variante « geant »",
        "objet « bague_perdue »",
        "pool « pool_perdu »",
        "objet « vitrine »",
        "n'est pas un consommable du jeu",
    ] {
        assert_has_error(&errors, LintErrorKind::BrokenReference, needle);
    }
    assert_has_error(&errors, LintErrorKind::DuplicateId, "clé « c » en double");
    assert_has_error(
        &errors,
        LintErrorKind::DuplicateId,
        "shop : clé « s » en double",
    );
    assert_has_error(&errors, LintErrorKind::UnreadStat, "modifiers");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "fuse_frames et radius");
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "le montant doit être > 0",
    );
    assert_eq!(errors.len(), 15, "{errors:#?}");
}

#[test]
fn un_jeu_a_profils_sans_arene_est_refuse() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sandbox_no_arena")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "sans arène");
    assert_eq!(errors.len(), 1, "{errors:#?}");
}

#[test]
fn une_arene_declaree_inconnue_est_refusee() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sandbox_unknown_arena")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "sandbox.arena");
}

#[test]
fn le_profil_par_defaut_est_exige() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sandbox_no_default_profile")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "sans profil « arene »",
    );
}

#[test]
fn un_champ_hors_sous_ensemble_de_scenario_est_une_erreur_de_parse() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sandbox_extra_field")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "frames");
}

/// Les jeux livrés et le gabarit : zéro erreur de lint, profils `arene` et `lourd` (le gabarit : `arene`).
#[test]
fn les_jeux_livres_ont_leurs_profils() {
    for (game, profiles) in [
        ("games/zombies", &["arene", "lourd"][..]),
        ("games/gungeon", &["arene", "lourd"][..]),
        ("games/isaac", &["arene", "lourd"][..]),
        ("games/throne", &["arene", "lourd"][..]),
        ("games/testbed", &["arene", "lourd"][..]),
        ("templates/starter", &["arene"][..]),
    ] {
        let (registry, manifest, errors) = load_and_lint(&engine_dir(game)).unwrap();
        assert!(errors.is_empty(), "{game} : {errors:#?}");
        let ids: Vec<&str> = registry
            .sandbox_profiles
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(ids, profiles, "{game}");
        assert!(manifest_arena(&manifest).is_some(), "{game} : arène");
    }
}

/// `alacod describe Sandbox` : schéma exhaustif (champs du sous-ensemble) et ids des profils.
#[test]
fn describe_expose_le_kind_et_son_schema() {
    let description = content::describe::describe(&engine_dir("games/testbed")).unwrap();
    let info = content::describe::find_kind(&description, "Sandbox").expect("kind Sandbox");
    assert_eq!(info.ids, ["arene", "lourd"]);
    let schema = serde_json::to_string(info.schema.as_ref().expect("schéma")).unwrap();
    for field in [
        "players",
        "characters",
        "chests",
        "shop",
        "mutations",
        "currency",
        "dash",
    ] {
        assert!(schema.contains(field), "champ {field} absent du schéma");
    }
    assert!(
        !schema.contains("\"inputs\""),
        "inputs n'est pas du sous-ensemble"
    );
}

/// `alacod new-content Sandbox <id>` : copie du premier profil, qui passe encore le lint.
#[test]
fn new_content_cree_un_profil_par_modele() {
    let dir = std::env::temp_dir().join(format!("alacod-sb1-new-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&fixture_dir("sandbox_ok"), &dir);
    let created = content::scaffold::new_content(&dir, "Sandbox", "lourd", None).unwrap();
    assert_eq!(created.model, "arene");
    assert!(dir.join("assets/sandbox/lourd.ron").is_file());
    let (registry, _, errors) = load_and_lint(&dir).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(registry.sandbox_profiles.contains_key("lourd"));
    std::fs::remove_dir_all(&dir).unwrap();
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
