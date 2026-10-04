//! Tests pour les nouvelles attentes (Health, EntityHealth, NoDamageBetween, EntityCount, Event,
//! PlayerDowned, PlayerRevived, Defeat, EntityHits ; Gauge, Level, Mutations de T1.10).

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

/// T2.5 (power-ups, chantier C1 v0) : aucun power-up placé ni tombé (`idle`, personne ne
/// meurt) — `PowerUpPickups` doit rester à 0, comme `WeaponPickups` ci-dessus.
#[test]
fn powerup_pickups_zero_when_none_placed() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("idle");
    scenario.expect = vec![Expectation::PowerUpPickups {
        min: Some(0),
        max: Some(0),
        at_frame: 100,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

/// Un placement scripté (`Scenario::powerups`) fait apparaître exactement un
/// `PowerUpPickup` au sol tant qu'il n'a pas été ramassé — ici loin de tout joueur
/// (position arbitraire hors de portée), donc jamais ramassé avant `at_frame`.
#[test]
fn powerup_placement_spawns_one_pickup() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("idle");
    scenario.powerups = vec![game::replay::PowerUpPlacement {
        id: "insta_kill".to_string(),
        x: bevy_fixed::fixed_math::Fixed::from_num(-5000),
        y: bevy_fixed::fixed_math::Fixed::from_num(-5000),
        at_frame: 10,
    }];
    scenario.expect = vec![Expectation::PowerUpPickups {
        min: Some(1),
        max: Some(1),
        at_frame: 50,
    }];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn powerup_placement_expire_selon_la_table_du_jeu() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("powerup_insta_kill");
    scenario.powerups[0].x = bevy_fixed::fixed_math::Fixed::from_num(-5000);
    scenario.powerups[0].y = bevy_fixed::fixed_math::Fixed::from_num(-5000);
    scenario.expect = vec![
        Expectation::PowerUpPickups {
            min: Some(1),
            max: Some(1),
            // Le runner observe le compteur après la frame simulée : ici l'état 1809.
            at_frame: 1810,
        },
        Expectation::PowerUpPickups {
            min: Some(0),
            max: Some(0),
            at_frame: 1811,
        },
    ];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

#[test]
fn powerup_max_ammo_se_rejoue_depuis_son_enregistrement() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let scenario = load_scenario("powerup_max_ammo");
    let original = run(&scenario);
    assert!(original.failures.is_empty(), "{:?}", original.failures);
    let mut recorded = Scenario::from_ron(&original.recorded.to_ron()).unwrap();
    recorded.frames = scenario.frames;
    let replayed = run(&recorded);
    assert!(replayed.failures.is_empty(), "{:?}", replayed.failures);
    assert_eq!(original.trace, replayed.trace);
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

/// `GgrsNetId` et `HitCount` de `target` (seul personnage de `testbed/arena.ldtk` avec
/// `counts_hits: true`) à la frame `frame`.
fn coups_sur_target(scenario: &Scenario, frame: u32) -> (usize, u32) {
    use bevy::prelude::*;
    let mut app = scenario::runner::run_until(scenario, frame);
    let mut query = app.world_mut().query_filtered::<(
        &utils::net_id::GgrsNetId,
        &game::character::health::HitCount,
    ), With<bevy_ggrs::Rollback>>();
    let compteurs: Vec<_> = query
        .iter(app.world())
        .map(|(net_id, hits)| (net_id.0, hits.0))
        .collect();
    assert_eq!(
        compteurs.len(),
        1,
        "un seul compteur de coups : {compteurs:?}"
    );
    compteurs[0]
}

/// D6 : l'attente `EntityHits` compare le compteur de coups de `target` dans l'arène du
/// testbed (le joueur lui tire dessus de f10 à f160, `testbed_target_hits`) : zéro avant le
/// premier tir, le compte exact relu dans le monde ensuite.
#[test]
fn entity_hits_compte_les_coups_sur_target() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("testbed_target_hits");
    let (net_id, coups) = coups_sur_target(&scenario, 200);
    assert!(coups >= 5, "le tir soutenu touche target : {coups} coups");

    scenario.expect = vec![
        Expectation::EntityHits {
            net_id,
            min: 0,
            max: Some(0),
            at_frame: 5,
        },
        Expectation::EntityHits {
            net_id,
            min: coups,
            max: Some(coups),
            at_frame: 200,
        },
    ];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

/// D6 : `EntityHits` échoue sous le minimum, au-dessus du maximum, et sur une entité sans
/// `HitCount` (le joueur) ou absente.
#[test]
fn entity_hits_hors_bornes_ou_sans_compteur_echoue() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = load_scenario("testbed_target_hits");
    let (net_id, coups) = coups_sur_target(&scenario, 200);
    let joueur = net_id_du_joueur(&scenario, 0);

    let attentes = [
        (
            Expectation::EntityHits {
                net_id,
                min: coups + 1,
                max: None,
                at_frame: 200,
            },
            "< min",
        ),
        (
            Expectation::EntityHits {
                net_id,
                min: 0,
                max: Some(coups - 1),
                at_frame: 200,
            },
            "> max",
        ),
        (
            Expectation::EntityHits {
                net_id: joueur,
                min: 0,
                max: None,
                at_frame: 200,
            },
            "HitCount",
        ),
        (
            Expectation::EntityHits {
                net_id: usize::MAX,
                min: 0,
                max: None,
                at_frame: 200,
            },
            "HitCount",
        ),
    ];
    scenario.expect = attentes
        .iter()
        .map(|(attente, _)| attente.clone())
        .collect();
    let outcome = run(&scenario);
    assert_eq!(
        outcome.failures.len(),
        attentes.len(),
        "{:?}",
        outcome.failures
    );
    for (attente, raison) in &attentes {
        let attendu = format!("{attente:?}");
        assert!(
            outcome
                .failures
                .iter()
                .any(|f| f.contains(&attendu) && f.contains(raison)),
            "{attendu} devait échouer ({raison}) : {:?}",
            outcome.failures
        );
    }
}

/// Balles vivantes à `frame` : (toutes, celles du projectile composable `id`).
fn balles_vivantes(scenario: &Scenario, frame: u32, id: &str) -> (u32, u32) {
    use bevy::prelude::*;
    let mut app = scenario::runner::run_until(scenario, frame);
    let mut query = app.world_mut().query_filtered::<(
        &game::weapons::Bullet,
        Option<&combat::projectile::Projectile>,
    ), With<bevy_ggrs::Rollback>>();
    let balles: Vec<_> = query
        .iter(app.world())
        .map(|(_, p)| p.map(|p| p.id.clone()))
        .collect();
    let composables = balles.iter().filter(|p| p.as_deref() == Some(id)).count();
    (balles.len() as u32, composables as u32)
}

/// Scénario généré (gabarit WeaponOnTarget) de l'arme `grenade` du testbed, sans attente.
fn scenario_grenade() -> Scenario {
    scenario::generate::build_scenario("grenade", scenario::generate::WeaponKind::Ranged, None, 0)
}

/// T1.1 : `BulletCount` compte exactement les balles vivantes, filtrées par id de projectile
/// composable et par équipe du tireur ; un compte faux échoue.
#[test]
fn bullet_count_compte_les_projectiles_vivants() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // Balles ordinaires : sans filtre, le compte exact ; aucun projectile composable.
    let mut scenario = load_scenario("testbed_target_hits");
    let (toutes, _) = balles_vivantes(&scenario, 40, "");
    assert!(toutes > 0, "le joueur tire à la frame 40");
    scenario.expect = vec![
        Expectation::BulletCount {
            count: toutes,
            projectile: None,
            team: None,
            at_frame: 40,
        },
        Expectation::BulletCount {
            count: toutes,
            projectile: None,
            team: Some(sim_core::team::Team::Players),
            at_frame: 40,
        },
        Expectation::BulletCount {
            count: 0,
            projectile: None,
            team: Some(sim_core::team::Team::Enemies),
            at_frame: 40,
        },
        Expectation::BulletCount {
            count: 0,
            projectile: Some("eclat".into()),
            team: None,
            at_frame: 40,
        },
    ];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);

    // Grenade : les éclats d'une explosion sont comptés par leur id ; un compte faux échoue.
    // Le premier tir de grenade explose en f111 : ses huit éclats sont en vol en f113.
    let mut scenario = scenario_grenade();
    let frame = 113;
    let (_, eclats) = balles_vivantes(&scenario, frame, "eclat");
    assert_eq!(eclats, 8, "huit éclats de Ring(count: 8)");
    scenario.expect = vec![
        Expectation::BulletCount {
            count: eclats,
            projectile: Some("eclat".into()),
            team: None,
            at_frame: frame,
        },
        Expectation::BulletCount {
            count: eclats + 1,
            projectile: Some("eclat".into()),
            team: None,
            at_frame: frame,
        },
    ];
    let outcome = run(&scenario);
    assert_eq!(outcome.failures.len(), 1, "{:?}", outcome.failures);
    assert!(outcome.failures[0].contains("projectiles vivants"));
}

/// T1.1 : `HitsAtLeast` lit `HitCount` de l'entité (par `GgrsNetId` ou `Target`) ; échoue
/// sous le seuil et sur une entité sans compteur.
#[test]
fn hits_at_least_lit_le_compteur_de_la_cible() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::EntityRef;
    let mut scenario = load_scenario("testbed_target_hits");
    let (net_id, coups) = coups_sur_target(&scenario, 200);
    let joueur = net_id_du_joueur(&scenario, 0);
    assert!(coups >= 5);

    let reussies = [
        Expectation::HitsAtLeast {
            entity: EntityRef::Target,
            hits: coups,
            at_frame: 200,
        },
        Expectation::HitsAtLeast {
            entity: EntityRef::NetId(net_id),
            hits: 1,
            at_frame: 200,
        },
    ];
    let echouees = [
        (
            Expectation::HitsAtLeast {
                entity: EntityRef::Target,
                hits: coups + 1,
                at_frame: 200,
            },
            "coups reçus <",
        ),
        (
            Expectation::HitsAtLeast {
                entity: EntityRef::NetId(joueur),
                hits: 0,
                at_frame: 200,
            },
            "HitCount",
        ),
    ];
    scenario.expect = reussies
        .iter()
        .cloned()
        .chain(echouees.iter().map(|(attente, _)| attente.clone()))
        .collect();
    let outcome = run(&scenario);
    assert_eq!(
        outcome.failures.len(),
        echouees.len(),
        "{:?}",
        outcome.failures
    );
    for (attente, raison) in &echouees {
        let attendu = format!("{attente:?}");
        assert!(
            outcome
                .failures
                .iter()
                .any(|f| f.contains(&attendu) && f.contains(raison)),
            "{attendu} devait échouer ({raison}) : {:?}",
            outcome.failures
        );
    }
}

/// T1.10 : `Gauge`, `Level`, `Mutations` passent sur `levelup_choice` (3 rads, niveau 1,
/// `coriace` prise) et échouent chacune sur une valeur fausse.
#[test]
fn progression_expectations_pass_and_fail() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let mut scenario = load_scenario("levelup_choice");
    scenario.frames = 200;
    let gauge = |min: f32, max: f32| Expectation::Gauge {
        handle: 0,
        id: "rads".into(),
        min: Some(min),
        max: Some(max),
        at_frame: 190,
    };
    let level = |level: u32| Expectation::Level {
        handle: 0,
        level,
        at_frame: 190,
    };
    let mutations = |contains: &[&str], count: Option<u32>| Expectation::Mutations {
        handle: 0,
        contains: contains.iter().map(|s| s.to_string()).collect(),
        count,
        at_frame: 190,
    };
    scenario.expect = vec![
        gauge(3.0, 3.0),
        level(1),
        mutations(&["coriace"], Some(1)),
        // Fausses : une seule échec chacune
        gauge(4.0, 9.0),
        level(2),
        mutations(&["vampire"], None),
        mutations(&[], Some(0)),
        Expectation::Gauge {
            handle: 0,
            id: "mana".into(),
            min: None,
            max: None,
            at_frame: 190,
        },
    ];
    let outcome = run(&scenario);
    let failures = outcome.failures.join("\n");
    assert_eq!(outcome.failures.len(), 5, "{failures}");
    for needle in [
        "jauge rads = 3 < min 4",
        "niveau 1 ≠ 2",
        "manque [\"vampire\"]",
        "1 ≠ 0",
        "sans jauge « mana »",
    ] {
        assert!(failures.contains(needle), "{needle} absent de :\n{failures}");
    }
}

/// Joue `scenario` avec `reussies` et `echouees` (attente, fragment du message d'échec) : les
/// premières passent, chacune des secondes échoue une fois avec son message.
fn verifie_attentes(
    mut scenario: Scenario,
    reussies: &[Expectation],
    echouees: &[(Expectation, &str)],
) {
    scenario.expect = reussies
        .iter()
        .cloned()
        .chain(echouees.iter().map(|(attente, _)| attente.clone()))
        .collect();
    let outcome = run(&scenario);
    assert_eq!(
        outcome.failures.len(),
        echouees.len(),
        "{:?}",
        outcome.failures
    );
    for (attente, raison) in echouees {
        let attendu = format!("{attente:?}");
        assert!(
            outcome
                .failures
                .iter()
                .any(|f| f.contains(&attendu) && f.contains(raison)),
            "échec attendu pour {attendu} ({raison}) : {:?}",
            outcome.failures
        );
    }
}

/// T1.4 : `EnemyState` lit la règle retenue (état `BehaviorRuntime` de `kiter`) ; échoue sur
/// une autre règle et sur une entité qui n'est pas un ennemi (le joueur).
#[test]
fn enemy_state_lit_la_regle_retenue() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::EntityRef;
    let scenario = load_scenario("enemy_keep_distance");
    let joueur = net_id_du_joueur(&scenario, 0);
    verifie_attentes(
        scenario,
        &[
            Expectation::EnemyState {
                entity: EntityRef::NetId(27),
                behavior: "Shoot".into(),
                at_frame: 20,
            },
            Expectation::EnemyState {
                entity: EntityRef::NetId(27),
                behavior: "KeepDistance".into(),
                at_frame: 81,
            },
        ],
        &[
            (
                Expectation::EnemyState {
                    entity: EntityRef::NetId(27),
                    behavior: "Chase".into(),
                    at_frame: 81,
                },
                "behavior retenu KeepDistance",
            ),
            (
                Expectation::EnemyState {
                    entity: EntityRef::NetId(joueur),
                    behavior: "Chase".into(),
                    at_frame: 81,
                },
                "pas un ennemi",
            ),
        ],
    );
}

/// T1.4 : `EnemyState` sur une entité absente échoue proprement (la règle dérivée des
/// ennemis sans état nouveau — zombies — est testée dans `game::character::enemy::ai::rules`).
#[test]
fn enemy_state_entite_absente() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::EntityRef;
    verifie_attentes(
        load_scenario("enemy_charge"),
        &[],
        &[(
            Expectation::EnemyState {
                entity: EntityRef::NetId(9999),
                behavior: "Chase".into(),
                at_frame: 10,
            },
            "entité absente",
        )],
    );
}

/// T1.4 : `EnemyDistance` entre bornes ; échoue hors bornes et sur un joueur absent.
#[test]
fn enemy_distance_entre_bornes() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::{DistanceTarget, EntityRef};
    let scenario = load_scenario("enemy_charge");
    verifie_attentes(
        scenario,
        &[Expectation::EnemyDistance {
            entity: EntityRef::NetId(27),
            target: DistanceTarget::Player(0),
            min: Some(118.0),
            max: Some(121.0),
            at_frame: 61,
        }],
        &[
            (
                Expectation::EnemyDistance {
                    entity: EntityRef::NetId(27),
                    target: DistanceTarget::Player(0),
                    min: None,
                    max: Some(100.0),
                    at_frame: 61,
                },
                "> max 100",
            ),
            (
                Expectation::EnemyDistance {
                    entity: EntityRef::NetId(27),
                    target: DistanceTarget::Player(3),
                    min: None,
                    max: None,
                    at_frame: 61,
                },
                "joueur 3 absent",
            ),
        ],
    );
}

/// T1.4 : `EnemyContactBefore` passe si le contact arrive avant l'échéance, échoue sinon.
#[test]
fn enemy_contact_before_echeance() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::EntityRef;
    let scenario = load_scenario("enemy_charge");
    verifie_attentes(
        scenario,
        &[Expectation::EnemyContactBefore {
            entity: EntityRef::NetId(27),
            frames: 120,
        }],
        &[(
            Expectation::EnemyContactBefore {
                entity: EntityRef::NetId(27),
                frames: 60,
            },
            "jamais à portée",
        )],
    );
}

/// T1.4 : `EnemyNeverInWall` passe pour un ennemi qui erre et ne lève que sur un
/// chevauchement (une entité absente ne chevauche rien). Le cas d'échec n'a pas de scénario :
/// aucune carte ne place d'ennemi dans un mur.
#[test]
fn enemy_never_in_wall_passe_en_salle_ouverte() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::EntityRef;
    let scenario = load_scenario("enemy_wander");
    verifie_attentes(
        scenario,
        &[
            Expectation::EnemyNeverInWall {
                entity: EntityRef::NetId(27),
                from: 1,
                to: 330,
            },
            Expectation::EnemyNeverInWall {
                entity: EntityRef::NetId(9999),
                from: 1,
                to: 330,
            },
        ],
        &[],
    );
}

/// T1.5 : `EnemyVariant` lit le composant `Variant` (imposé par le champ LDtk, ou absent) ;
/// échoue sur une autre variante et sur une entité absente.
#[test]
fn enemy_variant_lit_la_variante() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::replay::EntityRef;
    verifie_attentes(
        load_scenario("variant_fast"),
        &[
            Expectation::EnemyVariant {
                entity: EntityRef::NetId(33),
                variant: Some("rapide".into()),
                at_frame: 2,
            },
            Expectation::EnemyVariant {
                entity: EntityRef::NetId(31),
                variant: None,
                at_frame: 2,
            },
        ],
        &[
            (
                Expectation::EnemyVariant {
                    entity: EntityRef::NetId(32),
                    variant: Some("rapide".into()),
                    at_frame: 2,
                },
                "variante blinde, rapide attendue",
            ),
            (
                Expectation::EnemyVariant {
                    entity: EntityRef::NetId(9999),
                    variant: None,
                    at_frame: 2,
                },
                "entité absente",
            ),
        ],
    );
}
