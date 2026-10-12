use bevy::prelude::*;
use bevy_fixed::fixed_math;

#[derive(Component, Clone, Debug)]
pub struct EnemySpawnerComponent {
    /// Immutable world-space defence-room bounds, opt-in for exterior sources.
    pub activation_area: Option<(fixed_math::FixedVec2, fixed_math::FixedVec2)>,
    pub spawn_radius: fixed_math::Fixed,
    pub min_spawn_distance: fixed_math::Fixed,
    pub max_cooldown: u32,
    pub max_enemies: u32,
    pub enemy_types: Vec<String>,
}

impl Default for EnemySpawnerComponent {
    fn default() -> Self {
        Self {
            activation_area: None,
            spawn_radius: fixed_math::new(20.0),
            min_spawn_distance: fixed_math::new(80.0), // Reduced for smaller map
            max_cooldown: 900,                         // 15 seconds at 60fps (slower spawn rate)
            max_enemies: 2,                            // Per spawner
            enemy_types: vec!["zombie_full".to_string()],
        }
    }
}

impl std::hash::Hash for EnemySpawnerComponent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        // Preserve the established checksum for legacy spawners without a binding.
        self.spawn_radius.hash(state);
        self.min_spawn_distance.hash(state);
        self.max_cooldown.hash(state);
        self.max_enemies.hash(state);
        self.enemy_types.hash(state);
        if let Some(area) = self.activation_area {
            area.hash(state);
        }
    }
}

impl EnemySpawnerComponent {
    pub fn defends(&self, position: fixed_math::FixedVec2) -> bool {
        self.activation_area.is_none_or(|(min, size)| {
            position.x >= min.x
                && position.x < min.x + size.x
                && position.y >= min.y
                && position.y < min.y + size.y
        })
    }
}
