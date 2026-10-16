pub mod args;
pub mod audio;
pub mod balance;
pub mod boss;
pub mod camera;
pub mod cave_assets;
pub mod character;
pub mod clock;
pub mod collider;
pub mod collision_grid;
pub mod content_hot_reload;
pub mod core;
pub mod economy;
pub mod effects_runtime;
pub mod familiars;
pub mod feedback;
pub mod frame;
pub mod frame_events;
pub mod gen_test;
pub mod global_asset;
pub mod hub;
pub mod interaction;
pub mod items;
pub mod jjrs;
pub mod light;
#[cfg(not(target_arch = "wasm32"))]
pub mod logs;
pub mod loot;
pub mod patterns;
#[cfg(not(target_arch = "wasm32"))]
pub mod performance;
mod powerup_visuals;
pub mod powerups;
pub mod profile;
pub mod progression;
pub mod props;
pub mod recording;
#[cfg(not(target_arch = "wasm32"))]
pub mod remote;
pub mod replay;
pub mod rollback;
pub mod room_fog;
pub mod run_state;
pub mod shop;
pub mod state_trace;
pub mod statuses;
pub mod system_set;
pub mod ui;
pub mod waves;
pub mod web;
mod world_sprites;
pub use combat::weapons;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref GAME_SPEED: bevy_fixed::fixed_math::Fixed = bevy_fixed::fixed_math::new(60.);
}
