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
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/throne`), capturé à la compilation :
    // voir `games/zombies/src/main.rs` et `crates/content/src/manifest.rs`.
    let game_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (registry, manifest, errors) = content::load_and_lint(&game_dir).unwrap_or_else(|e| {
        eprintln!("games/throne : {e}");
        std::process::exit(1);
    });
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        eprintln!(
            "games/throne : {} erreur(s) de contenu, arrêt (voir `alacod lint games/throne`)",
            errors.len()
        );
        std::process::exit(1);
    }
    // `throne` ne déclare pas de dossier `Wave` (mode `Floors`, T1.0c) : `wave_mode` est donc
    // `false`.
    let wave_mode = !registry.waves.is_empty();

    let mut game_config = CoreSetupConfig::from_env("throne");
    // Assets du jeu : chemin explicite (dossier du manifeste capturé à la compilation), pour
    // que le binaire lancé hors de `cargo run` (CI de nuit, p2p headless) les trouve aussi.
    game_config.asset_root = Some(game_dir.join("assets").to_string_lossy().into_owned());

    let core_plugin = CoreSetupPlugin(game_config);

    App::new()
        // Configure the bevy default plugins from our core_plugin configuration
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        .insert_resource(GameRoot(game_dir))
        .insert_resource(registry)
        // T2.4, chantier F1 : voir `games/zombies/src/main.rs`.
        .insert_resource(manifest.clone())
        // Core systems and components
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Carte, graine et joueurs : point d'entrée du manifeste (`game.ron`, première caverne
        // de la séquence `Floors`).
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: manifest.entry.start_map.clone(),
            seed: manifest.entry.default_seed,
        }))
        .insert_resource(WaveModeEnabled(wave_mode))
        .run();
}
