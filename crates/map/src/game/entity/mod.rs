use bevy::ecs::{component::Component, entity::Entity};

pub mod map;

#[derive(Component, Clone)]
pub struct MapRollbackItem {
    pub parent: Entity,
    pub kind: String,
}

impl MapRollbackItem {
    pub fn new(parent: Entity, kind: String) -> Self {
        Self { parent, kind }
    }
}
