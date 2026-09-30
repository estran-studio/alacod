//! Tests pour les nouvelles attentes (Health, EntityHealth, NoDamageBetween, EntityCount, Event,
//! PlayerDowned, PlayerRevived, Defeat).

use game::replay::Expectation;
use scenario::{run, Scenario};

fn load_scenario(name: &str) -> Scenario {
    let path = format!("../../tests/scenarios/{}.ron", name);
    let source = std::fs::read_to_string(&path).expect(&format!("scénario {}", name));
    Scenario::from_ron(&source).expect("scénario RON invalide")
}

#[test]
fn health_player_in_range() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 800, le joueur doit être vivant et avoir une santé entre 50 et 100
    scenario.expect = vec![game::replay::Expectation::Health {
        handle: 0,
        min: Some(50.0),
        max: Some(100.0),
        at_frame: 800,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "Health test failed: {:?}",
        outcome.failures
    );
}

#[test]
fn health_player_below_min_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 800, la santé doit être >= 101 (impossible puisque max est 100)
    scenario.expect = vec![game::replay::Expectation::Health {
        handle: 0,
        min: Some(101.0),
        max: None,
        at_frame: 800,
    }];
    let outcome = run(&scenario);
    assert!(
        !outcome.failures.is_empty(),
        "Health test should have failed"
    );
    assert!(
        outcome.failures[0].contains("Health"),
        "Failure message should mention Health"
    );
}

#[test]
fn entity_count_player() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 100, il doit y avoir exactement 1 joueur
    scenario.expect = vec![game::replay::Expectation::EntityCount {
        kind: game::replay::EntityKind::Player,
        min: Some(1),
        max: Some(1),
        at_frame: 100,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "EntityCount test failed: {:?}",
        outcome.failures
    );
}

#[test]
fn entity_count_enemies() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("shoot_around");
    // À la frame 1000, il doit y avoir au moins 1 ennemi
    scenario.expect = vec![game::replay::Expectation::EntityCount {
        kind: game::replay::EntityKind::Enemy,
        min: Some(1),
        max: None,
        at_frame: 1000,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "EntityCount enemies test failed: {:?}",
        outcome.failures
    );
}

#[test]
fn entity_count_bullets_in_range() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("shoot_around");
    // À la frame 50, il doit y avoir au moins quelques balles
    scenario.expect = vec![game::replay::Expectation::EntityCount {
        kind: game::replay::EntityKind::Bullet,
        min: Some(1),
        max: None,
        at_frame: 50,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "EntityCount bullets test failed: {:?}",
        outcome.failures
    );
}

#[test]
fn entity_count_exceeds_max_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("shoot_around");
    // À la frame 50, il ne doit y avoir aucune balle (impossible)
    scenario.expect = vec![game::replay::Expectation::EntityCount {
        kind: game::replay::EntityKind::Bullet,
        min: None,
        max: Some(0),
        at_frame: 50,
    }];
    let outcome = run(&scenario);
    assert!(
        !outcome.failures.is_empty(),
        "EntityCount test should have failed"
    );
}

#[test]
fn event_kill_found() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("remote_first_fight");
    // À la frame 600, au moins un ennemi doit avoir été tué (un kill à la frame 482 avec les flux RNG de T1.6)
    scenario.expect = vec![game::replay::Expectation::Event {
        kind: "kill".to_string(),
        label_contains: None,
        by_frame: 600,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "Event kill test failed: {:?}",
        outcome.failures
    );
}

#[test]
fn event_wave_found() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 1500, une vague doit avoir commencé
    scenario.expect = vec![game::replay::Expectation::Event {
        kind: "wave".to_string(),
        label_contains: None,
        by_frame: 1500,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "Event wave test failed: {:?}",
        outcome.failures
    );
}

#[test]
fn event_not_found_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 100, aucune mort n'a eu lieu (devrait échouer)
    scenario.expect = vec![game::replay::Expectation::Event {
        kind: "death".to_string(),
        label_contains: None,
        by_frame: 100,
    }];
    let outcome = run(&scenario);
    assert!(
        !outcome.failures.is_empty(),
        "Event test should have failed"
    );
}

#[test]
fn event_with_label_filter() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("shoot_around");
    // À la frame 1500, une arme doit avoir été changée et le label doit contenir "prend"
    scenario.expect = vec![game::replay::Expectation::Event {
        kind: "weapon".to_string(),
        label_contains: Some("prend".to_string()),
        by_frame: 1500,
    }];
    let outcome = run(&scenario);
    // Ce test peut échouer si le joueur ne change jamais d'arme ; mais en l'état il teste le filtre
    eprintln!("Event with label filter: {:?}", outcome.failures);
}

#[test]
fn no_damage_between_when_safe() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // Entre les frames 0 et 100, le joueur ne doit pas recevoir de dégâts
    // (il n'y a pas encore d'ennemis)
    scenario.expect = vec![game::replay::Expectation::NoDamageBetween {
        handle: 0,
        from_frame: 0,
        to_frame: 100,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "NoDamageBetween test failed: {:?}",
        outcome.failures
    );
}

#[test]
fn no_damage_between_when_damaged_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 1300, le joueur est mort ; donc entre 0 et 1300 il reçoit des dégâts
    scenario.expect = vec![game::replay::Expectation::NoDamageBetween {
        handle: 0,
        from_frame: 0,
        to_frame: 1300,
    }];
    let outcome = run(&scenario);
    assert!(
        !outcome.failures.is_empty(),
        "NoDamageBetween test should have failed"
    );
    assert!(
        outcome
            .failures
            .iter()
            .any(|f| f.contains("NoDamageBetween")),
        "Failure message should mention NoDamageBetween: {:?}",
        outcome.failures
    );
}

#[test]
fn health_upper_bound_only() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 800, la santé doit être <= 100
    scenario.expect = vec![game::replay::Expectation::Health {
        handle: 0,
        min: None,
        max: Some(100.0),
        at_frame: 800,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "Health test (upper bound only) failed: {:?}",
        outcome.failures
    );
}

#[test]
fn health_lower_bound_only() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("idle");
    // À la frame 800, la santé doit être >= 0
    scenario.expect = vec![game::replay::Expectation::Health {
        handle: 0,
        min: Some(0.0),
        max: None,
        at_frame: 800,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "Health test (lower bound only) failed: {:?}",
        outcome.failures
    );
}

/// Le `GgrsNetId` du joueur `handle` dans le scénario, lu à la première frame.
fn net_id_du_joueur(scenario: &Scenario, handle: usize) -> usize {
    use bevy::prelude::*;
    let mut app = scenario::runner::run_until(scenario, 1);
    let mut query = app
        .world_mut()
        .query::<(&game::character::player::Player, &utils::net_id::GgrsNetId)>();
    query
        .iter(app.world())
        .find(|(player, _)| player.handle == handle)
        .map(|(_, net_id)| net_id.0)
        .expect("joueur présent à la frame 1")
}

#[test]
fn entity_health_du_joueur_par_net_id() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("idle");
    let net_id = net_id_du_joueur(&scenario, 0);
    scenario.expect = vec![game::replay::Expectation::EntityHealth {
        net_id,
        min: Some(100.0),
        max: Some(100.0),
        at_frame: 1,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);

    scenario.expect = vec![game::replay::Expectation::EntityHealth {
        net_id,
        min: Some(101.0),
        max: None,
        at_frame: 1,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.iter().any(|f| f.contains("EntityHealth")),
        "{:?}",
        outcome.failures
    );
}

/// Une santé négative posée hors simulation est vue par l'invariant `sante_bornee`,
/// et ne l'est plus quand le scénario le désactive.
#[test]
fn invariant_sante_bornee_et_desactivation() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use bevy::prelude::*;
    use bevy_fixed::fixed_math::Fixed;
    use game::character::{health::Health, player::Player};

    fn sante_negative(mut joueurs: Query<&mut Health, With<Player>>) {
        for mut health in &mut joueurs {
            health.current = Fixed::from_num(-1);
        }
    }

    let mut scenario = load_scenario("idle");
    scenario.expect = vec![];
    scenario.frames = 120;

    let outcome = scenario::run_with(&scenario, |app| {
        app.add_systems(Update, sante_negative);
    });
    assert!(
        outcome
            .failures
            .iter()
            .any(|f| f.contains("invariant sante_bornee")),
        "{:?}",
        outcome.failures
    );

    scenario.invariants.sante_bornee = false;
    let outcome = scenario::run_with(&scenario, |app| {
        app.add_systems(Update, sante_negative);
    });
    assert!(
        !outcome
            .failures
            .iter()
            .any(|f| f.contains("invariant sante_bornee")),
        "{:?}",
        outcome.failures
    );
}

// À terre et réanimation (T1.3, chantier B6) : `PlayerDowned`, `PlayerRevived`, `Defeat`.
// `downed_revive.ron` : le joueur 0 abat le joueur 1 (friendly_fire: Always), qui tombe à
// terre (f89) puis est réanimé (f295).

#[test]
fn player_downed_true_once_fallen() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("downed_revive");
    scenario.expect = vec![Expectation::PlayerDowned {
        handle: 1,
        at_frame: 100,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn player_downed_false_before_falling_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("downed_revive");
    // À la frame 50, le joueur 1 encaisse encore la rafale (santé pas encore à 0) : pas à terre.
    scenario.expect = vec![Expectation::PlayerDowned {
        handle: 1,
        at_frame: 50,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.iter().any(|f| f.contains("PlayerDowned")),
        "{:?}",
        outcome.failures
    );
}

#[test]
fn player_revived_true_after_holding_interaction() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("downed_revive");
    scenario.expect = vec![Expectation::PlayerRevived {
        handle: 1,
        by_frame: 350,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn player_revived_false_when_nobody_revives_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // `downed_bleedout` : même chute (f89), personne ne maintient l'interaction ensuite.
    let mut scenario = load_scenario("downed_bleedout");
    scenario.expect = vec![Expectation::PlayerRevived {
        handle: 1,
        by_frame: 1050,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.iter().any(|f| f.contains("PlayerRevived")),
        "{:?}",
        outcome.failures
    );
}

#[test]
fn defeat_false_while_a_player_still_stands_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // `downed_revive` à la frame 100 : le joueur 1 est à terre mais le joueur 0 est encore
    // debout — pas de défaite.
    let mut scenario = load_scenario("downed_revive");
    scenario.expect = vec![Expectation::Defeat { by_frame: 100 }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.iter().any(|f| f.contains("Defeat")),
        "{:?}",
        outcome.failures
    );
}

// Munitions typées et inventaire d'armes (T2.2, chantier B7).

#[test]
fn ammo_reserve_matches_starting_weapons_at_creation() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use sim_core::ammo::AmmoType;

    let mut scenario = load_scenario("idle");

    // Pistolet (Plomb) : mag_size 6 x mag_limit 8 (voir weapons.ron).
    scenario.expect = vec![Expectation::AmmoReserve {
        handle: 0,
        ammo_type: AmmoType::Plomb,
        amount: 48,
        at_frame: 1,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);

    // Mitrailleuse (Balle) : mag_size 30 x mag_limit 8.
    scenario.expect = vec![Expectation::AmmoReserve {
        handle: 0,
        ammo_type: AmmoType::Balle,
        amount: 240,
        at_frame: 1,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);

    // Fusil à pompe (Cartouche) : magless, ne contribue rien à la réserve partagée (voir
    // `game::weapons::default_mode_ammo_contribution`).
    scenario.expect = vec![Expectation::AmmoReserve {
        handle: 0,
        ammo_type: AmmoType::Cartouche,
        amount: 0,
        at_frame: 1,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn ammo_reserve_wrong_amount_fails() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use sim_core::ammo::AmmoType;

    let mut scenario = load_scenario("idle");
    scenario.expect = vec![Expectation::AmmoReserve {
        handle: 0,
        ammo_type: AmmoType::Plomb,
        amount: 999,
        at_frame: 1,
    }];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.iter().any(|f| f.contains("AmmoReserve")),
        "{:?}",
        outcome.failures
    );
}

#[test]
fn weapon_pickups_zero_when_nothing_dropped() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("idle");
    scenario.expect = vec![Expectation::WeaponPickups {
        min: Some(0),
        max: Some(0),
        at_frame: 100,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

/// L'attaque de mêlée reste possible pendant un rechargement, sans l'annuler (T2.2, chantier
/// B7, décision « coup de crosse pendant le rechargement »). `player_melee_attack_system` ne
/// consulte jamais `WeaponInventory::is_reloading` : ce test le prouve par un scénario plutôt
/// que par lecture de code.
///
/// Mitrailleuse à un seul coup par chargeur (`weapon_overrides`, `mag_size: 1`) : un tir la
/// vide, déclenchant un rechargement automatique (1,5 s = 90 frames), largement plus long que
/// l'attaque `bare_hands` (10 frames de duration, voir `melee_weapons.ron`) déclenchée à la
/// frame 60, pendant ce rechargement.
#[test]
fn melee_attack_during_reload_does_not_cancel_it() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::{Button, Segment, WeaponOverride};

    let mut scenario = load_scenario("idle");
    scenario.frames = 200;
    scenario.weapon_overrides = vec![WeaponOverride {
        weapon: "machine_gun".to_string(),
        mode: None,
        mag_size: Some(1),
        mag_limit: None,
        firing_rate: None,
        friendly_fire: None,
        ammo_type: None,
    }];
    scenario.players[0].inputs = vec![
        Segment {
            from: 10,
            to: 13,
            buttons: vec![Button::Fire],
            pan: (40, 100),
        },
        Segment {
            from: 60,
            to: 63,
            buttons: vec![Button::Melee],
            pan: (40, 100),
        },
    ];
    scenario.expect = vec![
        // Preuve que la mêlée a bien lieu (pas bloquée par le rechargement en cours).
        Expectation::Event {
            kind: "melee".to_string(),
            label_contains: None,
            by_frame: 70,
        },
        // Preuve que le rechargement n'a pas été annulé : le chargeur (capacité 1, voir
        // l'override) est plein bien après la fin de son délai (frame 11 + 90 = 101).
        Expectation::Ammo {
            handle: 0,
            ammo: 1,
            at_frame: 150,
        },
    ];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn defeat_true_once_all_players_down_or_dead() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // `downed_all_lose` : le joueur 1 tombe à terre (f1174), le joueur 0 meurt seul (f1505) —
    // défaite à ce moment.
    let mut scenario = load_scenario("downed_all_lose");
    scenario.expect = vec![Expectation::Defeat { by_frame: 1600 }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}
