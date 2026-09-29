// Movement with collision detection
pub mod pathing;

// New generic AI system
pub mod behavior;
pub mod debug;
pub mod navigation;
pub mod obstacle;
pub mod state;

// Re-exports for convenience
pub use behavior::{
    enemy_attack_system, enemy_movement_system, enemy_stun_recovery_system, enemy_target_selection,
};
pub use debug::{
    draw_enemy_state_debug, draw_flow_field_debug, toggle_enemy_state_debug,
    toggle_flow_field_debug, EnemyStateDebug, FlowFieldDebug,
};
pub use navigation::{
    update_flow_field_system, FlowField, FlowFieldCache, FlowFieldConfig, GridPos, NavProfile,
    GRID_CELL_SIZE,
};
pub use obstacle::{
    process_obstacle_damage, Obstacle, ObstacleAttackEvent, ObstacleConfig, ObstacleType,
};
pub use pathing::WallSlideTracker;
pub use state::{
    AttackTarget, EnemyAiConfig, EnemyAiConfigRon, EnemyTarget, MonsterState, MovementType,
    TargetType,
};
