//! Écran d'aide des touches, ouvert depuis la pause (« Touches ») : S1-1, `docs/conventions.md` §91.
//!
//! Le texte est **lu dans la table des liaisons** (`character::control::get_input_map`, clavier,
//! souris et manette) et dans les tables de navigation des écrans (`screens::input`) : une liaison
//! changée change l'aide, sans second endroit à tenir. Il est filtré par ce que le clone possède
//! ([`HelpContext`]) : une bombe ou une esquive que le personnage n'a pas n'y figurent pas.
//!
//! Pur (aucune ressource) : présentation seule, aucune trace ne change.

use bevy::input::gamepad::{GamepadAxis, GamepadButton};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::reflect::Reflect;
use leafwing_input_manager::prelude::*;

use super::input::{GAMEPAD_NAV, KEYBOARD_NAV};
use super::model::NavKey;
use crate::character::control::PlayerAction;

/// Ce que le clone offre au joueur ; chaque ligne d'aide qui dépend d'une capacité y est gardée.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HelpContext {
    /// Le personnage a une esquive (`dash`, ou `dash_distance` > 0).
    pub dash: bool,
    /// Le jeu a des objets actifs (`ItemKind::Active`).
    pub active: bool,
    /// Le jeu a un objet « blank » (`ItemDef::blank`).
    pub blank: bool,
    /// Le personnage a une bombe (`CharacterConfig::bomb`).
    pub bomb: bool,
    /// Le jeu a une progression à mutations (`ProgressionTable::active`).
    pub mutations: bool,
}

impl HelpContext {
    /// Toutes les lignes (tests, outillage).
    pub const ALL: Self = Self {
        dash: true,
        active: true,
        blank: true,
        bomb: true,
        mutations: true,
    };
}

/// Ce qu'une ligne suppose du clone.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Need {
    Always,
    Dash,
    Active,
    Blank,
    Bomb,
    Mutations,
}

impl Need {
    fn met(self, context: &HelpContext) -> bool {
        match self {
            Self::Always => true,
            Self::Dash => context.dash,
            Self::Active => context.active,
            Self::Blank => context.blank,
            Self::Bomb => context.bomb,
            Self::Mutations => context.mutations,
        }
    }
}

/// Une ligne de l'aide : son nom, les actions dont on lit les liaisons et ce qu'elle suppose.
struct Row {
    label: &'static str,
    actions: &'static [PlayerAction],
    need: Need,
}

use PlayerAction as A;

/// Les lignes, dans l'ordre d'affichage. `Pan` se lit dans les liaisons à deux axes (le clavier
/// n'a que la souris, qui n'est pas une liaison), `Pause` et `Inventaire` dans les tables de
/// navigation des écrans (`NavKey::Menu`, `NavKey::Inventory`).
const ROWS: &[Row] = &[
    Row {
        label: "Se déplacer",
        actions: &[A::MoveUp, A::MoveLeft, A::MoveDown, A::MoveRight],
        need: Need::Always,
    },
    Row {
        label: "Viser",
        actions: &[A::Pan],
        need: Need::Always,
    },
    Row {
        label: "Tirer",
        actions: &[A::PointerClick],
        need: Need::Always,
    },
    Row {
        label: "Recharger",
        actions: &[A::Reload],
        need: Need::Always,
    },
    Row {
        label: "Mêlée",
        actions: &[A::MeleeAttack],
        need: Need::Always,
    },
    Row {
        label: "Interagir",
        actions: &[A::Interaction],
        need: Need::Always,
    },
    Row {
        label: "Arme suivante",
        actions: &[A::SwitchWeapon],
        need: Need::Always,
    },
    Row {
        label: "Mode de l'arme",
        actions: &[A::SwitchWeaponMode],
        need: Need::Always,
    },
    Row {
        label: "Lâcher l'arme",
        actions: &[A::DropWeapon],
        need: Need::Always,
    },
    Row {
        label: "Esquive",
        actions: &[A::Dash],
        need: Need::Dash,
    },
    Row {
        label: "Esquive arrière",
        actions: &[A::Modifier],
        need: Need::Dash,
    },
    Row {
        label: "Sprint",
        actions: &[A::Sprint],
        need: Need::Always,
    },
    Row {
        label: "Objet actif",
        actions: &[A::UseActive],
        need: Need::Active,
    },
    Row {
        label: "Blank",
        actions: &[A::Blank],
        need: Need::Blank,
    },
    Row {
        label: "Bombe",
        actions: &[A::Bomb],
        need: Need::Bomb,
    },
    Row {
        label: "Mutation",
        actions: &[A::ChoicePrev, A::ChoiceNext, A::ChoiceConfirm],
        need: Need::Mutations,
    },
];

/// Nom français d'une touche ; une touche sans nom propre garde son code.
fn key_label(key: KeyCode) -> String {
    let named = match key {
        KeyCode::Space => "Espace",
        KeyCode::Enter => "Entrée",
        KeyCode::Escape => "Échap",
        KeyCode::Tab => "Tab",
        KeyCode::Backspace => "Retour",
        KeyCode::ShiftLeft => "Maj gauche",
        KeyCode::ControlLeft => "Ctrl gauche",
        KeyCode::ArrowUp => "↑",
        KeyCode::ArrowDown => "↓",
        KeyCode::ArrowLeft => "←",
        KeyCode::ArrowRight => "→",
        other => {
            let code = format!("{other:?}");
            return code
                .strip_prefix("Key")
                .or_else(|| code.strip_prefix("Digit"))
                .unwrap_or(&code)
                .to_string();
        }
    };
    named.to_string()
}

/// Nom d'un bouton de manette (disposition Xbox).
fn button_label(button: GamepadButton) -> String {
    match button {
        GamepadButton::South => "A".into(),
        GamepadButton::East => "B".into(),
        GamepadButton::West => "X".into(),
        GamepadButton::North => "Y".into(),
        GamepadButton::LeftTrigger => "LB".into(),
        GamepadButton::RightTrigger => "RB".into(),
        GamepadButton::LeftTrigger2 => "LT".into(),
        GamepadButton::RightTrigger2 => "RT".into(),
        GamepadButton::LeftThumb => "L3".into(),
        GamepadButton::RightThumb => "R3".into(),
        GamepadButton::Select => "Select".into(),
        GamepadButton::Start => "Start".into(),
        GamepadButton::DPadUp => "Croix ↑".into(),
        GamepadButton::DPadDown => "Croix ↓".into(),
        GamepadButton::DPadLeft => "Croix ←".into(),
        GamepadButton::DPadRight => "Croix →".into(),
        other => format!("{other:?}"),
    }
}

fn stick_label(axis: GamepadAxis) -> Option<String> {
    match axis {
        GamepadAxis::LeftStickX | GamepadAxis::LeftStickY => Some("Stick gauche".into()),
        GamepadAxis::RightStickX | GamepadAxis::RightStickY => Some("Stick droit".into()),
        _ => None,
    }
}

/// Étiquette d'une liaison et périphérique (`true` : manette).
fn binding_label(binding: &dyn Reflect) -> Option<(bool, String)> {
    if let Some(key) = binding.downcast_ref::<KeyCode>() {
        return Some((false, key_label(*key)));
    }
    if let Some(button) = binding.downcast_ref::<MouseButton>() {
        return Some((
            false,
            match button {
                MouseButton::Left => "Clic gauche".into(),
                MouseButton::Right => "Clic droit".into(),
                other => format!("{other:?}"),
            },
        ));
    }
    if let Some(button) = binding.downcast_ref::<GamepadButton>() {
        return Some((true, button_label(*button)));
    }
    if let Some(direction) = binding.downcast_ref::<GamepadControlDirection>() {
        return stick_label(direction.axis).map(|label| (true, label));
    }
    if let Some(stick) = binding.downcast_ref::<GamepadStick>() {
        return stick_label(stick.x).map(|label| (true, label));
    }
    None
}

/// Joint les étiquettes d'une ligne : sans doublon, et « Croix ← » « Croix → » devient « Croix ← → ».
fn join_labels(labels: Vec<String>) -> String {
    let mut joined: Vec<String> = Vec::new();
    for label in labels {
        if joined.contains(&label) {
            continue;
        }
        match (joined.last_mut(), label.strip_prefix("Croix ")) {
            (Some(last), Some(arrow)) if last.starts_with("Croix ") => {
                last.push(' ');
                last.push_str(arrow);
            }
            _ => joined.push(label),
        }
    }
    joined.join(" ")
}

/// `(clavier et souris, manette)` d'une ligne, lus dans `map`.
fn row_labels(map: &InputMap<PlayerAction>, row: &Row) -> (String, String) {
    let (mut keyboard, mut pad) = (Vec::new(), Vec::new());
    for action in row.actions {
        // Une seule liaison par périphérique et par action : la première de la table.
        let mut seen = (false, false);
        let mut take = |binding: &dyn Reflect| {
            if let Some((is_pad, label)) = binding_label(binding) {
                let slot = if is_pad { &mut seen.1 } else { &mut seen.0 };
                if !*slot {
                    *slot = true;
                    if is_pad { &mut pad } else { &mut keyboard }.push(label);
                }
            }
        };
        for binding in map.get_buttonlike(action).into_iter().flatten() {
            take(binding.as_ref());
        }
        for binding in map.get_dual_axislike(action).into_iter().flatten() {
            take(binding.as_ref());
        }
    }
    // La souris vise : ce n'est pas une liaison de la table.
    if row.actions == [A::Pan] {
        keyboard.push("Souris".into());
    }
    (join_labels(keyboard), join_labels(pad))
}

/// `(clavier, manette)` d'une touche de navigation des écrans.
fn nav_labels(key: NavKey) -> (String, String) {
    let keyboard = KEYBOARD_NAV
        .iter()
        .filter(|(_, nav)| *nav == key)
        .map(|(code, _)| key_label(*code))
        .collect();
    let pad = GAMEPAD_NAV
        .iter()
        .filter(|(_, nav)| *nav == key)
        .map(|(button, _)| button_label(*button))
        .collect();
    (join_labels(keyboard), join_labels(pad))
}

const LABEL_WIDTH: usize = 17;
const KEYBOARD_WIDTH: usize = 20;

fn line(label: &str, keyboard: &str, pad: &str) -> String {
    format!("{label:<LABEL_WIDTH$}{keyboard:<KEYBOARD_WIDTH$}{pad}",)
}

/// Le texte de l'aide : une ligne par action utile au clone, `Action  Clavier  Manette`.
pub fn help_text(map: &InputMap<PlayerAction>, context: &HelpContext) -> String {
    let mut lines = vec![line("Action", "Clavier / souris", "Manette")];
    for row in ROWS.iter().filter(|row| row.need.met(context)) {
        let (keyboard, pad) = row_labels(map, row);
        lines.push(line(row.label, &keyboard, &pad));
    }
    for (label, key) in [("Pause", NavKey::Menu), ("Inventaire", NavKey::Inventory)] {
        let (keyboard, pad) = nav_labels(key);
        lines.push(line(label, &keyboard, &pad));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::control::get_input_map;

    fn text(context: &HelpContext) -> String {
        help_text(&get_input_map(), context)
    }

    fn row<'a>(text: &'a str, label: &str) -> Option<&'a str> {
        text.lines().find(|line| line.starts_with(label))
    }

    /// Le tir à la manette est la gâchette droite, lu dans la table et pas écrit en dur.
    #[test]
    fn l_aide_lit_les_liaisons_de_la_table() {
        let all = text(&HelpContext::ALL);
        let fire = row(&all, "Tirer").unwrap();
        assert!(
            fire.contains("Clic gauche") && fire.contains("RT"),
            "{fire}"
        );
        let aim = row(&all, "Viser").unwrap();
        assert!(
            aim.contains("Souris") && aim.contains("Stick droit"),
            "{aim}"
        );
        assert!(row(&all, "Se déplacer").unwrap().contains("W A S D"));
        assert!(row(&all, "Se déplacer").unwrap().contains("Stick gauche"));

        let mut map = get_input_map();
        map.clear_action(&PlayerAction::PointerClick);
        map.insert(PlayerAction::PointerClick, GamepadButton::LeftTrigger2);
        let changed = help_text(&map, &HelpContext::ALL);
        assert!(row(&changed, "Tirer").unwrap().ends_with("LT"));
    }

    /// Une action qu'un clone n'a pas n'est pas affichée ; la pause et l'inventaire viennent de la
    /// table de navigation.
    #[test]
    fn l_aide_est_filtree_par_clone() {
        let bare = text(&HelpContext::default());
        for label in ["Bombe", "Blank", "Objet actif", "Esquive", "Mutation"] {
            assert!(row(&bare, label).is_none(), "{label} : {bare}");
        }
        assert!(row(&bare, "Tirer").is_some());
        assert!(row(&bare, "Pause").unwrap().contains("Échap"));
        assert!(row(&bare, "Pause").unwrap().contains("Start"));
        assert!(row(&bare, "Inventaire").unwrap().contains("Select"));
        let with_bomb = text(&HelpContext {
            bomb: true,
            ..Default::default()
        });
        assert!(row(&with_bomb, "Bombe").unwrap().ends_with('A'));
        assert!(row(&text(&HelpContext::ALL), "Mutation")
            .unwrap()
            .contains("Croix ← →"));
    }

    /// Chaque bouton de manette de la table apparaît dans l'aide complète : une liaison ajoutée
    /// sans ligne d'aide fait échouer ce test. Les flèches de la croix se lisent après « Croix »
    /// (« Croix ← → » regroupe deux boutons).
    #[test]
    fn chaque_bouton_de_manette_a_sa_ligne() {
        let map = get_input_map();
        let all = text(&HelpContext::ALL);
        let shown = |label: &str| match label.strip_prefix("Croix ") {
            Some(arrow) => all
                .lines()
                .filter_map(|line| line.find("Croix").map(|at| &line[at..]))
                .any(|pad| pad.contains(arrow)),
            None => all.contains(label),
        };
        for (action, binding) in map.buttonlike_bindings() {
            if let Some((true, label)) = binding_label(binding as &dyn Reflect) {
                assert!(shown(&label), "{action:?} : « {label} » absent de l'aide");
            }
        }
    }
}
