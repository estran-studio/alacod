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
//! `ALACOD_SCHEDULE_ALL=1` (D69 E7) ajoute `<jeu>.all.txt` (tous les schedules, avec les paires de
//! systèmes non ordonnés en conflit) et `presentation.all.txt` (avec `PresentationPlugin`) : la sortie
//! par défaut, celle de `make schedule_order_diff`, n'est pas modifiée.

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

        if std::env::var("ALACOD_SCHEDULE_ALL").is_ok_and(|v| v == "1") {
            std::fs::write(
                dir.join(format!("{game}.all.txt")),
                dump_all_schedules(&mut app),
            )
            .unwrap();
        }
    }
}

/// Option `ALACOD_SCHEDULE_ALL=1` (D69 E7) : **tous** les schedules de l'app headless, pas seulement ceux
/// du rollback (`Startup`, `Update`, `PostUpdate`, `OnEnter(..)`, `OnExit(..)`, `Last`…), dans l'ordre
/// exécutable, puis, pour chacun, les paires de systèmes **non ordonnés qui se disputent une donnée**
/// (`conflicting_systems`), écrites « premier < second » : ce sont les seules paires dont le résultat
/// peut dépendre de l'ordre d'insertion des plugins. Le tri topologique de bevy ne suit pas l'ordre
/// d'insertion : un plugin déplacé retourne aussi des paires qu'il ne touche pas ; elles se jugent
/// une à une (conditions d'état, ressources en jeu). Les schedules de `PresentationPlugin`
/// n'existent pas en headless : voir `dump_presentation_order`.
fn dump_all_schedules(app: &mut App) -> String {
    let labels: Vec<_> = app
        .world()
        .resource::<Schedules>()
        .iter()
        .map(|(_, schedule)| schedule.label())
        .collect();
    let mut sections: Vec<(String, String)> = Vec::new();
    for label in labels {
        let label_name = format!("{label:?}");
        let text = app
            .world_mut()
            .try_schedule_scope(label, |world, schedule| {
                let mut text = String::new();
                if let Err(e) = schedule.initialize(world) {
                    writeln!(text, "# {label_name} : {e}").unwrap();
                    return text;
                }
                writeln!(text, "# {label_name}").unwrap();
                let mut position = std::collections::BTreeMap::new();
                let mut names = std::collections::BTreeMap::new();
                for (i, (key, system)) in schedule.systems().unwrap().enumerate() {
                    let name = short(&system.name().to_string());
                    writeln!(text, "{i:04} {name}").unwrap();
                    position.insert(key, i);
                    names.insert(key, name);
                }
                let mut conflicts: Vec<String> = schedule
                    .graph()
                    .conflicting_systems()
                    .0
                    .iter()
                    .filter(|(a, b, _)| position.contains_key(a) && position.contains_key(b))
                    .map(|(a, b, _)| {
                        let (first, second) = if position[a] < position[b] {
                            (a, b)
                        } else {
                            (b, a)
                        };
                        format!("{} < {}", names[first], names[second])
                    })
                    .collect();
                conflicts.sort();
                for line in conflicts {
                    writeln!(text, "  conflit {line}").unwrap();
                }
                text
            })
            .unwrap_or_else(|_| format!("# {label_name} : absent\n"));
        sections.push((label_name, text));
    }
    sections.sort_by(|a, b| a.0.cmp(&b.0));
    sections.into_iter().map(|(_, text)| text).collect()
}

/// Ordre des schedules **avec** `PresentationPlugin` (`<dossier>/presentation.all.txt`) : pas de fenêtre
/// ni de GPU sur un serveur, donc les plugins par défaut sont ceux du mode headless (rendu sans
/// backend, pas de fenêtre) et seul `CoreSetupPlugin` reçoit `headless: false`, ce qui ajoute
/// `PresentationPlugin` à sa place exacte dans `build`. Aucune frame n'est jouée. Même format que
/// `ALACOD_SCHEDULE_ALL` ; c'est la preuve d'ordre des systèmes de présentation (`Update`,
/// `PostUpdate`…) entre deux commits.
#[test]
#[ignore]
fn dump_presentation_order() {
    // Option : `make schedule_order_diff` (sortie par défaut) n'écrit pas ce fichier.
    if std::env::var("ALACOD_SCHEDULE_ALL").map_or(true, |v| v != "1") {
        return;
    }
    let dir = PathBuf::from(
        std::env::var("ALACOD_SCHEDULE_DUMP").unwrap_or_else(|_| "target/schedule-order".into()),
    );
    std::fs::create_dir_all(&dir).unwrap();
    let config = |headless| game::session::CoreSetupConfig {
        app_name: "presentation".into(),
        headless,
        asset_root: Some(scenario::runner::assets_dir("zombies")),
    };
    let mut app = App::new();
    app.add_plugins(game::core::CoreSetupPlugin(config(true)).get_default_plugin())
        .insert_resource(game::profile::ProfileSettings { dir: None })
        .add_plugins(game::core::CoreSetupPlugin(config(false)));
    app.finish();
    app.cleanup();
    std::fs::write(
        dir.join("presentation.all.txt"),
        dump_all_schedules(&mut app),
    )
    .unwrap();
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
