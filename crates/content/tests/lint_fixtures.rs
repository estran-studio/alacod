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
fn pit_and_prop_fixtures_are_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("pit_fall_damage_missing")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "fall_damage");
    let (_, _, errors) = load_and_lint(&fixture_dir("pit_fall_damage_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "fall_damage = 0");
    let (_, _, errors) = load_and_lint(&fixture_dir("prop_barrel_radius_non_positive")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "radius = 0");
    let (_, _, errors) = load_and_lint(&fixture_dir("prop_table_no_bullets")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "bullets = 0");
}

#[test]
fn effect_fixtures_report_unsupported_and_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_unsupported")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "RefillAmmo");
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_name_empty")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Emit : nom vide");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "OnEvent : nom vide");
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
            "item_test_frames_zero",
            LintErrorKind::OutOfRange,
            &["objet « bottes »", "test.frames = 0"],
        ),
        (
            "item_test_consumable",
            LintErrorKind::Unsupported,
            &["objet « cle »", "ItemHeld", "consommable"],
        ),
        (
            "room_test_no_template",
            LintErrorKind::BrokenReference,
            &[
                "type de salle « arene »",
                "aucun gabarit LDtk de type « arene »",
            ],
        ),
        (
            "room_test_both_scenes",
            LintErrorKind::OutOfRange,
            &["type de salle « combat »", "exactement un de floors et map"],
        ),
        (
            "room_test_frames_zero",
            LintErrorKind::OutOfRange,
            &["type de salle « combat »", "test.frames = 0"],
        ),
        (
            "boss_fight_without_boss",
            LintErrorKind::OutOfRange,
            &["personnage « gardien »", "BossFight", "sans boss:"],
        ),
        (
            "champion_test_unknown_variant",
            LintErrorKind::BrokenReference,
            &[
                "personnage « grunt »",
                "test.champions",
                "« fantome » absente de `variants`",
            ],
        ),
        (
            "champion_test_duplicate_variant",
            LintErrorKind::DuplicateId,
            &[
                "personnage « grunt »",
                "test.champions",
                "« champion » répétée",
            ],
        ),
        (
            "champion_test_unknown_weapon",
            LintErrorKind::BrokenReference,
            &[
                "personnage « grunt »",
                "test.champions",
                "arme « fantome » inconnue",
            ],
        ),
        (
            "champion_test_kill_untranslatable",
            LintErrorKind::Unsupported,
            &[
                "personnage « grunt »",
                "kill: true sans effet de mort traduisible",
            ],
        ),
        (
            "champion_test_weapon_without_kill",
            LintErrorKind::OutOfRange,
            &["personnage « grunt »", "`weapon` sans `kill: true`"],
        ),
        (
            "tears_test_min_hits_zero",
            LintErrorKind::OutOfRange,
            &["objet « lentille »", "test.tears.hits", "with_item = 0"],
        ),
        (
            "tears_test_hits_not_above_control",
            LintErrorKind::OutOfRange,
            &[
                "objet « lentille »",
                "with_item (3) doit dépasser without_item (3)",
            ],
        ),
        (
            "tears_test_nothing_proven",
            LintErrorKind::OutOfRange,
            &["objet « lentille »", "ni `hits` ni `expect`"],
        ),
        (
            "tears_test_expect_without_control",
            LintErrorKind::OutOfRange,
            &["objet « lentille »", "`expect` et `control` vont ensemble"],
        ),
        (
            "tears_test_frames_zero",
            LintErrorKind::OutOfRange,
            &["objet « lentille »", "test.tears : frames = 0"],
        ),
        (
            "tears_test_hit_target_out_of_range",
            LintErrorKind::BrokenReference,
            &["objet « lentille »", "cible 2 absente"],
        ),
        (
            "tears_test_weapon_unknown",
            LintErrorKind::BrokenReference,
            &[
                "objet « lentille »",
                "test.tears : arme « fantome » inconnue",
            ],
        ),
        (
            "tears_test_target_unknown",
            LintErrorKind::BrokenReference,
            &["objet « lentille »", "« fantome » (targets[0]) inconnu"],
        ),
        (
            "tears_test_target_no_counts_hits",
            LintErrorKind::Unsupported,
            &[
                "objet « lentille »",
                "la cible « cible » ne compte pas ses coups",
            ],
        ),
        (
            "familiar_test_without_familiar",
            LintErrorKind::Unsupported,
            &["objet « bottes »", "test.familiar", "sans `familiar:`"],
        ),
        (
            "familiar_test_frames_zero",
            LintErrorKind::OutOfRange,
            &["objet « bottes »", "test.familiar : frames = 0"],
        ),
        (
            "room_kind_unknown",
            LintErrorKind::UnknownKind,
            &[
                "room_kind « boss » inconnu",
                "types de salle chargés : combat",
            ],
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
        (
            "floor_bounds_inverted",
            LintErrorKind::OutOfRange,
            &["minimum supérieur au maximum", "min_boss_distance = 6"],
        ),
        (
            "floor_required_without_template",
            LintErrorKind::BrokenReference,
            &["type de salle « boutique » sans gabarit"],
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
/// D48 : une caverne peuplée dont une graine de contrôle n'a aucun point d'apparition atteint par
/// le champ de flux de son gabarit est refusée ; une caverne ouverte passe. (L'automate ne produit
/// pas de caverne fermée au seul gabarit grand sur les graines de contrôle — balayage de tailles,
/// remplissages et seuils : aucune — ; l'exclusion propre à la navigation est couverte par les
/// tests de `world::nav` sur grilles construites et par le test des 1 000 graines de throne.)
#[test]
fn cave_spawns_unreachable_fixture() {
    let (registry, _, errors) = load_and_lint(&fixture_dir("cave_spawns_unreachable")).unwrap();
    assert!(registry.caves.values().all(|cave| cave.config.nav_large));
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "« fermee » : graine de contrôle",
    );
    assert_eq!(
        errors.len(),
        1,
        "seule « fermee » est refusée : {errors:#?}"
    );
}

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
        // M2-T11.
        ("pit_fall_damage_missing", LintErrorKind::Parse),
        ("pit_fall_damage_non_positive", LintErrorKind::OutOfRange),
        ("prop_barrel_radius_non_positive", LintErrorKind::OutOfRange),
        ("prop_table_no_bullets", LintErrorKind::OutOfRange),
        ("effect_unsupported", LintErrorKind::Unsupported),
        ("effect_name_empty", LintErrorKind::OutOfRange),
        ("effect_status_no_target", LintErrorKind::Unsupported),
        ("effect_status_unknown", LintErrorKind::BrokenReference),
        ("effect_status_stacks_zero", LintErrorKind::OutOfRange),
        ("effect_status_on_kill", LintErrorKind::Unsupported),
        (
            "effect_item_filter_wrong_trigger",
            LintErrorKind::Unsupported,
        ),
        ("effect_item_filter_unknown", LintErrorKind::BrokenReference),
        ("effect_per_copy_not_linear", LintErrorKind::Unsupported),
        ("effect_out_of_range", LintErrorKind::OutOfRange),
        ("effect_limits_invalid", LintErrorKind::OutOfRange),
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
        ("cave_spawns_unreachable", LintErrorKind::OutOfRange),
        ("entry_clock_unknown", LintErrorKind::BrokenReference),
        ("entry_difficulty_missing", LintErrorKind::BrokenReference),
        ("effect_broken_reference", LintErrorKind::BrokenReference),
        ("map_character_unknown", LintErrorKind::BrokenReference),
        ("room_kind_unknown", LintErrorKind::UnknownKind),
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

/// Assets : `volumes` règle une clé absente de `sounds` ; `audio_config` d'une arme vise un
/// mode de tir qu'elle n'a pas.
#[test]
fn sound_config_fixtures_report_key_and_mode() {
    let (_, _, errors) = load_and_lint(&fixture_dir("feedback_volume_unknown_key")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "volumes.hit : clé absente de sounds",
    );
    let (_, _, errors) = load_and_lint(&fixture_dir("audio_unknown_mode")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "audio_config.modes.rafale : mode de tir inconnu",
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
        // M2-T19
        ("feedback_blank_frames_zero", LintErrorKind::OutOfRange, 1),
        (
            "feedback_boss_zoom_out_of_range",
            LintErrorKind::OutOfRange,
            1,
        ),
        ("feedback_boss_freeze_zero", LintErrorKind::OutOfRange, 1),
        // M3-T17
        (
            "feedback_explosion_frames_zero",
            LintErrorKind::OutOfRange,
            1,
        ),
        (
            "feedback_explosion_amplitude_negative",
            LintErrorKind::OutOfRange,
            1,
        ),
        (
            "feedback_explosion_shake_frames_too_long",
            LintErrorKind::OutOfRange,
            1,
        ),
        (
            "feedback_explosion_color_out_of_range",
            LintErrorKind::OutOfRange,
            1,
        ),
        // T1.17
        (
            "feedback_by_weapon_unknown",
            LintErrorKind::BrokenReference,
            1,
        ),
        (
            "feedback_volume_unknown_key",
            LintErrorKind::BrokenReference,
            1,
        ),
        ("audio_unknown_mode", LintErrorKind::BrokenReference, 1),
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

/// T1.18 : source de HUD inconnue ;
/// une seule erreur par fixture en plus du start_map commun.
#[test]
fn hud_source_fixtures() {
    for (name, kind, needle) in [(
        "hud_unknown_source",
        LintErrorKind::UnknownKind,
        "hud : source inconnue « mana »",
    )] {
        let (_, _, errors) = load_and_lint(&fixture_dir(name)).unwrap();
        assert_has_error(&errors, kind, needle);
        let others: Vec<_> = errors
            .iter()
            .filter(|e| !e.message.contains("entry.start_map"))
            .collect();
        assert_eq!(others.len(), 1, "{name}: {errors:#?}");
    }
}

#[test]
fn html_screen_missing_row_fixture_reports_referenced_template() {
    let (registry, _, errors) = load_and_lint(&fixture_dir("html_screen_missing_row")).unwrap();
    assert!(registry.ui_files.contains(&"ui/pause_screen.html".into()));
    assert_has_error(&errors, LintErrorKind::BrokenReference, "ui/hui_row.html");
    let error = errors
        .iter()
        .find(|error| error.message.contains("ui/hui_row.html"))
        .unwrap();
    assert_eq!(error.file, "ui/pause_screen.html");
    assert_only_one_besides_start_map(&errors);
}

#[test]
fn html_templates_are_discovered_without_ron_companions() {
    let (registry, _, errors) = load_and_lint(&fixture_dir("html_screens_valid")).unwrap();
    assert!(
        errors
            .iter()
            .all(|error| error.message.contains("entry.start_map")),
        "{errors:#?}"
    );
    for file in [
        "ui/pause_screen.html",
        "ui/inventory_screen.html",
        "ui/hui_row.html",
        "ui/hud.ron",
        "other/menu.html",
    ] {
        assert!(registry.ui_files.contains(&file.into()), "{file}");
    }
    // Explicit single-file declarations must not be discovered twice.
    assert_eq!(registry.ui_files.len(), 5);
    assert_eq!(registry.hud_sources.len(), 1);
    assert_eq!(registry.hud_sources[0].1, ["health"]);
}

#[test]
fn referenced_html_must_be_readable_in_browser_inventory() {
    let (registry, _, errors) = content::load_embedded(&[
        (
            "game.ron",
            Some(
                r#"(
                    name: "html_unreadable",
                    content_folders: [(path: "ui", kind: "Ui")],
                    entry: (start_map: "unused", default_seed: 1),
                )"#,
            ),
        ),
        (
            "ui/inventory_screen.html",
            Some("<template><node id=\"weapons\"></node></template>"),
        ),
        // Inventoried, but unavailable as UTF-8 text.
        ("ui/hui_row.html", None),
    ])
    .unwrap();
    assert!(registry
        .ui_files
        .contains(&"ui/inventory_screen.html".into()));
    assert!(!registry.ui_files.contains(&"ui/hui_row.html".into()));
    assert!(
        errors
            .iter()
            .any(|error| { error.kind == LintErrorKind::Parse && error.file == "ui/hui_row.html" }),
        "{errors:#?}"
    );
    assert_has_error(&errors, LintErrorKind::BrokenReference, "ui/hui_row.html");
}

#[test]
fn explicitly_declared_missing_html_reports_structured_error() {
    let (_, _, errors) = content::load_embedded(&[(
        "game.ron",
        Some(
            r#"(
                name: "html_missing",
                content_folders: [(path: "ui/absent_screen.html", kind: "Ui")],
                entry: (start_map: "unused", default_seed: 1),
            )"#,
        ),
    )])
    .unwrap();
    assert!(
        errors.iter().any(|error| {
            error.kind == LintErrorKind::Parse && error.file == "ui/absent_screen.html"
        }),
        "{errors:#?}"
    );
    assert_only_one_besides_start_map(&errors);
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

/// D51 : `spread` doit rester dans [0, π] radians.
#[test]
fn weapon_spread_out_of_range_fixture_reports_spread() {
    let (_, _, errors) = load_and_lint(&fixture_dir("weapon_spread_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "spread");
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

// m2-lisibilite-gungeon : la légende du HUD ne cite que des sprites de `sprites.ron`.
#[test]
fn hud_legend_fixture_reports_unknown_sprite() {
    let (_, _, errors) = load_and_lint(&fixture_dir("hud_legend_unknown_sprite")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "sprite « dragon »");
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
fn pattern_spiral_on_expire_fixture_reports_multi_volley_pattern() {
    let (_, _, errors) = load_and_lint(&fixture_dir("pattern_spiral_on_expire")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Spiral");
}

#[test]
fn pattern_burst_interval_zero_fixture_reports_interval() {
    let (_, _, errors) = load_and_lint(&fixture_dir("pattern_burst_interval_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "interval = 0");
}

#[test]
fn item_blank_not_consumable_fixture_reports_unsupported() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_blank_not_consumable")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "blank");
}

#[test]
fn character_starting_consumable_unknown_fixture_reports_broken_reference() {
    let (_, _, errors) =
        load_and_lint(&fixture_dir("character_starting_consumable_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« fantome »");
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
fn player_skin_second_missing_fixture_reports_skin() {
    let (_, _, errors) = load_and_lint(&fixture_dir("player_skin_second_missing")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "skin « 2 »");
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

/// D48 (m1-d48-ennemis-hors-champ) : sur les trois cavernes de throne et 1 000 graines, chaque
/// point `ZombieSpawn` est atteint par le champ de flux du gabarit de **chaque** personnage de la
/// caverne (`world::nav::nav_distances` depuis les points des joueurs) : aucun ennemi ne naît
/// dans une poche hors de son champ (graine 43 : roi_rat, gabarit grand ; graine 162 : brute,
/// couloir d'une case). Le boss rend `niveau_3` grand (`nav_large`), pas les deux autres.
#[test]
fn points_ennemis_des_cavernes_de_throne_dans_le_champ_de_chaque_gabarit() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games/throne");
    let (registry, _, _) = load_and_lint(&dir).unwrap();
    for name in ["niveau_1", "niveau_2", "niveau_3"] {
        let cave = &registry.caves[&content::registry::CaveId::from(name.to_string())];
        let config = &cave.config;
        assert_eq!(config.nav_large, name == "niveau_3", "{name} : nav_large");
        let larges: Vec<bool> = config
            .characters
            .iter()
            .map(|id| {
                registry.characters[&content::registry::CharacterId::from(id.as_str())].body_large
            })
            .collect();
        for seed in 1..=1000u64 {
            let grid = world::generate(seed, config);
            let points = world::points_of_interest(
                &grid,
                4,
                config.enemy_spawns,
                config.spawn_clearance,
                config.nav_large,
            );
            assert!(
                !points.zombie_spawns.is_empty(),
                "{name} graine {seed} : aucun point"
            );
            for large in [false, true] {
                if !larges.contains(&large) {
                    continue;
                }
                let dist = world::nav::nav_distances(&grid, &points.player_spawns, large);
                for &(x, y) in &points.zombie_spawns {
                    assert_ne!(
                        dist[(y * grid.width + x) as usize],
                        u32::MAX,
                        "{name} graine {seed} : point ({x}, {y}) hors du champ (grand : {large})"
                    );
                }
            }
        }
    }
}

// M2-T0b (docs/conventions.md §36) : objets.
#[test]
fn item_charge_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_charge_zero")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "objet « fiole » : charge",
    );
}

#[test]
fn item_effect_unsupported_fixture_reports_unsupported() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_effect_unsupported")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "OnUse n'a de sens que pour un objet actif",
    );
}

/// M3-T2 (§67) : `ApplyStatus` d'un effet (hors cible, statut inconnu, `stacks = 0`) et noms vides.
#[test]
fn effect_status_fixtures_report_one_rule_each() {
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_status_no_target")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "pas de cible hors OnHit",
    );
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_status_unknown")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "statut « fantome » inconnu",
    );
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_status_stacks_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "stacks = 0");
}

/// M3-T2 (§67) : `ApplyStatus` sur `OnKill` (cible morte), filtre `item` de `OnPickup`, `PerCopy` non linéaire.
#[test]
fn effect_v3_contract_fixtures_report_one_rule_each() {
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_status_on_kill")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "OnKill : la cible est déjà morte",
    );
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_item_filter_wrong_trigger")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "filtre réservé à OnPickup",
    );
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_item_filter_unknown")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "objet « fantome » inconnu",
    );
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_per_copy_not_linear")).unwrap();
    for needle in ["Modifier", "Emit", "RefillAmmo"] {
        assert_has_error(&errors, LintErrorKind::Unsupported, needle);
    }
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.message.contains("Scaled(PerCopy)"))
            .count(),
        3
    );
    // témoin : les actions linéaires passent
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_per_copy_linear_ok")).unwrap();
    let others: Vec<_> = errors
        .iter()
        .filter(|e| !e.message.contains("entry.start_map") && !e.message.contains("start_map"))
        .collect();
    assert!(others.is_empty(), "{others:#?}");
}

/// M2-T3 : cooldown/charges nuls et recharge sans charges.
#[test]
fn effect_limits_invalid_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("effect_limits_invalid")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "cooldown = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "charges = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "recharge sans charges");
}

#[test]
fn item_unknown_stat_fixture_reports_parse_error() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_unknown_stat")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "Vitesse");
}

// M2-T0c (docs/conventions.md §37) : boss.
#[test]
fn boss_fixtures_report_their_rule() {
    for (fixture, kind, expected) in [
        (
            "boss_health_out_of_range",
            LintErrorKind::OutOfRange,
            "HealthBelow = 1.5",
        ),
        ("boss_phase_empty", LintErrorKind::OutOfRange, "phase vide"),
        (
            "boss_missing_until",
            LintErrorKind::OutOfRange,
            "until absent",
        ),
        (
            "boss_unknown_pattern",
            LintErrorKind::BrokenReference,
            "pattern « fantome » inconnu",
        ),
        (
            "boss_timeline_after_repeat",
            LintErrorKind::OutOfRange,
            ">= repeat",
        ),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert_has_error(&errors, kind, expected);
    }
}

#[test]
fn dash_charges_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("dash_charges_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "dash_charges = 0");
}

#[test]
fn dash_recharge_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("dash_recharge_zero")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "dash_recharge_frames = 0",
    );
}

#[test]
fn dash_duration_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("dash_duration_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "durée 0 hors InPlace");
}

#[test]
fn dash_iframes_over_duration_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("dash_iframes_over_duration")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "i-frames (12) > durée (8)",
    );
}

#[test]
fn synergy_unknown_item_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("synergy_unknown_item")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "objet « fantome » inconnu",
    );
}

#[test]
fn synergy_single_item_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("synergy_single_item")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "au moins deux objets");
}

#[test]
fn synergy_duplicate_set_fixture_reports_duplicate_id() {
    let (_, _, errors) = load_and_lint(&fixture_dir("synergy_duplicate_set")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::DuplicateId,
        "même ensemble d'objets",
    );
}

#[test]
fn item_uses_per_floor_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_uses_per_floor_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "uses_per_floor = 0");
}

#[test]
fn item_cost_not_consumable_fixture_reports_unsupported() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_cost_not_consumable")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "n'est pas un consommable",
    );
}

#[test]
fn loot_empty_table_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("loot_empty_table")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "entries vide");
}

#[test]
fn loot_weight_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("loot_weight_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "poids 0");
}

#[test]
fn loot_weight_over_cap_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("loot_weight_over_cap")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "poids 1001 (au plus 1000",
    );
}

#[test]
fn unlock_weight_over_cap_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_weight_over_cap")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "poids 1001 (au plus 1000",
    );
}

#[test]
fn loot_unknown_item_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("loot_unknown_item")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "fantome");
}

#[test]
fn loot_locked_chest_without_key_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("loot_locked_chest_without_key")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "« key »");
}

#[test]
fn formation_radius_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("formation_radius_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "radius = 0");
}

#[test]
fn formation_spread_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("formation_spread_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "doit être dans ]0, 2π]");
}

#[test]
fn formation_period_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("formation_period_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "period = 0");
}

#[test]
fn formation_inert_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("formation_inert")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "ni Shoot ni Chase");
}

#[test]
fn formation_no_squad_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("formation_no_squad")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "sans escouade");
}

// M2-T7 (docs/conventions.md §45) : arènes et actions de frise.
#[test]
fn arena_and_boss_t7_fixtures_report_their_rule() {
    for (fixture, kind, expected) in [
        ("arena_no_waves", LintErrorKind::OutOfRange, "aucune vague"),
        ("arena_clock_zero", LintErrorKind::OutOfRange, "clock = 0"),
        (
            "arena_unknown_enemy",
            LintErrorKind::BrokenReference,
            "personnage « fantome » inconnu",
        ),
        ("arena_count_zero", LintErrorKind::OutOfRange, "count = 0"),
        (
            "boss_spawn_enemies_unknown",
            LintErrorKind::BrokenReference,
            "personnage « fantome » inconnu",
        ),
        (
            "boss_spawn_enemies_count_zero",
            LintErrorKind::OutOfRange,
            "count = 0",
        ),
        (
            "boss_telegraph_alone",
            LintErrorKind::OutOfRange,
            "Telegraph sans SpawnPattern",
        ),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert_has_error(&errors, kind, expected);
    }
}

// M3-T0c (docs/conventions.md §62) : contrats du monde.
#[test]
fn world_contract_fixtures_report_their_rule() {
    for (fixture, kind, expected) in [
        (
            "explode_radius_zero",
            LintErrorKind::OutOfRange,
            "Explode : rayon = 0",
        ),
        ("door_lock_key_zero", LintErrorKind::OutOfRange, "Key(0)"),
        (
            "hazard_period_zero",
            LintErrorKind::OutOfRange,
            "period = 0",
        ),
        (
            "bomb_fuse_zero",
            LintErrorKind::OutOfRange,
            "fuse_frames = 0",
        ),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert_has_error(&errors, kind, expected);
    }
}

// M3-T5 (docs/conventions.md §70) : disposition en grille et trésor derrière une clé obtenable.
#[test]
fn grid_and_key_fixtures_report_their_rule() {
    for (fixture, kind, expected) in [
        (
            "grid_template_size_not_multiple",
            LintErrorKind::OutOfRange,
            "gabarit « Boss » (type « boss ») fait 6 × 3 cases : doit être un multiple",
        ),
        (
            "grid_template_missing_opening",
            LintErrorKind::OutOfRange,
            "gabarit « Boss » (type « boss », 10 × 3 cases, case de salle 5 × 3) sans ouverture centrée sur chaque segment de chaque côté (côté bas, segment 2/2 : aucune ouverture)",
        ),
        (
            "grid_post_boss_chance_out_of_range",
            LintErrorKind::OutOfRange,
            "post_boss.chance = 1.5",
        ),
        (
            "lock_from_floor_on_secret",
            LintErrorKind::OutOfRange,
            "lock_from_floor sur door_lock Secret",
        ),
        (
            "lock_from_floor_without_lock",
            LintErrorKind::OutOfRange,
            "lock_from_floor sans door_lock",
        ),
        (
            "key_room_without_key_source",
            LintErrorKind::Unsupported,
            "trésor fermé à clé sans source de clé",
        ),
        (
            "key_room_key_at_floor_zero_only",
            LintErrorKind::Unsupported,
            "trésor fermé à clé sans source de clé",
        ),
        (
            "key_room_floor_zero_locked_two_floors",
            LintErrorKind::Unsupported,
            "étage(s) d'indice 0 sans clé",
        ),
        (
            "grid_template_opening_widths_differ",
            LintErrorKind::OutOfRange,
            "gabarit « Boss » (type « boss ») a une ouverture de 5 case(s) mais gabarit « Depart » une de 3 case(s)",
        ),
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert_has_error(&errors, kind, expected);
        // une seule erreur : la règle visée, rien de collatéral (hors `entry.start_map`, commun à
        // la fixture sans carte)
        let others: Vec<_> = errors
            .iter()
            .filter(|e| !e.message.contains("entry.start_map"))
            .collect();
        assert_eq!(others.len(), 1, "{fixture} : {errors:#?}");
    }
}

/// Les grandes salles (1×1, 2×1, 1×2, 2×2) correctes et un trésor à clé dont la clé arrive à
/// l'étage du verrou ne produisent aucune erreur : la règle n'est pas durcie à tort.
#[test]
fn valid_grid_and_key_fixtures_have_no_lint_error() {
    for fixture in [
        "grid_large_rooms_ok",
        "key_room_with_key_source",
        "key_room_lock_never_applies",
    ] {
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert!(errors.is_empty(), "{fixture} : {errors:#?}");
    }
}

// D61 (docs/conventions.md §18) : bouclier et stats lues.
#[test]
fn shield_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shield_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "GainShield(0)");
}

// M2-T15 (docs/conventions.md §53) : stats sans lecteur et atteignabilité statique.
fn assert_has_warning(fixture: &str, kind: content::LintWarningKind, needle: &str) {
    let (registry, manifest, _) = load_and_lint(&fixture_dir(fixture)).unwrap();
    let warnings = content::warnings(&registry, &manifest);
    assert!(
        warnings
            .iter()
            .any(|w| w.kind == kind && w.message.contains(needle)),
        "attendu un avertissement {kind:?} contenant {needle:?}, obtenu :\n{warnings:#?}"
    );
}

#[test]
fn stat_unread_fixture_is_an_error() {
    let (_, _, errors) = load_and_lint(&fixture_dir("stat_unread")).unwrap();
    // M3-T0a : `Luck` est lue (chance d'effet) ; une stat `Custom` sans lecteur reste refusée.
    assert_has_error(&errors, LintErrorKind::UnreadStat, "superstition");
}

#[test]
fn reach_fixtures_report_their_warning() {
    use content::LintWarningKind::Unreachable;
    for (fixture, needle) in [
        ("reach_room_unreachable", "type de salle « escouade »"),
        ("reach_room_unreachable", "gabarit « Escouade »"),
        ("reach_enemy_unplaced", "ennemi « grunt »"),
        ("reach_item_no_source", "objet « bottes » : aucune source"),
        ("reach_weapon_no_source", "arme « b »"),
    ] {
        assert_has_warning(fixture, Unreachable, needle);
        // un avertissement n'est pas une erreur
        let (_, _, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        assert!(
            errors.iter().all(|e| !e.message.contains("inatteignable")),
            "{fixture} : {errors:#?}"
        );
    }
}

/// D63 : seules les règles des ennemis atteints donnent une source d'armes.
#[test]
fn reached_enemy_rules_are_weapon_sources() {
    let (registry, manifest, errors) = load_and_lint(&fixture_dir("reach_enemy_weapons")).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
    let warnings = content::warnings(&registry, &manifest);
    for weapon in ["shoot", "phase", "enter", "timeline"] {
        assert!(
            warnings
                .iter()
                .all(|w| !w.message.contains(&format!("arme « {weapon} »"))),
            "{warnings:#?}"
        );
    }
    for weapon in ["unused", "lab_weapon"] {
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains(&format!("arme « {weapon} »"))),
            "{warnings:#?}"
        );
    }
}

/// D63 : `test_only: true` retire l'avertissement d'un personnage et d'une arme ; les fixtures
/// jumelles sans marqueur (`reach_enemy_unplaced`, `reach_weapon_no_source`) le gardent.
#[test]
fn test_only_content_is_ignored_by_reach() {
    for (fixture, twin, needle) in [
        (
            "reach_test_only_character",
            "reach_enemy_unplaced",
            "ennemi « grunt »",
        ),
        (
            "reach_test_only_weapon",
            "reach_weapon_no_source",
            "arme « b »",
        ),
    ] {
        assert_has_warning(twin, content::LintWarningKind::Unreachable, needle);
        let (registry, manifest, errors) = load_and_lint(&fixture_dir(fixture)).unwrap();
        // `test_only` est un champ valide (les erreurs de la jumelle, ex. carte absente, restent).
        assert!(
            errors.iter().all(|e| !e.message.contains("test_only")),
            "{fixture} : {errors:#?}"
        );
        let warnings = content::warnings(&registry, &manifest);
        assert!(
            warnings.iter().all(|w| !w.message.contains(needle)),
            "{fixture} : {warnings:#?}"
        );
    }
}

#[test]
fn shipped_games_lint_without_error() {
    for game in ["zombies", "testbed", "throne", "gungeon", "isaac"] {
        let (registry, manifest, errors) = load_and_lint(&game_dir(game)).unwrap();
        assert!(errors.is_empty(), "{game} : {errors:#?}");
        for warning in content::warnings(&registry, &manifest) {
            eprintln!("avertissement {game} : {warning}");
        }
    }
}

#[test]
fn unlock_cost_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_cost_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "cost = 0");
}

#[test]
fn unlock_unknown_item_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_unknown_item")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "fantome");
}

#[test]
fn unlock_unknown_table_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_unknown_table")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "nulle");
}

#[test]
fn unlock_cycle_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_cycle")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "cycle de prérequis",
    );
}

#[test]
fn shop_price_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shop_price_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "un prix doit être > 0");
}

#[test]
fn shop_shells_inverted_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shop_shells_inverted")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "min > max");
}

#[test]
fn shop_without_table_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shop_without_table")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "boutique : pool « fantome » inconnu",
    );
}

#[test]
fn unlock_unknown_requires_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("unlock_unknown_requires")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "prérequis « zz »");
}

#[test]
fn shop_empty_table_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shop_empty_table")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "boutique : le pool « vide » est vide",
    );
}

#[test]
fn shop_no_shelves_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("shop_no_shelves")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "boutique sans présentoir",
    );
}

// D59 : `entry.floors` (séquence par défaut du mode `Floors`).
#[test]
fn entry_floors_fixtures_report_their_rule() {
    let (_, _, errors) = load_and_lint(&fixture_dir("entry_floors_unknown")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "entry.floors");
    let (_, _, errors) = load_and_lint(&fixture_dir("entry_floors_without_mode")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "exige entry.mode = Floors",
    );
}

// ---------------------------------------------------------------------------------------
// M3-T0a (docs/conventions.md §60) : munitions infinies, contrats de larmes, projectile d'objet
// ---------------------------------------------------------------------------------------

#[test]
fn weapon_infinite_with_ammo_type_fixture_is_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("weapon_infinite_with_ammo_type")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Infinite");
    assert_eq!(m4_t0a_errors(&errors), 1, "{errors:#?}");
}

#[test]
fn weapon_ammo_type_missing_fixture_is_refused() {
    let (_, _, errors) = load_and_lint(&fixture_dir("weapon_ammo_type_missing")).unwrap();
    assert_has_error(&errors, LintErrorKind::Parse, "ammo_type absent");
    assert_eq!(m4_t0a_errors(&errors), 1, "{errors:#?}");
}

/// M3-T1 (§66) : `Spectral`, `Split` et `Orbit` sont exécutés, une arme qui les porte passe.
#[test]
fn projectile_contract_fixture_is_clean() {
    let (_, _, errors) = load_and_lint(&fixture_dir("projectile_contract_ok")).unwrap();
    assert_eq!(m4_t0a_errors(&errors), 0, "{errors:#?}");
}

#[test]
fn projectile_split_out_of_range_fixture_reports_ranges() {
    let (_, _, errors) = load_and_lint(&fixture_dir("projectile_split_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "count = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "radius");
    assert_has_error(&errors, LintErrorKind::BrokenReference, "inconnu");
}

/// M3-T1 : un objet n'a pas de table `projectiles` : `Split` avec `child` et `on_expire: Spawn`
/// sont refusés.
#[test]
fn item_projectile_without_table_fixture_is_unsupported() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_projectile_no_table")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "Split : child « eclat »",
    );
    assert_has_error(&errors, LintErrorKind::Unsupported, "on_expire Spawn");
    assert_eq!(m4_t0a_errors(&errors), 2, "{errors:#?}");
}

#[test]
fn item_projectile_ok_fixture_is_clean() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_projectile_ok")).unwrap();
    assert_eq!(m4_t0a_errors(&errors), 0, "{errors:#?}");
}

#[test]
fn item_projectile_out_of_range_fixture_reports_ranges() {
    let (_, _, errors) = load_and_lint(&fixture_dir("item_projectile_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Homing");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Bounce répété");
}

/// M3-T1 : `Action::Chance` : base dans [0, 1], enveloppe une action à modificateur ou
/// `ApplyStatus` (statut vérifié à travers elle), jamais une autre `Chance` ni `DestroyTerrain` ;
/// `Orbit` refuse `Homing` et `Gravity` dans la même liste.
#[test]
fn projectile_chance_out_of_range_fixture_reports_each_rule() {
    let (_, _, errors) = load_and_lint(&fixture_dir("projectile_chance_out_of_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Chance : base = 1.5");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Chance : base = -0.1");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Chance : then Chance");
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "Chance : then DestroyTerrain",
    );
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "ApplyStatus : statut « inconnu » inconnu",
    );
    assert_has_error(&errors, LintErrorKind::Unsupported, "Orbit avec Homing");
}

#[test]
fn projectile_split_cycle_fixture_reports_cycle() {
    let (_, _, errors) = load_and_lint(&fixture_dir("projectile_split_cycle")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "Split se divise en lui-même",
    );
}

#[test]
fn status_poison_fear_fixture_reports_ranges() {
    let (_, _, errors) = load_and_lint(&fixture_dir("status_poison_fear_out_of_range")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "damage = 0 : doit être > 0 (Poison)",
    );
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "period = 0 : doit être > 0 (Poison)",
    );
    assert_has_error(&errors, LintErrorKind::OutOfRange, "frames = 0");
}

/// Erreurs d'une fixture M3-T0a hors la carte de départ absente (`start_map: "unused"`, commune
/// aux fixtures sans carte).
fn m4_t0a_errors(errors: &[content::LintError]) -> usize {
    errors
        .iter()
        .filter(|e| !e.message.contains("entry.start_map"))
        .count()
}

/// Les fixtures n'ont pas de carte : la seule erreur tolérée d'un contenu valide est celle de
/// `entry.start_map`.
fn only_missing_start_map(errors: &[content::LintError]) -> bool {
    errors
        .iter()
        .all(|error| error.message.contains("entry.start_map"))
}

/// M3-T0d (§63) : un familier valide (porteur, behaviors de familier) passe le lint.
#[test]
fn familiar_ok_fixture_is_clean() {
    let (_, _, errors) = load_and_lint(&fixture_dir("familiar_ok")).unwrap();
    assert!(only_missing_start_map(&errors), "{errors:?}");
}

#[test]
fn familiar_unknown_character_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("familiar_unknown_character")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "personnage « fantome » inconnu",
    );
}

#[test]
fn familiar_orphan_behavior_fixture_reports_missing_carrier() {
    let (_, _, errors) = load_and_lint(&fixture_dir("familiar_orphan_behavior")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "sans porteur");
}

#[test]
fn familiar_without_behavior_fixture_reports_unsupported() {
    let (_, _, errors) = load_and_lint(&fixture_dir("familiar_without_behavior")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "sans behavior de familier",
    );
}

#[test]
fn familiar_gap_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("familiar_gap_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Follow : gap = 0");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Orbit : radius = 0");
    assert_has_error(
        &errors,
        LintErrorKind::OutOfRange,
        "Orbit : frames_per_turn = 0",
    );
}

#[test]
fn champion_bad_death_fixture_reports_each_field() {
    let (_, _, errors) = load_and_lint(&fixture_dir("champion_bad_death")).unwrap();
    assert_has_error(&errors, LintErrorKind::Unsupported, "refusée à la mort");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "Explode : rayon = 0");
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "SpawnCharacters : personnage « fantome » inconnu",
    );
    assert_has_error(&errors, LintErrorKind::OutOfRange, "count = 0");
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "pool « fantome » inconnu",
    );
    assert_has_error(&errors, LintErrorKind::OutOfRange, "tint « rouge »");
    assert_has_error(&errors, LintErrorKind::OutOfRange, "behaviors vide");
}

#[test]
fn champion_ok_fixture_is_clean() {
    let (_, _, errors) = load_and_lint(&fixture_dir("champion_ok")).unwrap();
    assert!(only_missing_start_map(&errors), "{errors:?}");
}

#[test]
fn variant_chance_expression_range_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_chance_expression_range")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "doit être dans [0, 1]");
}

#[test]
fn variant_chance_expression_unknown_fixture_reports_identifier() {
    let (_, _, errors) = load_and_lint(&fixture_dir("variant_chance_expression_unknown")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "identifiant « minutes » inconnu",
    );
}

#[test]
fn hub_unknown_unlock_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("hub_unknown_unlock")).unwrap();
    assert_has_error(&errors, LintErrorKind::BrokenReference, "fantome");
}

#[test]
fn hub_stat_zero_fixture_reports_out_of_range() {
    let (_, _, errors) = load_and_lint(&fixture_dir("hub_stat_zero")).unwrap();
    assert_has_error(&errors, LintErrorKind::OutOfRange, "toujours vraie");
}

#[test]
fn hub_without_door_fixture_reports_broken_reference() {
    let (_, _, errors) = load_and_lint(&fixture_dir("hub_without_door")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::BrokenReference,
        "aucun service d'action EnterRun",
    );
}

/// M3-T7 (§72) : un `SpawnPattern` à la mort d'un champion ne peut pas être infini (`Ring` avec
/// `every > 0`) ; un pattern à `every: 0` passe.
#[test]
fn champion_death_infinite_pattern_fixture_refuses_only_the_infinite_one() {
    let (_, _, errors) = load_and_lint(&fixture_dir("champion_death_infinite_pattern")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "pattern « couronne » infini",
    );
    assert!(
        !errors.iter().any(|e| e.message.contains("« unique »")),
        "le pattern à every: 0 est accepté : {errors:#?}"
    );
}

/// M3-T7 : le pattern infini est aussi refusé quand il est atteint par `Named` (deux niveaux), une
/// chaîne cyclique est refusée, une chaîne qui aboutit à `every: 0` passe.
#[test]
fn champion_death_infinite_pattern_named_fixture_follows_the_chain() {
    let (_, _, errors) =
        load_and_lint(&fixture_dir("champion_death_infinite_pattern_named")).unwrap();
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "pattern « alias2 » infini",
    );
    assert_has_error(
        &errors,
        LintErrorKind::Unsupported,
        "pattern « boucle_a » : chaîne `Named` cyclique",
    );
    assert!(
        !errors
            .iter()
            .any(|e| e.message.contains("« alias_unique »")),
        "chaîne vers every: 0 acceptée : {errors:#?}"
    );
}
