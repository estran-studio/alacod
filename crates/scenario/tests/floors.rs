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
