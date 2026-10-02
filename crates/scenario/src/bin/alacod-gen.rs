//! `alacod-gen <games/jeu> [--play] [--bless]` (T2.10) : écrit un scénario par arme (à
//! distance et de corps à corps) du registre de `<games/jeu>` dans
//! `tests/scenarios/generated/<jeu>/weapon_<id>.ron` (gabarit `Template::WeaponOnTarget`,
//! voir `crates/scenario/src/generate.rs`), et supprime ceux dont l'arme n'existe plus.
//!
//! - `--play` : joue chaque scénario généré et affiche un tableau (arme, frames, coups sur la cible, attentes,
//!   trace) ; code de sortie 1 si une attente échoue, si un scénario n'atteint pas sa
//!   dernière frame, ou si sa trace diffère de la référence (`tests/scenarios/generated/<jeu>/<fichier>.trace`).
//! - `--bless` : comme `--play`, mais écrit la trace de référence au lieu de la comparer.
//!
//! `make gen GAME=zombies` appelle `alacod-gen games/$(GAME) --play`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use scenario::generate::{self, GeneratedScenario, WeaponKind};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let play = args.iter().any(|a| a == "--play");
    let bless = args.iter().any(|a| a == "--bless");
    let game_dir = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("usage : alacod-gen <games/jeu> [--play] [--bless]"));

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

    let generated = generate::generate_game(&game_dir).unwrap_or_else(|e| {
        panic!("alacod-gen : {} : {e}", game_dir.display());
    });

    let out_dir = generated_root_dir().join(&game_name);
    std::fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("alacod-gen : création de {} : {e}", out_dir.display()));

    // Écrit chaque scénario (en-tête + RON), déterministe (même contenu à chaque appel :
    // ordre du registre, aucune horloge/aléa dans `generate::build_scenario`).
    let mut keep = BTreeSet::new();
    for g in &generated {
        let path = out_dir.join(&g.file_name);
        let body = format!(
            "{}{}\n",
            generate::header_comment(&g.weapon_id, g.kind),
            g.scenario.to_ron()
        );
        std::fs::write(&path, body)
            .unwrap_or_else(|e| panic!("alacod-gen : écriture de {} : {e}", path.display()));
        keep.insert(g.file_name.clone());
    }
    remove_stale(&out_dir, &keep);

    let ranged = generated
        .iter()
        .filter(|g| g.kind == WeaponKind::Ranged)
        .count();
    let melee = generated.len() - ranged;
    println!(
        "alacod-gen : {} : {ranged} arme(s) à distance, {melee} de corps à corps -> {}",
        game_dir.display(),
        out_dir.display(),
    );

    if !play && !bless {
        return;
    }

    let all_ok = run_and_report(&generated, &out_dir, bless);
    if !all_ok {
        std::process::exit(1);
    }
}

/// Joue chaque scénario généré, affiche le tableau de résultats, blesse ou compare la trace.
/// Retourne `true` si tout est vert (aucune attente échouée, aucune trace différente/absente
/// hors `--bless`).
fn run_and_report(generated: &[GeneratedScenario], out_dir: &Path, bless: bool) -> bool {
    println!(
        "{:<20} {:>7} {:>7}  {:<8}  {}",
        "arme", "frames", "coups", "attentes", "trace"
    );
    let mut all_ok = true;
    for g in generated {
        let outcome = scenario::run(&g.scenario);
        let expect_ok = outcome.failures.is_empty();
        all_ok &= expect_ok;

        let trace_path = out_dir.join(&g.file_name).with_extension("trace");
        let trace_text = outcome.trace.join("\n") + "\n";
        let trace_status = if bless {
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

        println!(
            "{:<20} {:>7} {:>7}  {:<8}  {}",
            g.weapon_id,
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
            "alacod-gen : suppression de {} (arme disparue du registre)",
            path.display()
        );
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("trace"));
    }
}
