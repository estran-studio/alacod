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
        for (file, bounds) in [("etage", (8, 12)), ("grand", (15, 20)), ("court", (3, 3))] {
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
                let expected = if room.template == "CombatEW" || room.template == "CombatNS" {
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

    fn activation_field(name: &str, value: i32) -> bevy_ecs_ldtk::ldtk::FieldInstance {
        bevy_ecs_ldtk::ldtk::FieldInstance {
            identifier: name.into(),
            def_uid: 0,
            field_instance_type: "Int".into(),
            value: bevy_ecs_ldtk::ldtk::FieldValue::Int(Some(value)),
            tile: None,
            real_editor_values: vec![],
        }
    }

    fn set_activation_fields(data: &mut LdtkJson, fields: &[(&str, i32)]) {
        let mut count = 0;
        for level in &mut data.levels {
            for layer in level.layer_instances.iter_mut().flatten() {
                for entity in &mut layer.entity_instances {
                    if entity.identifier == crate::map_const::ENTITY_ZOMBIE_SPAWN_LOCATION {
                        entity.field_instances = fields
                            .iter()
                            .map(|(name, value)| activation_field(name, *value))
                            .collect();
                        count += 1;
                    }
                }
            }
        }
        assert!(count > 0, "fixture without ZombieSpawn");
    }

    fn zombie_library() -> LdtkJson {
        load_ldtk_json_file(get_crate_root_path!(
            "../../games/zombies/assets/maps/avant_poste.ldtk"
        ))
        .unwrap()
    }

    /// Both assemblers preserve the optional bounds without changing any generated position,
    /// UUID or other entity field (and therefore without consuming additional RNG).
    #[test]
    fn activation_bounds_survive_basic_and_floor_assembly() {
        use map::generation::config::MapGenerationMode;
        let zombies = zombie_library();
        let mut floors = load_ldtk_json_file(get_crate_root_path!(
            "../../games/gungeon/assets/maps/salles.ldtk"
        ))
        .unwrap();
        let identifier = crate::map_const::ENTITY_ZOMBIE_SPAWN_LOCATION;
        let definition = zombies
            .defs
            .entities
            .iter()
            .find(|entity| entity.identifier == identifier)
            .unwrap()
            .clone();
        let spawn = zombies
            .levels
            .iter()
            .flat_map(|level| level.layer_instances.iter().flatten())
            .flat_map(|layer| &layer.entity_instances)
            .find(|entity| entity.identifier == identifier)
            .unwrap()
            .clone();
        floors.defs.entities.push(definition);
        for level in &mut floors.levels {
            level
                .layer_instances
                .as_mut()
                .unwrap()
                .iter_mut()
                .find(|layer| layer.identifier == crate::map_const::LAYER_ENTITY)
                .unwrap()
                .entity_instances
                .push(spawn.clone());
        }
        #[derive(serde::Deserialize)]
        struct GrammarFile {
            grammar: world::FloorGrammar,
        }
        let grammar = ron::from_str::<GrammarFile>(
            &std::fs::read_to_string(get_crate_root_path!(
                "../../games/gungeon/assets/floor_grammars/etage.ron"
            ))
            .unwrap(),
        )
        .unwrap()
        .grammar;
        for (mut source, mode) in [
            (zombies, MapGenerationMode::Basic),
            (floors, MapGenerationMode::Floor(grammar)),
        ] {
            set_activation_fields(&mut source, &[]);
            let config = MapGenerationConfig {
                seed: 1,
                max_width: 1000,
                max_heigth: 1000,
                mode,
                ..Default::default()
            };
            let assemble = |source: LdtkJson| {
                let context = from_map(&source, config.clone());
                let mut output = GeneratedMap::create(source);
                map_generation(context, &mut output).unwrap();
                output.get_generated_map()
            };
            let baseline = assemble(source.clone());
            set_activation_fields(
                &mut source,
                &[
                    ("active_x", -17),
                    ("active_y", 29),
                    ("active_width", 333),
                    ("active_height", 177),
                ],
            );
            let mut bounded = assemble(source);
            let mut count = 0;
            for level in &mut bounded.levels {
                for layer in level.layer_instances.iter_mut().flatten() {
                    for entity in &mut layer.entity_instances {
                        if entity.identifier != identifier {
                            continue;
                        }
                        use bevy_ecs_ldtk::prelude::LdtkFields;
                        assert_eq!(
                            *entity.get_int_field("active_x").unwrap(),
                            level.world_x - 17
                        );
                        assert_eq!(
                            *entity.get_int_field("active_y").unwrap(),
                            level.world_y + 29
                        );
                        assert_eq!(*entity.get_int_field("active_width").unwrap(), 333);
                        assert_eq!(*entity.get_int_field("active_height").unwrap(), 177);
                        #[cfg(feature = "runtime")]
                        {
                            use bevy_fixed::fixed_math::{Fixed, FixedVec2};
                            let component =
                                crate::game::entity::enemy_spawn::enemy_spawner_component_from_field(entity);
                            let min = FixedVec2::new(
                                Fixed::from_num(level.world_x - 17),
                                Fixed::from_num(-level.world_y - 29 - 177),
                            );
                            assert!(component.defends(min), "lower bounds must be inclusive");
                            assert!(component.defends(
                                min + FixedVec2::new(Fixed::from_num(332), Fixed::from_num(176),)
                            ));
                            assert!(
                                !component.defends(min - FixedVec2::new(Fixed::ONE, Fixed::ZERO))
                            );
                            assert!(!component
                                .defends(min + FixedVec2::new(Fixed::from_num(333), Fixed::ZERO)));
                            assert!(!component
                                .defends(min + FixedVec2::new(Fixed::ZERO, Fixed::from_num(177))));
                        }
                        entity.field_instances.clear();
                        count += 1;
                    }
                }
            }
            assert!(count > 0);
            assert_eq!(
                serde_json::to_value(bounded).unwrap(),
                serde_json::to_value(baseline).unwrap(),
                "activation bounds changed legacy generation beyond the four fields"
            );
        }
    }

    #[test]
    #[should_panic(expected = "activation bounds require all four integer fields")]
    fn partial_activation_bounds_are_rejected() {
        let mut source = zombie_library();
        set_activation_fields(&mut source, &[("active_x", 0)]);
        from_map(&source, MapGenerationConfig::default());
    }

    #[test]
    #[should_panic(expected = "active_width and active_height must be positive")]
    fn negative_activation_width_is_rejected() {
        let mut source = zombie_library();
        set_activation_fields(
            &mut source,
            &[
                ("active_x", 0),
                ("active_y", 0),
                ("active_width", -1),
                ("active_height", 128),
            ],
        );
        from_map(&source, MapGenerationConfig::default());
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
