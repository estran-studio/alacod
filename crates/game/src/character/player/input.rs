use animation::AnimationState;
use animation::FacingDirection;
use bevy::window::PrimaryWindow;
use bevy::{platform::collections::hash_map::HashMap, prelude::*};
use bevy_fixed::fixed_math;
use bevy_ggrs::prelude::*;
use bevy_ggrs::{LocalInputs, LocalPlayers};
use combat::downed::Downed;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use sim_core::stats::StatId;
use stats::StatReader;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_mut_iter};

use crate::character::config::{CharacterConfig, CharacterConfigHandles};
use crate::character::dash::DashState;
use crate::character::health::Health;
use crate::character::movement::{grip, run_velocity, SprintState, Velocity};
use crate::character::player::{control::PlayerAction, Player};
use crate::collider::{is_colliding, Collider, CollisionLayer, CollisionSettings};

use super::jjrs::PeerConfig;
use super::LocalPlayer;

pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0; // 60 FPS fixed timestep

pub use combat::actors::{
    BoxInput, CursorPosition, INPUT_BLANK, INPUT_CHOICE_A, INPUT_CHOICE_B, INPUT_CHOICE_C,
    INPUT_DASH, INPUT_DOWN, INPUT_DROP_WEAPON, INPUT_FORCE_CRASH, INPUT_INTERACTION, INPUT_LEFT,
    INPUT_MELEE_ATTACK, INPUT_MODIFIER, INPUT_RELOAD, INPUT_RIGHT, INPUT_SPRINT,
    INPUT_SWITCH_WEAPON_MODE, INPUT_UP, INPUT_USE_ACTIVE,
};

const PAN_FACING_THRESHOLD: i16 = 5;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PointerWorldPosition(pub Vec2);

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
    /// Inputs décidés par `crates/bots` (T2.11) : aucune branche de [`read_local_inputs`] ne
    /// traite ce mode (il se comporte comme [`Self::Neutral`] ici, input neutre par défaut) ;
    /// un système séparé (`bots::read_bot_inputs`, ajouté à `ReadInputs` après
    /// [`read_local_inputs`]) remplace ensuite l'input des joueurs locaux présents dans la
    /// ressource `BotAssignments` de `crates/bots`, quel que soit le mode courant. Ce variant
    /// sert de mode de base explicite pour `alacod-sim` (aucun joueur scripté) ; les scénarios
    /// RON qui mélangent joueurs scriptés et bots gardent [`Self::Scripted`] comme mode de base.
    Bot,
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
    mutation_screen: Option<Res<crate::ui::mutation_screen::MutationScreenView>>,
) {
    let mut local_inputs = HashMap::new();

    if *input_source == InputSource::Remote {
        let remote = remote.expect("InputSource::Remote demande la ressource RemoteInputs");
        for handle in &local_players.0 {
            local_inputs.insert(
                *handle,
                remote.held.get(handle).copied().unwrap_or_default(),
            );
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

        if action_state.pressed(&PlayerAction::DropWeapon) {
            input.buttons |= INPUT_DROP_WEAPON;
        }

        if action_state.pressed(&PlayerAction::UseActive) {
            input.buttons |= INPUT_USE_ACTIVE;
        }
        if action_state.pressed(&PlayerAction::Blank) {
            input.buttons |= INPUT_BLANK;
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

        local_inputs.insert(player.handle, input);
    }

    // GGRS exige un input par joueur local à chaque frame : un joueur mort
    // (entité despawn) ou pas encore créé envoie un input neutre.
    for handle in &local_players.0 {
        local_inputs
            .entry(*handle)
            .or_insert_with(BoxInput::default);
    }

    commands.insert_resource(LocalInputs::<PeerConfig>(local_inputs));
}

/// Direction d'un dash : celle des touches ; sans touche, la visée ; sans visée, le regard.
/// Inversée par `INPUT_MODIFIER` (dash arrière).
fn dash_direction(input: &BoxInput, facing: FacingDirection) -> fixed_math::FixedVec2 {
    let move_direction = movement_direction(input);
    let look_direction = fixed_math::FixedVec2::new(
        fixed_math::Fixed::from_num(input.pan_x),
        fixed_math::Fixed::from_num(input.pan_y),
    );
    let direction = if move_direction != fixed_math::FixedVec2::ZERO {
        move_direction.normalize_or_zero()
    } else if look_direction.length_squared() > fixed_math::FIXED_ONE {
        look_direction.normalize_or_zero()
    } else {
        facing_vector(facing).normalize_or_zero()
    };
    if input.buttons & INPUT_MODIFIER != 0 {
        -direction
    } else {
        direction
    }
}

/// Composantes -1/0/1 d'une direction de regard (non normalisée, comme les touches).
fn facing_vector(facing: FacingDirection) -> fixed_math::FixedVec2 {
    let (x, y) = match facing {
        FacingDirection::Right => (1, 0),
        FacingDirection::UpRight => (1, 1),
        FacingDirection::Up => (0, 1),
        FacingDirection::UpLeft => (-1, 1),
        FacingDirection::Left => (-1, 0),
        FacingDirection::DownLeft => (-1, -1),
        FacingDirection::Down => (0, -1),
        FacingDirection::DownRight => (1, -1),
    };
    fixed_math::FixedVec2::new(
        fixed_math::Fixed::from_num(x),
        fixed_math::Fixed::from_num(y),
    )
}

pub fn apply_inputs(
    _commands: Commands,
    frame: Res<FrameCount>,
    inputs: Res<PlayerInputs<PeerConfig>>,
    character_configs: Res<Assets<CharacterConfig>>,
    stats: StatReader,
    mut query: Query<
        (
            &GgrsNetId,
            Entity,
            &mut DashState,
            &mut Velocity,
            &mut FacingDirection,
            &mut CursorPosition,
            &mut SprintState,
            &mut InteractionInput,
            &CharacterConfigHandles,
            &Player,
            Has<Downed>,
            // T1.3 : `Stun`/`Freeze` (§19) : inputs ignorés par la simulation (jamais à la
            // lecture des inputs : un rejeu `Scripted` doit rester identique).
            Option<&combat::status::Statuses>,
            Option<&mut Health>,
        ),
        With<Rollback>,
    >,
) {
    let dt = fixed_math::new(FIXED_TIMESTEP);
    for (
        _net_id,
        entity,
        mut dash_state,
        mut velocity,
        mut facing_direction,
        mut cursor_position,
        mut sprint_state,
        mut interaction_input,
        config_handles,
        player,
        is_downed,
        statuses,
        health,
    ) in order_mut_iter!(query)
    {
        let Some(config) = character_configs.get(&config_handles.config) else {
            continue;
        };
        let movement = &config.movement;
        let (mut input, _input_status) = inputs[player.handle];
        let incapacitated = combat::status::incapacitated(statuses);
        if incapacitated {
            // Visée gardée, aucun bouton (ni déplacement, ni dash, ni interaction)
            input = BoxInput {
                pan_x: input.pan_x,
                pan_y: input.pan_y,
                ..Default::default()
            };
        }

        if input.buttons & INPUT_FORCE_CRASH != 0 {
            panic!("FORCED CRASH BY PLAYER {}", player.handle);
        }

        // Visée : recalculée à chaque frame, AVANT tout `continue` (dash). `system_weapon_position`
        // en déduit la rotation de l'arme dans la même frame ; le composant n'est pas rollbacké
        // (donnée dérivée de l'input, jamais lue d'une frame à l'autre). Écrite après le dash,
        // elle restait figée pendant celui-ci et une frame resimulée lisait la visée d'une
        // frame plus récente : desync du synctest (scénario `dash_aim_change`).
        cursor_position.x = input.pan_x as i32;
        cursor_position.y = input.pan_y as i32;

        // Update interaction input state
        interaction_input.is_holding = (input.buttons & INPUT_INTERACTION) != 0;

        // Stats branchées (T1.2, chantier B2) : résolues (base + modificateurs actifs) à la
        // place des constantes `config.movement.*` — `StatReader` retombe sur la valeur de
        // config si l'entité n'a pas la stat (garde défensive, voir sa doc).
        let move_speed = stats.get(entity, &StatId::MoveSpeed, movement.max_speed);

        // Dash : part sur l'appui (ou un appui gardé `dash_buffer_frames` frames), avance dès
        // cette frame et sort à la vitesse de course. À terre ou étourdi (T1.3) : pas de dash,
        // même d'un appui gardé d'avant le statut.
        dash_state.begin_frame();
        let requested =
            dash_state.read_button(input.buttons & INPUT_DASH != 0, movement.dash_buffer_frames);
        if requested && !is_downed && !incapacitated && dash_state.can_dash() {
            dash_state.start_dash(
                dash_direction(&input, *facing_direction),
                movement.dash_distance,
                movement.dash_duration_frames,
                move_speed * dt,
                movement.dash_cooldown_frames,
            );
            // Invulnérable les `dash_iframes` premières frames, celle-ci comprise
            // (`rollback_resolve_damage_events` compare la frame du coup à cette borne)
            if let (Some(mut health), true) = (health, movement.dash_iframes > 0) {
                let until = frame.frame + movement.dash_iframes - 1;
                health.invulnerable_until_frame = Some(
                    health
                        .invulnerable_until_frame
                        .map_or(until, |current| current.max(until)),
                );
            }
        } else {
            dash_state.age_buffer();
        }
        // Pendant le dash, son pas est une vitesse appliquée par move_characters, qui gère
        // les collisions (un dash s'arrête aux murs au lieu de les traverser)
        if let Some(step) = dash_state.take_step() {
            velocity.main = dash_state.dash_direction * (step / dt);
            continue;
        }

        // À terre (T1.3) : pas de sprint (la vitesse réduite vient du modificateur
        // `downed`, voir `combat::downed::Downed`, pas de ce multiplicateur-ci).
        let is_sprinting = !is_downed && (input.buttons & INPUT_SPRINT != 0);
        sprint_state.is_sprinting = is_sprinting;

        if is_sprinting {
            sprint_state.sprint_factor += movement.sprint_acceleration_per_frame;
            sprint_state.sprint_factor = sprint_state.sprint_factor.min(fixed_math::FIXED_ONE);
        } else {
            sprint_state.sprint_factor -= movement.sprint_deceleration_per_frame;
            sprint_state.sprint_factor = sprint_state.sprint_factor.max(fixed_math::FIXED_ZERO);
        }

        *facing_direction = get_facing_direction(&input);

        // Course : chaque axe va vers la vitesse visée (touches × vitesse max, sprint compris ;
        // nulle sans touche) en accélérant ou en freinant (`movement::run_velocity`).
        // L'adhérence (stat `Acceleration` résolue / config, ex. glace) règle les deux.
        let sprint_mult_stat = stats.get(
            entity,
            &StatId::SprintMultiplier,
            movement.sprint_multiplier,
        );
        let sprint_multiplier = fixed_math::FIXED_ONE
            + (sprint_mult_stat - fixed_math::FIXED_ONE) * sprint_state.sprint_factor;
        let acceleration = stats.get(entity, &StatId::Acceleration, movement.acceleration);
        let deceleration = movement.deceleration() * grip(acceleration, movement.acceleration);
        let target =
            movement_direction(&input).normalize_or_zero() * (move_speed * sprint_multiplier);
        velocity.main = run_velocity(velocity.main, target, acceleration * dt, deceleration * dt);
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
            Option<&DashState>,
        ),
        (With<Rollback>, With<Player>),
    >,
    settings: Res<CollisionSettings>,
    grids: Res<crate::collision_grid::CollisionGrids>,
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
    for (_net_id, mut transform, mut velocity, player_collider, collision_layer, dash_state) in
        order_mut_iter!(query)
    {
        let total_velocity = velocity.main + velocity.knockback;
        let delta_x = total_velocity.x * fixed_math::new(FIXED_TIMESTEP);
        let delta_y = total_velocity.y * fixed_math::new(FIXED_TIMESTEP);

        // Check for HARD collisions only (walls, not enemies). Grille spatiale (T2.1,
        // chantier B4b) : candidats des deux grilles (murs statiques + fenêtres/obstacles de
        // la grille dynamique — jamais les joueurs, exclus comme avant par le filtre
        // `Without<Player>` de `collider_query`, qui rejette silencieusement un candidat
        // joueur via `.get()`) autour de `pos`, au lieu de toute la carte. Enemies are "soft" -
        // player can push through them.
        let check_hard_collision = |pos: &fixed_math::FixedVec3| -> bool {
            let aabb = crate::collision_grid::collider_aabb(pos, player_collider);
            for entry in grids
                .walls
                .query_aabb(&aabb)
                .into_iter()
                .chain(grids.characters.query_aabb(&aabb))
            {
                let Ok((_target_entity, target_transform, target_collider, target_layer)) =
                    collider_query.get(entry.entity)
                else {
                    continue;
                };
                // Skip if layers don't collide
                if !settings.layer_matrix[collision_layer.0][target_layer.0] {
                    continue;
                }
                // Only walls are hard collisions (enemy layer is soft)
                if target_layer.0 == settings.enemy_layer {
                    continue;
                }
                if is_colliding(
                    pos,
                    player_collider,
                    &target_transform.translation,
                    target_collider,
                ) {
                    return true;
                }
            }
            false
        };

        // Count enemy collisions for slowdown effect. Only the dynamic grid can hold
        // enemy-layer colliders (walls never do).
        let count_enemy_collisions = |pos: &fixed_math::FixedVec3| -> u32 {
            let aabb = crate::collision_grid::collider_aabb(pos, player_collider);
            let mut count = 0u32;
            for entry in grids.characters.query_aabb(&aabb) {
                let Ok((_target_entity, target_transform, target_collider, target_layer)) =
                    collider_query.get(entry.entity)
                else {
                    continue;
                };
                if target_layer.0 != settings.enemy_layer {
                    continue;
                }
                if is_colliding(
                    pos,
                    player_collider,
                    &target_transform.translation,
                    target_collider,
                ) {
                    count += 1;
                }
            }
            count
        };

        // Apply slowdown based on enemy collisions (more enemies = slower). None during a
        // dash: it goes through the horde
        let dashing = dash_state.is_some_and(|dash| dash.is_dashing);
        let enemy_count = if dashing {
            0
        } else {
            count_enemy_collisions(&transform.translation)
        };
        let slowdown = if enemy_count > 0 {
            // Each enemy reduces speed by 20%, min 30% speed
            let factor = fixed_math::FIXED_ONE
                - fixed_math::new(0.2) * fixed_math::Fixed::from_num(enemy_count);
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

        // Full movement blocked by wall - try sliding (X, then Y from the X obtained: D39,
        // shared with enemies)
        let start_x = transform.translation.x;
        let start_y = transform.translation.y;
        let (slid, mut moved_x, mut moved_y) = crate::collider::slide_axes(
            transform.translation,
            delta_x,
            delta_y,
            &check_hard_collision,
        );
        transform.translation = slid;

        // Opening assist: moving mainly along one axis and blocked, while a small side offset
        // (at most NUDGE_MAX) would clear the way (e.g. a door, 32 units high, entered a few
        // units off): slide toward that side at the movement speed
        const NUDGE_MAX: i32 = 12;
        let z = transform.translation.z;
        let half = |v: fixed_math::Fixed| v.abs() / fixed_math::new(2.0);
        let try_nudge = |primary: (fixed_math::Fixed, fixed_math::Fixed),
                         side: (fixed_math::Fixed, fixed_math::Fixed)| {
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
            if let Some(sign) = try_nudge(
                (delta_x, fixed_math::FIXED_ZERO),
                (fixed_math::FIXED_ZERO, fixed_math::FIXED_ONE),
            ) {
                let side_pos = fixed_math::FixedVec3::new(
                    start_x,
                    start_y + delta_x.abs() * sign,
                    transform.translation.z,
                );
                if !check_hard_collision(&side_pos) {
                    transform.translation.y = side_pos.y;
                    moved_y = true;
                }
            }
        }
        if !moved_y && delta_y != fixed_math::FIXED_ZERO && delta_x.abs() <= half(delta_y) {
            if let Some(sign) = try_nudge(
                (fixed_math::FIXED_ZERO, delta_y),
                (fixed_math::FIXED_ONE, fixed_math::FIXED_ZERO),
            ) {
                let side_pos = fixed_math::FixedVec3::new(
                    start_x + delta_y.abs() * sign,
                    start_y,
                    transform.translation.z,
                );
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

pub fn update_animation_state(
    mut query: Query<(&GgrsNetId, &Velocity, &mut AnimationState), With<Rollback>>,
) {
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

#[cfg(test)]
mod movement_tests {
    use super::*;
    use crate::collider::{ColliderShape, Wall};
    use bevy::ecs::system::RunSystemOnce;
    use fixed_math::{Fixed, FixedTransform3D, FixedVec2, FixedVec3};

    #[test]
    fn sliding_on_both_axes_never_enters_a_wall_corner() {
        let mut world = World::new();
        world.init_resource::<bevy_ggrs::RollbackOrdered>();
        world.init_resource::<crate::collision_grid::CollisionGrids>();
        world.insert_resource(CollisionSettings::default());
        let mut transform = FixedTransform3D::IDENTITY;
        transform.translation =
            FixedVec3::new(Fixed::from_num(-2), Fixed::from_num(2), Fixed::ZERO);
        let body = Collider {
            shape: ColliderShape::Rectangle {
                width: Fixed::from_num(2),
                height: Fixed::from_num(2),
            },
            offset: FixedVec3::ZERO,
        };
        let player = world
            .spawn((
                GgrsNetId(1, "player".into()),
                Rollback,
                Player::default(),
                transform,
                body.clone(),
                CollisionLayer(3),
                Velocity {
                    main: FixedVec2::new(Fixed::from_num(120), Fixed::from_num(-120)),
                    knockback: FixedVec2::ZERO,
                },
            ))
            .id();
        let wall_pos = FixedVec3::new(Fixed::from_num(16), Fixed::from_num(-16), Fixed::ZERO);
        let wall = Collider {
            shape: ColliderShape::Rectangle {
                width: Fixed::from_num(32),
                height: Fixed::from_num(32),
            },
            offset: FixedVec3::ZERO,
        };
        let mut wall_transform = FixedTransform3D::IDENTITY;
        wall_transform.translation = wall_pos;
        world.spawn((
            GgrsNetId(2, "wall".into()),
            Rollback,
            Wall,
            wall_transform,
            wall.clone(),
            CollisionLayer(4),
        ));
        world
            .run_system_once(crate::collision_grid::maybe_rebuild_wall_grid)
            .unwrap();
        world.run_system_once(move_characters).unwrap();
        let after = world.get::<FixedTransform3D>(player).unwrap();
        assert!(
            !is_colliding(&after.translation, &body, &wall_pos, &wall),
            "two individually free axis moves must not combine into a collision"
        );
        assert!(
            after.translation.x > Fixed::from_num(-2),
            "the free axis still slides"
        );
    }
}
