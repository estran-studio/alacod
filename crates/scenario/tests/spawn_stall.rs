//! Regression: an empty distance range must still produce a wave in synctest.
use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use game::{
    balance::ResolvedBalance,
    replay::{BotProfile, PlayerScript, Scenario},
    waves::WavePhase,
};
use scenario::{run_with_options, StopEarly};

// Les systèmes de vague lisent la config résolue (F5), pas l'asset d'origine
fn force_empty_spawn_range(balance: Option<ResMut<ResolvedBalance>>) {
    if let Some(mut balance) = balance {
        let config = &mut balance.waves;
        config.min_player_distance = Fixed::ZERO;
        config.max_player_distance = Fixed::ZERO;
        config.base_enemies = 3;
        config.max_random_variance = 0;
    }
}

#[test]
fn empty_distance_range_spawns_after_deadline_without_desync() {
    let scenario = Scenario {
        game: "zombies".into(),
        map: "exemples/test_map.ldtk".into(),
        map_seed: 123456,
        frames: 1000,
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
    };
    let outcome = run_with_options(
        &scenario,
        |app| {
            app.add_systems(
                bevy_ggrs::GgrsSchedule,
                // Écrit les mêmes constantes de vague à chaque frame : l'ordre face aux
                // interactions (qui ne lisent que prix et perks) est sans effet
                force_empty_spawn_range
                    .before(game::waves::systems::wave_state_machine_system)
                    .ambiguous_with_all(),
            );
        },
        Some(StopEarly::default()),
    );
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let dump = outcome.softlock.expect("snapshot at frame limit");
    let before = dump.previous.unwrap();
    assert_eq!(before.wave.enemies_spawned_this_wave, 0);
    assert!(!before.wave.spawn_fallback);
    assert_eq!(dump.final_state.wave.enemies_spawned_this_wave, 3);
    assert_eq!(dump.final_state.wave.enemies_to_spawn, 0);
    assert_eq!(dump.final_state.wave.phase, WavePhase::InProgress);
    assert!(dump.final_state.wave.spawn_fallback);

    // Recovery is checksum-visible, and the next wave restores ordinary selection.
    use std::hash::{Hash, Hasher};
    let mut wave = dump.final_state.wave.clone();
    let mut active = std::collections::hash_map::DefaultHasher::new();
    wave.hash(&mut active);
    wave.spawn_fallback = false;
    let mut inactive = std::collections::hash_map::DefaultHasher::new();
    wave.hash(&mut inactive);
    assert_ne!(active.finish(), inactive.finish());
    wave.spawn_fallback = true;
    wave.prepare_next_wave(3, Fixed::ONE, Fixed::ONE);
    assert!(!wave.spawn_fallback);
}
