use bevy::prelude::*;
use bevy_fixed::fixed_math::{self, FixedVec2};

use crate::generation::entity::room::RoomConfig;

#[derive(Default, Component, Reflect)]
pub struct RoomComponent {
    pub config: RoomConfig,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct RoomBounds {
    pub position: FixedVec2,
    pub size: FixedVec2,
}

impl RoomBounds {
    pub fn contains(&self, point: FixedVec2) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y
    }
}

/// Opt-in presentation bounds for rooms hidden behind unopened doors.
/// Not simulation state: visibility is derived from players and open doors.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct RoomFog(pub Option<RoomBounds>);
