use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemySpawnConfig {
    /// Optional activation rectangle `(x, y, width, height)` in level-local LDtk pixels.
    /// The exporter translates its origin to world pixels; dimensions stay unchanged.
    #[serde(default)]
    pub activation_area: Option<(i32, i32, i32, i32)>,
}
