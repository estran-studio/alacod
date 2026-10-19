//! Lecture du clavier et de la manette pour l'écran de mutation (D69 E5a).
//!
//! Adaptateur d'entrée (application) : le modèle de vue (`ui::mutation_screen`, aussi en
//! headless) ne lit que [`MutationNavInput`], écrite ici avant lui.

use bevy::prelude::*;
use combat::actors::{INPUT_CHOICE_A, INPUT_CHOICE_B, INPUT_CHOICE_C};
use leafwing_input_manager::prelude::ActionState;

use super::hud_model::CameraFollowOverride;
use super::mutation_screen::{update_mutation_screen_view, MutationNavInput};
use crate::character::control::PlayerAction;
use crate::character::player::{LocalPlayer, Player};
use crate::session::AppState;

/// Entrées de navigation de l'écran de mutation ; ajouté après le modèle (`ui::GameUiInputPlugin`).
pub struct MutationInputPlugin;

impl Plugin for MutationInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            read_mutation_nav
                .before(update_mutation_screen_view)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Boutons `ChoiceA/B/C` tenus et ←/→ pressés par le joueur affiché (même règle de choix que
/// `update_mutation_screen_view` : `CameraFollowOverride`, sinon le joueur local de plus petit
/// handle).
fn read_mutation_nav(
    follow_override: Option<Res<CameraFollowOverride>>,
    players: Query<(
        &Player,
        Has<LocalPlayer>,
        Option<&ActionState<PlayerAction>>,
    )>,
    mut nav: ResMut<MutationNavInput>,
) {
    let shown = match follow_override.as_deref() {
        Some(o) => players.iter().find(|(p, ..)| p.handle == o.0),
        None => players
            .iter()
            .filter(|(_, local, ..)| *local)
            .min_by_key(|(p, ..)| p.handle),
    };
    let mut next = MutationNavInput::default();
    if let Some((player, _, Some(actions))) = shown {
        next.handle = player.handle;
        for (action, bit) in [
            (PlayerAction::ChoiceA, INPUT_CHOICE_A),
            (PlayerAction::ChoiceB, INPUT_CHOICE_B),
            (PlayerAction::ChoiceC, INPUT_CHOICE_C),
        ] {
            if actions.pressed(&action) {
                next.held |= bit;
            }
        }
        if actions.just_pressed(&PlayerAction::ChoicePrev) {
            next.step -= 1;
        }
        if actions.just_pressed(&PlayerAction::ChoiceNext) {
            next.step += 1;
        }
    }
    if *nav != next {
        *nav = next;
    }
}
