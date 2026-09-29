use bevy::prelude::*;

#[derive(Resource, Default, Reflect, Hash, Clone, Copy, Debug)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}

impl std::fmt::Display for FrameCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}|{}", self.frame, self.frame % 60)
    }
}
