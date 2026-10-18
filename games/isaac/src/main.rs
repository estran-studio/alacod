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

/// Retourne l'`AppExit` (code de sortie du processus : desync p2p headless = 3).
fn main() -> AppExit {
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/isaac`), capturé à la compilation :
    // voir `games/zombies/src/main.rs` et `crates/content/src/manifest.rs`.
    #[cfg(not(target_arch = "wasm32"))]
    let game_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    #[cfg(not(target_arch = "wasm32"))]
    let loaded = content::load_and_lint(&game_dir);
    #[cfg(target_arch = "wasm32")]
    let loaded = content::load_embedded(EMBEDDED_CONTENT);
    let (registry, manifest, errors) = loaded.unwrap_or_else(|e| {
        eprintln!("games/isaac : {e}");
        std::process::exit(1);
    });
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        eprintln!(
            "games/isaac : {} erreur(s) de contenu, arrêt (voir `alacod lint games/isaac`)",
            errors.len()
        );
        std::process::exit(1);
    }
    // `isaac` ne déclare pas de dossier `Wave` (mode `Floors`, M3-T0f) : `wave_mode`
    // est donc `false`.
    let wave_mode = !registry.waves.is_empty();

    let mut game_config = CoreSetupConfig::from_env("isaac");
    // Assets du jeu : chemin explicite (dossier du manifeste capturé à la compilation), pour
    // que le binaire lancé hors de `cargo run` (CI de nuit, p2p headless) les trouve aussi.
    #[cfg(not(target_arch = "wasm32"))]
    {
        game_config.asset_root = Some(game_dir.join("assets").to_string_lossy().into_owned());
    }
    #[cfg(target_arch = "wasm32")]
    {
        game_config.asset_root = Some(format!("/builds/{}/isaac/assets", env!("APP_VERSION")));
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
        // Carte, graine et joueurs : point d'entrée du manifeste (`game.ron`, première caverne
        // de la séquence `Floors`).
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: manifest.entry.start_map.clone(),
            seed: manifest.entry.default_seed,
        }))
        .insert_resource(WaveModeEnabled(wave_mode));
    // `--seed <n>` : graine de carte explicite (rejouer une graine de campagne de bots, preuves p2p : les deux pairs
    // reçoivent la même). Sans elle, la graine du manifeste. Copie de `games/gungeon/src/main.rs`.
    app.add_systems(Startup, apply_launch_seed);
    app.run()
}

/// `--seed` (`LaunchSeed`) remplace la graine du manifeste, en local comme en ligne.
fn apply_launch_seed(launch: Res<game::args::LaunchSeed>, mut map: ResMut<LdtkGameMap>) {
    if let Some(seed) = launch.0 {
        map.seed = seed;
        info!("isaac : graine de carte imposée --seed {seed}");
    }
}
