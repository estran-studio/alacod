//! Le plafond est provoqué volontairement : aucune modification de règle de jeu.
use game::replay::{BotProfile, PlayerScript, Scenario};
use scenario::{run_with_options, StopEarly};

fn scenario(frames: u32) -> Scenario {
    Scenario {
        game: "zombies".into(),
        map: "exemples/test_map.ldtk".into(),
        map_seed: 123456,
        frames,
        players: vec![PlayerScript {
            bot: Some(BotProfile::Immobile),
            ..Default::default()
        }],
        expect: vec![],
        weapon_overrides: vec![],
        wave_overrides: None,
        invariants: Default::default(),
        powerups: vec![],
        powerup_drop_chance_override: None,
        floors: None,
        progression: None,
        clocks: None,
        difficulty: None,
        characters: vec![],
        mode: None,
    }
}

#[test]
fn plafond_dump_et_lecture_sans_effet_sur_la_trace() {
    let scenario = scenario(601);
    let outcome = run_with_options(&scenario, |_| {}, Some(StopEarly::default()));
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let dump = outcome.softlock.as_ref().expect("plafond atteint");
    assert_eq!(dump.previous.as_ref().unwrap().frame, 1);
    assert_eq!(dump.final_state.frame, 601);
    assert!(!dump.final_state.enemies.is_empty());
    assert_eq!(dump.final_state.players.len(), 1);
    assert!(!dump.final_state.closed_doors.is_empty());
    let json = serde_json::to_value(dump).unwrap();
    assert!(json["final_state"]["enemies"][0]["position"]["x"].is_string());
    assert!(json["final_state"]["wave"]["phase"].is_string());
    assert!(json["final_state"]["navigation"].is_string());
    let without_dump = scenario::run(&scenario);
    assert!(
        without_dump.failures.is_empty(),
        "{:?}",
        without_dump.failures
    );
    assert!(without_dump.softlock.is_none());
    assert_eq!(outcome.trace, without_dump.trace);
}

#[test]
fn objectif_atteint_a_la_frame_plafond_sans_dump() {
    let outcome = run_with_options(
        &scenario(1),
        |_| {},
        Some(StopEarly {
            until_floor: None,
            until_wave: Some(1),
            stop_when_all_players_dead: true,
            stop_when_run_ended: false,
        }),
    );
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert_eq!(outcome.metrics.frames, 1);
    assert!(outcome.softlock.is_none());
}
