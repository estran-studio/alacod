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

    /// M2-T10 : sur les neuf gabarits de `gungeon` (`maps/salles.ldtk`), chaque graine donne un étage
    /// dans les bornes de sa grammaire (`floor_grammars/*.ron`), avec un seul boss en cul-de-sac,
    /// sans chevauchement, et dont les ouvertures libres sont murées.
    #[test]
    fn gungeon_etages_de_grammaire() {
        use map::generation::config::MapGenerationMode;
        use world::FloorGrammar;
        let path = get_crate_root_path!("../../games/gungeon/assets/maps/salles.ldtk");
        for (file, bounds) in [("etage", (8, 12)), ("grand", (15, 20))] {
            let ron_path = get_crate_root_path!("../../games/gungeon/assets/floor_grammars/");
            let text = std::fs::read_to_string(format!("{ron_path}{file}.ron")).unwrap();
            #[derive(serde::Deserialize)]
            struct File {
                grammar: FloorGrammar,
            }
            let grammar = ron::from_str::<File>(&text).unwrap().grammar;
            assert_eq!(grammar.rooms, bounds);
            let started = std::time::Instant::now();
            for seed in 1..=200 {
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
                let bosses: Vec<_> = rooms.iter().filter(|r| r.template.starts_with("Boss")).collect();
                assert_eq!(bosses.len(), 1, "{file} graine {seed} : {} boss", bosses.len());
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
