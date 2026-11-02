//! T1.6 : une caverne (`cave:petite` du testbed) se charge par le chemin LDtk ordinaire, et
//! l'attente `CellState` lit `world::CellGrid`.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use bevy_ggrs::Rollback;
use game::character::player::Player;
use game::collider::Wall;
use game::replay::{Expectation, Scenario};
use scenario::{run, run_with};
use world::{CaveConfig, CellGrid, CellKind};

const SEED: i32 = 123456;

fn petite() -> CaveConfig {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../games/testbed/assets/caves/petite.ron"
    );
    ron::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn cave_scenario(frames: u32, expect: Vec<Expectation>) -> Scenario {
    let mut scenario = Scenario::from_ron(&format!(
        r#"(game: "testbed", map: "cave:petite", map_seed: {SEED}, frames: {frames},
            players: [()])"#
    ))
    .expect("scénario de caverne");
    scenario.expect = expect;
    scenario
}

/// Une case de chaque nature dans la grille générée pour `SEED`.
fn sample(grid: &CellGrid, kind: CellKind) -> (i32, i32) {
    (0..grid.height as i32)
        .flat_map(|y| (0..grid.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| grid.get(x, y) == Some(kind))
        .unwrap()
}

#[test]
fn cell_state_suit_la_grille_generee() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let grid = world::generate(SEED as u32 as u64, &petite());
    let expect = [CellKind::Wall, CellKind::Rock, CellKind::Floor]
        .into_iter()
        .map(|kind| {
            let (x, y) = sample(&grid, kind);
            Expectation::CellState {
                x,
                y,
                kind: Some(kind),
                surface: None,
                at_frame: 10,
            }
        })
        .collect();
    let outcome = run(&cave_scenario(20, expect));
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);

    // Mauvaise nature et case hors grille : échecs
    let (x, y) = sample(&grid, CellKind::Rock);
    let outcome = run(&cave_scenario(
        20,
        vec![
            Expectation::CellState {
                x,
                y,
                kind: Some(CellKind::Floor),
                surface: None,
                at_frame: 10,
            },
            Expectation::CellState {
                x: -1,
                y: 0,
                kind: Some(CellKind::Wall),
                surface: None,
                at_frame: 10,
            },
        ],
    ));
    assert_eq!(outcome.failures.len(), 2, "{:?}", outcome.failures);
}

/// Les murs physiques et la grille viennent des mêmes cases ; le joueur apparaît sur le sol.
#[test]
fn caverne_chargee_par_le_chemin_ldtk() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // (murs rollback, grille, case du joueur), relevés à chaque frame
    type Seen = Arc<Mutex<(usize, CellGrid, Option<(i32, i32)>)>>;
    let seen: Seen = Arc::default();
    let probe = seen.clone();
    let outcome = run_with(&cave_scenario(20, vec![]), move |app| {
        app.add_systems(
            Last,
            move |walls: Query<(), (With<Wall>, With<Rollback>)>,
                  grid: Res<CellGrid>,
                  players: Query<&FixedTransform3D, With<Player>>| {
                let mut seen = probe.lock().unwrap();
                seen.0 = walls.iter().count();
                seen.1 = grid.clone();
                seen.2 = players
                    .iter()
                    .next()
                    .map(|t| CellGrid::cell_of(t.translation.truncate()));
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let (walls, grid, player) = seen.lock().unwrap().clone();
    assert_eq!(grid, world::generate(SEED as u32 as u64, &petite()));
    assert!(walls > 0, "aucun mur de caverne");
    let (x, y) = player.expect("joueur absent");
    assert_eq!(
        grid.get(x, y),
        Some(CellKind::Floor),
        "joueur en ({x}, {y})"
    );
}

/// Une demande de destruction (sans projectile) creuse un cratère de roche, recrée les murs et
/// reste déterministe en synctest (rejeux à chaque frame : murs recréés et navigation
/// resynchronisée depuis `CellGrid`, état rollback).
#[test]
fn destruction_creuse_et_reconstruit_les_murs_en_synctest() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use bevy_ggrs::GgrsSchedule;
    use sim_core::frame_events::FrameEvents;
    use sim_core::system_set::RollbackSystemSet;
    use utils::frame::FrameCount;
    use world::DestroyTerrainRequest;

    const AT: u32 = 30;
    let grid = world::generate(SEED as u32 as u64, &petite());
    // Une case de roche voisine du sol : le cratère de rayon 40 la contient
    let (x, y) = (1..grid.height as i32 - 1)
        .flat_map(|y| (1..grid.width as i32 - 1).map(move |x| (x, y)))
        .find(|&(x, y)| {
            grid.get(x, y) == Some(CellKind::Rock) && grid.get(x + 1, y) == Some(CellKind::Floor)
        })
        .unwrap();
    let center = CellGrid::cell_center(x as u32, y as u32);
    let mut after = grid.clone();
    let destroyed = world::destroy_terrain(
        &mut after,
        center,
        bevy_fixed::bevy_fixed::fixed_math::Fixed::from_num(40),
    );
    assert!(destroyed.len() > 1);

    let mut expect = vec![
        Expectation::CellState {
            x,
            y,
            kind: Some(CellKind::Rock),
            surface: None,
            at_frame: AT - 1,
        },
        Expectation::CellState {
            x,
            y,
            kind: Some(CellKind::Floor),
            surface: None,
            at_frame: AT + 5,
        },
    ];
    // Les bordures restent des murs
    expect.push(Expectation::CellState {
        x: 0,
        y,
        kind: Some(CellKind::Wall),
        surface: None,
        at_frame: AT + 5,
    });
    let seen: Arc<Mutex<(usize, CellGrid)>> = Arc::default();
    let probe = seen.clone();
    let outcome = run_with(&cave_scenario(AT + 60, expect), move |app| {
        app.add_systems(
            GgrsSchedule,
            (move |frame: Res<FrameCount>,
                   mut requests: ResMut<FrameEvents<DestroyTerrainRequest>>| {
                if frame.frame == AT {
                    requests.send(DestroyTerrainRequest::new(
                        center,
                        bevy_fixed::bevy_fixed::fixed_math::Fixed::from_num(40),
                    ));
                }
            })
            .after(RollbackSystemSet::Projectiles)
            .before(RollbackSystemSet::World),
        )
        .add_systems(
            Last,
            move |walls: Query<(), (With<Wall>, With<Rollback>)>, grid: Res<CellGrid>| {
                *probe.lock().unwrap() = (walls.iter().count(), grid.clone());
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let (walls, final_grid) = seen.lock().unwrap().clone();
    assert_eq!(final_grid, after, "grille finale = grille creusée");
    assert!(walls > 0);
}

/// Mode `Floors` : une caverne est un niveau valide d'une séquence (`floors/caverne.ron` du
/// testbed : `floor_a`, `cave:petite`, `floor_b`). Mêmes inputs que `portal_next_floor` : le
/// follower de `floor_a` meurt, le joueur entre dans le portail et passe dans la caverne
/// (`CellGrid` remplie au passage) ; sans ennemi, le portail de la caverne s'ouvre aussitôt et
/// mène à `floor_b`, où `CellGrid` redevient vide.
#[test]
fn caverne_dans_une_sequence_floors() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = Scenario::from_ron(&format!(
        r#"(game: "testbed", floors: Some("caverne"), map_seed: {SEED}, frames: 300,
            powerup_drop_chance_override: "0.0",
            players: [(inputs: [
                (from: 0, to: 200, buttons: [Fire], pan: (-64, 48)),
                (from: 220, to: 226, buttons: [Right, Down], pan: (0, 0)),
            ])])"#
    ))
    .expect("scénario Floors");
    scenario.expect = vec![
        Expectation::FloorIndex {
            index: 0,
            at_frame: 220,
        },
        Expectation::FloorIndex {
            index: 2,
            at_frame: 299,
        },
    ];
    // Niveau courant -> la grille a-t-elle été vue remplie / vide pendant ce niveau
    type Seen = Arc<Mutex<std::collections::BTreeMap<u32, (bool, bool)>>>;
    let seen: Seen = Arc::default();
    let probe = seen.clone();
    let outcome = run_with(&scenario, move |app| {
        app.add_systems(
            Last,
            move |grid: Res<CellGrid>, floor: Res<run::FloorState>| {
                let mut seen = probe.lock().unwrap();
                let entry = seen.entry(floor.index).or_default();
                if grid.is_empty() {
                    entry.1 = true;
                } else {
                    entry.0 = true;
                }
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(
        seen.get(&1),
        Some(&(true, false)),
        "caverne : grille remplie ({seen:?})"
    );
    assert_eq!(
        seen.get(&2),
        Some(&(false, true)),
        "floor_b : grille vide ({seen:?})"
    );
}

/// `cave:bench` : un `follower` (équipe `enemies`) par point `ZombieSpawn` ; ils poursuivent
/// le joueur par la navigation de la caverne.
#[test]
fn followers_de_la_caverne_naviguent() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    use game::character::enemy::Enemy;
    use game::replay::EntityKind;

    let mut scenario = Scenario::from_ron(&format!(
        r#"(game: "testbed", map: "cave:bench", map_seed: {SEED}, frames: 400,
            players: [()])"#
    ))
    .expect("scénario bench");
    scenario.expect = vec![Expectation::EntityCount {
        kind: EntityKind::Enemy,
        min: Some(6),
        max: Some(6),
        at_frame: 10,
    }];
    // Distance totale des ennemis au joueur, à f10 puis à la dernière frame
    let seen: Arc<Mutex<Vec<f32>>> = Arc::default();
    let probe = seen.clone();
    let outcome = run_with(&scenario, move |app| {
        app.add_systems(
            Last,
            move |frame: Res<utils::frame::FrameCount>,
                  enemies: Query<&FixedTransform3D, With<Enemy>>,
                  players: Query<&FixedTransform3D, With<Player>>| {
                let Some(player) = players.iter().next() else {
                    return;
                };
                if frame.frame == 10 || frame.frame == 399 {
                    let p = player.translation.truncate();
                    let total: f32 = enemies
                        .iter()
                        .map(|e| (e.translation.truncate() - p).length().to_num::<f32>())
                        .sum();
                    probe.lock().unwrap().push(total);
                }
            },
        );
    });
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let seen = seen.lock().unwrap().clone();
    assert!(seen.len() >= 2, "{seen:?}");
    assert!(
        seen.last().unwrap() < &(seen[0] * 0.7),
        "les followers ne se rapprochent pas : {seen:?}"
    );
}

/// Critère T1.6 : `bench_cave` creuse au moins 50 fois (métrique `terrain_destroyed`). D51
/// (dispersion de la config enfin appliquée, m1-d51-dispersion-ignoree) : les tirs partent dans la
/// visée au lieu de ±0,5 rad et creusent moins de cases différentes : 39 destructions mesurées ;
/// seuil ramené à 35 (marge sous la mesure), le nom du test garde l'histoire du critère.
#[test]
fn bench_cave_detruit_au_moins_50() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/bench_cave.ron"
    );
    let scenario = Scenario::from_ron(&std::fs::read_to_string(path).unwrap()).unwrap();
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert!(
        outcome.metrics.terrain_destroyed >= 35,
        "{} destructions",
        outcome.metrics.terrain_destroyed
    );
}

/// Chemin `on_hit` (T1.6) : la `foreuse` (sans `on_expire`) creuse au point d'impact de chaque
/// mur touché (`ProjectileWallHit` → `DestroyTerrainRequest`). Depuis (23, 15), la première
/// roche à l'est est (26, 15).
#[test]
fn foreuse_creuse_au_contact() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let mut scenario = Scenario::from_ron(&format!(
        r#"(game: "testbed", map: "cave:petite", map_seed: {SEED}, frames: 200,
            players: [(weapon: "foreuse", inputs: [
                (from: 30, to: 32, buttons: [Fire], pan: (100, 0)),
            ])])"#
    ))
    .expect("scénario foreuse");
    scenario.expect = vec![
        Expectation::CellState {
            x: 26,
            y: 15,
            kind: Some(CellKind::Rock),
            surface: None,
            at_frame: 29,
        },
        Expectation::CellState {
            x: 26,
            y: 15,
            kind: Some(CellKind::Floor),
            surface: None,
            at_frame: 150,
        },
        Expectation::Event {
            kind: "terrain".into(),
            label_contains: None,
            by_frame: 150,
        },
    ];
    let outcome = run(&scenario);
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

/// S2-12 (D178, `docs/conventions.md` § 104) : audit des cavernes d'un jeu, sans simulation. Pour chaque caverne
/// du registre (config **calculée** par le registre : `spawn_clearance`, `nav_large`) et chaque graine, mesure le sol,
/// les **chambres** (composantes 4-connexes des cases dont le carré 5×5 est du sol, `is_open_within` à 2 ; on ne
/// compte que celles de 12 cases ou plus), les cases de couloir (sol hors `is_open`), les impasses (sol à un voisin
/// de sol ou moins), les `ZombieSpawn` placés et leur répartition par chambre (chambre d'un point = celle de la case de
/// chambre la plus proche par le sol). Diagnostic, rien n'est vérifié :
///
/// `ALACOD_CAVE_AUDIT=throne[:1..100] cargo test -p scenario --profile headless --no-default-features --test cave cave_audit -- --ignored --nocapture`
///
/// `ALACOD_CAVE_AUDIT_SET="niveau_3.fill_ratio=0.42,niveau_3.width=72"` essaie des réglages sans toucher aux fichiers
/// (champs `fill_ratio`, `width`, `height`, `iterations`, `birth`, `survive`, `min_floor_ratio`, `enemy_spawns`).
#[test]
#[ignore]
fn cave_audit() {
    use std::collections::VecDeque;
    let spec = std::env::var("ALACOD_CAVE_AUDIT").unwrap_or_else(|_| "throne".into());
    let (game, seeds) = match spec.split_once(':') {
        Some((game, range)) => {
            let (a, b) = range.split_once("..").expect("graines a..b");
            (game.to_string(), a.parse::<u64>().unwrap()..=b.parse::<u64>().unwrap())
        }
        None => (spec, 1..=100),
    };
    let (registry, _, _) =
        content::load_and_lint(&scenario::runner::game_dir(&game)).expect("manifeste");
    let overrides = std::env::var("ALACOD_CAVE_AUDIT_SET").unwrap_or_default();

    fn median(values: &mut [f64]) -> f64 {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        values[values.len() / 2]
    }
    fn stats(name: &str, mut values: Vec<f64>) {
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let med = median(&mut values);
        println!("  {name:<44} médiane {med:>8.2}   min {min:>8.2}   max {max:>8.2}");
    }

    for cave in registry.caves.values() {
        let id = cave.id.to_string();
        let mut config = cave.config.clone();
        for item in overrides.split(',').filter(|s| !s.is_empty()) {
            let (key, value) = item.split_once('=').expect("champ=valeur");
            let Some((cave_id, field)) = key.split_once('.') else {
                panic!("réglage « {item} » : <caverne>.<champ>=<valeur>")
            };
            if cave_id != id {
                continue;
            }
            match field {
                "fill_ratio" => config.fill_ratio = value.parse::<f64>().map(bevy_fixed::fixed_math::Fixed::from_num).unwrap(),
                "min_floor_ratio" => {
                    config.min_floor_ratio = value.parse::<f64>().map(bevy_fixed::fixed_math::Fixed::from_num).unwrap()
                }
                "width" => config.width = value.parse().unwrap(),
                "height" => config.height = value.parse().unwrap(),
                "iterations" => config.iterations = value.parse().unwrap(),
                "birth" => config.birth = value.parse().unwrap(),
                "survive" => config.survive = value.parse().unwrap(),
                "enemy_spawns" => config.enemy_spawns = value.parse().unwrap(),
                other => panic!("champ inconnu « {other} »"),
            }
        }
        if config.enemy_spawns == 0 {
            continue;
        }
        let (mut floor, mut chambers, mut largest, mut corridors, mut dead_ends) =
            (vec![], vec![], vec![], vec![], vec![]);
        let (mut placed, mut occupied, mut in_largest, mut same_pairs, mut dist_min, mut dist_max) =
            (vec![], vec![], vec![], vec![], vec![], vec![]);
        let mut short = Vec::new();
        for seed in seeds.clone() {
            let grid = world::generate(seed, &config);
            let (w, h) = (grid.width, grid.height);
            let is_floor = |x: i32, y: i32| grid.get(x, y) == Some(CellKind::Floor);
            let cells: Vec<(u32, u32)> =
                (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).collect();
            let floor_cells = cells.iter().filter(|&&(x, y)| is_floor(x as i32, y as i32)).count();
            floor.push(floor_cells as f64 / (w * h) as f64);
            corridors.push(
                cells
                    .iter()
                    .filter(|&&(x, y)| is_floor(x as i32, y as i32) && !world::cave::is_open(&grid, x, y))
                    .count() as f64,
            );
            dead_ends.push(
                cells
                    .iter()
                    .filter(|&&(x, y)| {
                        is_floor(x as i32, y as i32)
                            && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                                .iter()
                                .filter(|(dx, dy)| is_floor(x as i32 + dx, y as i32 + dy))
                                .count()
                                <= 1
                    })
                    .count() as f64,
            );
            // Chambres : composantes de cases à carré 5×5 de sol.
            let mut chamber = vec![usize::MAX; (w * h) as usize];
            let mut sizes: Vec<usize> = Vec::new();
            for &(x, y) in &cells {
                let i = (y * w + x) as usize;
                if chamber[i] != usize::MAX || !world::is_open_within(&grid, x, y, 2) {
                    continue;
                }
                let label = sizes.len();
                let mut size = 0;
                let mut queue = VecDeque::from([(x, y)]);
                chamber[i] = label;
                while let Some((cx, cy)) = queue.pop_front() {
                    size += 1;
                    for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, ny) = (cx as i32 + dx, cy as i32 + dy);
                        if !grid.in_bounds(nx, ny) {
                            continue;
                        }
                        let j = (ny as u32 * w + nx as u32) as usize;
                        if chamber[j] == usize::MAX && world::is_open_within(&grid, nx as u32, ny as u32, 2) {
                            chamber[j] = label;
                            queue.push_back((nx as u32, ny as u32));
                        }
                    }
                }
                sizes.push(size);
            }
            let big: Vec<usize> = (0..sizes.len()).filter(|&c| sizes[c] >= 12).collect();
            chambers.push(big.len() as f64);
            let biggest = (0..sizes.len()).max_by_key(|&c| (sizes[c], std::cmp::Reverse(c)));
            largest.push(biggest.map_or(0, |c| sizes[c]) as f64);
            let points = world::points_of_interest(
                &grid,
                4,
                config.enemy_spawns,
                config.spawn_clearance,
                config.nav_large,
            );
            placed.push(points.zombie_spawns.len() as f64);
            if (points.zombie_spawns.len() as u32) < config.enemy_spawns {
                short.push(seed);
            }
            // Chambre d'un point : celle de la première case de chambre atteinte par le sol (BFS).
            let chamber_of = |start: (u32, u32)| -> Option<usize> {
                let mut seen = vec![false; (w * h) as usize];
                let mut queue = VecDeque::from([start]);
                seen[(start.1 * w + start.0) as usize] = true;
                while let Some((cx, cy)) = queue.pop_front() {
                    let label = chamber[(cy * w + cx) as usize];
                    if label != usize::MAX {
                        return Some(label);
                    }
                    for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, ny) = (cx as i32 + dx, cy as i32 + dy);
                        if is_floor(nx, ny) && !seen[(ny as u32 * w + nx as u32) as usize] {
                            seen[(ny as u32 * w + nx as u32) as usize] = true;
                            queue.push_back((nx as u32, ny as u32));
                        }
                    }
                }
                None
            };
            let labels: Vec<Option<usize>> = points.zombie_spawns.iter().map(|&p| chamber_of(p)).collect();
            let mut distinct: Vec<usize> = labels.iter().flatten().copied().collect();
            distinct.sort_unstable();
            distinct.dedup();
            occupied.push(distinct.len() as f64);
            in_largest.push(labels.iter().filter(|l| **l == biggest && l.is_some()).count() as f64);
            let mut pairs = 0;
            for a in 0..labels.len() {
                for b in a + 1..labels.len() {
                    if labels[a].is_some() && labels[a] == labels[b] {
                        pairs += 1;
                    }
                }
            }
            same_pairs.push(pairs as f64);
            if let Some(&origin) = points.player_spawns.first() {
                let dist = world::cave::floor_distances(&grid, origin);
                let d: Vec<f64> = points
                    .zombie_spawns
                    .iter()
                    .map(|&(x, y)| dist[(y * w + x) as usize] as f64)
                    .collect();
                if !d.is_empty() {
                    dist_min.push(d.iter().cloned().fold(f64::INFINITY, f64::min));
                    dist_max.push(d.iter().cloned().fold(0.0, f64::max));
                }
            }
        }
        println!(
            "caverne {id} ({}×{}, fill_ratio {}, iterations {}, spawns {}, dégagement {}, gabarit {}) sur {} graines",
            config.width,
            config.height,
            config.fill_ratio,
            config.iterations,
            config.enemy_spawns,
            config.spawn_clearance,
            if config.nav_large { "grand" } else { "petit" },
            floor.len()
        );
        stats("ratio de sol", floor);
        stats("chambres (5×5, ≥ 12 cases)", chambers);
        stats("plus grande chambre (cases)", largest);
        stats("cases de couloir (sol hors is_open)", corridors);
        stats("impasses (sol à ≤ 1 voisin)", dead_ends);
        stats("ZombieSpawn placés", placed);
        stats("chambres distinctes occupées par les spawns", occupied);
        stats("spawns dans la plus grande chambre", in_largest);
        stats("paires de spawns dans la même chambre", same_pairs);
        stats("distance joueur → spawn le plus proche", dist_min);
        stats("distance joueur → spawn le plus loin", dist_max);
        println!("  graines à spawns manquants : {short:?}");
    }
}
