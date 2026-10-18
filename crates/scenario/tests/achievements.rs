//! M3-T10 (`docs/conventions.md` §75) : deux runs enchaînées dans le même processus. La première
//! (`achievement_unlocks`, isaac, graine 8) bat le gardien et gagne : à sa fin le profil du joueur 0
//! reçoit `gardien_vaincu` et `run_finie` et leurs déblocages. La relance (`RunRequest::Restart`, le
//! chemin de « Rejouer ») relit le profil : `RunUnlocks` contient ces déblocages, et le pool de
//! butin leur doit les objets qu'il cachait. Le runner de scénarios ne suit pas une relance de
//! session : ce test pilote lui-même la boucle, comme `tests/run.rs` et `tests/hub.rs`.

use bevy::prelude::*;
use game::{run_state::RunRequest, session::AppState};
use items::{ItemTable, LootTables, PoolState, Rarity, RunUnlocks, UnlockTable};
use run::Run;
use scenario::{
    runner::{build_app, PlayConfig},
    Scenario,
};
use std::collections::{BTreeMap, BTreeSet};
use utils::frame::FrameCount;

fn scenario() -> Scenario {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/achievement_unlocks.ron"
    );
    Scenario::from_ron(&std::fs::read_to_string(path).expect("achievement_unlocks.ron"))
        .expect("scénario valide")
}

fn frame(app: &App) -> u32 {
    app.world().resource::<FrameCount>().frame
}

/// L'objet que `PoolState::assign` (le chemin du jeu : tirage à la génération de l'étage, option A de §64)
/// donne pour chacun de 400 emplacements du pool `pool` à l'étage 0 de la partie de graine `run_seed`,
/// sous les déblocages `active`. Un `PoolState` neuf par emplacement : c'est le premier tirage du pool.
fn assigned(
    app: &App,
    pool: &str,
    run_seed: u32,
    active: &RunUnlocks,
) -> BTreeMap<u32, Option<String>> {
    let world = app.world();
    let tables = world.resource::<LootTables>();
    let items = world.resource::<ItemTable>();
    let unlocks = world.resource::<UnlockTable>();
    let table = items::unlocked_table(pool, &tables.0[pool], unlocks, active);
    let hidden = unlocks.hidden_items(active);
    (0..400u32)
        .map(|slot| {
            let item = PoolState::default().assign(
                pool,
                &table,
                items,
                Rarity::Common,
                &hidden,
                run_seed,
                0,
                &format!("slot{slot}"),
            );
            (slot, item)
        })
        .collect()
}

fn items_of(draws: &BTreeMap<u32, Option<String>>) -> BTreeSet<String> {
    draws.values().flatten().cloned().collect()
}

#[test]
fn les_accomplissements_d_une_run_entrent_dans_le_pool_de_la_suivante() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("test ignoré : compilé avec le rendu des tilemaps (--no-default-features)");
        return;
    }
    let scenario = scenario();
    let mut app = build_app(
        &scenario,
        true,
        &PlayConfig {
            follow_handle: None,
        },
    );
    app.finish();
    app.cleanup();

    // Première run : profil vide, aucun déblocage, le gardien est caché des pools.
    for _ in 0..20_000 + scenario.frames {
        app.update();
        if frame(&app) >= scenario.frames {
            break;
        }
    }
    assert!(
        app.world().resource::<Run>().is_ended(),
        "la première run n'est pas finie à f{}",
        frame(&app)
    );
    let none = RunUnlocks::default();
    let run_seed = app.world().resource::<Run>().seed;
    let boss_base = assigned(&app, "boss", run_seed, &none);
    let treasure_base = assigned(&app, "tresor", run_seed, &none);
    let (before_boss, before_treasure) = (items_of(&boss_base), items_of(&treasure_base));
    for hidden in ["medaille"] {
        assert!(
            !before_boss.contains(hidden),
            "{hidden} dans le pool de base"
        );
    }
    for hidden in ["couronne", "bague"] {
        assert!(
            !before_treasure.contains(hidden),
            "{hidden} dans le pool de base"
        );
    }

    // Profil écrit en fin de run (hors simulation).
    let profile = app
        .world()
        .resource::<meta::Profiles>()
        .0
        .get(&meta::local_id(0))
        .cloned()
        .expect("profil du joueur 0");
    assert_eq!(
        profile
            .achievements
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["gardien_vaincu", "run_finie"],
        "{profile:?}"
    );
    assert!(profile.has_unlock("medaille_libre") && profile.has_unlock("couronne_libre"));
    assert!(!profile.has_unlock("bague_libre"));

    // Run suivante : seul chemin d'entrée, `RunUnlocks` (les jalons du profil n'y sont pas).
    app.world_mut().insert_resource(RunRequest::Restart);
    let mut fresh = false;
    let mut left = false;
    for _ in 0..20_000 {
        app.update();
        let state = app.world().resource::<State<AppState>>().get().clone();
        if state != AppState::InGame {
            left = true;
        } else if left {
            fresh = true;
            break;
        }
    }
    assert!(fresh, "la relance n'a jamais ramené InGame");
    let active = app.world().resource::<RunUnlocks>().clone();
    assert_eq!(
        active.0.iter().map(String::as_str).collect::<Vec<_>>(),
        ["couronne_libre", "medaille_libre"],
        "{active:?}"
    );
    assert!(
        app.world()
            .resource::<game::profile::RunHistory>()
            .bosses_defeated
            .is_empty(),
        "l'historique de la run précédente survit à la relance"
    );

    // À graine égale (celle de la seconde run, qui est celle de la première) et mêmes emplacements : seul l'objet
    // change, et seulement vers un objet que le profil vient de débloquer.
    assert_eq!(app.world().resource::<Run>().seed, run_seed);
    let boss_now = assigned(&app, "boss", run_seed, &active);
    let treasure_now = assigned(&app, "tresor", run_seed, &active);
    let (after_boss, after_treasure) = (items_of(&boss_now), items_of(&treasure_now));
    assert!(after_boss.contains("medaille"), "{after_boss:?}");
    assert!(after_treasure.contains("couronne"), "{after_treasure:?}");
    assert!(
        !after_treasure.contains("bague"),
        "{after_treasure:?} : `second_etage` n'est pas obtenu"
    );
    for (base, now, unlocked) in [
        (&boss_base, &boss_now, ["medaille"].as_slice()),
        (&treasure_base, &treasure_now, ["couronne"].as_slice()),
    ] {
        let changed: Vec<_> = base
            .iter()
            .filter(|(slot, item)| now[slot] != **item)
            .collect();
        assert!(
            !changed.is_empty(),
            "aucun emplacement ne change : déblocage sans effet"
        );
        for (slot, _) in changed {
            let now = now[slot].as_deref().unwrap();
            assert!(
                unlocked.contains(&now),
                "emplacement {slot} : {now} n'est pas un objet débloqué"
            );
        }
    }
}
