//! K5-1 (`docs/taches/k5-1-partie-pilotable.md`, `docs/conventions.md` « Éditeur par l'IA ») :
//! rechargement du `Registry` hors lobby (scrutation des fichiers, événement d'asset, relance,
//! contenu invalide) et `alacod/set_modifier` (écriture hors `GgrsSchedule`, valable à
//! `check_distance = 0`). Le runner de scénarios n'a pas de `GameRoot` : chaque test en pose une,
//! sur une copie jetable du jeu `testbed`.

use std::{path::Path, time::Duration};

use bevy::prelude::*;
use bevy_fixed::fixed_math::FixedTransform3D;
use combat::status::StatusLibrary;
use content::registry::{Registry, StatusId};
use game::{
    character::player::Player,
    content_hot_reload::GameRoot,
    editor_mode::EditorMode,
    global_asset::GlobalAsset,
    jjrs::GggrsSessionConfiguration,
    remote::{apply_set_modifier, SetModifierParams},
    run_state::RunRequest,
    session::AppState,
    weapon_assets::WeaponsConfig,
};
use scenario::{
    runner::{build_app, PlayConfig},
    Scenario,
};
use sim_core::modifier::ModifierOp;
use utils::frame::FrameCount;

fn scenario(source: &str) -> Scenario {
    Scenario::from_ron(source).expect("scénario valide")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn frame(app: &App) -> u32 {
    app.world().resource::<FrameCount>().frame
}

fn app_state(app: &App) -> AppState {
    app.world().resource::<State<AppState>>().get().clone()
}

/// Avance jusqu'à la frame de simulation `target` (budget d'updates généreux : le chargement de la
/// carte précède la frame 0).
fn run_to_frame(app: &mut App, target: u32) {
    for _ in 0..20_000 {
        app.update();
        if app_state(app) == AppState::InGame && frame(app) >= target {
            return;
        }
    }
    panic!(
        "frame {target} jamais atteinte (état {:?}, frame {})",
        app_state(app),
        frame(app)
    );
}

/// Modèle de `run.rs::wait_for_fresh_in_game` : quitte `InGame` puis y revient, frame ≤ 1.
fn wait_for_fresh_in_game(app: &mut App, budget: u32) {
    let mut left = false;
    for _ in 0..budget {
        app.update();
        if app_state(app) != AppState::InGame {
            left = true;
            break;
        }
    }
    assert!(
        left,
        "jamais quitté InGame après {budget} updates malgré RunRequest"
    );
    for _ in 0..budget {
        app.update();
        if app_state(app) == AppState::InGame {
            assert!(
                frame(app) <= 1,
                "retour en InGame à la frame {}",
                frame(app)
            );
            return;
        }
    }
    panic!("pas revenu en InGame après {budget} updates");
}

fn registry_burn(app: &App) -> i32 {
    app.world()
        .resource::<Registry>()
        .statuses
        .get(&StatusId::from("brulure"))
        .expect("statut brulure du registre")
        .damage
        .to_num::<i32>()
}

fn library_burn(app: &App) -> i32 {
    app.world()
        .resource::<StatusLibrary>()
        .statuses
        .get("brulure")
        .expect("statut brulure de StatusLibrary")
        .damage
        .to_num::<i32>()
}

fn write_burn(tmp: &Path, damage: &str) {
    std::fs::write(
        tmp.join("assets/statuses/brulure.ron"),
        format!("(kind: Burn, frames: 120, damage: \"{damage}\", period: 30)"),
    )
    .unwrap();
}

/// Étapes A à D de la fiche : scrutation, événement d'asset, relance, fichier invalide.
#[test]
fn registry_reloads_outside_lobby_and_restart_rederives_tables() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/hit_iframes.ron"
    ))
    .unwrap();
    let scenario = scenario(&source);
    let tmp = std::env::temp_dir().join(format!("alacod-editor-reload-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/testbed"),
        &tmp,
    );

    let mut app = build_app(
        &scenario,
        true,
        &PlayConfig {
            follow_handle: None,
        },
    );
    app.world_mut().insert_resource(GameRoot(tmp.clone()));
    // Mode éditeur : la scrutation n'existe qu'avec lui, et la garde (`hot_edit_allowed`) exige
    // `check_distance = 0` hors lobby (le runner joue en synctest d = 2 par défaut).
    app.world_mut().insert_resource(EditorMode);
    app.world_mut()
        .resource_mut::<GggrsSessionConfiguration>()
        .connection
        .check_distance = 0;
    app.finish();
    app.cleanup();
    run_to_frame(&mut app, 10);
    assert_eq!(registry_burn(&app), 4);
    assert_eq!(library_burn(&app), 4);

    // A : scrutation des fichiers (aucun événement d'asset : le dossier est une copie).
    write_burn(&tmp, "9");
    std::thread::sleep(Duration::from_millis(1200));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        registry_burn(&app),
        9,
        "le registre suit le fichier (scrutation)"
    );
    assert_eq!(app_state(&app), AppState::InGame, "la partie continue");
    assert_eq!(
        library_burn(&app),
        4,
        "la table dérivée n'est pas recalculée en partie"
    );

    // B : événement d'asset, sans attendre la seconde de scrutation.
    write_burn(&tmp, "11");
    {
        let handle = app.world().resource::<GlobalAsset>().weapons.clone();
        let mut assets = app.world_mut().resource_mut::<Assets<WeaponsConfig>>();
        // L'événement `Modified` n'est émis que si la garde est déréférencée en mutable.
        let mut asset = assets.get_mut(&handle).expect("asset des armes");
        let _: &mut WeaponsConfig = &mut asset;
    }
    for _ in 0..2 {
        app.update();
    }
    assert_eq!(
        registry_burn(&app),
        11,
        "le registre suit l'événement d'asset"
    );

    // C : la relance redérive les tables depuis le registre.
    app.world_mut().insert_resource(RunRequest::Restart);
    wait_for_fresh_in_game(&mut app, 20_000);
    assert_eq!(
        library_burn(&app),
        11,
        "StatusLibrary redérivée au GameLoading"
    );

    // D : fichier invalide écrit pendant que le jeu tourne (scrutation seule : aucun appel direct),
    // registre inchangé, pas de panique.
    std::fs::write(tmp.join("assets/statuses/brulure.ron"), "(((").unwrap();
    std::thread::sleep(Duration::from_millis(1200));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(registry_burn(&app), 11, "ancien registre gardé");
    assert_eq!(app_state(&app), AppState::InGame);
    let _ = std::fs::remove_dir_all(&tmp);
}

const WALK: &str = r#"Scenario(
    game: "testbed",
    map: "testbed/arena.ldtk",
    frames: 200,
    powerup_drop_chance_override: "0.0",
    players: [(inputs: [(from: 10, to: 100, buttons: [Right], pan: (0, 0))])],
    expect: [],
)"#;

fn player_x(app: &mut App) -> f32 {
    app.world_mut()
        .query::<(&Player, &FixedTransform3D)>()
        .iter(app.world())
        .find(|(player, _)| player.handle == 0)
        .map(|(_, t)| t.translation.x.to_num::<f32>())
        .expect("joueur 0")
}

fn modifier(stat: &str, value: &str, handle: usize) -> SetModifierParams {
    SetModifierParams {
        handle: Some(handle),
        net_id: None,
        stat: stat.into(),
        op: ModifierOp::Mul,
        value: value.into(),
        source: None,
        until_frames: None,
    }
}

/// Distance parcourue entre les frames 20 et 90, avec ou sans modificateur posé à la frame 20.
fn walked(with_modifier: bool) -> f32 {
    let mut app = build_app(
        &scenario(WALK),
        true,
        &PlayConfig {
            follow_handle: None,
        },
    );
    // Le runner joue en synctest d=2 : une écriture hors `GgrsSchedule` y serait perdue au
    // rechargement du snapshot. Le mode éditeur impose d=0.
    app.world_mut()
        .resource_mut::<GggrsSessionConfiguration>()
        .connection
        .check_distance = 0;
    app.finish();
    app.cleanup();
    run_to_frame(&mut app, 20);
    let start = player_x(&mut app);
    if with_modifier {
        let count = apply_set_modifier(app.world_mut(), &modifier("MoveSpeed", "0.5", 0))
            .expect("modificateur posé");
        assert!(count >= 1);
    }
    run_to_frame(&mut app, 90);
    player_x(&mut app) - start
}

#[test]
fn set_modifier_slows_the_player_and_rejects_bad_params() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let free = walked(false);
    let slowed = walked(true);
    assert!(free > 1.0, "le témoin avance : {free}");
    assert!(slowed < free, "avec MoveSpeed ×0.5 : {slowed} < {free}");

    let source = scenario(WALK);
    let mut app = build_app(
        &source,
        true,
        &PlayConfig {
            follow_handle: None,
        },
    );
    app.finish();
    app.cleanup();
    run_to_frame(&mut app, 12);
    assert!(apply_set_modifier(app.world_mut(), &modifier("Nope", "0.5", 0)).is_err());
    assert!(apply_set_modifier(app.world_mut(), &modifier("MoveSpeed", "abc", 0)).is_err());
    assert!(apply_set_modifier(app.world_mut(), &modifier("MoveSpeed", "0.5", 9)).is_err());
}

/// Hors mode éditeur, la scrutation ne tourne pas (un binaire normal ne la paie pas). La garde
/// (`check_distance`, en ligne) du remplacement automatique est couverte par `editor_mode::tests`.
#[test]
fn registry_is_not_polled_without_editor_mode() {
    if map_ldtk::RENDER_ENABLED {
        return;
    }
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/hit_iframes.ron"
    ))
    .unwrap();
    let tmp = std::env::temp_dir().join(format!("alacod-editor-guard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../games/testbed"),
        &tmp,
    );
    // Sans EditorMode, d = 0 : pas de scrutation.
    let mut app = build_app(
        &scenario(&source),
        true,
        &PlayConfig {
            follow_handle: None,
        },
    );
    app.world_mut().insert_resource(GameRoot(tmp.clone()));
    app.world_mut()
        .resource_mut::<GggrsSessionConfiguration>()
        .connection
        .check_distance = 0;
    app.finish();
    app.cleanup();
    run_to_frame(&mut app, 10);
    write_burn(&tmp, "9");
    std::thread::sleep(Duration::from_millis(1200));
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        registry_burn(&app),
        4,
        "sans mode éditeur : pas de scrutation"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}
