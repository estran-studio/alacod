use crate::core::AppState;
use bevy::prelude::*;
use combat::downed::RunOutcome;

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
struct ReloadButton;

/// Affiche le game over à la défaite (T1.3, chantier B6 : `RunOutcome.defeat_at_frame`,
/// tous les joueurs à terre ou morts — avant ce chantier, un seul joueur mort suffisait ;
/// maintenant la partie continue tant qu'il en reste un debout, voir
/// `combat::downed::RunOutcome`). Dérivé de l'état et non d'un événement : si un rollback
/// annule la défaite, l'écran disparaît.
fn update_game_over_ui(
    mut commands: Commands,
    run_outcome: Option<Res<RunOutcome>>,
    q_existing_ui: Query<Entity, With<GameOverUiRoot>>,
) {
    let game_over = run_outcome.is_some_and(|outcome| outcome.defeat_at_frame.is_some());

    match (game_over, q_existing_ui.single()) {
        (true, Err(_)) => spawn_game_over_ui(&mut commands),
        (false, Ok(ui)) => commands.entity(ui).despawn(),
        _ => {}
    }
}

fn spawn_game_over_ui(commands: &mut Commands) {
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
                Text::new("GAME OVER"),
                TextFont {
                    font_size: FontSize::Px(60.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Reload/Back Button
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
                        Text::new("Restart"),
                        TextFont {
                            font_size: FontSize::Px(30.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ReloadButton>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Reload page (for WASM) or Exit (for Native)
                #[cfg(target_arch = "wasm32")]
                {
                    let window = web_sys::window().unwrap();
                    let _ = window.location().reload();
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    std::process::exit(0);
                }
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
