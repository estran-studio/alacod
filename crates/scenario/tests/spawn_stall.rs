//! Regression: an empty distance range must still produce a wave in synctest.
use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use game::{
    global_asset::GlobalAsset,
    replay::{BotProfile, PlayerScript, Scenario},
    waves::{WaveConfig, WavePhase},
};
use scenario::{run_with_options, StopEarly};

fn force_empty_spawn_range(global: Res<GlobalAsset>, mut configs: ResMut<Assets<WaveConfig>>) {
    if let Some(mut config) = global.wave_config.as_ref().and_then(|h| configs.get_mut(h)) {
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
    };
    let outcome = run_with_options(
        &scenario,
        |app| {
            app.add_systems(
                bevy_ggrs::GgrsSchedule,
                force_empty_spawn_range.before(game::waves::systems::wave_state_machine_system),
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
