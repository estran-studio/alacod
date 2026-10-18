//! Compteur de frames de la simulation : `increase_frame_system` ouvre `RollbackSystemSet::FrameCounter`.
//! Les systèmes qui lisent la frame courante s'y ordonnent `.before`/`.after` (`clock`, `curses`,
//! `profile`, `jjrs::debug_desync_system`).

use bevy::prelude::*;
use utils::frame::FrameCount;

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}
