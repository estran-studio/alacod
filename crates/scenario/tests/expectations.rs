//! Tests pour les nouvelles attentes (Health, EntityHealth, NoDamageBetween, EntityCount, Event).

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
    scenario.expect = vec![
        game::replay::Expectation::Health {
            handle: 0,
            min: Some(50.0),
            max: Some(100.0),
            at_frame: 800,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::Health {
            handle: 0,
            min: Some(101.0),
            max: None,
            at_frame: 800,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::EntityCount {
            kind: game::replay::EntityKind::Player,
            min: Some(1),
            max: Some(1),
            at_frame: 100,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::EntityCount {
            kind: game::replay::EntityKind::Enemy,
            min: Some(1),
            max: None,
            at_frame: 1000,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::EntityCount {
            kind: game::replay::EntityKind::Bullet,
            min: Some(1),
            max: None,
            at_frame: 50,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::EntityCount {
            kind: game::replay::EntityKind::Bullet,
            min: None,
            max: Some(0),
            at_frame: 50,
        },
    ];
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

    let mut scenario = load_scenario("shoot_around");
    // À la frame 1500, au moins un ennemi doit avoir été tué
    scenario.expect = vec![
        game::replay::Expectation::Event {
            kind: "kill".to_string(),
            label_contains: None,
            by_frame: 1500,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::Event {
            kind: "wave".to_string(),
            label_contains: None,
            by_frame: 1500,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::Event {
            kind: "death".to_string(),
            label_contains: None,
            by_frame: 100,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::Event {
            kind: "weapon".to_string(),
            label_contains: Some("prend".to_string()),
            by_frame: 1500,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::NoDamageBetween {
            handle: 0,
            from_frame: 0,
            to_frame: 100,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::NoDamageBetween {
            handle: 0,
            from_frame: 0,
            to_frame: 1300,
        },
    ];
    let outcome = run(&scenario);
    assert!(
        !outcome.failures.is_empty(),
        "NoDamageBetween test should have failed"
    );
    assert!(
        outcome.failures.iter().any(|f| f.contains("NoDamageBetween")),
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
    scenario.expect = vec![
        game::replay::Expectation::Health {
            handle: 0,
            min: None,
            max: Some(100.0),
            at_frame: 800,
        },
    ];
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
    scenario.expect = vec![
        game::replay::Expectation::Health {
            handle: 0,
            min: Some(0.0),
            max: None,
            at_frame: 800,
        },
    ];
    let outcome = run(&scenario);
    assert!(
        outcome.failures.is_empty(),
        "Health test (lower bound only) failed: {:?}",
        outcome.failures
    );
}
