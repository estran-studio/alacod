//! T2.4, chantier F1 : preuve de la relance sans relancer le binaire. Joue `idle` en
//! headless jusqu'à la défaite (ou 1500 frames), pose `RunRequest::Restart`, mesure le
//! temps mural jusqu'au retour en `InGame` à la frame 0, rejoue 300 frames et vérifie que
//! la trace d'état de ces 300 premières frames est identique à celle de la première partie
//! (même graine → même simulation : c'est la preuve que la relance ne laisse rien
//! traîner). `ToLobby` : vérifié au moins pour le changement d'état (voir la tâche).

use std::time::Instant;

use bevy::prelude::*;
use game::{core::AppState, run_state::RunRequest, state_trace::StateTraceRecorder};
use run::Run;
use scenario::{
    runner::{build_app, PlayConfig},
    Scenario,
};
use utils::frame::FrameCount;

fn idle() -> Scenario {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/scenarios/idle.ron"
    );
    let source = std::fs::read_to_string(path).expect("idle.ron");
    Scenario::from_ron(&source).expect("scénario valide")
}

/// Une app headless prête à jouer `idle` (même chemin que `runner::run_with_options`, sans
/// son évaluation d'attentes : ce test pilote la boucle `app.update()` lui-même pour poser
/// `RunRequest` entre deux parties).
fn idle_app() -> App {
    let scenario = idle();
    let config = PlayConfig {
        follow_handle: None,
    };
    let mut app = build_app(&scenario, true, &config);
    app.finish();
    app.cleanup();
    app
}

fn frame(app: &App) -> u32 {
    app.world().resource::<FrameCount>().frame
}

fn app_state(app: &App) -> AppState {
    app.world().resource::<State<AppState>>().get().clone()
}

/// Avance `app` jusqu'à ce que `Run.step` devienne `Ended` (défaite T1.3) ou que
/// `max_frame` soit atteinte, au premier des deux. Budget d'updates généreux : le
/// chargement de la map (avant la première frame simulée) prend un nombre d'updates
/// variable, pas compté dans `max_frame`.
fn run_until_defeat_or(app: &mut App, max_frame: u32) -> u32 {
    let budget = 20_000 + max_frame;
    let mut last_frame = 0;
    for _ in 0..budget {
        app.update();
        last_frame = frame(app);
        if last_frame > 0 {
            if let Some(run) = app.world().get_resource::<Run>() {
                if run.is_ended() {
                    break;
                }
            }
        }
        if last_frame >= max_frame {
            break;
        }
    }
    last_frame
}

/// Avance `app` jusqu'à ce qu'il soit de retour `InGame` à la frame 0 (une nouvelle partie
/// a démarré), ou que `budget` updates se soient écoulées sans y parvenir (échec). Retourne
/// le temps mural écoulé depuis l'appel.
fn wait_for_fresh_in_game(app: &mut App, budget: u32) -> std::time::Duration {
    let start = Instant::now();
    for _ in 0..budget {
        app.update();
        if app_state(app) == AppState::InGame && frame(app) == 0 {
            return start.elapsed();
        }
    }
    panic!(
        "pas revenu en InGame à la frame 0 après {budget} updates (état={:?}, frame={})",
        app_state(app),
        frame(app)
    );
}

#[test]
fn restart_replays_identically_under_ten_seconds() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!(
            "test ignoré : compilé avec le rendu des tilemaps (utiliser --no-default-features)"
        );
        return;
    }

    let mut app = idle_app();

    // Première partie : jusqu'à la défaite (idle.ron : un joueur immobile, mort constatée
    // f1052 d'après le commentaire du scénario) ou 1500 frames.
    let defeat_frame = run_until_defeat_or(&mut app, 1500);
    assert!(
        app.world()
            .get_resource::<Run>()
            .is_some_and(Run::is_ended),
        "idle.ron n'a pas atteint la défaite en {defeat_frame} frames (voir sa doc : mort attendue vers f1052)"
    );
    assert!(
        defeat_frame >= 300,
        "défaite trop tôt ({defeat_frame} < 300) pour comparer 300 frames : ajuster le scénario ou ce test"
    );

    // Trace des 300 premières frames de la première partie, capturée avant la relance :
    // `StateTraceRecorder::frames` est une `BTreeMap<frame, ligne>` partagée par toute la
    // vie de l'app — la deuxième partie écrasera les mêmes clés (0..300) avec ses propres
    // lignes, il faut donc une copie possédée ici.
    let first_run_trace: Vec<String> = app
        .world()
        .resource::<StateTraceRecorder>()
        .lines_until(300)
        .map(str::to_string)
        .collect();
    assert_eq!(
        first_run_trace.len(),
        300,
        "trace de la première partie incomplète avant la relance"
    );

    // Relance (T2.4) : pose la commande, mesure le temps mural jusqu'au retour en InGame à
    // la frame 0 — doit rester sous dix secondes (acceptation de la tâche).
    app.world_mut().insert_resource(RunRequest::Restart);
    let elapsed = wait_for_fresh_in_game(&mut app, 20_000);
    assert!(
        elapsed.as_secs_f64() < 10.0,
        "relance trop lente : {elapsed:?} (attendu < 10 s)"
    );

    // Rejoue 300 frames de la deuxième partie et compare : même graine, même carte, mêmes
    // joueurs (RunRequest::Restart ne les change pas, voir game::run_state) -> la
    // simulation doit être bit-à-bit identique, preuve que la relance n'a rien laissé
    // traîner (entité, ressource rollback, session GGRS) de la première partie.
    for _ in 0..300 {
        app.update();
    }
    assert_eq!(
        frame(&app),
        300,
        "la deuxième partie n'a pas atteint la frame 300"
    );
    let second_run_trace: Vec<String> = app
        .world()
        .resource::<StateTraceRecorder>()
        .lines_until(300)
        .map(str::to_string)
        .collect();

    let mut first_divergence = None;
    for (i, (a, b)) in first_run_trace
        .iter()
        .zip(second_run_trace.iter())
        .enumerate()
    {
        if a != b {
            first_divergence = Some((i, a.clone(), b.clone()));
            break;
        }
    }
    assert!(
        first_divergence.is_none(),
        "trace divergente après relance à la frame {} :\n  partie 1 : {}\n  partie 2 : {}",
        first_divergence.as_ref().unwrap().0,
        first_divergence.as_ref().unwrap().1,
        first_divergence.as_ref().unwrap().2,
    );
    assert_eq!(
        first_run_trace, second_run_trace,
        "traces de longueurs différentes (300 lignes attendues des deux côtés)"
    );
}

#[test]
fn to_lobby_changes_app_state() {
    if map_ldtk::RENDER_ENABLED {
        eprintln!(
            "test ignoré : compilé avec le rendu des tilemaps (utiliser --no-default-features)"
        );
        return;
    }

    let mut app = idle_app();

    // Pas besoin d'atteindre la défaite : entre en InGame, avance quelques frames pour
    // avoir une partie bien établie, puis demande le retour au lobby.
    for _ in 0..600 {
        app.update();
        if app_state(&app) == AppState::InGame && frame(&app) >= 60 {
            break;
        }
    }
    assert_eq!(
        app_state(&app),
        AppState::InGame,
        "la partie n'a pas démarré (map pas chargée ?)"
    );

    app.world_mut().insert_resource(RunRequest::ToLobby);

    // Première frame où l'app quitte InGame : un scénario synctest local reste
    // `OnlineState::Offline`, donc la cible est `LobbyLocal` (voir
    // `game::run_state::apply_run_request_system`).
    let mut left_ingame_state = None;
    for _ in 0..100 {
        app.update();
        if app_state(&app) != AppState::InGame {
            left_ingame_state = Some(app_state(&app));
            break;
        }
    }
    assert_eq!(
        left_ingame_state,
        Some(AppState::LobbyLocal),
        "RunRequest::ToLobby n'a pas fait quitter InGame vers LobbyLocal"
    );
}
