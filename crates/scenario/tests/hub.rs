//! M2-T13 (`docs/conventions.md` §57) : la porte du hub décide dans la simulation, la run démarre
//! une fois la frame de sortie confirmée. Le runner de scénarios ne suit pas une relance de
//! session (sa boucle compte les frames d'une seule session) : ce test pilote lui-même la boucle,
//! comme `tests/run.rs` pour le restart. Preuve en deux parties avec `hub_use_stations` (état
//! `HubExit` dans la simulation) et `hub_buy_unlock` (achat hors simulation).

use bevy::prelude::*;
use game::{
    core::AppState,
    hub::{HubSession, HubState},
    run_state::RunRequest,
};
use run::{Run, RunMode};
use scenario::{
    runner::{build_app, PlayConfig},
    Scenario,
};
use utils::frame::FrameCount;

/// Hub de gungeon, un joueur qui possède « tresor_trefle » (acheté avant la partie) et marche
/// jusqu'à la porte.
fn scenario() -> Scenario {
    Scenario::from_ron(
        r#"Scenario(
    powerup_drop_chance_override: "0.0",
    game: "gungeon",
    map: "maps/etage_1.ldtk",
    floors: Some("etage_1"),
    hub: Some((stay: false)),
    frames: 600,
    profile: true,
    profile_setup: Some((
        profiles: [(handle: 0, unlocks: ["first_run"], essence: 20)],
        purchases: [(handle: 0, unlock: "tresor_trefle")],
    )),
    players: [(inputs: [
        (from: 10, to: 80, buttons: [Right]),
        (from: 80, to: 93, buttons: [Up]),
        (from: 100, to: 102, buttons: [Interaction]),
    ])],
)"#,
    )
    .expect("scénario valide")
}

fn frame(app: &App) -> u32 {
    app.world().resource::<FrameCount>().frame
}

fn app_state(app: &App) -> AppState {
    app.world().resource::<State<AppState>>().get().clone()
}

#[test]
fn la_porte_du_hub_lance_la_run_avec_les_deblocages_du_profil() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!("test ignoré : compilé avec le rendu des tilemaps (--no-default-features)");
        return;
    }
    let scenario = scenario();
    let config = PlayConfig {
        follow_handle: None,
    };
    let mut app = build_app(&scenario, true, &config);
    app.finish();
    app.cleanup();

    // Au hub : mode Sandbox, aucun déblocage dans la partie (profil lu à la session, ici déjà
    // acheté : le hub, lui, ne change rien à RunUnlocks tant que la porte n'est pas franchie).
    let mut exit_frame = None;
    for _ in 0..20_000 {
        app.update();
        if let Some(state) = app.world().get_resource::<HubState>() {
            if let Some(exit) = state.exit {
                exit_frame = Some(exit.frame);
                break;
            }
        }
    }
    let exit_frame = exit_frame.expect("la porte n'a jamais été franchie");
    assert!(
        app.world().resource::<HubSession>().in_hub,
        "la run ne doit pas démarrer avant la confirmation de la frame {exit_frame}"
    );
    assert_eq!(app.world().resource::<Run>().mode, RunMode::Sandbox);

    // La frame de sortie se confirme, `hub_exit_trigger` pose la requête, la session repart.
    let mut left_in_game = false;
    for _ in 0..2_000 {
        app.update();
        if app_state(&app) != AppState::InGame {
            left_in_game = true;
            break;
        }
    }
    assert!(left_in_game, "InGame jamais quitté après la sortie du hub");
    assert!(!app.world().resource::<HubSession>().in_hub);
    assert!(app.world().get_resource::<RunRequest>().is_none());

    let mut back = false;
    for _ in 0..20_000 {
        app.update();
        if app_state(&app) == AppState::InGame && frame(&app) > 0 {
            back = true;
            break;
        }
    }
    assert!(back, "pas revenu en InGame après la sortie du hub");

    // La run : séquence `etage_1` (mode Floors), étage 0, sortie du hub oubliée, et les
    // déblocages du profil (achetés au hub) y sont désormais.
    assert_eq!(
        app.world().resource::<Run>().mode,
        RunMode::Floors {
            config: "etage_1".into()
        }
    );
    assert_eq!(app.world().resource::<run::FloorState>().index, 0);
    assert!(app.world().resource::<HubState>().exit.is_none());
    let unlocks = app.world().resource::<items::RunUnlocks>();
    assert!(
        unlocks.has("tresor_trefle"),
        "RunUnlocks de la run : {:?}",
        unlocks.0
    );
}
