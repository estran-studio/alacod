//! Lecture des périphériques du joueur local, moitié **application** (D69 étape 2, E4) : clavier, manette
//! et souris (leafwing), fenêtre et caméra. La simulation (`character::player::input::read_local_inputs`)
//! ne lit plus que la ressource [`DeviceInputs`] que ce module écrit ; elle ne nomme ni `ActionState`, ni
//! `Window`, ni `Camera`.
//!
//! Absent en headless (`CharacterAppPlugin::headless`) : aucun périphérique, aucun leafwing, input neutre.
//!
//! Chemin final (E9) : `game_app/src/character/input_devices.rs`.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leafwing_input_manager::prelude::ActionState;

use crate::character::control::{get_input_map, PlayerAction};
use crate::character::player::input::{
    BoxInput, DeviceInputs, InputSource, INPUT_BLANK, INPUT_BOMB, INPUT_CHOICE_A, INPUT_CHOICE_B,
    INPUT_CHOICE_C, INPUT_DASH, INPUT_DOWN, INPUT_DROP_WEAPON, INPUT_FORCE_CRASH,
    INPUT_INTERACTION, INPUT_LEFT, INPUT_MELEE_ATTACK, INPUT_MODIFIER, INPUT_RELOAD, INPUT_RIGHT,
    INPUT_SPRINT, INPUT_SWITCH_WEAPON_MODE, INPUT_UP, INPUT_USE_ACTIVE,
};
use crate::character::player::{LocalPlayer, Player};

/// `PreUpdate`, avant `RunGgrsSystems` : le joueur local (créé par la simulation, sans périphérique) reçoit son
/// `ActionState` et son `InputMap` (et `AiDriver` sous `harmonium`). D4 : l'`ActionState` arrive au plus
/// une frame de rendu après la création du joueur ; sans effet sur les inputs (neutres jusque-là).
pub fn attach_local_player_input(mut commands: Commands, added: Query<Entity, Added<LocalPlayer>>) {
    for entity in &added {
        commands
            .entity(entity)
            .try_insert((ActionState::<PlayerAction>::default(), get_input_map()));

        #[cfg(feature = "harmonium")]
        commands
            .entity(entity)
            .try_insert(harmonium_bevy::components::AiDriver {
                ai_influence: 1.0,
                detection_radius: 300.0,
                ..default()
            });
    }
}

/// `ReadInputs`, avant `read_local_inputs` : l'input de chaque joueur local (`InputSource::Devices`) d'après son
/// `ActionState`, la position du curseur dans le monde (fenêtre, caméra) et la carte de mutation
/// (`confirm_input`). Écrit [`DeviceInputs`], lue par la simulation.
pub fn collect_device_inputs(
    input_source: Res<InputSource>,
    players: Query<(&ActionState<PlayerAction>, &Transform, &Player), With<LocalPlayer>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mutation_screen: Option<Res<crate::ui::mutation_screen::MutationScreenView>>,
    mut device_inputs: ResMut<DeviceInputs>,
) {
    device_inputs.0.clear();
    if *input_source != InputSource::Devices {
        return;
    }
    for (action_state, transform, player) in &players {
        let mut input = BoxInput::default();

        if action_state.pressed(&PlayerAction::MoveUp) {
            input.buttons |= INPUT_UP;
        }
        if action_state.pressed(&PlayerAction::MoveDown) {
            input.buttons |= INPUT_DOWN;
        }
        if action_state.pressed(&PlayerAction::MoveLeft) {
            input.buttons |= INPUT_LEFT;
        }
        if action_state.pressed(&PlayerAction::MoveRight) {
            input.buttons |= INPUT_RIGHT;
        }

        if action_state.pressed(&PlayerAction::PointerClick) {
            input.fire = true;
        }

        if action_state.pressed(&PlayerAction::SwitchWeapon) {
            input.switch_weapon = true;
        }
        if action_state.pressed(&PlayerAction::SwitchWeaponMode) {
            input.buttons |= INPUT_SWITCH_WEAPON_MODE;
        }

        if action_state.pressed(&PlayerAction::Reload) {
            input.buttons |= INPUT_RELOAD;
        }

        if action_state.pressed(&PlayerAction::Sprint) {
            input.buttons |= INPUT_SPRINT;
        }

        if action_state.pressed(&PlayerAction::Dash) {
            input.buttons |= INPUT_DASH;
        }

        if action_state.pressed(&PlayerAction::Interaction) {
            input.buttons |= INPUT_INTERACTION;
        }

        if action_state.pressed(&PlayerAction::Modifier) {
            input.buttons |= INPUT_MODIFIER;
        }

        if action_state.pressed(&PlayerAction::MeleeAttack) {
            input.buttons |= INPUT_MELEE_ATTACK;
        }

        if action_state.just_pressed(&PlayerAction::DropWeapon) {
            input.buttons |= INPUT_DROP_WEAPON;
        }

        if action_state.pressed(&PlayerAction::UseActive) {
            input.buttons |= INPUT_USE_ACTIVE;
        }
        if action_state.pressed(&PlayerAction::Blank) {
            input.buttons |= INPUT_BLANK;
        }
        if action_state.pressed(&PlayerAction::Bomb) {
            input.buttons |= INPUT_BOMB;
        }

        if action_state.pressed(&PlayerAction::ChoiceA) {
            input.buttons |= INPUT_CHOICE_A;
        }
        if action_state.pressed(&PlayerAction::ChoiceB) {
            input.buttons |= INPUT_CHOICE_B;
        }
        if action_state.pressed(&PlayerAction::ChoiceC) {
            input.buttons |= INPUT_CHOICE_C;
        }
        // T1.16 : valider sur l'écran de mutation = le bit de la carte surlignée.
        if action_state.pressed(&PlayerAction::ChoiceConfirm) {
            input.buttons |= crate::ui::mutation_screen::confirm_input(
                mutation_screen.as_deref(),
                player.handle,
            );
        }

        // F12 to force crash (debug)
        if action_state.pressed(&PlayerAction::DebugForceCrash) {
            input.buttons |= INPUT_FORCE_CRASH;
        }

        if let Ok(window) = q_window.single() {
            if let Ok((camera, camera_transform)) = q_camera.single() {
                if let Some(cursor_position) = window.cursor_position() {
                    if let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_transform, cursor_position)
                    {
                        let player_position = transform.translation.truncate();
                        let pointer_distance = world_position - player_position;

                        input.pan_x = (pointer_distance.x)
                            .round()
                            .clamp(i16::MIN as f32, i16::MAX as f32)
                            as i16;
                        input.pan_y = (pointer_distance.y)
                            .round()
                            .clamp(i16::MIN as f32, i16::MAX as f32)
                            as i16;
                    }
                }
            }
        }

        device_inputs.0.insert(player.handle, input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::player::input::read_local_inputs;
    use crate::character::player::jjrs::PeerConfig;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_ggrs::{LocalInputs, LocalPlayers};
    use utils::frame::FrameCount;

    /// Maintenir Lâcher n'envoie qu'une requête et se réarme au relâchement ; maintenir Interaction reste
    /// supporté. Le test passe par `collect_device_inputs` puis `read_local_inputs`, comme `ReadInputs`.
    #[test]
    fn held_drop_sends_one_request_and_rearms_after_release() {
        let mut world = World::new();
        world.insert_resource(InputSource::Devices);
        world.init_resource::<DeviceInputs>();
        world.insert_resource(LocalPlayers(vec![0]));
        world.insert_resource(FrameCount::default());
        let mut actions = ActionState::<PlayerAction>::default();
        actions.press(&PlayerAction::DropWeapon);
        actions.press(&PlayerAction::Interaction);
        let player = world
            .spawn((
                actions,
                Transform::default(),
                Player::default(),
                LocalPlayer {},
            ))
            .id();
        let read = |world: &mut World| {
            world.run_system_once(collect_device_inputs).unwrap();
            world.run_system_once(read_local_inputs).unwrap();
            world.resource::<LocalInputs<PeerConfig>>().0[&0].buttons
        };
        assert_ne!(read(&mut world) & INPUT_DROP_WEAPON, 0);
        let instant = bevy::time::Time::<bevy::time::Real>::default().startup();
        for _ in 0..120 {
            {
                let mut entity = world.entity_mut(player);
                let mut actions = entity.get_mut::<ActionState<PlayerAction>>().unwrap();
                actions.tick(instant, instant);
                actions.press(&PlayerAction::DropWeapon);
            }
            let buttons = read(&mut world);
            assert_eq!(buttons & INPUT_DROP_WEAPON, 0, "held drop must not repeat");
            assert_ne!(
                buttons & INPUT_INTERACTION,
                0,
                "holding interaction remains supported"
            );
        }
        {
            let mut entity = world.entity_mut(player);
            let mut actions = entity.get_mut::<ActionState<PlayerAction>>().unwrap();
            actions.release(&PlayerAction::DropWeapon);
            actions.tick(instant, instant);
            actions.press(&PlayerAction::DropWeapon);
        }
        assert_ne!(read(&mut world) & INPUT_DROP_WEAPON, 0);
    }

    /// Un joueur local sans périphérique (headless, `DeviceInputs` absente) envoie un input neutre :
    /// aucune entrée, aucune panique.
    #[test]
    fn without_device_inputs_the_local_player_is_neutral() {
        let mut world = World::new();
        world.insert_resource(InputSource::Devices);
        world.insert_resource(LocalPlayers(vec![0]));
        world.insert_resource(FrameCount::default());
        world.run_system_once(read_local_inputs).unwrap();
        assert_eq!(world.resource::<LocalInputs<PeerConfig>>().0[&0].buttons, 0);
    }
}
