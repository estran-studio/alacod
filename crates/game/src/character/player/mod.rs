pub mod control;
pub mod create;
pub mod input;
pub mod jjrs;

use bevy::prelude::*;
pub use combat::actors::Player;

#[derive(Component, Reflect, Default, Debug, Copy, Clone)]
#[reflect(Component)]
pub struct LocalPlayer {}
