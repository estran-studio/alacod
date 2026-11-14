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
    MusicClock,
};
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

/// Installe la musique si le manifeste du jeu la demande. Refus (profil inconnu, période nulle) : journal, pas
/// de musique, le jeu continue.
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
    ExternalClock::new(sim_hz(), 0, pulse.num, pulse.den).map_err(|error| error.to_string())?;
    let mut control = HarmoniumControl::default();
    control
        .set_profile_id(&entry.profile)
        .map_err(|error| format!("profil « {} » : {error}", entry.profile))?;
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

/// Journal de l'état de la musique (présentation, jamais lu par la simulation) : à chaque changement d'état
/// (chargement, hôte prêt, lecture, phase du suivi) puis toutes les 5 s de temps réel.
fn log_music_status(
    status: Res<HarmoniumStatus>,
    clock: Res<MusicClock>,
    game_clock: Res<GameClock>,
    frame: Res<FrameCount>,
    time: Res<Time<Real>>,
    mut last: Local<Option<(String, f32)>>,
) {
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
    let state = format!(
        "{status:?} valid={} playing={} suivi={}",
        clock.valid,
        clock.playing,
        follow.as_deref().map_or("aucun", |_| "oui"),
    );
    let now = time.elapsed_secs();
    let phase = clock.follow.map(|follow| format!("{:?}", follow.phase));
    let key = format!("{state} {phase:?}");
    let due = match last.as_ref() {
        Some((previous, at)) => *previous != key || now - at >= 5.0,
        None => true,
    };
    if !due {
        return;
    }
    *last = Some((key, now));
    info!(
        "musique f{} {state} mesure={} pulsation={}/{} bpm={:.1} horloge_jeu={} statut_externe={}",
        frame.frame,
        clock.bar,
        clock.pulse,
        clock.pulses_per_bar,
        clock.bpm,
        game_clock.clock.is_some(),
        follow.as_deref().unwrap_or("aucun"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use content::manifest::PulseFrames;

    fn entry(profile: &str, num: u32, den: u32) -> MusicEntry {
        MusicEntry {
            profile: profile.to_string(),
            pulse_frames: PulseFrames { num, den },
            main_bank: "harmonium/a.sf3".to_string(),
            percussion_bank: "harmonium/b.sf2".to_string(),
        }
    }

    #[test]
    fn un_profil_inconnu_ou_une_periode_nulle_est_refuse() {
        assert!(prepare(&entry("pas_un_profil", 30, 1), false).is_err());
        assert!(prepare(&entry("gmd_pop", 0, 1), false).is_err());
        assert!(prepare(&entry("gmd_pop", 30, 0), false).is_err());
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
