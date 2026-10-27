#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));

use bevy::prelude::*;
use entry::GameSpec;
use game::extension::GamePlugin;
use map_ldtk::game::local::LdtkGameMap;

/// Ce que `zombies` ajoute à l'entrée commune (`entry::run`, X-1) : la graine tirée au hasard d'une
/// partie locale et l'overlay de debug des vagues. Rien dans la simulation.
struct Zombies;

impl GamePlugin for Zombies {
    fn assemble(&self, app: &mut App) {
        app.add_systems(
            OnEnter(game::session::AppState::LobbyLocal),
            choose_local_seed.before(map_ldtk::game::local::configure_map),
        )
        // Enable wave debug UI (toggle with F3)
        .insert_resource(game::waves::WaveDebugEnabled(game::core::debug_ui_enabled()));
    }
}

/// Retourne l'`AppExit` : en headless, un desync p2p sort en code 3 (`SESSION_LOST_EXIT_CODE`),
/// une trace écrite en 0 ; sans ce retour, le processus sortait toujours en 0.
fn main() -> AppExit {
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/zombies`), capturé à la compilation : fiable même si
    // le binaire est ensuite lancé hors de `cargo run` (voir `crates/content/src/manifest.rs` et
    // `docs/conventions.md` §3).
    #[cfg(not(target_arch = "wasm32"))]
    let spec = GameSpec::new("zombies", env!("CARGO_MANIFEST_DIR"));
    #[cfg(target_arch = "wasm32")]
    let spec = GameSpec::embedded("zombies", EMBEDDED_CONTENT, env!("APP_VERSION"));
    entry::run(spec.plugin(Zombies))
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
