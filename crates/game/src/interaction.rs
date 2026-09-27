use bevy::{log::{tracing::span, Level}, prelude::*};
use bevy_fixed::fixed_math;
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackApp};
use serde::{Deserialize, Serialize};
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter};

use crate::{
    collider::{Collider, CollisionLayer},
    core::AppState,
    frame_events::{FrameEvents, FrameEventsAppExt},
    system_set::RollbackSystemSet,
};

/// Component marker for the interaction prompt text UI
#[derive(Component)]
pub struct InteractionPromptText;

/// Resource that configures window repair behavior
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct WindowRepairConfig {
    /// Number of frames to wait between repairs
    pub repair_cooldown_frames: u32,
    /// Interaction range for repairing windows
    pub repair_range: fixed_math::Fixed,
}

impl Default for WindowRepairConfig {
    fn default() -> Self {
        Self {
            repair_cooldown_frames: 60, // 1 second at 60 FPS
            repair_range: fixed_math::new(50.0),
        }
    }
}

/// Component that marks an entity as interactable
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Interactable {
    /// Range within which interaction is possible
    pub interaction_range: fixed_math::Fixed,
    /// Type of interaction
    pub interaction_type: InteractionType,
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            interaction_range: fixed_math::new(50.0),
            interaction_type: InteractionType::Door,
        }
    }
}

/// Types of interactions available
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Reflect, PartialEq, Eq)]
pub enum InteractionType {
    Door,
    Window,
    // Future: Crate, Weapon, Soda, etc.
}

/// Component that marks an entity as capable of interacting
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct Interactor;

/// Interaction déclenchée par un joueur, consommée dans la même frame GGRS
/// (voir [`FrameEvents`]).
#[derive(Clone, Debug)]
pub struct InteractionEvent {
    /// The entity performing the interaction
    pub interactor: Entity,
    /// The GGRS net ID of the interactor (for deterministic logging)
    pub interactor_net_id: GgrsNetId,
    /// The entity being interacted with
    pub interactable: Entity,
    /// The type of interaction
    pub interaction_type: InteractionType,
    /// The GGRS net ID of the interactable (for deterministic lookup)
    pub interactable_net_id: GgrsNetId,
}

/// System that detects interactions within the GGRS schedule
pub fn interaction_detection_system(
    frame: Res<FrameCount>,
    mut event_writer: ResMut<FrameEvents<InteractionEvent>>,
    interactors: Query<
        (&GgrsNetId, Entity, &fixed_math::FixedTransform3D, &crate::character::player::input::InteractionInput),
        (With<Interactor>, With<Rollback>),
    >,
    interactables: Query<
        (
            &GgrsNetId,
            Entity,
            &fixed_math::FixedTransform3D,
            &Interactable,
            Option<&crate::collider::Collider>,
        ),
        With<Rollback>,
    >,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "interaction_detection_system");
    let _enter = system_span.enter();

    for (interactor_net_id, interactor_entity, interactor_transform, interaction_input) in order_iter!(interactors) {
        // Only process if the interaction button is being held
        if !interaction_input.is_holding {
            continue;
        }

        let interactor_pos = interactor_transform.translation;

        // Track the closest interactable within range
        let mut closest_interactable: Option<(fixed_math::FixedWide, GgrsNetId, Entity, InteractionType)> = None;

        // Check each interactable to find the closest one
        for (net_id, interactable_entity, interactable_transform, interactable, collider_opt) in
            order_iter!(interactables)
        {
            // Compute squared distance from the interactor to the interactable.
            // If the interactable has a collider, measure distance to the collider surface;
            // otherwise fall back to entity-center distance.
            let distance_sq: fixed_math::FixedWide = if let Some(collider) = collider_opt {
                point_to_collider_surface_distance_sq(
                    interactor_pos,
                    interactable_transform.translation,
                    collider,
                )
            } else {
                (interactable_transform.translation - interactor_pos).length_squared()
            };

            // Convert range to FixedWide for comparison
            // Direct conversion from Fixed to FixedWide to maintain precision
            let range_fw = fixed_math::FixedWide::from_num(interactable.interaction_range.to_num::<i64>());
            let range_sq_fw = range_fw.saturating_mul(range_fw);

            // If within range, check if this is the closest one
            if distance_sq <= range_sq_fw {
                match &closest_interactable {
                    None => {
                        // First interactable found
                        closest_interactable = Some((distance_sq, net_id.clone(), interactable_entity, interactable.interaction_type));
                    }
                    Some((closest_dist_sq, _, _, _)) => {
                        // Compare distances; if this one is closer, use it
                        if distance_sq < *closest_dist_sq {
                            closest_interactable = Some((distance_sq, net_id.clone(), interactable_entity, interactable.interaction_type));
                        }
                    }
                }
            }
        }

        // Only send interaction event for the closest interactable
        if let Some((distance_sq, net_id, interactable_entity, interaction_type)) = closest_interactable {
            let interaction_type_str = match interaction_type {
                InteractionType::Door => "Door",
                InteractionType::Window => "Window",
            };
            info!("{} interaction detected: interactor {} with {} ({}) at distance_sq {:?}", 
                  frame.as_ref(), interactor_net_id, net_id, interaction_type_str,
                  fixed_math::to_f32(fixed_math::Fixed::from_num(distance_sq.to_num::<f32>())));
            event_writer.send(InteractionEvent {
                interactor: interactor_entity,
                interactor_net_id: interactor_net_id.clone(),
                interactable: interactable_entity,
                interaction_type,
                interactable_net_id: net_id,
            });
        }
    }
}

// Helper: compute squared distance (FixedWide) from a point to the surface of a collider
fn point_to_collider_surface_distance_sq(
    point: fixed_math::FixedVec3,
    collider_pos: fixed_math::FixedVec3,
    collider: &crate::collider::Collider,
) -> fixed_math::FixedWide {
    use crate::collider::ColliderShape;

    // Apply collider offset to get the collider's actual world-center
    let collider_center = collider_pos + collider.offset;

    match &collider.shape {
        ColliderShape::Rectangle { width, height } => {
            let two = fixed_math::new(2.0);
            let half_w = width.saturating_div(two);
            let half_h = height.saturating_div(two);
            let closest_x = point.x.max(collider_center.x - half_w).min(collider_center.x + half_w);
            let closest_y = point.y.max(collider_center.y - half_h).min(collider_center.y + half_h);

            let diff = fixed_math::FixedVec2::new(point.x - closest_x, point.y - closest_y);
            diff.length_squared()
        }
        ColliderShape::Circle { radius } => {
            // Distance from point to circle center
            let diff = fixed_math::FixedVec2::new(point.x - collider_center.x, point.y - collider_center.y);
            let dist_sq_fw: fixed_math::FixedWide = diff.length_squared();

            // Convert radius to FixedWide
            let radius_fw = fixed_math::FixedWide::from_num(radius.to_num::<f32>());
            let radius_sq_fw = radius_fw.saturating_mul(radius_fw);

            if dist_sq_fw <= radius_sq_fw {
                // Inside the circle: distance to surface is zero
                fixed_math::FixedWide::from_num(0.0)
            } else {
                // distance_to_surface = sqrt(dist_sq) - radius
                let dist_fw = dist_sq_fw.sqrt();
                let d_surface = dist_fw.saturating_sub(radius_fw);
                d_surface.saturating_mul(d_surface)
            }
        }
    }

}

/// System that handles door interactions
pub fn handle_door_interaction(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    mut commands: Commands,
    door_query: Query<(Entity, &map::game::entity::map::door::DoorComponent), (With<Interactable>, With<Rollback>)>,
    all_doors_query: Query<(Entity, &GgrsNetId, &map::game::entity::map::door::DoorGridPosition), (With<map::game::entity::map::door::DoorComponent>, With<Rollback>)>,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "handle_door_interaction");
    let _enter = system_span.enter();

    for event in events.iter() {
        // Only handle door interactions
        if event.interaction_type != InteractionType::Door {
            continue;
        }

        // Verify the interactable entity exists and is a rollback entity
        if let Ok((door_entity, door_component)) = door_query.get(event.interactable) {
            info!(
                "{} door interaction triggered: interactor {} on door {}",
                frame.as_ref(), event.interactor_net_id, event.interactable_net_id
            );

            // Remove the collider from the door entity (in GGRS schedule)
            // This makes the door passable
            commands.entity(door_entity)
                .remove::<Collider>()
                .remove::<CollisionLayer>()
                .remove::<Interactable>();

            info!(
                "{} door {} components removed (Collider, CollisionLayer, Interactable)",
                frame.as_ref(), event.interactable_net_id
            );
            
            // If this door has a paired door, open it too
            if let Some((paired_level_iid, (paired_x, paired_y))) = &door_component.config.paired_door {
                // Find the paired door by matching level_iid and grid position
                // Note: We use iter() instead of order_iter! here since we're searching for a specific door
                // and the ordering doesn't matter for this lookup
                for (paired_door_entity, paired_net_id, paired_grid_pos) in all_doors_query.iter() {
                    // Match by level_iid and grid position
                    if &paired_grid_pos.level_iid == paired_level_iid 
                        && paired_grid_pos.grid_x == *paired_x 
                        && paired_grid_pos.grid_y == *paired_y {
                        info!(
                            "{} opening paired door {} at grid position ({}, {}) in level {}",
                            frame.as_ref(), paired_net_id, paired_x, paired_y, paired_level_iid
                        );
                        
                        // Remove components from paired door
                        commands.entity(paired_door_entity)
                            .remove::<Collider>()
                            .remove::<CollisionLayer>()
                            .remove::<Interactable>();
                        
                        break;
                    }
                }
            }
        } else {
            warn!(
                "InteractionEvent received for entity {:?} that is not a rollback interactable",
                event.interactable
            );
        }
    }
}

/// System that handles window repair interactions
pub fn handle_window_repair(
    frame: Res<FrameCount>,
    events: Res<FrameEvents<InteractionEvent>>,
    repair_config: Res<WindowRepairConfig>,
    mut window_query: Query<
        (
            Entity,
            &GgrsNetId,
            &mut map::game::entity::map::window::WindowHealth,
            Option<&mut crate::character::enemy::ai::obstacle::Obstacle>,
        ),
        (With<Interactable>, With<Rollback>),
    >,
) {
    let system_span = span!(Level::INFO, "ggrs", f = frame.frame, s = "handle_window_repair_system");
    let _enter = system_span.enter();

    // Events are delivered in deterministic order from interaction_detection_system,
    // which uses order_iter!
    for event in events.iter() {
        // Only handle window interactions
        if event.interaction_type != InteractionType::Window {
            continue;
        }

        info!(
            "{} [GGRS] window repair event received: interactor {} targeting window {}",
            frame.as_ref(),
            event.interactor_net_id,
            event.interactable_net_id
        );

        // Verify the interactable entity exists and is a rollback entity with WindowHealth
        if let Ok((window_entity, window_net_id, mut window_health, obstacle_opt)) =
            window_query.get_mut(event.interactable)
        {
            info!(
                "{} [GGRS] window {} state before repair: health={}/{}, cooldown_frame={:?}",
                frame.as_ref(),
                window_net_id,
                window_health.current,
                window_health.max,
                window_health.can_repair_after_frame
            );

            // Check if we can repair (cooldown check)
            if let Some(cooldown_frame) = window_health.can_repair_after_frame {
                if frame.frame < cooldown_frame {
                    info!(
                        "{} [GGRS] window {} repair BLOCKED: cooldown active until frame {} (current: {})",
                        frame.as_ref(),
                        window_net_id,
                        cooldown_frame,
                        frame.frame
                    );
                    continue;
                }
            }

            // Check if window is already at max health
            if window_health.current >= window_health.max {
                info!(
                    "{} [GGRS] window {} repair BLOCKED: already at max health {}/{}",
                    frame.as_ref(),
                    window_net_id,
                    window_health.current,
                    window_health.max
                );
                continue;
            }

            // Repair one health point
            let old_health = window_health.current;
            window_health.current += 1;
            let new_cooldown_frame = frame.frame + repair_config.repair_cooldown_frames;
            window_health.can_repair_after_frame = Some(new_cooldown_frame);

            // Garder l'Obstacle (utilisé par l'IA) aligné sur la santé réelle de la fenêtre
            if let Some(mut obstacle) = obstacle_opt {
                if obstacle.health == Some(0) {
                    obstacle.blocks_movement = true;
                }
                if obstacle.health.is_some() {
                    obstacle.health = Some(window_health.current as u32);
                }
            }

            info!(
                "{} [GGRS] window {} REPAIRED: health {}→{}/{}, cooldown set to frame {}, config_cooldown={}",
                frame.as_ref(),
                window_net_id,
                old_health,
                window_health.current,
                window_health.max,
                new_cooldown_frame,
                repair_config.repair_cooldown_frames
            );

            // If window is now at max health, it becomes solid again
            if window_health.current >= window_health.max {
                info!(
                    "{} [GGRS] window {} FULLY REPAIRED - should restore collision",
                    frame.as_ref(),
                    window_net_id
                );
            }


            info!(
                "{} [GGRS] window {} state after repair: health={}/{}, cooldown_frame={:?}, entity={:?}",
                frame.as_ref(),
                window_net_id,
                window_health.current,
                window_health.max,
                window_health.can_repair_after_frame,
                window_entity
            );
        } else {
            warn!(
                "{} [GGRS] window repair FAILED: entity {:?} (net_id {}) not found or not a valid window",
                frame.as_ref(),
                event.interactable,
                event.interactable_net_id
            );
        }
    }
}

/// Plugin for the interaction system
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_frame_events::<InteractionEvent>();

        // Initialize window repair config
        app.init_resource::<WindowRepairConfig>();

        // Register rollback components
        app.rollback_component_with_clone::<Interactable>()
            .rollback_component_with_clone::<Interactor>()
            .rollback_component_with_clone::<map::game::entity::map::window::WindowHealth>()
            .rollback_resource_with_clone::<WindowRepairConfig>()
            .rollback_component_with_clone::<crate::character::player::input::InteractionInput>();

        // Add interaction detection to GGRS schedule
        // This runs after input processing but before movement
        // Note: handle_zombie_window_damage removed - window damage now handled by
        // process_obstacle_damage via ObstacleAttackEvent from enemy_attack_system
        app.add_systems(
            GgrsSchedule,
            (
                interaction_detection_system,
                handle_door_interaction,
                handle_window_repair,
            )
                .chain()
                .after(RollbackSystemSet::Input)
                .before(RollbackSystemSet::Movement)
                .in_set(RollbackSystemSet::Interaction),
        );

        // Setup UI on entering InGame state
        app.add_systems(OnEnter(AppState::InGame), setup_interaction_ui);

        // Add visual feedback systems (outside GGRS schedule)
        app.add_systems(
            Update,
            (
                update_door_visuals,
                update_window_health_bars,
                display_interaction_prompts,
            ).run_if(in_state(AppState::InGame)),
        );
    }
}

/// Cache l'entité visuelle (LDtk) des portes ouvertes.
/// Hors GGRS : l'état est lu à chaque frame, donc il reste juste après un rollback
/// (une porte dont l'ouverture est annulée redevient visible).
pub fn update_door_visuals(
    doors: Query<
        (&map::game::entity::MapRollbackItem, Has<Interactable>),
        With<map::game::entity::map::door::DoorComponent>,
    >,
    mut visibilities: Query<&mut Visibility>,
) {
    for (rollback_item, closed) in doors.iter() {
        let target = if closed { Visibility::Inherited } else { Visibility::Hidden };
        if let Ok(mut visibility) = visibilities.get_mut(rollback_item.parent) {
            visibility.set_if_neq(target);
        }
    }
}

/// Component marker for window health bar
#[derive(Component)]
pub struct WindowHealthBar;

/// Ajuste la barre de vie des fenêtres à leur santé actuelle (réparations comme dégâts).
/// Hors GGRS, dérivé de l'état : reste juste après un rollback.
pub fn update_window_health_bars(
    windows: Query<(
        &map::game::entity::MapRollbackItem,
        &map::game::entity::map::window::WindowHealth,
    )>,
    children_query: Query<&Children>,
    mut health_bar_query: Query<&mut Sprite, With<WindowHealthBar>>,
) {
    for (rollback_item, window_health) in windows.iter() {
        let Ok(children) = children_query.get(rollback_item.parent) else {
            continue;
        };
        let health_ratio = window_health.current as f32 / window_health.max.max(1) as f32;
        let size = Some(Vec2::new(16.0 * health_ratio, 2.0));
        for child in children.iter() {
            if let Ok(mut sprite) = health_bar_query.get_mut(child) {
                if sprite.custom_size != size {
                    sprite.custom_size = size;
                }
                break;
            }
        }
    }
}

/// Setup the interaction prompt UI text
fn setup_interaction_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        InteractionPromptText,
        Text::new(""),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(12.0),
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0), // Position in the upper-middle of the screen
            left: Val::Percent(50.0),
            ..default()
        },
    ));
}

/// System that displays interaction prompts when player is near interactable
/// Only shows prompts for local players (supports split-screen multiplayer)
pub fn display_interaction_prompts(
    mut gizmos: Gizmos,
    mut text_query: Query<&mut Text, With<InteractionPromptText>>,
    local_interactors: Query<
        &fixed_math::FixedTransform3D,
        (With<Interactor>, With<Rollback>, With<crate::character::player::LocalPlayer>),
    >,
    interactables: Query<
        (
            Entity,
            &fixed_math::FixedTransform3D,
            &Interactable,
            Option<&map::game::entity::map::door::DoorComponent>,
            Option<&map::game::entity::map::window::WindowHealth>,
        ),
        (Without<Interactor>, With<Rollback>),
    >,
) {
    // Track the closest door across all LOCAL players
    // Store: (distance, cost, position, range)
    let mut closest_door_info: Option<(f32, i32, Vec3, f32)> = None;
    // Track the closest window
    // Store: (distance, current_health, max_health, position, range)
    let mut closest_window_info: Option<(f32, u8, u8, Vec3, f32)> = None;
    
    // Only check local players
    for interactor_transform in local_interactors.iter() {
        for (_interactable_entity, interactable_transform, interactable, door_component_opt, window_health_opt) in interactables.iter() {
            // Calculate distance
            let distance_vec = interactable_transform.translation - interactor_transform.translation;
            let distance_sq: fixed_math::FixedWide = distance_vec.length_squared();
            
            // Convert range to FixedWide for comparison
            let range_fw = fixed_math::FixedWide::from_num(interactable.interaction_range.to_num::<i64>());
            let range_sq_fw = range_fw.saturating_mul(range_fw);

            // If within range, check what type of interactable it is
            if distance_sq <= range_sq_fw {
                let distance = fixed_math::to_f32(fixed_math::Fixed::from_num(distance_sq.to_num::<f32>().sqrt()));
                let pos = Vec3::new(
                    fixed_math::to_f32(interactable_transform.translation.x),
                    fixed_math::to_f32(interactable_transform.translation.y),
                    fixed_math::to_f32(interactable_transform.translation.z),
                );
                let interaction_range = fixed_math::to_f32(interactable.interaction_range);

                // Check if it's a door
                if let Some(door_component) = door_component_opt {
                    match &closest_door_info {
                        None => {
                            closest_door_info = Some((distance, door_component.config.cost, pos, interaction_range));
                        }
                        Some((closest_dist, _, _, _)) => {
                            if distance < *closest_dist {
                                closest_door_info = Some((distance, door_component.config.cost, pos, interaction_range));
                            }
                        }
                    }
                }

                // Check if it's a window
                if let Some(window_health) = window_health_opt {
                    match &closest_window_info {
                        None => {
                            closest_window_info = Some((distance, window_health.current, window_health.max, pos, interaction_range));
                        }
                        Some((closest_dist, _, _, _, _)) => {
                            if distance < *closest_dist {
                                closest_window_info = Some((distance, window_health.current, window_health.max, pos, interaction_range));
                            }
                        }
                    }
                }
            }
        }
    }

    // Priority: show door prompt if there's a door nearby, otherwise show window prompt
    if let Some((_distance, cost, door_pos, interaction_range)) = closest_door_info {
        // Draw outer range circle in yellow with low opacity
        gizmos.circle(
            Isometry3d::from_translation(door_pos),
            interaction_range,
            Color::srgba(1.0, 1.0, 0.0, 0.3),
        );
        
        // Update the text UI
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = format!("Press H to open door (Cost: {})", cost);
        }
    } else if let Some((_distance, current_health, max_health, window_pos, interaction_range)) = closest_window_info {
        // Draw outer range circle in green with low opacity for windows
        gizmos.circle(
            Isometry3d::from_translation(window_pos),
            interaction_range,
            Color::srgba(0.0, 1.0, 0.0, 0.3),
        );
        
        // Update the text UI
        if let Ok(mut text) = text_query.single_mut() {
            if current_health < max_health {
                text.0 = format!("Press H to repair window ({}/{})", current_health, max_health);
            } else {
                text.0 = format!("Window fully repaired ({}/{})", current_health, max_health);
            }
        }
    } else {
        // Clear the text if no interactable is in range for any local player
        if let Ok(mut text) = text_query.single_mut() {
            text.0.clear();
        }
    }
}
