pub mod args;
pub mod audio;
pub mod camera;
pub mod character;
pub mod collider;
pub mod core;
pub mod frame;
pub mod frame_events;
pub mod global_asset;
pub mod interaction;
pub mod jjrs;
pub mod light;
pub mod system_set;
pub mod ui;
pub mod waves;
pub mod weapons;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref GAME_SPEED: bevy_fixed::fixed_math::Fixed = bevy_fixed::fixed_math::new(60.);
}
