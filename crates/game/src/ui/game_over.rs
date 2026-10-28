//! Écran de fin en `bevy_hui` : résumé dérivé de `Run`, commandes locales uniquement.
#![cfg(feature = "ui_hui")]

use crate::run_state::RunRequest;
use crate::session::AppState;
use bevy::prelude::*;
use bevy_hui::prelude::*;
use run::{Run, RunEnd, RunStep};

pub struct GameOverUiPlugin;

impl Plugin for GameOverUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_game_over_ui
                .before(bevy_hui::HuiSystems::Build)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, button_system.run_if(in_state(AppState::InGame)))
        .add_systems(OnExit(AppState::InGame), despawn_game_over);
    }
}

#[derive(Component)]
struct GameOverUiRoot {
    summary: String,
}

fn despawn_game_over(mut commands: Commands, roots: Query<Entity, With<GameOverUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Affiche l'écran de fin à la fin de la partie (T1.3 défaite, T2.4 victoire/`Run.step`,
/// chantier B6/F1 : tous les joueurs à terre ou morts — avant T1.3, un seul joueur mort
/// suffisait ; maintenant la partie continue tant qu'il en reste un debout, voir
/// `combat::downed::Downed`). Dérivé de l'état (`Run.step`) et non d'un événement : si un
/// rollback annule l'issue, l'écran disparaît. Remplace `RunOutcome` (T2.4, `Run` seule
/// source de vérité).
fn update_game_over_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    run: Option<Res<Run>>,
    earned: Option<Res<crate::profile::AchievementsEarned>>,
    achievements: Option<Res<meta::Achievements>>,
    mut roots: Query<(Entity, &mut GameOverUiRoot, &mut TemplateProperties)>,
) {
    let ended = run.as_deref().and_then(|run| match run.step {
        RunStep::Ended { outcome, .. } => Some(outcome),
        _ => None,
    });
    let Some(outcome) = ended else {
        for (entity, ..) in &roots {
            commands.entity(entity).despawn();
        }
        return;
    };
    let obtained = earned
        .as_deref()
        .map(|earned| earned.all_names(achievements.as_deref()))
        .unwrap_or_default();
    let summary = summary_text(outcome, run.as_deref(), &obtained);
    if let Ok((entity, mut root, mut properties)) = roots.single_mut() {
        if root.summary != summary {
            properties.set("summary", &summary);
            root.summary = summary;
            commands.trigger(CompileContextEvent { entity });
        }
    } else {
        commands.spawn((
            HtmlNode(asset_server.load("ui/game_over_screen.html")),
            TemplateProperties::default().with("summary", &summary),
            GameOverUiRoot { summary },
        ));
    }
}

/// Titre + résumé lisible (T2.4). `Run.summary` n'existe qu'à partir de la frame qui suit
/// `step` devenant `Ended` (voir `finalize_run_summary_system`, même `RollbackSystemSet`
/// mais après le système qui pose `step`) : la toute première frame affiche encore le titre
/// seul, complété dès que le résumé arrive.
///
/// M3-T10 (§75) : les accomplissements obtenus à la fin de cette run (`obtained`, noms) sont listés
/// sous le résumé ; rien sans accomplissement. Dérivé de `AchievementsEarned` (hors simulation).
fn summary_text(outcome: RunEnd, run: Option<&Run>, obtained: &[String]) -> String {
    let text = run_summary_text(outcome, run);
    if obtained.is_empty() {
        return text;
    }
    let list: Vec<String> = obtained.iter().map(|name| format!("* {name}")).collect();
    format!("{text}\n\nAccomplissements :\n{}", list.join("\n"))
}

fn run_summary_text(outcome: RunEnd, run: Option<&Run>) -> String {
    let title = match outcome {
        RunEnd::Defeat => "DÉFAITE",
        RunEnd::Victory => "VICTOIRE",
        RunEnd::Abandon => "PARTIE ABANDONNÉE",
    };
    let floors = run.is_some_and(|run| matches!(run.mode, run::RunMode::Floors { .. }));
    match run.and_then(|run| run.summary) {
        // T1.8 : en mode `Floors`, le niveau atteint (affiché à partir de 1) remplace la vague.
        Some(summary) if floors => format!(
            "{title}\n\nÉtage {}  ·  {} éliminés\n{}:{:02}",
            summary.floor_reached + 1,
            summary.kills,
            summary.frames / (crate::session::SIM_FPS as u32 * 60),
            summary.frames / crate::session::SIM_FPS as u32 % 60
        ),
        Some(summary) => format!(
            "{title}\n\nvague {} - {} kills - {} points\n{} frames",
            summary.wave_reached, summary.kills, summary.points_total, summary.frames
        ),
        None => title.to_string(),
    }
}

/// Secondes pendant lesquelles la manette n'agit pas sur l'écran de fin : A et B y valident, mais
/// ils servent aussi à jouer (bombe, mêlée) et la partie peut finir pendant qu'on les tient.
const GAMEPAD_GRACE_SECONDS: f32 = 0.8;

/// La demande posée par la manette sur l'écran de fin : A (Sud) rejoue, B (Est) retourne au salon,
/// une fois le délai passé ; A l'emporte si les deux sont pressés ensemble.
fn gamepad_request(
    replay: RunRequest,
    south_pressed: bool,
    east_pressed: bool,
    shown_for: f32,
) -> Option<RunRequest> {
    if shown_for < GAMEPAD_GRACE_SECONDS {
        return None;
    }
    if south_pressed {
        Some(replay)
    } else if east_pressed {
        Some(RunRequest::ToLobby)
    } else {
        None
    }
}

/// Bouton et touche `R` : les deux posent `RunRequest::Restart` (en p2p, redirigé vers le
/// lobby par `apply_run_request_system` — voir sa doc et celle de `RunRequest::Restart`).
/// Bouton « Lobby » (T2.12) : `RunRequest::ToLobby`. Manette : A rejoue, B va au lobby
/// ([`gamepad_request`]).
/// La touche `R` ne compte que si l'écran de fin est affiché : c'est aussi la touche de
/// rechargement, et ce système tourne pendant toute la partie (`InGame`) — avant la revue
/// M0 (R5), recharger abandonnait la partie et la relançait.
/// `ButtonInput<KeyCode>` existe aussi en headless (aucune fenêtre ne génère jamais
/// d'évènement clavier, `just_pressed` ne déclenche donc jamais : sans danger pour les
/// scénarios/tests, qui posent `RunRequest` directement).
fn button_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Option<Res<Time<Real>>>,
    mut shown_for: Local<f32>,
    screen: Query<(), With<GameOverUiRoot>>,
    interaction_query: Query<(&Interaction, &UiId), Changed<Interaction>>,
    hub: Option<Res<crate::hub::HubSession>>,
) {
    // M2-T13 : dans un jeu à hub, « Rejouer » ramène au hub (le profil de la run est déjà écrit).
    let replay = if hub.is_some() {
        RunRequest::ToHub
    } else {
        RunRequest::Restart
    };
    if screen.is_empty() {
        *shown_for = 0.0;
    } else {
        *shown_for += time.map_or(0.0, |time| time.delta_secs());
    }
    if keyboard.just_pressed(KeyCode::KeyR) && !screen.is_empty() {
        commands.insert_resource(replay);
    }
    let pressed = |button| gamepads.iter().any(|gamepad| gamepad.just_pressed(button));
    if !screen.is_empty() {
        if let Some(request) = gamepad_request(
            replay,
            pressed(GamepadButton::South),
            pressed(GamepadButton::East),
            *shown_for,
        ) {
            commands.insert_resource(request);
        }
    }

    for (interaction, id) in &interaction_query {
        if *interaction != Interaction::Pressed || screen.is_empty() {
            continue;
        }
        match id.id().as_str() {
            "restart" => commands.insert_resource(replay),
            "lobby" => commands.insert_resource(RunRequest::ToLobby),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press_r(screen_shown: bool) -> Option<RunRequest> {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, button_system);
        if screen_shown {
            app.world_mut().spawn(GameOverUiRoot {
                summary: String::new(),
            });
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        app.update();
        app.world().get_resource::<RunRequest>().copied()
    }

    /// M3-T10 : les accomplissements obtenus sont listés sous le titre ; aucun, rien de plus.
    #[test]
    fn l_ecran_de_fin_liste_les_accomplissements() {
        let none = summary_text(RunEnd::Victory, None, &[]);
        assert_eq!(none, "VICTOIRE");
        let some = summary_text(
            RunEnd::Victory,
            None,
            &["Gardien vaincu".to_string(), "Run finie".to_string()],
        );
        assert_eq!(
            some,
            "VICTOIRE\n\nAccomplissements :\n* Gardien vaincu\n* Run finie"
        );
    }

    /// Revue M0 (R5) : `R` recharge pendant la partie, ne relance que depuis l'écran de fin.
    #[test]
    fn r_ne_relance_que_sur_l_ecran_de_fin() {
        assert_eq!(press_r(false), None);
        assert_eq!(press_r(true), Some(RunRequest::Restart));
    }

    /// A rejoue, B retourne au salon, mais pas dans la seconde où l'écran apparaît : un joueur qui
    /// tient B (mêlée) ou A (bombe) quand la partie finit ne quitte pas sans le vouloir.
    #[test]
    fn la_manette_attend_avant_de_valider_la_fin() {
        let replay = RunRequest::Restart;
        assert_eq!(gamepad_request(replay, true, false, 0.2), None);
        assert_eq!(gamepad_request(replay, false, true, 0.2), None);
        assert_eq!(gamepad_request(replay, true, false, 1.0), Some(replay));
        assert_eq!(
            gamepad_request(replay, false, true, 1.0),
            Some(RunRequest::ToLobby)
        );
        assert_eq!(gamepad_request(replay, true, true, 1.0), Some(replay));
        assert_eq!(gamepad_request(replay, false, false, 1.0), None);
        assert_eq!(
            gamepad_request(RunRequest::ToHub, true, false, 1.0),
            Some(RunRequest::ToHub)
        );
    }
}
