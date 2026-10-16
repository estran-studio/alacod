pub mod building;
pub mod cave;
mod from;
mod to;

pub use from::from_map;
pub use to::{get_new_entity, GeneratedMap, GeneratedRoom};

#[cfg(test)]
mod tests {

    use bevy_ecs_ldtk::ldtk::LdtkJson;
    use bevy_fixed::rng::RollbackRng;
    use utils::get_crate_root_path;

    use map::generation::{config::MapGenerationConfig, map_generation};

    use crate::loader::file::load_ldtk_json_file;

    use self::to::GeneratedMap;

    use super::*;

    #[test]
    fn test_load_map() {
        let generated_map = get_context();

        assert_eq!(
            "ab6b0200-b0a0-11ee-bca1-dd7a22611e78",
            generated_map.ldtk_json.iid
        );
    }

    #[test]
    fn test_generated_level() {
        let generated_map = get_context();

        let _first_level = generated_map.generated_rooms.first().unwrap();
    }

    fn generate(path: &str, seed: i32) -> GeneratedMap {
        let data: LdtkJson = load_ldtk_json_file(path).expect("Failed to deserialize JSON");
        // Même config que la partie (`game::local::configure_map`).
        let config = MapGenerationConfig {
            seed,
            max_width: 1000,
            max_heigth: 1000,
            ..Default::default()
        };
        let context = from_map(&data, config);
        let mut generator = GeneratedMap::create(data);
        map_generation(context, &mut generator).expect("Failed to generate map");
        generator
    }

    /// D45/D46 : sur `avant_poste` (9 gabarits), les graines 1..20 donnent plusieurs cartes
    /// distinctes, et aucune salle n'en chevauche une autre. `ALACOD_MAP_SIGNATURES=1` imprime
    /// la liste des gabarits placés par graine (mesure du rapport m1-assembleur-d45-d47).
    #[test]
    fn avant_poste_cartes_distinctes_sans_chevauchement() {
        let path = get_crate_root_path!("../../games/zombies/assets/maps/avant_poste.ldtk");
        let mut signatures = std::collections::BTreeSet::new();
        let mut chevauchements = vec![];
        for seed in 1..=20 {
            let map = generate(&path, seed);
            let rooms = &map.generated_rooms;
            if std::env::var("ALACOD_MAP_SIGNATURES").is_ok_and(|v| v == "1") {
                let templates: Vec<&str> = rooms.iter().map(|r| r.template.as_str()).collect();
                println!("graine {seed} : {} salles {:?}", rooms.len(), templates);
            }
            for (i, a) in rooms.iter().enumerate() {
                for b in &rooms[i + 1..] {
                    let (ax, ay, aw, ah) = a.world_rect();
                    let (bx, by, bw, bh) = b.world_rect();
                    let disjoint = ax + aw <= bx || bx + bw <= ax || ay + ah <= by || by + bh <= ay;
                    if !disjoint {
                        chevauchements
                            .push(format!("graine {seed} : {} / {}", a.template, b.template));
                    }
                }
            }
            // La signature ignore la translation de la salle de départ (tirée par la graine
            // même avant D45) : seule compte la forme de la carte.
            let (ox, oy, _, _) = rooms[0].world_rect();
            let forme: Vec<String> = rooms
                .iter()
                .map(|r| {
                    let (x, y, _, _) = r.world_rect();
                    format!("{}@{},{}", r.template, x - ox, y - oy)
                })
                .collect();
            signatures.insert(forme);
        }
        if std::env::var("ALACOD_MAP_SIGNATURES").is_ok_and(|v| v == "1") {
            println!("cartes distinctes : {}", signatures.len());
            println!("chevauchements : {}", chevauchements.len());
        }
        assert!(
            chevauchements.is_empty(),
            "salles chevauchantes : {chevauchements:?}"
        );
        assert!(
            signatures.len() > 1,
            "les 20 graines produisent toutes la même carte"
        );
    }

    /// M2-T10 : sur les gabarits de `gungeon` (`maps/salles.ldtk`, quinze depuis M2-T14d), chaque graine
    /// donne un étage dans les bornes de sa grammaire (`floor_grammars/*.ron`), avec un seul boss en
    /// cul-de-sac, sans chevauchement, et dont les ouvertures libres sont murées.
    #[test]
    fn gungeon_etages_de_grammaire() {
        use map::generation::config::MapGenerationMode;
        use world::FloorGrammar;
        let path = get_crate_root_path!("../../games/gungeon/assets/maps/salles.ldtk");
        for (file, bounds) in [("etage", (10, 14)), ("grand", (15, 20)), ("court", (3, 3))] {
            let ron_path = get_crate_root_path!("../../games/gungeon/assets/floor_grammars/");
            let text = std::fs::read_to_string(format!("{ron_path}{file}.ron")).unwrap();
            #[derive(serde::Deserialize)]
            struct File {
                grammar: FloorGrammar,
            }
            let grammar = ron::from_str::<File>(&text).unwrap().grammar;
            assert_eq!(grammar.rooms, bounds);
            let started = std::time::Instant::now();
            let derived = |s: i32| s.wrapping_add(0x3C6E_F372);
            // `ALACOD_FLOOR_SEEDS=1000` : la passe longue de M2-T10 et M2-T7 (200 par défaut).
            let count: i32 = std::env::var("ALACOD_FLOOR_SEEDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200);
            let seeds: Vec<i32> = (1..=count).chain((1..=count).map(derived)).collect();
            for seed in seeds
                .into_iter()
                .take(if file == "court" { 400 } else { count as usize })
            {
                let data: LdtkJson = load_ldtk_json_file(&path).unwrap();
                let config = MapGenerationConfig {
                    seed,
                    max_width: 1000,
                    max_heigth: 1000,
                    mode: MapGenerationMode::Floor(grammar.clone()),
                    ..Default::default()
                };
                let context = from_map(&data, config);
                let mut generator = GeneratedMap::create(data);
                map_generation(context, &mut generator)
                    .unwrap_or_else(|_| panic!("{file} graine {seed} : génération échouée"));
                generator.close_unused_openings();
                let rooms = &generator.generated_rooms;
                if std::env::var("ALACOD_FLOOR_DUMP").is_ok_and(|v| v == file) {
                    let (ox, oy, _, _) = rooms[0].world_rect();
                    let list: Vec<String> = rooms
                        .iter()
                        .map(|r| {
                            let (x, y, _, _) = r.world_rect();
                            format!("{}@{},{}", r.template, (x - ox) / 128, (y - oy) / 128)
                        })
                        .collect();
                    println!("graine {seed} : {}", list.join(" "));
                }
                assert!(
                    (bounds.0..=bounds.1).contains(&rooms.len()),
                    "{file} graine {seed} : {} salles",
                    rooms.len()
                );
                let bosses: Vec<_> = rooms
                    .iter()
                    .filter(|r| r.template.starts_with("Boss"))
                    .collect();
                assert_eq!(
                    bosses.len(),
                    1,
                    "{file} graine {seed} : {} boss",
                    bosses.len()
                );
                // M2-T7 : une grammaire qui exige une arène en place exactement une.
                if let Some(wanted) = grammar.required.get("arene") {
                    let arenas = rooms
                        .iter()
                        .filter(|r| r.template.starts_with("Arena"))
                        .count();
                    assert_eq!(arenas, *wanted, "{file} graine {seed} : {arenas} arène(s)");
                }
                // M2-T14d : chaque type exigé par la grammaire est posé exactement autant de fois
                // (le gabarit porte le type en préfixe de son nom).
                for (kind, prefix) in [
                    ("escouade", "Escouade"),
                    ("coffre", "Coffre"),
                    ("secret", "Secret"),
                    ("boutique", "Boutique"),
                    ("recompense", "Recompense"),
                ] {
                    if let Some(wanted) = grammar.required.get(kind) {
                        let found = rooms
                            .iter()
                            .filter(|r| r.template.starts_with(prefix))
                            .count();
                        assert_eq!(found, *wanted, "{file} graine {seed} : {found} {kind}");
                    }
                }
                for (i, a) in rooms.iter().enumerate() {
                    for b in &rooms[i + 1..] {
                        let (ax, ay, aw, ah) = a.world_rect();
                        let (bx, by, bw, bh) = b.world_rect();
                        assert!(
                            ax + aw <= bx || bx + bw <= ax || ay + ah <= by || by + bh <= ay,
                            "{file} graine {seed} : {} chevauche {}",
                            a.template,
                            b.template
                        );
                    }
                }
            }
            println!("{file} : 200 graines en {:?}", started.elapsed());
        }
    }

    fn isaac_grammar(file: &str) -> world::FloorGrammar {
        #[derive(serde::Deserialize)]
        struct File {
            grammar: world::FloorGrammar,
        }
        let ron_path = get_crate_root_path!("../../games/isaac/assets/floor_grammars/");
        let text = std::fs::read_to_string(format!("{ron_path}{file}.ron")).unwrap();
        ron::from_str::<File>(&text).unwrap().grammar
    }

    /// Un étage `isaac` généré comme le fait le chargeur (grille : murage exact des ouvertures).
    fn isaac_floor(data: &LdtkJson, grammar: &world::FloorGrammar, seed: i32) -> GeneratedMap {
        use map::generation::config::MapGenerationMode;
        let config = MapGenerationConfig {
            seed,
            max_width: 1000,
            max_heigth: 1000,
            mode: MapGenerationMode::Floor(grammar.clone()),
            ..Default::default()
        };
        let context = from_map(data, config);
        let mut generator = GeneratedMap::create(data.clone());
        map_generation(context, &mut generator)
            .unwrap_or_else(|_| panic!("graine {seed} : génération échouée"));
        generator.close_unused_openings_exact();
        generator
    }

    /// M3-T5 (`docs/conventions.md` §70) : sur les neuf gabarits de `isaac` (`maps/salles.ldtk`,
    /// 1x1, 2x1, 1x2 et 2x2 cases de salle de 15 x 9 cases), 1 000 graines de chacune des deux
    /// grammaires en grille donnent un étage dans les bornes, avec un seul boss (1x1 ou 2x2), un
    /// trésor (1x1, 2x1 ou 1x2), une salle secrète, au plus une salle du diable accrochée au boss,
    /// sans chevauchement, alignées sur la grille, dont chaque ouverture est une porte ou est murée,
    /// et dont les trois exigences de jeu tiennent : la secrète touche au moins deux salles
    /// ordinaires atteignables sans la franchir (une bombe y accède), le trésor est un cul-de-sac
    /// derrière une salle ordinaire, le boss est le cul-de-sac le plus loin.
    #[test]
    fn isaac_etages_de_grille_sur_1000_graines() {
        use std::collections::{BTreeMap, BTreeSet, VecDeque};
        let path = get_crate_root_path!("../../games/isaac/assets/maps/salles.ldtk");
        let data: LdtkJson = load_ldtk_json_file(&path).unwrap();
        let count: i32 = std::env::var("ALACOD_FLOOR_SEEDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(500);
        let derived = |s: i32| s.wrapping_add(0x3C6E_F372);
        for (file, bounds) in [("sous_sol", (8, 12)), ("sous_sol_grand", (12, 15))] {
            let grammar = isaac_grammar(file);
            assert_eq!(grammar.rooms, bounds);
            let (mut with_post, mut with_loop, mut shapes) =
                (0, 0, BTreeMap::<String, usize>::new());
            for seed in (1..=count).chain((1..=count).map(derived)) {
                let generator = isaac_floor(&data, &grammar, seed);
                let rooms = &generator.generated_rooms;
                let label = format!("{file} graine {seed}");
                assert!(
                    (bounds.0..=bounds.1).contains(&rooms.len()),
                    "{label} : {} salles",
                    rooms.len()
                );
                let named = |prefix: &str| -> Vec<usize> {
                    (0..rooms.len())
                        .filter(|i| rooms[*i].template.starts_with(prefix))
                        .collect()
                };
                let (boss, tresor, secrete, diable) = (
                    named("Boss"),
                    named("Tresor"),
                    named("Secrete"),
                    named("Diable"),
                );
                assert_eq!(
                    (boss.len(), tresor.len(), secrete.len()),
                    (1, 1, 1),
                    "{label}"
                );
                assert!(diable.len() <= 1, "{label}");
                with_post += diable.len();
                for room in rooms {
                    let (_, _, w, h) = room.world_rect();
                    assert!(
                        w % 240 == 0 && h % 144 == 0,
                        "{label} : {} hors grille",
                        room.template
                    );
                }
                let (ox, oy, _, _) = rooms[0].world_rect();
                for room in rooms {
                    let (x, y, _, _) = room.world_rect();
                    assert!(
                        (x - ox) % 240 == 0 && (y - oy) % 144 == 0,
                        "{label} : {} décalée",
                        room.template
                    );
                }
                for (i, a) in rooms.iter().enumerate() {
                    for b in &rooms[i + 1..] {
                        let (ax, ay, aw, ah) = a.world_rect();
                        let (bx, by, bw, bh) = b.world_rect();
                        assert!(
                            ax + aw <= bx || bx + bw <= ax || ay + ah <= by || by + bh <= ay,
                            "{label} : {} chevauche {}",
                            a.template,
                            b.template
                        );
                    }
                }
                // graphe des portes, relu depuis la carte générée
                let index: BTreeMap<&str, usize> = rooms
                    .iter()
                    .enumerate()
                    .map(|(i, r)| (r.iid(), i))
                    .collect();
                let mut adjacent = vec![BTreeSet::new(); rooms.len()];
                for (i, room) in rooms.iter().enumerate() {
                    for other in room.neighbour_iids() {
                        let j = index[other.as_str()];
                        adjacent[i].insert(j);
                        adjacent[j].insert(i);
                    }
                }
                // chaque ouverture non murée d'une salle est une porte vers une voisine : trois
                // cases d'ouverture par porte
                for (i, room) in rooms.iter().enumerate() {
                    assert_eq!(
                        room.open_cells(),
                        3 * room.neighbour_iids().len(),
                        "{label} : {}",
                        room.template
                    );
                    let _ = i;
                }
                let secret = secrete[0];
                let ordinary =
                    |i: usize| rooms[i].template == "Depart" || rooms[i].template == "Combat";
                assert!(
                    adjacent[secret].len() >= 2,
                    "{label} : secrète à {} portes",
                    adjacent[secret].len()
                );
                assert!(
                    adjacent[secret].iter().all(|r| ordinary(*r)),
                    "{label} : secrète reliée à une salle spéciale"
                );
                // atteignables du départ sans franchir la secrète (une bombe y accède)
                let mut seen = vec![false; rooms.len()];
                seen[0] = true;
                let mut queue = VecDeque::from([0]);
                let mut depth = vec![0usize; rooms.len()];
                while let Some(room) = queue.pop_front() {
                    for next in &adjacent[room] {
                        if *next != secret && !seen[*next] {
                            seen[*next] = true;
                            depth[*next] = depth[room] + 1;
                            queue.push_back(*next);
                        }
                    }
                }
                assert!((0..rooms.len()).all(|i| i == secret || seen[i]), "{label}");
                assert!(adjacent[secret].iter().all(|r| seen[*r]), "{label}");
                // trésor : cul-de-sac derrière une salle ordinaire ; boss : cul-de-sac le plus loin
                assert_eq!(adjacent[tresor[0]].len(), 1, "{label}");
                assert!(adjacent[tresor[0]].iter().all(|r| ordinary(*r)), "{label}");
                let post_degree = diable.len();
                assert_eq!(adjacent[boss[0]].len(), 1 + post_degree, "{label}");
                for i in 0..rooms.len() {
                    if i != boss[0] && i != secret && !diable.contains(&i) {
                        assert!(
                            depth[i] < depth[boss[0]],
                            "{label} : {} aussi loin que le boss",
                            rooms[i].template
                        );
                    }
                }
                if let Some(post) = diable.first() {
                    assert!(adjacent[*post] == BTreeSet::from([boss[0]]), "{label}");
                }
                let edges: usize = adjacent.iter().map(|a| a.len()).sum::<usize>() / 2;
                let tree = rooms.len() - 1 + adjacent[secret].len() - 1;
                with_loop += usize::from(edges > tree);
                for i in boss.iter().chain(tresor.iter()) {
                    *shapes.entry(rooms[*i].template.clone()).or_default() += 1;
                }
            }
            println!(
                "isaac {file} : {} graines, salle du diable {with_post}, boucles {with_loop}, gabarits {shapes:?}",
                2 * count
            );
        }
    }

    /// M3-T5 : un étage de 12 à 15 salles en grille se génère en moins de deux fois le temps de
    /// `grand` de gungeon (15 à 20 salles en arbre), comparés graine à graine dans le même processus
    /// (planification, génération des niveaux LDtk et murage).
    #[test]
    fn isaac_grille_de_12_a_15_salles_sous_deux_fois_grand_de_gungeon() {
        use map::generation::config::MapGenerationMode;
        let isaac: LdtkJson = load_ldtk_json_file(&get_crate_root_path!(
            "../../games/isaac/assets/maps/salles.ldtk"
        ))
        .unwrap();
        let gungeon: LdtkJson = load_ldtk_json_file(&get_crate_root_path!(
            "../../games/gungeon/assets/maps/salles.ldtk"
        ))
        .unwrap();
        let grid = isaac_grammar("sous_sol_grand");
        #[derive(serde::Deserialize)]
        struct File {
            grammar: world::FloorGrammar,
        }
        let grand = ron::from_str::<File>(
            &std::fs::read_to_string(get_crate_root_path!(
                "../../games/gungeon/assets/floor_grammars/grand.ron"
            ))
            .unwrap(),
        )
        .unwrap()
        .grammar;
        let (mut grid_time, mut grand_time) =
            (std::time::Duration::ZERO, std::time::Duration::ZERO);
        let runs = 100;
        let timed = |work: &mut dyn FnMut()| {
            let started = std::time::Instant::now();
            work();
            started.elapsed()
        };
        for seed in 1..=runs {
            grid_time += timed(&mut || {
                let generator = isaac_floor(&isaac, &grid, seed);
                std::hint::black_box(generator.generated_rooms.len());
            });
            grand_time += timed(&mut || {
                let config = MapGenerationConfig {
                    seed,
                    max_width: 1000,
                    max_heigth: 1000,
                    mode: MapGenerationMode::Floor(grand.clone()),
                    ..Default::default()
                };
                let context = from_map(&gungeon, config);
                let mut generator = GeneratedMap::create(gungeon.clone());
                map_generation(context, &mut generator).unwrap();
                generator.close_unused_openings();
                std::hint::black_box(generator.generated_rooms.len());
            });
        }
        println!(
            "génération : isaac 12-15 salles en grille {:?} par étage, gungeon `grand` {:?} (rapport {:.2})",
            grid_time / runs as u32,
            grand_time / runs as u32,
            grid_time.as_secs_f64() / grand_time.as_secs_f64()
        );
        assert!(
            grid_time < grand_time * 2,
            "{grid_time:?} contre {grand_time:?}"
        );
    }

    /// M2-T11 (`docs/conventions.md` §46) : les gabarits `CombatEW` et `CombatNS` de `gungeon`
    /// portent deux barils et une table. Sur 1 000 graines de la grammaire `etage`, la génération
    /// réussit, chaque salle générée à partir de ces gabarits garde ses trois objets de décor
    /// (champ `prop` vide : défaut du type), à l'intérieur de la salle, sans recouvrir une porte,
    /// un point d'apparition de joueur ni un personnage, et la grammaire reste valide (un boss,
    /// pas de chevauchement).
    #[test]
    fn gungeon_decor_sur_1000_graines() {
        use bevy_ecs_ldtk::EntityInstance;
        use map::generation::config::MapGenerationMode;
        use world::FloorGrammar;
        let path = get_crate_root_path!("../../games/gungeon/assets/maps/salles.ldtk");
        let ron_path = get_crate_root_path!("../../games/gungeon/assets/floor_grammars/etage.ron");
        let text = std::fs::read_to_string(ron_path).unwrap();
        #[derive(serde::Deserialize)]
        struct File {
            grammar: FloorGrammar,
        }
        let grammar = ron::from_str::<File>(&text).unwrap().grammar;
        let overlaps = |a: &EntityInstance, b: &EntityInstance| {
            a.px.x < b.px.x + b.width
                && b.px.x < a.px.x + a.width
                && a.px.y < b.px.y + b.height
                && b.px.y < a.px.y + a.height
        };
        let (mut decorated_rooms, mut bosses) = (0usize, 0usize);
        for seed in 1..=1000 {
            let data: LdtkJson = load_ldtk_json_file(&path).unwrap();
            let config = MapGenerationConfig {
                seed,
                max_width: 1000,
                max_heigth: 1000,
                mode: MapGenerationMode::Floor(grammar.clone()),
                ..Default::default()
            };
            let context = from_map(&data, config);
            let mut generator = GeneratedMap::create(data);
            map_generation(context, &mut generator)
                .unwrap_or_else(|_| panic!("graine {seed} : génération échouée"));
            generator.close_unused_openings();
            let rooms = &generator.generated_rooms;
            assert!(
                (grammar.rooms.0..=grammar.rooms.1).contains(&rooms.len()),
                "graine {seed} : {} salles",
                rooms.len()
            );
            bosses += rooms
                .iter()
                .filter(|r| r.template.starts_with("Boss"))
                .count();
            for room in rooms {
                let entities: Vec<&EntityInstance> = room.entities().collect();
                let props: Vec<&&EntityInstance> = entities
                    .iter()
                    .filter(|e| e.identifier == "Table" || e.identifier == "Barrel")
                    .collect();
                let expected = if matches!(
                    room.template.as_str(),
                    "CombatEW" | "CombatNS" | "Coffre" | "Secret" | "ArenaNS"
                ) {
                    3
                } else {
                    0
                };
                assert_eq!(
                    props.len(),
                    expected,
                    "graine {seed} : {} porte {} objets de décor",
                    room.template,
                    props.len()
                );
                if expected > 0 {
                    decorated_rooms += 1;
                }
                let (rx, ry, rw, rh) = room.world_rect();
                for prop in &props {
                    let (px, py) = (prop.world_x.unwrap(), prop.world_y.unwrap());
                    assert!(
                        px >= rx
                            && py >= ry
                            && px + prop.width <= rx + rw
                            && py + prop.height <= ry + rh,
                        "graine {seed} : {} sort de {}",
                        prop.identifier,
                        room.template
                    );
                    for other in &entities {
                        if std::ptr::eq(**prop, *other) {
                            continue;
                        }
                        assert!(
                            !overlaps(prop, other),
                            "graine {seed} : {} recouvre {} dans {}",
                            prop.identifier,
                            other.identifier,
                            room.template
                        );
                    }
                }
            }
        }
        assert_eq!(bosses, 1000, "un boss par étage");
        assert!(decorated_rooms > 100, "salles décorées : {decorated_rooms}");
    }

    fn get_context() -> GeneratedMap {
        let seed = 1;
        let data: LdtkJson = load_ldtk_json_file(get_crate_root_path!(
            "../../games/zombies/assets/exemples/test_map.ldtk"
        ))
        .expect("Failed to deserialize JSON");

        let config = MapGenerationConfig {
            seed,
            ..Default::default()
        };
        let context = from_map(&data, config);
        let mut generator = GeneratedMap::create(data);

        map_generation(context, &mut generator).expect("Failed to generate map");

        let _data = generator.get_generated_map();

        generator
    }
}
