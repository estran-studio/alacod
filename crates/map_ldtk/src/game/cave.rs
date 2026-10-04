//! Cavernes (T1.6, `docs/conventions.md` §21) : branchement ECS du terrain destructible.
//!
//! - Au chargement (`LdtkMapLoadingEvent`), `CellGrid` reçoit la grille de la caverne de
//!   l'emplacement 0 (la même que celle écrite dans le niveau LDtk, voir
//!   `generation::cave`), ou redevient vide pour une carte ordinaire ; les niveaux de caverne
//!   portent `Destructible`.
//! - Dans `RollbackSystemSet::World`, après `world::plugin::apply_destroy_terrain_system` :
//!   les murs de la caverne sont détruits (`despawn_rollback`) et recréés depuis `CellGrid`
//!   quand une destruction a eu lieu dans la frame, et les cases murées de la navigation sont
//!   rechargées depuis la même grille (`FlowFieldCache::reload_walls`).
//!
//! `CellGrid`, les murs et `FlowFieldCache` sont tous trois rollback : un rejeu qui remonte
//! avant une destruction les retrouve cohérents, et la destruction rejouée refait les deux
//! reconstructions. Aucun état hors rollback ne doit retenir la dernière grille appliquée.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ggrs::{GgrsSchedule, Rollback, RollbackDespawnCommandExtension};
use std::collections::BTreeSet;

use game::character::enemy::ai::navigation::{FlowFieldCache, GridPos};
use game::collider::{CollisionSettings, Wall};
use sim_core::frame_events::FrameEvents;
use sim_core::system_set::RollbackSystemSet;
use utils::frame::FrameCount;
use utils::net_id::{GgrsNetId, GgrsNetIdFactory};
use utils::order_iter;
use world::{CellGrid, CellKind, Destructible, TerrainDestroyed};

use super::collider::spawn_cave_walls;
use super::floors::FloorSlots;
use super::plugin::LdtkMapLoadingEvent;
use crate::generation::cave::cave_grid;
use crate::loader::CaveSlots;

pub struct CavePlugin;

impl Plugin for CavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CaveSlots>()
            .add_systems(
                Update,
                fill_cave_grid_on_load.run_if(on_message::<LdtkMapLoadingEvent>),
            )
            .add_systems(
                Update,
                cave_cells_visual_system
                    .run_if(|| crate::RENDER_ENABLED)
                    .run_if(resource_changed::<CellGrid>),
            )
            .add_systems(
                GgrsSchedule,
                rebuild_cave_walls_system
                    .after(world::plugin::apply_destroy_terrain_system)
                    .in_set(RollbackSystemSet::World),
            );
    }
}

/// Grille de la caverne d'un emplacement de monde, ou grille vide.
pub fn grid_for_slot(caves: &CaveSlots, slot: usize) -> CellGrid {
    caves
        .0
        .get(&slot)
        .map(|(seed, config)| cave_grid(*seed, config))
        .unwrap_or_default()
}

fn fill_cave_grid_on_load(
    mut commands: Commands,
    caves: Res<CaveSlots>,
    mut grid: ResMut<CellGrid>,
    levels: Query<Entity, With<LevelIid>>,
    slots: FloorSlots,
) {
    *grid = grid_for_slot(&caves, 0);
    for level in &levels {
        if caves.0.contains_key(&slots.slot_of(level)) {
            commands.entity(level).insert(Destructible);
        }
    }
    if !grid.is_empty() {
        info!(
            "caverne {}x{} : {} cases de sol, {} de roche",
            grid.width,
            grid.height,
            grid.count(CellKind::Floor),
            grid.count(CellKind::Rock)
        );
    }
}

/// Recrée tous les murs depuis `CellGrid` quand la frame a détruit du terrain, et recharge les
/// cases murées de la navigation depuis la même grille (reconstruction complète du flow field
/// au prochain pas). Dans une caverne, tous les `Wall` rollback sont les murs de la caverne
/// (niveau unique ; en `Floors`, ceux du niveau quitté ont été détruits au passage).
#[allow(clippy::too_many_arguments)]
fn rebuild_cave_walls_system(
    mut commands: Commands,
    frame: Res<FrameCount>,
    grid: Res<CellGrid>,
    destroyed: Res<FrameEvents<TerrainDestroyed>>,
    walls: Query<(&GgrsNetId, Entity), (With<Wall>, With<Rollback>)>,
    collision_settings: Res<CollisionSettings>,
    mut id_factory: ResMut<GgrsNetIdFactory>,
    mut cache: ResMut<FlowFieldCache>,
) {
    if destroyed.is_empty() || grid.is_empty() {
        return;
    }
    let mut removed = 0;
    for (_, entity) in order_iter!(walls) {
        commands.entity(entity).despawn_rollback();
        removed += 1;
    }
    let created = spawn_cave_walls(
        &mut commands,
        &grid,
        &collision_settings,
        &mut id_factory,
        frame.frame,
    );
    cache.last_wall_entity_count = created;
    cache.reload_walls(solid_cells(&grid));
    info!(
        "ggrs{{f={} cave_walls_rebuilt removed={} created={}}}",
        frame.frame, removed, created
    );
}

/// Cases solides de la grille, en cases de navigation (grille à l'origine, cases de 16).
fn solid_cells(grid: &CellGrid) -> BTreeSet<GridPos> {
    (0..grid.height)
        .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
        .filter(|&(x, y)| grid.get(x as i32, y as i32).is_some_and(CellKind::is_solid))
        .map(|(x, y)| GridPos {
            x: x as i32,
            y: y as i32,
        })
        .collect()
}

/// Présentation minimale du terrain d'une caverne (rien en headless) : un carré par case
/// solide, reconstruit quand `CellGrid` change (destruction, rejeu, chargement). Le gabarit
/// n'a pas de tuiles d'autolayer : sans ces carrés, les murs seraient invisibles. Lit
/// `CellGrid`, aucun état de rendu propre.
#[derive(Component)]
struct CaveCellVisual;

fn cave_cells_visual_system(
    mut commands: Commands,
    grid: Res<CellGrid>,
    visuals: Query<Entity, With<CaveCellVisual>>,
) {
    for entity in &visuals {
        commands.entity(entity).despawn();
    }
    let size = world::CELL_SIZE as f32;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let color = match grid.get(x as i32, y as i32) {
                Some(CellKind::Wall) => Color::srgb(0.18, 0.17, 0.2),
                Some(CellKind::Rock) => Color::srgb(0.42, 0.33, 0.24),
                _ => continue,
            };
            commands.spawn((
                CaveCellVisual,
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz((x as f32 + 0.5) * size, (y as f32 + 0.5) * size, 1.0),
            ));
        }
    }
}
