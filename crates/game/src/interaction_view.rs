//! Présentation des interactions (D69 étape 2, E5c) : image des portes ouvertes, barre de vie des
//! fenêtres et cercle de portée de l'interactable proche. Dérivé de l'état dans `Update`, jamais lu
//! par la simulation ; absent en headless (`PresentationPlugin`).

use bevy::prelude::*;
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use combat::downed::Downed;

use crate::{
    collider::Collider,
    interaction::{Interactable, InteractionType, Interactor},
    session::AppState,
};

/// Systèmes de présentation des interactions, ajouté par `PresentationPlugin`.
pub struct InteractionPresentationPlugin;

impl Plugin for InteractionPresentationPlugin {
    fn build(&self, app: &mut App) {
        // Add visual feedback systems (outside GGRS schedule)
        app.add_systems(
            Update,
            (
                update_door_visuals,
                update_window_health_bars,
                display_interaction_prompts,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Cache l'entité visuelle (LDtk) des portes ouvertes.
/// Hors GGRS : l'état est lu à chaque frame, donc il reste juste après un rollback
/// (une porte dont l'ouverture est annulée redevient visible).
/// Une porte est ouverte quand elle n'a plus de collider : une porte non interactive
/// n'a pas d'`Interactable` mais reste fermée.
pub fn update_door_visuals(
    doors: Query<
        (
            &map::game::entity::MapRollbackItem,
            Has<Collider>,
            Has<crate::world_sprites::DoorSprite>,
        ),
        With<map::game::entity::map::door::DoorComponent>,
    >,
    mut visibilities: Query<&mut Visibility>,
) {
    for (rollback_item, closed, has_image) in doors.iter() {
        // Une porte qui porte sa propre image d'état (`world_sprites`, enfant de cette entité) reste
        // visible : l'image « ouverte » ne doit pas être cachée avec son parent ; la tuile d'origine est
        // transparente.
        let target = if closed || has_image {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
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

/// Cercle de portée autour de l'interactable proche d'un joueur local (porte, joueur à
/// réanimer, fenêtre). Le texte du prompt (« Ouvrir — $750 », « Acheter … ») est dans le HUD
/// depuis T2.12 (source `prompt`, `ui::hud`), qui couvre aussi armes et perks ; ce système
/// ne dessine plus que les cercles.
pub fn display_interaction_prompts(
    mut gizmos: Gizmos,
    local_interactors: Query<
        &fixed_math::FixedTransform3D,
        (
            With<Interactor>,
            With<Rollback>,
            With<crate::character::player::LocalPlayer>,
            // À terre (T1.3) : un joueur à terre ne peut interagir avec rien (voir
            // `interaction_detection_system`), le prompt ne doit donc pas lui être montré.
            Without<Downed>,
        ),
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
    // À terre (T1.3) : joueur le plus proche à réanimer. Store: (distance, position, range)
    let mut closest_revive_info: Option<(f32, Vec3, f32)> = None;

    // Only check local players
    for interactor_transform in local_interactors.iter() {
        for (
            _interactable_entity,
            interactable_transform,
            interactable,
            door_component_opt,
            window_health_opt,
        ) in interactables.iter()
        {
            // Calculate distance
            let distance_vec =
                interactable_transform.translation - interactor_transform.translation;
            let distance_sq: fixed_math::FixedWide = distance_vec.length_squared();

            // Convert range to FixedWide for comparison
            let range_fw =
                fixed_math::FixedWide::from_num(interactable.interaction_range.to_num::<i64>());
            let range_sq_fw = range_fw.saturating_mul(range_fw);

            // If within range, check what type of interactable it is
            if distance_sq <= range_sq_fw {
                let distance = fixed_math::to_f32(fixed_math::Fixed::from_num(
                    distance_sq.to_num::<f32>().sqrt(),
                ));
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
                            closest_door_info = Some((
                                distance,
                                door_component.config.cost,
                                pos,
                                interaction_range,
                            ));
                        }
                        Some((closest_dist, _, _, _)) => {
                            if distance < *closest_dist {
                                closest_door_info = Some((
                                    distance,
                                    door_component.config.cost,
                                    pos,
                                    interaction_range,
                                ));
                            }
                        }
                    }
                }

                // À terre (T1.3) : joueur à réanimer.
                if interactable.interaction_type == InteractionType::Revive {
                    match &closest_revive_info {
                        None => closest_revive_info = Some((distance, pos, interaction_range)),
                        Some((closest_dist, _, _)) if distance < *closest_dist => {
                            closest_revive_info = Some((distance, pos, interaction_range));
                        }
                        _ => {}
                    }
                }

                // Check if it's a window
                if let Some(window_health) = window_health_opt {
                    match &closest_window_info {
                        None => {
                            closest_window_info = Some((
                                distance,
                                window_health.current,
                                window_health.max,
                                pos,
                                interaction_range,
                            ));
                        }
                        Some((closest_dist, _, _, _, _)) => {
                            if distance < *closest_dist {
                                closest_window_info = Some((
                                    distance,
                                    window_health.current,
                                    window_health.max,
                                    pos,
                                    interaction_range,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Priority: door, then revive (T1.3), then window.
    if let Some((_distance, _cost, door_pos, interaction_range)) = closest_door_info {
        // Draw outer range circle in yellow with low opacity
        gizmos.circle(
            Isometry3d::from_translation(door_pos),
            interaction_range,
            Color::srgba(1.0, 1.0, 0.0, 0.3),
        );
    } else if let Some((_distance, revive_pos, interaction_range)) = closest_revive_info {
        gizmos.circle(
            Isometry3d::from_translation(revive_pos),
            interaction_range,
            Color::srgba(0.0, 0.6, 1.0, 0.3),
        );
    } else if let Some((_distance, _current, _max, window_pos, interaction_range)) =
        closest_window_info
    {
        // Draw outer range circle in green with low opacity for windows
        gizmos.circle(
            Isometry3d::from_translation(window_pos),
            interaction_range,
            Color::srgba(0.0, 1.0, 0.0, 0.3),
        );
    }
}
