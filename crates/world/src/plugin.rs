//! Branchement ECS du terrain (T1.6) : enregistrement rollback et application des demandes de
//! destruction dans `RollbackSystemSet::World`.

use bevy::prelude::*;
use bevy_fixed::fixed_math::{Fixed, FixedVec2};
use bevy_ggrs::GgrsSchedule;
use sim_core::frame_events::{FrameEvents, FrameEventsAppExt};
use sim_core::system_set::RollbackSystemSet;
use utils::frame::FrameCount;
use utils::rollback::RollbackTraceApp;

use crate::destroy::destroy_terrain;
use crate::grid::CellGrid;

/// Demande de destruction de terrain (`effects::Action::DestroyTerrain`) émise par un projectile
/// (`on_hit` sur un mur, `on_expire`) avant `RollbackSystemSet::World`. Position en champs
/// séparés : `FixedVec2` n'est pas `Hash`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DestroyTerrainRequest {
    pub x: Fixed,
    pub y: Fixed,
    pub radius: Fixed,
}

impl DestroyTerrainRequest {
    pub fn new(position: FixedVec2, radius: Fixed) -> Self {
        Self {
            x: position.x,
            y: position.y,
            radius,
        }
    }
}

/// Cases détruites dans la frame (une entrée par demande qui a creusé), lues par la
/// reconstruction des murs (`map_ldtk`) et le détecteur de moments clés.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainDestroyed {
    pub cells: Vec<(u32, u32)>,
}

/// Enregistre `CellGrid` (rollback, checksum neutre : grille vide = 0) et les deux files
/// d'événements (neutres : vides hors caverne), et applique les demandes de la frame.
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CellGrid>()
            .rollback_and_trace_resource_neutral::<CellGrid>()
            // T1.7 : surfaces (creuse, vide hors couche `Surfaces` : neutre)
            .init_resource::<crate::surface::SurfaceGrid>()
            .rollback_and_trace_resource_neutral::<crate::surface::SurfaceGrid>()
            .init_resource::<crate::surface::SurfaceTable>()
            // M2-E1 : salles typées (vides hors carte avec `room_kind` : neutres)
            .init_resource::<crate::rooms::RoomStates>()
            .rollback_and_trace_resource_neutral::<crate::rooms::RoomStates>()
            .rollback_and_trace_neutral::<crate::rooms::RoomDormant>()
            .init_resource::<crate::rooms::RoomKindTable>()
            .add_frame_events_neutral::<crate::rooms::RoomChanged>()
            .add_frame_events_neutral::<DestroyTerrainRequest>()
            .add_frame_events_neutral::<TerrainDestroyed>()
            .add_systems(
                GgrsSchedule,
                apply_destroy_terrain_system.in_set(RollbackSystemSet::World),
            );
    }
}

/// Applique les demandes de la frame dans leur ordre d'émission ; `Wall` ne change jamais.
pub fn apply_destroy_terrain_system(
    frame: Res<FrameCount>,
    requests: Res<FrameEvents<DestroyTerrainRequest>>,
    mut grid: ResMut<CellGrid>,
    mut destroyed: ResMut<FrameEvents<TerrainDestroyed>>,
) {
    if requests.is_empty() || grid.is_empty() {
        return;
    }
    for request in requests.iter() {
        let center = FixedVec2::new(request.x, request.y);
        let cells = destroy_terrain(&mut grid, center, request.radius);
        if cells.is_empty() {
            continue;
        }
        info!(
            "ggrs{{f={} terrain_destroyed x={} y={} radius={} cells={}}}",
            frame.frame,
            request.x,
            request.y,
            request.radius,
            cells.len()
        );
        destroyed.send(TerrainDestroyed { cells });
    }
}
