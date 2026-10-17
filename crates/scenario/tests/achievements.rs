//! M3-T10 (`docs/conventions.md` §75) : deux runs enchaînées dans le même processus. La première
//! (`achievement_unlocks`, isaac, graine 8) bat le gardien et gagne : à sa fin le profil du joueur 0
//! reçoit `gardien_vaincu` et `run_finie` et leurs déblocages. La relance (`RunRequest::Restart`, le
//! chemin de « Rejouer ») relit le profil : `RunUnlocks` contient ces déblocages, et le pool de
//! butin leur doit les objets qu'il cachait. Le runner de scénarios ne suit pas une relance de
//! session : ce test pilote lui-même la boucle, comme `tests/run.rs` et `tests/hub.rs`.

use bevy::prelude::*;
use bevy_fixed::rng::RollbackRng;
use game::{core::AppState, run_state::RunRequest};
use items::{draw_loot_with, ItemTable, LootTables, Rarity, RunUnlocks, UnlockTable};
use run::Run;
use scenario::{
    runner::{build_app, PlayConfig},
    Scenario,
};
use std::collections::BTreeSet;
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

/// Ce que `pool` donne sur 400 graines, sous les déblocages `active`.
fn draws(app: &App, pool: &str, active: &RunUnlocks) -> BTreeSet<String> {
    let world = app.world();
    let tables = world.resource::<LootTables>();
    let items = world.resource::<ItemTable>();
    let unlocks = world.resource::<UnlockTable>();
    let table = items::unlocked_table(pool, &tables.0[pool], unlocks, active);
    let hidden = unlocks.hidden_items(active);
    (0..400u32)
        .filter_map(|seed| {
            let mut rng = RollbackRng::new(seed);
            draw_loot_with(&table, items, None, Rarity::Common, false, &hidden, &mut rng)
        })
        .collect()
}

#[test]
fn les_accomplissements_d_une_run_entrent_dans_le_pool_de_la_suivante() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("test ignoré : compilé avec le rendu des tilemaps (--no-default-features)");
        return;
    }
    let scenario = scenario();
    let mut app = build_app(&scenario, true, &PlayConfig { follow_handle: None });
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
    let before_boss = draws(&app, "boss", &none);
    let before_treasure = draws(&app, "tresor", &none);
    for hidden in ["oeil_vif"] {
        assert!(!before_boss.contains(hidden), "{hidden} dans le pool de base");
    }
    for hidden in ["lunette_longue", "trefle_chat"] {
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
        profile.achievements.iter().map(String::as_str).collect::<Vec<_>>(),
        ["gardien_vaincu", "run_finie"],
        "{profile:?}"
    );
    assert!(profile.has_unlock("oeil_vif_libre") && profile.has_unlock("lunette_longue_libre"));
    assert!(!profile.has_unlock("trefle_chat_libre"));

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
        ["lunette_longue_libre", "oeil_vif_libre"],
        "{active:?}"
    );
    assert!(
        app.world()
            .resource::<game::profile::RunHistory>()
            .bosses_defeated
            .is_empty(),
        "l'historique de la run précédente survit à la relance"
    );

    // À graine égale (mêmes 400 graines), le pool montre maintenant les objets débloqués, et pas les autres.
    let after_boss = draws(&app, "boss", &active);
    let after_treasure = draws(&app, "tresor", &active);
    assert!(after_boss.contains("oeil_vif"), "{after_boss:?}");
    assert!(after_treasure.contains("lunette_longue"), "{after_treasure:?}");
    assert!(
        !after_treasure.contains("trefle_chat"),
        "{after_treasure:?} : `second_etage` n'est pas obtenu"
    );
    assert!(before_boss.is_subset(&after_boss) && before_treasure.is_subset(&after_treasure));
}
