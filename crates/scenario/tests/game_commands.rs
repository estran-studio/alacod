//! K5-2 : commandes de partie indexées par frame (`GameCommands`) et `Stats` rechargées à chaud.
//!
//! (a) un `SpawnCharacter` posé à la frame 31 crée l'entité à cette frame, avec sa clé de config et
//! le net id que donne un placement de scénario équivalent ; (b) la session enregistrée contient le
//! placement et son rejeu par le runner donne la même trace ; (c) tout cela en synctest d=2 (défaut
//! du runner) sans desync : la table n'est jamais dépilée ; (d) `max_speed` édité à chaud change
//! `Stats` du joueur vivant sans toucher `Modifiers`.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_fixed::fixed_math::Fixed;
use combat::actors::Player;
use game::character::config::CharacterConfigKey;
use game::character::config_asset::CharacterConfigAsset;
use game::game_commands::{GameCommand, GameCommands};
use game::global_asset::GlobalAsset;
use game::recording::InputRecorder;
use game::replay::CharacterPlacement;
use scenario::Scenario;
use sim_core::modifier::{ModifierOp, ModifierSource, Modifiers};
use sim_core::stats::{StatId, Stats};
use stats::StatReader;
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;

const AT: u32 = 31;
const FRAMES: u32 = 60;

fn base_scenario() -> Scenario {
    let path = format!(
        "{}/../../tests/scenarios/testbed_arena_idle.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut scenario = Scenario::from_ron(&std::fs::read_to_string(path).unwrap()).unwrap();
    scenario.frames = FRAMES;
    scenario.expect.clear();
    scenario
}

fn spawn_target() -> GameCommand {
    GameCommand::SpawnCharacter {
        character: "target".into(),
        x: Fixed::from_num(100),
        y: Fixed::from_num(100),
        team: None,
        variant: None,
    }
}

/// Par frame de jeu : `(nombre de « target », net ids)`, relevé à chaque `Update`.
type Seen = Arc<Mutex<BTreeMap<u32, (usize, Vec<usize>)>>>;

fn observe(app: &mut App, seen: &Seen) {
    let sink = seen.clone();
    app.add_systems(
        Update,
        move |frame: Res<FrameCount>, query: Query<(&GgrsNetId, &CharacterConfigKey)>| {
            let mut ids: Vec<usize> = query
                .iter()
                .filter(|(_, key)| key.0 == "target")
                .map(|(id, _)| id.0)
                .collect();
            ids.sort();
            sink.lock().unwrap().insert(frame.frame, (ids.len(), ids));
        },
    );
}

/// Joue le scénario avec la commande posée à `AT` (comme le fait le handler BRP : table + enregistreur).
fn play_with_command(scenario: &Scenario, seen: &Seen) -> scenario::runner::ScenarioOutcome {
    scenario::runner::run_with(scenario, |app| {
        observe(app, seen);
        // `GameCommands` est vidée à `OnEnter(GameLoading)` : la remplir après.
        app.add_systems(
            OnEnter(game::session::AppState::GameLoading),
            (|mut commands: ResMut<GameCommands>| {
                commands.0.entry(AT).or_default().push(spawn_target());
            })
            .after(map_ldtk::game::commands::clear_game_commands),
        );
        app.world_mut()
            .resource_mut::<InputRecorder>()
            .record_command(&spawn_target(), AT);
    })
}

fn placement_scenario() -> Scenario {
    let mut scenario = base_scenario();
    scenario.characters = vec![CharacterPlacement {
        character: "target".into(),
        x: Fixed::from_num(100),
        y: Fixed::from_num(100),
        at_frame: AT,
        variant: None,
        team: None,
    }];
    scenario
}

#[test]
fn commande_spawn_cree_l_entite_a_la_frame_visee() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let seen: Seen = Default::default();
    let outcome = play_with_command(&base_scenario(), &seen);
    assert!(outcome.failures.is_empty(), "{:#?}", outcome.failures);
    let seen = seen.lock().unwrap();
    // `FrameCount` relu dans `Update` vaut la prochaine frame à simuler : la commande de la frame
    // AT s'exécute pendant le tick AT et devient visible quand le compteur passe à AT + 1.
    let (before, _) = seen[&AT].clone();
    let (after, ids) = seen[&(AT + 1)].clone();
    assert_eq!(
        after,
        before + 1,
        "une « target » de plus après la frame {AT}"
    );
    assert_eq!(seen[&FRAMES].0, before + 1, "elle reste en vie");

    // Net id attendu : celui d'un placement de scénario équivalent (même ordre d'allocation).
    let reference: Seen = Default::default();
    let placed = scenario::runner::run_with(&placement_scenario(), |app| observe(app, &reference));
    assert!(placed.failures.is_empty(), "{:#?}", placed.failures);
    assert_eq!(reference.lock().unwrap()[&(AT + 1)].1, ids);
}

#[test]
fn session_enregistree_rejoue_la_meme_trace_en_synctest() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    // (c) le runner joue en synctest d=2 : une resimulation qui perdrait la commande divergerait.
    let seen: Seen = Default::default();
    let outcome = play_with_command(&base_scenario(), &seen);
    assert!(outcome.failures.is_empty(), "{:#?}", outcome.failures);
    // (b) l'enregistrement porte le placement à la frame AT.
    assert_eq!(outcome.recorded.characters.len(), 1);
    let recorded = &outcome.recorded.characters[0];
    assert_eq!(recorded.character, "target");
    assert_eq!(recorded.at_frame, AT);
    let replay = scenario::run(&outcome.recorded);
    assert!(replay.failures.is_empty(), "{:#?}", replay.failures);
    assert!(!outcome.trace.is_empty());
    assert_eq!(outcome.trace, replay.trace, "même trace au rejeu");
    // Et la trace diffère de celle sans commande : la commande agit vraiment.
    let without = scenario::run(&base_scenario());
    assert_ne!(outcome.trace, without.trace);
}

#[test]
fn max_speed_edite_change_les_stats_vivantes_sans_toucher_aux_modificateurs() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    const EDITED: i32 = 321;
    #[derive(Default)]
    struct Measure {
        before: Option<(Stats, Modifiers, Fixed)>,
        after: Option<(Stats, Modifiers, Fixed)>,
    }
    let measure = Arc::new(Mutex::new(Measure::default()));
    let sink = measure.clone();
    scenario::runner::run_with(&base_scenario(), move |app| {
        // Hors synctest : les stats vivantes ne se rechargent qu'à d=0.
        app.world_mut()
            .resource_mut::<game::jjrs::GggrsSessionConfiguration>()
            .connection
            .check_distance = 0;
        app.add_systems(Update, move |world: &mut World| {
            let frame = world.resource::<FrameCount>().frame;
            if ![20, 30, 40].contains(&frame) {
                return;
            }
            let (player, key) = world
                .query_filtered::<(Entity, &CharacterConfigKey), With<Player>>()
                .iter(world)
                .next()
                .map(|(e, k)| (e, k.0.clone()))
                .expect("joueur vivant");
            if frame == 20 {
                world
                    .get_mut::<Modifiers>(player)
                    .expect("Modifiers")
                    .push_from(
                        ModifierSource::Named("test".into()),
                        StatId::MoveSpeed,
                        ModifierOp::Mul,
                        Fixed::from_num(0.5),
                        None,
                    );
            }
            if frame == 30 {
                let handle = world
                    .resource::<GlobalAsset>()
                    .character_configs
                    .get(&key)
                    .expect("config du joueur")
                    .clone();
                world
                    .resource_mut::<Assets<CharacterConfigAsset>>()
                    .get_mut(&handle)
                    .unwrap()
                    .0
                    .movement
                    .max_speed = Fixed::from_num(EDITED);
            }
            let resolved = world
                .run_system_once(move |reader: StatReader| {
                    reader.try_get(player, &StatId::MoveSpeed).unwrap()
                })
                .unwrap();
            let snapshot = (
                world.get::<Stats>(player).unwrap().clone(),
                world.get::<Modifiers>(player).unwrap().clone(),
                resolved,
            );
            let mut m = sink.lock().unwrap();
            match frame {
                30 => m.before = Some(snapshot),
                40 => m.after = Some(snapshot),
                _ => {}
            }
        });
    });
    let m = measure.lock().unwrap();
    let (stats_before, mods_before, resolved_before) = m.before.clone().expect("mesure avant");
    let (stats_after, mods_after, resolved_after) = m.after.clone().expect("mesure après");
    let old = stats_before.get(&StatId::MoveSpeed).unwrap();
    assert_ne!(old, Fixed::from_num(EDITED));
    assert_eq!(
        stats_after.get(&StatId::MoveSpeed),
        Some(Fixed::from_num(EDITED))
    );
    assert_eq!(mods_before.0.len(), 1);
    assert_eq!(mods_after, mods_before, "Modifiers inchangé");
    assert_eq!(resolved_before, old * Fixed::from_num(0.5));
    assert_eq!(
        resolved_after,
        Fixed::from_num(EDITED) * Fixed::from_num(0.5),
        "le Mul 0.5 survit : vitesse résolue = base × 0,5"
    );
}

/// Diagnostic (ignoré) : rejoue par le runner une session sauvée par `alacod remote save` et imprime
/// l'état de fin (frame, joueurs, ennemis : net id, clé, position), à comparer à `alacod remote state` :
/// `ALACOD_K52_SESSION=/tmp/s.ron cargo test -p scenario --test game_commands rejoue_une_session_sauvee -- --ignored --nocapture`.
#[test]
#[ignore]
fn rejoue_une_session_sauvee() {
    use game::character::enemy::Enemy;
    use game::replay::Scenario;
    let path = std::env::var("ALACOD_K52_SESSION").expect("ALACOD_K52_SESSION=<scénario.ron>");
    let mut scenario = Scenario::from_ron(&std::fs::read_to_string(path).unwrap()).unwrap();
    scenario.expect.clear();
    println!("placements : {:?}", scenario.characters);
    let frames = scenario.frames;
    let first = scenario::run(&scenario);
    let second = scenario::run(&scenario);
    assert_eq!(first.trace, second.trace, "deux rejeux : même trace");
    println!(
        "trace : {} lignes, dernière : {:?}",
        first.trace.len(),
        first.trace.last()
    );
    let mut app = scenario::runner::run_until(&scenario, frames);
    let world = app.world_mut();
    let frame = world.resource::<FrameCount>().frame;
    let mut rows: Vec<_> = world
        .query::<(
            &GgrsNetId,
            &CharacterConfigKey,
            &bevy_fixed::fixed_math::FixedTransform3D,
            Option<&Enemy>,
        )>()
        .iter(world)
        .map(|(id, key, t, enemy)| {
            (
                id.0,
                key.0.clone(),
                enemy.is_some(),
                t.translation.x.to_num::<f32>(),
                t.translation.y.to_num::<f32>(),
            )
        })
        .collect();
    rows.sort_by_key(|r| r.0);
    println!("frame {frame}");
    for (id, key, enemy, x, y) in rows {
        println!(
            "  {} {id} {key} ({x:.1}, {y:.1})",
            if enemy { "ennemi" } else { "joueur" }
        );
    }
}
