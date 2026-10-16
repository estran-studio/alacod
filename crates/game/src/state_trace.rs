//! Trace de l'état de simulation, frame par frame, pour vérifier le déterminisme.
//!
//! Inactif par défaut. Variables d'environnement :
//! - `ALACOD_STATE_TRACE=<fichier>` : écrit une ligne par frame simulée
//!   (`<frame> <checksum GGRS 128 bits> <nb entités rollback>`) ;
//! - `ALACOD_STATE_TRACE_FULL=1` : écrit aussi l'état détaillé de chaque ressource et
//!   entité rollback tracées (voir [`crate::rollback::StateTracers`]) ;
//! - `ALACOD_EXIT_AT_FRAME=<n>` : écrit la trace et quitte le jeu une fois la frame `n`
//!   atteinte (requis : la trace n'est écrite qu'à ce moment). La trace compte alors `n - 1`
//!   lignes, pas `n` : la toute dernière frame n'a pas le temps d'être enregistrée avant
//!   l'arrêt (voir [`write_trace_at_exit_frame`]).
//! - `ALACOD_RESTART_AT_FRAME=<r>` (D14, §33) : deux parties. À la frame `r` de la première,
//!   écrit sa trace dans `<fichier>-g1`, puis pose `RunRequest::Restart`
//!   [`RESTART_GRACE_FRAMES`] frames plus tard (le temps que le pair atteigne lui aussi `r`) ;
//!   la seconde partie écrit `<fichier>-g2` à `ALACOD_EXIT_AT_FRAME` (compté depuis son début)
//!   et quitte.
//!
//! Deux runs avec les mêmes inputs et les mêmes seeds doivent produire le même
//! fichier. En synctest, une frame resimulée remplace sa ligne précédente.
//!
//! Le hash de chaque ligne est le [`Checksum`](bevy_ggrs::Checksum) GGRS : la même valeur
//! que celle comparée par le synctest et la détection de desync p2p (donc exactement ce
//! qu'`app.rollback_and_trace::<T>()`, via `checksum_component`/`checksum_resource`, fait
//! contribuer). Ce checksum n'est finalisé par bevy_ggrs qu'entre les sets
//! `SaveWorldSystems::Checksum` et `SaveWorldSystems::Snapshot` du schedule `SaveWorld`
//! (voir `bevy_ggrs::snapshot::checksum` : `ChecksumPlugin::update` s'exécute
//! `.after(Checksum).before(Snapshot)`) : [`record_state`] est donc un système du
//! schedule `SaveWorld`, dans le set `Snapshot`, et non plus du `GgrsSchedule` où
//! vivait l'ancienne version (qui aurait lu le checksum de la `SaveGameState`
//! précédente, pas celle de la frame en cours). Comme `SaveWorld` tourne juste après
//! l'`AdvanceFrame` de la même frame (même appel à `handle_requests` côté bevy_ggrs), le
//! numéro de frame utilisé est [`bevy_ggrs::RollbackFrameCount`] (et non la ressource
//! `FrameCount` du jeu, dont l'incrément a lieu plus tôt, dans le `GgrsSchedule`) — moins
//! un : `RollbackFrameCount` lu à ce point vaut systématiquement un de plus que le numéro
//! de frame au sens du jeu (vérifié empiriquement, voir [`current_rollback_frame`]).
//!
//! Depuis le code (scénarios) : [`StateTraceRecorderPlugin`] enregistre la trace dans la
//! ressource [`StateTraceRecorder`], sans fichier ni arrêt.

use std::{collections::BTreeMap, fmt::Write as _, path::PathBuf};

use bevy::prelude::*;
use bevy_ggrs::{Checksum, Rollback, RollbackFrameCount, SaveWorld, SaveWorldSystems};
use utils::{frame::FrameCount, net_id::GgrsNetId};

use crate::rollback::StateTracers;

/// Trace enregistrée : une ligne par frame simulée, indexée par numéro de frame.
#[derive(Resource, Default)]
pub struct StateTraceRecorder {
    /// Ajoute l'état détaillé de chaque ressource et entité tracée sous la ligne de hash.
    pub full: bool,
    frames: BTreeMap<u32, String>,
    /// Frames dont la resimulation (synctest) a produit un autre checksum que le premier
    /// passage : les trois premières, avec les deux versions de la ligne.
    pub divergences: Vec<Divergence>,
    /// Les dernières lignes enregistrées, dans l'ordre d'enregistrement (frame, ligne) : en
    /// synctest, une frame y figure plusieurs fois (premier passage, resimulations). Sert au
    /// diagnostic d'une divergence signalée par GGRS, quelle que soit la numérotation.
    pub history: std::collections::VecDeque<(u32, String)>,
    /// Lignes complètes (hash puis détail), pour **toutes** les frames simulées, sans la
    /// limite de `history` (`HISTORY_LEN`) : peuplé seulement quand
    /// [`StateTraceRecorderPlugin::dump`] est actif (outil de preuve permanent, T1.2 —
    /// voir `docs/conventions.md` §10 « Blesser une trace : la preuve »). `None` sinon,
    /// pour ne rien coûter en usage normal (tests de scénario, jeu).
    dump: Option<BTreeMap<u32, String>>,
    /// Enregistrements d'une frame déjà vue : resimulations après rollback. En synctest,
    /// chaque frame l'est `check_distance` fois ; en p2p, c'est la mesure des prédictions
    /// ratées (zéro = aucun rollback n'a eu lieu, le run ne prouve rien du rollback réseau).
    pub resimulated: u32,
}

/// Nombre de lignes gardées dans `history` (une frame = jusqu'à trois versions).
const HISTORY_LEN: usize = 400;

/// Une frame enregistrée deux fois (passage initial, puis resimulation après rollback)
/// avec deux états différents : c'est un bug de déterminisme.
#[derive(Debug, Clone)]
pub struct Divergence {
    pub frame: u32,
    pub before: String,
    pub after: String,
}

impl StateTraceRecorder {
    /// Oublie tout ce qui a été enregistré (nouvelle partie, D14).
    pub fn clear(&mut self) {
        self.frames.clear();
        self.divergences.clear();
        self.history.clear();
        if let Some(dump) = self.dump.as_mut() {
            dump.clear();
        }
    }

    /// Lignes des frames `0..end`, dans l'ordre.
    pub fn lines_until(&self, end: u32) -> impl Iterator<Item = &str> {
        self.frames.range(..end).map(|(_, line)| line.as_str())
    }

    /// Comme [`Self::lines_until`], avec la ligne détaillée (hash + détail, comme en mode
    /// `full`) de **toutes** les frames `0..end`, sans la limite de `history`. `None` si
    /// [`StateTraceRecorderPlugin::dump`] n'était pas actif pour ce run.
    pub fn dump_lines_until(&self, end: u32) -> Option<impl Iterator<Item = &str>> {
        self.dump
            .as_ref()
            .map(|frames| frames.range(..end).map(|(_, line)| line.as_str()))
    }

    /// Diagnostic pour les frames que GGRS déclare divergentes (numérotation GGRS) : pour
    /// chaque frame `f` et `f - 1`, toutes les versions enregistrées ; quand deux versions
    /// consécutives ont un checksum différent, les lignes d'état (`full`) présentes d'un
    /// seul côté, pour nommer le composant qui diverge.
    pub fn report_for_frames(&self, frames: &[i32]) -> String {
        let mut out = String::new();
        let mut keys: Vec<u32> = frames
            .iter()
            .flat_map(|f| {
                [
                    (*f - 2).max(0) as u32,
                    (*f - 1).max(0) as u32,
                    (*f).max(0) as u32,
                ]
            })
            .collect();
        keys.sort_unstable();
        keys.dedup();
        for key in keys {
            let versions: Vec<&String> = self
                .history
                .iter()
                .filter(|(f, _)| *f == key)
                .map(|(_, l)| l)
                .collect();
            let _ = writeln!(
                out,
                "  frame {key} : {} version(s) enregistrée(s)",
                versions.len()
            );
            for pair in versions.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let (ha, hb) = (
                    a.lines().next().unwrap_or(""),
                    b.lines().next().unwrap_or(""),
                );
                if ha == hb {
                    continue;
                }
                let _ = writeln!(out, "    checksums : {ha}  →  {hb}");
                if self.full {
                    let sa: std::collections::BTreeSet<&str> = a.lines().skip(1).collect();
                    let sb: std::collections::BTreeSet<&str> = b.lines().skip(1).collect();
                    for l in a.lines().skip(1).filter(|l| !sb.contains(l)).take(10) {
                        let _ = writeln!(out, "    avant : {l}");
                    }
                    for l in b.lines().skip(1).filter(|l| !sa.contains(l)).take(10) {
                        let _ = writeln!(out, "    après : {l}");
                    }
                } else {
                    out.push_str("    (ALACOD_DIAG=1 pour le détail par composant)\n");
                }
            }
        }
        out
    }

    /// Décrit la première divergence : la frame, les deux checksums et, si la trace est
    /// détaillée (`full`), les lignes d'état qui ne sont que d'un côté (composant par
    /// composant), pour nommer ce qui diverge.
    pub fn first_divergence_report(&self) -> Option<String> {
        let d = self.divergences.first()?;
        let mut out = format!(
            "frame {} rejouée différemment après rollback : {} puis {}",
            d.frame,
            d.before.lines().next().unwrap_or(""),
            d.after.lines().next().unwrap_or("")
        );
        if self.full {
            let before: std::collections::BTreeSet<&str> = d.before.lines().skip(1).collect();
            let after: std::collections::BTreeSet<&str> = d.after.lines().skip(1).collect();
            let only_before: Vec<&str> = d
                .before
                .lines()
                .skip(1)
                .filter(|l| !after.contains(l))
                .take(12)
                .collect();
            let only_after: Vec<&str> = d
                .after
                .lines()
                .skip(1)
                .filter(|l| !before.contains(l))
                .take(12)
                .collect();
            let _ = write!(
                out,
                "\n  passage initial :\n    {}\n  resimulation :\n    {}",
                only_before.join("\n    "),
                only_after.join("\n    ")
            );
        } else {
            out.push_str(" (ALACOD_DIAG=1 pour le détail par composant)");
        }
        Some(out)
    }
}

/// Enregistre la trace d'état dans [`StateTraceRecorder`].
pub struct StateTraceRecorderPlugin {
    pub full: bool,
    /// Outil de preuve permanent (T1.2) : quand `Some`, la trace détaillée (hash + détail,
    /// comme en mode `full`) de **toutes** les frames est conservée sans la limite de
    /// `history`, lisible ensuite via [`StateTraceRecorder::dump_lines_until`]. La valeur
    /// elle-même (un dossier, typiquement `ALACOD_DUMP_TRACE`) n'est pas utilisée par ce
    /// plugin : c'est l'appelant (`crates/scenario/tests/scenarios.rs`) qui sait dans quel
    /// fichier — `<dossier>/<scénario>.full` — écrire ces lignes une fois le scénario
    /// terminé (voir `docs/conventions.md` §10 « Blesser une trace : la preuve »).
    pub dump: Option<PathBuf>,
}

impl Plugin for StateTraceRecorderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StateTraceRecorder {
            // Le dump a besoin du détail par ressource/entité, comme `full` : actif dès
            // que `dump` est demandé, même si `full` ne l'est pas par ailleurs (ALACOD_DIAG).
            full: self.full || self.dump.is_some(),
            frames: BTreeMap::new(),
            divergences: Vec::new(),
            dump: self.dump.is_some().then(BTreeMap::new),
            history: std::collections::VecDeque::new(),
            resimulated: 0,
        })
        .add_systems(SaveWorld, record_state.in_set(SaveWorldSystems::Snapshot));
    }
}

/// Systèmes de `Last` qui peuvent demander l'arrêt du jeu (`AppExit`). Ce qui doit réagir
/// à l'arrêt dans le même update (ex. écrire un enregistrement) s'ordonne après.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExitRequests;

/// Frames jouées après l'écriture de `<fichier>-g1` avant de poser `RunRequest::Restart`.
pub const RESTART_GRACE_FRAMES: u32 = 60;

/// Destination fichier de la trace (variables d'environnement).
#[derive(Resource)]
struct StateTraceFile {
    path: PathBuf,
    exit_at_frame: u32,
    written: bool,
    /// D14 : `ALACOD_RESTART_AT_FRAME`.
    restart_at_frame: Option<u32>,
    /// Partie courante (1, puis 2 après le restart).
    game: u32,
    /// `<fichier>-g1` écrit, `RunRequest::Restart` posé.
    first_written: bool,
    restart_requested: bool,
    /// Depuis quand l'écriture attend la confirmation des dernières frames (voir
    /// [`confirmation`]).
    waiting_since: Option<std::time::Instant>,
}

/// Attente maximale de la confirmation des dernières frames avant d'écrire seulement les
/// frames confirmées (pair parti sans avoir confirmé, session perdue).
const CONFIRMATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Ce que [`confirmation`] décide pour une trace qui s'arrête avant `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confirmation {
    /// Des frames avant `end` ne sont pas encore confirmées : attendre (le jeu continue).
    Wait,
    /// Écrire les frames `0..end` (toutes confirmées, ou `end` ramené aux confirmées après
    /// l'attente maximale).
    Write(u32),
}

/// Une frame prédite (entrée d'un pair devinée) peut être resimulée : seule une frame
/// confirmée (`frame <= confirmed`, `ConfirmedFrameCount` de GGRS) est définitive. À l'arrêt
/// demandé, le client en avance sur son pair a ses dernières frames prédites : sans cette
/// attente, sa trace diffère de celle du pair sur ces lignes alors qu'il n'y a pas de desync.
/// `confirmed` vaut `None` hors session. En synctest, la confirmation suit la frame courante
/// avec `check_distance` de retard : l'attente est de quelques frames.
fn confirmation(
    recorded: impl Iterator<Item = u32>,
    confirmed: Option<i32>,
    end: u32,
    waited: std::time::Duration,
) -> Confirmation {
    let Some(last) = recorded.filter(|frame| *frame < end).max() else {
        return Confirmation::Write(end);
    };
    let confirmed = confirmed.unwrap_or(-1);
    if i64::from(confirmed) >= i64::from(last) {
        Confirmation::Write(end)
    } else if waited >= CONFIRMATION_TIMEOUT {
        Confirmation::Write(end.min(u32::try_from(confirmed + 1).unwrap_or(0)))
    } else {
        Confirmation::Wait
    }
}

/// Ce que [`write_trace_at_exit_frame`] fait à cette frame (pur, D14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceStep {
    Nothing,
    /// Écrire la trace de la partie `game` (suffixe `-g{game}` si restart) et quitter.
    WriteAndExit,
    /// Écrire `-g1` (restart à venir).
    WriteFirst,
    /// Poser `RunRequest::Restart`.
    Restart,
}

fn trace_step(
    frame: u32,
    exit_at_frame: u32,
    restart_at_frame: Option<u32>,
    game: u32,
    first_written: bool,
    restart_requested: bool,
) -> TraceStep {
    match restart_at_frame {
        Some(restart_at) if game == 1 => {
            if !first_written && frame >= restart_at {
                TraceStep::WriteFirst
            } else if first_written
                && !restart_requested
                && frame >= restart_at + RESTART_GRACE_FRAMES
            {
                TraceStep::Restart
            } else {
                TraceStep::Nothing
            }
        }
        _ if frame >= exit_at_frame => TraceStep::WriteAndExit,
        _ => TraceStep::Nothing,
    }
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
            // Le dump (T1.2) est propre au harnais de scénarios (`ALACOD_DUMP_TRACE`,
            // voir `crates/scenario/src/runner.rs::build_app`) : ce plugin-ci, piloté par
            // `ALACOD_STATE_TRACE`, écrit déjà un fichier complet à la sortie
            // (`write_trace_at_exit_frame`), pas besoin du dump en plus.
            dump: None,
        })
        .insert_resource(StateTraceFile {
            path: path.into(),
            exit_at_frame,
            written: false,
            restart_at_frame: std::env::var("ALACOD_RESTART_AT_FRAME").ok().map(|v| {
                v.parse()
                    .expect("ALACOD_RESTART_AT_FRAME doit être un entier")
            }),
            game: 1,
            first_written: false,
            restart_requested: false,
            waiting_since: None,
        })
        .add_systems(Last, write_trace_at_exit_frame.in_set(ExitRequests))
        .add_systems(
            OnExit(crate::core::AppState::InGame),
            next_game_after_restart,
        );
    }
}

/// Numéro de frame GGRS de la `SaveGameState` en cours (voir la doc du module). `None`
/// avant que la première frame n'ait été simulée, ou hors session GGRS (schedule
/// `SaveWorld` non déclenché de toute façon dans ce cas).
///
/// `RollbackFrameCount` lu ici vaut systématiquement un de plus que le numéro de frame
/// tel que le jeu (et l'ancienne trace, en `GgrsSchedule`) le compte : vérifié
/// empiriquement en comparant les traces avant/après ce changement de schedule (les
/// entités apparaissent une frame plus tard sans le `- 1`, de façon reproductible à
/// l'identique sur plusieurs runs). `- 1` réaligne les deux numérotations ; documenté ici
/// faute d'avoir trouvé où bevy_ggrs 0.22 le documente lui-même.
fn current_rollback_frame(world: &World) -> Option<u32> {
    let frame = world.get_resource::<RollbackFrameCount>()?.0 - 1;
    u32::try_from(frame).ok()
}

/// Système exclusif (`&mut World`) : les tracers génériques de [`StateTracers`] prennent
/// `&World`/`&World, Entity`, et itérer toutes les entités rollback triées par
/// `GgrsNetId` demande une query ad hoc (`World::query_filtered`, qui exige `&mut World`
/// pour s'enregistrer). Voir la doc du module pour l'ordonnancement dans `SaveWorld`.
fn record_state(world: &mut World) {
    let Some(frame) = current_rollback_frame(world) else {
        return;
    };

    let mut rollback_entities = world.query_filtered::<(&GgrsNetId, Entity), With<Rollback>>();
    let mut entities: Vec<(GgrsNetId, Entity)> = rollback_entities
        .iter(world)
        .map(|(id, entity)| (id.clone(), entity))
        .collect();
    entities.sort_unstable_by_key(|(id, _)| id.0);

    let checksum = world.resource::<Checksum>().0;
    let full = world.resource::<StateTraceRecorder>().full;

    // La ligne de trace (comparée aux références) ne porte que le hash ; le détail va dans
    // `history`, pour les diagnostics.
    let hash_line = format!("{frame} {checksum:032x} {}", entities.len());
    let mut line = hash_line.clone();
    if full {
        line.push('\n');
        // Emprunt partagé pour toute la phase de lecture : tracers + monde restent tous
        // les deux accessibles en lecture simultanément (voir la doc de `record_state`).
        let world: &World = world;
        let tracers = world.resource::<StateTracers>();
        for &(name, tracer) in &tracers.resources {
            if let Some(value) = tracer(world) {
                let _ = writeln!(line, "{name}={value}");
            }
        }
        for (id, entity) in &entities {
            let _ = write!(line, "{id:?}");
            for &(name, tracer) in &tracers.components {
                if let Some(value) = tracer(world, *entity) {
                    let _ = write!(line, " {name}={value}");
                }
            }
            let _ = writeln!(line);
        }
    }

    let mut recorder = world.resource_mut::<StateTraceRecorder>();
    if recorder.history.len() >= HISTORY_LEN {
        recorder.history.pop_front();
    }
    recorder.history.push_back((frame, line.clone()));
    if let Some(dump) = recorder.dump.as_mut() {
        dump.insert(frame, line.clone());
    }
    if let Some(previous) = recorder.frames.get(&frame).cloned() {
        recorder.resimulated += 1;
        if previous != hash_line && recorder.divergences.len() < 3 {
            let before = previous;
            recorder.divergences.push(Divergence {
                frame,
                before,
                after: line,
            });
        }
    }
    recorder.frames.insert(frame, hash_line);
}

/// D14 : à la sortie de la première partie (restart demandé), la trace repart de zéro. Aussi à
/// la sortie du hub (M2-T13, `RunRequest::EnterRun`) : les frames de la run repartent de 0 et ne
/// doivent ni se mêler à celles du hub ni compter comme des frames resimulées.
fn next_game_after_restart(
    mut file: ResMut<StateTraceFile>,
    mut trace: ResMut<StateTraceRecorder>,
) {
    if file.restart_requested && file.game == 1 {
        file.game = 2;
    }
    trace.clear();
}

/// Avec `ALACOD_STATE_TRACE_FULL=1` : écrit aussi `<fichier>.full`, le détail des dernières
/// frames enregistrées (`history`, dernière version de chaque frame `< end`), pour trouver
/// l'état qui diffère entre deux traces (`scripts/trace-diff.py`).
fn write_full_history(path: &std::path::Path, trace: &StateTraceRecorder, end: u32) {
    if !trace.full {
        return;
    }
    let mut last: BTreeMap<u32, &str> = BTreeMap::new();
    for (frame, line) in &trace.history {
        if *frame < end {
            last.insert(*frame, line);
        }
    }
    let out: String = last.values().map(|line| format!("{line}\n")).collect();
    let mut name = path.as_os_str().to_owned();
    name.push(".full");
    std::fs::write(PathBuf::from(name), out).expect("écriture de la trace détaillée");
}

/// Fichier de la trace de la partie `game` : `<fichier>-g{game}` en mode restart.
fn trace_path(path: &std::path::Path, restart: bool, game: u32) -> PathBuf {
    if restart {
        let mut name = path.as_os_str().to_owned();
        name.push(format!("-g{game}"));
        PathBuf::from(name)
    } else {
        path.to_path_buf()
    }
}

/// Fin (exclue) des frames à écrire, ou `None` tant que les dernières ne sont pas confirmées.
fn confirmed_end(
    file: &mut StateTraceFile,
    trace: &StateTraceRecorder,
    confirmed: Option<&bevy_ggrs::ConfirmedFrameCount>,
    end: u32,
) -> Option<u32> {
    let since = *file
        .waiting_since
        .get_or_insert_with(std::time::Instant::now);
    match confirmation(
        trace.frames.keys().copied(),
        confirmed.map(|c| c.0),
        end,
        since.elapsed(),
    ) {
        Confirmation::Wait => None,
        Confirmation::Write(written_end) => {
            if written_end < end {
                warn!(
                    "trace d'état tronquée à la frame {written_end} : dernières frames jamais confirmées"
                );
            }
            file.waiting_since = None;
            Some(written_end)
        }
    }
}

/// Nombre de lignes d'une trace qui s'arrête à la frame `at_frame` : `at_frame - 1`, toujours.
/// La frame `at_frame - 1` n'est jamais écrite : `record_state` vit dans `SaveWorld`, et la
/// `SaveGameState` de cette frame n'a pas encore été traitée quand le système voit `FrameCount`
/// atteindre `at_frame` (constaté, reproductible en synctest). Un client en avance sur son pair,
/// qui attend la confirmation (voir `confirmation`) pendant que sa frame courante avance,
/// écrirait sinon une ligne de plus que le pair.
fn last_trace_line(at_frame: u32) -> u32 {
    at_frame.saturating_sub(1)
}

fn write_trace_at_exit_frame(
    mut commands: Commands,
    frame: Res<FrameCount>,
    trace: Res<StateTraceRecorder>,
    mut file: ResMut<StateTraceFile>,
    mut exit: MessageWriter<AppExit>,
    snapshot_profile: Option<Res<utils::snapshot_profile::SnapshotProfile>>,
    set_timings: Option<Res<utils::system_profile::SetTimings>>,
    confirmed: Option<Res<bevy_ggrs::ConfirmedFrameCount>>,
) {
    if file.written {
        return;
    }
    let step = trace_step(
        frame.frame,
        file.exit_at_frame,
        file.restart_at_frame,
        file.game,
        file.first_written,
        file.restart_requested,
    );
    let restart = file.restart_at_frame.is_some();
    match step {
        TraceStep::Nothing => return,
        TraceStep::Restart => {
            info!(
                "ALACOD_RESTART_AT_FRAME : RunRequest::Restart à la frame {}",
                frame.frame
            );
            commands.insert_resource(crate::run_state::RunRequest::Restart);
            file.restart_requested = true;
            return;
        }
        TraceStep::WriteFirst => {
            // Même plafond que la trace de sortie : l'attente de confirmation fait avancer la
            // simulation, la frame `restart_at - 1` n'est pas fiable (voir `last_trace_line`).
            let restart_at = last_trace_line(file.restart_at_frame.unwrap_or(0));
            let Some(end) = confirmed_end(&mut file, &trace, confirmed.as_deref(), restart_at)
            else {
                return;
            };
            let path = trace_path(&file.path, true, 1);
            let out: String = trace
                .lines_until(end)
                .map(|line| format!("{line}\n"))
                .collect();
            std::fs::write(&path, out).expect("écriture de la trace d'état");
            write_full_history(&path, &trace, end);
            info!("trace d'état de la partie 1 écrite dans {path:?}");
            file.first_written = true;
            return;
        }
        TraceStep::WriteAndExit => {}
    }
    let exit_at = last_trace_line(file.exit_at_frame);
    let Some(end) = confirmed_end(&mut file, &trace, confirmed.as_deref(), exit_at) else {
        return;
    };
    let mut out = String::new();
    for line in trace.lines_until(end) {
        out.push_str(line);
        out.push('\n');
    }
    let path = trace_path(&file.path, restart, file.game);
    std::fs::write(&path, out).expect("écriture de la trace d'état");
    write_full_history(&path, &trace, end);
    info!(
        "trace d'état écrite dans {path:?} ; frames resimulées après rollback : {}",
        trace.resimulated
    );
    if let Some(profile) = snapshot_profile {
        info!("profil rollback : {}", profile.summary());
        for sample in profile.heaviest(5) {
            info!(
                "profil rollback : {} ×{} ~{} octets",
                sample.name, sample.count, sample.bytes
            );
        }
    }
    if let Some(timings) = set_timings {
        for row in timings.report() {
            info!("profil sets : {row}");
        }
    }
    file.written = true;
    exit.write(AppExit::Success);
}

#[cfg(test)]
mod restart_tests {
    use super::*;

    #[test]
    fn deux_parties() {
        // Sans restart : inchangé.
        assert_eq!(
            trace_step(599, 600, None, 1, false, false),
            TraceStep::Nothing
        );
        assert_eq!(
            trace_step(600, 600, None, 1, false, false),
            TraceStep::WriteAndExit
        );
        // Partie 1 : -g1 à 600, restart 60 frames plus tard, jamais d'arrêt.
        let step = |frame, first, requested| trace_step(frame, 400, Some(600), 1, first, requested);
        assert_eq!(
            step(450, false, false),
            TraceStep::Nothing,
            "exit ignoré en partie 1"
        );
        assert_eq!(step(600, false, false), TraceStep::WriteFirst);
        assert_eq!(step(620, true, false), TraceStep::Nothing);
        assert_eq!(step(660, true, false), TraceStep::Restart);
        assert_eq!(step(661, true, true), TraceStep::Nothing);
        // Partie 2 : exit relatif à son début.
        assert_eq!(
            trace_step(399, 400, Some(600), 2, true, true),
            TraceStep::Nothing
        );
        assert_eq!(
            trace_step(400, 400, Some(600), 2, true, true),
            TraceStep::WriteAndExit
        );
    }

    #[test]
    fn plafond_des_lignes_independant_de_la_simulation() {
        use std::time::Duration;
        assert_eq!(last_trace_line(600), 599);
        assert_eq!(last_trace_line(0), 0);
        // La simulation a avancé pendant l'attente (trace jusqu'à 602, tout confirmé) :
        // la partie 1 (`-g1`) comme la sortie s'arrêtent à 599 lignes, jamais 600.
        assert_eq!(
            confirmation(0..602, Some(601), last_trace_line(600), Duration::ZERO),
            Confirmation::Write(599)
        );
    }

    #[test]
    fn attend_la_confirmation_des_dernieres_frames() {
        use std::time::Duration;
        let zero = Duration::ZERO;
        // Confirmé jusqu'à la dernière frame enregistrée : tout s'écrit.
        assert_eq!(
            confirmation(0..598, Some(597), 600, zero),
            Confirmation::Write(600)
        );
        // Frames prédites en fin de trace : on attend.
        assert_eq!(
            confirmation(0..598, Some(590), 600, zero),
            Confirmation::Wait
        );
        // Pair parti : après l'attente maximale, seulement les frames confirmées.
        assert_eq!(
            confirmation(0..598, Some(590), 600, CONFIRMATION_TIMEOUT),
            Confirmation::Write(591)
        );
        assert_eq!(
            confirmation(0..598, None, 600, CONFIRMATION_TIMEOUT),
            Confirmation::Write(0)
        );
        // Les frames au-delà de `end` ne comptent pas.
        assert_eq!(
            confirmation(0..700, Some(599), 600, zero),
            Confirmation::Write(600)
        );
    }

    #[test]
    fn suffixe_des_fichiers() {
        let path = std::path::Path::new("/tmp/p2p-0.trace");
        assert_eq!(
            trace_path(path, false, 1),
            PathBuf::from("/tmp/p2p-0.trace")
        );
        assert_eq!(
            trace_path(path, true, 1),
            PathBuf::from("/tmp/p2p-0.trace-g1")
        );
        assert_eq!(
            trace_path(path, true, 2),
            PathBuf::from("/tmp/p2p-0.trace-g2")
        );
    }
}
