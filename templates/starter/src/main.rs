use bevy::prelude::*;
use entry::GameSpec;

/// Entrée du jeu : l'engine fournit le reste (`entry::run`) et sert la partie, le runner de scénarios
/// (`cargo run -- scenarios gen|play`) et le lint (`cargo run -- lint`). Le plugin Rust du jeu, s'il y
/// en a un, s'ajoute ici : `entry::run(GameSpec::new(..).plugin(MonPlugin))` (voir `CLAUDE.md`,
/// « Plugin Rust du jeu »).
fn main() -> AppExit {
    // `CARGO_MANIFEST_DIR` de ce binaire (dossier du jeu), capturé à la compilation : voir
    // `games/zombies/src/main.rs` et `crates/content/src/manifest.rs`.
    entry::run(GameSpec::new("{{name}}", env!("CARGO_MANIFEST_DIR")))
}
