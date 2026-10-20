//! Génération et rendu de cartes sans démarrer le jeu (`docs/conventions.md` §1).

use std::path::PathBuf;

use clap::Parser;
use content::registry::{cave_designation, floor_designation};
use map::generation::config::{MapGenerationConfig, MapGenerationMode};
use map_ldtk::preview;

#[derive(Parser, Debug)]
#[command(
    name = "alacod-map",
    about = "Aperçu PNG + LDtk de cartes générées, sans le jeu"
)]
struct Args {
    /// Bibliothèque `.ldtk` ou dossier d'un jeu (`games/zombies`)
    source: PathBuf,
    /// Graines séparées par des virgules (défaut : 1 à 6, ou la graine du jeu)
    #[arg(long, value_delimiter = ',')]
    seeds: Option<Vec<i32>>,
    /// Dossier de sortie
    #[arg(long, default_value = "target/map_preview")]
    out: PathBuf,
    /// Jeu dont on prend les réglages `entry.map` pour une bibliothèque
    #[arg(long)]
    game: Option<PathBuf>,
    /// Grammaire d'étage du jeu à assembler à la place de la carte choisie
    #[arg(long)]
    floor: Option<String>,
    /// Plan abstrait seulement (pas de tuiles)
    #[arg(long)]
    plain: bool,
}

fn main() -> std::process::ExitCode {
    match run(Args::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("alacod-map : {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let source_is_game = args.source.is_dir();
    if source_is_game && args.game.is_some() {
        return Err(
            "source est déjà un dossier de jeu ; --game s'utilise avec une bibliothèque".into(),
        );
    }
    let game_dir = if source_is_game {
        Some(&args.source)
    } else {
        args.game.as_ref()
    };
    let manifest = game_dir
        .map(|dir| content::GameManifest::load(dir))
        .transpose()?;
    let selected = args
        .floor
        .as_ref()
        .map(|id| format!("floor:{id}"))
        .unwrap_or_else(|| {
            if source_is_game {
                manifest
                    .as_ref()
                    .expect("manifeste chargé")
                    .entry
                    .start_map
                    .clone()
            } else {
                args.source.to_string_lossy().into_owned()
            }
        });
    let mut config = MapGenerationConfig::from_settings(
        &manifest
            .as_ref()
            .map(|m| m.entry.map.clone())
            .unwrap_or_default(),
    );
    config.map_path = selected.clone();
    let floor_id = floor_designation(&selected);
    let cave_id = cave_designation(&selected);
    let (library, stem) = if floor_id.is_some() || cave_id.is_some() {
        let dir =
            game_dir.ok_or("une désignation floor:/cave: demande un dossier de jeu ou --game")?;
        let (registry, _, errors) = content::load_and_lint(dir)?;
        if !errors.is_empty() {
            return Err(format!("{} : contenu invalide :\n{errors:#?}", dir.display()).into());
        }
        let (template, id) = if let Some(id) = floor_id {
            let floor = registry
                .floor_grammars
                .get(&id)
                .ok_or_else(|| format!("grammaire d'étage « {id} » inconnue"))?;
            config.mode = MapGenerationMode::Floor(floor.grammar.clone());
            (floor.templates.clone(), id.to_string())
        } else {
            let id = cave_id.expect("désignation de caverne");
            let cave = registry
                .caves
                .get(&id)
                .ok_or_else(|| format!("caverne « {id} » inconnue"))?;
            config.mode = MapGenerationMode::Cave(cave.config.clone());
            (cave.template.clone(), id.to_string())
        };
        (content::GameManifest::assets_dir(dir).join(template), id)
    } else {
        let path = if source_is_game {
            content::GameManifest::assets_dir(&args.source).join(&selected)
        } else {
            args.source.clone()
        };
        let stem = path
            .file_stem()
            .ok_or("bibliothèque sans nom de fichier")?
            .to_string_lossy()
            .into_owned();
        (path, stem)
    };
    if !library.is_file() {
        return Err(format!("bibliothèque introuvable : {}", library.display()).into());
    }
    let seeds = args.seeds.unwrap_or_else(|| {
        if source_is_game {
            vec![
                manifest
                    .as_ref()
                    .expect("manifeste chargé")
                    .entry
                    .default_seed,
            ]
        } else {
            (1..=6).collect()
        }
    });
    std::fs::create_dir_all(&args.out)?;
    println!(
        "bibliothèque {} ; {} pièces max, bornes ±{}×±{} px, prises {}, boucles {}, murage {}",
        library.display(),
        config.max_room,
        config.max_width,
        config.max_heigth,
        if config.aligned_openings {
            "alignées"
        } else {
            "libres"
        },
        if config.close_loops { "oui" } else { "non" },
        if config.wall_unused_openings {
            "oui"
        } else {
            "non"
        },
    );
    for seed in seeds {
        config.seed = seed;
        let generated = preview::generate(&library, config.clone());
        let ldtk_path = args.out.join(format!("{stem}_{seed}.ldtk"));
        preview::write_ldtk(&generated, &library, &ldtk_path);
        let png_path = args.out.join(format!("{stem}_{seed}.png"));
        let report = preview::render(&generated, &library, !args.plain, &png_path);
        println!(
            "graine {seed} : {} pièces, {} boucle(s), {} prise(s) murée(s)",
            generated.ldtk.levels.len(),
            report.loops,
            report.walled
        );
        for (i, line) in report.rooms.iter().enumerate() {
            println!("  {i:>2} {line}");
        }
        for missing in &report.missing_tilesets {
            eprintln!("  tileset illisible : {missing} (calque dessiné sans tuiles)");
        }
        println!("  → {} / {}", png_path.display(), ldtk_path.display());
    }
    Ok(())
}
