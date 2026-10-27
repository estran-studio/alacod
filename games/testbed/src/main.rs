use bevy::prelude::*;
use entry::GameSpec;

/// Retourne l'`AppExit` (code de sortie du processus : desync p2p headless = 3). Le jeu ne déclare
/// aucun plugin : tout vient de son contenu (`game.ron`) et de l'entrée commune (`entry::run`, X-1).
fn main() -> AppExit {
    // `CARGO_MANIFEST_DIR` de ce binaire (`games/testbed`), capturé à la compilation : voir
    // `crates/content/src/manifest.rs` et `docs/conventions.md` §3.
    entry::run(GameSpec::new("testbed", env!("CARGO_MANIFEST_DIR")))
}
