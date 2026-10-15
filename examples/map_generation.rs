use bevy_ecs_ldtk::ldtk::LdtkJson;
use map_ldtk::generation::{from_map, GeneratedMap};
use map_ldtk::loader::file::load_ldtk_json_file;
use serde_json::to_string_pretty;
use std::fs::File;
use std::io::Write;

use map::generation::{config::MapGenerationConfig, map_generation};

fn main() {
    let usage = "Usage: <base_map_path> <output_map_path> [seed]";

    // Get the file path from the command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args.len() > 4 {
        eprintln!("{}", usage);
        std::process::exit(1);
    }

    let map_path = &args[1];
    let map_output_path = &args[2];

    let default_seed = 1;

    // Parse the second argument as a number or use the default value
    let seed: i32 = args.get(3).map_or(default_seed, |arg| {
        arg.parse().unwrap_or_else(|_| {
            eprintln!(
                "Error parsing the seed argument. Using default value: {}",
                default_seed
            );
            default_seed
        })
    });

    println!("loading base map {}", map_path);

    // Open the file
    let data: LdtkJson = load_ldtk_json_file(map_path).expect("Failed to deserialize JSON");

    let config = MapGenerationConfig {
        seed,
        ..Default::default()
    };
    let data = if map_ldtk::generation::building::is_building_template(&data) {
        map_ldtk::generation::building::build_building_ldtk(&data, seed)
            .expect("Invalid building modules")
    } else {
        let context = from_map(&data, config);
        let mut generator = GeneratedMap::create(data);
        map_generation(context, &mut generator).expect("Failed to generate map");
        generator.get_generated_map()
    };

    // Convert JSON data to a pretty formatted string
    let pretty_json_string = to_string_pretty(&data).expect("Failed to serialize JSON");

    // Open the file for writing
    let mut file = File::create(map_output_path).expect("Failed to create file");

    // Write the pretty formatted JSON string to the file
    file.write_all(pretty_json_string.as_bytes())
        .expect("Failed to write to file");

    println!("Generated map written to {}", map_output_path);
}
