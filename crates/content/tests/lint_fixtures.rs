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
fn entry_mode_floors_without_floors_fixture_reports_missing_floors_folder() {
    let (_, _, errors) = load_and_lint(&fixture_dir("entry_mode_floors_without_floors")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "entry.mode = Floors",
    );
}

#[test]
fn floors_empty_fixture_reports_empty_levels() {
    let (_, _, errors) = load_and_lint(&fixture_dir("floors_empty")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "levels vide");
}

#[test]
fn surface_fixtures_report_duplicate_value_and_non_positive_factors() {
    let (_, _, errors) = load_and_lint(&fixture_dir("surface_duplicate_value")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "intgrid_value = 1");
    let (_, _, errors) = load_and_lint(&fixture_dir("surface_factor_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "move_speed = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "acceleration = -1");
}

#[test]
fn effect_fixtures_report_unsupported_and_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_unsupported")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "OnHit");
    assert_has_error(&errors, LintErrorKind::Unsupported, "SquadSize");
    assert_has_error(&errors, LintErrorKind::Unsupported, "RefillAmmo");
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Tick(0)");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Heal = 0");
}

/// T1.12 : fixtures de l'audit des kinds de M1 (références vérifiées sans fixture avant).
#[test]
fn m1_audit_fixtures() {
    let cases: &[(&str, LintErrorKind, &[&str])] = &[
        (
            "behavior_shoot_unknown",
            LintErrorKind::BrokenReference,
            &[
                "arme inconnue « lance_flammes »",
                "pattern inconnu « spirale »",
            ],
        ),
        (
            "cave_unknown_character",
            LintErrorKind::BrokenReference,
            &["« fantome »"],
        ),
        (
            "cave_out_of_range",
            LintErrorKind::OutOfRange,
            &["au moins 16 × 16", "fill_ratio = 1.5", "birth = 9"],
        ),
        (
            "floors_unknown_cave",
            LintErrorKind::BrokenReference,
            &["aucune caverne chargée"],
        ),
        (
            "entry_clock_unknown",
            LintErrorKind::BrokenReference,
            &["entry.clocks : horloge inconnue « absente »"],
        ),
        (
            "entry_difficulty_missing",
            LintErrorKind::BrokenReference,
            &["entry.difficulty"],
        ),
        (
            "effect_broken_reference",
            LintErrorKind::BrokenReference,
            &["pattern « spirale » inconnu", "arme « laser » inconnue"],
        ),
        (
            "map_character_unknown",
            LintErrorKind::BrokenReference,
            &["personnage inconnu « fantome »"],
        ),
        (
            "powerup_apply_status",
            LintErrorKind::OutOfRange,
            &["ApplyStatus"],
        ),
        (
            "entry_progression_unknown",
            LintErrorKind::BrokenReference,
            &["progression inconnue « absente »"],
        ),
        (
            "cave_template_missing",
            LintErrorKind::BrokenReference,
            &["gabarit LDtk"],
        ),
    ];
    for (name, kind, needles) in cases {
        let (_, _, errors) = load_and_lint(&fixture_dir(name)).unwrap();
        for needle in *needles {
            assert_has_error(&errors, *kind, needle);
        }
    }
}

/// D36 : une caverne sans ennemi dans une séquence `Floors` est refusée (pas de personnage, ou
/// pas de point d'apparition), sauf `transit: true` ; une caverne peuplée passe.
#[test]
fn floors_cave_without_enemies_fixture() {
    let (_, _, errors) = load_and_lint(&fixture_dir("floors_cave_without_enemies")).unwrap();
    let d36: Vec<_> = errors
        .iter()
        .filter(|e| e.message.contains("caverne sans ennemi"))
        .collect();
    assert_eq!(d36.len(), 2, "{errors:#?}");
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« cave:vide » : caverne sans ennemi",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« cave:sans_points » : caverne sans ennemi",
    );
    assert_eq!(
        errors.len(),
        2,
        "aucune autre erreur attendue : {errors:#?}"
    );
}

#[test]
fn status_fixtures() {
    let (_, _, errors) = load_and_lint(&fixture_dir("status_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "frames = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "damage = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "period = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "factor = 2");
    let (_, _, errors) = load_and_lint(&fixture_dir("status_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« givre »");
}

#[test]
fn progression_and_mutation_fixtures() {
    let (_, _, errors) = load_and_lint(&fixture_dir("progression_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "per_kill = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "levels[1] = 2");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "choices = 5");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "choice_frames = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "weapon_drop_chance = 2");
    let (_, _, errors) = load_and_lint(&fixture_dir("progression_broken_reference")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« absente »");
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« laser »");
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« mana »");
    let (_, _, errors) = load_and_lint(&fixture_dir("mutation_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "weight = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "max_stacks = 0");
}

#[test]
fn floors_unknown_map_fixture_reports_broken_level() {
    let (_, _, errors) = load_and_lint(&fixture_dir("floors_unknown_map")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "cartes/absente.ldtk",
    );
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
        // D40.
        (
            "powerup_refill_ammo_unknown",
            LintErrorKind::BrokenReference,
        ),
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
        // T1.8.
        (
            "entry_mode_floors_without_floors",
            LintErrorKind::BrokenReference,
        ),
        ("floors_empty", LintErrorKind::OutOfRange),
        ("floors_unknown_map", LintErrorKind::BrokenReference),
        // T1.7.
        ("surface_duplicate_value", LintErrorKind::DuplicateId),
        ("surface_factor_non_positive", LintErrorKind::OutOfRange),
        ("effect_unsupported", LintErrorKind::Unsupported),
        ("effect_out_of_range", LintErrorKind::OutOfRange),
        ("progression_out_of_range", LintErrorKind::OutOfRange),
        (
            "progression_broken_reference",
            LintErrorKind::BrokenReference,
        ),
        ("mutation_out_of_range", LintErrorKind::OutOfRange),
        ("status_out_of_range", LintErrorKind::OutOfRange),
        ("status_unknown", LintErrorKind::BrokenReference),
        ("behavior_shoot_unknown", LintErrorKind::BrokenReference),
        ("cave_unknown_character", LintErrorKind::BrokenReference),
        ("cave_out_of_range", LintErrorKind::OutOfRange),
        ("floors_unknown_cave", LintErrorKind::BrokenReference),
        ("floors_cave_without_enemies", LintErrorKind::OutOfRange),
        ("entry_clock_unknown", LintErrorKind::BrokenReference),
        ("entry_difficulty_missing", LintErrorKind::BrokenReference),
        ("effect_broken_reference", LintErrorKind::BrokenReference),
        ("map_character_unknown", LintErrorKind::BrokenReference),
        ("powerup_apply_status", LintErrorKind::OutOfRange),
        ("entry_progression_unknown", LintErrorKind::BrokenReference),
        ("cave_template_missing", LintErrorKind::BrokenReference),
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

#[test]
fn weapons_duplicate_key_fixture_reports_repeated_key() {
    let (_, _, errors) = load_and_lint(&fixture_dir("weapons_duplicate_key")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "rifle");
}

#[test]
fn melee_duplicate_key_fixture_reports_repeated_key() {
    let (_, _, errors) = load_and_lint(&fixture_dir("melee_duplicate_key")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "club");
}

#[test]
fn powerups_duplicate_key_fixture_reports_error() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerups_duplicate_key")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "max_ammo");
}

#[test]
fn powerup_timed_mul_non_positive_fixture_reports_error() {
    let (mut registry, manifest, errors) =
        load_and_lint(&fixture_dir("powerup_timed_mul_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "value");
    use sim_core::modifier::ModifierOp;
    for (op, value, invalid) in [
        (ModifierOp::Mul, "-1.0", true),
        (ModifierOp::Mul, "0.0", true),
        (ModifierOp::Mul, "1.0", false),
        (ModifierOp::Set, "0.0", false),
        (ModifierOp::Add, "-1.0", false),
        (ModifierOp::Pct, "-0.5", false),
    ] {
        let powerup = registry.powerups.values_mut().next().unwrap();
        powerup.actions = vec![effects::Action::TimedModifier {
            stat: sim_core::stats::StatId::MoveSpeed,
            op,
            value: value.parse().unwrap(),
            frames: 1800,
        }];
        let errors = content::lint::run(&registry, &manifest);
        assert_eq!(
            errors.iter().any(|e| e.kind == LintErrorKind::OutOfRange),
            invalid,
            "{op:?} {value}: {errors:#?}"
        );
    }
}

#[test]
fn feedback_frames_zero_fixture_reports_error() {
    let (_, _, errors) = load_and_lint(&fixture_dir("feedback_frames_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "hit_flash.frames");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "shake.frames");
}

#[test]
fn feedback_amplitude_negative_fixture_reports_error() {
    let (mut registry, manifest, errors) =
        load_and_lint(&fixture_dir("feedback_amplitude_negative")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "amplitude");
    for amplitude in [0.0, 1.0] {
        registry.feedback[0].settings.shake.amplitude = amplitude;
        let errors = content::lint::run(&registry, &manifest);
        assert!(
            !errors.iter().any(|e| e.kind == LintErrorKind::OutOfRange),
            "{errors:#?}"
        );
    }
}

#[test]
fn feedback_missing_sound_fixture_reports_error() {
    let (_, _, errors) = load_and_lint(&fixture_dir("feedback_missing_sound")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "sounds/absent.ogg");
    assert!(!errors
        .iter()
        .any(|e| e.message.contains("sounds/present.ogg")));
}

#[test]
fn feedback_t1_17_fixtures_report_their_field() {
    let (_, _, errors) = load_and_lint(&fixture_dir("feedback_by_weapon_unknown")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "by_weapon « fantome »",
    );
    let (_, _, errors) = load_and_lint(&fixture_dir("feedback_override_out_of_range")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "by_kind.Explosion.shake.frames",
    );
}

#[test]
fn starting_weapons_exceed_slots_fixture_reports_error() {
    let (mut registry, manifest, errors) =
        load_and_lint(&fixture_dir("starting_weapons_exceed_slots")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "weapon_slots");
    for slots in [3, 4] {
        registry
            .characters
            .values_mut()
            .next()
            .unwrap()
            .weapon_slots = slots;
        let errors = content::lint::run(&registry, &manifest);
        assert!(
            !errors.iter().any(|e| e.kind == LintErrorKind::OutOfRange),
            "{errors:#?}"
        );
    }
}

/// T3.4 : exactement l'erreur visée par fixture, en plus du start_map commun.
#[test]
fn t3_4_fixtures_have_a_single_rule_failure() {
    for (name, kind, count) in [
        ("weapons_duplicate_key", LintErrorKind::DuplicateId, 1),
        ("melee_duplicate_key", LintErrorKind::DuplicateId, 1),
        ("powerups_duplicate_key", LintErrorKind::DuplicateId, 1),
        (
            "powerup_timed_mul_non_positive",
            LintErrorKind::OutOfRange,
            1,
        ),
        ("feedback_frames_zero", LintErrorKind::OutOfRange, 2),
        ("feedback_amplitude_negative", LintErrorKind::OutOfRange, 1),
        ("feedback_missing_sound", LintErrorKind::BrokenReference, 1),
        // T1.17
        (
            "feedback_by_weapon_unknown",
            LintErrorKind::BrokenReference,
            1,
        ),
        (
            "feedback_override_out_of_range",
            LintErrorKind::OutOfRange,
            1,
        ),
        (
            "starting_weapons_exceed_slots",
            LintErrorKind::OutOfRange,
            1,
        ),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(name)).unwrap();
        let others: Vec<_> = errors
            .iter()
            .filter(|e| !e.message.contains("entry.start_map"))
            .collect();
        assert_eq!(others.len(), count, "{name}: {errors:#?}");
        assert!(others.iter().all(|e| e.kind == kind), "{name}: {errors:#?}");
    }
}

/// T1.16 : écran de mutation (`ui/mutation_screen.ron`) — police absente, deux emplacements ;
/// T1.18 : source de HUD inconnue ;
/// une seule erreur par fixture en plus du start_map commun.
#[test]
fn mutation_screen_fixtures() {
    for (name, kind, needle) in [
        (
            "mutation_screen_font_missing",
            LintErrorKind::BrokenReference,
            "champ font = « fonts/absente.ttf »",
        ),
        (
            "mutation_screen_two_slots",
            LintErrorKind::OutOfRange,
            "2 emplacements, il en faut 3",
        ),
        (
            "hud_unknown_source",
            LintErrorKind::UnknownKind,
            "hud : source inconnue « mana »",
        ),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(name)).unwrap();
        assert_has_error(&errors, kind, needle);
        let others: Vec<_> = errors
            .iter()
            .filter(|e| !e.message.contains("entry.start_map"))
            .collect();
        assert_eq!(others.len(), 1, "{name}: {errors:#?}");
    }
}

// D3 : table des feuilles de sprites (kind `SpriteSheet`).

/// Une seule erreur en plus de `start_map: "unused"`, commune à toutes les fixtures.
fn assert_only_one_besides_start_map(errors: &[LintError]) {
    let others: Vec<_> = errors
        .iter()
        .filter(|e| !e.message.contains("start_map"))
        .collect();
    assert_eq!(others.len(), 1, "une seule erreur attendue :\n{errors:#?}");
}

#[test]
fn sprite_sheet_missing_file_fixture_reports_missing_layer_sheet() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sprite_sheet_missing_file")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "sprites/hero_sheet.ron",
    );
    assert_only_one_besides_start_map(&errors);
}

#[test]
fn sprite_sheet_missing_image_fixture_reports_missing_png() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sprite_sheet_missing_image")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "sprites/hero.png");
    assert_only_one_besides_start_map(&errors);
}

#[test]
fn weapon_sprite_unknown_fixture_reports_unknown_sheet() {
    let (_, _, errors) = load_and_lint(&fixture_dir("weapon_sprite_unknown")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "sprite_config.name",
    );
    assert_only_one_besides_start_map(&errors);
}

#[test]
fn sprite_sheet_duplicate_key_fixture_reports_repeated_id() {
    let (_, _, errors) = load_and_lint(&fixture_dir("sprite_sheet_duplicate_key")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "hero");
    assert_only_one_besides_start_map(&errors);
}

// T1.1 (B5 v1) : projectiles composables.

#[test]
fn projectile_fixtures_have_a_single_rule_failure() {
    for (name, kind, needle) in [
        (
            "projectile_broken_reference",
            LintErrorKind::BrokenReference,
            "« shrapnel » absent",
        ),
        (
            "projectile_temporal_pattern",
            LintErrorKind::OutOfRange,
            "pattern temporel",
        ),
        // Le cycle a -> b -> a est vu depuis chacun des deux projectiles.
        ("projectile_cycle", LintErrorKind::OutOfRange, "cycle"),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(name)).unwrap();
        let others: Vec<_> = errors
            .iter()
            .filter(|e| !e.message.contains("entry.start_map"))
            .collect();
        assert!(!others.is_empty(), "{name}: {errors:#?}");
        assert!(
            others
                .iter()
                .all(|e| e.kind == kind && e.message.contains(needle)),
            "{name}: {errors:#?}"
        );
    }
}

// T1.2 : patterns nommés, émetteurs et tir ennemi.

#[test]
fn pattern_unknown_name_fixture_reports_broken_named_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("pattern_unknown_name")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "« absent » inconnu",
    );
}

#[test]
fn pattern_scatter_on_expire_fixture_reports_emitter_only_pattern() {
    let (_, _, errors) = load_and_lint(&fixture_dir("pattern_scatter_on_expire")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Scatter");
}

#[test]
fn ranged_projectile_missing_fixture_reports_projectile_absent_from_weapon_table() {
    let (_, _, errors) = load_and_lint(&fixture_dir("ranged_projectile_missing")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« ember »");
}

#[test]
fn ranged_cooldown_zero_fixture_reports_cooldown() {
    let (_, _, errors) = load_and_lint(&fixture_dir("ranged_cooldown_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "cooldown_frames = 0");
}

// T1.4 : behaviors composables (`ai.behaviors`, `ai.targeting`).

#[test]
fn behavior_melee_unknown_fixture_reports_unknown_melee_weapon() {
    let (_, _, errors) = load_and_lint(&fixture_dir("behavior_melee_unknown")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "« griffes_absentes »",
    );
}

#[test]
fn behavior_keep_distance_inverted_fixture_reports_empty_band() {
    let (_, _, errors) = load_and_lint(&fixture_dir("behavior_keep_distance_inverted")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "KeepDistance");
}

#[test]
fn behavior_charge_zero_fixture_reports_zero_telegraph() {
    let (_, _, errors) = load_and_lint(&fixture_dir("behavior_charge_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "telegraph = 0");
}

#[test]
fn behavior_unknown_profile_fixture_reports_chase_profile() {
    let (_, _, errors) = load_and_lint(&fixture_dir("behavior_unknown_profile")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« Swimming »");
}

#[test]
fn targeting_unknown_tag_fixture_reports_ignored_tag() {
    let (_, _, errors) = load_and_lint(&fixture_dir("targeting_unknown_tag")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« fantome »");
}

// T1.5 : variantes et élites (`variants`, champ LDtk `variant`).

#[test]
fn variant_weight_zero_fixture_reports_weight() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_weight_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "weight = 0");
}

#[test]
fn variant_chance_out_of_range_fixture_reports_chance() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_chance_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "chance = 1.5");
}

#[test]
fn variant_skin_unknown_fixture_reports_skin() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_skin_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "skin « dore »");
}

#[test]
fn variant_move_speed_ai_fixture_points_to_enemy_move_speed() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_move_speed_ai")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "EnemyMoveSpeed");
}

#[test]
fn variant_duplicate_fixture_reports_duplicate_name() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_duplicate")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "« rapide » en double");
}

#[test]
fn variant_ldtk_unknown_fixture_reports_forced_variant() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_ldtk_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "variant « dore »");
}

// T1.9 : horloges et difficulté.

#[test]
fn clock_duplicate_id_fixture_reports_duplicate_event() {
    let (_, _, errors) = load_and_lint(&fixture_dir("clock_duplicate_id")).unwrap();
    assert_has_error(&errors, LintErrorKind::DuplicateId, "« tic » en double");
}

#[test]
fn clock_unordered_fixture_reports_decreasing_deadline() {
    let (_, _, errors) = load_and_lint(&fixture_dir("clock_unordered")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "croissantes");
}

#[test]
fn clock_repeat_zero_fixture_reports_repeat() {
    let (_, _, errors) = load_and_lint(&fixture_dir("clock_repeat_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "repeat = 0");
}

#[test]
fn difficulty_unknown_identifier_fixture_lists_allowed_identifiers() {
    let (_, _, errors) = load_and_lint(&fixture_dir("difficulty_unknown_identifier")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "identifiants admis");
}

#[test]
fn difficulty_non_positive_fixture_reports_value() {
    let (_, _, errors) = load_and_lint(&fixture_dir("difficulty_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "doit être > 0");
}

#[test]
fn character_test_frames_zero_fixture_reports_frames() {
    let (_, _, errors) = load_and_lint(&fixture_dir("character_test_frames_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "test.frames = 0");
}

#[test]
fn character_test_no_template_fixture_reports_templates() {
    let (_, _, errors) = load_and_lint(&fixture_dir("character_test_no_template")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "aucun gabarit actif");
}

#[test]
fn character_test_expect_inactive_fixture_reports_template() {
    let (_, _, errors) = load_and_lint(&fixture_dir("character_test_expect_inactive")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "expect_moving sans gabarit moving",
    );
}

#[test]
fn generate_template_unknown_map_fixture_reports_map() {
    let (_, _, errors) = load_and_lint(&fixture_dir("generate_template_unknown_map")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "generate_template.map",
    );
}

#[test]
fn generate_template_target_without_hits_fixture_reports_counts_hits() {
    let (_, _, errors) =
        load_and_lint(&fixture_dir("generate_template_target_without_hits")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "counts_hits: true");
}

#[test]
fn powerup_refill_ammo_unknown_fixture_reports_ammo() {
    let (_, _, errors) = load_and_lint(&fixture_dir("powerup_refill_ammo_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "RefillAmmoOf");
}

/// D41 (m1-d41-spawns-degages) : sur le contenu réel, toutes les cavernes gardent un
/// dégagement de 1 (points d'apparition inchangés) ; le boss de throne (collider 20, `scale`
/// 1.4 : 28 px en jeu) s'étend à 22,4 px du centre.
#[test]
fn degagement_des_cavernes_du_contenu_reel() {
    for game in ["throne", "testbed", "zombies"] {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../games/{game}"));
        let (registry, _, _) = load_and_lint(&dir).unwrap();
        for (id, cave) in &registry.caves {
            assert_eq!(cave.config.spawn_clearance, 1, "{game} : caverne {id}");
        }
        if game == "throne" {
            let boss = &registry.characters[&content::registry::CharacterId::from("roi_rat")];
            assert!(
                (boss.body_extent.to_num::<f64>() - 22.4).abs() < 0.01,
                "{}",
                boss.body_extent
            );
        }
    }
}
