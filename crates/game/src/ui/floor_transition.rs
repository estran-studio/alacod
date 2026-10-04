//! Transition de niveau (T1.16, `docs/conventions.md` §30) : présentation seule.
//!
//! À chaque changement de `run::floors::FloorState::index` lu côté présentation (pas
//! `FloorEntered`, émis seulement avec horloges ou difficulté), l'écran part du noir et
//! s'éclaircit en `floor_fade_seconds` (`camera.ron`), et la caméra est posée sur sa cible
//! pendant le fondu au lieu d'y glisser depuis l'ancien niveau (`floor_recenter`). Le premier index lu (début de partie)
//! ne déclenche rien. Un rollback qui ferait reculer l'index déclenche aussi : l'affichage suit.

use bevy::prelude::*;
use run::floors::FloorState;

use crate::camera::{camera_control_system, CameraSettings, GameCamera};
use crate::core::AppState;

/// Index vu à la frame d'affichage précédente et début du fondu en cours.
#[derive(Resource, Default, Debug)]
pub struct FloorTransition {
    pub last_index: Option<u32>,
    /// Temps (`Time::elapsed_secs`) du dernier changement de niveau.
    pub started_at: Option<f32>,
    /// Changement vu cette frame : la caméra se recentre.
    pub just_changed: bool,
}

/// Le niveau a changé depuis la dernière lecture (pas à la première).
pub fn floor_changed(last: Option<u32>, current: u32) -> bool {
    last.is_some_and(|last| last != current)
}

/// Opacité du voile noir `elapsed` secondes après le changement : 1 puis décroissance
/// linéaire jusqu'à 0 en `duration` ; 0 sans fondu.
pub fn fade_alpha(elapsed: f32, duration: f32) -> f32 {
    if duration <= 0.0 || elapsed < 0.0 {
        return 0.0;
    }
    (1.0 - elapsed / duration).clamp(0.0, 1.0)
}

/// Voile noir plein écran.
#[derive(Component)]
struct FloorFadeOverlay;

pub struct FloorTransitionPlugin;

impl Plugin for FloorTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloorTransition>()
            .add_systems(OnEnter(AppState::InGame), reset_floor_transition)
            .add_systems(
                Update,
                (
                    detect_floor_change,
                    recenter_camera_on_floor_change.after(camera_control_system),
                    draw_floor_fade,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), despawn_floor_fade);
    }
}

fn reset_floor_transition(mut transition: ResMut<FloorTransition>) {
    *transition = FloorTransition::default();
}

fn detect_floor_change(
    time: Res<Time>,
    floor: Option<Res<FloorState>>,
    mut transition: ResMut<FloorTransition>,
) {
    transition.just_changed = false;
    let Some(floor) = floor else {
        return;
    };
    if floor_changed(transition.last_index, floor.index) {
        transition.started_at = Some(time.elapsed_secs());
        transition.just_changed = true;
    }
    transition.last_index = Some(floor.index);
}

/// Pendant tout le fondu (et à la frame du changement) la caméra est posée sur sa cible : la
/// position du joueur peut n'arriver qu'une frame d'affichage plus tard.
fn recenter_camera_on_floor_change(
    time: Res<Time>,
    transition: Res<FloorTransition>,
    settings: Res<CameraSettings>,
    mut cameras: Query<(&GameCamera, &mut Transform)>,
) {
    let fading = transition.started_at.is_some_and(|start| {
        time.elapsed_secs() - start < settings.floor_fade_seconds
    });
    if !settings.floor_recenter || !(transition.just_changed || fading) {
        return;
    }
    for (camera, mut transform) in &mut cameras {
        transform.translation.x = camera.target_position.x;
        transform.translation.y = camera.target_position.y;
    }
}

fn draw_floor_fade(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    transition: Res<FloorTransition>,
    mut overlays: Query<(Entity, &mut BackgroundColor), With<FloorFadeOverlay>>,
) {
    let alpha = transition
        .started_at
        .map_or(0.0, |start| {
            fade_alpha(time.elapsed_secs() - start, settings.floor_fade_seconds)
        });
    match (overlays.single_mut(), alpha > 0.0) {
        (Ok((_, mut color)), true) => color.0 = Color::BLACK.with_alpha(alpha),
        (Ok((entity, _)), false) => commands.entity(entity).despawn(),
        (Err(_), true) => {
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(alpha)),
                GlobalZIndex(20),
                FloorFadeOverlay,
            ));
        }
        (Err(_), false) => {}
    }
}

fn despawn_floor_fade(mut commands: Commands, overlays: Query<Entity, With<FloorFadeOverlay>>) {
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changement_de_niveau() {
        assert!(!floor_changed(None, 0), "première lecture : rien");
        assert!(!floor_changed(None, 2));
        assert!(!floor_changed(Some(1), 1));
        assert!(floor_changed(Some(0), 1));
        assert!(floor_changed(Some(2), 1), "rollback : l'affichage suit");
    }

    #[test]
    fn fondu_lineaire() {
        assert_eq!(fade_alpha(0.0, 0.4), 1.0);
        assert!((fade_alpha(0.1, 0.4) - 0.75).abs() < 1e-6);
        assert!((fade_alpha(0.2, 0.4) - 0.5).abs() < 1e-6);
        assert_eq!(fade_alpha(0.4, 0.4), 0.0);
        assert_eq!(fade_alpha(1.0, 0.4), 0.0);
        assert_eq!(fade_alpha(0.0, 0.0), 0.0, "sans fondu");
        assert_eq!(fade_alpha(-0.1, 0.4), 0.0);
    }

    #[test]
    fn camera_ron_des_jeux() {
        for text in [
            include_str!("../../../../games/testbed/assets/camera.ron"),
            include_str!("../../../../games/throne/assets/camera.ron"),
            include_str!("../../../../games/zombies/assets/camera.ron"),
        ] {
            let settings: crate::camera::CameraSettingsAsset =
                ron::from_str(text).expect("camera.ron");
            assert!((settings.0.floor_fade_seconds - 0.4).abs() < 1e-6);
            assert!(settings.0.floor_recenter);
        }
    }
}
