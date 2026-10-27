//! Musique calée sur l'horloge de simulation (X-3, présentation seule, `docs/conventions.md` §90).
//!
//! `ui/feedback.ron` peut déclarer `music: Some((track, clock, event, loop_events, max_drift_ms,
//! volume_db))` : une piste (`ogg`) qui boucle sur `loop_events` occurrences de l'événement
//! `event` de l'horloge `clock` (kind `Clock`, §23), sa position 0 tombant sur une occurrence.
//! [`sync_music`] ne fait que **lire** `FrameCount`, `Clock` et `ClockSchedule` : la simulation ne
//! dépend ni de la piste ni de la carte son (elle tourne aussi sans périphérique audio). La position
//! attendue de la piste est celle de l'horloge **de simulation** : une pause (la simulation ne
//! progresse plus) met la piste en pause, une relance ou un rollback (le temps recule) la replace, et
//! un écart supérieur à `max_drift_ms` entre la position jouée et la position attendue est rattrapé
//! par un déplacement dans la piste. `ALACOD_MUTE=1` (captures, scénarios) : rien n'est joué.
//!
//! La piste doit durer exactement `loop_events × repeat` frames de l'événement (à 60 Hz) : sinon la
//! boucle de la piste dérive de la boucle de l'horloge et chaque tour est rattrapé.

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use run::Clock;
use utils::frame::FrameCount;

use crate::clock::ClockSchedule;
use crate::feedback_log::FeedbackConfigLoaded;

/// Frames de simulation par seconde (`run::clock::FRAMES_PER_SECOND`).
const FPS: f64 = run::clock::FRAMES_PER_SECOND as f64;

/// Temps réel sans nouvelle frame simulée au bout duquel la simulation est considérée à l'arrêt
/// (pause, fenêtre figée) : la piste est alors mise en pause.
const STALL_SECS: f64 = 0.25;

/// Une ligne de mesure (`music f… position=… expected=… drift_ms=…`) par seconde de temps réel.
const LOG_EVERY_SECS: f64 = 1.0;

/// Position, en frames, dans une boucle de `loop_frames` frames dont l'origine tombe à
/// `at` (frame de l'horloge) pour le temps écoulé `elapsed` (frames). Avant `at` (avant la première
/// occurrence) la position compte à rebours : la piste est déjà en cours. Pure, entiers.
pub fn loop_position_frames(elapsed: u32, at: u32, loop_frames: u32) -> u32 {
    let loop_frames = i64::from(loop_frames.max(1));
    (i64::from(elapsed) - i64::from(at)).rem_euclid(loop_frames) as u32
}

/// Écart signé (secondes) entre la position jouée et la position attendue dans une boucle de
/// `loop_secs` : le plus court chemin autour de la boucle, dans `]-loop/2, loop/2]`.
pub fn wrapped_drift(actual: f64, expected: f64, loop_secs: f64) -> f64 {
    let mut drift = (actual - expected).rem_euclid(loop_secs);
    if drift > loop_secs / 2.0 {
        drift -= loop_secs;
    }
    drift
}

/// Ce que [`sync_music`] fait de la piste pour cette image rendue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicAction {
    /// Rien : la piste est là où l'horloge l'attend.
    Keep,
    /// Écart supérieur à la tolérance : se placer à la position attendue (secondes).
    Seek(f64),
    /// La simulation est à l'arrêt : mettre la piste en pause.
    Pause,
    /// La simulation reprend : relancer la piste à la position attendue (secondes).
    Resume(f64),
}

/// Décision pure : piste en lecture à `played` (secondes) ou en pause, simulation à l'arrêt ou non.
pub fn decide(
    played: f64,
    paused: bool,
    stalled: bool,
    expected: f64,
    loop_secs: f64,
    max_drift_secs: f64,
) -> MusicAction {
    match (paused, stalled) {
        (false, true) => MusicAction::Pause,
        (true, true) => MusicAction::Keep,
        (true, false) => MusicAction::Resume(expected),
        (false, false) => {
            if wrapped_drift(played, expected, loop_secs).abs() > max_drift_secs {
                MusicAction::Seek(expected)
            } else {
                MusicAction::Keep
            }
        }
    }
}

/// Mémoire de présentation de la piste (hors simulation, jamais rollbackée).
#[derive(Default)]
pub struct MusicState {
    instance: Option<Handle<AudioInstance>>,
    paused: bool,
    last_frame: Option<u32>,
    last_change_secs: f64,
    last_log_secs: f64,
}

/// Lance, cale et arrête la piste de `music` (voir le module). Une image rendue sans nouvelle frame
/// simulée ajoute la fraction de frame écoulée (au plus une frame) à la position attendue.
#[allow(clippy::too_many_arguments)]
pub fn sync_music(
    config: Option<Res<FeedbackConfigLoaded>>,
    schedule: Option<Res<ClockSchedule>>,
    clock: Res<Clock>,
    frame: Res<FrameCount>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut instances: ResMut<Assets<AudioInstance>>,
    time: Res<Time<Real>>,
    mut state: Local<MusicState>,
) {
    let wanted = config
        .as_ref()
        .and_then(|config| config.0.music.as_ref())
        .and_then(|music| {
            let (_, def) = schedule
                .as_ref()?
                .clocks
                .iter()
                .find(|(name, _)| *name == music.clock)?;
            let event = def.events.iter().find(|event| event.id == music.event)?;
            let period = event.repeat?.frames();
            (period > 0).then_some((music, def.scope, event.at.frames(), period))
        });
    let Some((music, scope, at, period)) = wanted else {
        // Plus de piste demandée (retour au lobby, autre partie sans horloge) : arrêt.
        if let Some(handle) = state.instance.take() {
            if let Some(mut instance) = instances.get_mut(&handle) {
                instance.stop(AudioTween::default());
            }
            state.paused = false;
            state.last_frame = None;
        }
        return;
    };
    if crate::feedback::muted() {
        return;
    }

    let now_secs = time.elapsed_secs_f64();
    let sim_frame = frame.frame;
    if state.last_frame != Some(sim_frame) {
        state.last_frame = Some(sim_frame);
        state.last_change_secs = now_secs;
    }
    let since_change = now_secs - state.last_change_secs;
    let stalled = since_change > STALL_SECS;

    let loop_frames = period.saturating_mul(music.loop_events.max(1));
    let loop_secs = f64::from(loop_frames) / FPS;
    let elapsed = clock.scope_frames(scope, sim_frame);
    let expected = f64::from(loop_position_frames(elapsed, at, loop_frames)) / FPS
        + since_change.clamp(0.0, 1.0 / FPS);
    let expected = expected.rem_euclid(loop_secs);
    let max_drift = f64::from(music.max_drift_ms) / 1000.0;

    let Some(handle) = state.instance.clone() else {
        if stalled {
            return;
        }
        let handle = audio
            .play(asset_server.load(music.track.clone()))
            .looped()
            .with_volume(music.volume_db)
            .start_from(expected)
            .handle();
        info!(
            "music f{} start track={} expected={:.3}",
            sim_frame, music.track, expected
        );
        state.instance = Some(handle);
        state.paused = false;
        state.last_log_secs = now_secs;
        return;
    };
    let Some(mut instance) = instances.get_mut(&handle) else {
        return;
    };
    let played = match instance.state() {
        PlaybackState::Playing { position } | PlaybackState::Paused { position } => position,
        // En file (piste pas encore chargée), en cours d'arrêt ou perdue : on retente à l'image suivante.
        PlaybackState::Stopped => {
            state.instance = None;
            return;
        }
        _ => return,
    };
    match decide(
        played,
        state.paused,
        stalled,
        expected,
        loop_secs,
        max_drift,
    ) {
        MusicAction::Keep => {}
        MusicAction::Seek(position) => {
            info!(
                "music f{} resync drift_ms={:.1}",
                sim_frame,
                wrapped_drift(played, expected, loop_secs) * 1000.0
            );
            instance.seek_to(position);
        }
        MusicAction::Pause => {
            instance.pause(AudioTween::default());
            state.paused = true;
            info!("music f{} pause (simulation à l'arrêt)", sim_frame);
        }
        MusicAction::Resume(position) => {
            instance.seek_to(position);
            instance.resume(AudioTween::default());
            state.paused = false;
            info!("music f{} resume expected={:.3}", sim_frame, position);
        }
    }
    if now_secs - state.last_log_secs >= LOG_EVERY_SECS {
        state.last_log_secs = now_secs;
        info!(
            "music f{} position={:.3} expected={:.3} drift_ms={:.1}",
            sim_frame,
            played,
            expected,
            wrapped_drift(played, expected, loop_secs) * 1000.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_dans_la_boucle() {
        // Origine à 40, boucle de 80 frames : f40 = 0, f79 = 39, f120 = 0 (tour suivant).
        assert_eq!(loop_position_frames(40, 40, 80), 0);
        assert_eq!(loop_position_frames(79, 40, 80), 39);
        assert_eq!(loop_position_frames(120, 40, 80), 0);
        // Avant la première occurrence : la piste compte à rebours depuis la fin de la boucle.
        assert_eq!(loop_position_frames(0, 40, 80), 40);
        assert_eq!(loop_position_frames(39, 40, 80), 79);
        // Boucle nulle : jamais de division par zéro.
        assert_eq!(loop_position_frames(10, 0, 0), 0);
        // Extrêmes sans panique.
        let _ = loop_position_frames(u32::MAX, 0, 3);
    }

    #[test]
    fn ecart_par_le_plus_court_chemin() {
        let loop_secs = 1.0;
        assert!((wrapped_drift(0.05, 0.0, loop_secs) - 0.05).abs() < 1e-9);
        assert!((wrapped_drift(0.0, 0.05, loop_secs) + 0.05).abs() < 1e-9);
        // Autour du tour : 0,98 joué pour 0,02 attendu = 40 ms de retard... en arrière.
        assert!((wrapped_drift(0.98, 0.02, loop_secs) + 0.04).abs() < 1e-9);
        assert!((wrapped_drift(0.02, 0.98, loop_secs) - 0.04).abs() < 1e-9);
    }

    #[test]
    fn decisions_de_la_piste() {
        let (loop_secs, tol) = (1.0, 0.06);
        // Dans la tolérance : rien ; au-delà : se replacer.
        assert_eq!(
            decide(0.50, false, false, 0.53, loop_secs, tol),
            MusicAction::Keep
        );
        assert_eq!(
            decide(0.50, false, false, 0.60, loop_secs, tol),
            MusicAction::Seek(0.60)
        );
        // Le tour de boucle ne compte pas comme un écart.
        assert_eq!(
            decide(0.99, false, false, 0.01, loop_secs, tol),
            MusicAction::Keep
        );
        // Simulation à l'arrêt : pause une fois, puis rien ; à la reprise : replacement exact.
        assert_eq!(
            decide(0.5, false, true, 0.9, loop_secs, tol),
            MusicAction::Pause
        );
        assert_eq!(
            decide(0.5, true, true, 0.9, loop_secs, tol),
            MusicAction::Keep
        );
        assert_eq!(
            decide(0.5, true, false, 0.9, loop_secs, tol),
            MusicAction::Resume(0.9)
        );
    }
}
