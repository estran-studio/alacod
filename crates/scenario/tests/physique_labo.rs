//! P-1e (`docs/conventions.md` § 125) : la physique et les gestes du joueur au chiffre du banc du labo. Verdict de William
//! (20 nov, 1837 `f2ac3bf`) : « notre physique est juste plus lente et lourde ». Mesuré au vrai jeu (rapport
//! `docs/taches/rapports/p-1e-physique-labo.md`) : vitesses, accélération, balles et gestes de la simulation au chiffre
//! du banc, mais la **partie locale** appliquait chaque input 5 frames plus tard (délai d'entrée du jeu en ligne) : départ,
//! arrêt, demi-tour, tir et pas partaient 83 ms après l'appui, et le joueur glissait après chaque relâché. Au banc, l'appui
//! agit à l'image suivante.
//!
//! Testbed (engine) : la session du client hors ligne ([`game::args::input_delay_from_env`] sans `ALACOD_INPUT_DELAY`)
//! réagit à la frame qui suit l'input ; les gestes d'une arme à gestes s'étirent sur la durée de l'arme ; la première
//! esquive joue le premier clip de `dash`. Salle réelle d'un jeu (test ignoré) : les nombres du joueur d'un jeu externe
//! contre ceux de son banc, `ALACOD_P1E_GAME_DIR=/chemin/du/jeu cargo test -p scenario --profile headless
//! --no-default-features --test physique_labo -- --ignored` (valeurs attendues lues dans l'environnement : voir
//! [`real_room_matches_the_bench`]).

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use game::character::player::Player;
use game::model3d::playback::ModelPlayback;
use scenario::Scenario;

/// Le joueur 0 à chaque frame : (frame, x, y).
fn player_track(
    scenario: &Scenario,
    mut also: impl FnMut(&mut World, u32),
) -> Vec<(u32, f32, f32)> {
    let mut track = Vec::new();
    scenario::runner::run_visiting(scenario, scenario.frames, |world, frame| {
        also(world, frame);
        let mut query = world.query::<(&Player, &FixedTransform3D)>();
        if let Some((_, t)) = query.iter(world).find(|(p, _)| p.handle == 0) {
            track.push((
                frame,
                t.translation.x.to_num::<f32>(),
                t.translation.y.to_num::<f32>(),
            ));
        }
    });
    track
}

/// Déplacement en x de la frame `f - 1` à `f`.
fn step_at(track: &[(u32, f32, f32)], f: u32) -> f32 {
    let at = |f: u32| track.iter().find(|(g, ..)| *g == f).map(|(_, x, _)| *x);
    at(f).zip(at(f - 1)).map_or(f32::NAN, |(b, a)| b - a)
}

/// Première frame après `from` dont le pas en x quitte celui de la frame `from` (réaction à un input posé à `from`).
fn reaction(track: &[(u32, f32, f32)], from: u32) -> u32 {
    let before = step_at(track, from);
    (from + 1..from + 30)
        .find(|f| (step_at(track, *f) - before).abs() > 1e-3)
        .expect("aucune réaction dans les 30 frames")
}

fn pantin(inputs: &str, frames: u32, weapon: &str) -> Scenario {
    let mut scenario = Scenario::from_ron(&format!(
        r#"Scenario(
            powerup_drop_chance_override: "0.0",
            game: "testbed",
            map: "testbed/arena.ldtk",
            frames: {frames},
            players: [(character: "pantin", weapon: "{weapon}", inputs: [{inputs}])],
        )"#
    ))
    .expect("scénario");
    // La session du client hors ligne, comme la partie locale la construit (`args::get_args`).
    scenario.input_delay = Some(game::args::input_delay_from_env(false));
    scenario
}

#[test]
fn la_partie_locale_reagit_a_la_frame_qui_suit_l_input() {
    let scenario = pantin(
        "(from: 30, to: 60, buttons: [Right], pan: (100, 0)),
         (from: 80, to: 100, buttons: [Left], pan: (100, 0)),
         (from: 100, to: 120, buttons: [Right], pan: (100, 0))",
        140,
        "pistol",
    );
    let track = player_track(&scenario, |_, _| {});
    // Banc : l'appui agit à l'image qui suit l'événement (`armes.js` `pas()` lit `touches`). Ici : l'input de la
    // frame N change le pas de N + 1 au plus tard. Avec le délai de 5 frames d'avant P-1e : N + 6.
    for (input, what) in [
        (30, "départ"),
        (60, "arrêt"),
        (80, "départ à gauche"),
        (100, "demi-tour"),
    ] {
        let at = reaction(&track, input);
        assert!(
            at <= input + 1,
            "{what} : input à la frame {input}, première réaction à la frame {at} (banc : {} au plus)",
            input + 1
        );
    }
}

/// Clip joué par le joueur 0 à chaque frame : (frame, clip, position dans le clip en s).
fn clips(scenario: &Scenario, mut setup: impl FnMut(&mut World)) -> Vec<(u32, String, f32)> {
    let mut out = Vec::new();
    let mut ready = false;
    scenario::runner::run_visiting(scenario, scenario.frames, |world, frame| {
        if !ready {
            setup(world);
            ready = true;
        }
        let mut query = world.query::<(&Player, &ModelPlayback)>();
        if let Some((_, playback)) = query.iter(world).find(|(p, _)| p.handle == 0) {
            if let (Some(clip), Some(time)) = (playback.clip(), playback.time()) {
                out.push((frame, clip.to_string(), time));
            }
        }
    });
    out
}

fn span(clips: &[(u32, String, f32)], clip: &str) -> Option<(u32, u32)> {
    let frames: Vec<u32> = clips
        .iter()
        .filter(|(_, c, _)| c == clip)
        .map(|(f, ..)| *f)
        .collect();
    Some((*frames.first()?, *frames.last()?))
}

#[test]
fn les_gestes_d_une_arme_a_gestes_durent_ceux_de_l_arme() {
    // `arme_a_gestes` : épaulé 9 frames, tir 18, rechargement 72. Les clips du pantin sont allongés au double (épaulé
    // 0,3 s, tir 0,6 s) : le banc les étire sur la durée du geste (`armes.js` `lancer`), quelle que soit leur longueur.
    let scenario = pantin(
        "(from: 30, to: 31, buttons: [Fire], pan: (100, 0)),
         (from: 31, to: 200, buttons: [], pan: (100, 0))",
        200,
        "arme_a_gestes",
    );
    let played = clips(&scenario, |world| {
        let mut registry = world.resource_mut::<content::registry::Registry>();
        let table = registry.models.as_mut().expect("models.ron du testbed");
        let manifest = table.manifests.get_mut("pantin").unwrap().as_mut().unwrap();
        manifest.clips.insert("shoulder".into(), (0.0, 7.2));
        manifest.clips.insert("shoot".into(), (0.0, 14.4));
    });
    let (s0, s1) = span(&played, "shoulder").expect("épaulé joué");
    let shoulder_end = played
        .iter()
        .filter(|(_, c, _)| c == "shoulder")
        .map(|(_, _, t)| *t)
        .fold(0.0, f32::max);
    assert!(
        s1 - s0 + 1 >= 8,
        "épaulé joué {} frames (geste de 9)",
        s1 - s0 + 1
    );
    assert!(
        shoulder_end >= 0.8 * 0.3,
        "épaulé : le clip (0,3 s) n'est montré que jusqu'à {shoulder_end:.3} s en 9 frames (banc : clip entier)"
    );
    let (t0, t1) = span(&played, "shoot").expect("tir joué");
    assert!(
        (17..=19).contains(&(t1 - t0 + 1)),
        "tir joué {} frames (geste de tir de l'arme : 18)",
        t1 - t0 + 1
    );
    let reload = played
        .iter()
        .find(|(_, c, _)| c == "reload")
        .expect("rechargement joué");
    let around: Vec<_> = played
        .iter()
        .filter(|(f, ..)| *f + 3 >= t1 && *f <= t1 + 4)
        .collect();
    assert!(
        reload.0 <= t1 + 1 && reload.2 <= 0.05 * 1.2,
        "rechargement entré à la frame {} à {:.3} s du clip (fin du tir : {t1} ; banc : dès la fin du tir, à 0) ; {around:?}",
        reload.0,
        reload.2
    );
}

#[test]
fn la_premiere_esquive_joue_le_premier_clip_de_dash() {
    let scenario = pantin(
        "(from: 30, to: 31, buttons: [Right, Dash], pan: (100, 0)),
         (from: 31, to: 90, buttons: [], pan: (100, 0)),
         (from: 90, to: 91, buttons: [Right, Dash], pan: (100, 0))",
        130,
        "pistol",
    );
    // Deux clips d'esquive : le banc joue le premier à la première esquive (`armes.js` : `J.pas` part de −1).
    let played = clips(&scenario, |world| {
        let mut registry = world.resource_mut::<content::registry::Registry>();
        let table = registry.models.as_mut().expect("models.ron du testbed");
        table.def.models.get_mut("pantin").unwrap().dash = vec!["roll".into(), "strike".into()];
    });
    let order: Vec<&str> = played
        .iter()
        .map(|(_, c, _)| c.as_str())
        .filter(|c| *c == "roll" || *c == "strike")
        .fold(Vec::new(), |mut v, c| {
            if v.last() != Some(&c) {
                v.push(c);
            }
            v
        });
    assert_eq!(
        order,
        ["roll", "strike"],
        "ordre des clips des deux esquives"
    );
}

/// Salle réelle d'un jeu (P-1e) : le joueur de `ALACOD_P1E_SCENARIO` (scénario du jeu, entrées gardées, attentes retirées)
/// joué avec la session du client hors ligne, comparé aux nombres de son banc donnés par l'environnement :
/// `ALACOD_P1E_SPEED` (u/s), `ALACOD_P1E_BULLET` (u/s), et les frames d'input `ALACOD_P1E_MOVES` (« 30,60,80,100 » :
/// départ, arrêt, départ, demi-tour), `ALACOD_P1E_FIRE` (frame de l'appui) et `ALACOD_P1E_RAISE` (épaulé, frames) ;
/// à l'écran, `ALACOD_P1E_SHOOT` (« tir:18 » : clip du tir et durée du geste en frames) et `ALACOD_P1E_FIRST_DASH`
/// (« clip:frame » : clip de la première esquive, joué à cette frame).
#[test]
#[ignore = "jeu externe : ALACOD_P1E_GAME_DIR"]
fn real_room_matches_the_bench() {
    if map_ldtk::RENDER_ENABLED {
        panic!("à lancer sans le rendu (--no-default-features)");
    }
    let var = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k}"));
    let dir = PathBuf::from(var("ALACOD_P1E_GAME_DIR"));
    let text = std::fs::read_to_string(dir.join(format!(
        "tests/scenarios/{}.ron",
        var("ALACOD_P1E_SCENARIO")
    )))
    .expect("scénario du jeu");
    let game = text[text.find("game:").expect("game:") + 5..]
        .split('"')
        .nth(1)
        .unwrap()
        .to_string();
    std::env::set_var("ALACOD_GAME_NAME", &game);
    std::env::set_var("ALACOD_GAME_DIR", &dir);
    let mut scenario = Scenario::from_ron(&text).expect("scénario");
    scenario.expect.clear();
    scenario.input_delay = Some(game::args::input_delay_from_env(false));
    let speed: f32 = var("ALACOD_P1E_SPEED").parse().unwrap();
    let bullet: f32 = var("ALACOD_P1E_BULLET").parse().unwrap();
    let fire: u32 = var("ALACOD_P1E_FIRE").parse().unwrap();
    let raise: u32 = var("ALACOD_P1E_RAISE").parse().unwrap();
    let moves: Vec<u32> = var("ALACOD_P1E_MOVES")
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();

    let (shoot_clip, shoot_frames) = var("ALACOD_P1E_SHOOT")
        .split_once(':')
        .map(|(c, n)| (c.to_string(), n.parse::<u32>().unwrap()))
        .expect("ALACOD_P1E_SHOOT : clip:frames");
    let (first_dash, dash_frame) = var("ALACOD_P1E_FIRST_DASH")
        .split_once(':')
        .map(|(c, n)| (c.to_string(), n.parse::<u32>().unwrap()))
        .expect("ALACOD_P1E_FIRST_DASH : clip:frame");
    let mut bullets: Vec<(u32, f32, f32)> = Vec::new();
    let mut played: Vec<(u32, String, f32)> = Vec::new();
    let track = player_track(&scenario, |world, frame| {
        let mut query = world.query::<(&combat::weapons::Bullet, &FixedTransform3D)>();
        if let Some((_, t)) = query.iter(world).find(|(b, _)| b.player_handle == 0) {
            bullets.push((
                frame,
                t.translation.x.to_num::<f32>(),
                t.translation.y.to_num::<f32>(),
            ));
        }
        let mut query = world.query::<(&Player, &ModelPlayback)>();
        if let Some((_, p)) = query.iter(world).find(|(p, _)| p.handle == 0) {
            if let (Some(clip), Some(time)) = (p.clip(), p.time()) {
                played.push((frame, clip.to_string(), time));
            }
        }
    });
    let mut failures = Vec::new();
    for &input in &moves {
        let at = reaction(&track, input);
        println!("input à {input} : réaction à {at}");
        if at > input + 1 {
            failures.push(format!(
                "input à {input} : réaction à la frame {at} (banc : {} au plus)",
                input + 1
            ));
        }
    }
    let steady = step_at(&track, moves[0] + 20).abs() * 60.0;
    println!("vitesse établie {steady:.2} u/s (banc {speed})");
    if (steady - speed).abs() > speed * 0.01 {
        failures.push(format!("vitesse {steady:.2} u/s (banc {speed} ± 1 %)"));
    }
    match (bullets.first(), bullets.get(10)) {
        (Some(a), Some(b)) => {
            let v = ((b.1 - a.1).powi(2) + (b.2 - a.2).powi(2)).sqrt() * 60.0 / (b.0 - a.0) as f32;
            println!(
                "balle : visible dès {} (appui {fire}), {v:.1} u/s (banc {bullet})",
                a.0
            );
            if (v - bullet).abs() > bullet * 0.01 {
                failures.push(format!("balle {v:.1} u/s (banc {bullet} ± 1 %)"));
            }
            if a.0 > fire + raise + 2 {
                failures.push(format!(
                    "balle visible à la frame {} pour un appui à {fire} (banc : épaulé {raise} + 1 image)",
                    a.0
                ));
            }
        }
        _ => failures.push("aucune balle du joueur 0".into()),
    }
    match span(&played, &shoot_clip) {
        Some((a, b)) => {
            println!(
                "clip {shoot_clip} : frames {a}..{b} ({} frames, geste {shoot_frames})",
                b - a + 1
            );
            if b - a + 1 > shoot_frames + 1 {
                failures.push(format!(
                    "clip {shoot_clip} joué {} frames (banc : geste de {shoot_frames})",
                    b - a + 1
                ));
            }
        }
        None => failures.push(format!("clip {shoot_clip} jamais joué")),
    }
    let dash = played
        .iter()
        .find(|(f, ..)| *f == dash_frame)
        .map(|(_, c, _)| c.clone());
    println!("première esquive, frame {dash_frame} : {dash:?} (banc {first_dash})");
    if dash.as_deref() != Some(first_dash.as_str()) {
        failures.push(format!(
            "première esquive : {dash:?} à la frame {dash_frame} (banc : {first_dash})"
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
