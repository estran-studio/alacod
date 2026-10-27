mod plugin;

use bevy::prelude::*;
use entry::GameSpec;

fn main() -> AppExit {
    entry::run(GameSpec::new("plugdemo", env!("CARGO_MANIFEST_DIR")).plugin(plugin::Demo))
}
