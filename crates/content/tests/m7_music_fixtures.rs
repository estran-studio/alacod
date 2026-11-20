//! M-7 (`docs/conventions.md` §121) : le champ `music` du manifeste (profil et période de la pulsation) est lu et
//! borné par le lint ; l'existence du profil et le tempo exact (selon le mètre du profil) relèvent d'harmonium, jugés
//! au lancement par `game` ; le lint refuse seulement une pulsation impossible pour tout mètre (hors 20–300 par minute).

use content::{load_and_lint, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

#[test]
fn a_valid_music_entry_is_accepted() {
    let (_, manifest, errors) = load_and_lint(&fixture_dir("music_ok")).unwrap();
    // Seule l'erreur d'entrée (carte absente de la fixture) est attendue.
    let others: Vec<_> = errors
        .iter()
        .filter(|e| !e.message.contains("entry.start_map"))
        .collect();
    assert!(others.is_empty(), "{others:#?}");
    let music = manifest.music.expect("music lu");
    assert_eq!(
        (music.pulse_frames.num, music.pulse_frames.den),
        (3600, 230)
    );
}

#[test]
fn a_malformed_music_entry_is_reported() {
    for (fixture, expected) in [
        ("music_pulse_zero", "music.pulse_frames = 3600/0"),
        ("music_profile_empty", "music.profile"),
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

/// Suivi M-7 (P2) : 11 frames = 327 pulsations par minute (au moins 327 BPM à la noire), 181 frames = 19,9 par minute
/// (au plus 39,8 BPM, deux noires par pulsation) : refusées quel que soit le profil ; 180 frames (20 par minute, borne
/// comprise) passe.
#[test]
fn a_pulse_impossible_for_every_meter_is_reported() {
    for (fixture, expected) in [
        ("music_pulse_too_fast", "music.pulse_frames = 11/1"),
        ("music_pulse_too_slow", "music.pulse_frames = 181/1"),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert!(
            errors.iter().any(|e| e.kind == LintErrorKind::OutOfRange
                && e.message.contains(expected)
                && e.message.contains("20 à 300")),
            "{fixture} : {errors:#?}"
        );
    }
    let (_, _, errors) = load_and_lint(&fixture_dir("music_pulse_bounds_ok")).unwrap();
    let music: Vec<_> = errors
        .iter()
        .filter(|e| e.message.contains("music"))
        .collect();
    assert!(music.is_empty(), "{music:#?}");
}
