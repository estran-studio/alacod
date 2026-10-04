use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::{LdtkProjectHandle, LevelIid};
use bevy_fixed::fixed_math;
use bevy_ggrs::Rollback;
use game::{
    character::enemy::ai::Obstacle,
    character::enemy::spawning::EnemySpawnerState,
    collider::{Collider, CollisionLayer, CollisionSettings, Wall, Window},
    core::AppState,
};
use map::game::entity::{
    map::{
        door::{DoorComponent, DoorGridPosition},
        enemy_spawn::EnemySpawnerComponent,
        level_id::LevelId,
        map_rollback::MapRollbackMarker,
    },
    MapRollbackItem,
};
use map::generation::entity::door::DoorConfig;
use utils::net_id::GgrsNetIdFactory;

use super::floors::{compute_floor_plan, FloorPlan, FloorSlots, FloorWorldsReady};
use crate::{
    game::{
        collider::create_wall_colliders_from_ldtk, entity::door::LdtkEntitySize,
        utility::load_levels_if_not_present,
    },
    loader::{get_asset_loader_generation, setup_generated_map},
};

pub struct LdtkMapLoadingPlugin;

#[derive(Clone)]
pub struct LdtkMapEntityLoading {
    pub id: String,
    pub kind: String,
    pub global_transform: GlobalTransform,
    pub entity: Entity,
    pub sprite_size: Option<Vec2>,
    pub door_config: Option<DoorConfig>,
    pub door_grid_position: Option<DoorGridPosition>,
    pub spawner_config: Option<EnemySpawnerComponent>,
    pub level_id: Option<LevelId>,
    /// Emplacement du monde LDtk de l'entité dans la séquence du mode `Floors` (T1.8,
    /// [`FloorWorld`]) ; `0` pour la carte unique des autres modes.
    pub slot: usize,
}

#[derive(Resource, Clone)]
pub struct LdtkMapEntityLoadingRegistry {
    pub entities: Vec<LdtkMapEntityLoading>,
    pub registered_entities: std::collections::HashSet<Entity>,
    pub last_update_time: f32,
    pub timeout_duration: f32,
    pub loading_complete: bool,
    // Number of update ticks since we last saw a new entity registered.
    pub frames_since_last_update: u32,
    // How many consecutive frames with no new entities we consider "stable" (default a few frames).
    pub required_stable_frames: u32,
}

impl Default for LdtkMapEntityLoadingRegistry {
    fn default() -> Self {
        Self {
            entities: vec![],
            registered_entities: std::collections::HashSet::new(),
            last_update_time: 0.0,
            timeout_duration: 0.5,
            loading_complete: false,
            frames_since_last_update: 0,
            required_stable_frames: 3,
        }
    }
}

#[derive(Event, Message, Default, Debug)]
pub struct LdtkMapLoadingEvent;

impl Plugin for LdtkMapLoadingPlugin {
    fn build(&self, app: &mut App) {
        let level_loader = get_asset_loader_generation();

        app.register_asset_loader(level_loader);

        app.init_resource::<LdtkMapEntityLoadingRegistry>();
        app.init_resource::<crate::loader::MapLoaderSettings>();
        app.add_message::<LdtkMapLoadingEvent>();

        // T2.4, chantier F1 : `GameLoading` peut être ré-entré par une relance
        // (`RunRequest::Restart`, `game::run_state`), pas seulement au premier
        // chargement — remettre `LdtkMapEntityLoadingRegistry` à zéro avant
        // `setup_generated_map`, sinon `wait_for_all_map_rollback_entity` la voit déjà
        // `loading_complete` (partie précédente) et ne relit plus jamais les entités de la
        // nouvelle carte (`LdtkMapLoadingEvent` jamais réémis, la partie reste bloquée en
        // `GameLoading`).
        // T1.8 : `compute_floor_plan` décide (mode `Floors` ou non) avant le chargement des
        // cartes, qui en dépend (un monde LDtk par niveau, voir `super::floors`).
        app.add_systems(
            OnEnter(AppState::GameLoading),
            (
                reset_map_loading_registry,
                compute_floor_plan,
                setup_generated_map,
            )
                .chain(),
        );
        app.add_plugins(super::floors::FloorsPlugin);
        // T1.6 : terrain destructible des cavernes (`CellGrid` au chargement, murs et
        // navigation après destruction).
        app.add_plugins(super::cave::CavePlugin);
        // Deterministic order at the end of map loading: door level iids, then map entity
        // ids (this system also sends LdtkMapLoadingEvent), then walls, then players (see
        // MapNetIdAssignment)
        app.add_systems(
            Update,
            (
                load_levels_if_not_present,
                populate_door_level_iids,
                wait_for_all_map_rollback_entity
                    .after(populate_door_level_iids)
                    .in_set(MapNetIdAssignment),
            )
                .run_if(in_state(AppState::GameLoading)),
        );

        // Transition from GameLoading to GameStarting when map loading is complete
        app.add_systems(
            Update,
            transition_to_game_starting.run_if(on_message::<LdtkMapLoadingEvent>),
        );

        app.add_systems(
            Update,
            create_wall_colliders_from_ldtk
                .run_if(on_message::<LdtkMapLoadingEvent>)
                .after(wait_for_all_map_rollback_entity)
                .in_set(MapNetIdAssignment),
        );

        // T2.4, chantier F1 : détruit l'arbre LDtk (niveaux, calques, tuiles, et toute
        // entité LDtk encore dessous — portes, fenêtres, spawners, `WeaponLocation`/
        // `SodaLocation`...) à la sortie d'`InGame`, quelle qu'en soit la cause (relance ou
        // retour au lobby, voir `game::run_state`). `game` ne dépend pas de
        // `bevy_ecs_ldtk` : ce nettoyage-ci vit dans `map_ldtk`, sur le même hook d'état
        // (`OnExit(AppState::InGame)`) que `game::run_state::cleanup_rollback_world_system`
        // (entités `Rollback`, session GGRS) — les deux sont indépendants, sans appel direct
        // entre les deux crates.
        app.add_systems(OnExit(AppState::InGame), despawn_ldtk_world_on_exit_ingame);
    }
}

/// Voir la doc de `LdtkMapLoadingPlugin::build` (`OnEnter(AppState::GameLoading)`).
fn reset_map_loading_registry(mut registry: ResMut<LdtkMapEntityLoadingRegistry>) {
    *registry = LdtkMapEntityLoadingRegistry::default();
}

/// Voir la doc de `LdtkMapLoadingPlugin::build` (`OnExit(AppState::InGame)`). `try_despawn`
/// (pas `despawn`) pour la même raison que
/// `game::run_state::cleanup_rollback_world_system` : un despawn recursif est sûr même si
/// une autre entité de la hiérarchie a déjà été détruite par ailleurs cette même frame.
fn despawn_ldtk_world_on_exit_ingame(
    mut commands: Commands,
    worlds: Query<Entity, With<LdtkProjectHandle>>,
) {
    for entity in &worlds {
        commands.entity(entity).try_despawn();
    }
}

/// Systèmes qui attribuent des `GgrsNetId` aux entités de la map à la fin de son chargement :
/// entités de la map (qui émet aussi `LdtkMapLoadingEvent`), puis murs. Tout système qui crée
/// des entités rollback en réponse à cet événement (ex. les joueurs) doit être ordonné
/// `.after(MapNetIdAssignment)` : sinon l'ordre d'exécution, et donc la numérotation, varie
/// d'un client à l'autre (et change dès qu'on ajoute un système).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapNetIdAssignment;

/// System to populate the level_iid in DoorGridPosition from the parent level entity
fn populate_door_level_iids(
    mut door_query: Query<(&ChildOf, &mut DoorGridPosition), With<MapRollbackMarker>>,
    parent_query: Query<&ChildOf>,
    level_query: Query<&LevelIid>,
) {
    for (door_parent, mut grid_pos) in door_query.iter_mut() {
        // If level_iid is empty, try to get it from parent hierarchy
        if grid_pos.level_iid.is_empty() {
            // Access the entity through the ChildOf component
            let parent_entity = door_parent.get();

            // First check if the immediate parent is a level
            if let Ok(level_iid) = level_query.get(parent_entity) {
                grid_pos.level_iid = level_iid.to_string();
            } else if let Ok(grandparent) = parent_query.get(parent_entity) {
                // Check if the grandparent (parent's parent) is a level
                let grandparent_entity = grandparent.get();
                if let Ok(level_iid) = level_query.get(grandparent_entity) {
                    grid_pos.level_iid = level_iid.to_string();
                }
            }
        }
    }
}

fn wait_for_all_map_rollback_entity(
    mut commands: Commands,
    mut entity_registery: ResMut<LdtkMapEntityLoadingRegistry>,
    mut ev_loading_map: MessageWriter<LdtkMapLoadingEvent>,

    query_map_entity: Query<
        (
            Entity,
            &GlobalTransform,
            &MapRollbackMarker,
            Option<&LdtkEntitySize>,
            Option<&DoorComponent>,
            Option<&DoorGridPosition>,
            Option<&EnemySpawnerComponent>,
            Option<&ChildOf>,
        ),
        With<MapRollbackMarker>,
    >,

    collision_settings: Res<CollisionSettings>,

    mut id_factory: ResMut<GgrsNetIdFactory>,

    time: Res<Time>,
    level_query: Query<&LevelIid>,
    slots: FloorSlots,
    plan: Option<Res<FloorPlan>>,
    floor_worlds: FloorWorldsReady,
) {
    if entity_registery.loading_complete {
        return;
    }
    // T1.8 : en mode `Floors`, plusieurs mondes LDtk se chargent en parallèle ; ne commencer
    // à compter les frames stables qu'une fois tous leurs niveaux apparus (sinon un monde en
    // retard serait oublié). Hors `Floors` : comportement inchangé.
    if let Some(plan) = &plan {
        if !floor_worlds.all_spawned(plan) {
            return;
        }
    }

    let current_time = time.elapsed_secs();
    let previous_size = entity_registery.entities.len();

    // Collect and sort entities by their marker name and position for deterministic order
    let mut entities_to_process: Vec<_> = query_map_entity
        .iter()
        .filter(|(e, _, _, _, _, _, _, _)| !entity_registery.registered_entities.contains(e))
        .collect();

    // Sort by marker name first, then by position (x, y) for determinism
    entities_to_process.sort_by(|a, b| {
        let name_cmp = a.2 .0.cmp(&b.2 .0);
        if name_cmp != std::cmp::Ordering::Equal {
            return name_cmp;
        }
        // If names are equal, sort by position
        let pos_a = a.1.translation();
        let pos_b = b.1.translation();
        pos_a
            .x
            .partial_cmp(&pos_b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                pos_a
                    .y
                    .partial_cmp(&pos_b.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    for (
        e,
        global_transform,
        rollback_marker,
        ldtk_size,
        door_component,
        door_grid_pos,
        spawner_component,
        parent,
    ) in entities_to_process
    {
        // Skip if already registered (should not happen due to filter above, but keeping for safety)
        if entity_registery.registered_entities.contains(&e) {
            continue;
        }

        let translation = global_transform.translation();

        // Determine the LevelIid of the current entity by checking its parent
        let mut entity_level_id: Option<LevelId> = None;
        if let Some(parent_component) = parent {
            if let Ok(level_iid) = level_query.get(parent_component.get()) {
                entity_level_id = Some(LevelId(level_iid.to_string()));
            }
        }

        // Only register entities with valid (non-zero) global transforms
        // GlobalTransform gets updated by Bevy's transform propagation system
        if translation.x != 0.0 || translation.y != 0.0 {
            let sprite_size = ldtk_size.map(|s| Vec2::new(s.width, s.height));
            let door_config = door_component.map(|dc| dc.config.clone());
            let door_grid_position = door_grid_pos.cloned();
            let spawner_config = spawner_component.cloned();
            info!(
                "Found {} entity {:?} at position {} with LDTK size {:?} and door config {:?}",
                rollback_marker.0, e, translation, sprite_size, door_config
            );

            entity_registery.entities.push(LdtkMapEntityLoading {
                id: rollback_marker.0.clone(),
                kind: rollback_marker.0.clone(),
                entity: e.clone(),
                global_transform: *global_transform,
                sprite_size,
                door_config,
                door_grid_position,
                spawner_config,
                level_id: entity_level_id,
                slot: slots.slot_of(e),
            });
            entity_registery.registered_entities.insert(e);
        }
    }

    let new_size = entity_registery.entities.len();

    // If new entities were added, update the last update time
    if new_size > previous_size {
        entity_registery.last_update_time = current_time;
        entity_registery.frames_since_last_update = 0;
        info!(
            "Added {} new map entities, total: {}",
            new_size - previous_size,
            new_size
        );
        return;
    }

    // No new entities were added this frame: increment stable-frame counter
    entity_registery.frames_since_last_update =
        entity_registery.frames_since_last_update.saturating_add(1);

    // Consider loading complete when either:
    //  - we've observed a few consecutive frames with no new entities (stable), or
    //  - the original timeout has elapsed (fallback for unusual scheduling scenarios).
    if !entity_registery.entities.is_empty()
        && (entity_registery.frames_since_last_update >= entity_registery.required_stable_frames
            || (current_time - entity_registery.last_update_time)
                >= entity_registery.timeout_duration)
    {
        // Les entités arrivent sur plusieurs frames selon le chargement LDtk : trier le
        // registre complet pour que les GgrsNetId ne dépendent pas de ce timing
        entity_registery.entities.sort_by(|a, b| {
            let pos_a = a.global_transform.translation();
            let pos_b = b.global_transform.translation();
            a.id.cmp(&b.id)
                .then_with(|| pos_a.x.total_cmp(&pos_b.x))
                .then_with(|| pos_a.y.total_cmp(&pos_b.y))
        });

        // T1.8 : seules les entités du premier niveau (emplacement 0) ; les autres niveaux
        // du mode `Floors` sont créés au passage du portail (`super::floors`), depuis ce même
        // registre (trié, figé après le chargement).
        spawn_map_items(
            &mut commands,
            entity_registery
                .entities
                .iter()
                .filter(|item| item.slot == 0),
            &mut id_factory,
            &collision_settings,
            true,
        );

        ev_loading_map.write_default();
        entity_registery.loading_complete = true;
    }
}

/// Crée les entités rollback des entités de carte (`door`, `window`, `enemy_spawn`...) du
/// registre, dans l'ordre donné (trié par le registre : numérotation déterministe).
/// `window_bars` : ajoute la barre de vie (visuel, hors rollback) sous l'entité LDtk d'une
/// fenêtre — `false` pendant la simulation (passage de niveau du mode `Floors`, où un visuel
/// créé dans `GgrsSchedule` serait dupliqué par un rollback ; `super::floors` les ajoute
/// dans `Update`).
pub(crate) fn spawn_map_items<'a>(
    commands: &mut Commands,
    items: impl Iterator<Item = &'a LdtkMapEntityLoading>,
    id_factory: &mut GgrsNetIdFactory,
    collision_settings: &CollisionSettings,
    window_bars: bool,
) {
    for item in items {
        let rollback_item = MapRollbackItem::new(item.entity.clone(), item.kind.clone());
        let id = id_factory.next(item.id.clone());

        // Use the exact world position from the LDTK-spawned entity.
        // bevy_ecs_ldtk already applies pivot and coordinate system conversions,
        // so using the GlobalTransform directly keeps visuals and physics aligned.
        let world_position = item.global_transform.translation();
        let transform = Transform::from_translation(world_position);
        let fixed_transform = fixed_math::FixedTransform3D::from_bevy_transform(&transform);

        info!(
            "spawning rollback map item {} at {} (fixed: {:?}) for parent {}",
            id, world_position, fixed_transform.translation, item.entity
        );
        let mut cmd = commands.spawn((fixed_transform, rollback_item, id));

        if let Some(level_id) = &item.level_id {
            cmd.insert(level_id.clone());
        }

        match item.kind.as_str() {
            "door" => {
                // Use sprite size if available, otherwise fall back to default size
                let (width, height) = if let Some(size) = item.sprite_size {
                    (size.x, size.y)
                } else {
                    info!("No sprite size for door, using default 64x32");
                    (64.0, 32.0)
                };

                let max_dimension = width.max(height);
                let interaction_range = max_dimension;

                // Get the DoorConfig from the LDTK entity, or use default
                let door_config = item.door_config.clone().unwrap_or_default();

                // Get the DoorGridPosition if available
                let door_grid_position = item.door_grid_position.clone();

                cmd.insert((
                    Wall,
                    DoorComponent {
                        config: door_config.clone(),
                    },
                    Collider {
                        shape: game::collider::ColliderShape::Rectangle {
                            width: fixed_math::Fixed::from_num(width),
                            height: fixed_math::Fixed::from_num(height),
                        },
                        offset: fixed_math::FixedVec3::ZERO,
                    },
                    CollisionLayer(collision_settings.wall_layer),
                ));

                // Add grid position if available
                if let Some(grid_pos) = door_grid_position {
                    cmd.insert(grid_pos);
                }

                // Only add Interactable component if the door is actually interactable
                if door_config.interactable {
                    cmd.insert(game::interaction::Interactable {
                        interaction_range: fixed_math::new(interaction_range),
                        interaction_type: game::interaction::InteractionType::Door,
                    });
                    info!("adding collider to door entity with size {}x{}, interaction range {}, and config {:?}", 
                          width, height, interaction_range, door_config);
                } else {
                    info!("adding collider to NON-INTERACTABLE door entity with size {}x{} and config {:?}", 
                          width, height, door_config);
                }
            }
            "window" => {
                // Use sprite size if available, otherwise fall back to default size
                let (width, height) = if let Some(size) = item.sprite_size {
                    (size.x, size.y)
                } else {
                    info!("No sprite size for window, using default 16x16");
                    (16.0, 16.0)
                };

                let max_dimension = width.max(height);
                let interaction_range = max_dimension * 0.8; // Smaller range for windows - need to be close

                cmd.insert((
                    Window,
                    Obstacle::window(), // For flow field pathfinding (GroundBreaker can pass)
                    Collider {
                        shape: game::collider::ColliderShape::Rectangle {
                            width: fixed_math::Fixed::from_num(width),
                            height: fixed_math::Fixed::from_num(height),
                        },
                        offset: fixed_math::FixedVec3::ZERO,
                    },
                    CollisionLayer(collision_settings.window_layer),
                    map::game::entity::map::window::WindowHealth {
                        current: 3,
                        max: 3,
                        can_repair_after_frame: None,
                    },
                    game::interaction::Interactable {
                        interaction_range: fixed_math::new(interaction_range),
                        interaction_type: game::interaction::InteractionType::Window,
                    },
                ));
                info!("adding collider and Obstacle to window entity with size {}x{}, interaction range {}",
                      width, height, interaction_range);
            }
            "enemy_spawn" => {
                // Get spawner config from LDTK entity, or use default
                let spawner = item.spawner_config.clone().unwrap_or_default();

                cmd.insert((spawner, EnemySpawnerState::default()));
                info!("adding enemy spawner at {:?}", world_position);
            }
            _ => {}
        }

        // Register the entity with GGRS rollback system
        let _rollback_entity = cmd.insert(Rollback).id();

        // Add window-specific visual children
        // Note: This must be done after the Rollback marker is inserted to avoid mutable borrow conflicts
        // since add_children() requires exclusive access to Commands
        if window_bars && item.kind.as_str() == "window" {
            commands.entity(item.entity).with_children(|parent| {
                parent.spawn((
                    game::interaction::WindowHealthBar,
                    Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0),      // Green health bar
                        custom_size: Some(Vec2::new(0.0, 2.0)), // Start at 0 width (0 health), smaller height
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(0.0, 8.0, 0.1)), // Closer to window
                ));
            });
            info!("Added health bar to window entity {:?}", item.entity);
        }
    }
}

/// System to transition from GameLoading to GameStarting when the map finishes loading
fn transition_to_game_starting(
    mut app_state: ResMut<NextState<AppState>>,
    mut ev_map_loaded: MessageReader<LdtkMapLoadingEvent>,
) {
    for _event in ev_map_loaded.read() {
        info!("Map loading complete, transitioning to GameStarting state");
        app_state.set(AppState::GameStarting);
    }
}
