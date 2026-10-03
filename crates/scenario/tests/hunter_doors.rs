//! A chasing zombie without a firing post must not suppress paid doors or repairs.
//! Seeds 11/12 previously capped at 20_000 frames in wave 1 with four healthy buyers.
use game::replay::{BotProfile, PlayerScript, Scenario};
use scenario::{run_with_options, StopEarly};

#[test]
fn buyers_finish_the_first_wave_when_awake_zombies_are_inaccessible() {
    for seed in [11, 12] {
        let scenario = Scenario {
            game: "zombies".into(),
            map: "exemples/test_map.ldtk".into(),
            map_seed: seed,
            frames: 3500,
            players: (0..4)
                .map(|_| PlayerScript {
                    bot: Some(BotProfile::Acheteur),
                    ..Default::default()
                })
                .collect(),
            expect: vec![],
            weapon_overrides: vec![],
            wave_overrides: None,
            invariants: Default::default(),
            powerups: vec![],
            powerup_drop_chance_override: None,
        };
        let outcome = run_with_options(
            &scenario,
            |_| {},
            Some(StopEarly {
                until_wave: Some(2),
                stop_when_all_players_dead: true,
            }),
        );
        assert!(
            outcome.failures.is_empty(),
            "seed {seed}: {:?}",
            outcome.failures
        );
        assert!(
            outcome.metrics.final_wave >= 2,
            "seed {seed}: wave {}, kills {}, alive {}",
            outcome.metrics.final_wave,
            outcome.metrics.kills,
            outcome.metrics.players_alive
        );
        assert_eq!(outcome.metrics.players_alive, 4, "seed {seed}");
        assert!(outcome.softlock.is_none(), "seed {seed}: capped");
    }
}
