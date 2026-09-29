use bevy::prelude::*;
use game::{
    args::BaseArgsPlugin,
    core::{CoreSetupConfig, CoreSetupPlugin},
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};

fn main() {
    let game_config = CoreSetupConfig::from_env("testbed");

    let core_plugin = CoreSetupPlugin(game_config);

    App::new()
        // Configure the bevy default plugins from our core_plugin configuration
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        // Core systems and components
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Map, seed and player spawning (testbed with minimal map)
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: "testbed/testbed_empty.ldtk".into(),
            seed: 123456,
        }))
        .run();
}
