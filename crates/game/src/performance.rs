//! Opt-in native frame timings, outside rollback. `ALACOD_PERF_CSV=<file>`.
//! Intervals measure main-app cadence, not GPU timestamps or presented images.

use crate::{core::AppState, system_set::RollbackSystemSet, weapons::Bullet};
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
            "frame,interval_ms,main_cpu_ms,simulation_ms,simulation_steps,bullets"
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

fn record_frame(
    mut capture: ResMut<Capture>,
    frame: Res<FrameCount>,
    bullets: Query<(), With<Bullet>>,
) {
    let cpu = capture.start.elapsed().as_secs_f64() * 1000.0;
    let interval = capture.interval_ms;
    let simulation = capture.simulation.as_secs_f64() * 1000.0;
    let steps = capture.simulation_steps;
    writeln!(
        capture.output,
        "{},{interval:.4},{cpu:.4},{simulation:.4},{steps},{}",
        frame.frame,
        bullets.iter().len()
    )
    .unwrap();
    capture.rows += 1;
    if capture.rows % 120 == 0 {
        capture.output.flush().unwrap();
    }
}
