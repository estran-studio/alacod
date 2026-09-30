//! Enregistrement d'une session jouée en scénario rejouable.
//!
//! Chaque frame, les inputs envoyés à GGRS par les joueurs locaux (clavier, remote ou
//! script) sont mémorisés. Le résultat est un [`Scenario`] au format de
//! `tests/scenarios` : la session se rejoue à l'identique en headless.
//!
//! - `ALACOD_RECORD=<fichier.ron>` : enregistre et écrit le scénario à la fermeture du jeu ;
//! - le contrôle remote peut aussi l'écrire à la demande (`alacod/save_recording`).
//!
//! Limite : seuls les joueurs locaux sont enregistrés (sessions locales/synctest).

use std::{collections::BTreeMap, path::PathBuf};

use bevy::prelude::*;
use bevy_ggrs::{LocalInputs, ReadInputs};
use map::generation::config::MapGenerationConfig;
use utils::frame::FrameCount;

use crate::{
    character::player::{
        input::{read_local_inputs, BoxInput},
        jjrs::PeerConfig,
    },
    replay::{PlayerScript, Scenario, Segment},
};

/// Inputs des joueurs locaux, par frame puis par handle.
#[derive(Resource, Default)]
pub struct InputRecorder {
    frames: BTreeMap<u32, BTreeMap<usize, BoxInput>>,
    /// Fichier écrit à la fermeture du jeu.
    pub path: Option<PathBuf>,
}

impl InputRecorder {
    /// Nombre de frames enregistrées (dernière frame + 1).
    pub fn frame_count(&self) -> u32 {
        self.frames.keys().next_back().map_or(0, |f| f + 1)
    }

    /// Scénario rejouant la session enregistrée, sans attentes.
    pub fn to_scenario(&self, map: Option<&MapGenerationConfig>) -> Scenario {
        let player_count = self
            .frames
            .values()
            .flat_map(|inputs| inputs.keys())
            .max()
            .map_or(0, |handle| handle + 1);

        let players = (0..player_count)
            .map(|handle| PlayerScript {
                inputs: self.segments(handle),
                // Un enregistrement capture le BoxInput réellement envoyé à GGRS (bot ou pas :
                // voir `bots::read_bot_inputs`), donc rejoue toujours en `Scripted` (T2.11).
                bot: None,
                tags: vec![],
                immune_to: vec![],
                modifiers: vec![],
                // Un enregistrement ne choisit jamais l'arme (T2.10) : le joueur garde
                // l'équipement de son personnage, comme avant ce champ.
                weapon: None,
            })
            .collect();

        let mut scenario = Scenario {
            // TODO(T1.5) : lire le jeu courant depuis son manifeste, pas en dur
            game: "zombies".into(),
            map: "exemples/test_map.ldtk".into(),
            map_seed: 123456,
            frames: self.frame_count(),
            players,
            expect: vec![],
            weapon_overrides: vec![],
            invariants: Default::default(),
        };
        if let Some(map) = map {
            scenario.map = map.map_path.clone();
            scenario.map_seed = map.seed;
        }
        scenario
    }

    /// Regroupe les frames consécutives au même input ; les inputs neutres sont omis.
    fn segments(&self, handle: usize) -> Vec<Segment> {
        let mut segments = Vec::new();
        let mut current: Option<(u32, u32, BoxInput)> = None;

        for (frame, inputs) in &self.frames {
            let input = inputs.get(&handle).copied().unwrap_or_default();
            match &mut current {
                Some((_, to, held)) if *held == input && *to == *frame => *to = frame + 1,
                _ => {
                    if let Some((from, to, held)) = current.take() {
                        segments.push((from, to, held));
                    }
                    current = Some((*frame, frame + 1, input));
                }
            }
        }
        segments.extend(current);

        segments
            .into_iter()
            .filter(|(_, _, input)| *input != BoxInput::default())
            .map(|(from, to, input)| Segment::from_input(from, to, &input))
            .collect()
    }
}

/// Écrit le scénario enregistré dans `path`.
pub fn write_recording(
    recorder: &InputRecorder,
    map: Option<&MapGenerationConfig>,
    path: &std::path::Path,
) -> std::io::Result<()> {
    let scenario = recorder.to_scenario(map);
    let header = format!(
        "// Session enregistrée : {} frames, {} joueur(s).\n",
        scenario.frames,
        scenario.players.len()
    );
    std::fs::write(path, header + &scenario.to_ron() + "\n")
}

pub struct RecordingPlugin;

impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputRecorder {
            frames: BTreeMap::new(),
            path: std::env::var("ALACOD_RECORD").ok().map(PathBuf::from),
        })
        .add_systems(ReadInputs, record_local_inputs.after(read_local_inputs))
        // Après l'arrêt demandé par la trace d'état : l'app s'arrête à la fin de l'update
        // qui émet AppExit, le message doit être lu dans le même update
        .add_systems(
            Last,
            write_recording_on_exit.after(crate::state_trace::ExitRequests),
        );
    }
}

/// `pub` pour que `crates/bots` puisse ordonner son système `read_bot_inputs` avant celui-ci
/// (`.before(record_local_inputs)`) : sinon l'enregistrement capturerait l'input neutre
/// provisoire posé par `read_local_inputs` plutôt que la décision du bot qui le remplace
/// ensuite dans le même schedule `ReadInputs` (T2.11 ; la partie jouée par GGRS n'est pas
/// affectée par cet ordre, seul l'enregistrement le serait).
pub fn record_local_inputs(
    frame: Res<FrameCount>,
    local_inputs: Option<Res<LocalInputs<PeerConfig>>>,
    mut recorder: ResMut<InputRecorder>,
) {
    let Some(local_inputs) = local_inputs else {
        return;
    };
    recorder.frames.insert(
        frame.frame,
        local_inputs.0.iter().map(|(h, i)| (*h, *i)).collect(),
    );
}

fn write_recording_on_exit(
    mut exit: MessageReader<AppExit>,
    recorder: Res<InputRecorder>,
    map: Option<Res<MapGenerationConfig>>,
) {
    if exit.read().next().is_none() {
        return;
    }
    let Some(path) = &recorder.path else {
        return;
    };
    match write_recording(&recorder, map.as_deref(), path) {
        Ok(()) => info!(
            "session enregistrée dans {path:?} ({} frames)",
            recorder.frame_count()
        ),
        Err(err) => error!("écriture de l'enregistrement {path:?} : {err}"),
    }
}
