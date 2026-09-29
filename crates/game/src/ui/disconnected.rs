use crate::core::AppState;
use crate::jjrs::GameDisconnectedEvent;
use bevy::prelude::*;

pub struct DisconnectedUiPlugin;

impl Plugin for DisconnectedUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_disconnect_event.run_if(in_state(AppState::InGame)),
        );
        app.add_systems(Update, button_system.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
struct DisconnectedUiRoot;

#[derive(Component)]
struct ReloadButton;

fn handle_disconnect_event(
    mut commands: Commands,
    mut events: MessageReader<GameDisconnectedEvent>,
    q_existing_ui: Query<Entity, With<DisconnectedUiRoot>>,
) {
    // Check if UI is already visible to avoid duplicates if multiple events fire
    if !q_existing_ui.is_empty() {
        return;
    }

    for event in events.read() {
        spawn_disconnect_ui(&mut commands, event.0.clone());
    }
}

fn spawn_disconnect_ui(commands: &mut Commands, message: String) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            // We attach this UI to the current state so it cleans up if state changes (though we likely reload)
            DisconnectedUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME DISCONNECTED"),
                TextFont {
                    font_size: FontSize::Px(50.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));

            parent.spawn((
                Text::new(message),
                TextFont {
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
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
                        Text::new("Back to Menu"),
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
                    // For native, usually we'd go back to main menu state, but that requires full cleanup.
                    // Exiting is a safe prototype behavior.
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
