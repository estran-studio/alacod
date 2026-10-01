use crate::core::AppState;
use crate::run_state::RunRequest;
use bevy::prelude::*;
use run::{Run, RunEnd, RunStep};

pub struct GameOverUiPlugin;

impl Plugin for GameOverUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_game_over_ui.run_if(in_state(AppState::InGame)),
        );
        app.add_systems(Update, button_system.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
struct GameOverUiRoot;

#[derive(Component)]
struct SummaryText;

#[derive(Component)]
struct ReloadButton;

/// Affiche l'écran de fin à la fin de la partie (T1.3 défaite, T2.4 victoire/`Run.step`,
/// chantier B6/F1 : tous les joueurs à terre ou morts — avant T1.3, un seul joueur mort
/// suffisait ; maintenant la partie continue tant qu'il en reste un debout, voir
/// `combat::downed::Downed`). Dérivé de l'état (`Run.step`) et non d'un événement : si un
/// rollback annule l'issue, l'écran disparaît. Remplace `RunOutcome` (T2.4, `Run` seule
/// source de vérité).
fn update_game_over_ui(
    mut commands: Commands,
    run: Option<Res<Run>>,
    q_existing_ui: Query<Entity, With<GameOverUiRoot>>,
    mut q_summary_text: Query<&mut Text, With<SummaryText>>,
) {
    let ended = run.as_deref().and_then(|run| match run.step {
        RunStep::Ended { outcome, .. } => Some(outcome),
        _ => None,
    });

    match (ended, q_existing_ui.single()) {
        (Some(outcome), Err(_)) => {
            let text = summary_text(outcome, run.as_deref());
            spawn_game_over_ui(&mut commands, &text);
        }
        (Some(outcome), Ok(_)) => {
            // Rollback qui confirme/précise l'issue (ex. `summary` posé une frame après
            // `step`, voir `game::run_state::finalize_run_summary_system`) : met à jour le
            // texte en place plutôt que de respawn l'UI (évite un flash).
            if let Ok(mut text) = q_summary_text.single_mut() {
                *text = Text::new(summary_text(outcome, run.as_deref()));
            }
        }
        (None, Ok(ui)) => commands.entity(ui).despawn(),
        (None, Err(_)) => {}
    }
}

/// Titre + résumé lisible (T2.4). `Run.summary` n'existe qu'à partir de la frame qui suit
/// `step` devenant `Ended` (voir `finalize_run_summary_system`, même `RollbackSystemSet`
/// mais après le système qui pose `step`) : la toute première frame affiche encore le titre
/// seul, complété dès que le résumé arrive.
fn summary_text(outcome: RunEnd, run: Option<&Run>) -> String {
    let title = match outcome {
        RunEnd::Defeat => "DÉFAITE",
        RunEnd::Victory => "VICTOIRE",
        RunEnd::Abandon => "PARTIE ABANDONNÉE",
    };
    match run.and_then(|run| run.summary) {
        Some(summary) => format!(
            "{title}\n\nvague {} - {} kills - {} points\n{} frames",
            summary.wave_reached, summary.kills, summary.points_total, summary.frames
        ),
        None => title.to_string(),
    }
}

fn spawn_game_over_ui(commands: &mut Commands, text: &str) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)), // Red tint
            GameOverUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::justify(Justify::Center),
                SummaryText,
            ));

            // Bouton de relance (T2.4, chantier F1) : pose `RunRequest::Restart`, lu par
            // `game::run_state::apply_run_request_system` (jamais de relance synchrone ici
            // — voir la doc du module `run_state`). Touche `R` : même commande, voir
            // `button_system`.
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        margin: UiRect::top(Val::Px(40.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    ReloadButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Rejouer (R)"),
                        TextFont {
                            font_size: FontSize::Px(30.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Bouton et touche `R` : les deux posent `RunRequest::Restart` (en p2p, redirigé vers le
/// lobby par `apply_run_request_system` — voir sa doc et celle de `RunRequest::Restart`).
/// `ButtonInput<KeyCode>` existe aussi en headless (aucune fenêtre ne génère jamais
/// d'évènement clavier, `just_pressed` ne déclenche donc jamais : sans danger pour les
/// scénarios/tests, qui posent `RunRequest` directement).
fn button_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ReloadButton>),
    >,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        commands.insert_resource(RunRequest::Restart);
    }

    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                commands.insert_resource(RunRequest::Restart);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
        }
    }
}
