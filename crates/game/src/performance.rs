//! Opt-in native frame timings, outside rollback. `ALACOD_PERF_CSV=<file>`.
//! Intervals measure main-app cadence, not GPU timestamps or presented images.

use crate::{session::AppState, system_set::RollbackSystemSet, weapons::Bullet};
use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};
use utils::frame::FrameCount;

pub struct PerformanceCapturePlugin(pub PathBuf);

#[derive(Resource)]
struct Capture {
    output: BufWriter<File>,
    previous: Option<Instant>,
    start: Instant,
    interval_ms: f64,
    simulation_start: Instant,
    simulation: Duration,
    simulation_steps: u32,
    rows: u32,
}

impl Plugin for PerformanceCapturePlugin {
    fn build(&self, app: &mut App) {
        let mut output = BufWriter::new(File::create(&self.0).expect("create ALACOD_PERF_CSV"));
        writeln!(
            output,
            "frame,interval_ms,main_cpu_ms,simulation_ms,simulation_steps,bullets,player_x,player_y,camera_x,camera_y,alpha"
        )
        .unwrap();
        let now = Instant::now();
        app.insert_resource(Capture {
            output,
            previous: None,
            start: now,
            interval_ms: 0.0,
            simulation_start: now,
            simulation: Duration::ZERO,
            simulation_steps: 0,
            rows: 0,
        })
        .add_systems(First, begin_frame)
        .add_systems(
            GgrsSchedule,
            begin_simulation.before(RollbackSystemSet::FrameStart),
        )
        .add_systems(
            GgrsSchedule,
            end_simulation.after(RollbackSystemSet::FrameCounter),
        )
        .add_systems(Last, record_frame.run_if(in_state(AppState::InGame)));
    }
}

fn begin_frame(mut capture: ResMut<Capture>) {
    let now = Instant::now();
    capture.interval_ms = capture.previous.map_or(0.0, |previous| {
        now.duration_since(previous).as_secs_f64() * 1000.0
    });
    capture.previous = Some(now);
    capture.start = now;
    capture.simulation = Duration::ZERO;
    capture.simulation_steps = 0;
}

fn begin_simulation(mut capture: ResMut<Capture>) {
    capture.simulation_start = Instant::now();
}

fn end_simulation(mut capture: ResMut<Capture>) {
    let elapsed = capture.simulation_start.elapsed();
    capture.simulation += elapsed;
    capture.simulation_steps += 1;
}

/// Une ligne par image. Colonnes d'affichage (S1-6, §94) : position **rendue** du joueur local de plus
/// petit handle et de la caméra de jeu (après interpolation et suivi), et l'`alpha` de l'interpolation ;
/// vides hors partie. `scripts/frame-pacing.py` en tire la régularité du mouvement à l'écran.
fn record_frame(
    mut capture: ResMut<Capture>,
    frame: Res<FrameCount>,
    bullets: Query<(), With<Bullet>>,
    players: Query<
        (&crate::character::player::Player, &GlobalTransform),
        With<crate::character::player::LocalPlayer>,
    >,
    cameras: Query<&GlobalTransform, With<crate::camera::GameCamera>>,
    clock: Option<Res<crate::display_interpolation::InterpolationClock>>,
) {
    let cpu = capture.start.elapsed().as_secs_f64() * 1000.0;
    let interval = capture.interval_ms;
    let simulation = capture.simulation.as_secs_f64() * 1000.0;
    let steps = capture.simulation_steps;
    let player = players
        .iter()
        .min_by_key(|(player, _)| player.handle)
        .map(|(_, transform)| transform.translation());
    let camera = cameras.iter().next().map(GlobalTransform::translation);
    let xy = |position: Option<Vec3>| {
        position.map_or(",".to_string(), |p| format!("{:.4},{:.4}", p.x, p.y))
    };
    let alpha = clock.map_or(String::new(), |clock| format!("{:.4}", clock.alpha));
    writeln!(
        capture.output,
        "{},{interval:.4},{cpu:.4},{simulation:.4},{steps},{},{},{},{alpha}",
        frame.frame,
        bullets.iter().len(),
        xy(player),
        xy(camera),
    )
    .unwrap();
    capture.rows += 1;
    if capture.rows % 120 == 0 {
        capture.output.flush().unwrap();
    }
}
