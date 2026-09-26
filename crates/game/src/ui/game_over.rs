use bevy::prelude::*;
use crate::character::health::PlayerDiedEvent;
use crate::core::AppState;

pub struct GameOverUiPlugin;

impl Plugin for GameOverUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_player_death.run_if(in_state(AppState::InGame)));
        app.add_systems(Update, button_system.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
struct GameOverUiRoot;

#[derive(Component)]
struct ReloadButton;

fn handle_player_death(
    mut commands: Commands,
    mut events: MessageReader<PlayerDiedEvent>,
    q_existing_ui: Query<Entity, With<GameOverUiRoot>>,
) {
    if !q_existing_ui.is_empty() {
        return;
    }

    for _event in events.read() {
        spawn_game_over_ui(&mut commands);
        break; 
    }
}

fn spawn_game_over_ui(commands: &mut Commands) {
    commands.spawn((
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
    )).with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont { font_size: FontSize::Px(60.0), ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Reload/Back Button
        parent.spawn((
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
        )).with_children(|parent| {
             parent.spawn((
                Text::new("Restart"),
                TextFont { font_size: FontSize::Px(30.0), ..default() },
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
