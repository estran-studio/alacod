use bevy::prelude::*;
use game::{
    args::BaseArgsPlugin,
    content_hot_reload::GameRoot,
    core::{CoreSetupConfig, CoreSetupPlugin},
    waves::WaveModeEnabled,
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};
use std::path::PathBuf;

fn main() {
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/testbed`), capturé à la compilation :
    // voir `games/zombies/src/main.rs` et `crates/content/src/manifest.rs`.
    let game_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (registry, manifest, errors) = content::load_and_lint(&game_dir).unwrap_or_else(|e| {
        eprintln!("games/testbed : {e}");
        std::process::exit(1);
    });
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        eprintln!(
            "games/testbed : {} erreur(s) de contenu, arrêt (voir `alacod lint games/testbed`)",
            errors.len()
        );
        std::process::exit(1);
    }
    // Le testbed ne déclare pas de dossier `Wave` : `wave_mode` est donc `false`, comme
    // avant T1.5 (`WaveModeEnabled` n'était pas inséré, ce qui vaut `false` par défaut).
    let wave_mode = !registry.waves.is_empty();

    let game_config = CoreSetupConfig::from_env("testbed");

    let core_plugin = CoreSetupPlugin(game_config);

    App::new()
        // Configure the bevy default plugins from our core_plugin configuration
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        .insert_resource(GameRoot(game_dir))
        .insert_resource(registry)
        // Core systems and components
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Map, seed and player spawning (testbed with minimal map). Point d'entrée du
        // manifeste (`game.ron`, T1.5) plutôt qu'en dur.
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: manifest.entry.start_map.clone(),
            seed: manifest.entry.default_seed,
        }))
        .insert_resource(WaveModeEnabled(wave_mode))
        .run();
}
