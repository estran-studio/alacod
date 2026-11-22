//! C-1 : le journal au niveau par défaut du jeu (INFO) ne répète pas une ligne INFO/WARN à chaque
//! frame (même cible, même message aux frames près). Le diagnostic par frame est en `trace!`.
use game::replay::{BotProfile, PlayerScript, Scenario};
use std::{
    collections::BTreeMap,
    io::Write,
    sync::{Mutex, OnceLock},
};

static JOURNAL: Mutex<Vec<u8>> = Mutex::new(Vec::new());

struct Capture;
impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        JOURNAL.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
    type Writer = Capture;
    fn make_writer(&'a self) -> Capture {
        Capture
    }
}

fn install() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
            .with_writer(Capture)
            .init();
    });
}

/// Retire `f=<n>` : deux lignes de frames différentes ont la même clé.
fn cle(ligne: &str) -> String {
    let mut out = String::new();
    let mut rest = ligne;
    while let Some(i) = rest.find("f=") {
        out.push_str(&rest[..i + 2]);
        rest = rest[i + 2..].trim_start_matches(|c: char| c.is_ascii_digit());
    }
    out.push_str(rest);
    out
}

fn scenario(game: &str, map: &str, frames: u32) -> Scenario {
    Scenario {
        game: game.into(),
        map: map.into(),
        map_seed: 123456,
        frames,
        players: vec![PlayerScript {
            bot: Some(BotProfile::Immobile),
            ..Default::default()
        }],
        expect: vec![],
        weapon_overrides: vec![],
        wave_overrides: None,
        invariants: Default::default(),
        powerups: vec![],
        items: vec![],
        chests: vec![],
        room_reward_chance_override: None,
        shop: Default::default(),
        profile: false,
        profile_setup: None,
        hub: None,
        shopping: false,
        explorer: None,
        powerup_drop_chance_override: None,
        floors: None,
        progression: None,
        clocks: None,
        difficulty: None,
        characters: vec![],
        mode: None,
        ui: Vec::new(),
        tags: Vec::new(),
        input_delay: None,
    }
}

fn lignes_repetees(game: &str, map: &str, frames: u32) -> Vec<(usize, String)> {
    JOURNAL.lock().unwrap().clear();
    let outcome = scenario::run_with(&scenario(game, map, frames), |app| {
        // Comme la partie réelle d'un jeu sans vagues : le runner force `true`.
        app.insert_resource(game::waves::WaveModeEnabled(false));
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let journal = String::from_utf8(JOURNAL.lock().unwrap().clone()).unwrap();
    let info = journal
        .lines()
        .filter(|l| l.trim_start().starts_with("INFO"))
        .count();
    eprintln!("{game} {map} : {frames} frames, {info} lignes INFO");
    let mut compte: BTreeMap<String, usize> = BTreeMap::new();
    for ligne in journal.lines() {
        if ligne.trim_start().starts_with("INFO") || ligne.trim_start().starts_with("WARN") {
            *compte.entry(cle(ligne)).or_default() += 1;
        }
    }
    compte
        .into_iter()
        .filter(|(_, n)| *n * 2 >= frames as usize)
        .map(|(l, n)| (n, l))
        .collect()
}

/// 600 frames ; `ALACOD_C1_FRAMES=3600` (60 s) pour le chiffre du rapport.
fn frames_jouees() -> u32 {
    std::env::var("ALACOD_C1_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600)
}

#[test]
fn aucune_ligne_info_ou_warn_par_frame() {
    install();
    // Testbed (vagues désactivées : le spawner d'ennemis tourne à chaque frame) ; zombies (vagues : il est coupé).
    for (game, map) in [
        ("testbed", "testbed/arena.ldtk"),
        ("testbed", "testbed/corridor.ldtk"),
        ("testbed", "testbed/salles.ldtk"),
        ("zombies", "exemples/test_map.ldtk"),
    ] {
        let repetees = lignes_repetees(game, map, frames_jouees());
        assert!(
            repetees.is_empty(),
            "{game} {map} : lignes INFO/WARN répétées à chaque frame : {repetees:#?}"
        );
    }
}
