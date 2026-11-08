//! Le feu follet et les points de reprise, **dérivés de l'état dans `Update`** (M-5,
//! `docs/conventions.md` §113) : présentation seule, hors simulation, absente en headless.
//!
//! - un **feu follet** (`combat::downed::Wisp`) est dessiné en petite lumière chaude qui pulse (trois
//!   cercles concentriques) à la place de son corps ; un arc autour de lui montre la progression de la
//!   relève par un coéquipier (`Reviving`) et, en second, celle de la reprise à un point de reprise ;
//! - un **point de reprise** (`RespawnPoint`) est un anneau à sa portée, un foyer en son centre.
//!
//! Gizmos seulement (aucune entité, aucun asset) : rien à ressusciter après un rollback, et un jeu sans
//! feu follet ne dessine rien.

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::downed::{RespawnPoint, Reviving, Wisp};
use utils::frame::FrameCount;

use crate::character::config::{CharacterConfigKey, CharacterConfigs};
use crate::character::player::Player;

pub struct WispVisualPlugin;

impl Plugin for WispVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_wisps_and_respawn_points);
    }
}

/// Lumière du feu follet et de l'anneau de reprise.
const GLOW: Color = Color::srgb(1.0, 0.82, 0.35);
const GLOW_SOFT: Color = Color::srgba(1.0, 0.82, 0.35, 0.45);
const REVIVE: Color = Color::srgb(0.45, 0.9, 1.0);

fn draw_wisps_and_respawn_points(
    frame: Res<FrameCount>,
    configs: Res<CharacterConfigs>,
    wisps: Query<
        (
            &FixedTransform3D,
            &Wisp,
            Option<&Reviving>,
            Option<&CharacterConfigKey>,
        ),
        With<Player>,
    >,
    points: Query<(&FixedTransform3D, &RespawnPoint)>,
    mut gizmos: Gizmos,
) {
    let center =
        |transform: &FixedTransform3D| transform.to_bevy_transform().translation.truncate();

    for (transform, point) in &points {
        let at = center(transform);
        gizmos.circle_2d(at, point.range.to_num::<f32>(), GLOW_SOFT);
        gizmos.circle_2d(at, 3.0, GLOW);
    }

    // Pulsation : présentation seulement, `f32` permis hors simulation.
    let pulse = ((frame.frame % 60) as f32 / 60.0 * std::f32::consts::TAU).sin();
    for (transform, wisp, reviving, key) in &wisps {
        let at = center(transform);
        gizmos.circle_2d(at, 3.0 + pulse, GLOW);
        gizmos.circle_2d(at, 6.0 + pulse * 1.5, GLOW_SOFT);
        gizmos.circle_2d(at, 10.0 + pulse * 2.0, GLOW_SOFT);

        if let Some(reviving) = reviving {
            let needed = key
                .and_then(|key| configs.get(&key.0))
                .map_or(180, |config| config.revive_frames);
            let fraction = (reviving.progress_frames as f32 / needed.max(1) as f32).min(1.0);
            gizmos.arc_2d(
                Isometry2d::from_translation(at),
                std::f32::consts::TAU * fraction,
                14.0,
                REVIVE,
            );
        }
        if let Some(needed) = wisp.respawn_frames.filter(|_| wisp.respawn_progress > 0) {
            let fraction = (wisp.respawn_progress as f32 / needed.max(1) as f32).min(1.0);
            gizmos.arc_2d(
                Isometry2d::from_translation(at),
                std::f32::consts::TAU * fraction,
                18.0,
                GLOW,
            );
        }
    }
}
