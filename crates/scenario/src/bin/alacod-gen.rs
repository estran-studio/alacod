//! `alacod-gen <games/jeu> [--play] [--bless]` (T2.10, T1.13) : écrit un scénario par arme (à
//! distance et de corps à corps) du registre de `<games/jeu>` dans
//! `tests/scenarios/generated/<jeu>/weapon_<id>.ron` (gabarit `Template::WeaponOnTarget`,
//! voir `crates/scenario/src/generate.rs`), un par gabarit actif de chaque personnage qui
//! déclare `test:` (`enemy_<id>_still.ron`, `enemy_<id>_moving.ron`), un par objet
//! (`item_<id>.ron`), type de salle (`room_<id>.ron`) et boss (`boss_<id>.ron`) qui déclarent
//! `test:` (M2-T17, `docs/conventions.md` §54), et supprime ceux dont le sujet n'existe plus.
//!
//! - `--play` : joue chaque scénario généré et affiche un tableau (arme, frames, coups sur la cible, attentes,
//!   trace) ; code de sortie 1 si une attente échoue, si un scénario n'atteint pas sa
//!   dernière frame, ou si sa trace diffère de la référence (`tests/scenarios/generated/<jeu>/<fichier>.trace`).
//! - `--bless` : comme `--play`, mais écrit la trace de référence au lieu de la comparer.
//!
//! - `--only <id>[,<id>]` : ne génère et ne joue que ces sujets (`amulette` ou `item_amulette`) ;
//!   rien n'est supprimé. `--list` : liste les sujets sans rien écrire.
//! - `--no-trace` : ne compare ni n'écrit aucune trace (seules les attentes comptent).
//! - `--written` : inclut aussi `<jeu>/tests/scenarios/*.ron` (T-1, `alacod test`).
//!   Leurs traces sont obligatoires, même avec `--no-trace` (réservé aux générés).
//! - `--out <dossier>` : où écrire scénarios et traces (défaut : `tests/scenarios/generated/<jeu>`
//!   de l'engine ; pour un jeu hors de l'engine, `<jeu>/tests/generated`).
//! - `--json` : un tableau JSON des résultats sur stdout au lieu du tableau texte.
//!
//! Un jeu hors de `games/` (créé par `alacod new`) est joué grâce à `ALACOD_GAME_NAME` et
//! `ALACOD_GAME_DIR`, posés ici d'après son manifeste (`scenario::runner::game_dir`).
//!
//! `make gen GAME=zombies` appelle `alacod-gen games/$(GAME) --play`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use scenario::generate::{self, GeneratedKind, GeneratedScenario, WeaponKind};

/// Valeur d'une option `--nom <valeur>`.
fn option_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let play = args.iter().any(|a| a == "--play");
    let bless = args.iter().any(|a| a == "--bless");
    let no_trace = args.iter().any(|a| a == "--no-trace");
    let json = args.iter().any(|a| a == "--json");
    let include_written = args.iter().any(|a| a == "--written");
    let list = args.iter().any(|a| a == "--list");
    let only: Option<Vec<String>> =
        option_value(&args, "--only").map(|v| v.split(',').map(str::to_string).collect());
    let out_override = option_value(&args, "--out").map(PathBuf::from);
    // Le dossier du jeu est le premier argument positionnel (hors valeurs d'options).
    let game_dir = args
        .iter()
        .enumerate()
        .find(|(i, a)| {
            !a.starts_with("--")
                && !(*i > 0 && matches!(args[i - 1].as_str(), "--only" | "--out"))
        })
        .map(|(_, a)| PathBuf::from(a))
        .unwrap_or_else(|| {
            panic!("usage : alacod-gen <games/jeu> [--play] [--bless] [--only id] [--list] [--no-trace] [--out dossier] [--json]")
        });

    let game_name = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| {
            panic!(
                "alacod-gen : chemin de jeu invalide : {}",
                game_dir.display()
            )
        })
        .to_string();

    // Jeu hors de l'engine : le runner le retrouve par son nom (voir `runner::game_dir`).
    let manifest = content::GameManifest::load(&game_dir)
        .unwrap_or_else(|e| panic!("alacod-gen : {} : {e}", game_dir.display()));
    let in_engine = game_dir
        .canonicalize()
        .ok()
        .zip(
            scenario::runner::game_dir(&manifest.name)
                .canonicalize()
                .ok(),
        )
        .is_some_and(|(a, b)| a == b);
    if !in_engine {
        let absolute = game_dir.canonicalize().unwrap_or_else(|e| {
            panic!("alacod-gen : {} : {e}", game_dir.display());
        });
        std::env::set_var("ALACOD_GAME_NAME", &manifest.name);
        std::env::set_var("ALACOD_GAME_DIR", absolute);
    }

    let mut written = if include_written {
        written_scenarios(&game_dir).unwrap_or_else(|error| {
            eprintln!("alacod-gen : {error}");
            std::process::exit(1);
        })
    } else {
        Vec::new()
    };
    // L'id qualifié d'un scénario écrit ne dépend pas de la génération du contenu.
    let only_written = only.as_ref().is_some_and(|ids| {
        ids.iter()
            .all(|id| written.iter().any(|path| id == &written_id(path)))
    });
    let mut generation_ok = true;
    let mut generated = if only_written {
        Vec::new()
    } else {
        match generate::generate_game(&game_dir) {
            Ok(generated) => generated,
            Err(error) if include_written => {
                eprintln!("alacod-gen : {} : {error}", game_dir.display());
                generation_ok = false;
                Vec::new()
            }
            Err(error) => panic!("alacod-gen : {} : {error}", game_dir.display()),
        }
    };

    if list {
        for g in &generated {
            println!("{}\t{}", g.file_name.trim_end_matches(".ron"), g.subject_id);
        }
        for path in &written {
            let id = written_id(path);
            println!("{id}\t{id}");
        }
        if !generation_ok {
            std::process::exit(1);
        }
        return;
    }
    if let Some(only) = &only {
        let stem = |g: &GeneratedScenario| g.file_name.trim_end_matches(".ron").to_string();
        let wanted =
            |g: &GeneratedScenario| only.iter().any(|o| *o == stem(g) || *o == g.subject_id);
        let mut known: Vec<String> = generated.iter().map(stem).collect();
        known.extend(written.iter().map(|path| written_id(path)));
        generated.retain(|g| wanted(g));
        written.retain(|path| {
            let id = written_id(path);
            only.iter()
                .any(|o| o == &id || Some(o.as_str()) == path.file_stem().and_then(|s| s.to_str()))
        });
        if generated.is_empty() && written.is_empty() {
            eprintln!(
                "alacod-gen : aucun sujet « {} » (disponibles : {})",
                only.join(", "),
                known.join(", ")
            );
            std::process::exit(2);
        }
    }

    let out_dir = out_override.unwrap_or_else(|| {
        if in_engine {
            generated_root_dir().join(&game_name)
        } else {
            game_dir.join("tests").join("generated")
        }
    });
    std::fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("alacod-gen : création de {} : {e}", out_dir.display()));

    // Écrit chaque scénario (en-tête + RON), déterministe (même contenu à chaque appel :
    // ordre du registre, aucune horloge/aléa dans `generate::build_scenario`).
    let mut keep = BTreeSet::new();
    for g in &generated {
        let path = out_dir.join(&g.file_name);
        let body = format!(
            "{}{}\n",
            generate::header_comment(&g.subject_id, g.kind),
            g.scenario.to_ron()
        );
        std::fs::write(&path, body)
            .unwrap_or_else(|e| panic!("alacod-gen : écriture de {} : {e}", path.display()));
        keep.insert(g.file_name.clone());
    }
    if only.is_none() && generation_ok {
        remove_stale(&out_dir, &keep);
    }

    let quiet = json;
    let count = |kind: GeneratedKind| generated.iter().filter(|g| g.kind == kind).count();
    let ranged = count(GeneratedKind::Weapon(WeaponKind::Ranged));
    let melee = count(GeneratedKind::Weapon(WeaponKind::Melee));
    let items = count(GeneratedKind::Item);
    let rooms = count(GeneratedKind::Room);
    let bosses = count(GeneratedKind::Boss);
    // M3-T13 : gabarits de larmes, de familiers et de champions.
    let tears = count(GeneratedKind::Tears);
    let familiars = count(GeneratedKind::Familiar);
    let champions = count(GeneratedKind::Champion);
    let controls = count(GeneratedKind::Control);
    let characters = generated.len()
        - ranged
        - melee
        - items
        - rooms
        - bosses
        - tears
        - familiars
        - champions
        - controls;
    if !quiet {
        println!(
        "alacod-gen : {} : {ranged} arme(s) à distance, {melee} de corps à corps, {characters} gabarit(s) de personnage, {items} objet(s), {rooms} salle(s), {bosses} boss, {tears} larme(s), {familiars} familier(s), {champions} champion(s), {controls} témoin(s) -> {}",
        game_dir.display(),
        out_dir.display(),
    );
    }

    if !play && !bless {
        return;
    }

    let written: Vec<_> = written
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                eprintln!("alacod-gen : {} : {error}", path.display());
                std::process::exit(1);
            });
            let scenario = scenario::Scenario::from_ron(&source).unwrap_or_else(|error| {
                eprintln!("alacod-gen : {} : RON invalide : {error}", path.display());
                std::process::exit(1);
            });
            (path, scenario)
        })
        .collect();
    let cases = generated
        .iter()
        .map(|g| RunCase {
            id: g.file_name.trim_end_matches(".ron").to_string(),
            subject: &g.subject_id,
            scenario: &g.scenario,
            trace_path: out_dir.join(&g.file_name).with_extension("trace"),
            compare_trace: !no_trace,
        })
        .chain(written.iter().map(|(path, scenario)| RunCase {
            id: written_id(path),
            subject: path.file_stem().and_then(|s| s.to_str()).expect("nom RON"),
            scenario,
            trace_path: path.with_extension("trace"),
            compare_trace: true,
        }));
    let all_ok = run_and_report(cases, bless, json);
    if !all_ok || !generation_ok {
        std::process::exit(1);
    }
}

/// Scénarios écrits uniquement à la racine du dossier dédié ; jamais réécrits ou supprimés
/// par la génération. Le RON n'est lu qu'après sélection par id.
fn written_scenarios(game_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = game_dir.join("tests/scenarios");
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{} : {error}", dir.display())),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|error| format!("{} : {error}", dir.display()))?
            .path();
        if path.extension().is_some_and(|ext| ext == "ron") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn written_id(path: &Path) -> String {
    format!(
        "scenarios/{}",
        path.file_stem().expect("nom RON").to_string_lossy()
    )
}

struct RunCase<'a> {
    id: String,
    subject: &'a str,
    scenario: &'a scenario::Scenario,
    trace_path: PathBuf,
    compare_trace: bool,
}

/// Même `scenario::run` que le harnais de l'engine : attentes, invariants, fin atteinte.
fn run_and_report<'a>(cases: impl Iterator<Item = RunCase<'a>>, bless: bool, json: bool) -> bool {
    let mut results: Vec<serde_json::Value> = Vec::new();
    if !json {
        println!(
            "{:<20} {:>7} {:>7}  {:<8}  {}",
            "sujet", "frames", "coups", "attentes", "trace"
        );
    }
    let mut all_ok = true;
    for case in cases {
        let outcome = scenario::run(case.scenario);
        let expect_ok = outcome.failures.is_empty();
        all_ok &= expect_ok;

        let trace_path = &case.trace_path;
        let trace_text = outcome.trace.join("\n") + "\n";
        let trace_status = if !case.compare_trace {
            "ignorée"
        } else if bless {
            std::fs::write(&trace_path, &trace_text).unwrap_or_else(|e| {
                panic!("alacod-gen : écriture de {} : {e}", trace_path.display())
            });
            "blessée"
        } else {
            match std::fs::read_to_string(&trace_path) {
                Ok(existing) if existing == trace_text => "ok",
                Ok(_) => {
                    all_ok = false;
                    "différente"
                }
                Err(_) => {
                    all_ok = false;
                    "absente"
                }
            }
        };

        if json {
            results.push(serde_json::json!({
                "scenario": case.id,
                "subject": case.subject,
                "ok": expect_ok && matches!(trace_status, "ok" | "blessée" | "ignorée"),
                "trace_compared": matches!(trace_status, "ok" | "différente"),
                "frames": outcome.metrics.frames,
                "hits": outcome.entity_hits.values().sum::<u32>(),
                "expectations": case.scenario.expect.len(),
                "expectations_ok": expect_ok,
                "failures": outcome.failures.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
                "trace": trace_status,
            }));
            continue;
        }
        println!(
            "{:<20} {:>7} {:>7}  {:<8}  {}",
            case.id,
            outcome.metrics.frames,
            outcome.entity_hits.values().sum::<u32>(),
            if expect_ok { "ok" } else { "échec" },
            trace_status,
        );
        if !expect_ok {
            for failure in &outcome.failures {
                println!("    {failure}");
            }
        }
    }
    if json {
        println!("{}", serde_json::Value::Array(results));
    }
    all_ok
}

fn generated_root_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/generated"
    ))
}

/// Supprime, dans `out_dir`, tout scénario généré (`.ron` + `.trace` sœur) dont le nom de
/// fichier n'est pas dans `keep` : l'arme correspondante n'existe plus dans le registre.
fn remove_stale(out_dir: &Path, keep: &BTreeSet<String>) {
    let Ok(read_dir) = std::fs::read_dir(out_dir) else {
        return;
    };
    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".ron") || keep.contains(name) {
            continue;
        }
        println!(
            "alacod-gen : suppression de {} (sujet disparu du registre)",
            path.display()
        );
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("trace"));
    }
}
