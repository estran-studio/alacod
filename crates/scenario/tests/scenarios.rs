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
    let config = scenario::runner::PlayConfig { follow_handle: None };
    let mut app = scenario::runner::build_app(&Scenario::from_ron(&source).unwrap(), true, &config);
    app.finish();
    app.cleanup();
    #[derive(Default)]
    struct Z { spawn: u32, contact: Option<u32>, window: Option<u32>, history: Vec<(u32, f32, f32, bool)>, clip_frames: u32, clip_max: f32, clip_at: (u32, f32, f32) }
    // Murs (y compris portes fermées) en rectangles monde, lus à chaque frame
    use game::collider::{Collider, ColliderShape, Wall};
    const SPRITE_HALF: f32 = 16.0; // sprite 32x32 centré sur le zombie
    const CLIP_TOLERANCE: f32 = 3.0;
    let mut clip_kind: BTreeMap<&str, u32> = BTreeMap::new();
    let mut clip_spots: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    let mut zombies: BTreeMap<usize, Z> = BTreeMap::new();
    for _ in 0..20_000 {
        app.update();
        let frame = app.world().resource::<FrameCount>().frame;
        let world = app.world_mut();
        let mut wq = world.query_filtered::<(&FixedTransform3D, &Collider), With<Wall>>();
        let walls: Vec<(f32, f32, f32, f32)> = wq.iter(world).filter_map(|(t, c)| {
            let ColliderShape::Rectangle { width, height } = c.shape else { return None };
            let (x, y, w, h) = (t.translation.x.to_num::<f32>(), t.translation.y.to_num::<f32>(), width.to_num::<f32>() / 2.0, height.to_num::<f32>() / 2.0);
            Some((x - w, x + w, y - h, y + h))
        }).collect();
        // Ouvertures : fenêtres (obstacles) et portes
        let mut oq = world.query_filtered::<&FixedTransform3D, Or<(With<game::character::enemy::ai::Obstacle>, With<map::game::entity::map::door::DoorComponent>)>>();
        let openings: Vec<(f32, f32)> = oq.iter(world).map(|t| (t.translation.x.to_num(), t.translation.y.to_num())).collect();
        let mut q = world.query_filtered::<(&GgrsNetId, &FixedTransform3D, &MonsterState), With<Enemy>>();
        for (id, t, state) in q.iter(world) {
            let z = zombies.entry(id.0).or_insert_with(|| Z { spawn: frame, ..Default::default() });
            let attacking_player = matches!(state, MonsterState::Attacking { target: AttackTarget::Player { .. }, .. });
            let attacking_obstacle = matches!(state, MonsterState::Attacking { target: AttackTarget::Obstacle { .. }, .. });
            if attacking_player && z.contact.is_none() { z.contact = Some(frame); }
            if attacking_obstacle && z.window.is_none() { z.window = Some(frame); }
            z.history.push((frame, t.translation.x.to_num(), t.translation.y.to_num(), matches!(state, MonsterState::Chasing)));
            // Chevauchement visuel : profondeur du sprite dans le mur le plus enfoncé
            let (px, py): (f32, f32) = (t.translation.x.to_num(), t.translation.y.to_num());
            let depth = walls.iter().map(|(x0, x1, y0, y1)| {
                let ox = (px + SPRITE_HALF).min(*x1) - (px - SPRITE_HALF).max(*x0);
                let oy = (py + SPRITE_HALF).min(*y1) - (py - SPRITE_HALF).max(*y0);
                if ox > 0.0 && oy > 0.0 { ox.min(oy) } else { 0.0 }
            }).fold(0.0, f32::max);
            if depth > CLIP_TOLERANCE {
                z.clip_frames += 1;
                let near_opening = openings.iter().any(|(ox, oy)| (ox - px).abs() < 40.0 && (oy - py).abs() < 40.0);
                let kind = if near_opening { "ouverture (porte/fenêtre)" } else if matches!(state, MonsterState::Attacking { .. }) { "en attaque" } else { "longe un mur" };
                *clip_kind.entry(kind).or_default() += 1;
                if kind == "longe un mur" { *clip_spots.entry(((px / 16.0) as i32, (py / 16.0) as i32)).or_default() += 1; }
            }
            if depth > z.clip_max { z.clip_max = depth; z.clip_at = (frame, px, py); }
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
        println!("zombie {id:>3} apparu f{:<5} attaque fenêtre {:<7} contact joueur {:<7} bloqué max {:>4} frames  sprite dans un mur {:>4} frames (max {:>4.1} px){}",
            z.spawn, z.window.map_or("-".into(), |f| format!("f{f}")), z.contact.map_or("-".into(), |f| format!("f{f}")),
            longest, z.clip_frames, z.clip_max, if z.clip_max > CLIP_TOLERANCE { format!(" à f{} ({:.0},{:.0})", z.clip_at.0, z.clip_at.1, z.clip_at.2) } else { String::new() } + &if longest > 0 { format!(" bloqué vers ({:.0},{:.0})", where_.0, where_.1) } else { String::new() });
    }
    let contacts = zombies.values().filter(|z| z.contact.is_some()).count();
    println!("répartition : {clip_kind:?}");
    let mut spots: Vec<_> = clip_spots.into_iter().collect();
    spots.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    println!("cases où les sprites longent un mur : {:?}", &spots[..spots.len().min(8)]);
    let clip_total: u32 = zombies.values().map(|z| z.clip_frames).sum();
    let alive_total: usize = zombies.values().map(|z| z.history.len()).sum();
    println!("{name} jusqu'à f{frames} : {} zombies, {contacts} au contact d'un joueur, {total_stuck} bloqués, sprite dans un mur {clip_total}/{alive_total} frames-zombie ({:.1} %)",
        zombies.len(), 100.0 * clip_total as f32 / alive_total.max(1) as f32);
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

/// Diagnostic : état des armes d'un joueur, frame par frame.
/// `ALACOD_WEAPON_PROBE=<scénario>:<de>:<à>:<pas>`
#[test]
#[ignore]
fn weapon_probe() {
    use bevy::prelude::*;
    use game::character::{dash::DashState, movement::SprintState, player::Player};
    use game::weapons::{melee::MeleeAttackState, WeaponInventory, WeaponModesState, WeaponState};
    use utils::frame::FrameCount;
    let spec = std::env::var("ALACOD_WEAPON_PROBE").unwrap_or_else(|_| "weapons_workout:480:570:5".into());
    let parts: Vec<&str> = spec.split(':').collect();
    let (name, from, to, step): (&str, u32, u32, u32) = (parts[0], parts[1].parse().unwrap(), parts[2].parse().unwrap(), parts[3].parse().unwrap());
    let source = std::fs::read_to_string(scenarios_dir().join(format!("{name}.ron"))).unwrap();
    let mut app = scenario::runner::run_until(&Scenario::from_ron(&source).unwrap(), from);
    loop {
        let frame = app.world().resource::<FrameCount>().frame;
        if frame >= to { break; }
        if (frame - from) % step == 0 {
            let world = app.world_mut();
            let mut q = world.query::<(&Player, &WeaponInventory, &SprintState, &DashState, &MeleeAttackState)>();
            let rows: Vec<_> = q.iter(world).map(|(p, inv, sprint, dash, melee)| (p.handle, inv.active_weapon_index, inv.weapons.get(inv.active_weapon_index).map(|(e, w)| (*e, w.config.name.clone())), inv.reloading_ending_frame, inv.frame_switched, sprint.is_sprinting, dash.is_dashing, melee.is_attacking)).collect();
            for (handle, idx, weapon, reload_end, switched, sprinting, dashing, meleeing) in rows {
                let (entity, wname) = weapon.unwrap();
                let (state, modes) = world.query::<(&WeaponState, &WeaponModesState)>().get(world, entity).unwrap();
                let m = modes.modes.get(&state.active_mode).unwrap();
                println!("f{frame} j{handle} arme#{idx} {wname}/{} balles {} chargeurs {} dernier_tir f{} recharge_fin {:?} changé f{switched} sprint={sprinting} dash={dashing} mêlée={meleeing}",
                    state.active_mode, m.mag_ammo, m.mag_quantity, state.last_fire_frame, reload_end);
            }
        }
        app.update();
    }
}

/// Diagnostic : positions des joueurs, portes et fenêtres (et leur état) à une frame.
/// `ALACOD_MAP_PROBE=<scénario>:<frame>`
#[test]
#[ignore]
fn map_probe() {
    use bevy::prelude::*;
    use bevy_fixed::fixed_math::FixedTransform3D;
    use game::character::{enemy::ai::Obstacle, player::Player};
    use game::collider::Collider;
    use game::interaction::Interactable;
    use map::game::entity::map::{door::DoorComponent, window::WindowHealth};
    use utils::net_id::GgrsNetId;
    let spec = std::env::var("ALACOD_MAP_PROBE").unwrap_or_else(|_| "idle:5".into());
    let (name, frame) = spec.split_once(':').unwrap();
    let source = std::fs::read_to_string(scenarios_dir().join(format!("{name}.ron"))).unwrap();
    let mut app = scenario::runner::run_until(&Scenario::from_ron(&source).unwrap(), frame.parse().unwrap());
    let world = app.world_mut();
    let pos = |t: &FixedTransform3D| (t.translation.x.to_num::<f32>(), t.translation.y.to_num::<f32>());
    for (p, t) in world.query::<(&Player, &FixedTransform3D)>().iter(world) { println!("joueur {} {:?}", p.handle, pos(t)); }
    let mut windows: Vec<_> = world.query::<(&GgrsNetId, &FixedTransform3D, &WindowHealth, &Obstacle, Option<&Interactable>)>().iter(world)
        .map(|(id, t, h, o, i)| (id.0, pos(t), h.current, o.blocks_movement, i.map(|i| i.interaction_range.to_num::<f32>()))).collect();
    windows.sort_by_key(|w| w.0);
    for w in windows { println!("fenêtre {} {:?} santé {} bloque_zombies={} portée_interaction={:?}", w.0, w.1, w.2, w.3, w.4); }
    let mut doors: Vec<_> = world.query::<(&GgrsNetId, &FixedTransform3D, Option<&Collider>, Option<&Interactable>, &DoorComponent)>().iter(world)
        .map(|(id, t, c, i, d)| (id.0, pos(t), c.is_some(), i.map(|i| i.interaction_range.to_num::<f32>()), d.config.interactable)).collect();
    doors.sort_by_key(|d| d.0);
    for d in doors { println!("porte {} {:?} fermée={} portée_interaction={:?} interactive={}", d.0, d.1, d.2, d.3, d.4); }
}

/// Diagnostic : GgrsNetId des entités rollback à une frame (`ALACOD_IDS=<scénario>:<frame>`).
#[test]
#[ignore]
fn net_ids() {
    use utils::net_id::GgrsNetId;
    let spec = std::env::var("ALACOD_IDS").unwrap_or_else(|_| "idle:1".into());
    let (name, frame) = spec.split_once(':').unwrap();
    let source = std::fs::read_to_string(scenarios_dir().join(format!("{name}.ron"))).unwrap();
    let mut app = scenario::runner::run_until(&Scenario::from_ron(&source).unwrap(), frame.parse().unwrap());
    let world = app.world_mut();
    let mut ids: Vec<(usize, String)> = world.query::<&GgrsNetId>().iter(world).map(|id| (id.0, id.1.clone())).collect();
    ids.sort();
    for (id, name) in ids { println!("id {id:>3} {name}"); }
}

/// Diagnostic : balles vivantes et la plus à gauche/droite, frame par frame
/// (`ALACOD_BULLETS=<scénario>:<de>:<à>`).
#[test]
#[ignore]
fn bullets_probe() {
    use bevy::prelude::*;
    use bevy_fixed::fixed_math::FixedTransform3D;
    use game::weapons::Bullet;
    use utils::frame::FrameCount;
    let spec = std::env::var("ALACOD_BULLETS").unwrap_or_else(|_| "bullets_walls:0:400".into());
    let p: Vec<&str> = spec.split(':').collect();
    let (from, to): (u32, u32) = (p[1].parse().unwrap(), p[2].parse().unwrap());
    let source = std::fs::read_to_string(scenarios_dir().join(format!("{}.ron", p[0]))).unwrap();
    let mut app = scenario::runner::run_until(&Scenario::from_ron(&source).unwrap(), from);
    let (mut min_x, mut max_x, mut max_y, mut max_alive) = (f32::MAX, f32::MIN, f32::MIN, 0usize);
    while app.world().resource::<FrameCount>().frame < to {
        app.update();
        let world = app.world_mut();
        let ps: Vec<(f32, f32)> = world.query_filtered::<&FixedTransform3D, With<Bullet>>().iter(world).map(|t| (t.translation.x.to_num(), t.translation.y.to_num())).collect();
        max_alive = max_alive.max(ps.len());
        for (x, y) in ps { min_x = min_x.min(x); max_x = max_x.max(x); max_y = max_y.max(y); }
    }
    println!("balles : au plus {max_alive} en vol, x {min_x:.1}..{max_x:.1}, y max {max_y:.1}");
}
