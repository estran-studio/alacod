//! T1.8, mode `Floors` : chargement du niveau suivant pendant la partie et continuité des net
//! ids. Joue `tests/scenarios/portal_next_floor.ron` (séquence `deux_niveaux` du testbed)
//! frame par frame jusqu'au passage du portail, et compare le monde juste avant et juste
//! après : l'ancien niveau a disparu, le nouveau est numéroté à la suite, les joueurs gardent
//! leur id, leur monnaie et leurs armes et sont placés sur les points de départ du niveau.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use bevy_ggrs::Rollback;
use game::character::{enemy::Enemy, player::Player};
use game::collider::Wall;
use game::weapons::WeaponInventory;
use run::currency::Currency;
use run::FloorState;
use scenario::{runner, Scenario};
use utils::frame::FrameCount;
use utils::net_id::GgrsNetId;

fn portal_next_floor() -> Scenario {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/portal_next_floor.ron"
    );
    let source = std::fs::read_to_string(path).expect("portal_next_floor.ron");
    Scenario::from_ron(&source).expect("scénario valide")
}

/// État lisible d'un instant de la partie.
#[derive(Debug)]
struct Snapshot {
    frame: u32,
    floor: FloorState,
    /// net id → nom, toutes les entités rollback sauf les joueurs et leurs armes.
    others: BTreeMap<usize, String>,
    /// handle → (net id, monnaie, nombre d'armes, position)
    players: BTreeMap<usize, (usize, u32, usize, (i32, i32))>,
    enemies: usize,
    walls: usize,
    intgrid_wall_cells: usize,
}

fn snapshot(app: &mut App) -> Snapshot {
    let world = app.world_mut();
    let frame = world.resource::<FrameCount>().frame;
    let floor = world.resource::<FloorState>().clone();
    let intgrid_wall_cells = world
        .resource::<game::character::enemy::ai::navigation::FlowFieldCache>()
        .intgrid_wall_cells
        .len();

    let mut players = BTreeMap::new();
    let mut player_owned = BTreeSet::new();
    let mut q = world.query::<(
        Entity,
        &GgrsNetId,
        &Player,
        &Currency,
        &WeaponInventory,
        &FixedTransform3D,
    )>();
    for (entity, net_id, player, currency, inventory, transform) in q.iter(world) {
        player_owned.insert(entity);
        for (weapon, _) in &inventory.weapons {
            player_owned.insert(*weapon);
        }
        players.insert(
            player.handle,
            (
                net_id.0,
                currency.0,
                inventory.weapons.len(),
                (
                    transform.translation.x.to_num::<i32>(),
                    transform.translation.y.to_num::<i32>(),
                ),
            ),
        );
    }
    let mut q = world.query_filtered::<(Entity, &GgrsNetId, Option<&ChildOf>), With<Rollback>>();
    let mut others = BTreeMap::new();
    for (entity, net_id, parent) in q.iter(world) {
        let owned = player_owned.contains(&entity)
            || parent.is_some_and(|p| player_owned.contains(&p.parent()));
        if !owned {
            others.insert(net_id.0, net_id.1.clone());
        }
    }
    let enemies = world
        .query_filtered::<(), With<Enemy>>()
        .iter(world)
        .count();
    let walls = world
        .query_filtered::<(), (With<Wall>, With<Rollback>)>()
        .iter(world)
        .count();
    Snapshot {
        frame,
        floor,
        others,
        players,
        enemies,
        walls,
        intgrid_wall_cells,
    }
}

/// Avance jusqu'à la première frame où `FloorState::index` vaut `index` ; rend l'état de la
/// frame précédente et de celle-ci.
fn step_until_floor(app: &mut App, index: u32, max_frames: u32) -> (Snapshot, Snapshot) {
    let mut before = snapshot(app);
    for _ in 0..max_frames * 4 {
        let previous_frame = app.world().resource::<FrameCount>().frame;
        app.update();
        if app.world().resource::<FrameCount>().frame == previous_frame {
            continue;
        }
        let now = snapshot(app);
        if now.floor.index == index {
            return (before, now);
        }
        before = now;
    }
    panic!(
        "niveau {index} jamais atteint en {max_frames} frames (dernier état : {:?})",
        before.floor
    );
}

#[test]
fn passage_au_niveau_suivant_et_continuite_des_net_ids() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let scenario = portal_next_floor();
    let mut app = runner::run_until(&scenario, 1);

    // Premier niveau : `floor_a` (un follower), portail fermé au centre des points de départ.
    let start = snapshot(&mut app);
    assert_eq!(start.floor.index, 0);
    assert!(!start.floor.portal_open);
    assert!(start.floor.anchor.is_some());
    assert_eq!(start.floor.enemies_placed, 1);
    assert_eq!(start.enemies, 1);
    assert!(start.walls > 0);

    let (before, after) = step_until_floor(&mut app, 1, scenario.frames);
    assert_eq!(after.frame, before.frame + 1);
    assert!(before.floor.portal_open, "le portail était ouvert");
    assert_eq!(before.enemies, 0, "plus d'ennemi avant le passage");

    // Le niveau quitté a disparu : aucun id d'avant ne survit hors joueurs.
    let max_before = before
        .others
        .keys()
        .chain(before.players.values().map(|p| &p.0))
        .copied()
        .max()
        .unwrap();
    assert!(
        after.others.keys().all(|id| *id > max_before),
        "ids du nouveau niveau ({:?}) pas tous après {max_before}",
        after.others
    );
    // Continuité : le nouveau niveau commence juste après le dernier id distribué.
    let first_new = *after
        .others
        .keys()
        .next()
        .expect("entités du nouveau niveau");
    assert!(first_new > max_before);
    let ids: Vec<_> = after.others.keys().copied().collect();
    assert_eq!(
        ids,
        (first_new..first_new + ids.len()).collect::<Vec<_>>(),
        "numérotation continue, sans trou"
    );

    // Nouveau niveau `floor_b` : breacher + follower, murs et cases murées rechargés,
    // portail refermé et déplacé au centre des nouveaux points de départ.
    assert_eq!(after.floor.index, 1);
    assert!(!after.floor.portal_open);
    assert_eq!(after.floor.enemies_placed, 3);
    assert_eq!(after.enemies, 2);
    assert!(after.walls > 0);
    assert!(after.intgrid_wall_cells > 0);
    assert_eq!(after.intgrid_wall_cells, start.intgrid_wall_cells);

    // Joueurs : même id, même monnaie, mêmes armes ; placés sur leur point de départ.
    for (handle, (net_id, currency, weapons, _)) in &before.players {
        let (after_id, after_currency, after_weapons, _) = after.players[handle];
        assert_eq!(after_id, *net_id);
        assert_eq!(after_currency, *currency);
        assert_eq!(after_weapons, *weapons);
    }
    assert_eq!(
        after.players[&0].3, start.players[&0].3,
        "floor_b a les mêmes points de départ que floor_a"
    );
}

/// Même graine, même séquence : deux parties identiques au bit près (le synctest vérifie déjà
/// chaque frame contre ses propres rollbacks ; ceci compare deux processus d'app distincts).
#[test]
fn deux_parties_floors_ont_la_meme_trace() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let scenario = portal_next_floor();
    let a = scenario::run(&scenario);
    let b = scenario::run(&scenario);
    assert!(a.failures.is_empty(), "{:?}", a.failures);
    assert_eq!(a.trace, b.trace);
}

/// X-6f (`docs/conventions.md` §128) : relance après un passage de niveau par la porte = retour au premier niveau, même
/// graine. Joue `sortie_porte_solo` jusqu'au niveau 1, relance la partie (`RunRequest::Restart`, comme l'écran de fin ou
/// `alacod/restart`) : le niveau redevient 0, la clé d'équipe consommée est remise à zéro, le joueur est sur son départ
/// du niveau 0 et la partie rejouée (mêmes inputs) repasse la porte à la même frame avec **la même trace d'état**.
#[test]
fn la_relance_apres_la_porte_revient_au_niveau_zero_et_rejoue_a_l_identique() {
    use game::{run_state::RunRequest, session::AppState, state_trace::StateTraceRecorder};
    use scenario::runner::{build_app, PlayConfig};

    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/sortie_porte_solo.ron"
    );
    let scenario =
        Scenario::from_ron(&std::fs::read_to_string(path).expect("sortie_porte_solo.ron"))
            .expect("scénario valide");
    let mut app = build_app(
        &scenario,
        true,
        &PlayConfig {
            follow_handle: None,
        },
    );
    app.finish();
    app.cleanup();

    let frame = |app: &App| app.world().resource::<FrameCount>().frame;
    let position = |app: &mut App| {
        let world = app.world_mut();
        let mut query = world.query::<(&Player, &FixedTransform3D)>();
        let (_, transform) = query
            .iter(world)
            .find(|(p, _)| p.handle == 0)
            .expect("joueur 0");
        (transform.translation.x, transform.translation.y)
    };

    // Première partie : jusqu'au niveau 1 (la porte est franchie vers f269), puis quelques frames de plus.
    let (_, after) = step_until_floor(&mut app, 1, 1000);
    assert_eq!(after.floor.index, 1);
    let stop = after.frame + 10;
    while frame(&app) < stop {
        app.update();
    }
    let crossing_frame = after.frame;
    let first_trace: Vec<String> = app
        .world()
        .resource::<StateTraceRecorder>()
        .lines_until(stop - 1)
        .map(str::to_string)
        .collect();

    app.world_mut().insert_resource(RunRequest::Restart);
    // Quitter `InGame`, puis y revenir avec une partie fraîche.
    let mut left = false;
    for _ in 0..20_000 {
        app.update();
        if *app.world().resource::<State<AppState>>().get() != AppState::InGame {
            left = true;
            break;
        }
    }
    assert!(left, "jamais quitté InGame après la relance");
    let mut back = false;
    for _ in 0..20_000 {
        app.update();
        if *app.world().resource::<State<AppState>>().get() == AppState::InGame {
            back = true;
            break;
        }
    }
    assert!(back, "pas revenu en InGame après la relance");

    // Partie fraîche : niveau 0, clé d'équipe remise à zéro, joueur sur le départ du niveau 0.
    assert_eq!(app.world().resource::<FloorState>().index, 0);
    assert!(
        app.world().resource::<run::TeamKeys>().0.is_empty(),
        "la clé consommée devait être remise à zéro"
    );
    let start = position(&mut app);
    assert!(
        (start.0.to_num::<f32>() - 632.0).abs() < 4.0
            && (start.1.to_num::<f32>() + 104.0).abs() < 4.0,
        "joueur au départ du niveau 0 attendu en (632, -104), vu {start:?}"
    );

    // Rejoué : même passage de niveau à la même frame, même trace.
    let (_, again) = step_until_floor(&mut app, 1, 1000);
    assert_eq!(again.frame, crossing_frame, "passage de la porte décalé");
    while frame(&app) < stop {
        app.update();
    }
    let second_trace: Vec<String> = app
        .world()
        .resource::<StateTraceRecorder>()
        .lines_until(stop - 1)
        .map(str::to_string)
        .collect();
    assert_eq!(first_trace, second_trace, "trace divergente après relance");
}

/// Relevé des salles du monde (`RoomBounds`, entiers) : la carte telle qu'elle est posée.
fn room_layout(app: &mut App) -> BTreeSet<(i32, i32, i32, i32)> {
    let world = app.world_mut();
    let mut query = world.query::<&map::game::entity::map::room::RoomBounds>();
    query
        .iter(world)
        .map(|r| {
            (
                r.position.x.to_num::<i32>(),
                r.position.y.to_num::<i32>(),
                r.size.x.to_num::<i32>(),
                r.size.y.to_num::<i32>(),
            )
        })
        .collect()
}

fn interieur_scenario(file: &str, map_seed: Option<i32>) -> Scenario {
    let path = format!(
        "{}/../../tests/scenarios/{file}",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut scenario =
        Scenario::from_ron(&std::fs::read_to_string(&path).expect(file)).expect("scénario valide");
    if let Some(seed) = map_seed {
        scenario.map_seed = seed;
    }
    scenario
}

/// X-6g (`docs/conventions.md` § 129) : la graine par défaut d'un jeu (`entry.default_seed` du manifeste du testbed, lue ici et égale à
/// la graine d'un scénario sans `map_seed`) pose **le même intérieur** à chaque relance (`RunRequest::Restart`, pas de sauvegarde) ; une graine explicite en pose un autre.
/// Intérieur seul (`interieur_seul`, la graine de la partie est celle de l'intérieur) : même relevé des salles avant et après la relance,
/// relevé différent avec `map_seed: 3`.
#[test]
fn la_graine_par_defaut_rejoue_le_meme_interieur_et_une_graine_explicite_un_autre() {
    use game::{run_state::RunRequest, session::AppState};
    use scenario::runner::{build_app, PlayConfig};

    if map_ldtk::RENDER_ENABLED {
        eprintln!("SAUTÉ : la_graine_par_defaut_rejoue… ne se joue pas en build rendu (headless seulement)");
        return;
    }
    let play = |scenario: &Scenario| {
        let mut app = build_app(
            scenario,
            true,
            &PlayConfig {
                follow_handle: None,
            },
        );
        app.finish();
        app.cleanup();
        let target = app.world().resource::<FrameCount>().frame + 12;
        while app.world().resource::<FrameCount>().frame < target {
            app.update();
        }
        app
    };
    // Graine par défaut du testbed : celle de son manifeste (`entry.default_seed`, lue par `games/testbed/src/main.rs`),
    // la même que celle d'un scénario qui n'écrit pas `map_seed` : relevé, relance, relevé.
    let manifest = content::manifest::GameManifest::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../games/testbed"
    )))
    .expect("manifeste du testbed");
    assert_eq!(
        manifest.entry.default_seed,
        Scenario::from_ron("Scenario(frames: 1, players: [])")
            .expect("scénario minimal")
            .map_seed,
        "la graine par défaut du testbed n'est plus celle d'un scénario sans map_seed"
    );
    let default_seed =
        interieur_scenario("interieur_graine_3.ron", Some(manifest.entry.default_seed));
    let mut app = play(&default_seed);
    let before = room_layout(&mut app);
    assert!(before.len() >= 8, "intérieur trop petit : {before:?}");
    app.world_mut().insert_resource(RunRequest::Restart);
    let mut left = false;
    for _ in 0..20_000 {
        app.update();
        if *app.world().resource::<State<AppState>>().get() != AppState::InGame {
            left = true;
            break;
        }
    }
    assert!(left, "jamais quitté InGame après la relance");
    let mut back = false;
    for _ in 0..20_000 {
        app.update();
        if *app.world().resource::<State<AppState>>().get() == AppState::InGame {
            back = true;
            break;
        }
    }
    assert!(back, "pas revenu en InGame après la relance");
    for _ in 0..12 {
        app.update();
    }
    let after = room_layout(&mut app);
    assert_eq!(before, after, "la relance a posé un autre intérieur");

    // Une graine explicite en pose un autre.
    let mut other = play(&interieur_scenario("interieur_graine_3.ron", None));
    assert_ne!(
        room_layout(&mut other),
        before,
        "la graine 3 pose le même intérieur que la graine par défaut"
    );
}
