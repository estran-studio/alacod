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
//! - à la sortie de partie, [`release_music_clock`] retire l'horloge (la musique se met en pause, voir plus bas).
//!
//! **Continuité** (M-7b, `docs/conventions.md` §124) : la musique ne joue que pendant qu'une horloge est posée
//! ([`gate_music`] : pause et volume 0 sinon) : sans elle, l'hôte jouait au tempo des émotions dès ses banques chargées, puis se taisait à
//! l'ancrage, le temps de se caler (« ça stoppe un peu au début »). Le suivi rattrape au tempo, jusqu'à 5 %
//! ([`follow_config`]), les écarts que crée une frame longue ou le ralenti `run_slow` de `bevy_ggrs`, au lieu d'un saut
//! de mesure qui laissait jusqu'à une mesure de silence.
//!
//! Sans `music` dans le manifeste du jeu, le plugin n'est pas installé : pas de musique. `ALACOD_MUTE=1` pose le
//! volume à 0 (la musique tourne, muette : l'horloge prouve la sortie sans écoute).

use bevy::prelude::*;
use content::manifest::{GameManifest, MusicEntry};
use harmonium_bevy::{
    ExternalClock, FollowConfig, GameClock, HarmoniumConfig, HarmoniumControl, HarmoniumPlugin,
    HarmoniumStatus, MusicClock, Phase, StyleProfile,
};
use harmonium_core::time_plan::{BarTiming, Swing, Tempo, correction::CorrectionPolicy};
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
    install(app, control, config, pulse);
}

/// Plugin, cibles et systèmes de la musique ; partagé par [`add_music`] et le contrôle de continuité (tests).
/// [`gate_music`] (`Update`, avant `HarmoniumSet` en `PostUpdate`) tient la musique en pause et muette jusqu'à l'ancrage.
fn install(app: &mut App, control: HarmoniumControl, config: HarmoniumConfig, pulse: MusicPulse) {
    app.insert_resource(MusicVolume(control.volume))
        .insert_resource(control)
        .insert_resource(pulse)
        .add_plugins(HarmoniumPlugin::new(config))
        .add_systems(OnEnter(AppState::InGame), anchor_music_clock)
        .add_systems(OnExit(AppState::InGame), release_music_clock)
        .add_systems(
            Update,
            (feed_music_clock, gate_music, log_music_status).chain(),
        );
}

/// Réglages du suivi : ceux d'harmonium, sauf le micro-ajustement, porté de 5 ‰ sur 8 pulsations à 5 % (le plafond du
/// playback, `MAX_NUDGE_PPM`) sur 16. À 5 ‰, le suivi n'absorbe qu'environ 20 ms d'écart (pulsation de 0,5 s) : une frame
/// de 300 ms (50 ms de simulation perdus, `Time<Virtual>` borné à 250 ms) déclenchait un saut à la mesure suivante, dont
/// l'attente est silencieuse (jusqu'à une mesure). À 5 % sur 16 pulsations, il absorbe environ 400 ms au tempo (8 s de
/// rattrapage à cette pulsation), sans silence ; au-delà, le saut d'harmonium reste.
fn follow_config() -> FollowConfig {
    FollowConfig {
        policy: CorrectionPolicy {
            max_nudge_per_mille: 50,
            window_pulses: 16,
            ..default()
        },
        ..default()
    }
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
        follow: follow_config(),
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

/// Volume de la musique pendant qu'une horloge est posée (`HarmoniumControl::volume` d'origine, 0 si muette).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct MusicVolume(pub f32);

/// `Update`, après [`feed_music_clock`] : tant qu'aucune horloge n'est posée, la musique est en pause **et** à volume 0.
/// L'hôte d'harmonium démarre dès ses banques chargées, avant la partie ; sans cette garde il jouait au tempo des émotions,
/// puis se taisait à l'ancrage (attente silencieuse du suivi) avant de repartir sur l'horloge (« ça stoppe un peu au
/// début »). Les deux, parce qu'aucun ne suffit seul : la pause arrête le flux sur CoreAudio et le web, mais sur ALSA
/// la pause demandée par l'hôte juste après l'ouverture du flux échoue en silence (`snd_pcm_pause` avant `RUNNING`,
/// erreur ignorée par cpal, mesuré : la musique jouait 1,5 s avant l'ancrage) ; le volume 0 vaut partout, sauf les
/// 10 ms de rampe depuis le gain initial (1) du playback. Web : l'`AudioContext` reprend au premier geste après
/// l'ancrage (les touches de la partie), pas à un geste du lobby.
pub fn gate_music(
    game_clock: Res<GameClock>,
    volume: Res<MusicVolume>,
    mut control: ResMut<HarmoniumControl>,
) {
    let anchored = game_clock.clock.is_some();
    let target = if anchored { volume.0 } else { 0.0 };
    if control.paused == anchored {
        control.paused = !anchored;
    }
    if control.volume != target {
        control.volume = target;
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

/// Contrôle de continuité de la musique (M-7b, `docs/conventions.md` § 124) : la **vraie sortie** d'harmonium, sans carte son.
///
/// L'hôte est détaché (`LiveHost::detached`) ; le banc joue le rôle de la carte son et du jeu sur une horloge murale
/// simulée (déterministe, plus rapide que le temps réel) : il rend chaque tampon audio (`PlaybackEngine::process_buffer`, le
/// callback cpal) à son instant, fait avancer `FrameCount` comme `bevy_ggrs` (accumulateur, `Time<Virtual>` borné à 250 ms
/// par frame) et lance une frame Bevy (`HarmoniumPlugin` et les systèmes de la musique, installés par [`install`]).
///
/// Les à-coups injectés sont ceux mesurés dans la salle de 1837 (rapport `m-7b-musique-continue.md`) : une frame de rendu
/// longue (premier chargement d'un son, d'un visuel d'impact) fait perdre du temps à la simulation, et un pair p2p en avance
/// court au ralenti de 10 % (`run_slow` de `bevy_ggrs`). Le contrôle échoue si, après la première note entendue, la musique
/// se tait (tampons sous -60 dBFS ou playback « en attente » pendant au moins 150 ms) ou saute (mesure qui recule ou en
/// saute une), et si elle ne part pas dans les 3 s qui suivent l'entrée en partie.
///
/// Banques : `GeneralUser-GS.sf3` et `harmonium_trad.sf2` dans `$ALACOD_HARMONIUM_BANKS_DIR` (défaut : le cache de
/// `scripts/harmonium-banks.sh`) ou `games/testbed/assets/harmonium/`. Sans elles (CI), l'énergie n'est pas jugée (le synthé
/// rend du silence) : seuls le playback « en attente » et la suite des mesures, publiés par le callback, le sont.
#[cfg(test)]
mod continuite {
    use super::*;
    use bevy::{app::TaskPoolPlugin, asset::AssetPlugin, state::app::StatesPlugin};
    use content::manifest::PulseFrames;
    use harmonium::{LiveHost, playback::PlaybackEngine};
    use std::time::{Duration, Instant};

    const RATE: u32 = 48_000;
    const BUFFER: usize = 512;
    const SILENCE_DB: f32 = -60.0;
    const MAX_HOLE: Duration = Duration::from_millis(150);
    const SIM_FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);
    /// `Time<Virtual>::max_delta` de bevy : au-delà, une frame longue perd du temps de simulation.
    const MAX_SIM_DELTA: Duration = Duration::from_millis(250);

    fn banks() -> Option<(Vec<u8>, Vec<u8>)> {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut dirs = vec![manifest.join("../../games/testbed/assets/harmonium")];
        if let Ok(dir) = std::env::var("ALACOD_HARMONIUM_BANKS_DIR") {
            dirs.insert(0, dir.into());
        } else if let Ok(home) = std::env::var("HOME") {
            dirs.insert(
                0,
                std::path::Path::new(&home).join("orca/workspaces/alacod/packs/harmonium-banks"),
            );
        }
        dirs.iter().find_map(|dir| {
            Some((
                std::fs::read(dir.join("GeneralUser-GS.sf3")).ok()?,
                std::fs::read(dir.join("harmonium_trad.sf2")).ok()?,
            ))
        })
    }

    /// Ce que le banc a entendu : un tampon audio (instant de début, dBFS) ou une frame (instant, horloge publiée).
    struct Banc {
        app: App,
        playback: PlaybackEngine,
        t0: Instant,
        wall: Duration,
        audio_until: Duration,
        accumulator: Duration,
        frame: u32,
        buffer: Vec<f32>,
        levels: Vec<(Duration, f32)>,
        clocks: Vec<(Duration, MusicClock)>,
        anchor: Option<Duration>,
        /// La pause arrête-t-elle le flux ? CoreAudio et le web : oui ; ALSA : la pause demandée par l'hôte juste après
        /// l'ouverture échoue (`snd_pcm_pause` avant `RUNNING`, erreur ignorée par cpal) et le callback continue.
        pause_stops_output: bool,
    }

    impl Banc {
        fn new(main: Option<&[u8]>, percussion: Option<&[u8]>, pause_stops_output: bool) -> Self {
            // Le réglage `music` neutre du testbed (`games/testbed/assets/game.ron`).
            let entry = MusicEntry {
                profile: "gmd_pop".to_string(),
                pulse_frames: PulseFrames { num: 30, den: 1 },
                main_bank: "harmonium/GeneralUser-GS.sf3".to_string(),
                percussion_bank: "harmonium/harmonium_trad.sf2".to_string(),
            };
            let (control, mut config, pulse) = prepare(&entry, false).unwrap();
            // L'hôte est fourni par le banc : rien à charger par l'`AssetServer`.
            config.main_bank = None;
            config.percussion_bank = None;
            let mut app = App::new();
            app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default(), StatesPlugin))
                .init_state::<AppState>()
                .insert_resource(FrameCount { frame: 0 });
            let t0 = Instant::now();
            app.insert_resource(Time::<Real>::new(t0));
            install(&mut app, control, config, pulse);
            let (host, playback) =
                LiveHost::detached(main, percussion, &[], f64::from(RATE)).unwrap();
            app.insert_non_send(harmonium_bevy::HarmoniumRuntime::new(host).unwrap());
            Self {
                app,
                playback,
                t0,
                wall: Duration::ZERO,
                audio_until: Duration::ZERO,
                accumulator: Duration::ZERO,
                frame: 0,
                buffer: vec![0.0; BUFFER * 2],
                levels: Vec::new(),
                clocks: Vec::new(),
                anchor: None,
                pause_stops_output,
            }
        }

        /// Une frame de présentation de `dt` (temps mur) ; `sim_slow` : frames de simulation 10 % plus longues (`run_slow`).
        fn frame(&mut self, dt: Duration, in_game: bool, sim_slow: bool) {
            self.wall += dt;
            // La carte son : chaque tampon est rendu avant d'être entendu (un tampon d'avance). Flux en pause (cpal
            // `pause`, que l'hôte détaché ne fait pas) : pas de callback, la carte rend du silence.
            let buffer_time = Duration::from_nanos(BUFFER as u64 * 1_000_000_000 / u64::from(RATE));
            let paused = self.pause_stops_output
                && self
                    .app
                    .world()
                    .non_send_resource::<harmonium_bevy::HarmoniumRuntime>()
                    .paused();
            while self.audio_until < self.wall + buffer_time {
                let audible = self.t0 + self.audio_until;
                if paused {
                    self.buffer.fill(0.0);
                } else {
                    self.playback.process_buffer(&mut self.buffer, 2, audible);
                }
                let energy =
                    self.buffer.iter().map(|v| v * v).sum::<f32>() / self.buffer.len() as f32;
                let db = if energy > 1e-20 {
                    10.0 * energy.log10()
                } else {
                    -200.0
                };
                self.levels.push((self.audio_until, db));
                self.audio_until += buffer_time;
            }
            // La simulation : accumulateur de `bevy_ggrs`, delta borné comme `Time<Virtual>`.
            if in_game {
                let period = if sim_slow {
                    SIM_FRAME * 11 / 10
                } else {
                    SIM_FRAME
                };
                self.accumulator += dt.min(MAX_SIM_DELTA);
                while self.accumulator >= period {
                    self.accumulator -= period;
                    self.frame += 1;
                }
                self.app.world_mut().resource_mut::<FrameCount>().frame = self.frame;
                if self.anchor.is_none() {
                    self.anchor = Some(self.wall);
                    self.app
                        .world_mut()
                        .resource_mut::<NextState<AppState>>()
                        .set(AppState::InGame);
                }
            }
            self.app
                .world_mut()
                .resource_mut::<Time<Real>>()
                .update_with_instant(self.t0 + self.wall);
            self.app.update();
            let mut clock = *self.app.world().resource::<MusicClock>();
            // Sans banques, l'horloge publiée tient lieu d'oreille : un playback qui tourne à volume 0 ne s'entend pas.
            if self.app.world().resource::<HarmoniumControl>().volume == 0.0 {
                clock.valid = false;
            }
            self.clocks.push((self.wall, clock));
        }

        /// Trous et sauts après la première note ; `judge_energy` : seulement avec les banques.
        fn incidents(&self, judge_energy: bool) -> (Option<Duration>, Vec<String>) {
            let heard = |db: f32| db >= SILENCE_DB;
            let first_valid = self
                .clocks
                .iter()
                .find(|(_, c)| c.valid && c.playing)
                .map(|(t, _)| *t);
            // Avant l'ancrage, un tampon audible isolé est la rampe de 10 ms du gain initial du playback d'harmonium (1,
            // jamais 0 : D220 (2)) quand la pause n'arrête pas le flux (ALSA) : compté et affiché, pas pris pour le début
            // de la musique. Deux tampons de suite (21 ms) ou plus avant l'ancrage sont de la musique. Quand la pause
            // arrête le flux, rien ne doit sonner avant l'ancrage, pas même ce tampon.
            let anchor = self.anchor.unwrap_or(Duration::MAX);
            let audible = |i: usize| self.levels.get(i).is_some_and(|(_, db)| heard(*db));
            let ramps = (0..self.levels.len())
                .filter(|&i| audible(i) && self.levels[i].0 < anchor)
                .filter(|&i| !audible(i + 1) && (i == 0 || !audible(i - 1)))
                .count();
            let mut out = Vec::new();
            if ramps > 0 {
                println!("continuité : {ramps} tampon(s) audible(s) isolé(s) avant l'ancrage (rampe du gain initial)");
                if self.pause_stops_output {
                    out.push(format!("{ramps} tampon(s) audible(s) avant l'ancrage malgré la pause"));
                }
            }
            let first_sound = (0..self.levels.len())
                .find(|&i| audible(i) && (self.levels[i].0 >= anchor || audible(i + 1)))
                .map(|i| self.levels[i].0);
            let start = if judge_energy {
                first_sound
            } else {
                first_valid
            };
            let Some(start) = start else {
                return (None, out);
            };
            let end = self.wall.saturating_sub(Duration::from_millis(500));
            let mut hole = |label: &str, runs: &mut Option<Duration>, t: Duration, quiet: bool| {
                match (quiet, *runs) {
                    (true, None) => *runs = Some(t),
                    (false, Some(from)) => {
                        if t - from >= MAX_HOLE {
                            out.push(format!(
                                "{label} de {:.2} s à {:.2} s ({} ms)",
                                from.as_secs_f64(),
                                t.as_secs_f64(),
                                (t - from).as_millis()
                            ));
                        }
                        *runs = None;
                    }
                    _ => {}
                }
            };
            let mut silent = None;
            if judge_energy {
                for (t, db) in self.levels.iter().filter(|(t, _)| *t >= start && *t < end) {
                    hole("silence", &mut silent, *t, !heard(*db));
                }
                hole("silence", &mut silent, end, false);
            }
            let mut waiting = None;
            let mut previous: Option<usize> = None;
            let mut jumps = Vec::new();
            for (t, clock) in self.clocks.iter().filter(|(t, _)| *t >= start && *t < end) {
                hole("playback en attente", &mut waiting, *t, !clock.valid);
                if clock.valid {
                    if let Some(bar) = previous {
                        if clock.bar < bar || clock.bar > bar + 1 {
                            jumps.push(format!(
                                "saut de la mesure {bar} à la mesure {} à {:.2} s",
                                clock.bar,
                                t.as_secs_f64()
                            ));
                        }
                    }
                    previous = Some(clock.bar);
                }
            }
            hole("playback en attente", &mut waiting, end, false);
            out.extend(jumps);
            (Some(start), out)
        }
    }

    /// 16 s de jeu dans le banc ; la première note et les incidents (texte), préfixés par le mode de pause.
    fn jouer(banks: Option<&(Vec<u8>, Vec<u8>)>, pause_stops_output: bool) -> Vec<String> {
        let mode = if pause_stops_output {
            "pause qui arrête le flux (CoreAudio, web)"
        } else {
            "pause sans effet (ALSA)"
        };
        let (main, percussion) =
            banks.map_or((None, None), |(m, p)| (Some(m.as_slice()), Some(p.as_slice())));
        let mut banc = Banc::new(main, percussion, pause_stops_output);
        // 0,5 s de chargement (hôte prêt, pas encore de partie), puis la partie : 16 s en tout.
        // À-coups : frame de 300 ms à 4 s et de 450 ms à 8 s (premier son, premier impact), ralenti `run_slow` de 11 à 12 s.
        let mut hitches = [
            (Duration::from_secs(4), Duration::from_millis(300)),
            (Duration::from_secs(8), Duration::from_millis(450)),
        ]
        .into_iter()
        .peekable();
        while banc.wall < Duration::from_secs(16) {
            let in_game = banc.wall >= Duration::from_millis(500);
            let dt = match hitches.peek() {
                Some((at, long)) if banc.wall >= *at => {
                    let long = *long;
                    hitches.next();
                    long
                }
                _ => SIM_FRAME,
            };
            let slow = (11.0..12.0).contains(&banc.wall.as_secs_f64());
            banc.frame(dt, in_game, slow);
        }
        let follow = banc.app.world().resource::<MusicClock>().follow;
        println!("continuité [{mode}] : suivi en fin de partie : {follow:?}");
        let (start, mut incidents) = banc.incidents(banks.is_some());
        let anchor = banc.anchor.unwrap();
        match start {
            None => incidents.push("la musique n'a jamais joué".to_string()),
            Some(start) => {
                println!(
                    "continuité [{mode}] : première note à {:.2} s (entrée en partie à {:.2} s)",
                    start.as_secs_f64(),
                    anchor.as_secs_f64()
                );
                if start > anchor + Duration::from_secs(3) {
                    incidents.push("la musique part plus de 3 s après l'entrée en partie".to_string());
                }
            }
        }
        incidents
            .into_iter()
            .map(|incident| format!("[{mode}] {incident}"))
            .collect()
    }

    #[test]
    fn la_musique_ne_se_tait_pas_au_demarrage_ni_sous_les_a_coups() {
        let banks = banks();
        println!(
            "continuité : banques {}",
            if banks.is_some() {
                "trouvées : énergie jugée"
            } else {
                "absentes : énergie non jugée (attente et mesures seulement)"
            }
        );
        let mut incidents = jouer(banks.as_ref(), true);
        incidents.extend(jouer(banks.as_ref(), false));
        println!("continuité : incidents : {}", incidents.len());
        for incident in &incidents {
            println!("continuité : {incident}");
        }
        assert!(
            incidents.is_empty(),
            "la musique se tait ou saute : {incidents:#?}"
        );
    }
}
