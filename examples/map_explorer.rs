use bevy::prelude::*;
use game::{
    args::BaseArgsPlugin,
    core::{CoreSetupConfig, CoreSetupPlugin},
    waves::{WaveDebugEnabled, WaveModeEnabled},
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};

fn main() {
    let game_config = CoreSetupConfig::from_env("zrl-character_tester");

    let core_plugin = CoreSetupPlugin(game_config);

    App::new()
        // Configure the bevy default plugins from our core_plugin configuration
        // if you don't need special overwrite
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        // Core systems and components
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Map, seed and player spawning (shared with the scenario tests)
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: "exemples/test_map.ldtk".into(),
            seed: 123456,
        }))
        // Enable wave-based spawning mode (CoD Zombies style)
        .insert_resource(WaveModeEnabled(true))
        // Enable wave debug UI (toggle with F3)
        .insert_resource(WaveDebugEnabled(true))
        .run();
}
