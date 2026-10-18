//! Outil de preuve « l'ordre de la simulation n'a pas bougé » (D69 étape 2,
//! `docs/taches/dette-d69-etape2.md` §Preuve, `docs/taches/rapports/dette-d69-plan-etape2.md` §7).
//!
//! Construit l'app headless d'un scénario par jeu, **sans jouer une seule frame**, et écrit :
//! - `<dossier>/<jeu>.schedules.txt` : pour chaque schedule du rollback (`ReadInputs`,
//!   `GgrsSchedule`, `AdvanceWorld`, `SaveWorld`, `LoadWorld`), les systèmes dans l'ordre exact de
//!   l'ordre exécutable (celui de l'exécuteur monothread, voir `single_threaded_rollback_schedules`),
//!   `ApplyDeferred` compris ;
//! - `<dossier>/<jeu>.rollback.txt` : les types enregistrés par `RollbackTraceApp`
//!   (`utils::rollback::TracedTypes`), dans l'ordre d'enregistrement.
//!
//! Les noms sont **normalisés** : le chemin de module est retiré (`a::b::f` → `f`, `a::B<c::D>` →
//! `B<D>`), pour que déplacer une fonction d'un crate à l'autre ne change rien à la sortie ; seul
//! l'ordre, le nombre et le nom court des systèmes comptent. Les deux fichiers de deux commits
//! doivent être **identiques octet pour octet** (`diff -r`) : `scripts/schedule-order-diff.sh`.
//!
//! `#[ignore]` : outil, pas test de suite. `ALACOD_SCHEDULE_DUMP=<dossier>` (défaut
//! `target/schedule-order`), `ALACOD_SCHEDULE_SCENARIOS=jeu:nom,jeu:nom` pour d'autres scénarios.

use std::fmt::Write as _;
use std::path::PathBuf;

use bevy::ecs::schedule::ScheduleLabel as _;
use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;
use scenario::Scenario;
use utils::rollback::TracedTypes;

/// Un scénario par jeu : le contenu du jeu peut conditionner des enregistrements.
const DEFAULT_SCENARIOS: &[(&str, &str)] = &[
    ("zombies", "idle"),
    ("gungeon", "bot_explorer_floor"),
    ("isaac", "bench_floor_isaac"),
    ("throne", "throne_floor_1"),
    ("testbed", "aim_four_way"),
];

/// `bevy_ecs::schedule::executor::apply_deferred` et les chemins de modules : seul le nom court
/// reste. `a::b::f<c::D, e::F>` → `f<D, F>`.
fn short(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut segment = String::new();
    let chars: Vec<char> = name.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ':' && chars.get(i + 1) == Some(&':') {
            segment.clear();
            i += 2;
            continue;
        }
        if c.is_alphanumeric() || c == '_' || c == '{' || c == '}' {
            segment.push(c);
        } else {
            out.push_str(&segment);
            segment.clear();
            out.push(c);
        }
        i += 1;
    }
    out.push_str(&segment);
    out
}

fn scenarios_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios"
    ))
}

fn scenario_source(name: &str, game: &str) -> String {
    let flat = scenarios_dir().join(format!("{name}.ron"));
    let generated = scenarios_dir()
        .join("generated")
        .join(game)
        .join(format!("{name}.ron"));
    std::fs::read_to_string(&flat)
        .or_else(|_| std::fs::read_to_string(&generated))
        .unwrap_or_else(|e| panic!("scénario {name} (jeu {game}) introuvable : {e}"))
}

#[test]
#[ignore]
fn dump_schedule_order() {
    let dir = PathBuf::from(
        std::env::var("ALACOD_SCHEDULE_DUMP").unwrap_or_else(|_| "target/schedule-order".into()),
    );
    std::fs::create_dir_all(&dir).unwrap();
    let chosen: Vec<(String, String)> = match std::env::var("ALACOD_SCHEDULE_SCENARIOS") {
        Ok(list) => list
            .split(',')
            .filter_map(|s| s.split_once(':'))
            .map(|(g, n)| (g.to_string(), n.to_string()))
            .collect(),
        Err(_) => DEFAULT_SCENARIOS
            .iter()
            .map(|(g, n)| (g.to_string(), n.to_string()))
            .collect(),
    };

    for (game, name) in chosen {
        let scenario = Scenario::from_ron(&scenario_source(&name, &game)).unwrap();
        let mut app = scenario::runner::build_app(
            &scenario,
            true,
            &scenario::runner::PlayConfig {
                follow_handle: None,
            },
        );
        app.finish();
        app.cleanup();

        let mut schedules = String::new();
        let labels = [
            ("ReadInputs", bevy_ggrs::ReadInputs.intern()),
            ("GgrsSchedule", GgrsSchedule.intern()),
            ("AdvanceWorld", bevy_ggrs::AdvanceWorld.intern()),
            ("SaveWorld", bevy_ggrs::SaveWorld.intern()),
            ("LoadWorld", bevy_ggrs::LoadWorld.intern()),
        ];
        for (label_name, label) in labels {
            app.world_mut()
                .try_schedule_scope(label, |world, schedule| {
                    // Une ambiguïté d'ordre refusée par `ambiguity_detection: Error` échoue ici,
                    // comme à la première frame du jeu.
                    schedule
                        .initialize(world)
                        .unwrap_or_else(|e| panic!("{label_name} : {e}"));
                    writeln!(schedules, "# {label_name}").unwrap();
                    for (i, (_, system)) in schedule.systems().unwrap().enumerate() {
                        writeln!(schedules, "{i:04} {}", short(&system.name().to_string()))
                            .unwrap();
                    }
                })
                .unwrap_or_else(|_| panic!("schedule {label_name} absent"));
        }
        std::fs::write(dir.join(format!("{game}.schedules.txt")), schedules).unwrap();

        let mut rollback = String::new();
        let traced = app.world().resource::<TracedTypes>();
        for (i, name) in traced.0.iter().enumerate() {
            writeln!(rollback, "{i:04} {}", short(name)).unwrap();
        }
        std::fs::write(dir.join(format!("{game}.rollback.txt")), rollback).unwrap();
    }
}

#[test]
fn short_retire_les_chemins_de_module() {
    assert_eq!(short("game::weapons::fire_system"), "fire_system");
    assert_eq!(
        short("game::a::spawn<bevy_ecs::entity::Entity, game::b::C>"),
        "spawn<Entity, C>"
    );
    assert_eq!(short("game::core::{{closure}}"), "{{closure}}");
}
