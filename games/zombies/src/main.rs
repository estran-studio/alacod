use bevy::prelude::*;
use game::{
    args::BaseArgsPlugin,
    content_hot_reload::GameRoot,
    core::{CoreSetupConfig, CoreSetupPlugin},
    waves::{WaveDebugEnabled, WaveModeEnabled},
};
use map_ldtk::{
    game::local::{LdtkGameMap, LdtkLocalGamePlugin},
    plugins::LdtkRoguePlugin,
};
use std::path::PathBuf;

fn main() {
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/zombies`), capturé à la compilation :
    // fiable même si le binaire est ensuite lancé hors de `cargo run` (voir
    // `crates/content/src/manifest.rs` et `docs/conventions.md` §3).
    let game_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Manifeste, registre et lint (T1.5) : refuse de démarrer sur contenu invalide,
    // exactement comme `alacod lint games/zombies` (même code, `content::load_and_lint`).
    let (registry, manifest, errors) = content::load_and_lint(&game_dir).unwrap_or_else(|e| {
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

    let mut game_config = CoreSetupConfig::from_env("zrl-character_tester");
    // Assets du jeu : chemin explicite (dossier du manifeste capturé à la compilation), pour
    // que le binaire lancé hors de `cargo run` (CI de nuit, p2p headless) les trouve aussi.
    game_config.asset_root = Some(game_dir.join("assets").to_string_lossy().into_owned());

    let core_plugin = CoreSetupPlugin(game_config);

    App::new()
        // Configure the bevy default plugins from our core_plugin configuration
        // if you don't need special overwrite
        .add_plugins(core_plugin.get_default_plugin())
        // Load default arguments from cli or query params (MUST be before core_plugin for --debug-ai to work)
        .add_plugins(BaseArgsPlugin)
        .insert_resource(GameRoot(game_dir))
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
        }))
        .add_systems(
            OnEnter(game::core::AppState::LobbyLocal),
            choose_local_seed.before(map_ldtk::game::local::configure_map),
        )
        // Enable wave-based spawning mode (CoD Zombies style)
        .insert_resource(WaveModeEnabled(wave_mode))
        // Enable wave debug UI (toggle with F3)
        .insert_resource(WaveDebugEnabled(true))
        .run();
}

/// A fresh local run chooses a seed outside simulation; --seed reproduces a recorded run.
/// Online games retain the shared manifest seed until seed negotiation is implemented.
fn choose_local_seed(
    mut map: ResMut<LdtkGameMap>,
    explicit: Res<game::args::LaunchSeed>,
    mut windows: Query<&mut Window>,
) {
    map.seed = explicit.0.unwrap_or_else(|| {
        let mut bytes = [0u8; 4];
        getrandom::getrandom(&mut bytes).expect("unable to choose a local map seed");
        i32::from_le_bytes(bytes)
    });
    info!(
        "Le Relais — graine {} (rejouer avec --seed {})",
        map.seed, map.seed
    );
    for mut window in &mut windows {
        window.title = format!("Alacod Zombies — graine {}", map.seed);
    }
}
