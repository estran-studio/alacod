pub mod config;
pub mod create;
pub mod dash;
pub mod enemy;
pub mod health;
pub mod movement;
pub mod player;
pub mod variant;
pub mod visuals;

use animation::set_sprite_flip;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ggrs::{GgrsSchedule, ReadInputs};
use combat::downed::{Downed, Reviving};
use leafwing_input_manager::plugin::InputManagerPlugin;
use map::game::entity::map::enemy_spawn::EnemySpawnerComponent;
use sim_core::kinds::{KindDecl, KindRegistry};
use stats::{expire_modifiers_system, StatsPlugin};

use crate::frame_events::FrameEventsAppExt;
use crate::{
    args::DebugAiConfig,
    character::{
        config::CharacterConfig,
        dash::DashState,
        enemy::{
            ai::{
                // New AI behavior systems
                behavior::{
                    enemy_attack_damage_translate_system, enemy_attack_system,
                    enemy_target_selection,
                },
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
            rollback_apply_accumulated_damage, rollback_apply_bleedout, rollback_apply_death,
            rollback_check_defeat, rollback_health_regeneration, rollback_resolve_damage_events,
            sync_health_from_stats, ui::update_health_bars, DamageAccumulator, Death, Health,
            HealthRegen, HitCount,
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
        // Stats et modificateurs (T1.2, chantier B2) : enregistre `Stats`/`Modifiers` en
        // rollback et l'expiration des modificateurs (`RollbackSystemSet::Status`). Voir
        // `docs/conventions.md` §7.
        app.add_plugins(StatsPlugin);
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
        // `FrameEvents<DamageEvent>` (T1.1, chantier B1) : voir la doc de
        // `sim_core::damage` pour les trois émetteurs et le résolveur unique.
        sim_core::damage::add_damage_events(app);

        // Kinds existants (lus par le lint de contenu, `crates/content`, T1.5). Les noms
        // viennent de `global_asset.rs` (`character_configs`) : ce sont les seuls types
        // d'ennemis chargés aujourd'hui, en dur ; T1.5 les lira depuis le manifeste RON.
        app.register_kinds([
            KindDecl::new("enemy", "zombie_1"),
            KindDecl::new("enemy", "zombie_2"),
            KindDecl::new("enemy", "zombie_full"),
        ]);

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
            // T1.2 : tir à distance. Checksum neutre (aucun ennemi existant n'en porte, voir
            // `RollbackTraceApp::rollback_and_trace_neutral`).
            .rollback_and_trace_neutral::<enemy::ai::state::RangedAttackState>()
            // T1.4 : état des behaviors nouveaux (KeepDistance, Strafe, Charge, Flee, Wander),
            // checksum neutre, posé seulement sur les ennemis qui en listent un.
            .rollback_and_trace_neutral::<enemy::ai::state::BehaviorRuntime>()
            // T1.5 : variante d'un personnage, checksum neutre (posée seulement si une
            // variante est choisie, voir `variant`).
            .rollback_and_trace_neutral::<variant::Variant>()
            // Ressource de présentation (curseur) glissée dans le rollback : rollback +
            // trace pour ne rien changer au snapshot, mais hors checksum (voir sa doc).
            .rollback_and_trace_copy_resource_no_checksum::<PointerWorldPosition>()
            .rollback_and_trace::<Health>()
            .rollback_and_trace::<HealthRegen>()
            .rollback_and_trace::<DamageAccumulator>()
            // Hors checksum GGRS (T2.9, voir la doc de `HitCount` et de
            // `RollbackTraceApp::rollback_and_trace_no_checksum`) : ne déplace pas le
            // checksum agrégé des scénarios zombies existants, qui n'en posent jamais.
            .rollback_and_trace_no_checksum::<HitCount>()
            .rollback_and_trace::<DashState>()
            .rollback_and_trace::<SprintState>()
            .rollback_and_trace::<Velocity>()
            .rollback_and_trace::<Death>()
            .rollback_and_trace::<Player>()
            .rollback_and_trace::<Enemy>()
            // À terre (T1.3, chantier B6) : `Downed`/`Reviving` n'apparaissent sur une
            // entité que quand un joueur tombe à terre (jamais en solo, voir la doc de
            // `combat::downed::Downed`).
            .rollback_and_trace::<Downed>()
            .rollback_and_trace::<Reviving>();

        // `run::run::Run` (T2.4, chantier F1, remplace `RunOutcome`) est enregistrée par
        // `run::RunPlugin` (`crate::core::CoreSetupPlugin`), pas ici : elle n'a pas de
        // valeur par défaut sensée (voir sa doc), contrairement à `RunOutcome` qu'elle
        // remplace.

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
                // DÉGÂTS (T1.1, chantier B1) : traducteur de l'attaque d'ennemi (frame
                // précédente, voir sa doc) puis résolveur unique
                // (`combat::damage::resolve_damage`) de tous les `DamageEvent` de la frame
                // (balles, mêlée, ennemis) — avant `DeathManagement`.
                (
                    enemy_attack_damage_translate_system,
                    // T1.4 : dégât de contact d'une ruée (`Charge`), même délai d'une frame.
                    enemy::ai::rules::charge_damage_translate_system
                        .after(enemy_attack_damage_translate_system),
                    rollback_resolve_damage_events
                        .after(enemy::ai::rules::charge_damage_translate_system),
                )
                    .in_set(RollbackSystemSet::CollisionDamage),
                // STATS (T1.2, chantier B2) : `Health.max`/`HealthRegen.regen_rate`
                // recalculés depuis les stats résolues, après l'expiration des
                // modificateurs (`stats::expire_modifiers_system`, ajouté par
                // `StatsPlugin`) — les deux sont dans `RollbackSystemSet::Status` et
                // touchent `Modifiers`, l'ordre doit être explicite (voir la doc de
                // `expire_modifiers_system`).
                sync_health_from_stats
                    .after(expire_modifiers_system)
                    .in_set(RollbackSystemSet::Status),
                // HEALTH — à terre (T1.3, chantier B6) : `rollback_apply_bleedout` juste
                // après (un joueur peut tomber à terre puis, une fois `bleedout_frames`
                // plus tard sans réanimation, mourir — jamais la même frame, voir sa doc) ;
                // `rollback_check_defeat` juste avant le despawn, pour voir les `Death`
                // posés cette frame (même contrainte que le suivi des kills de vagues, voir
                // `waves::WaveSystemPlugin`).
                (
                    rollback_apply_accumulated_damage,
                    rollback_apply_bleedout.after(rollback_apply_accumulated_damage),
                    rollback_health_regeneration.after(rollback_apply_bleedout),
                    rollback_check_defeat.after(rollback_health_regeneration),
                    rollback_apply_death.after(rollback_check_defeat),
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
                    // T1.4 : sélection par priorité des ennemis à behaviors nouveaux (les
                    // autres n'ont pas d'état nouveau, voir `enemy::ai::rules`).
                    enemy::ai::rules::behavior_select_system.after(enemy_target_selection),
                    update_enemy_targets.after(enemy::ai::rules::behavior_select_system),
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
