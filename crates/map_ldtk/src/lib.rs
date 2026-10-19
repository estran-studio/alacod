#[cfg(feature = "runtime")]
pub mod game;
pub mod generation;
pub mod loader;
pub mod map_const;
#[cfg(feature = "runtime")]
pub mod plugins;
pub mod preview;

/// Vrai si les tilemaps sont compilées avec leur rendu (feature `render`). Le rendu
/// exige un GPU : les runs headless doivent être compilés sans.
pub const RENDER_ENABLED: bool = cfg!(feature = "render");
