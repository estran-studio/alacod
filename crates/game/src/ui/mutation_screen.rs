//! Écran de mutation (T1.16, `docs/conventions.md` §30) : présentation seule, aucune trace ne
//! change.
//!
//! **Modèle de vue** ([`MutationScreenView`], ressource `Update` hors rollback, rempli aussi en
//! headless par [`MutationScreenModelPlugin`]) : dérivé en lecture seule du [`MutationChoice`]
//! du joueur affiché (celui que suit la caméra : `CameraFollowOverride`, sinon le joueur local
//! de plus petit handle, comme le HUD).
//!
//! - cartes : id, nom (`Mutation.name`) et description générée depuis les effets
//!   ([`effects::describe::describe_effects`]) ; aucun champ de contenu en plus ;
//! - `frames_left` : frames avant que la première option soit prise d'office
//!   (`since_frame + choice_frames`) ; la simulation ne s'arrête jamais ;
//! - `highlighted` : carte surlignée, état UI local. ←/→ (flèches, D-pad) la déplacent ; un
//!   bouton `ChoiceA/B/C` tenu par le joueur affiché (touches 1/2/3, ou script de scénario) la
//!   pose ; un nouveau choix la remet à 0. Valider (A / Entrée, [`PlayerAction::ChoiceConfirm`])
//!   émet le bit de la carte surlignée ([`confirm_bit`]) dans `read_local_inputs` : seul chemin
//!   vers la simulation, le même que les touches 1/2/3.
//!
//! **Rendu** : `ui::mutation_hui` (`PresentationPlugin`, feature `ui_hui`), gabarit
//! `ui/mutation_screen.html` ; trois cartes, la surlignée bordée, une barre de temps qui se vide.

use bevy::prelude::*;
use combat::actors::{INPUT_CHOICE_A, INPUT_CHOICE_B, INPUT_CHOICE_C};
use content::registry::Registry;
use leafwing_input_manager::prelude::ActionState;
use utils::frame::FrameCount;

use crate::character::player::control::PlayerAction;
use crate::character::player::input::ScriptedInputs;
use crate::character::player::{LocalPlayer, Player};
use crate::progression::{MutationChoice, ProgressionTable};
use crate::session::AppState;
use crate::ui::hud_model::CameraFollowOverride;

/// Nombre de cartes : une par bit `ChoiceA/B/C`.
pub const CARD_SLOTS: usize = 3;

/// Une carte de l'écran.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MutationCard {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Ce que l'écran affiche cette frame.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct MutationScreenView {
    pub open: bool,
    /// Handle du joueur dont le choix est affiché.
    pub handle: Option<usize>,
    pub options: Vec<MutationCard>,
    pub frames_left: u32,
    /// `choice_frames` de la progression (barre de temps : `frames_left / total_frames`).
    pub total_frames: u32,
    pub highlighted: usize,
    /// `since_frame` du choix affiché : un autre choix remet la surbrillance à 0.
    pub since_frame: u32,
}

impl MutationScreenView {
    /// Fraction de temps restante, dans `[0, 1]`.
    pub fn time_fraction(&self) -> f32 {
        if self.total_frames == 0 {
            return 0.0;
        }
        (self.frames_left as f32 / self.total_frames as f32).clamp(0.0, 1.0)
    }
}

/// Bit `ChoiceA/B/C` de la carte `index` (`None` au-delà de trois).
pub fn confirm_bit(index: usize) -> Option<u32> {
    [INPUT_CHOICE_A, INPUT_CHOICE_B, INPUT_CHOICE_C]
        .get(index)
        .copied()
}

/// Carte désignée par un bouton `ChoiceA/B/C` tenu (le plus petit si plusieurs).
pub fn held_choice(buttons: u32) -> Option<usize> {
    [INPUT_CHOICE_A, INPUT_CHOICE_B, INPUT_CHOICE_C]
        .iter()
        .position(|bit| buttons & bit != 0)
}

/// Déplace la surbrillance (`-1` gauche, `+1` droite), en boucle sur `len` cartes.
pub fn step_highlight(highlighted: usize, len: usize, step: i32) -> usize {
    if len == 0 {
        return 0;
    }
    (highlighted as i64 + step as i64).rem_euclid(len as i64) as usize
}

/// Le modèle de vue d'un choix (pur). `card` : nom et description d'une mutation par id
/// (`None` : l'id tel quel, sans description). La surbrillance de `previous` est gardée tant
/// que c'est le même choix (même joueur, même `since_frame`), remise à 0 sinon.
pub fn build_view(
    choice: Option<&MutationChoice>,
    handle: Option<usize>,
    choice_frames: u32,
    frame: u32,
    previous: &MutationScreenView,
    card: impl Fn(&str) -> Option<(String, String)>,
) -> MutationScreenView {
    let Some(choice) = choice else {
        return MutationScreenView::default();
    };
    let options: Vec<MutationCard> = choice
        .options
        .iter()
        .map(|id| {
            let (name, description) = card(id).unwrap_or_else(|| (id.clone(), String::new()));
            MutationCard {
                id: id.clone(),
                name,
                description,
            }
        })
        .collect();
    let same_choice =
        previous.open && previous.handle == handle && previous.since_frame == choice.since_frame;
    let highlighted = if same_choice {
        previous.highlighted.min(options.len().saturating_sub(1))
    } else {
        0
    };
    MutationScreenView {
        open: true,
        handle,
        frames_left: choice
            .since_frame
            .saturating_add(choice_frames)
            .saturating_sub(frame),
        total_frames: choice_frames,
        highlighted,
        since_frame: choice.since_frame,
        options,
    }
}

/// Surbrillance après les entrées de la frame (pur) : un bouton `ChoiceA/B/C` tenu l'emporte,
/// sinon ←/→.
pub fn apply_navigation(view: &mut MutationScreenView, held: Option<usize>, step: i32) {
    if !view.open {
        return;
    }
    if let Some(index) = held.filter(|index| *index < view.options.len()) {
        view.highlighted = index;
    } else if step != 0 {
        view.highlighted = step_highlight(view.highlighted, view.options.len(), step);
    }
}

/// Nom et description d'une mutation du registre.
pub fn registry_card(registry: &Registry, id: &str) -> Option<(String, String)> {
    registry
        .mutations
        .values()
        .find(|m| m.id.as_str() == id)
        .map(|m| {
            (
                m.name.clone(),
                effects::describe::describe_effects(&m.effects),
            )
        })
}

/// Bit à ajouter à l'input du joueur `handle` quand il valide (A / Entrée) : celui de la carte
/// surlignée, si l'écran affiche son choix.
pub fn confirm_input(view: Option<&MutationScreenView>, handle: usize) -> u32 {
    view.filter(|view| view.open && view.handle == Some(handle))
        .and_then(|view| confirm_bit(view.highlighted))
        .unwrap_or(0)
}

/// Modèle de vue, en headless comme à l'écran (lecture seule de la simulation).
pub struct MutationScreenModelPlugin;

impl Plugin for MutationScreenModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MutationScreenView>().add_systems(
            Update,
            update_mutation_screen_view.run_if(in_state(AppState::InGame)),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_mutation_screen_view(
    mut view: ResMut<MutationScreenView>,
    frame: Res<FrameCount>,
    table: Option<Res<ProgressionTable>>,
    registry: Option<Res<Registry>>,
    follow_override: Option<Res<CameraFollowOverride>>,
    scripted: Option<Res<ScriptedInputs>>,
    players: Query<(
        &Player,
        Has<LocalPlayer>,
        Option<&MutationChoice>,
        Option<&ActionState<PlayerAction>>,
    )>,
) {
    let shown = match follow_override.as_deref() {
        Some(o) => players.iter().find(|(p, ..)| p.handle == o.0),
        None => players
            .iter()
            .filter(|(_, local, ..)| *local)
            .min_by_key(|(p, ..)| p.handle),
    };
    let choice_frames = table
        .as_ref()
        .and_then(|t| t.active.as_ref())
        .map_or(0, |def| def.choice_frames);
    let (handle, choice, actions) = match shown {
        Some((player, _, choice, actions)) => (Some(player.handle), choice, actions),
        None => (None, None, None),
    };
    let mut next = build_view(choice, handle, choice_frames, frame.frame, &view, |id| {
        registry.as_ref().and_then(|r| registry_card(r, id))
    });
    let mut held_buttons = 0;
    let mut step = 0;
    if let Some(actions) = actions {
        for (action, bit) in [
            (PlayerAction::ChoiceA, INPUT_CHOICE_A),
            (PlayerAction::ChoiceB, INPUT_CHOICE_B),
            (PlayerAction::ChoiceC, INPUT_CHOICE_C),
        ] {
            if actions.pressed(&action) {
                held_buttons |= bit;
            }
        }
        if actions.just_pressed(&PlayerAction::ChoicePrev) {
            step -= 1;
        }
        if actions.just_pressed(&PlayerAction::ChoiceNext) {
            step += 1;
        }
    }
    if let (Some(scripted), Some(handle)) = (scripted.as_deref(), handle) {
        held_buttons |= scripted.input_at(handle, frame.frame).buttons;
    }
    apply_navigation(&mut next, held_choice(held_buttons), step);
    if *view != next {
        *view = next;
    }
}

/// Shorten wording without removing triggers, conditions, amounts or durations.
/// The full generated description remains available in the view model/tooling.
pub fn compact_description(description: &str) -> String {
    description
        .replace("À chaque ennemi tué", "Ennemi tué")
        .replace("Quand vous êtes touché", "Coup reçu")
        .replace("À chaque niveau", "Par niveau")
        .replace("sans avoir été touché depuis", "sans dégâts reçus depuis")
        .replace("si la cible est champion", "si champion")
        .replace("regagnez ", "+")
        .replace("points de vie", "PV")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_wording_keeps_effect_conditions_and_quantities() {
        assert_eq!(
            compact_description(
                "Chaque seconde, sans avoir été touché depuis 5 s : regagnez 3 points de vie."
            ),
            "Chaque seconde, sans dégâts reçus depuis 5 s : +3 PV."
        );
        assert_eq!(
            compact_description(
                "À chaque ennemi tué, si la cible est champion : dégâts × 2 pendant 5 s."
            ),
            "Ennemi tué, si champion : dégâts × 2 pendant 5 s."
        );
        assert_eq!(
            compact_description("À chaque niveau : +25 vie max, regagnez 25 points de vie."),
            "Par niveau : +25 vie max, +25 PV."
        );
    }

    fn choice(since_frame: u32) -> MutationChoice {
        MutationChoice {
            options: vec!["vampire".into(), "tireur".into(), "coriace".into()],
            since_frame,
            pending: 0,
            armed: true,
        }
    }

    fn card(id: &str) -> Option<(String, String)> {
        (id != "tireur").then(|| (format!("Nom {id}"), format!("Effet de {id}.")))
    }

    #[test]
    fn ferme_sans_choix() {
        let view = build_view(None, Some(0), 600, 50, &MutationScreenView::default(), card);
        assert_eq!(view, MutationScreenView::default());
        assert!(!view.open);
    }

    #[test]
    fn ouvert_avec_cartes_et_temps() {
        let view = build_view(
            Some(&choice(104)),
            Some(0),
            600,
            134,
            &MutationScreenView::default(),
            card,
        );
        assert!(view.open);
        assert_eq!(view.handle, Some(0));
        assert_eq!(view.frames_left, 570);
        assert_eq!(view.total_frames, 600);
        assert_eq!(view.highlighted, 0);
        assert_eq!(view.options[0].name, "Nom vampire");
        assert_eq!(view.options[0].description, "Effet de vampire.");
        assert_eq!(
            view.options[1].name, "tireur",
            "id inconnu : affiché tel quel"
        );
        assert!(view.options[1].description.is_empty());
        assert!((view.time_fraction() - 0.95).abs() < 1e-6);
        let late = build_view(Some(&choice(104)), Some(0), 600, 900, &view, card);
        assert_eq!(late.frames_left, 0);
        assert_eq!(late.time_fraction(), 0.0);
    }

    #[test]
    fn surbrillance_gardee_puis_remise_a_zero() {
        let mut view = build_view(
            Some(&choice(104)),
            Some(0),
            600,
            110,
            &MutationScreenView::default(),
            card,
        );
        apply_navigation(&mut view, None, 1);
        apply_navigation(&mut view, None, 1);
        assert_eq!(view.highlighted, 2);
        apply_navigation(&mut view, None, 1);
        assert_eq!(view.highlighted, 0, "en boucle");
        apply_navigation(&mut view, None, -1);
        assert_eq!(view.highlighted, 2);
        let same = build_view(Some(&choice(104)), Some(0), 600, 111, &view, card);
        assert_eq!(same.highlighted, 2, "même choix : gardée");
        let other = build_view(Some(&choice(400)), Some(0), 600, 401, &same, card);
        assert_eq!(other.highlighted, 0, "nouveau choix : remise à 0");
    }

    #[test]
    fn bouton_tenu_l_emporte() {
        let mut view = build_view(
            Some(&choice(104)),
            Some(0),
            600,
            134,
            &MutationScreenView::default(),
            card,
        );
        apply_navigation(&mut view, held_choice(INPUT_CHOICE_C), 1);
        assert_eq!(view.highlighted, 2);
        apply_navigation(&mut view, held_choice(INPUT_CHOICE_A | INPUT_CHOICE_B), 0);
        assert_eq!(view.highlighted, 0, "le plus petit");
        let mut closed = MutationScreenView::default();
        apply_navigation(&mut closed, Some(1), 1);
        assert_eq!(closed.highlighted, 0, "fermé : rien ne bouge");
    }

    #[test]
    fn valider_emet_le_bit_de_la_carte() {
        assert_eq!(confirm_bit(0), Some(INPUT_CHOICE_A));
        assert_eq!(confirm_bit(1), Some(INPUT_CHOICE_B));
        assert_eq!(confirm_bit(2), Some(INPUT_CHOICE_C));
        assert_eq!(confirm_bit(3), None);
        let mut view = build_view(
            Some(&choice(104)),
            Some(1),
            600,
            134,
            &MutationScreenView::default(),
            card,
        );
        apply_navigation(&mut view, None, 1);
        assert_eq!(confirm_input(Some(&view), 1), INPUT_CHOICE_B);
        assert_eq!(confirm_input(Some(&view), 0), 0, "autre joueur");
        assert_eq!(confirm_input(None, 1), 0);
        assert_eq!(confirm_input(Some(&MutationScreenView::default()), 1), 0);
    }

    #[test]
    fn mutations_des_jeux_decrites() {
        for game in ["testbed", "throne"] {
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../games")
                .join(game);
            let (registry, _, _) = content::load_and_lint(&dir).expect("registre");
            assert!(!registry.mutations.is_empty(), "{game}");
            for mutation in registry.mutations.values() {
                let (name, description) =
                    registry_card(&registry, mutation.id.as_str()).expect("carte");
                assert!(!name.is_empty());
                assert!(
                    description.ends_with('.') && !description.contains('{'),
                    "{game}/{} : « {description} »",
                    mutation.id.as_str()
                );
                println!("{game}/{} : {name} — {description}", mutation.id.as_str());
            }
        }
    }
}
