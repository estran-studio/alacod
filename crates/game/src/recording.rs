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
use bevy_fixed::fixed_math::Fixed;
use bevy_ggrs::{LocalInputs, ReadInputs};
use map::generation::config::MapGenerationConfig;
use utils::frame::FrameCount;

use crate::{
    character::player::{
        input::{read_local_inputs, BoxInput},
        jjrs::PeerConfig,
    },
    replay::{
        CharacterPlacement, PlayerScript, PowerUpPlacement, Scenario, Segment, WaveOverride,
        WeaponOverride,
    },
};

/// Réglages de la partie qui ne passent pas par les inputs mais changent la simulation
/// rejouée (D19) : l'enregistreur les reçoit à sa construction et [`InputRecorder::
/// to_scenario`] les reporte tels quels. Avant D19, `to_scenario` ne les connaissait pas et
/// `scenario::runner::run_with_options` les recopiait après coup dans le scénario enregistré.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedSettings {
    pub game: String,
    pub weapon_overrides: Vec<WeaponOverride>,
    pub wave_overrides: Option<WaveOverride>,
    pub powerups: Vec<PowerUpPlacement>,
    pub items: Vec<crate::replay::ItemPlacement>,
    pub profile: bool,
    pub powerup_drop_chance_override: Option<Fixed>,
    /// T1.8 : séquence de niveaux imposée (`Scenario::floors`).
    pub floors: Option<String>,
    /// T1.9 : horloges et difficulté actives (`Scenario::clocks`, `Scenario::difficulty`).
    pub clocks: Option<Vec<String>>,
    pub difficulty: Option<bool>,
    /// T1.13 : placements scriptés de personnages (`Scenario::characters`).
    pub characters: Vec<CharacterPlacement>,
    /// Mode de run imposé (`Scenario::mode`).
    pub mode: Option<content::EntryMode>,
    /// T1.10 : progression imposée (`Scenario::progression`).
    pub progression: Option<String>,
    /// Réglages de chaque joueur (index = handle) : `PlayerScript` du scénario d'origine, sans
    /// inputs ni bot (arme, tags, immunités, modificateurs, solde, mutations). Audit T1.15 :
    /// sans eux, un scénario réenregistré perdait l'arme imposée (scénarios générés) et les
    /// mutations, et ses attentes ne tenaient plus.
    pub players: Vec<PlayerScript>,
}

impl Default for RecordedSettings {
    /// Une partie jouée (`ALACOD_RECORD`, contrôle remote) : aucun réglage de scénario.
    fn default() -> Self {
        Self {
            // Legacy fallback for callers without a live manifest; native() refreshes it.
            game: "zombies".into(),
            weapon_overrides: vec![],
            wave_overrides: None,
            powerups: vec![],
            items: vec![],
            profile: false,
            powerup_drop_chance_override: None,
            floors: None,
            clocks: None,
            difficulty: None,
            characters: vec![],
            mode: None,
            progression: None,
            players: vec![],
        }
    }
}

impl RecordedSettings {
    /// Les réglages d'un scénario joué par `scenario::runner`.
    pub fn from_scenario(scenario: &Scenario) -> Self {
        Self {
            game: scenario.game.clone(),
            weapon_overrides: scenario.weapon_overrides.clone(),
            wave_overrides: scenario.wave_overrides.clone(),
            powerups: scenario.powerups.clone(),
            items: scenario.items.clone(),
            profile: scenario.profile,
            powerup_drop_chance_override: scenario.powerup_drop_chance_override,
            floors: scenario.floors.clone(),
            clocks: scenario.clocks.clone(),
            difficulty: scenario.difficulty,
            characters: scenario.characters.clone(),
            mode: scenario.mode,
            progression: scenario.progression.clone(),
            players: scenario
                .players
                .iter()
                .map(|player| PlayerScript {
                    inputs: vec![],
                    bot: None,
                    ..player.clone()
                })
                .collect(),
        }
    }
}

/// Inputs des joueurs locaux, par frame puis par handle.
#[derive(Resource)]
pub struct InputRecorder {
    frames: BTreeMap<u32, BTreeMap<usize, BoxInput>>,
    /// Fichier écrit à la fermeture du jeu.
    pub path: Option<PathBuf>,
    /// Reportés dans le scénario enregistré (D19).
    settings: RecordedSettings,
    /// Native sessions refresh settings on entry, while scenario settings stay explicit.
    native: bool,
    started: bool,
    run_number: u32,
    base_path: Option<PathBuf>,
    archive_failed: bool,
    initial_map: Option<(String, i32)>,
}

impl InputRecorder {
    /// Enregistreur vide, écrit à la fermeture dans `ALACOD_RECORD` s'il est défini.
    pub fn new(settings: RecordedSettings) -> Self {
        Self {
            frames: BTreeMap::new(),
            path: std::env::var("ALACOD_RECORD").ok().map(PathBuf::from),
            settings,
            native: false,
            started: false,
            run_number: 0,
            base_path: None,
            archive_failed: false,
            initial_map: None,
        }
    }

    /// Live settings are captured when the session has finished loading.
    pub fn native() -> Self {
        Self {
            native: true,
            ..Self::new(RecordedSettings::default())
        }
    }

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
                // Réglages du joueur dans le scénario d'origine (arme, tags, immunités,
                // modificateurs, solde, mutations) ; aucun pour une partie jouée.
                ..self
                    .settings
                    .players
                    .get(handle)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();

        let settings = self.settings.clone();
        let mut scenario = Scenario {
            game: settings.game,
            map: "exemples/test_map.ldtk".into(),
            map_seed: 123456,
            frames: self.frame_count(),
            players,
            expect: vec![],
            weapon_overrides: settings.weapon_overrides,
            wave_overrides: settings.wave_overrides,
            invariants: Default::default(),
            powerups: settings.powerups,
            items: settings.items,
            profile: settings.profile,
            powerup_drop_chance_override: settings.powerup_drop_chance_override,
            floors: settings.floors,
            // T1.10 : progression imposée par le scénario d'origine (sinon celle du manifeste).
            progression: settings.progression,
            clocks: settings.clocks,
            difficulty: settings.difficulty,
            characters: settings.characters,
            mode: settings.mode,
        };
        if let Some((path, seed)) = &self.initial_map {
            scenario.map = path.clone();
            scenario.map_seed = *seed;
        } else if let Some(map) = map {
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
        // `scenario::runner` replaces this with explicit settings before loading.
        app.insert_resource(InputRecorder::native())
            .add_systems(
                ReadInputs,
                start_native_recording.before(record_local_inputs),
            )
            .add_systems(ReadInputs, record_local_inputs.after(read_local_inputs))
            // Après l'arrêt demandé par la trace d'état : l'app s'arrête à la fin de l'update
            // qui émet AppExit, le message doit être lu dans le même update. Idem pour la
            // fermeture de la fenêtre : `exit_on_all_closed` émet AppExit dans `Last` (R7)
            .add_systems(
                Last,
                write_recording_on_exit
                    .after(crate::state_trace::ExitRequests)
                    .after(bevy::window::ExitSystems),
            );
    }
}

/// Capture effective settings before the first input. Preserve the previous run on
/// restart and use a new filename (`run-r2.ron`, ...), rather than mixing frame-zero
/// inputs with the previous run's tail. This is presentation/bookkeeping only.
fn start_native_recording(
    mut recorder: ResMut<InputRecorder>,
    frame: Res<FrameCount>,
    manifest: Option<Res<content::manifest::GameManifest>>,
    run: Option<Res<run::Run>>,
    clocks: Option<Res<crate::clock::ClockSchedule>>,
    difficulty: Option<Res<crate::clock::DifficultyConfig>>,
    progression: Res<crate::progression::ProgressionTable>,
    map: Option<Res<MapGenerationConfig>>,
) {
    if !recorder.native || (recorder.started && frame.frame != 0) {
        return;
    }
    let (Some(manifest), Some(run)) = (manifest, run) else {
        return;
    };
    if recorder.started && recorder.frame_count() > 0 {
        if let Some(path) = &recorder.path {
            if let Err(error) = write_recording(&recorder, None, path) {
                error!("écriture de l'enregistrement {path:?} : {error}; enregistrement suspendu pour préserver la partie précédente");
                // Don't overwrite old inputs with the restarted run if archiving failed.
                recorder.archive_failed = true;
                return;
            }
        }
    }
    recorder.run_number += 1;
    if recorder.started {
        recorder.path = recorder.base_path.as_ref().map(|base| {
            let mut filename = base.file_stem().unwrap_or_default().to_os_string();
            filename.push(format!("-r{}", recorder.run_number));
            let mut path = base.with_file_name(filename);
            if let Some(extension) = base.extension() {
                path.set_extension(extension);
            }
            path
        });
    } else {
        recorder.base_path = recorder.path.clone();
    }
    recorder.started = true;
    recorder.archive_failed = false;
    recorder.frames.clear();
    recorder.initial_map = map.map(|m| (m.map_path.clone(), m.seed));
    recorder.settings = live_settings(
        &manifest,
        &run,
        clocks.as_deref(),
        difficulty.is_some(),
        &progression,
    );
}

fn live_settings(
    manifest: &content::manifest::GameManifest,
    run: &run::Run,
    clocks: Option<&crate::clock::ClockSchedule>,
    difficulty: bool,
    progression: &crate::progression::ProgressionTable,
) -> RecordedSettings {
    use run::RunMode;
    let (mode, floors) = match &run.mode {
        RunMode::Floors { config } => (content::EntryMode::Floors, Some(config.clone())),
        RunMode::Waves { .. } => (content::EntryMode::Waves, None),
        RunMode::Sandbox => (content::EntryMode::Sandbox, None),
    };
    RecordedSettings {
        game: manifest.name.clone(),
        floors,
        mode: Some(mode),
        clocks: Some(
            clocks
                .map(|c| c.clocks.iter().map(|(id, _)| id.clone()).collect())
                .unwrap_or_default(),
        ),
        difficulty: Some(difficulty),
        progression: progression.active.as_ref().map(|p| p.id.clone()),
        ..Default::default()
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
    if recorder.archive_failed {
        return;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_scenario_reporte_les_reglages_recus_a_la_construction() {
        let settings = RecordedSettings {
            game: "testbed".into(),
            weapon_overrides: vec![],
            wave_overrides: None,
            powerups: vec![PowerUpPlacement {
                id: "max_ammo".into(),
                x: Fixed::from_num(423),
                y: Fixed::from_num(695),
                at_frame: 10,
            }],
            items: vec![],
            profile: false,
            powerup_drop_chance_override: Some(Fixed::ZERO),
            floors: Some("deux_niveaux".into()),
            clocks: None,
            difficulty: None,
            characters: vec![],
            mode: None,
            progression: Some("base".into()),
            players: vec![PlayerScript {
                weapon: Some("pistol".into()),
                mutations: vec!["vampire".into()],
                ..Default::default()
            }],
        };
        let mut recorder = InputRecorder::new(settings.clone());
        recorder
            .frames
            .insert(0, [(0, BoxInput::default())].into_iter().collect());
        let scenario = recorder.to_scenario(None);
        assert_eq!(scenario.game, "testbed");
        assert_eq!(scenario.powerups, settings.powerups);
        assert_eq!(scenario.powerup_drop_chance_override, Some(Fixed::ZERO));
        assert_eq!(scenario.progression.as_deref(), Some("base"));
        assert_eq!(scenario.players[0].mutations, ["vampire"]);
        assert_eq!(scenario.players[0].weapon.as_deref(), Some("pistol"));
        assert_eq!(RecordedSettings::from_scenario(&scenario), settings);

        // Partie jouée : réglages par défaut, comme avant D19.
        let played = InputRecorder::new(RecordedSettings::default()).to_scenario(None);
        assert_eq!(played.game, "zombies");
        assert!(played.powerups.is_empty() && played.weapon_overrides.is_empty());
        assert_eq!(played.wave_overrides, None);
        assert_eq!(played.powerup_drop_chance_override, None);
    }
    #[test]
    fn native_restart_archives_previous_inputs_and_resets_the_frame_range() {
        let path = std::env::temp_dir().join(format!(
            "alacod-recording-restart-{}.ron",
            std::process::id()
        ));
        let mut app = App::new();
        let mut recorder = InputRecorder::native();
        recorder.path = Some(path.clone());
        let manifest =
            content::manifest::GameManifest::load(std::path::Path::new("../../games/throne"))
                .unwrap();
        app.insert_resource(recorder)
            .insert_resource(FrameCount::default())
            .insert_resource(manifest)
            .insert_resource(run::Run::new(
                123456,
                run::RunMode::Floors {
                    config: "run".into(),
                },
                vec![0],
                0,
            ))
            .insert_resource(crate::progression::ProgressionTable::default())
            .insert_resource(MapGenerationConfig {
                map_path: "cave:niveau_1".into(),
                seed: 123456,
                ..Default::default()
            })
            .add_systems(Update, start_native_recording);
        app.update();
        {
            let mut recorder = app.world_mut().resource_mut::<InputRecorder>();
            recorder
                .frames
                .insert(20, [(0, BoxInput::default())].into_iter().collect());
        }
        app.update(); // frame 0 of a second native run
        let previous = Scenario::from_ron(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(previous.game, "throne");
        assert_eq!(previous.map, "cave:niveau_1");
        assert_eq!(previous.map_seed, 123456);
        assert_eq!(previous.frames, 21);
        let recorder = app.world().resource::<InputRecorder>();
        assert_eq!(recorder.frame_count(), 0);
        assert_eq!(
            recorder.path,
            Some(path.with_file_name(format!(
                "alacod-recording-restart-{}-r2.ron",
                std::process::id()
            )))
        );
        std::fs::remove_file(path).unwrap();
    }
}
