#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));

use bevy::prelude::*;
use entry::GameSpec;

/// Retourne l'`AppExit` (code de sortie du processus : desync p2p headless = 3). Le jeu ne déclare
/// aucun plugin : tout vient de son contenu (`game.ron`) et de l'entrée commune (`entry::run`, X-1).
fn main() -> AppExit {
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/throne`), capturé à la compilation : voir
    // `crates/content/src/manifest.rs` et `docs/conventions.md` §3.
    #[cfg(not(target_arch = "wasm32"))]
    let spec = GameSpec::new("throne", env!("CARGO_MANIFEST_DIR"));
    #[cfg(target_arch = "wasm32")]
    let spec = GameSpec::embedded("throne", EMBEDDED_CONTENT, env!("APP_VERSION"));
    entry::run(spec)
}
