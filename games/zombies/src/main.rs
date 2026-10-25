#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));

use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use game::content_hot_reload::GameRoot;
use game::{
    args::BaseArgsPlugin,
    core::CoreSetupPlugin,
    session::CoreSetupConfig,
    waves::{WaveDebugEnabled, WaveModeEnabled},
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Retourne l'`AppExit` : en headless, un desync p2p sort en code 3 (`SESSION_LOST_EXIT_CODE`),
/// une trace écrite en 0 ; sans ce retour, le processus sortait toujours en 0.
fn main() -> AppExit {
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/zombies`), capturé à la compilation :
    // fiable même si le binaire est ensuite lancé hors de `cargo run` (voir
    // `crates/content/src/manifest.rs` et `docs/conventions.md` §3).
    #[cfg(not(target_arch = "wasm32"))]
    let game_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Manifeste, registre et lint (T1.5) : refuse de démarrer sur contenu invalide,
    // exactement comme `alacod lint games/zombies` (même code, `content::load_and_lint`).
    #[cfg(not(target_arch = "wasm32"))]
    let loaded = content::load_and_lint(&game_dir);
    #[cfg(target_arch = "wasm32")]
    let loaded = content::load_embedded(EMBEDDED_CONTENT);
    let (registry, manifest, errors) = loaded.unwrap_or_else(|e| {
        eprintln!("games/zombies : {e}");
        std::process::exit(1);
    });
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{e}");
        }
        eprintln!(
            "games/zombies : {} erreur(s) de contenu, arrêt (voir `alacod lint games/zombies`)",
            errors.len()
        );
        std::process::exit(1);
    }
    // Mode vagues activé si (et seulement si) le jeu déclare un dossier de contenu `Wave` :
    // avant T1.5, `WaveModeEnabled(true)` était codé en dur ici.
    let wave_mode = !registry.waves.is_empty();

    let mut game_config = CoreSetupConfig::from_env("zombies");
    // Assets du jeu : chemin explicite (dossier du manifeste capturé à la compilation), pour
    // que le binaire lancé hors de `cargo run` (CI de nuit, p2p headless) les trouve aussi.
    #[cfg(not(target_arch = "wasm32"))]
    {
        game_config.asset_root = Some(game_dir.join("assets").to_string_lossy().into_owned());
    }
    #[cfg(target_arch = "wasm32")]
    {
        game_config.asset_root = Some(format!("/builds/{}/zombies/assets", env!("APP_VERSION")));
    }

    let core_plugin = CoreSetupPlugin(game_config);

    let mut app = App::new();
    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(GameRoot(game_dir));
    app
        // Configure the bevy default plugins from our core_plugin configuration
        // if you don't need special overwrite
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        .insert_resource(registry)
        // T2.4, chantier F1 : `game::jjrs::{local, p2p}` résout `RunMode` depuis
        // `entry.mode` au démarrage de session (voir `game::run_state::resolve_run_mode`).
        .insert_resource(manifest.clone())
        // Core systems and components
        .add_plugins(core_plugin)
        // Plugins for rogue like map with ldtk
        .add_plugins(LdtkRoguePlugin)
        // Map, seed and player spawning (shared with the scenario tests). Point d'entrée
        // du manifeste (`game.ron`, T1.5) plutôt qu'en dur.
        .add_plugins(LdtkLocalGamePlugin(LdtkGameMap {
            map_path: manifest.entry.start_map.clone(),
            seed: manifest.entry.default_seed,
            settings: manifest.entry.map.clone(),
        }))
        .add_systems(
            OnEnter(game::session::AppState::LobbyLocal),
            choose_local_seed.before(map_ldtk::game::local::configure_map),
        )
        // Enable wave-based spawning mode (CoD Zombies style)
        .insert_resource(WaveModeEnabled(wave_mode))
        // Enable wave debug UI (toggle with F3)
        .insert_resource(WaveDebugEnabled(game::core::debug_ui_enabled()))
        .run()
}

/// A fresh local run chooses a seed outside simulation; --seed (applied at `Startup` by the
/// shared `map_ldtk::game::local::apply_launch_seed`) reproduces a recorded run. Only the random
/// draw is zombies' own policy: the other games keep the manifest seed without `--seed`.
/// Online games retain the shared manifest seed (or `--seed`) until seed negotiation is implemented.
fn choose_local_seed(
    mut map: ResMut<LdtkGameMap>,
    explicit: Res<game::args::LaunchSeed>,
    mut windows: Query<&mut Window>,
) {
    if explicit.0.is_none() {
        let mut bytes = [0u8; 4];
        getrandom::getrandom(&mut bytes).expect("unable to choose a local map seed");
        map.seed = i32::from_le_bytes(bytes);
    }
    info!(
        "Le Relais — graine {} (rejouer avec --seed {})",
        map.seed, map.seed
    );
    for mut window in &mut windows {
        window.title = format!("Alacod Zombies — graine {}", map.seed);
    }
}
