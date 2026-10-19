//! Présentation du blank (D69 étape 2, E5c) : l'onde dérivée de `BlankUsed`. Absente en headless
//! (`PresentationPlugin`), jamais lue par la simulation.

use bevy::prelude::*;
use items::BlankUsed;
use utils::frame::FrameCount;

use crate::frame_events::FrameEvents;

/// Dernier blank utilisé, pour l'onde de présentation (M2-T1) : dérivé de `FrameEvents<BlankUsed>`
/// dans `Update`, jamais lu par la simulation. Un rollback qui réémet l'événement réécrit les
/// mêmes valeurs.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct LastBlank(pub Option<BlankUsedAt>);

#[derive(Clone, Copy, Debug)]
pub struct BlankUsedAt {
    pub frame: u32,
    pub center: Vec2,
    pub radius: f32,
}

/// Durée de l'onde, en frames de simulation.
pub const BLANK_WAVE_FRAMES: u32 = 24;

fn track_blank_system(events: Res<FrameEvents<BlankUsed>>, mut last: ResMut<LastBlank>) {
    for blank in events.iter() {
        last.0 = Some(BlankUsedAt {
            frame: blank.frame,
            center: Vec2::new(blank.x.to_num::<f32>(), blank.y.to_num::<f32>()),
            radius: blank.radius.to_num::<f32>(),
        });
    }
}

/// Onde du blank : un cercle qui s'étend jusqu'au rayon et s'estompe (`Update`, présentation).
fn draw_blank_wave_system(
    last: Res<LastBlank>,
    frame: Res<FrameCount>,
    config: Option<Res<crate::feedback_log::FeedbackConfigLoaded>>,
    mut gizmos: Gizmos,
) {
    // M2-T19 (§55) : l'anneau de `ui/feedback.ron` (`blank`) remplace cette onde quand il est actif.
    if config.is_some_and(|config| config.0.blank.enabled) {
        return;
    }
    let Some(blank) = last.0 else {
        return;
    };
    let age = frame.frame.saturating_sub(blank.frame);
    if age >= BLANK_WAVE_FRAMES {
        return;
    }
    let progress = age as f32 / BLANK_WAVE_FRAMES as f32;
    let color = Color::srgba(0.7, 0.9, 1.0, 1.0 - progress);
    gizmos.circle_2d(
        Isometry2d::from_translation(blank.center),
        blank.radius * progress,
        color,
    );
}

/// Présentation du blank (absente en headless, avec `PresentationPlugin`).
pub struct BlankPresentationPlugin;

impl Plugin for BlankPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastBlank>().add_systems(
            Update,
            (
                track_blank_system,
                draw_blank_wave_system.after(track_blank_system),
            ),
        );
    }
}
