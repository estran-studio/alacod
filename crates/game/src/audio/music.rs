//! Musique générative du jeu (feature `harmonium`, M-7, `docs/conventions.md` §121).
//!
//! **Présentation seule.** Rien ici ne tourne dans `GgrsSchedule`, aucun type n'est rollback et la simulation ne
//! lit jamais `MusicClock` (principe II d'harmonium : le jeu lit sa propre horloge, `FrameCount`). C'est la
//! musique qui **suit** la simulation (option a de William, 2026-11-06) :
//!
//! - à l'entrée en partie (`OnEnter(InGame)`), [`anchor_music_clock`] pose `GameClock::clock` : 60 Hz, ancrage =
//!   `FrameCount` du moment (la pulsation 1 de la mesure 1 commence à cette frame), période de la pulsation dansée
//!   du profil en fraction de frames (donnée du jeu : `music` de `game.ron`) ;
//! - à chaque frame de présentation, [`feed_music_clock`] écrit `GameClock::sim_frame` depuis `FrameCount`, en
//!   `Update`, donc avant `HarmoniumSet` (`PostUpdate`) ;
//! - à la sortie de partie, [`release_music_clock`] retire l'horloge (le tempo revient aux émotions).
//!
//! Sans `music` dans le manifeste du jeu, le plugin n'est pas installé : pas de musique. `ALACOD_MUTE=1` pose le
//! volume à 0 (la musique tourne, muette : l'horloge prouve la sortie sans écoute).

use bevy::prelude::*;
use content::manifest::{GameManifest, MusicEntry};
use harmonium_bevy::{
    ExternalClock, GameClock, HarmoniumConfig, HarmoniumControl, HarmoniumPlugin, HarmoniumStatus,
    MusicClock, Phase, StyleProfile,
};
use harmonium_core::time_plan::{BarTiming, Swing, Tempo};
use utils::frame::FrameCount;

use crate::session::{AppState, SIM_FPS};

/// Période de la pulsation dansée, en frames de simulation (`num / den`), lue dans `music.pulse_frames`.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicPulse {
    pub num: u32,
    pub den: u32,
}

/// Fréquence de simulation passée à l'horloge externe (60 Hz).
fn sim_hz() -> u32 {
    SIM_FPS as u32
}

/// Installe la musique si le manifeste du jeu la demande. Refus (profil inconnu, période nulle, tempo hors de 40 à
/// 300 BPM pour le mètre du profil) : journal, pas de musique, le jeu continue.
pub fn add_music(app: &mut App) {
    let Some(entry) = app
        .world()
        .get_resource::<GameManifest>()
        .and_then(|manifest| manifest.music.clone())
    else {
        return;
    };
    let (control, config, pulse) = match prepare(&entry, crate::feedback::muted()) {
        Ok(prepared) => prepared,
        Err(message) => {
            warn!("musique : {message} ; aucune musique");
            return;
        }
    };
    info!(
        "musique : profil « {} », pulsation {}/{} frames, volume {}",
        entry.profile, pulse.num, pulse.den, control.volume
    );
    app.insert_resource(control)
        .insert_resource(pulse)
        .add_plugins(HarmoniumPlugin::new(config))
        .add_systems(OnEnter(AppState::InGame), anchor_music_clock)
        .add_systems(OnExit(AppState::InGame), release_music_clock)
        .add_systems(Update, (feed_music_clock, log_music_status));
}

/// Cibles et configuration d'harmonium à partir de la donnée du jeu. `muted` : volume 0 (`ALACOD_MUTE=1`).
fn prepare(
    entry: &MusicEntry,
    muted: bool,
) -> Result<(HarmoniumControl, HarmoniumConfig, MusicPulse), String> {
    let pulse = MusicPulse {
        num: entry.pulse_frames.num,
        den: entry.pulse_frames.den,
    };
    // Même refus que l'horloge au moment de la poser : une période nulle échoue ici, pas au premier match.
    let clock =
        ExternalClock::new(sim_hz(), 0, pulse.num, pulse.den).map_err(|error| error.to_string())?;
    let mut control = HarmoniumControl::default();
    control
        .set_profile_id(&entry.profile)
        .map_err(|error| format!("profil « {} » : {error}", entry.profile))?;
    profile_tempo(&clock, &control.profile).map_err(|bpm| {
        format!(
            "pulsation de {}/{} frames à {} Hz : {bpm} BPM à la noire pour le profil « {} » ({}/{}), hors de 40 à 300 BPM",
            pulse.num,
            pulse.den,
            sim_hz(),
            entry.profile,
            control.profile.meter.time_signature.numerator,
            control.profile.meter.time_signature.denominator,
        )
    })?;
    if muted {
        control.volume = 0.0;
    }
    let config = HarmoniumConfig {
        main_bank: Some(entry.main_bank.clone()),
        percussion_bank: Some(entry.percussion_bank.clone()),
        ..default()
    };
    Ok((control, config, pulse))
}

/// Tempo à la noire que l'hôte d'harmonium installera pour cette horloge sur le mètre du profil : le même calcul que
/// `LiveHost::external_tempo` (`ExternalClock::tempo_for`, bornes de `MusicalParams::bpm`). `Err` : le tempo arrondi
/// au BPM, hors de 40 à 300 (l'hôte refuserait l'horloge au premier tour, la musique ne partirait jamais).
fn profile_tempo(clock: &ExternalClock, profile: &StyleProfile) -> Result<Tempo, u64> {
    let meter = &profile.meter;
    let timing = BarTiming {
        tempo: Tempo::MIN,
        swing: Swing::STRAIGHT,
        numerator: meter.time_signature.numerator as u16,
        denominator: meter.time_signature.denominator as u16,
        steps: meter.steps_per_bar as u16,
        steps_per_pulse: meter.steps_per_pulse as u8,
        pulses_per_bar: meter.pulses_per_bar as u8,
        ramp_in: false,
    };
    clock.tempo_for(&timing).ok_or_else(|| {
        // Même formule, sans borne, pour l'erreur : noires par pulsation × pulsations par minute.
        let quarters = u64::from(timing.steps_per_pulse) * u64::from(timing.numerator.max(1)) * 4;
        let (num, den) = clock.pulse_frames();
        let per = u64::from(timing.steps.max(1)) * u64::from(timing.denominator.max(1));
        (60 * quarters * u64::from(den) * u64::from(sim_hz()) + per * u64::from(num) / 2)
            / (per * u64::from(num))
    })
}

/// L'horloge musicale d'une partie : ancrée à la frame où elle commence.
fn music_clock(anchor_frame: u32, pulse: MusicPulse) -> Result<ExternalClock, String> {
    ExternalClock::new(sim_hz(), u64::from(anchor_frame), pulse.num, pulse.den)
        .map_err(|error| error.to_string())
}

/// `OnEnter(InGame)` : la mesure 1 commence à la frame courante (`FrameCount` est remis à zéro à la sortie de la partie
/// précédente : 0 d'ordinaire ; en ligne, les deux clients entrent à la même frame de simulation).
pub fn anchor_music_clock(
    frame: Res<FrameCount>,
    pulse: Res<MusicPulse>,
    mut game_clock: ResMut<GameClock>,
) {
    match music_clock(frame.frame, *pulse) {
        Ok(clock) => game_clock.clock = Some(clock),
        Err(message) => warn!("musique : horloge refusée ({message})"),
    }
    game_clock.sim_frame = u64::from(frame.frame);
}

/// `OnExit(InGame)` : plus d'horloge, la musique ne suit plus rien (la prochaine partie en pose une autre).
pub fn release_music_clock(mut game_clock: ResMut<GameClock>) {
    game_clock.clock = None;
}

/// `Update` (avant `HarmoniumSet`, qui est en `PostUpdate`) : la frame de simulation qu'on présente.
pub fn feed_music_clock(frame: Res<FrameCount>, mut game_clock: ResMut<GameClock>) {
    let sim_frame = u64::from(frame.frame);
    if game_clock.sim_frame != sim_frame {
        game_clock.sim_frame = sim_frame;
    }
}

/// Ce qui déclenche une ligne du journal de la musique : comparé à chaque frame **sans allouer** (`Copy`) ; la ligne,
/// elle, n'est formatée qu'au changement ou toutes les 5 s.
#[derive(Clone, Copy, Debug, PartialEq)]
struct MusicLogKey {
    /// Discriminant de `HarmoniumStatus` (0 chargement, 1 prêt, 2 échec) : le message d'un échec ne change pas sans
    /// changement d'état.
    status: u8,
    valid: bool,
    playing: bool,
    phase: Option<Phase>,
}

impl MusicLogKey {
    fn of(status: &HarmoniumStatus, clock: &MusicClock) -> Self {
        Self {
            status: match status {
                HarmoniumStatus::Loading => 0,
                HarmoniumStatus::Ready => 1,
                HarmoniumStatus::Failed(_) => 2,
            },
            valid: clock.valid,
            playing: clock.playing,
            phase: clock.follow.map(|follow| follow.phase),
        }
    }
}

/// Journal de l'état de la musique (présentation, jamais lu par la simulation) : à chaque changement d'état
/// (chargement, hôte prêt, lecture, phase du suivi) puis toutes les 5 s de temps réel.
fn log_music_status(
    status: Res<HarmoniumStatus>,
    clock: Res<MusicClock>,
    game_clock: Res<GameClock>,
    frame: Res<FrameCount>,
    time: Res<Time<Real>>,
    mut last: Local<Option<(MusicLogKey, f32)>>,
) {
    if let Some(line) = music_status_line(
        &status,
        &clock,
        game_clock.clock.is_some(),
        frame.frame,
        time.elapsed_secs(),
        &mut last,
    ) {
        info!("{line}");
    }
}

/// La ligne du journal si elle est due, sinon `None` **sans aucune allocation** (test `le_journal_n_alloue_qu_au_changement`).
fn music_status_line(
    status: &HarmoniumStatus,
    clock: &MusicClock,
    game_clock: bool,
    frame: u32,
    now: f32,
    last: &mut Option<(MusicLogKey, f32)>,
) -> Option<String> {
    let key = MusicLogKey::of(status, clock);
    let due = match *last {
        Some((previous, at)) => previous != key || now - at >= 5.0,
        None => true,
    };
    if !due {
        return None;
    }
    *last = Some((key, now));
    let follow = clock.follow.map(|follow| {
        format!(
            "{:?} écart={} nudges={} sauts={}",
            follow.phase,
            follow
                .gap_ms()
                .map_or_else(|| "?".to_string(), |gap| format!("{gap:.1}ms")),
            follow.nudges,
            follow.resyncs
        )
    });
    Some(format!(
        "musique f{frame} {status:?} valid={} playing={} suivi={} mesure={} pulsation={}/{} bpm={:.1} horloge_jeu={game_clock} statut_externe={}",
        clock.valid,
        clock.playing,
        if follow.is_some() { "oui" } else { "aucun" },
        clock.bar,
        clock.pulse,
        clock.pulses_per_bar,
        clock.bpm,
        follow.as_deref().unwrap_or("aucun"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use content::manifest::PulseFrames;
    use harmonium_audio::realtime::rt_check;

    fn entry(profile: &str, num: u32, den: u32) -> MusicEntry {
        MusicEntry {
            profile: profile.to_string(),
            pulse_frames: PulseFrames { num, den },
            main_bank: "harmonium/a.sf3".to_string(),
            percussion_bank: "harmonium/b.sf2".to_string(),
        }
    }

    /// P3 (revue du merge `00fc97f1`) : le journal formatait l'état (cinq `String`) à **chaque** frame pour le comparer
    /// au précédent. Il compare maintenant une clé `Copy` : aucune allocation tant que rien ne change, une ligne (donc
    /// des allocations) au changement d'état et toutes les 5 s. Preuve par l'allocateur global d'harmonium
    /// (`RTCheckAllocator`, installé dans tout binaire qui lie `harmonium_host` avec `debug_assertions`, dont le profil
    /// `headless`) : entre `enter_audio_context` et `exit_audio_context`, toute allocation ou libération du thread
    /// panique (« REAL-TIME VIOLATION ») et fait échouer le binaire de test.
    #[test]
    fn le_journal_n_alloue_qu_au_changement() {
        let mut clock = MusicClock {
            valid: true,
            playing: true,
            bar: 3,
            pulse: 2,
            pulses_per_bar: 4,
            bpm: 120.0,
            follow: Some(harmonium_bevy::ExternalStatus {
                phase: Phase::Tracking,
                gap: Some(1_000),
                last_correction: None,
                nudges: 4,
                resyncs: 1,
            }),
            ..default()
        };
        let mut last = None;
        let line = music_status_line(&HarmoniumStatus::Ready, &clock, true, 10, 0.0, &mut last);
        let line = line.expect("première ligne");
        assert!(
            line.contains("Ready valid=true playing=true suivi=oui"),
            "{line}"
        );
        assert!(line.contains("statut_externe=Tracking"), "{line}");

        // 299 frames sans changement (5 s à 60 Hz), sous la garde temps réel : aucune ligne, aucune allocation.
        let mut lines = 0;
        rt_check::enter_audio_context();
        for i in 1..300 {
            clock.bar = 3 + (i / 120) as usize;
            let line = music_status_line(
                &HarmoniumStatus::Ready,
                &clock,
                true,
                10 + i,
                i as f32 / 60.0,
                &mut last,
            );
            lines += usize::from(line.is_some());
        }
        rt_check::exit_audio_context();
        assert_eq!(lines, 0, "aucune ligne sans changement d'état");

        // Changement de phase du suivi : une ligne.
        clock.follow.as_mut().unwrap().phase = Phase::Starting;
        let line = music_status_line(&HarmoniumStatus::Ready, &clock, true, 310, 5.0, &mut last);
        assert!(line.is_some_and(|l| l.contains("statut_externe=Starting")));
        // Puis 5 s de temps réel : la ligne périodique.
        let quiet = music_status_line(&HarmoniumStatus::Ready, &clock, true, 311, 9.9, &mut last);
        assert!(quiet.is_none());
        let periodic =
            music_status_line(&HarmoniumStatus::Ready, &clock, true, 312, 10.0, &mut last);
        assert!(periodic.is_some());
    }

    #[test]
    fn un_profil_inconnu_ou_une_periode_nulle_est_refuse() {
        assert!(prepare(&entry("pas_un_profil", 30, 1), false).is_err());
        assert!(prepare(&entry("gmd_pop", 0, 1), false).is_err());
        assert!(prepare(&entry("gmd_pop", 30, 0), false).is_err());
    }

    /// La période est jugée en tempo à la noire **sur le mètre du profil** (règle de l'hôte, 40 à 300 BPM) :
    /// refusée au lancement (erreur claire, pas de musique) au lieu d'un échec de l'hôte à la pose de l'horloge.
    #[test]
    fn une_periode_hors_de_40_a_300_bpm_pour_le_profil_est_refusee() {
        // 4/4 (pulsation = noire) : 12 frames = 300 BPM, 90 frames = 40 BPM, bornes comprises.
        assert!(prepare(&entry("gmd_pop", 12, 1), false).is_ok());
        assert!(prepare(&entry("gmd_pop", 90, 1), false).is_ok());
        let trop_vite = prepare(&entry("gmd_pop", 11, 1), false)
            .err()
            .expect("301 BPM refusé");
        assert!(trop_vite.contains("40 à 300 BPM"), "{trop_vite}");
        assert!(
            prepare(&entry("gmd_pop", 91, 1), false).is_err(),
            "39,6 BPM"
        );
        assert!(
            prepare(&entry("gmd_pop", 3600, 10), false).is_err(),
            "10 BPM"
        );
        // 2/2 (pulsation = blanche, deux noires) : la même période de 20 frames vaut 360 BPM à la noire.
        assert!(prepare(&entry("gmd_pop", 20, 1), false).is_ok());
        assert!(prepare(&entry("reel_quebecois", 20, 1), false).is_err());
        assert!(
            prepare(&entry("reel_quebecois", 30, 1), false).is_ok(),
            "240 BPM"
        );
    }

    #[test]
    fn le_mute_coupe_le_volume_sans_pauser_la_musique() {
        let (control, config, pulse) = prepare(&entry("gmd_pop", 3600, 230), true).unwrap();
        assert_eq!(control.volume, 0.0);
        assert!(
            !control.paused,
            "muette mais en marche : l'horloge doit tourner"
        );
        assert_eq!(
            pulse,
            MusicPulse {
                num: 3600,
                den: 230
            }
        );
        assert_eq!(config.main_bank.as_deref(), Some("harmonium/a.sf3"));
        let (control, ..) = prepare(&entry("gmd_pop", 30, 1), false).unwrap();
        assert!(control.volume > 0.0);
    }

    /// L'ancrage est la frame d'entrée en partie (pas zéro d'office) et `sim_frame` suit `FrameCount` ;
    /// la sortie de partie retire l'horloge.
    #[test]
    fn l_horloge_est_ancree_a_l_entree_puis_suit_la_simulation() {
        let mut world = World::new();
        world.insert_resource(FrameCount { frame: 12 });
        world.insert_resource(MusicPulse { num: 30, den: 1 });
        world.init_resource::<GameClock>();

        world.run_system_once(anchor_music_clock).unwrap();
        let expected = music_clock(12, MusicPulse { num: 30, den: 1 }).unwrap();
        assert_eq!(world.resource::<GameClock>().clock, Some(expected));
        assert_eq!(world.resource::<GameClock>().sim_frame, 12);

        world.resource_mut::<FrameCount>().frame = 400;
        world.run_system_once(feed_music_clock).unwrap();
        assert_eq!(world.resource::<GameClock>().sim_frame, 400);
        assert_eq!(
            world.resource::<GameClock>().clock,
            Some(expected),
            "le suivi ne ré-ancre pas l'horloge"
        );

        world.run_system_once(release_music_clock).unwrap();
        assert_eq!(world.resource::<GameClock>().clock, None);
    }
}
