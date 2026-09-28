//! Joue chaque scénario de `tests/scenarios/*.ron`, vérifie ses attentes et compare sa
//! trace d'état à `tests/scenarios/<nom>.trace`.
//!
//! - `make test_scenarios` : compile sans rendu, avec le profil `headless`.
//! - `ALACOD_BLESS=1` : réécrit les traces de référence (après un changement voulu).
//! - `ALACOD_SCENARIO=<nom>` : ne joue que ce scénario.

use std::path::PathBuf;

use scenario::{run, Scenario};

fn scenarios_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/scenarios"))
}

#[test]
fn scenarios() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("scénarios ignorés : compilés avec le rendu des tilemaps (utiliser `make test_scenarios`)");
        return;
    }

    let bless = std::env::var("ALACOD_BLESS").is_ok_and(|v| v == "1");
    let only = std::env::var("ALACOD_SCENARIO").ok().filter(|name| !name.is_empty());

    let mut paths: Vec<PathBuf> = std::fs::read_dir(scenarios_dir())
        .expect("dossier tests/scenarios")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ron"))
        .collect();
    paths.sort();

    let mut failures = Vec::new();
    for path in paths {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if only.as_ref().is_some_and(|only| *only != name) {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        let scenario = match Scenario::from_ron(&source) {
            Ok(scenario) => scenario,
            Err(err) => {
                failures.push(format!("{name}: RON invalide : {err}"));
                continue;
            }
        };

        let started = std::time::Instant::now();
        let outcome = run(&scenario);
        eprintln!("{name}: {} ({:.1?})", outcome.summary, started.elapsed());
        if std::env::var("ALACOD_EVENTS").is_ok_and(|v| v == "1") {
            for event in &outcome.events {
                eprintln!("  f{:>5} {:<7} {}", event.frame, event.kind, event.label);
            }
        }

        failures.extend(outcome.failures.iter().map(|f| format!("{name}: {f}")));

        let golden_path = path.with_extension("trace");
        let trace = outcome.trace.join("\n") + "\n";
        if bless {
            std::fs::write(&golden_path, &trace).unwrap();
            eprintln!("{name}: trace de référence écrite");
        } else if let Ok(golden) = std::fs::read_to_string(&golden_path) {
            if let Some((line, (expected, actual))) = golden
                .lines()
                .zip(trace.lines())
                .enumerate()
                .find(|(_, (expected, actual))| expected != actual)
            {
                failures.push(format!(
                    "{name}: trace différente de la référence à la ligne {} (attendu `{expected}`, obtenu `{actual}`) ; ALACOD_BLESS=1 si le changement est voulu",
                    line + 1
                ));
            } else if golden.lines().count() != trace.lines().count() {
                failures.push(format!("{name}: la trace n'a pas la même longueur que la référence"));
            }
        } else {
            failures.push(format!("{name}: pas de trace de référence (lancer avec ALACOD_BLESS=1)"));
        }
    }

    assert!(failures.is_empty(), "\n{}\n", failures.join("\n"));
}

/// Un scénario rejoué depuis son propre enregistrement donne la même trace : ce que
/// l'enregistrement capture suffit à reproduire la partie.
#[test]
fn recording_replays_identically() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }

    let source = std::fs::read_to_string(scenarios_dir().join("shoot_around.ron")).unwrap();
    let original = run(&Scenario::from_ron(&source).unwrap());

    // Le RON écrit doit se relire
    let mut recorded = Scenario::from_ron(&original.recorded.to_ron()).unwrap();
    recorded.frames = original.trace.len() as u32;
    let replayed = run(&recorded);

    assert_eq!(original.trace, replayed.trace, "le replay de l'enregistrement diverge");
}

/// Diagnostic : grille de navigation d'un scénario à une frame donnée.
/// `ALACOD_NAV=idle:400 cargo test -p scenario --profile headless --test scenarios nav_map -- --ignored --nocapture`
#[test]
#[ignore]
fn nav_map() {
    let spec = std::env::var("ALACOD_NAV").unwrap_or_else(|_| "idle:400".into());
    let (name, frame) = spec.split_once(':').expect("ALACOD_NAV=<scénario>:<frame>");
    let source = std::fs::read_to_string(scenarios_dir().join(format!("{name}.ron"))).unwrap();
    let scenario = Scenario::from_ron(&source).unwrap();
    let mut app = scenario::runner::run_until(&scenario, frame.parse().unwrap());
    let arrows = std::env::var("ALACOD_NAV_ARROWS").is_ok_and(|v| v == "1");
    println!("{name} frame {frame}\n{}", scenario::nav_debug::nav_ascii(app.world_mut(), arrows));
}


/// Diagnostic : pour chaque zombie, apparition, premier contact avec un joueur, et plus
/// longue période bloquée en poursuite. `ALACOD_NAV=<scénario>:<frames>`.
#[test]
#[ignore]
fn nav_stats() {
    use bevy::prelude::*;
    use bevy_fixed::fixed_math::FixedTransform3D;
    use game::character::enemy::{ai::{AttackTarget, MonsterState}, Enemy};
    use std::collections::BTreeMap;
    use utils::{frame::FrameCount, net_id::GgrsNetId};
    let spec = std::env::var("ALACOD_NAV").unwrap_or_else(|_| "idle:1180".into());
    let (name, frames) = spec.split_once(':').unwrap();
    let frames: u32 = frames.parse().unwrap();
    let source = std::fs::read_to_string(scenarios_dir().join(format!("{name}.ron"))).unwrap();
    let mut app = scenario::runner::build_app(&Scenario::from_ron(&source).unwrap(), true);
    app.finish();
    app.cleanup();
    #[derive(Default)]
    struct Z { spawn: u32, contact: Option<u32>, window: Option<u32>, history: Vec<(u32, f32, f32, bool)> }
    let mut zombies: BTreeMap<usize, Z> = BTreeMap::new();
    for _ in 0..20_000 {
        app.update();
        let frame = app.world().resource::<FrameCount>().frame;
        let world = app.world_mut();
        let mut q = world.query_filtered::<(&GgrsNetId, &FixedTransform3D, &MonsterState), With<Enemy>>();
        for (id, t, state) in q.iter(world) {
            let z = zombies.entry(id.0).or_insert_with(|| Z { spawn: frame, ..Default::default() });
            let attacking_player = matches!(state, MonsterState::Attacking { target: AttackTarget::Player { .. }, .. });
            let attacking_obstacle = matches!(state, MonsterState::Attacking { target: AttackTarget::Obstacle { .. }, .. });
            if attacking_player && z.contact.is_none() { z.contact = Some(frame); }
            if attacking_obstacle && z.window.is_none() { z.window = Some(frame); }
            z.history.push((frame, t.translation.x.to_num(), t.translation.y.to_num(), matches!(state, MonsterState::Chasing)));
        }
        if frame >= frames { break; }
    }
    let mut total_stuck = 0;
    for (id, z) in &zombies {
        // Plus longue suite de fenêtres de 60 frames en poursuite avec moins de 2 px de déplacement
        let mut stuck = 0; let mut longest = 0; let mut where_ = (0.0, 0.0);
        for w in z.history.windows(60) {
            let (a, b) = (w[0], w[59]);
            let moved = ((b.1 - a.1).powi(2) + (b.2 - a.2).powi(2)).sqrt();
            if w.iter().all(|h| h.3) && moved < 2.0 { stuck += 1; if stuck > longest { longest = stuck; where_ = (b.1, b.2); } } else { stuck = 0; }
        }
        if longest > 0 { total_stuck += 1; }
        println!("zombie {id:>3} apparu f{:<5} contact joueur {:<7} fenêtre {:<7} bloqué max {:>4} frames{}",
            z.spawn, z.contact.map_or("-".into(), |f| format!("f{f}")), z.window.map_or("-".into(), |f| format!("f{f}")),
            longest, if longest > 0 { format!(" vers ({:.0},{:.0})", where_.0, where_.1) } else { String::new() });
    }
    let contacts = zombies.values().filter(|z| z.contact.is_some()).count();
    println!("{name} jusqu'à f{frames} : {} zombies, {contacts} au contact d'un joueur, {total_stuck} bloqués", zombies.len());
}

/// Diagnostic : un zombie à une frame (`ALACOD_PROBE=<net_id>:<frame>`, scénario idle).
#[test]
#[ignore]
fn nav_probe() {
    use bevy::prelude::*;
    use bevy_fixed::fixed_math::{self, FixedTransform3D};
    use game::character::enemy::{ai::{FlowFieldCache, GridPos, MonsterState, NavProfile}, Enemy};
    use game::character::movement::Velocity;
    use game::collider::{Collider, ColliderShape, Wall};
    use utils::net_id::GgrsNetId;
    let spec = std::env::var("ALACOD_PROBE").unwrap_or_else(|_| "116:560".into());
    let (id, frame) = spec.split_once(':').unwrap();
    let (id, frame): (usize, u32) = (id.parse().unwrap(), frame.parse().unwrap());
    let source = std::fs::read_to_string(scenarios_dir().join("idle.ron")).unwrap();
    let mut app = scenario::runner::run_until(&Scenario::from_ron(&source).unwrap(), frame);
    let world = app.world_mut();
    let cache = world.resource::<FlowFieldCache>().clone();
    let field = cache.get_flow_field(NavProfile::GroundBreaker).cloned().unwrap_or_default();
    let mut q = world.query_filtered::<(&GgrsNetId, &FixedTransform3D, &Velocity, &MonsterState, &Collider), With<Enemy>>();
    let Some((_, t, v, state, c)) = q.iter(world).find(|(n, ..)| n.0 == id) else { println!("zombie {id} absent"); return };
    let pos = t.translation.truncate();
    let cell = GridPos::from_fixed(pos);
    let next = field.get_direction(cell);
    let body = game::character::enemy::ai::navigation::AgentBody::from_collider(c);
    println!("zombie {id} f{frame} pos ({:.2},{:.2}) case {:?} → {:?} coût {:?} état {state:?}",
        pos.x.to_num::<f32>(), pos.y.to_num::<f32>(), (cell.x, cell.y), next.map(|n| (n.x, n.y)), field.costs.get(&cell));
    if let Some(n) = next { let sp = cache.steering_point(n, NavProfile::GroundBreaker, &body); println!("   point visé ({:.1},{:.1})", sp.x.to_num::<f32>(), sp.y.to_num::<f32>()); }
    println!("   vitesse main ({:.2},{:.2}) knockback ({:.2},{:.2}) collider {:?}", v.main.x.to_num::<f32>(), v.main.y.to_num::<f32>(), v.knockback.x.to_num::<f32>(), v.knockback.y.to_num::<f32>(), c.shape);
    for dy in (-2..=2).rev() {
        let row: String = (-3..=3).map(|dx| {
            let p = GridPos::new(cell.x + dx, cell.y + dy);
            if dx == 0 && dy == 0 { 'Z' } else if cache.is_blocked(&p, NavProfile::GroundBreaker) { '#' } else if field.directions.contains_key(&p) { '.' } else { ' ' }
        }).collect();
        println!("   {row}   y={}", cell.y + dy);
    }
    let mut walls = world.query_filtered::<(&FixedTransform3D, &Collider), With<Wall>>();
    for (wt, wc) in walls.iter(world) {
        let ColliderShape::Rectangle { width, height } = wc.shape else { continue };
        let (x0, x1) = ((wt.translation.x - width / fixed_math::new(2.0)).to_num::<f32>(), (wt.translation.x + width / fixed_math::new(2.0)).to_num::<f32>());
        let (y0, y1) = ((wt.translation.y - height / fixed_math::new(2.0)).to_num::<f32>(), (wt.translation.y + height / fixed_math::new(2.0)).to_num::<f32>());
        let (px, py) = (pos.x.to_num::<f32>(), pos.y.to_num::<f32>());
        let gap_x = (x0 - px).max(px - x1).max(0.0); let gap_y = (y0 - py).max(py - y1).max(0.0);
        if gap_x < 25.0 && gap_y < 25.0 { println!("   mur x {x0}..{x1} y {y0}..{y1} (écart {gap_x:.1},{gap_y:.1})"); }
    }
}
