use animation::AnimationState;
use animation::{ActiveLayers, FacingDirection};
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, platform::collections::hash_map::HashMap};
use bevy_fixed::fixed_math;
use bevy_ggrs::prelude::*;
use bevy_ggrs::{LocalInputs, LocalPlayers};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use utils::{frame::FrameCount, order_mut_iter, net_id::GgrsNetId};

use crate::character::config::{CharacterConfig, CharacterConfigHandles};
use crate::character::dash::DashState;
use crate::character::movement::{SprintState, Velocity};
use crate::character::player::{control::PlayerAction, Player};
use crate::collider::{is_colliding, Collider, CollisionLayer, CollisionSettings};
use crate::weapons::WeaponInventory;

use super::jjrs::PeerConfig;
use super::LocalPlayer;

pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0; // 60 FPS fixed timestep

pub const INPUT_UP: u16 = 1 << 0;
pub const INPUT_DOWN: u16 = 1 << 1;
pub const INPUT_LEFT: u16 = 1 << 2;
pub const INPUT_RIGHT: u16 = 1 << 3;
pub const INPUT_RELOAD: u16 = 1 << 4;
pub const INPUT_SWITCH_WEAPON_MODE: u16 = 1 << 5;
pub const INPUT_SPRINT: u16 = 1 << 6;
pub const INPUT_DASH: u16 = 1 << 7;
pub const INPUT_MODIFIER: u16 = 1 << 8;
pub const INPUT_INTERACTION: u16 = 1 << 9;
pub const INPUT_MELEE_ATTACK: u16 = 1 << 10;
pub const INPUT_FORCE_CRASH: u16 = 1 << 11;

const PAN_FACING_THRESHOLD: i16 = 5;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoxInput {
    pub buttons: u16,
    pub pan_x: i16,
    pub pan_y: i16,

    pub fire: bool,
    pub switch_weapon: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PointerWorldPosition(pub Vec2);

/// Component for the weapon sprite's position relative to player
#[derive(Component, Clone, Copy, Default)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
}

/// Component that tracks interaction input state
#[derive(Component, Clone, Copy, Default, Debug, Hash, Serialize, Deserialize)]
pub struct InteractionInput {
    pub is_holding: bool,
}

/// Direction de déplacement demandée par les touches (non normalisée, composantes -1/0/1).
fn movement_direction(input: &BoxInput) -> fixed_math::FixedVec2 {
    let mut direction = fixed_math::FixedVec2::ZERO;
    if input.buttons & INPUT_UP != 0 {
        direction.y += fixed_math::FIXED_ONE;
    }
    if input.buttons & INPUT_DOWN != 0 {
        direction.y -= fixed_math::FIXED_ONE;
    }
    if input.buttons & INPUT_LEFT != 0 {
        direction.x -= fixed_math::FIXED_ONE;
    }
    if input.buttons & INPUT_RIGHT != 0 {
        direction.x += fixed_math::FIXED_ONE;
    }
    direction
}

fn get_facing_direction(input: &BoxInput) -> FacingDirection {
    // Use pan (cursor) input for 8-directional aiming if available
    if input.pan_x.abs() > PAN_FACING_THRESHOLD || input.pan_y.abs() > PAN_FACING_THRESHOLD {
        let direction_vec = fixed_math::FixedVec2::new(
            fixed_math::new(input.pan_x as f32),
            fixed_math::new(input.pan_y as f32),
        );
        // Normalize to prevent overflow in atan2 calculations with large input values
        let normalized = direction_vec.normalize_or_zero();
        return FacingDirection::from_fixed_vector(normalized);
    }
    
    // Fallback to movement keys for 8-directional movement
    let mut direction = fixed_math::FixedVec2::ZERO;
    
    if input.buttons & INPUT_RIGHT != 0 {
        direction.x += fixed_math::FIXED_ONE;
    }
    if input.buttons & INPUT_LEFT != 0 {
        direction.x -= fixed_math::FIXED_ONE;
    }
    if input.buttons & INPUT_UP != 0 {
        direction.y += fixed_math::FIXED_ONE;
    }
    if input.buttons & INPUT_DOWN != 0 {
        direction.y -= fixed_math::FIXED_ONE;
    }
    
    if direction.length_squared() > fixed_math::new(0.01) {
        FacingDirection::from_fixed_vector(direction)
    } else {
        // Default to right if no input
        FacingDirection::Right
    }
}

/// Source des inputs des joueurs locaux.
///
/// Choisie par la variable d'environnement `ALACOD_INPUT` (`devices` par défaut).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputSource {
    /// Clavier et souris.
    #[default]
    Devices,
    /// Aucun input : les joueurs locaux ne bougent pas. Rend un run reproductible,
    /// indépendamment de la position de la souris.
    Neutral,
    /// Inputs lus dans la ressource [`ScriptedInputs`] (scénarios, replays).
    Scripted,
    /// Inputs maintenus dans [`RemoteInputs`], modifiés par le contrôle remote.
    Remote,
}

/// Input maintenu par joueur (handle GGRS) en mode [`InputSource::Remote`] ; un joueur
/// absent envoie un input neutre.
#[derive(Resource, Clone, Debug, Default)]
pub struct RemoteInputs {
    pub held: HashMap<usize, BoxInput>,
}

/// Input d'un joueur sur les frames `from..to`.
#[derive(Clone, Debug)]
pub struct InputSegment {
    pub from: u32,
    pub to: u32,
    pub input: BoxInput,
}

/// Pistes d'inputs scriptées, une par joueur (index = handle GGRS). Hors de ses
/// segments, un joueur envoie un input neutre.
#[derive(Resource, Clone, Debug, Default)]
pub struct ScriptedInputs {
    pub players: Vec<Vec<InputSegment>>,
}

impl ScriptedInputs {
    pub fn input_at(&self, handle: usize, frame: u32) -> BoxInput {
        self.players
            .get(handle)
            .and_then(|segments| segments.iter().find(|s| (s.from..s.to).contains(&frame)))
            .map(|s| s.input)
            .unwrap_or_default()
    }
}

impl InputSource {
    pub fn from_env() -> Self {
        match std::env::var("ALACOD_INPUT").as_deref() {
            Err(_) | Ok("devices") => Self::Devices,
            Ok("neutral") => Self::Neutral,
            Ok(other) => panic!("ALACOD_INPUT inconnu : {other} (attendu : devices, neutral)"),
        }
    }
}

pub fn read_local_inputs(
    mut commands: Commands,
    input_source: Res<InputSource>,
    players: Query<(&ActionState<PlayerAction>, &Transform, &Player), With<LocalPlayer>>,

    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    local_players: Res<LocalPlayers>,
    frame: Res<FrameCount>,
    scripted: Option<Res<ScriptedInputs>>,
    remote: Option<Res<RemoteInputs>>,
) {
    let mut local_inputs = HashMap::new();

    if *input_source == InputSource::Remote {
        let remote = remote.expect("InputSource::Remote demande la ressource RemoteInputs");
        for handle in &local_players.0 {
            local_inputs.insert(*handle, remote.held.get(handle).copied().unwrap_or_default());
        }
    }

    if *input_source == InputSource::Scripted {
        let scripted = scripted.expect("InputSource::Scripted demande la ressource ScriptedInputs");
        for handle in &local_players.0 {
            local_inputs.insert(*handle, scripted.input_at(*handle, frame.frame));
        }
    }

    for (action_state, transform, player) in players
        .iter()
        .filter(|_| *input_source == InputSource::Devices)
    {
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

        local_inputs.insert(player.handle, input);
    }

    // GGRS exige un input par joueur local à chaque frame : un joueur mort
    // (entité despawn) ou pas encore créé envoie un input neutre.
    for handle in &local_players.0 {
        local_inputs.entry(*handle).or_insert_with(BoxInput::default);
    }

    commands.insert_resource(LocalInputs::<PeerConfig>(local_inputs));
}

pub fn apply_inputs(
    _commands: Commands,
    inputs: Res<PlayerInputs<PeerConfig>>,
    character_configs: Res<Assets<CharacterConfig>>,
    mut query: Query<
        (
            &GgrsNetId,
            Entity,
            &WeaponInventory,
            &mut fixed_math::FixedTransform3D,
            &mut DashState,
            &mut Velocity,
            &mut ActiveLayers,
            &mut FacingDirection,
            &mut CursorPosition,
            &mut SprintState,
            &mut InteractionInput,
            &CharacterConfigHandles,
            &Player,
        ),
        With<Rollback>,
    >,
) {
    for (
        _net_id,
        _entity,
        _inventory,
        transform,
        mut dash_state,
        mut velocity,
        _active_layers,
        mut facing_direction,
        mut cursor_position,
        mut sprint_state,
        mut interaction_input,
        config_handles,
        player,
    ) in order_mut_iter!(query)
    {
        if let Some(config) = character_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];

            if input.buttons & INPUT_FORCE_CRASH != 0 {
                panic!("FORCED CRASH BY PLAYER {}", player.handle);
            }

            let was_dashing = dash_state.is_dashing;
            dash_state.update();
            if was_dashing && !dash_state.is_dashing {
                // End of the dash: stop instead of coasting at dash speed
                velocity.main = fixed_math::FixedVec2::ZERO;
            }

            // Update interaction input state
            interaction_input.is_holding = (input.buttons & INPUT_INTERACTION) != 0;

            // While dashing, the dash is a velocity applied by move_characters, which handles
            // collisions (a dash stops at walls instead of going through them)
            if dash_state.is_dashing {
                let dash_duration = fixed_math::Fixed::from_num(config.movement.dash_duration_frames.max(1));
                let distance_per_frame = dash_state.dash_total_distance / dash_duration;
                velocity.main = dash_state.dash_direction * distance_per_frame
                    / fixed_math::new(FIXED_TIMESTEP);
                continue;
            }

            // Check if player is trying to dash
            if (input.buttons & INPUT_DASH != 0) && dash_state.can_dash() {
                let move_direction = movement_direction(&input);
                let look_direction = fixed_math::FixedVec2::new(
                    fixed_math::Fixed::from_num(input.pan_x),
                    fixed_math::Fixed::from_num(input.pan_y),
                );

                let is_reverse_dash = (input.buttons & INPUT_MODIFIER) != 0;

                // Dash where the player is moving; if not moving, where they aim; otherwise
                // where they face
                let mut dash_direction = if move_direction != fixed_math::FixedVec2::ZERO {
                    move_direction.normalize_or_zero()
                } else if look_direction.length_squared() > fixed_math::FIXED_ONE {
                    look_direction.normalize_or_zero()
                } else {
                    fixed_math::FixedVec2::new(
                        fixed_math::new(facing_direction.to_int() as f32),
                        fixed_math::new(0.0),
                    )
                };

                if is_reverse_dash {
                    dash_direction = -dash_direction;
                }

                // Start dash with current position
                dash_state.start_dash(
                    dash_direction,
                    transform.translation,
                    config.movement.dash_distance,
                    config.movement.dash_duration_frames,
                );
                dash_state.set_cooldown(config.movement.dash_cooldown_frames);

                // Zero out velocity to prevent normal movement physics
                velocity.main = fixed_math::FixedVec2::ZERO;
                continue;
            }

            let is_sprinting = input.buttons & INPUT_SPRINT != 0;
            sprint_state.is_sprinting = is_sprinting;

            if is_sprinting {
                sprint_state.sprint_factor += config.movement.sprint_acceleration_per_frame;
                sprint_state.sprint_factor = sprint_state.sprint_factor.min(fixed_math::FIXED_ONE);
            } else {
                sprint_state.sprint_factor -= config.movement.sprint_deceleration_per_frame;
                sprint_state.sprint_factor = sprint_state.sprint_factor.max(fixed_math::FIXED_ZERO);
            }

            let direction = movement_direction(&input);

            *facing_direction = get_facing_direction(&input);

            cursor_position.x = input.pan_x as i32;
            cursor_position.y = input.pan_y as i32;

            if direction != fixed_math::FixedVec2::ZERO {
                let sprint_multiplier = fixed_math::FIXED_ONE
                    + (config.movement.sprint_multiplier - fixed_math::FIXED_ONE)
                        * sprint_state.sprint_factor;
                // Using FIXED_TIMESTEP instead of time.delta()
                let move_delta = direction.normalize_or_zero()
                    * config.movement.acceleration
                    * sprint_multiplier
                    * fixed_math::new(FIXED_TIMESTEP);
                velocity.main += move_delta;

                let max_speed = config.movement.max_speed * sprint_multiplier;
                velocity.main = velocity.main.clamp_length_max(max_speed);
            }
        }
    }
}

pub fn apply_friction(
    inputs: Res<PlayerInputs<PeerConfig>>,
    movement_configs: Res<Assets<CharacterConfig>>,
    mut query: Query<(&GgrsNetId, &mut Velocity, &CharacterConfigHandles, &Player, &DashState), With<Rollback>>,
) {
    for (_net_id, mut velocity, config_handles, player, dash_state) in order_mut_iter!(query) {
        // The dash velocity is constant for its whole duration
        if dash_state.is_dashing {
            continue;
        }
        if let Some(config) = movement_configs.get(&config_handles.config) {
            let (input, _input_status) = inputs[player.handle];

            let moving = input.buttons & INPUT_RIGHT != 0
                || input.buttons & INPUT_LEFT != 0
                || input.buttons & INPUT_UP != 0
                || input.buttons & INPUT_DOWN != 0;

            if !moving && velocity.main.length_squared() > 0.1 {
                velocity.main = velocity.main
                    * (fixed_math::FIXED_ONE
                        - config.movement.friction * fixed_math::new(FIXED_TIMESTEP))
                    .max(fixed_math::FIXED_ZERO);
                if velocity.main.length_squared() < 1.0 {
                    velocity.main = fixed_math::FixedVec2::ZERO;
                }
            }
        }
    }
}

pub fn move_characters(
    mut query: Query<
        (
            &GgrsNetId,
            &mut fixed_math::FixedTransform3D,
            &mut Velocity,
            &Collider,
            &CollisionLayer,
        ),
        (With<Rollback>, With<Player>),
    >,
    settings: Res<CollisionSettings>,
    collider_query: Query<
        (
            Entity,
            &fixed_math::FixedTransform3D,
            &Collider,
            &CollisionLayer,
        ),
        (With<Collider>, Without<Player>, With<Rollback>),
    >,
) {
    for (_net_id, mut transform, mut velocity, player_collider, collision_layer) in order_mut_iter!(query) {
        let total_velocity = velocity.main + velocity.knockback;
        let delta_x = total_velocity.x * fixed_math::new(FIXED_TIMESTEP);
        let delta_y = total_velocity.y * fixed_math::new(FIXED_TIMESTEP);

        // Check for HARD collisions only (walls, not enemies)
        // Enemies are "soft" - player can push through them
        let check_hard_collision = |pos: &fixed_math::FixedVec3| -> bool {
            for (_target_entity, target_transform, target_collider, target_layer) in
                collider_query.iter()
            {
                // Skip if layers don't collide
                if !settings.layer_matrix[collision_layer.0][target_layer.0] {
                    continue;
                }
                // Only walls are hard collisions (enemy layer is soft)
                if target_layer.0 == settings.enemy_layer {
                    continue;
                }
                if is_colliding(pos, player_collider, &target_transform.translation, target_collider) {
                    return true;
                }
            }
            false
        };

        // Count enemy collisions for slowdown effect
        let count_enemy_collisions = |pos: &fixed_math::FixedVec3| -> u32 {
            let mut count = 0u32;
            for (_target_entity, target_transform, target_collider, target_layer) in
                collider_query.iter()
            {
                if target_layer.0 != settings.enemy_layer {
                    continue;
                }
                if is_colliding(pos, player_collider, &target_transform.translation, target_collider) {
                    count += 1;
                }
            }
            count
        };

        // Apply slowdown based on enemy collisions (more enemies = slower)
        let enemy_count = count_enemy_collisions(&transform.translation);
        let slowdown = if enemy_count > 0 {
            // Each enemy reduces speed by 20%, min 30% speed
            let factor = fixed_math::FIXED_ONE - fixed_math::new(0.2) * fixed_math::Fixed::from_num(enemy_count);
            factor.max(fixed_math::new(0.3))
        } else {
            fixed_math::FIXED_ONE
        };

        let delta_x = delta_x * slowdown;
        let delta_y = delta_y * slowdown;

        // Try full movement (X + Y)
        let full_pos = fixed_math::FixedVec3::new(
            transform.translation.x + delta_x,
            transform.translation.y + delta_y,
            transform.translation.z,
        );

        if !check_hard_collision(&full_pos) {
            transform.translation = full_pos;
            continue;
        }

        // Full movement blocked by wall - try sliding
        let mut moved_x = false;
        let mut moved_y = false;
        let start_x = transform.translation.x;
        let start_y = transform.translation.y;

        // Try X only
        if delta_x != fixed_math::FIXED_ZERO {
            let x_only_pos = fixed_math::FixedVec3::new(
                start_x + delta_x,
                start_y,
                transform.translation.z,
            );
            if !check_hard_collision(&x_only_pos) {
                transform.translation.x = x_only_pos.x;
                moved_x = true;
            }
        }

        // Try Y only
        if delta_y != fixed_math::FIXED_ZERO {
            let y_only_pos = fixed_math::FixedVec3::new(
                start_x,
                start_y + delta_y,
                transform.translation.z,
            );
            if !check_hard_collision(&y_only_pos) {
                transform.translation.y = y_only_pos.y;
                moved_y = true;
            }
        }

        // Opening assist: moving mainly along one axis and blocked, while a small side offset
        // (at most NUDGE_MAX) would clear the way (e.g. a door, 32 units high, entered a few
        // units off): slide toward that side at the movement speed
        const NUDGE_MAX: i32 = 12;
        let z = transform.translation.z;
        let half = |v: fixed_math::Fixed| v.abs() / fixed_math::new(2.0);
        let try_nudge = |primary: (fixed_math::Fixed, fixed_math::Fixed), side: (fixed_math::Fixed, fixed_math::Fixed)| {
            for n in 1..=NUDGE_MAX {
                for sign in [fixed_math::FIXED_ONE, -fixed_math::FIXED_ONE] {
                    let offset = fixed_math::Fixed::from_num(n) * sign;
                    let target = fixed_math::FixedVec3::new(
                        start_x + primary.0 + side.0 * offset,
                        start_y + primary.1 + side.1 * offset,
                        z,
                    );
                    if !check_hard_collision(&target) {
                        return Some(sign);
                    }
                }
            }
            None
        };
        if !moved_x && delta_x != fixed_math::FIXED_ZERO && delta_y.abs() <= half(delta_x) {
            if let Some(sign) = try_nudge((delta_x, fixed_math::FIXED_ZERO), (fixed_math::FIXED_ZERO, fixed_math::FIXED_ONE)) {
                let side_pos = fixed_math::FixedVec3::new(start_x, start_y + delta_x.abs() * sign, transform.translation.z);
                if !check_hard_collision(&side_pos) {
                    transform.translation.y = side_pos.y;
                    moved_y = true;
                }
            }
        }
        if !moved_y && delta_y != fixed_math::FIXED_ZERO && delta_x.abs() <= half(delta_y) {
            if let Some(sign) = try_nudge((fixed_math::FIXED_ZERO, delta_y), (fixed_math::FIXED_ONE, fixed_math::FIXED_ZERO)) {
                let side_pos = fixed_math::FixedVec3::new(start_x + delta_y.abs() * sign, start_y, transform.translation.z);
                if !check_hard_collision(&side_pos) {
                    transform.translation.x = side_pos.x;
                    moved_x = true;
                }
            }
        }

        // If blocked by walls on all sides, zero velocity
        if !moved_x && !moved_y {
            velocity.main = fixed_math::FixedVec2::ZERO;
        }
    }
}

pub fn update_animation_state(mut query: Query<(&GgrsNetId, &Velocity, &mut AnimationState), With<Rollback>>) {
    for (_net_id, velocity, mut state) in order_mut_iter!(query) {
        let current_state_name = state.0.clone();
        let new_state_name = if (velocity.main + velocity.knockback).length_squared() > 0.5 {
            "Run"
        } else {
            "Idle"
        };
        if current_state_name != new_state_name {
            state.0 = new_state_name.to_string();
        }
    }
}
