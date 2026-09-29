pub mod config;
pub mod create;
pub mod dash;
pub mod enemy;
pub mod health;
pub mod movement;
pub mod player;
pub mod visuals;

use animation::set_sprite_flip;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ggrs::{GgrsSchedule, ReadInputs};
use leafwing_input_manager::plugin::InputManagerPlugin;
use map::game::entity::map::enemy_spawn::EnemySpawnerComponent;

use crate::frame_events::FrameEventsAppExt;
use crate::{
    args::DebugAiConfig,
    character::{
        config::CharacterConfig,
        dash::DashState,
        enemy::{
            ai::{
                // New AI behavior systems
                behavior::{enemy_attack_system, enemy_target_selection},
                debug::{
                    draw_enemy_state_debug, draw_flow_field_debug, toggle_enemy_state_debug,
                    toggle_flow_field_debug, EnemyStateDebug, FlowFieldDebug,
                },
                // Flow field navigation
                navigation::{update_flow_field_system, FlowFieldCache, FlowFieldConfig},
                obstacle::{process_obstacle_damage, Obstacle, ObstacleAttackEvent},
                pathing::{move_enemies, update_enemy_targets, EnemyPath, PathfindingConfig},
                state::{EnemyAiConfig, EnemyTarget, MonsterState},
            },
            spawning::{enemy_spawn_from_spawners_system, EnemySpawnerState},
            Enemy,
        },
        health::{
            rollback_apply_accumulated_damage, rollback_apply_death, rollback_health_regeneration,
            ui::update_health_bars, DamageAccumulator, Death, Health, HealthRegen,
        },
        movement::{apply_knockback_damping, KnockbackDampingConfig, SprintState, Velocity},
        player::{
            control::PlayerAction,
            input::{
                apply_friction, apply_inputs, move_characters, read_local_inputs,
                update_animation_state, PointerWorldPosition,
            },
            Player,
        },
    },
    system_set::RollbackSystemSet,
    waves::WaveModeEnabled,
};

#[derive(Component, Clone, Copy, Default)]
pub struct Character;

pub struct BaseCharacterGamePlugin {}

impl Plugin for BaseCharacterGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RonAssetPlugin::<CharacterConfig>::new(&["ron"]),));

        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.init_resource::<PointerWorldPosition>();

        // Resources
        app.init_resource::<PathfindingConfig>();
        app.init_resource::<KnockbackDampingConfig>();

        // AI system resources
        app.init_resource::<FlowFieldCache>();
        app.init_resource::<FlowFieldConfig>();

        // Initialize debug resources with --debug-ai flag if present
        let debug_ai_enabled = app
            .world()
            .get_resource::<DebugAiConfig>()
            .map(|c| c.enabled)
            .unwrap_or(false);

        app.insert_resource(FlowFieldDebug {
            enabled: debug_ai_enabled,
            ..FlowFieldDebug::new()
        });
        app.insert_resource(EnemyStateDebug {
            enabled: debug_ai_enabled,
            ..EnemyStateDebug::new()
        });
        app.add_frame_events::<ObstacleAttackEvent>();

        // Rollback registration
        use crate::rollback::RollbackTraceApp;

        app.rollback_and_trace_resource::<PathfindingConfig>()
            .rollback_and_trace_resource::<KnockbackDampingConfig>()
            .rollback_and_trace::<EnemySpawnerComponent>()
            .rollback_and_trace::<EnemySpawnerState>()
            .rollback_and_trace::<EnemyPath>()
            .rollback_and_trace::<Obstacle>()
            .rollback_and_trace::<visuals::CharacterAppearance>()
            .rollback_and_trace::<enemy::ai::pathing::WallSlideTracker>()
            // New AI components
            .rollback_and_trace::<EnemyAiConfig>()
            .rollback_and_trace::<EnemyTarget>()
            .rollback_and_trace::<MonsterState>()
            // Ressource de présentation (curseur) glissée dans le rollback : rollback +
            // trace pour ne rien changer au snapshot, mais hors checksum (voir sa doc).
            .rollback_and_trace_copy_resource_no_checksum::<PointerWorldPosition>()
            .rollback_and_trace::<Health>()
            .rollback_and_trace::<HealthRegen>()
            .rollback_and_trace::<DamageAccumulator>()
            .rollback_and_trace::<DashState>()
            .rollback_and_trace::<SprintState>()
            .rollback_and_trace::<Velocity>()
            .rollback_and_trace::<Death>()
            .rollback_and_trace::<Player>()
            .rollback_and_trace::<Enemy>();

        // Rollback registration - Flow field cache
        app.rollback_and_trace_resource::<FlowFieldCache>();
        // Note: FlowFieldConfig is not rolled back (static configuration)

        app.insert_resource(player::input::InputSource::from_env());
        app.add_systems(ReadInputs, read_local_inputs);

        // Non-rollback systems: update visuals and debug

        app.add_systems(
            GgrsSchedule,
            (
                // HANDLE ALL PLAYERS INPUT
                (apply_inputs,).in_set(RollbackSystemSet::Input),
                // MOVEMENT CHARACTERS
                (apply_friction, move_characters.after(apply_friction))
                    .in_set(RollbackSystemSet::Movement),
                // HEALTH
                (
                    rollback_apply_accumulated_damage,
                    rollback_health_regeneration.after(rollback_apply_accumulated_damage),
                    rollback_apply_death.after(rollback_health_regeneration),
                )
                    .in_set(RollbackSystemSet::DeathManagement),
                // KNOCKBACK DAMPING - Apply after weapons (which apply knockback) but before animation/AI
                (apply_knockback_damping,)
                    .after(RollbackSystemSet::Weapon)
                    .before(RollbackSystemSet::AnimationUpdates)
                    .before(RollbackSystemSet::EnemyAI),
                // ANIMATION CRATE
                (update_animation_state,).in_set(RollbackSystemSet::AnimationUpdates),
                // SPAWNING (disabled when wave mode is enabled)
                (enemy_spawn_from_spawners_system,)
                    .run_if(|wave_mode: Res<WaveModeEnabled>| !wave_mode.0)
                    .in_set(RollbackSystemSet::EnemySpawning),
                // FLOW FIELD UPDATE (runs before EnemyAI)
                (update_flow_field_system,)
                    .after(RollbackSystemSet::EnemySpawning)
                    .before(RollbackSystemSet::EnemyAI),
                // ENEMY AI - Flow field navigation with collision
                // Uses new behavior.rs systems with MonsterState/EnemyTarget
                (
                    enemy_target_selection,
                    update_enemy_targets.after(enemy_target_selection),
                    move_enemies.after(update_enemy_targets),
                    enemy_attack_system.after(move_enemies),
                )
                    .in_set(RollbackSystemSet::EnemyAI),
                // OBSTACLE DAMAGE PROCESSING
                (process_obstacle_damage,)
                    .after(RollbackSystemSet::EnemyAI)
                    .before(RollbackSystemSet::FrameCounter),
            ),
        );
    }
}

/// Sprites, barres de vie et debug visuel de l'IA. Ajouté par `PresentationPlugin`.
pub struct CharacterPresentationPlugin;

impl Plugin for CharacterPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                visuals::attach_character_visuals,
                set_sprite_flip,
                update_health_bars,
                // Debug toggles
                toggle_flow_field_debug,
                toggle_enemy_state_debug,
                // Debug drawing
                draw_flow_field_debug,
                draw_enemy_state_debug,
            ),
        );
    }
}
