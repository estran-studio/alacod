//! Opt-in assembly of authored LDtk rooms into one building and a connected exterior.
//! Construction runs during asset loading, before rollback. No M2 room-state contracts.
use std::collections::{BTreeSet, VecDeque};

use bevy_ecs_ldtk::ldtk::LdtkJson;
use bevy_fixed::rng::RollbackRng;
use serde_json::{json, Value};

const ROLE: &str = "building_role";
const MARGIN: i32 = 6;
const ROOM_W: i32 = 24;
const ROOM_H: i32 = 20;
const MAP_W: i32 = 82;
const MAP_H: i32 = 70;
type Cell = (i32, i32);

#[derive(Debug, Clone)]
pub struct BuildingRoom {
    pub cell: Cell,
    pub template: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct BuildingPlan {
    pub rooms: Vec<BuildingRoom>,
    pub doors: Vec<(usize, usize)>,
    pub radio: usize,
}

pub fn is_building_template(template: &LdtkJson) -> bool {
    template
        .levels
        .iter()
        .any(|l| l.field_instances.iter().any(|f| f.identifier == ROLE))
}

fn adjacent(a: Cell, b: Cell) -> bool {
    (a.0 - b.0).abs() + (a.1 - b.1).abs() == 1
}

fn pick(rng: &mut RollbackRng, count: usize) -> usize {
    // A private construction stream: changing it does not change the engine RNG.
    (rng.next_u32() as usize) % count
}

fn outside_connected(cells: &BTreeSet<Cell>) -> bool {
    let mut seen = BTreeSet::from([(-1, -1)]);
    let mut queue = VecDeque::from([(-1, -1)]);
    while let Some((x, y)) = queue.pop_front() {
        for next in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if (-1..=3).contains(&next.0)
                && (-1..=3).contains(&next.1)
                && !cells.contains(&next)
                && seen.insert(next)
            {
                queue.push_back(next);
            }
        }
    }
    (0..3).all(|x| (0..3).all(|y| cells.contains(&(x, y)) || seen.contains(&(x, y))))
}

pub fn door_distances(plan: &BuildingPlan) -> Vec<usize> {
    let mut distances = vec![usize::MAX; plan.rooms.len()];
    distances[0] = 0;
    let mut queue = VecDeque::from([0]);
    while let Some(a) = queue.pop_front() {
        for &(u, v) in &plan.doors {
            let b = if u == a {
                v
            } else if v == a {
                u
            } else {
                continue;
            };
            if distances[b] == usize::MAX {
                distances[b] = distances[a] + 1;
                queue.push_back(b);
            }
        }
    }
    distances
}

fn module_role(level: &Value) -> Option<&str> {
    level["fieldInstances"]
        .as_array()?
        .iter()
        .find(|f| f["__identifier"] == ROLE)?["__value"]
        .as_str()
}

pub fn building_plan(template: &LdtkJson, seed: i32) -> Result<BuildingPlan, String> {
    let data = serde_json::to_value(template).map_err(|e| e.to_string())?;
    let levels = data["levels"]
        .as_array()
        .ok_or("building without modules")?;
    if levels
        .iter()
        .any(|l| l["pxWid"] != 384 || l["pxHei"] != 320)
    {
        return Err("building modules must be 24 × 20 cells of 16 px".into());
    }
    let group = |role: &str| -> Vec<usize> {
        levels
            .iter()
            .enumerate()
            .filter(|(_, l)| module_role(l) == Some(role))
            .map(|(i, _)| i)
            .collect()
    };
    let starts = group("Accueil");
    let radios = group("Radio");
    let workshops = group("Atelier");
    if starts.is_empty() || radios.is_empty() || workshops.is_empty() {
        return Err("building needs Accueil, Radio and Atelier modules".into());
    }
    let mut pool: Vec<usize> = levels
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            matches!(
                module_role(l),
                Some("Atelier" | "Infirmerie" | "Reserve" | "Passage")
            )
        })
        .map(|(i, _)| i)
        .collect();
    if pool.len() < 6 {
        return Err("building needs at least six optional modules".into());
    }
    let mut rng = RollbackRng::new(seed as u32);
    let count = 5 + pick(&mut rng, 4);
    // Two distinct expansion choices at the start, and an always-exposed western facade.
    let root = (0, 1);
    let mut cells = BTreeSet::from([root, (0, 0), (1, 1)]);
    while cells.len() < count {
        let candidates: Vec<Cell> = (0..3)
            .flat_map(|x| (0..3).map(move |y| (x, y)))
            .filter(|c| !cells.contains(c) && cells.iter().any(|a| adjacent(*a, *c)))
            .filter(|c| {
                let mut next = cells.clone();
                next.insert(*c);
                outside_connected(&next)
            })
            .collect();
        let c = candidates
            .get(pick(&mut rng, candidates.len()))
            .ok_or("no valid building placement")?;
        cells.insert(*c);
    }
    let locations: Vec<Cell> = std::iter::once(root)
        .chain(cells.iter().copied().filter(|c| *c != root))
        .collect();
    let index = |c| locations.iter().position(|a| *a == c).unwrap();
    let mut doors = vec![(0, index((0, 0))), (0, index((1, 1)))];
    let mut visited = BTreeSet::from([0, index((0, 0)), index((1, 1))]);
    while visited.len() < count {
        let candidates: Vec<_> = visited
            .iter()
            .flat_map(|&a| (0..count).map(move |b| (a, b)))
            .filter(|(a, b)| !visited.contains(b) && adjacent(locations[*a], locations[*b]))
            .collect();
        let (a, b) = candidates[pick(&mut rng, candidates.len())];
        doors.push((a.min(b), a.max(b)));
        visited.insert(b);
    }
    let mut shortcuts: Vec<_> = (0..count)
        .flat_map(|a| ((a + 1)..count).map(move |b| (a, b)))
        .filter(|&(a, b)| adjacent(locations[a], locations[b]) && !doors.contains(&(a, b)))
        .collect();
    let extra = pick(&mut rng, 3).min(shortcuts.len());
    for _ in 0..extra {
        let i = pick(&mut rng, shortcuts.len());
        doors.push(shortcuts.remove(i));
    }
    doors.sort_unstable();
    let mut plan = BuildingPlan {
        rooms: locations
            .iter()
            .map(|&cell| BuildingRoom {
                cell,
                template: String::new(),
                role: String::new(),
            })
            .collect(),
        doors,
        radio: 0,
    };
    let distances = door_distances(&plan);
    let eligible: Vec<_> = (1..count)
        .filter(|&i| (2..=4).contains(&distances[i]))
        .collect();
    if eligible.is_empty() {
        return Err("radio must be 2–4 doors from start".into());
    }
    plan.radio = eligible[pick(&mut rng, eligible.len())];
    let early: Vec<_> = (1..count)
        .filter(|&i| i != plan.radio && distances[i] <= 2)
        .collect();
    let workshop_room = early[pick(&mut rng, early.len())];
    let workshop_module = workshops[pick(&mut rng, workshops.len())];
    pool.retain(|&m| m != workshop_module);
    for i in 0..count {
        let module = if i == 0 {
            starts[pick(&mut rng, starts.len())]
        } else if i == plan.radio {
            radios[pick(&mut rng, radios.len())]
        } else if i == workshop_room {
            workshop_module
        } else {
            pool[pick(&mut rng, pool.len())]
        };
        pool.retain(|&m| m != module);
        plan.rooms[i].template = levels[module]["identifier"].as_str().unwrap().into();
        plan.rooms[i].role = module_role(&levels[module]).unwrap().into();
    }
    Ok(plan)
}

fn make_entity(
    data: &Value,
    kind: &str,
    x: i32,
    y: i32,
    fields: Vec<(&str, Value)>,
    iid: String,
) -> Value {
    let def = data["defs"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["identifier"] == kind)
        .unwrap();
    let instances: Vec<_> = fields
        .into_iter()
        .map(|(name, value)| {
            let field = def["fieldDefs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["identifier"] == name)
                .unwrap();
            json!({"__identifier": name, "__type":field["__type"], "__value":value,
               "__tile":null,"defUid":field["uid"],"realEditorValues":[]})
        })
        .collect();
    json!({"__identifier":kind,"__grid":[x,y],"__pivot":[0,0],"__tags":[],
           "__tile":def["tileRect"],"__smartColor":def["color"],"iid":iid,
           "width":def["width"],"height":def["height"],"defUid":def["uid"],
           "px":[x*16,y*16],"fieldInstances":instances,"__worldX":x*16,"__worldY":y*16})
}

pub fn build_building_ldtk(template: &LdtkJson, seed: i32) -> Result<LdtkJson, String> {
    let plan = building_plan(template, seed)?;
    let mut data = serde_json::to_value(template).map_err(|e| e.to_string())?;
    let modules = data["levels"].as_array().unwrap().clone();
    let (w, h) = (MAP_W, MAP_H);
    let mut walls = vec![0_i32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                walls[(y * w + x) as usize] = 1;
            }
        }
    }
    let mut entities = Vec::new();
    let origin = |cell: Cell| {
        (
            MARGIN + cell.0 * (ROOM_W - 1),
            MARGIN + cell.1 * (ROOM_H - 1),
        )
    };
    for (i, room) in plan.rooms.iter().enumerate() {
        let module = modules
            .iter()
            .find(|l| l["identifier"] == room.template)
            .unwrap();
        let (ox, oy) = origin(room.cell);
        let layers = module["layerInstances"]
            .as_array()
            .ok_or("external building modules are unsupported")?;
        let grid = layers
            .iter()
            .find(|l| l["__identifier"] == "Walls")
            .ok_or("module without Walls")?["intGridCsv"]
            .as_array()
            .ok_or("module without wall grid")?;
        if grid.len() != 480 {
            return Err("invalid building module wall grid".into());
        }
        for y in 0..20 {
            for x in 0..24 {
                if grid[y * 24 + x] == 1 {
                    walls[((oy + y as i32) * w + ox + x as i32) as usize] = 1;
                }
            }
        }
        for e in layers
            .iter()
            .flat_map(|l| l["entityInstances"].as_array().unwrap())
            .filter(|e| {
                matches!(
                    e["__identifier"].as_str(),
                    Some("WeaponLocation" | "SodaLocation")
                ) || (i == 0 && e["__identifier"] == "PlayerSpawn")
            })
        {
            let mut e = e.clone();
            let x = ox + e["__grid"][0].as_i64().unwrap() as i32;
            let y = oy + e["__grid"][1].as_i64().unwrap() as i32;
            e["__grid"] = json!([x, y]);
            e["px"] = json!([x * 16, y * 16]);
            e["__worldX"] = json!(x * 16);
            e["__worldY"] = json!(y * 16);
            e["iid"] = json!(format!("building-{seed}-item-{}", entities.len()));
            entities.push(e);
        }
    }
    let mut rng = RollbackRng::new((seed as u32) ^ 0x6275_696c);
    let mut opening = |kind: &str, x: i32, y: i32| {
        let e = make_entity(
            &data,
            kind,
            x,
            y,
            if kind.starts_with("Door") {
                vec![("price", json!(750)), ("electrify", json!(false))]
            } else {
                vec![]
            },
            format!("building-{seed}-opening-{}", entities.len()),
        );
        for yy in y..y + e["height"].as_i64().unwrap() as i32 / 16 {
            for xx in x..x + e["width"].as_i64().unwrap() as i32 / 16 {
                walls[(yy * w + xx) as usize] = 0;
            }
        }
        entities.push(e);
    };
    for &(a, b) in &plan.doors {
        let ca = plan.rooms[a].cell;
        let cb = plan.rooms[b].cell;
        if ca.0 != cb.0 {
            let (x, y) = origin((ca.0.min(cb.0), ca.1));
            opening("DoorVertical", x + 23, y + 9);
        } else {
            let (x, y) = origin((ca.0, ca.1.min(cb.1)));
            opening("DoorHorizontal", x + 10, y + 19);
        }
    }
    let occupied: BTreeSet<_> = plan.rooms.iter().map(|r| r.cell).collect();
    for (i, room) in plan.rooms.iter().enumerate() {
        let (x, y) = origin(room.cell);
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if occupied.contains(&(room.cell.0 + dx, room.cell.1 + dy)) {
                continue;
            }
            if i == 0 && dx == -1 {
                opening("WindowHorizontal", x, y + 3);
                opening("WindowHorizontal", x, y + 15);
                opening("DoorVertical", x, y + 9);
            } else if pick(&mut rng, 3) != 0 {
                let offset = if pick(&mut rng, 2) == 0 { 4 } else { 14 };
                if dx != 0 {
                    opening(
                        "WindowHorizontal",
                        x + if dx < 0 { 0 } else { 23 },
                        y + offset,
                    );
                } else {
                    opening(
                        "WindowVertical",
                        x + offset,
                        y + if dy < 0 { 0 } else { 19 },
                    );
                }
            }
        }
    }
    // Each exterior source belongs to a physical defence room, including enclosed rooms.
    // Choose a clear outside point beside the closest exposed facade.
    let facades: Vec<_> = plan
        .rooms
        .iter()
        .flat_map(|room| {
            let (x, y) = origin(room.cell);
            [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .into_iter()
                .filter(|&(dx, dy)| !occupied.contains(&(room.cell.0 + dx, room.cell.1 + dy)))
                .map(move |(dx, dy)| {
                    if dx != 0 {
                        (x + if dx < 0 { -3 } else { 26 }, y + 9)
                    } else {
                        (x + 10, y + if dy < 0 { -3 } else { 22 })
                    }
                })
        })
        .collect();
    for (i, room) in plan.rooms.iter().enumerate() {
        let (ox, oy) = origin(room.cell);
        let &(x, y) = facades
            .iter()
            .min_by_key(|&&(x, y)| (x - (ox + 12)).abs() + (y - (oy + 10)).abs())
            .unwrap();
        entities.push(make_entity(
            &data,
            "ZombieSpawn",
            x,
            y,
            vec![
                ("active_x", json!((ox + 1) * 16)),
                ("active_y", json!((oy + 1) * 16)),
                ("active_width", json!((ROOM_W - 2) * 16)),
                ("active_height", json!((ROOM_H - 2) * 16)),
            ],
            format!("building-{seed}-spawn-{i}"),
        ));
    }
    let mut level = modules[0].clone();
    level["identifier"] = json!(format!("Relais_{seed}"));
    level["iid"] = json!(format!("building-{seed}"));
    level["pxWid"] = json!(w * 16);
    level["pxHei"] = json!(h * 16);
    level["worldX"] = json!(0);
    level["worldY"] = json!(0);
    level["__neighbours"] = json!([]);
    // Generated levels are ordinary gameplay maps, never recursively assembled modules.
    level["fieldInstances"]
        .as_array_mut()
        .unwrap()
        .retain(|f| f["__identifier"] != ROLE);
    data["defs"]["levelFields"]
        .as_array_mut()
        .unwrap()
        .retain(|f| f["identifier"] != ROLE);
    for layer in level["layerInstances"].as_array_mut().unwrap() {
        layer["__cWid"] = json!(w);
        layer["__cHei"] = json!(h);
        layer["iid"] = json!(format!(
            "building-{seed}-{}",
            layer["__identifier"].as_str().unwrap()
        ));
        layer["gridTiles"] = json!([]);
        layer["autoLayerTiles"] = json!([]);
        layer["entityInstances"] = json!([]);
        if layer["__identifier"] == "Entities" {
            layer["entityInstances"] = json!(entities);
        } else if layer["__identifier"] == "Walls" {
            layer["intGridCsv"] = json!(walls);
            let tiles: Vec<_> = (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let (sx, sy) = if walls[(y * w + x) as usize] == 1 {
                        (256, 96)
                    } else {
                        (320, 272)
                    };
                    json!({"px":[x*16,y*16],"src":[sx,sy],"f":0,"t":sy/16*23+sx/16,"d":[5,0],"a":1})
                })
                .collect();
            layer["autoLayerTiles"] = json!(tiles);
        } else {
            layer["intGridCsv"] = json!(vec![0; (w * h) as usize]);
        }
    }
    data["levels"] = json!([level]);
    serde_json::from_value(data).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::file::load_ldtk_json_file;
    use utils::get_crate_root_path;

    fn template() -> LdtkJson {
        load_ldtk_json_file(get_crate_root_path!(
            "../../games/zombies/assets/maps/le_relais_modules.ldtk"
        ))
        .unwrap()
    }

    /// A role-labelled spatial graph, canonical under translation, reflection and rotation.
    fn signature(plan: &BuildingPlan) -> String {
        let mut signatures = Vec::new();
        for swap in [false, true] {
            for fx in [false, true] {
                for fy in [false, true] {
                    let transformed: Vec<_> = plan
                        .rooms
                        .iter()
                        .enumerate()
                        .map(|(i, r)| {
                            let (mut x, mut y) = if swap { (r.cell.1, r.cell.0) } else { r.cell };
                            if fx {
                                x = -x;
                            }
                            if fy {
                                y = -y;
                            }
                            (x, y, i)
                        })
                        .collect();
                    let min_x = transformed.iter().map(|r| r.0).min().unwrap();
                    let min_y = transformed.iter().map(|r| r.1).min().unwrap();
                    let mut sorted: Vec<_> = transformed
                        .iter()
                        .map(|&(x, y, i)| (x - min_x, y - min_y, i))
                        .collect();
                    sorted.sort_unstable();
                    let labels: Vec<_> = sorted
                        .iter()
                        .map(|&(x, y, i)| format!("{x},{y}:{}", plan.rooms[i].role))
                        .collect();
                    let index = |i| sorted.iter().position(|r| r.2 == i).unwrap();
                    let mut edges: Vec<_> = plan
                        .doors
                        .iter()
                        .map(|&(a, b)| {
                            let (a, b) = (index(a), index(b));
                            (a.min(b), a.max(b))
                        })
                        .collect();
                    edges.sort_unstable();
                    signatures.push(format!("{labels:?}{edges:?}"));
                }
            }
        }
        signatures.into_iter().min().unwrap()
    }

    #[test]
    fn two_hundred_connected_reproducible_varied_buildings() {
        let t = template();
        let mut signatures = BTreeSet::new();
        let mut sizes = BTreeSet::new();
        let mut cycles = BTreeSet::new();
        for seed in 1..=200 {
            let p = building_plan(&t, seed).unwrap();
            assert!((5..=8).contains(&p.rooms.len()));
            let d = door_distances(&p);
            assert!(d.iter().all(|&n| n != usize::MAX));
            assert!((2..=4).contains(&d[p.radio]));
            assert!(p.doors.iter().filter(|&&(a, b)| a == 0 || b == 0).count() >= 2);
            let cells = p.rooms.iter().map(|r| r.cell).collect();
            assert!(outside_connected(&cells));
            let modules: BTreeSet<_> = p.rooms.iter().map(|r| &r.template).collect();
            assert_eq!(
                modules.len(),
                p.rooms.len(),
                "duplicate module, seed {seed}"
            );
            assert_eq!(p.rooms.iter().filter(|r| r.role == "Radio").count(), 1);
            assert_eq!(p.rooms.iter().filter(|r| r.role == "Accueil").count(), 1);
            sizes.insert(p.rooms.len());
            cycles.insert(p.doors.len() + 1 - p.rooms.len());
            let s = signature(&p);
            assert_eq!(s, signature(&building_plan(&t, seed).unwrap()));
            signatures.insert(s);
        }
        println!(
            "200 seeds: {} functional plans, room counts {sizes:?}, cycles {cycles:?}",
            signatures.len()
        );
        assert!(
            signatures.len() >= 80,
            "insufficient functional diversity: {}",
            signatures.len()
        );
        assert_eq!(sizes.len(), 4);
        assert!(cycles.len() >= 2);
        let a = serde_json::to_value(build_building_ldtk(&t, 17).unwrap()).unwrap();
        let b = serde_json::to_value(build_building_ldtk(&t, 17).unwrap()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn rejects_missing_roles_and_wrong_module_dimensions() {
        let mut t = template();
        t.levels[0].px_wid = 208;
        assert!(building_plan(&t, 1).unwrap_err().contains("24 × 20"));
        let mut t = template();
        for level in &mut t.levels {
            level
                .field_instances
                .retain(|field| field.identifier != ROLE);
        }
        assert!(building_plan(&t, 1).unwrap_err().contains("Accueil"));
    }

    #[test]
    fn all_exterior_spawns_reach_start_with_closed_doors() {
        let t = template();
        // Every distinct seed is built, and geometry is sampled across all room counts.
        for seed in 1..=200 {
            let data = serde_json::to_value(build_building_ldtk(&t, seed).unwrap()).unwrap();
            let l = &data["levels"][0];
            let layers = l["layerInstances"].as_array().unwrap();
            let grid = layers
                .iter()
                .find(|a| a["__identifier"] == "Walls")
                .unwrap()["intGridCsv"]
                .as_array()
                .unwrap();
            let es = layers
                .iter()
                .find(|a| a["__identifier"] == "Entities")
                .unwrap()["entityInstances"]
                .as_array()
                .unwrap();
            for entity in es.iter().filter(|e| e["__identifier"] == "ZombieSpawn") {
                let instance = serde_json::from_value(entity.clone()).unwrap();
                let config =
                    crate::game::entity::enemy_spawn::enemy_spawner_component_from_field(&instance);
                let (min, size) = config
                    .activation_area
                    .expect("generated room binding must be loaded");
                assert!(size.x > bevy_fixed::fixed_math::Fixed::ZERO);
                assert!(config.defends(min + size / bevy_fixed::fixed_math::Fixed::from_num(2)));
            }
            let purchases: Vec<_> = es
                .iter()
                .filter(|e| {
                    matches!(
                        e["__identifier"].as_str(),
                        Some("WeaponLocation" | "SodaLocation")
                    )
                })
                .collect();
            for (i, item) in purchases.iter().enumerate() {
                let x = item["__grid"][0].as_i64().unwrap() as i32;
                let y = item["__grid"][1].as_i64().unwrap() as i32;
                // Reserve 48 × 48 px for the station, rather than only its LDtk point.
                for yy in y - 1..=y + 1 {
                    for xx in x - 1..=x + 1 {
                        assert_ne!(
                            grid[(yy * MAP_W + xx) as usize],
                            1,
                            "station overlaps wall, seed {seed}"
                        );
                    }
                }
                for other in purchases.iter().skip(i + 1) {
                    let dx = x - other["__grid"][0].as_i64().unwrap() as i32;
                    let dy = y - other["__grid"][1].as_i64().unwrap() as i32;
                    assert!(
                        dx.abs() >= 4 || dy.abs() >= 4,
                        "overlapping stations, seed {seed}"
                    );
                }
                for door in es
                    .iter()
                    .filter(|e| e["__identifier"].as_str().unwrap().starts_with("Door"))
                {
                    let dx = door["__grid"][0].as_i64().unwrap() as i32;
                    let dy = door["__grid"][1].as_i64().unwrap() as i32;
                    let dw = door["width"].as_i64().unwrap() as i32 / 16;
                    let dh = door["height"].as_i64().unwrap() as i32 / 16;
                    assert!(
                        x + 2 <= dx || x - 1 >= dx + dw || y + 2 <= dy || y - 1 >= dy + dh,
                        "station blocks door, seed {seed}"
                    );
                }
            }
            for ground_breaker in [true, false] {
                let mut blocked: BTreeSet<Cell> = grid
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| **v == 1)
                    .map(|(i, _)| ((i % MAP_W as usize) as i32, (i / MAP_W as usize) as i32))
                    .collect();
                for e in es.iter().filter(|e| {
                    e["__identifier"]
                        .as_str()
                        .unwrap()
                        .starts_with(if ground_breaker { "Door" } else { "Window" })
                }) {
                    let x = e["__grid"][0].as_i64().unwrap() as i32;
                    let y = e["__grid"][1].as_i64().unwrap() as i32;
                    for xx in x..x + e["width"].as_i64().unwrap() as i32 / 16 {
                        for yy in y..y + e["height"].as_i64().unwrap() as i32 / 16 {
                            blocked.insert((xx, yy));
                        }
                    }
                }
                // Expand walls by the body's 10 px half-extent on an 8 px navigation grid.
                let mut forbidden = BTreeSet::new();
                for (x, y) in blocked {
                    for a in x * 2 - 1..=x * 2 + 3 {
                        for b in y * 2 - 1..=y * 2 + 3 {
                            forbidden.insert((a, b));
                        }
                    }
                }
                let player = es
                    .iter()
                    .find(|e| e["__identifier"] == "PlayerSpawn")
                    .unwrap();
                let start = (
                    player["px"][0].as_i64().unwrap() as i32 / 8 + 1,
                    player["px"][1].as_i64().unwrap() as i32 / 8 + 1,
                );
                assert!(!forbidden.contains(&start), "blocked start: seed {seed}");
                let mut seen = BTreeSet::from([start]);
                let mut queue = VecDeque::from([start]);
                while let Some((x, y)) = queue.pop_front() {
                    for c in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                        if (0..MAP_W * 2).contains(&c.0)
                            && (0..MAP_H * 2).contains(&c.1)
                            && !forbidden.contains(&c)
                            && seen.insert(c)
                        {
                            queue.push_back(c);
                        }
                    }
                }
                for e in es.iter().filter(|e| {
                    if ground_breaker {
                        e["__identifier"] == "ZombieSpawn"
                    } else {
                        matches!(
                            e["__identifier"].as_str(),
                            Some("PlayerSpawn" | "WeaponLocation" | "SodaLocation")
                        )
                    }
                }) {
                    let c = (
                        e["px"][0].as_i64().unwrap() as i32 / 8 + 1,
                        e["px"][1].as_i64().unwrap() as i32 / 8 + 1,
                    );
                    assert!(
                    seen.contains(&c),
                    "unreachable entity, seed {seed}, ground_breaker {ground_breaker}, cell {c:?}"
                );
                }
                if !ground_breaker {
                    for room in building_plan(&t, seed).unwrap().rooms {
                        let c = (
                            (MARGIN + room.cell.0 * (ROOM_W - 1) + 3) * 2 + 1,
                            (MARGIN + room.cell.1 * (ROOM_H - 1) + 3) * 2 + 1,
                        );
                        assert!(
                            seen.contains(&c),
                            "unreachable room, seed {seed}, {}",
                            room.template
                        );
                    }
                }
            }
        }
    }
}
