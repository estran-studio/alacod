// Version web (`alacod build --web`, `docs/conventions.md` § 108) : le contenu (RON, LDtk, JSON) est
// embarqué dans le wasm par `build.rs`, les binaires (PNG, sons) sont servis par HTTP.
#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));

use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use game::content_hot_reload::GameRoot;
use game::{
    args::BaseArgsPlugin, core::CoreSetupPlugin, session::CoreSetupConfig, waves::WaveModeEnabled,
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

fn main() {
    // `CARGO_MANIFEST_DIR` de ce binaire (dossier du jeu), capturé à la compilation :
    // voir `games/zombies/src/main.rs` et `crates/content/src/manifest.rs`.
    #[cfg(not(target_arch = "wasm32"))]
    let game_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    #[cfg(not(target_arch = "wasm32"))]
    let loaded = content::load_and_lint(&game_dir);
    #[cfg(target_arch = "wasm32")]
    let loaded = content::load_embedded(EMBEDDED_CONTENT);
    let (registry, manifest, errors) = loaded.unwrap_or_else(|e| {
        eprintln!("{{name}} : {e}");
        std::process::exit(1);
    });
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        eprintln!(
            "{{name}} : {} erreur(s) de contenu, arrêt (voir `alacod lint {{name}}`)",
            errors.len()
        );
        std::process::exit(1);
    }
    // Ce jeu ne déclare pas de dossier `Wave` : `wave_mode` est donc `false`, comme
    // avant T1.5 (`WaveModeEnabled` n'était pas inséré, ce qui vaut `false` par défaut).
    let wave_mode = !registry.waves.is_empty();

    let mut game_config = CoreSetupConfig::from_env("{{name}}");
    // Assets du jeu : chemin explicite (dossier du manifeste capturé à la compilation), pour
    // que le binaire lancé hors de `cargo run` (CI de nuit, p2p headless) les trouve aussi.
    #[cfg(not(target_arch = "wasm32"))]
    {
        game_config.asset_root = Some(game_dir.join("assets").to_string_lossy().into_owned());
    }
    // Web : `alacod build --web` pose les assets sous `/builds/<APP_VERSION>/<nom du jeu>/assets` et refuse
    // un wasm qui ne contient pas cette chaîne (un seul littéral : `concat!`, pas `format!`).
    #[cfg(target_arch = "wasm32")]
    {
        game_config.asset_root =
            Some(concat!("/builds/", env!("APP_VERSION"), "/{{name}}/assets").to_string());
    }

    let core_plugin = CoreSetupPlugin(game_config);

    let mut app = App::new();
    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(GameRoot(game_dir));
    app
        // Configure the bevy default plugins from our core_plugin configuration
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        .insert_resource(registry)
        // T2.4, chantier F1 : voir `games/zombies/src/main.rs`.
        .insert_resource(manifest.clone())
        // Core systems and components
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Map, seed and player spawning (carte minimale). Point d'entrée du
        // manifeste (`game.ron`, T1.5) plutôt qu'en dur.
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: manifest.entry.start_map.clone(),
            seed: manifest.entry.default_seed,
            settings: manifest.entry.map.clone(),
        }))
        .insert_resource(WaveModeEnabled(wave_mode))
        .run();
}
