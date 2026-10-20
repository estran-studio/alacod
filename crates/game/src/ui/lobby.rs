use crate::jjrs::GggrsSessionConfiguration;
use crate::run_state::LocalLobbyHold;
use crate::session::AppState;
use bevy::prelude::*;
use bevy_ggrs::ggrs::PlayerType;
use bevy_matchbox::prelude::MatchboxSocket;
use run::{RunEnd, RunSummary};

pub struct LobbyUiPlugin;

impl Plugin for LobbyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LobbyOnline), spawn_lobby_ui);
        app.add_systems(OnExit(AppState::LobbyOnline), despawn_lobby_ui);
        // We only update if the socket changed (players connected/disconnected)
        app.add_systems(
            Update,
            (update_lobby_ui, update_lobby_status).run_if(in_state(AppState::LobbyOnline)),
        );
        // D13 : lobby local en attente après un retour de partie (`LocalLobbyHold`).
        app.add_systems(OnEnter(AppState::LobbyLocal), spawn_local_hold_ui);
        app.add_systems(OnExit(AppState::LobbyLocal), despawn_local_hold_ui);
        app.add_systems(
            Update,
            release_local_hold.run_if(in_state(AppState::LobbyLocal)),
        );
    }
}

/// Police de l'écran d'attente : celle de l'écran de fin (la police par défaut de Bevy n'a
/// pas les accents).
const LOBBY_FONT: &str = "fonts/FiraMono-Medium.ttf";

#[derive(Component)]
struct LocalHoldUiRoot;

#[derive(Component)]
struct NewGameButton;

/// Texte de l'écran d'attente du lobby local (D13) : le résumé de la partie quittée, puis
/// comment relancer.
fn local_hold_text(summary: Option<RunSummary>) -> String {
    let last = match summary {
        Some(summary) => {
            let what = match summary.outcome {
                RunEnd::Defeat => "Défaite",
                RunEnd::Victory => "Victoire",
                RunEnd::Abandon => "Partie abandonnée",
            };
            format!(
                "{what} : vague {} - {} kills - {} points\n",
                summary.wave_reached, summary.kills, summary.points_total
            )
        }
        None => String::new(),
    };
    format!("{last}Entrée : nouvelle partie")
}

fn spawn_local_hold_ui(
    mut commands: Commands,
    hold: Option<Res<LocalLobbyHold>>,
    asset_server: Res<AssetServer>,
) {
    let Some(hold) = hold else {
        return;
    };
    let font: Handle<Font> = asset_server.load(LOBBY_FONT);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            LocalHoldUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(local_hold_text(hold.summary)),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::justify(Justify::Center),
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    NewGameButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Nouvelle partie"),
                        TextFont {
                            font: font.into(),
                            font_size: FontSize::Px(28.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn despawn_local_hold_ui(mut commands: Commands, roots: Query<Entity, With<LocalHoldUiRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

/// Entrée ou « Nouvelle partie » : retire `LocalLobbyHold`, `setup_ggrs_local` relance la
/// même configuration à la frame suivante (l'écran disparaît en quittant `LobbyLocal`).
fn release_local_hold(
    mut commands: Commands,
    hold: Option<Res<LocalLobbyHold>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
) {
    if hold.is_none() {
        return;
    }
    let key = keys.is_some_and(|keys| {
        keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)
    });
    let clicked = buttons.iter().any(|i| *i == Interaction::Pressed);
    if key || clicked {
        commands.remove_resource::<LocalLobbyHold>();
    }
}

#[derive(Component)]
struct LobbyUiRoot;

#[derive(Component)]
struct PlayerListContainer;

/// D109 : ligne d'état de la connexion des pairs (attente lente, tentative abandonnée).
#[derive(Component)]
struct LobbyStatusText;

fn despawn_lobby_ui(mut commands: Commands, q_root: Query<Entity, With<LobbyUiRoot>>) {
    for entity in q_root.iter() {
        // In Bevy 0.17, despawn() on a UI node hierarchy should handle children via the hierarchy system
        commands.entity(entity).despawn();
    }
}

fn spawn_lobby_ui(mut commands: Commands) {
    // Root node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            // StateScoped removed, managed by OnExit
            LobbyUiRoot,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Waiting for Players..."),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Player List Container
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                PlayerListContainer,
            ));

            // D109 : vide tant que la connexion se passe normalement.
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.4, 0.2)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                LobbyStatusText,
            ));
        });
}

/// D109 : texte de [`LobbyStatusText`] d'après `LobbyConnectWatch` (horloge murale).
fn update_lobby_status(
    watch: Res<crate::jjrs::p2p_socket::LobbyConnectWatch>,
    mut texts: Query<&mut Text, With<LobbyStatusText>>,
) {
    let now = bevy::platform::time::Instant::now();
    let notice = watch
        .notice
        .map(|(notice, since)| (notice, now.saturating_duration_since(since).as_secs_f32()));
    let room_full = watch.room_full.map(|full| (full.peers, full.max));
    let status =
        crate::jjrs::p2p_socket::connect_status_text(watch.pending_secs, room_full, notice)
            .unwrap_or_default();
    for mut text in &mut texts {
        if text.0 != status {
            text.0.clone_from(&status);
        }
    }
}

fn update_lobby_ui(
    mut commands: Commands,
    mut socket: Option<ResMut<MatchboxSocket>>,
    config: Res<GggrsSessionConfiguration>,
    q_container: Query<Entity, With<PlayerListContainer>>,
    q_children: Query<&Children>,
    previous: Res<crate::jjrs::p2p::PreviousSessionPeers>,
) {
    let Some(mut socket) = socket else { return };

    // Only update if the socket has changed.
    // Since MatchboxSocket update is handled in another system (wait_for_players),
    // we can check if it is changed here.
    if !socket.is_changed() {
        // If it hasn't changed, we *could* skip, but initially the UI is empty.
        // We need to check if UI is empty.
    }

    let Ok(container_entity) = q_container.single() else {
        return;
    };

    // Hack: Despawn all children and rebuild. Ideally we'd diff, but for < 4 players it's fine.
    if socket.is_changed()
        || q_children
            .get(container_entity)
            .map(|c| c.is_empty())
            .unwrap_or(true)
    {
        if let Ok(children) = q_children.get(container_entity) {
            for &child in children {
                commands.entity(child).despawn();
            }
        }

        commands.entity(container_entity).with_children(|parent| {
            // D86 : sans les pairs d'une session déjà jouée.
            let socket_players = crate::jjrs::p2p::live_players(socket.players(), &previous.0);
            let player_configs = &config.players;

            // Display each player config with their connection status
            for player_config in player_configs.iter() {
                let status_text;
                let status_color;

                if player_config.is_local {
                    // Local player - check if we have a Local player type in socket
                    let is_connected = socket_players
                        .iter()
                        .any(|p| matches!(p, PlayerType::Local));
                    if is_connected {
                        status_text = "Ready".to_string();
                        status_color = Color::srgb(0.2, 0.8, 0.2); // Green
                    } else {
                        status_text = "Connecting...".to_string();
                        status_color = Color::srgb(0.8, 0.5, 0.2); // Orange
                    }
                } else {
                    // Remote player - check if we have enough remote players connected
                    let remote_count = socket_players
                        .iter()
                        .filter(|p| matches!(p, PlayerType::Remote(_)))
                        .count();
                    let expected_remotes = player_configs.iter().filter(|p| !p.is_local).count();

                    if remote_count >= expected_remotes {
                        status_text = "Connected".to_string();
                        status_color = Color::srgb(0.2, 0.8, 0.2); // Green
                    } else {
                        status_text = "Waiting...".to_string();
                        status_color = Color::srgb(0.8, 0.5, 0.2); // Orange
                    }
                }

                parent
                    .spawn((Node {
                        margin: UiRect::all(Val::Px(5.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!("{}: ", player_config.name)),
                            TextFont {
                                font_size: FontSize::Px(24.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        row.spawn((
                            Text::new(status_text),
                            TextFont {
                                font_size: FontSize::Px(24.0),
                                ..default()
                            },
                            TextColor(status_color),
                        ));
                    });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecran_d_attente_avec_et_sans_resume() {
        assert_eq!(local_hold_text(None), "Entrée : nouvelle partie");
        let summary = RunSummary {
            wave_reached: 2,
            kills: 9,
            points_total: 1300,
            frames: 4000,
            outcome: RunEnd::Abandon,
            floor_reached: 0,
        };
        assert_eq!(
            local_hold_text(Some(summary)),
            "Partie abandonnée : vague 2 - 9 kills - 1300 points\nEntrée : nouvelle partie"
        );
    }
}
