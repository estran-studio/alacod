//! Trace de l'état de simulation, frame par frame, pour vérifier le déterminisme.
//!
//! Inactif par défaut. Variables d'environnement :
//! - `ALACOD_STATE_TRACE=<fichier>` : écrit une ligne par frame simulée
//!   (`<frame> <hash> <nb entités>`) ;
//! - `ALACOD_STATE_TRACE_FULL=1` : écrit aussi l'état détaillé de chaque entité ;
//! - `ALACOD_EXIT_AT_FRAME=<n>` : écrit la trace et quitte le jeu une fois la frame `n`
//!   atteinte (requis : la trace n'est écrite qu'à ce moment).
//!
//! Deux runs avec les mêmes inputs et les mêmes seeds doivent produire le même
//! fichier. En synctest, une frame resimulée remplace sa ligne précédente.
//!
//! Depuis le code (scénarios) : [`StateTraceRecorderPlugin`] enregistre la trace dans la
//! ressource [`StateTraceRecorder`], sans fichier ni arrêt.

use std::{collections::BTreeMap, fmt::Write as _, path::PathBuf};

use bevy::prelude::*;
use bevy_fixed::{fixed_math::FixedTransform3D, rng::RollbackRng};
use bevy_ggrs::{GgrsSchedule, Rollback};
use map::game::entity::map::window::WindowHealth;
use utils::{frame::FrameCount, net_id::GgrsNetId, order_iter};

use crate::{
    character::{
        enemy::ai::{EnemyTarget, MonsterState, Obstacle},
        health::Health,
        movement::Velocity,
    },
    frame::increase_frame_system,
    rollback::fnv1a,
    system_set::RollbackSystemSet,
    waves::WaveState,
};

/// Trace enregistrée : une ligne par frame simulée, indexée par numéro de frame.
#[derive(Resource, Default)]
pub struct StateTraceRecorder {
    /// Ajoute l'état détaillé de chaque entité sous la ligne de hash.
    pub full: bool,
    frames: BTreeMap<u32, String>,
}

impl StateTraceRecorder {
    /// Lignes des frames `0..end`, dans l'ordre.
    pub fn lines_until(&self, end: u32) -> impl Iterator<Item = &str> {
        self.frames.range(..end).map(|(_, line)| line.as_str())
    }
}

/// Enregistre la trace d'état dans [`StateTraceRecorder`].
pub struct StateTraceRecorderPlugin {
    pub full: bool,
}

impl Plugin for StateTraceRecorderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StateTraceRecorder {
            full: self.full,
            frames: BTreeMap::new(),
        })
        .add_systems(
            GgrsSchedule,
            record_state
                .in_set(RollbackSystemSet::FrameCounter)
                .before(increase_frame_system),
        );
    }
}

/// Systèmes de `Last` qui peuvent demander l'arrêt du jeu (`AppExit`). Ce qui doit réagir
/// à l'arrêt dans le même update (ex. écrire un enregistrement) s'ordonne après.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExitRequests;

/// Destination fichier de la trace (variables d'environnement).
#[derive(Resource)]
struct StateTraceFile {
    path: PathBuf,
    exit_at_frame: u32,
    written: bool,
}

/// Trace d'état pilotée par les variables d'environnement (voir le module).
pub struct StateTracePlugin;

impl Plugin for StateTracePlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var("ALACOD_STATE_TRACE") else {
            return;
        };
        let exit_at_frame = std::env::var("ALACOD_EXIT_AT_FRAME")
            .expect("ALACOD_STATE_TRACE demande ALACOD_EXIT_AT_FRAME")
            .parse()
            .expect("ALACOD_EXIT_AT_FRAME doit être un entier");

        app.add_plugins(StateTraceRecorderPlugin {
            full: std::env::var("ALACOD_STATE_TRACE_FULL").is_ok_and(|v| v == "1"),
        })
        .insert_resource(StateTraceFile {
            path: path.into(),
            exit_at_frame,
            written: false,
        })
        .add_systems(Last, write_trace_at_exit_frame.in_set(ExitRequests));
    }
}

type TracedEntity<'a> = (
    &'a GgrsNetId,
    Option<&'a FixedTransform3D>,
    Option<&'a Health>,
    Option<&'a Velocity>,
    Option<&'a MonsterState>,
    Option<&'a EnemyTarget>,
    Option<&'a WindowHealth>,
    Option<&'a Obstacle>,
);

fn record_state(
    frame: Res<FrameCount>,
    mut trace: ResMut<StateTraceRecorder>,
    entities: Query<TracedEntity, With<Rollback>>,
    wave_state: Option<Res<WaveState>>,
    rng: Option<Res<RollbackRng>>,
) {
    let mut state = String::new();
    let _ = writeln!(state, "wave={wave_state:?} rng={rng:?}", wave_state = wave_state.as_deref(), rng = rng.as_deref());
    let items = order_iter!(entities);
    for (id, transform, health, velocity, monster, target, window, obstacle) in &items {
        let _ = writeln!(
            state,
            "{id:?} t={transform:?} h={health:?} v={velocity:?} m={monster:?} tg={target:?} w={window:?} o={obstacle:?}"
        );
    }

    let mut line = format!("{} {:016x} {}", frame.frame, fnv1a(state.as_bytes()), items.len());
    if trace.full {
        line.push('\n');
        line.push_str(&state);
    }
    trace.frames.insert(frame.frame, line);
}

fn write_trace_at_exit_frame(
    frame: Res<FrameCount>,
    trace: Res<StateTraceRecorder>,
    mut file: ResMut<StateTraceFile>,
    mut exit: MessageWriter<AppExit>,
) {
    if file.written || frame.frame < file.exit_at_frame {
        return;
    }

    let mut out = String::new();
    for line in trace.lines_until(file.exit_at_frame) {
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(&file.path, out).expect("écriture de la trace d'état");
    info!("trace d'état écrite dans {:?}", file.path);
    file.written = true;
    exit.write(AppExit::Success);
}
