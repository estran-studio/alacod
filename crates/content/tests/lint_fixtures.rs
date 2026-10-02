//! Une fixture par erreur de lint (carte T1.5, `docs/taches.md`) : `tests/fixtures/<erreur>/`
//! est un mini-jeu (`assets/game.ron` + contenu minimal) avec un seul problème volontaire.
//! Plus un test qui linte `games/zombies` et `games/testbed` sans erreur.

use content::{load_and_lint, LintError, LintErrorKind};
use std::path::PathBuf;

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

/// `games/<jeu>`, relatif à `CARGO_MANIFEST_DIR` de `content` (`crates/content`).
fn game_dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../games")).join(name)
}

fn assert_has_error(errors: &[LintError], kind: LintErrorKind, needle: &str) {
    let found = errors
        .iter()
        .find(|e| e.kind == kind && e.message.contains(needle));
    assert!(
        found.is_some(),
        "attendu une erreur {kind:?} contenant {needle:?}, obtenu :\n{errors:#?}"
    );
}

#[test]
fn broken_reference_fixture_reports_unknown_weapon() {
    let (_, _, errors) = load_and_lint(&fixture_dir("broken_reference")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "plasma_rifle");
}

#[test]
fn duplicate_id_fixture_reports_repeated_asset_name_ref() {
    let (_, _, errors) = load_and_lint(&fixture_dir("duplicate_id")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "hero");
}

#[test]
fn out_of_range_fixture_reports_non_positive_health() {
    let (_, _, errors) = load_and_lint(&fixture_dir("out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "base_health.max");
}

#[test]
fn out_of_range_stat_fixture_reports_negative_stat() {
    let (_, _, errors) = load_and_lint(&fixture_dir("out_of_range_stat")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "stats.MoveSpeed");
}

#[test]
fn unknown_kind_fixture_reports_unsupported_content_kind() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unknown_kind")).unwrap();
    assert_has_error(&errors, LintErrorKind::UnknownKind, "Npc");
}

#[test]
fn float_literal_fixture_reports_bare_number_in_fixed_field() {
    let (_, _, errors) = load_and_lint(&fixture_dir("float_literal")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "littéral");
}

#[test]
fn powerup_weight_zero_fixture_reports_zero_weight() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_weight_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "weight");
}

#[test]
fn powerup_drop_chance_out_of_range_fixture_reports_drop_chance() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_drop_chance_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "drop_chance");
}

#[test]
fn powerup_duplicate_id_fixture_reports_repeated_powerup_id() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_duplicate_id")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "insta_kill");
}

#[test]
fn powerup_unknown_stat_fixture_reports_parse_error() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_unknown_stat")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "PasUneStatConnue");
}

// ---------------------------------------------------------------------------------------
// T2.8 : règles des kinds ajoutés depuis T1.5 (munitions, tir ami, perks, power-ups,
// économie, sons d'arme). Une fixture par règle, un seul problème par fixture (en plus de
// `start_map: "unused"`, commun à toutes les fixtures).
// ---------------------------------------------------------------------------------------

#[test]
fn entry_mode_waves_without_waves_fixture_reports_missing_wave_folder() {
    let (_, _, errors) = load_and_lint(&fixture_dir("entry_mode_waves_without_waves")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "entry.mode");
}

#[test]
fn ammo_type_empty_custom_fixture_reports_empty_name() {
    let (_, _, errors) = load_and_lint(&fixture_dir("ammo_type_empty_custom")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "ammo_type");
}

#[test]
fn ammo_type_unknown_fixture_reports_parse_error_with_field_and_value() {
    let (_, _, errors) = load_and_lint(&fixture_dir("ammo_type_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "ammo_type");
    assert_has_error(&errors, LintErrorKind::Parse, "Laser");
}

#[test]
fn friendly_fire_unknown_fixture_reports_parse_error_with_field_and_value() {
    let (_, _, errors) = load_and_lint(&fixture_dir("friendly_fire_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "friendly_fire");
    assert_has_error(&errors, LintErrorKind::Parse, "Sometimes");
}

#[test]
fn perk_no_modifiers_fixture_reports_empty_modifiers() {
    let (_, _, errors) = load_and_lint(&fixture_dir("perk_no_modifiers")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "modifiers");
}

#[test]
fn perk_mul_non_positive_fixture_reports_value() {
    let (_, _, errors) = load_and_lint(&fixture_dir("perk_mul_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "value");
}

#[test]
fn perk_unknown_stat_fixture_reports_parse_error_with_field_and_value() {
    let (_, _, errors) = load_and_lint(&fixture_dir("perk_unknown_stat")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "stat");
    assert_has_error(&errors, LintErrorKind::Parse, "Bonheur");
}

#[test]
fn perk_duplicate_id_fixture_reports_repeated_key_in_same_file() {
    let (_, _, errors) = load_and_lint(&fixture_dir("perk_duplicate_id")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "juggernog");
}

#[test]
fn powerup_frames_zero_fixture_reports_each_timed_action() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_frames_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "TimedModifier");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "CurrencyMultiplier");
    let frames_errors = errors
        .iter()
        .filter(|e| e.kind == LintErrorKind::OutOfRange && e.message.contains("frames"))
        .count();
    assert_eq!(
        frames_errors, 2,
        "une erreur `frames` par action :\n{errors:#?}"
    );
}

#[test]
fn powerup_factor_non_positive_fixture_reports_factor() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_factor_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "factor");
}

#[test]
fn powerup_no_actions_fixture_reports_empty_actions() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_no_actions")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "actions");
}

#[test]
fn powerup_lifetime_zero_fixture_reports_lifetime_frames() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_lifetime_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "lifetime_frames");
}

#[test]
fn powerup_pickup_range_non_positive_fixture_reports_pickup_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_pickup_range_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "pickup_range");
}

#[test]
fn economy_ratio_out_of_range_fixture_reports_refill_price_ratio() {
    let (_, _, errors) = load_and_lint(&fixture_dir("economy_ratio_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "refill_price_ratio");
}

#[test]
fn audio_missing_file_fixture_reports_missing_sound_path() {
    let (_, _, errors) = load_and_lint(&fixture_dir("audio_missing_file")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "sounds/rifle-reloading.ogg",
    );
    // `firing` pointe un fichier présent : aucune erreur pour lui.
    assert!(
        !errors
            .iter()
            .any(|e| e.message.contains("sounds/rifle.ogg")),
        "son présent signalé à tort :\n{errors:#?}"
    );
}

/// Chaque fixture ne porte qu'un problème : en dehors de `start_map: "unused"` (commun à
/// toutes), elle ne produit que les erreurs du kind attendu.
#[test]
fn t2_8_fixtures_have_a_single_problem() {
    let cases: &[(&str, LintErrorKind)] = &[
        (
            "entry_mode_waves_without_waves",
            LintErrorKind::BrokenReference,
        ),
        ("ammo_type_empty_custom", LintErrorKind::OutOfRange),
        ("ammo_type_unknown", LintErrorKind::Parse),
        ("friendly_fire_unknown", LintErrorKind::Parse),
        ("perk_no_modifiers", LintErrorKind::OutOfRange),
        ("perk_mul_non_positive", LintErrorKind::OutOfRange),
        ("perk_unknown_stat", LintErrorKind::Parse),
        ("perk_duplicate_id", LintErrorKind::DuplicateId),
        ("powerup_frames_zero", LintErrorKind::OutOfRange),
        ("powerup_factor_non_positive", LintErrorKind::OutOfRange),
        ("powerup_weight_zero", LintErrorKind::OutOfRange),
        (
            "powerup_drop_chance_out_of_range",
            LintErrorKind::OutOfRange,
        ),
        ("powerup_no_actions", LintErrorKind::OutOfRange),
        ("powerup_lifetime_zero", LintErrorKind::OutOfRange),
        (
            "powerup_pickup_range_non_positive",
            LintErrorKind::OutOfRange,
        ),
        ("economy_ratio_out_of_range", LintErrorKind::OutOfRange),
        ("audio_missing_file", LintErrorKind::BrokenReference),
    ];
    for (name, kind) in cases {
        let (_, _, errors) = load_and_lint(&fixture_dir(name)).unwrap();
        let others: Vec<_> = errors
            .iter()
            .filter(|e| !e.message.contains("entry.start_map"))
            .collect();
        assert!(
            !others.is_empty() && others.iter().all(|e| e.kind == *kind),
            "fixture {name} : attendu uniquement des erreurs {kind:?}, obtenu :\n{errors:#?}"
        );
    }
}

#[test]
fn zombies_and_testbed_lint_without_error() {
    for game in ["zombies", "testbed"] {
        let (registry, _, errors) = load_and_lint(&game_dir(game))
            .unwrap_or_else(|e| panic!("games/{game} : manifeste invalide : {e}"));
        assert!(
            errors.is_empty(),
            "games/{game} : erreurs de lint inattendues :\n{errors:#?}"
        );
        assert!(
            !registry.characters.is_empty(),
            "games/{game} : aucun personnage chargé (game.ron incomplet ?)"
        );
    }
}
