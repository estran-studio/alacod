//! Fuzz d'inputs (diagnostic, test ignoré) : joue des parties aléatoires d'un joueur (déplacements,
//! tir, dash, sprint, mêlée, rechargement, interaction, visée) sur la carte du jeu et signale toute
//! frame que le synctest rejoue différemment après rollback (bug de déterminisme). Un scénario qui
//! diverge est écrit dans `ALACOD_FUZZ_OUT` (`/tmp/alacod-fuzz` par défaut), rejouable tel quel ;
//! `ALACOD_DIAG=1` nomme le composant qui diverge.
//!
//! - `ALACOD_FUZZ=<de>:<à>` : graines des inputs (`0:8` par défaut) ;
//! - `ALACOD_FUZZ_FRAMES` : frames par partie (3600 par défaut) ;
//! - `ALACOD_FUZZ_MAP_SEED` : graine de la carte (celle du jeu, 123456, par défaut) ;
//! - `ALACOD_FUZZ_SPARSE=1` : peu de zombies à la fois, pour que le joueur survive et que le
//!   fuzz couvre aussi le milieu de partie (sinon il meurt dans les 1 000 premières frames).
//!
//! `cargo test -p scenario --profile headless --test fuzz_inputs -- --ignored --nocapture`

use std::fmt::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

use scenario::{run, Scenario};

/// Générateur xorshift64* : les tests n'ont pas de dépendance à `rand`, et une graine donne
/// toujours la même partie.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

const DIRECTIONS: [&[&str]; 8] = [
    &["Up"],
    &["Down"],
    &["Left"],
    &["Right"],
    &["Up", "Left"],
    &["Up", "Right"],
    &["Down", "Left"],
    &["Down", "Right"],
];

/// Scénario RON d'un joueur : des phases de 6 à 30 frames (déplacement, tir, visée tirés au
/// hasard), avec parfois un dash de 3 frames en début de phase.
fn generate(seed: u64, frames: u32, map_seed: i32, sparse: bool) -> String {
    let mut rng = Rng::new(seed);
    let mut per_frame: Vec<(Vec<&'static str>, (i16, i16))> = Vec::with_capacity(frames as usize);
    while per_frame.len() < frames as usize {
        let length = 6 + rng.below(25) as usize;
        let mut base: Vec<&'static str> = Vec::new();
        if !rng.chance(25) {
            base.extend_from_slice(DIRECTIONS[rng.below(8) as usize]);
        }
        for (button, percent) in [
            ("Fire", 60),
            ("Sprint", 10),
            ("Modifier", 5),
            ("Melee", 5),
            ("Interaction", 10),
            ("Reload", 3),
            ("SwitchWeaponMode", 2),
        ] {
            if rng.chance(percent) {
                base.push(button);
            }
        }
        let pan = (rng.below(241) as i16 - 120, rng.below(241) as i16 - 120);
        let dash = rng.chance(12);
        let switch_weapon = rng.chance(4);
        for i in 0..length {
            let mut buttons = base.clone();
            if dash && i < 3 {
                buttons.push("Dash");
            }
            if switch_weapon && i == 0 {
                buttons.push("SwitchWeapon");
            }
            per_frame.push((buttons, pan));
        }
    }
    per_frame.truncate(frames as usize);

    let mut segments = String::new();
    let mut start = 0;
    for i in 1..=per_frame.len() {
        if i == per_frame.len() || per_frame[i] != per_frame[start] {
            let (buttons, pan) = &per_frame[start];
            let _ = writeln!(
                segments,
                "            (from: {start}, to: {i}, buttons: [{}], pan: ({}, {})),",
                buttons.join(", "),
                pan.0,
                pan.1
            );
            start = i;
        }
    }
    let waves = if sparse {
        "    wave_overrides: (base_enemies: 3, enemies_per_wave: 1, max_concurrent_enemies: 2, \
         spawn_batch_size: 1, spawn_interval_frames: 240),\n"
    } else {
        ""
    };
    format!(
        "Scenario(\n    game: \"zombies\",\n    map: \"maps/avant_poste.ldtk\",\n    map_seed: {map_seed},\n    \
         frames: {frames},\n{waves}    players: [\n        (inputs: [\n{segments}        ]),\n    ],\n)\n"
    )
}

fn env_or<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore]
fn fuzz_inputs() {
    let spec = std::env::var("ALACOD_FUZZ").unwrap_or_else(|_| "0:8".into());
    let (from, to) = spec.split_once(':').expect("ALACOD_FUZZ=<de>:<à>");
    let (from, to): (u64, u64) = (from.parse().unwrap(), to.parse().unwrap());
    let frames: u32 = env_or("ALACOD_FUZZ_FRAMES", 3600);
    let map_seed: i32 = env_or("ALACOD_FUZZ_MAP_SEED", 123456);
    let sparse = std::env::var("ALACOD_FUZZ_SPARSE").is_ok_and(|v| v == "1");
    let out_dir = PathBuf::from(
        std::env::var("ALACOD_FUZZ_OUT").unwrap_or_else(|_| "/tmp/alacod-fuzz".into()),
    );
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut diverging = Vec::new();
    for seed in from..to {
        let source = generate(seed, frames, map_seed, sparse);
        let scenario = Scenario::from_ron(&source).expect("scénario généré invalide");
        let outcome = catch_unwind(AssertUnwindSafe(|| run(&scenario)));
        let failures: Vec<String> = match &outcome {
            Ok(outcome) => outcome
                .failures
                .iter()
                .filter(|f| f.contains("synctest") || f.contains("divergent"))
                .cloned()
                .collect(),
            Err(_) => vec!["panique pendant la partie".into()],
        };
        match &outcome {
            Ok(outcome) => println!(
                "graine {seed} : {} frames, vague {}, {} kills, {} vivant(s), {:.0} fps{}",
                outcome.metrics.frames,
                outcome.metrics.final_wave,
                outcome.metrics.kills,
                outcome.metrics.players_alive,
                outcome.metrics.sim_fps,
                if failures.is_empty() {
                    ""
                } else {
                    " : DIVERGENCE"
                }
            ),
            Err(_) => println!("graine {seed} : PANIQUE"),
        }
        if !failures.is_empty() {
            let path = out_dir.join(format!("seed-{seed}.ron"));
            std::fs::write(&path, &source).unwrap();
            println!("{}\n  scénario : {}", failures.join("\n"), path.display());
            diverging.push(seed);
        }
    }
    assert!(
        diverging.is_empty(),
        "graines divergentes : {diverging:?} (scénarios dans {})",
        out_dir.display()
    );
}
