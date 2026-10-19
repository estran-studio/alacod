//! Plugins **application** du personnage (D69 étape 2, E4), sortis de `character/mod.rs` (simulation) :
//! [`CharacterAppPlugin`] (chargeur des configurations, périphériques, ressources de debug de l'IA) et
//! [`CharacterPresentationPlugin`] (sprites, barres de vie, debug visuel). Rien ici n'enregistre de type
//! rollback ni de système de `GgrsSchedule`.
//!
//! Chemin final (E9) : `game_app/src/character/mod.rs`.

use animation::set_sprite_flip;
use bevy::prelude::*;
use bevy_ggrs::ReadInputs;
use leafwing_input_manager::plugin::InputManagerPlugin;

use crate::args::DebugAiConfig;
use crate::character::{
    ai_debug::{
        draw_enemy_state_debug, draw_flow_field_debug, toggle_enemy_state_debug,
        toggle_flow_field_debug, EnemyStateDebug, FlowFieldDebug,
    },
    config_asset::CharacterConfigAssetPlugin,
    control::PlayerAction,
    health_bar::update_health_bars,
    input_devices::{attach_local_player_input, collect_device_inputs},
    player::input::{read_local_inputs, DeviceInputs},
    visuals,
};

/// Moitié application de `BaseCharacterGamePlugin`, ajoutée **juste après lui** (`core.rs`) :
/// - le chargeur des configurations de personnages (`CharacterConfigAsset`, E3a) ;
/// - hors headless : leafwing (`InputManagerPlugin::<PlayerAction>`), la ressource [`DeviceInputs`], la lecture des
///   périphériques (`collect_device_inputs`, `ReadInputs`, **avant** `read_local_inputs`) et la pose de
///   l'`ActionState` du joueur local (`attach_local_player_input`, `PreUpdate` avant `RunGgrsSystems`). En
///   headless il n'y a ni périphérique ni leafwing (l'ancien `skip_device_input` disparaît) : l'input d'un
///   joueur local sous `InputSource::Devices` est neutre, comme avant ;
/// - les ressources de debug de l'IA (`--debug-ai`).
pub struct CharacterAppPlugin {
    pub headless: bool,
}

impl Plugin for CharacterAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CharacterConfigAssetPlugin);

        if !self.headless {
            app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
            app.init_resource::<DeviceInputs>();
            app.add_systems(ReadInputs, collect_device_inputs.before(read_local_inputs));
            app.add_systems(
                PreUpdate,
                attach_local_player_input.before(bevy_ggrs::RunGgrsSystems),
            );
        }

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
                visuals::remove_revived_death_ghosts.before(visuals::spawn_death_ghosts),
                visuals::spawn_death_ghosts.after(visuals::attach_character_visuals),
                visuals::animate_death_ghosts,
                set_sprite_flip,
                visuals::face_stationary_enemies.after(set_sprite_flip),
                visuals::orient_familiars.after(set_sprite_flip),
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
