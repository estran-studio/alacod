use bevy::prelude::*;
use bevy_fixed::fixed_math;

#[derive(Component, Default, Clone, Debug, Hash)]
pub struct DashState {
    pub is_dashing: bool,
    pub dash_direction: fixed_math::FixedVec2,
    pub dash_frames_remaining: u32,
    pub dash_cooldown_remaining: u32,
    pub dash_distance_per_frame: fixed_math::Fixed, // Distance to move each frame
    pub dash_start_position: fixed_math::FixedVec3, // Starting position for the dash
    pub dash_total_distance: fixed_math::Fixed,     // Total distance for current dash
}

impl DashState {
    pub fn can_dash(&self) -> bool {
        !self.is_dashing && self.dash_cooldown_remaining == 0
    }

    pub fn start_dash(
        &mut self,
        direction: fixed_math::FixedVec2,
        start_position: fixed_math::FixedVec3,
        total_distance: fixed_math::Fixed,
        duration_frames: u32,
    ) {
        // Ensure duration is at least 1 to prevent division by zero
        let safe_duration = duration_frames.max(1);

        self.is_dashing = true;
        self.dash_direction = direction.normalize_or_zero();
        self.dash_frames_remaining = safe_duration;
        self.dash_start_position = start_position;
        self.dash_total_distance = total_distance;
        self.dash_distance_per_frame = total_distance / fixed_math::new(safe_duration as f32);
    }

    pub fn update(&mut self) {
        if self.is_dashing {
            self.dash_frames_remaining = self.dash_frames_remaining.saturating_sub(1);
            if self.dash_frames_remaining == 0 {
                self.is_dashing = false;
            }
        }

        if self.dash_cooldown_remaining > 0 {
            self.dash_cooldown_remaining = self.dash_cooldown_remaining.saturating_sub(1);
        }
    }

    pub fn set_cooldown(&mut self, cooldown_frames: u32) {
        self.dash_cooldown_remaining = cooldown_frames;
    }
}
