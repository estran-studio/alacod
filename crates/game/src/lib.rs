pub mod args;
pub mod audio;
pub mod balance;
pub mod camera;
pub mod character;
pub mod clock;
pub mod collider;
pub mod collision_grid;
pub mod content_hot_reload;
pub mod core;
pub mod economy;
pub mod effects_runtime;
pub mod feedback;
pub mod frame;
pub mod frame_events;
pub mod global_asset;
pub mod interaction;
pub mod jjrs;
pub mod light;
pub mod patterns;
pub mod powerups;
pub mod progression;
pub mod recording;
#[cfg(not(target_arch = "wasm32"))]
pub mod remote;
pub mod replay;
pub mod rollback;
pub mod run_state;
pub mod state_trace;
pub mod statuses;
pub mod system_set;
pub mod ui;
pub mod waves;
pub use combat::weapons;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref GAME_SPEED: bevy_fixed::fixed_math::Fixed = bevy_fixed::fixed_math::new(60.);
}
