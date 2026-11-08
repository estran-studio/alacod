//! M-5 (`docs/conventions.md` §113) : le mode « feu follet » d'un personnage (`wisp`) est lu et borné par le lint.

use content::{load_and_lint, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

#[test]
fn a_valid_wisp_is_accepted() {
    let (registry, _, errors) = load_and_lint(&fixture_dir("wisp_ok")).unwrap();
    // Seule l'erreur d'entrée (carte absente de la fixture) est attendue.
    let others: Vec<_> = errors
        .iter()
        .filter(|e| !e.message.contains("entry.start_map"))
        .collect();
    assert!(others.is_empty(), "{others:#?}");
    let wisp = registry
        .characters
        .values()
        .find_map(|character| character.wisp)
        .expect("le feu follet du personnage est lu");
    assert_eq!(wisp.respawn_frames, Some(90));
    assert_eq!(wisp.bleedout_frames, None);
}

#[test]
fn out_of_range_wisp_numbers_are_reported() {
    for (fixture, expected) in [
        ("wisp_respawn_zero", "wisp.respawn_frames = 0"),
        ("wisp_lure_range_zero", "wisp.lure.range = 0"),
        ("wisp_speed_too_high", "wisp.speed_mult = 5"),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert!(
            errors
                .iter()
                .any(|e| e.kind == LintErrorKind::OutOfRange && e.message.contains(expected)),
            "{fixture} : {errors:#?}"
        );
    }
}
