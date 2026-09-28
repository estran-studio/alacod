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
pub mod recording;
#[cfg(not(target_arch = "wasm32"))]
pub mod remote;
pub mod replay;
pub mod light;
pub mod state_trace;
pub mod system_set;
pub mod ui;
pub mod waves;
pub mod weapons;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref GAME_SPEED: bevy_fixed::fixed_math::Fixed = bevy_fixed::fixed_math::new(60.);
}
